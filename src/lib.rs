//! Snowflakes with six-fold symmetry, drawn as text.
//!
//! One arm is grown on a triangular lattice and then copied through the 12
//! rotations and reflections of a hexagon. The arm's shape comes from a short
//! list of "weather" values, one per step out from the centre. A terminal
//! cell is about twice as tall as it is wide, so the lattice fits the
//! character grid with very little distortion.
//!
//! Every list of weather values gives a different flake, so each flake can
//! have a `u64` ID. Up to size 24 the IDs count the flakes exactly, from 0 to
//! one less than the number of flakes. Past that there are more flakes than
//! `u64` values, so every `u64` is a valid ID but most flakes don't have one.
//!
//! ```
//! use asciiflake::{Snowflake, Style};
//!
//! assert_eq!(Snowflake::count(6), Some(144));
//! let flake = Snowflake::from_id(6, 42).unwrap();
//! println!("{flake}");                         // ASCII
//! println!("{flake:#}");                       // Unicode
//! println!("{}", flake.render(Style::UNICODE)); // same, explicitly
//! println!("{}", Snowflake::from_id(30, u64::MAX).unwrap());
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter::successors;

/// A snowflake, stored as the weather it grew through: one value per step out
/// from the centre. `weather[t]` is how far side branches that start at time
/// `t` can grow, and zero means none start then. Different weather always
/// grows a different flake, so this is all a flake needs to hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snowflake {
    weather: Vec<i32>,
}

impl Snowflake {
    /// The biggest size where every flake has its own `u64` ID.
    pub const MAX_ID_SIZE: u32 = 24;

    /// How many different flakes there are of this size, or `None` past
    /// [`MAX_ID_SIZE`](Self::MAX_ID_SIZE) where the count doesn't fit in a `u64`.
    pub fn count(size: u32) -> Option<u64> {
        (0..size).try_fold(1u64, |n, t| n.checked_mul(Self::choices(size, t)))
    }

    /// The flake with this ID. Different IDs always give different flakes. If
    /// [`count(size)`](Self::count) is `Some(n)`, IDs run from 0 to `n - 1` and
    /// this returns `None` for anything bigger. Otherwise every `u64` is a
    /// valid ID.
    ///
    /// Listing every weather in order, with the value nearest the centre
    /// changing fastest, the ID is the weather's place in that list. That's a
    /// mixed-radix number with one digit per time, where digit `t` has base
    /// `choices(size, t)`.
    ///
    /// When there are more flakes than `u64` values, using the ID directly
    /// would only ever reach the start of the list, which leaves the outermost
    /// digits stuck at zero. Instead the ID is read as the fraction `id / 2^64`
    /// of the way through the list, so IDs are spread evenly over all of it.
    /// Each step multiplies the fraction by the next base and takes the whole
    /// part as the digit, starting from the outermost. Because there are more
    /// places in the list than IDs, two IDs never land on the same one.
    pub fn from_id(size: u32, id: u64) -> Option<Self> {
        let mut weather = vec![0; size as usize];
        match Self::count(size) {
            Some(count) => {
                if id >= count {
                    return None;
                }
                let mut rest = id;
                for (t, w) in (0..size).zip(&mut weather) {
                    *w = (rest % Self::choices(size, t)) as i32;
                    rest /= Self::choices(size, t);
                }
            }
            None => {
                let mut fraction = id;
                for (t, w) in (0..size).zip(&mut weather).rev() {
                    let scaled = u128::from(fraction) * u128::from(Self::choices(size, t));
                    *w = (scaled >> 64) as i32;
                    fraction = scaled as u64;
                }
            }
        }
        Some(Snowflake { weather })
    }

    /// Draw the flake with the given glyphs.
    pub fn render(&self, style: Style) -> String {
        // Maps each character cell to a bitmask of the line directions through it.
        let mut cells = BTreeMap::<(i32, i32), u8>::new();
        for Edge(a, b) in self.edges() {
            let ((x0, y0), (x1, y1)) = (a.screen(), b.screen());
            let (sx, sy) = ((x1 - x0).signum(), (y1 - y0).signum());
            let axis = 1 << (sx * sy + 1); // ╲ = 0b100, ─ = 0b010, ╱ = 0b001
            for k in 0..=(x1 - x0).abs().max((y1 - y0).abs()) {
                *cells.entry((y0 + k * sy, x0 + k * sx)).or_default() |= axis;
            }
        }

        let reach = |f: fn(&(i32, i32)) -> i32| cells.keys().map(f).max().unwrap_or(0);
        let (h, w) = (reach(|c| c.0.abs()), reach(|c| c.1.abs()));
        (-h..=h)
            .map(|y| {
                let row: String = (-w..=w).map(|x| style.glyph(cells.get(&(y, x)))).collect();
                row.trim_end().to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// How many weather values make a difference at time `t` in a flake of
    /// this size. The longest side branch that can start at time `t` is the one
    /// on the spine, with room for `min(t, size - t)` steps, so anything longer
    /// acts the same as that. Adding 1 counts the calm value, zero.
    fn choices(size: u32, t: u32) -> u64 {
        u64::from(t.min(size - t)) + 1
    }

    /// Every edge of the flake: the twelve copies of one half-arm.
    fn edges(&self) -> BTreeSet<Edge> {
        self.arm().into_iter().flat_map(Edge::orbit).collect()
    }

    /// Half of one arm: the spine heading east and the side branches on its
    /// north side. The rest of the flake is copies of this.
    fn arm(&self) -> BTreeSet<Edge> {
        let size = self.weather.len() as i32;
        let spine = Branch {
            at: Hex(0, 0),
            dir: 0,
            turn: 1,
            born: 0,
            len: size,
            strip: size,
        };
        let mut edges = BTreeSet::new();
        self.grow(spine, &mut edges);
        edges
    }

    /// Lay down `branch` and grow its side branches. Whenever the weather is
    /// nonzero for the time a step is reached, a side branch starts there.
    ///
    /// A point's time is the same as its distance from the centre, so one weather
    /// value puts side branches on a whole hexagonal ring. Real snow crystals have
    /// the same kind of banding.
    ///
    /// A side branch is never longer than its distance to either end of its
    /// parent. That keeps it inside its half of the 60° wedge, so the copies of
    /// the arm don't overlap, and it makes branches shorter toward the tip.
    ///
    /// Side branches also split the space beside their parent into strips, one
    /// per branch, running from that branch to the next one along. A branch's
    /// own side branches can't be longer than its strip is wide. Since no
    /// branch reaches into a sibling's strip, branches never share an edge, and
    /// each side branch off the spine is drawn at exactly the length the
    /// weather gave it. That makes the weather readable back from the flake, so
    /// different weather always gives a different flake.
    fn grow(&self, branch: Branch, edges: &mut BTreeSet<Edge>) {
        // Walk from the tip back so the next side branch along is already known.
        let mut next = branch.len;
        for i in (0..branch.len).rev() {
            let p = branch.at.step(branch.dir, i);
            edges.insert(Edge::new(p, p.step(branch.dir, 1)));
            let reach = self.weather[(branch.born + i) as usize]
                .min(i)
                .min(branch.len - i)
                .min(branch.strip);
            if reach > 0 {
                self.grow(branch.sprout(i, reach, next - i), edges);
                next = i;
            }
        }
    }
}

/// Draws ASCII by default, or Unicode with the alternate flag (`{:#}`).
impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.render(if f.alternate() {
            Style::UNICODE
        } else {
            Style::ASCII
        }))
    }
}

/// The glyphs a flake is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub flat: char,
    pub rise: char,
    pub fall: char,
    /// Used where lines in different directions meet.
    pub knot: char,
}

impl Style {
    pub const ASCII: Style = Style {
        flat: '-',
        rise: '/',
        fall: '\\',
        knot: '*',
    };
    pub const UNICODE: Style = Style {
        flat: '─',
        rise: '╱',
        fall: '╲',
        knot: '•',
    };

    fn glyph(self, axes: Option<&u8>) -> char {
        match axes {
            None => ' ',
            Some(0b001) => self.rise,
            Some(0b010) => self.flat,
            Some(0b100) => self.fall,
            Some(_) => self.knot,
        }
    }
}

/// One straight branch of an arm.
#[derive(Clone, Copy)]
struct Branch {
    /// Where it starts.
    at: Hex,
    /// Which way it grows, in sixths of a turn counter-clockwise from east.
    dir: usize,
    /// How far its side branches turn from it, in sixths of a turn. Theirs
    /// turn back the other way, which keeps them parallel to this branch.
    turn: usize,
    /// The time it starts. Time goes up by one with each step.
    born: i32,
    /// How many steps long it is.
    len: i32,
    /// How wide its strip is, which caps how long its side branches can be.
    strip: i32,
}

impl Branch {
    /// The side branch that starts `i` steps along this one.
    fn sprout(&self, i: i32, len: i32, strip: i32) -> Branch {
        Branch {
            at: self.at.step(self.dir, i),
            dir: (self.dir + self.turn) % 6,
            turn: 6 - self.turn,
            born: self.born + i,
            len,
            strip,
        }
    }
}

/// A point on the triangular lattice, in axial coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hex(i32, i32);

impl Hex {
    /// Unit steps, counter-clockwise from east.
    const DIRS: [Hex; 6] = [
        Hex(1, 0),
        Hex(1, -1),
        Hex(0, -1),
        Hex(-1, 0),
        Hex(-1, 1),
        Hex(0, 1),
    ];

    fn step(self, dir: usize, n: i32) -> Hex {
        let Hex(dq, dr) = Hex::DIRS[dir];
        Hex(self.0 + n * dq, self.1 + n * dr)
    }

    /// Rotate 60° counter-clockwise about the origin.
    fn rotate(self) -> Hex {
        Hex(self.0 + self.1, -self.0)
    }

    /// Mirror across the east-west axis.
    fn reflect(self) -> Hex {
        Hex(self.0 + self.1, -self.1)
    }

    /// The character cell (column, row) for this point. One step east is two columns.
    fn screen(self) -> (i32, i32) {
        (2 * self.0 + self.1, self.1)
    }
}

/// An undirected unit edge between neighbouring lattice points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(Hex, Hex);

impl Edge {
    fn new(a: Hex, b: Hex) -> Edge {
        Edge(a.min(b), a.max(b))
    }

    fn map(self, f: fn(Hex) -> Hex) -> Edge {
        Edge::new(f(self.0), f(self.1))
    }

    /// The 12 copies of this edge under the hexagon's rotations and reflections.
    fn orbit(self) -> impl Iterator<Item = Edge> {
        successors(Some(self), |e| Some(e.map(Hex::rotate)))
            .take(6)
            .flat_map(|e| [e, e.map(Hex::reflect)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flip the art top to bottom or left to right, swapping / and \ to match.
    fn mirror(art: &str, vertical: bool) -> String {
        let width = art.lines().map(|l| l.chars().count()).max().unwrap_or(0);
        let swap = |c| match c {
            '/' => '\\',
            '\\' => '/',
            c => c,
        };
        let rows = art.lines().map(|l| format!("{l:width$}"));
        let rows: Vec<String> = if vertical {
            rows.rev().map(|l| l.chars().map(swap).collect()).collect()
        } else {
            rows.map(|l| l.chars().rev().map(swap).collect()).collect()
        };
        rows.iter()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn mirror_symmetric() {
        for id in (0..64).map(|k| k * 225) {
            let art = Snowflake::from_id(9, id).unwrap().to_string();
            assert_eq!(mirror(&art, true), art, "id {id}");
            assert_eq!(mirror(&art, false), art, "id {id}");
        }
    }

    #[test]
    fn arm_stays_in_its_wedge() {
        // The arm has to stay between east and 30 degrees north of east,
        // which means r <= 0 and q + 2r >= 0.
        for id in 0..Snowflake::count(8).unwrap() {
            let arm = Snowflake::from_id(8, id).unwrap().arm();
            for Hex(q, r) in arm.into_iter().flat_map(|Edge(a, b)| [a, b]) {
                assert!(r <= 0 && q + 2 * r >= 0, "id {id}: {:?}", Hex(q, r));
            }
        }
    }

    #[test]
    fn every_id_is_a_different_flake() {
        for size in 1..=8 {
            let count = Snowflake::count(size).unwrap();
            let flakes: BTreeSet<_> = (0..count)
                .map(|id| Snowflake::from_id(size, id).unwrap().edges())
                .collect();
            assert_eq!(flakes.len() as u64, count, "size {size}");
            assert_eq!(Snowflake::from_id(size, count), None);
        }
    }

    #[test]
    fn ids_past_max_size_cover_the_whole_flake() {
        let size = Snowflake::MAX_ID_SIZE + 1;
        assert_eq!(Snowflake::count(size - 1), Some(2_982_752_926_433_280_000));
        assert_eq!(Snowflake::count(size), None);
        // The outermost weather value still changes across the range of IDs.
        assert_eq!(Snowflake::from_id(size, 0).unwrap().weather[24], 0);
        assert_eq!(Snowflake::from_id(size, u64::MAX).unwrap().weather[24], 1);
        let ends = (0..256).chain(u64::MAX - 255..=u64::MAX);
        let flakes: BTreeSet<_> = ends
            .map(|id| Snowflake::from_id(size, id).unwrap().edges())
            .collect();
        assert_eq!(flakes.len(), 512);
    }

    #[test]
    fn group_closes() {
        let h = Hex(3, -1);
        assert_eq!(successors(Some(h), |h| Some(h.rotate())).nth(6), Some(h));
        assert_eq!(h.reflect().reflect(), h);
    }
}

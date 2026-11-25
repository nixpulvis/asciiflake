//! Rust source laid out along a snowflake's branches.
//!
//! The flake is drawn with thick strokes and the program's tokens fill them
//! in reading order, so the text itself draws the flake. Rust doesn't care
//! where whitespace goes between tokens, so the result is the same program.
//! Leftover room in a stroke goes to extra spaces between tokens, or to
//! `/***/` block comments where there's too much for that.
//!
//! A few things have to change to fit:
//!
//! - Ordinary comments are dropped, since a line comment would eat the rest of
//!   its row.
//! - Doc comments become `#[doc = ...]` attributes, which is what the compiler
//!   turns them into anyway. Long ones are cut into pieces joined by
//!   `concat!` so they can wrap across strokes.
//! - String literals that span lines are rewritten on one line with escapes.
//!
//! Tokens are never split, so one longer than its stroke is wide spills out
//! past the edge.

use crate::{Edge, Hex, Snowflake};
use std::collections::BTreeMap;

/// Lay `source` out on `flake`, with strokes reaching at least `thickness`
/// columns either side of each branch. The flake is scaled up, strokes and
/// all, to the smallest size the whole program fits in, and the code is
/// spread out over all of it.
///
/// ```
/// use asciiflake::{Snowflake, code};
///
/// let flake = Snowflake::from_id(3, 2).unwrap();
/// println!("{}", code::format(&flake, "fn main() { println!(\"hi\"); }", 3.0));
/// ```
pub fn format(flake: &Snowflake, source: &str, thickness: f64) -> String {
    // Below this scale neighbouring branches run together.
    let start = thickness + 1.0;
    let attempt = |scale: f64| {
        let runs = strokes(flake, scale, thickness * scale / start);
        // Doc comments are cut to fit a typical stroke, less one for a comma.
        let mut widths: Vec<usize> = runs.iter().map(|r| r.len).collect();
        widths.sort_unstable();
        let chunk = widths[widths.len() / 2].saturating_sub(1);
        let tokens = lex(source, chunk);
        layout(&runs, &tokens, 1.0).map(|_| (runs, tokens))
    };

    // Grow until the program fits, then close in on the smallest scale that
    // still fits, since any room past that has to be filled with something.
    let (mut small, mut big) = (start, start);
    let mut fit = attempt(big);
    while fit.is_none() {
        (small, big) = (big, big * 1.25);
        fit = attempt(big);
    }
    for _ in 0..12 {
        let middle = (small + big) / 2.0;
        match attempt(middle) {
            Some(found) => (big, fit) = (middle, Some(found)),
            None => small = middle,
        }
    }
    let (runs, tokens) = fit.unwrap();

    // Use as few of the runs as possible, so the leftover room is spread
    // out all over rather than piled up at the bottom.
    (1..=50)
        .find_map(|n| layout(&runs, &tokens, f64::from(n) / 50.0))
        .unwrap()
}

/// A horizontal stretch of a stroke: a row, its first column and its width.
#[derive(Clone, Copy)]
struct Run {
    row: usize,
    col: usize,
    len: usize,
}

/// The flake's branches drawn `radius` columns thick, as runs in reading
/// order. Each lattice step is `scale` rows tall. Distances count a row as two
/// columns, which is about how a terminal cell is shaped.
fn strokes(flake: &Snowflake, scale: f64, radius: f64) -> Vec<Run> {
    let point = |h: Hex| {
        let (x, y) = h.screen();
        (f64::from(x) * scale, f64::from(2 * y) * scale)
    };
    let mut rows = BTreeMap::<i32, Vec<i32>>::new();
    for Edge(a, b) in flake.edges() {
        let ((ax, ay), (bx, by)) = (point(a), point(b));
        let (dx, dy) = (bx - ax, by - ay);
        let top = ((ay.min(by) - radius) / 2.0).floor() as i32;
        let bottom = ((ay.max(by) + radius) / 2.0).ceil() as i32;
        let left = (ax.min(bx) - radius).floor() as i32;
        let right = (ax.max(bx) + radius).ceil() as i32;
        for row in top..=bottom {
            for col in left..=right {
                let (px, py) = (f64::from(col), f64::from(2 * row));
                let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                let (ex, ey) = (ax + t * dx - px, ay + t * dy - py);
                if ex * ex + ey * ey <= radius * radius {
                    rows.entry(row).or_default().push(col);
                }
            }
        }
    }

    let top = rows.keys().next().copied().unwrap_or(0);
    let left = rows.values().flatten().min().copied().unwrap_or(0);
    let mut runs = Vec::<Run>::new();
    for (row, mut cols) in rows {
        cols.sort_unstable();
        cols.dedup();
        let row = (row - top) as usize;
        for col in cols {
            let col = (col - left) as usize;
            match runs.last_mut() {
                Some(run) if run.row == row && run.col + run.len == col => run.len += 1,
                _ => runs.push(Run { row, col, len: 1 }),
            }
        }
    }
    runs
}

/// Fill `runs` with `tokens`, or `None` if they don't all fit. Only about
/// `share` of the room goes to code, and the rest is left as spaces between
/// tokens or comments.
fn layout(runs: &[Run], tokens: &[Token], share: f64) -> Option<String> {
    let mut grid: Vec<Vec<char>> = Vec::new();
    // The first column in each row that an overlong token hasn't taken.
    let mut free = BTreeMap::<usize, usize>::new();
    let mut next = 0;
    let mut credit = 0.0;
    for (r, run) in runs.iter().enumerate() {
        let start = run.col.max(free.get(&run.row).copied().unwrap_or(0));
        let Some(room) = (run.col + run.len).checked_sub(start).filter(|&n| n > 0) else {
            continue;
        };
        // Code gets `share` of the room so far. Leaving less than a comment's
        // four cells over isn't worth it, so then the whole run is used.
        credit += share * room as f64;
        let budget = match credit.round().clamp(0.0, room as f64) as usize {
            b if b + 4 > room => room,
            b => b,
        };

        // Take as many tokens as fit, with a space wherever one would run
        // into the one before.
        let mut taken: Vec<(usize, &Token)> = Vec::new();
        let mut used = 0;
        while let Some(token) = tokens.get(next + taken.len()) {
            let gap = taken
                .last()
                .map_or(0, |&(_, before)| usize::from(joins(before, token)));
            if used + gap + token.len() > budget {
                break;
            }
            used += gap + token.len();
            taken.push((gap, token));
        }

        let mut text = String::new();
        if let (true, Some(token)) = (
            taken.is_empty(),
            tokens.get(next).filter(|t| t.len() > room),
        ) {
            // The next token doesn't fit. Skip ahead if a wide enough run is
            // coming up soon, otherwise let it spill past the edge here.
            let soon = runs[r + 1..]
                .iter()
                .take(6)
                .any(|later| later.len >= token.len());
            if !soon {
                text.push_str(&token.text);
                next += 1;
                free.insert(run.row, start + token.len() + 1);
                used = token.len();
            }
        }
        if text.is_empty() {
            // Spread what's left between the tokens, up to two spaces more
            // each, and fill it with a comment when that's not enough.
            let spare = room - used;
            let gaps = taken.len().saturating_sub(1);
            let spread = if spare > 2 * gaps && spare >= 4 {
                text = format!("/{}/", "*".repeat(spare - 2));
                0
            } else if gaps == 0 {
                text = " ".repeat(spare);
                0
            } else {
                spare
            };
            for (i, (gap, token)) in taken.iter().enumerate() {
                let extra = match i {
                    0 => 0,
                    i => spread / gaps + usize::from(i <= spread % gaps),
                };
                text.push_str(&" ".repeat(gap + extra));
                text.push_str(&token.text);
            }
            next += taken.len();
        }
        credit -= used as f64;

        if grid.len() <= run.row {
            grid.resize(run.row + 1, Vec::new());
        }
        let line = &mut grid[run.row];
        for (i, c) in text.chars().enumerate() {
            if line.len() <= start + i {
                line.resize(start + i + 1, ' ');
            }
            line[start + i] = c;
        }
    }

    // The tips can be too narrow for a comment, which leaves them blank.
    (next == tokens.len()).then(|| {
        let lines: Vec<String> = grid
            .iter()
            .map(|line| line.iter().collect::<String>().trim_end().to_owned())
            .collect();
        lines.join("\n").trim_matches('\n').to_owned()
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// Identifiers, keywords and lifetimes.
    Word,
    /// Numbers, strings and characters.
    Literal,
    /// Operators, glued together exactly as they were in the source.
    Punct,
    /// Brackets, commas and semicolons, which never join with anything.
    Delim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    kind: Kind,
    text: String,
}

impl Token {
    fn new(kind: Kind, text: impl Into<String>) -> Token {
        Token {
            kind,
            text: text.into(),
        }
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }
}

/// Whether two tokens need a space between them. Side by side they could lex
/// as something else, like `a` `b` as `ab`, `-` `>` as `->`, or `b` `'x'` as
/// `b'x'`. And `1` `.` would be the float `1.` if a line break came after the
/// dot, as in `1.max(2)` split before `max`.
fn joins(before: &Token, after: &Token) -> bool {
    let (x, y) = (
        before.text.chars().last().unwrap(),
        after.text.chars().next().unwrap(),
    );
    ((ident_continue(x) || before.kind == Kind::Literal)
        && (ident_continue(y) || "'\"#".contains(y)))
        || (operator(x) && operator(y))
        || (x == '#' && y == '"')
        || (before.kind == Kind::Literal && (x.is_ascii_digit() || x == '_') && after.text == ".")
}

fn operator(c: char) -> bool {
    "=<>!&|+-*/%^.:@#$?~".contains(c)
}

fn ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Split Rust source into tokens, dropping comments and whitespace. Doc
/// comments come back as `#[doc]` attributes with their text cut into string
/// literals of at most `chunk` characters.
///
/// This only needs to find where tokens start and end, and erring towards
/// longer tokens is always safe, since each one is written out exactly as it
/// was. Gluing all adjacent operators together is one example of that.
fn lex(source: &str, chunk: usize) -> Vec<Token> {
    let s: Vec<char> = source.chars().collect();
    let at = |i: usize| s.get(i).copied().unwrap_or('\0');
    let text = |a: usize, b: usize| s[a..b.min(s.len())].iter().collect::<String>();
    // The end of a run of identifier characters, or of a literal's suffix.
    let word_end = |mut i: usize| {
        while ident_continue(at(i)) {
            i += 1;
        }
        i
    };
    // The end of a quoted literal whose opening quote is at `i`.
    let quoted_end = |mut i: usize| {
        let quote = s[i];
        i += 1;
        while i < s.len() && s[i] != quote {
            i += if s[i] == '\\' { 2 } else { 1 };
        }
        word_end(i + 1)
    };

    let mut tokens = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '/' && at(i + 1) == '/' {
            let end = (i..s.len()).find(|&j| s[j] == '\n').unwrap_or(s.len());
            let line = text(i, end);
            let line = line.trim_end_matches('\r');
            if let Some(body) = line.strip_prefix("//!") {
                doc(&mut tokens, true, body, chunk);
            } else if line.starts_with("///") && !line.starts_with("////") {
                doc(&mut tokens, false, &line[3..], chunk);
            }
            i = end;
        } else if c == '/' && at(i + 1) == '*' {
            let (mut end, mut depth) = (i, 0);
            while end < s.len() {
                if s[end] == '/' && at(end + 1) == '*' {
                    depth += 1;
                    end += 2;
                } else if s[end] == '*' && at(end + 1) == '/' {
                    depth -= 1;
                    end += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    end += 1;
                }
            }
            let body = text(i + 3, end.saturating_sub(2).max(i + 3));
            if at(i + 2) == '!' {
                doc(&mut tokens, true, &body, chunk);
            } else if at(i + 2) == '*' && !matches!(at(i + 3), '*' | '/') {
                doc(&mut tokens, false, &body, chunk);
            }
            i = end;
        } else if ident_start(c) {
            let end = word_end(i);
            let hashes = (end..).take_while(|&j| at(j) == '#').count();
            let (kind, end) = match (text(i, end).as_str(), at(end)) {
                ("r" | "br" | "cr", '"' | '#') if at(end + hashes) == '"' => {
                    let close = format!("\"{}", "#".repeat(hashes));
                    let body = end + hashes + 1;
                    let stop = (body..s.len())
                        .find(|&j| text(j, j + close.len()) == close)
                        .map_or(s.len(), |j| j + close.len());
                    (Kind::Literal, word_end(stop))
                }
                ("b" | "c", '"') | ("b", '\'') => (Kind::Literal, quoted_end(end)),
                ("r", '#') if ident_start(at(end + 1)) => (Kind::Word, word_end(end + 1)),
                _ => (Kind::Word, end),
            };
            tokens.push(Token::new(kind, flatten(&text(i, end))));
            i = end;
        } else if c.is_ascii_digit() {
            let radix = c == '0' && matches!(at(i + 1), 'x' | 'o' | 'b');
            let mut end = i;
            loop {
                let d = at(end);
                if ident_continue(d)
                    || (d == '.' && !radix && at(end + 1) != '.' && !ident_start(at(end + 1)))
                    || ("+-".contains(d) && !radix && "eE".contains(at(end - 1)))
                {
                    end += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token::new(Kind::Literal, text(i, end)));
            i = end;
        } else if c == '"' {
            let end = quoted_end(i);
            tokens.push(Token::new(Kind::Literal, flatten(&text(i, end))));
            i = end;
        } else if c == '\'' {
            // 'x' and '\n' are characters, 'a without a closing quote is a lifetime.
            let (kind, end) = if at(i + 1) == '\\' || at(i + 2) == '\'' {
                (Kind::Literal, quoted_end(i))
            } else if at(i + 1) == 'r' && at(i + 2) == '#' {
                (Kind::Word, word_end(i + 3))
            } else {
                (Kind::Word, word_end(i + 1))
            };
            tokens.push(Token::new(kind, text(i, end)));
            i = end;
        } else if "()[]{},;".contains(c) {
            tokens.push(Token::new(Kind::Delim, c));
            i += 1;
        } else {
            let mut end = i + 1;
            while operator(at(end)) && !(at(end) == '/' && "/*".contains(at(end + 1))) {
                end += 1;
            }
            tokens.push(Token::new(Kind::Punct, text(i, end)));
            i = end;
        }
    }
    tokens
}

/// Push the `#[doc]` attribute a doc comment stands for. Text longer than
/// `chunk` is cut into pieces joined with `concat!`, which gives the same
/// string. Pieces break between words where they can.
fn doc(tokens: &mut Vec<Token>, inner: bool, body: &str, chunk: usize) {
    let fits = |piece: &str| format!("{piece:?}").chars().count() <= chunk;
    let mut pieces = Vec::new();
    let mut piece = String::new();
    for word in body.split_inclusive(' ') {
        if !piece.is_empty() && !fits(&(piece.clone() + word)) {
            pieces.push(std::mem::take(&mut piece));
        }
        for c in word.chars() {
            piece.push(c);
            if piece.chars().nth(1).is_some() && !fits(&piece) {
                piece.pop();
                pieces.push(std::mem::replace(&mut piece, c.to_string()));
            }
        }
    }
    pieces.push(piece);

    tokens.push(Token::new(Kind::Punct, if inner { "#!" } else { "#" }));
    tokens.push(Token::new(Kind::Delim, '['));
    tokens.push(Token::new(Kind::Word, "doc"));
    tokens.push(Token::new(Kind::Punct, "="));
    let literals = pieces
        .iter()
        .map(|p| Token::new(Kind::Literal, format!("{p:?}")));
    if pieces.len() == 1 {
        tokens.extend(literals);
    } else {
        tokens.push(Token::new(Kind::Word, "concat"));
        tokens.push(Token::new(Kind::Punct, "!"));
        tokens.push(Token::new(Kind::Delim, '('));
        for literal in literals {
            tokens.push(literal);
            tokens.push(Token::new(Kind::Delim, ','));
        }
        tokens.push(Token::new(Kind::Delim, ')'));
    }
    tokens.push(Token::new(Kind::Delim, ']'));
}

/// Rewrite a string literal that has line breaks or tabs in it to fit on one
/// line, using escapes. Raw strings become ordinary ones, since raw strings
/// have no escapes. Anything else is returned as it is.
fn flatten(literal: &str) -> String {
    let Some(open) = literal.find('"') else {
        return literal.to_owned();
    };
    if !literal.contains(['\n', '\r', '\t']) {
        return literal.to_owned();
    }
    let prefix = &literal[..open];
    let raw = prefix.contains('r');
    let hashes = prefix.matches('#').count();
    let close = literal.rfind(&format!("\"{}", "#".repeat(hashes))).unwrap();
    let (body, suffix) = (&literal[open + 1..close], &literal[close + 1 + hashes..]);

    let mut out = prefix.replace(['r', '#'], "") + "\"";
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // A backslash at the end of a line skips the line break and the
            // indentation after it.
            '\\' if !raw => match chars.next() {
                Some('\n' | '\r') => {
                    while chars
                        .next_if(|c| matches!(c, ' ' | '\t' | '\n' | '\r'))
                        .is_some()
                    {}
                }
                Some(e) => {
                    out.push('\\');
                    out.push(e);
                }
                None => {}
            },
            '\\' | '"' => {
                out.push('\\');
                out.push(c);
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out + "\"" + suffix
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Laying code out never changes its tokens, which is what keeps it the
    /// same program. Adjacent tokens that would lex as one show up here.
    #[test]
    fn keeps_tokens() {
        let source = r##"
            macro_rules! m { ($x:expr) => { $x + $x }; }
            fn f<'a>(s: &'a str) -> Vec<Vec<u8>> { vec![s.bytes().collect()] }
            fn main() {
                let a = 1.max(2) - -3 / 2 * -1;
                let b = (b'x', 'y', '\'', br#"r"#, c"c", 1.5e-3f64, ((1, 2), 3).0.1);
                let c = &&a != &&4 && !false || a >= 2 << 1 >> 1;
                let d = r"multi
                    line" == "con\
                    tinued";
                'outer: for r#type in 0..=m!(a) { break 'outer; }
                let e = 10 / *&a;
            }
        "##;
        for (size, id, thickness) in [(1, 0, 1.0), (3, 1, 2.0), (4, 7, 3.0), (5, 20, 4.5)] {
            let flake = Snowflake::from_id(size, id).unwrap();
            let art = format(&flake, source, thickness);
            assert_eq!(lex(&art, 0), lex(source, 0), "size {size} id {id}\n{art}");
        }
    }
}

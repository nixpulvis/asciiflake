//! Print `quineflake`, a quine laid out on flake 0 by `code::format`.
//!
//! The quine is a cargo script. Running it prints itself laid out on the flake
//! with the next ID, which does the same in turn.
//!
//! ```sh
//! cargo run -q --bin metaquineflake > quineflake.rs
//! chmod +x quineflake.rs
//! ./quineflake.rs
//! ```

use asciiflake::{Snowflake, code};

/// The cargo script frontmatter, kept as it is at the top of the quine.
/// `REPOSITORY` is this crate's git repository, from `Cargo.toml`, so the
/// quine builds anywhere. `code` is only on the `fmt-n-quine` branch so far.
const FRONT: &str = r#"#!/usr/bin/env -S cargo +nightly -Zscript
---
[package]
edition = "2024"
[dependencies]
asciiflake.git = REPOSITORY
asciiflake.branch = "fmt-n-quine"
---"#;

/// The rest of the quine. `ID` is where its ID goes and `DATA` is where its
/// own text goes, as an array of short strings so they fit in the strokes. It
/// puts its text back together the same way, with `\u{1}` and `\u{2}` marking
/// those spots. The code is all on one line, so the last line break is where
/// the frontmatter ends. The flake size and thickness here have to match
/// `SIZE` and `THICKNESS`.
const CODE: &str = r#"
use asciiflake::{Snowflake, code};
fn main() {
    let n: u64 = ID;
    let d = [DATA];
    let size = 5;
    let next = (n + 1) % Snowflake::count(size).unwrap();
    let data = d.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>().join(",");
    let source = d.concat().replacen('\u{1}', &next.to_string(), 1).replacen('\u{2}', &data, 1);
    let (front, code) = source.rsplit_once('\n').unwrap();
    println!("{front}\n{}", code::format(&Snowflake::from_id(size, next).unwrap(), code, 3.0));
}
"#;

const SIZE: u32 = 5;
const THICKNESS: f64 = 3.0;

/// The longest piece of text in one string, so each fits a typical stroke.
const PIECE: usize = 5;

fn main() {
    let repository = format!("{:?}", env!("CARGO_PKG_REPOSITORY"));
    let front = FRONT.replacen("REPOSITORY", &repository, 1);
    // Line breaks and indentation in the code don't survive the layout, so
    // there's no need to carry them around. Single spaces keep words apart.
    let code = CODE
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replacen("ID", "\u{1}", 1)
        .replacen("DATA", "\u{2}", 1);
    let text = format!("{front}\n{code}");

    let mut pieces = vec![String::new()];
    for c in text.chars() {
        let piece = pieces.last_mut().unwrap();
        if format!("{:?}", format!("{piece}{c}")).len() > PIECE + 2 && !piece.is_empty() {
            pieces.push(String::new());
        }
        pieces.last_mut().unwrap().push(c);
    }

    // The same steps the quine takes, for ID 0.
    let data = pieces
        .iter()
        .map(|p| format!("{p:?}"))
        .collect::<Vec<_>>()
        .join(",");
    let code = code.replacen('\u{1}', "0", 1).replacen('\u{2}', &data, 1);
    let flake = Snowflake::from_id(SIZE, 0).unwrap();
    println!("{front}\n{}", code::format(&flake, &code, THICKNESS));
}

use std::io::Read;
use std::path::PathBuf;

use asciiflake::{Snowflake, code};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

/// Lay a Rust program out along the branches of a snowflake. The result is
/// still the same program.
#[derive(Parser)]
#[command(name = "rustflake", version)]
struct Args {
    /// Rust source file to read [default: stdin]
    file: Option<PathBuf>,

    /// Arm length in lattice steps
    #[arg(short, long, default_value_t = 4, value_parser = clap::value_parser!(u32).range(1..))]
    size: u32,

    /// Which flake to use, unique up to size 24 [default: random]
    #[arg(short, long)]
    id: Option<u64>,

    /// How many columns the strokes reach out either side of each branch
    #[arg(short, long, default_value_t = 3.0)]
    thickness: f64,
}

fn fail(kind: ErrorKind, message: String) -> ! {
    Args::command().error(kind, message).exit()
}

fn main() {
    let args = Args::parse();

    let source = match &args.file {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut source = String::new();
            std::io::stdin().read_to_string(&mut source).map(|_| source)
        }
    }
    .unwrap_or_else(|e| fail(ErrorKind::Io, e.to_string()));

    if !args.thickness.is_finite() || args.thickness < 1.0 {
        fail(
            ErrorKind::ValueValidation,
            "thickness must be at least 1".into(),
        );
    }

    let last_id = Snowflake::count(args.size).map_or(u64::MAX, |n| n - 1);
    let id = args.id.unwrap_or_else(|| rand::random_range(0..=last_id));
    let Some(flake) = Snowflake::from_id(args.size, id) else {
        fail(
            ErrorKind::ValueValidation,
            format!("size {} has IDs from 0 to {last_id}", args.size),
        )
    };
    println!("{}", code::format(&flake, &source, args.thickness));
}

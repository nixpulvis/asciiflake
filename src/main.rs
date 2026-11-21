use clap::builder::{BoolValueParser, TypedValueParser};
use clap::error::ErrorKind;
use clap::{ArgAction, CommandFactory, Parser};
use asciiflake::{Snowflake, Style};

/// Draw random snowflakes in the terminal.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Arm length in lattice steps
    #[arg(short, long, default_value_t = 8, value_parser = clap::value_parser!(u32).range(1..))]
    size: u32,

    /// Which flake to draw, unique up to size 24 [default: random]
    #[arg(short, long)]
    id: Option<u64>,

    /// Number of flakes to draw
    #[arg(short = 'n', long, default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
    count: u64,

    /// Use Unicode line-drawing characters instead of ASCII
    #[arg(
        short = 'u',
        long = "unicode",
        action = ArgAction::SetTrue,
        value_parser = BoolValueParser::new().map(|on| if on { Style::UNICODE } else { Style::ASCII }),
    )]
    style: Style,

    /// Print the ID under each flake
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let size = args.size;
    let total = Snowflake::count(size);
    let last_id = total.map_or(u64::MAX, |n| n - 1);
    if let Some(first) = args.id
        && !first
            .checked_add(args.count - 1)
            .is_some_and(|last| last <= last_id)
    {
        let message = match total {
            Some(n) => format!("size {size} has {n} flakes, with IDs from 0 to {last_id}"),
            None => format!("IDs go from 0 to {last_id}"),
        };
        Args::command()
            .error(ErrorKind::ValueValidation, message)
            .exit();
    }

    for i in 0..args.count {
        let id = args
            .id
            .map_or_else(|| rand::random_range(0..=last_id), |first| first + i);
        if i > 0 {
            println!();
        }
        println!(
            "{}",
            Snowflake::from_id(size, id).unwrap().render(args.style)
        );
        if args.verbose {
            println!("\nid {id}");
        }
    }
}

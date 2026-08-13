// US-QA.5 — CLI wrapper. All generator logic lives in `bvconsole_lib::qa_dataset`
// (see that module's own doc comment for why) — this binary is just argument
// parsing and file setup.
//
// Usage: cargo run --release --bin generate_dataset -- --scale 500 --out
// /path/to/console.db [--seed 42] [--branching 8]
use std::path::PathBuf;

use bvconsole_lib::db;
use bvconsole_lib::qa_dataset::{entries_per_month, generate_dataset_into};

struct Args {
    scale: usize,
    out: PathBuf,
    seed: u64,
    branching: usize,
    max_depth: Option<usize>,
}

fn parse_args() -> Args {
    let mut scale = 500usize;
    let mut out: Option<PathBuf> = None;
    let mut seed = 42u64;
    let mut branching = 8usize;
    let mut max_depth = None;

    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--scale" => scale = it.next().expect("--scale needs a value").parse().unwrap(),
            "--out" => out = Some(PathBuf::from(it.next().expect("--out needs a value"))),
            "--seed" => seed = it.next().expect("--seed needs a value").parse().unwrap(),
            "--branching" => {
                branching = it
                    .next()
                    .expect("--branching needs a value")
                    .parse()
                    .unwrap()
            }
            "--max-depth" => {
                max_depth = Some(
                    it.next()
                        .expect("--max-depth needs a value")
                        .parse()
                        .unwrap(),
                )
            }
            other => panic!("unknown flag: {other}"),
        }
    }

    Args {
        scale,
        out: out.expect("--out <path> is required"),
        seed,
        branching,
        max_depth,
    }
}

fn main() {
    let args = parse_args();
    assert!(
        [500, 5_000, 25_000].contains(&args.scale),
        "T-QA.5-1: scale must be one of the three named sizes (500 / 5,000 / 25,000), got {}",
        args.scale
    );

    let conn = db::open_encrypted(&args.out, "dev-only-dataset-key")
        .expect("open_encrypted (fresh file, migrated and seeded)");

    let member_ids =
        generate_dataset_into(&conn, args.seed, args.scale, args.branching, args.max_depth);

    println!(
        "generated {} members and {} entries/month × 12 into {}",
        member_ids.len(),
        entries_per_month(member_ids.len()),
        args.out.display()
    );
}

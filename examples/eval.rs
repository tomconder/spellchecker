//! Measures correction accuracy and speed on Peter Norvig's test sets.
//!
//! Run from the repository root: `cargo run --release --example eval`
//! Add `-- -v` to print every wrong correction.

use spellchecker::Checker;
use std::{fs, time::Instant};

fn main() {
    let verbose = std::env::args().any(|a| a == "-v");

    let mut checker = Checker::new();
    checker.train(&fs::read_to_string("src/bin/big.txt").expect("read src/bin/big.txt"));

    // minimum correct count per test set; raise it when accuracy improves
    let sets = [
        ("data/spell-testset1.txt", 212),
        ("data/spell-testset2.txt", 286),
    ];
    let mut failed = false;

    for (path, min_good) in sets {
        let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        // each line is "right: wrong1 wrong2 ..."
        let pairs: Vec<(&str, &str)> = text
            .lines()
            .filter_map(|line| line.split_once(':'))
            .flat_map(|(right, wrongs)| wrongs.split_whitespace().map(move |w| (right.trim(), w)))
            .collect();

        let start = Instant::now();
        let (mut good, mut unknown) = (0, 0);
        for &(right, wrong) in &pairs {
            let got = checker.correct(wrong);
            if got == right {
                good += 1;
                continue;
            }
            let is_unknown = !checker.is_known(right);
            unknown += is_unknown as usize;
            if verbose {
                let note = if is_unknown {
                    " (right word not in training text)"
                } else {
                    ""
                };
                println!("  {wrong} -> {got}, expected {right}{note}");
            }
        }
        let secs = start.elapsed().as_secs_f64();
        let n = pairs.len();

        println!(
            "{path}: {:.0}% of {n} correct ({good}, minimum {min_good}) ({:.0}% unknown) at {:.0} words/s",
            100.0 * good as f64 / n as f64,
            100.0 * unknown as f64 / n as f64,
            n as f64 / secs,
        );
        if good < min_good {
            eprintln!("{path}: accuracy dropped, {good} correct is below the minimum {min_good}");
            failed = true;
        }
    }

    if failed {
        std::process::exit(1);
    }
}

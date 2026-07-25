use spellchecker::Checker;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: spellchecker <training-file> <word>");
        eprintln!("Example: {} training.txt tometo", args[0]);
        std::process::exit(1);
    }

    let mut spellchecker = Checker::new();
    let contents =
        fs::read_to_string(&args[1]).expect("Something went wrong reading the file");
    spellchecker.train(&contents);

    println!("{} -> {}", args[2], spellchecker.correct(&args[2]));
}

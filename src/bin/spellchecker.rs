extern crate spellchecker;
use spellchecker::Checker;
use std::fs;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        writeln!(std::io::stderr(), "Usage: spellchecker <training-file> <word>").unwrap();
        writeln!(std::io::stderr(), "Example: {} training.txt tometo", args[0]).unwrap();
        std::process::exit(1);
    }

    let mut spellchecker = Checker::new();
    let contents = fs::read_to_string(&args[1])
        .expect("Something went wrong reading the file");
    spellchecker.train(&contents);

    println!("{} -> {}", &args[2], spellchecker.correct(&args[2]));
}
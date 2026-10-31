# spellchecker

A Rust library that corrects misspelled words, using the method that Peter
Norvig describes in [How to Write a Spelling
Corrector](http://norvig.com/spell-correct.html).

## Usage

Add the library to your `Cargo.toml` file:

```toml
[dependencies]
spellchecker = "0.1"
```

Train a `Checker` on a large text file, and then call `correct` on a word:

```rust
use spellchecker::Checker;
use std::fs;

let mut spellchecker = Checker::new();

let contents = fs::read_to_string("big.txt").expect("Something went wrong");
spellchecker.train(&contents);

// prints "peotryy -> poetry"
println!("{} -> {}", "peotryy", spellchecker.correct("peotryy"));
```

This repository includes a sample training file at `src/bin/big.txt`. You can
call `train` more than once, and each call adds to the word counts.

## Command-line tool

The crate also has a small command-line tool that trains on a file and corrects
one word. You can install it with `cargo install spellchecker`, or you can run
it from the repository root with Cargo:

```sh
cargo run --bin spellchecker -- src/bin/big.txt peotryy
```

The tool prints `peotryy -> poetry`. It takes exactly two arguments, the
training file and the word, and it exits with status 1 if you give it a
different number of arguments.

## How it works

The checker counts how often each word appears in the training text, and it
picks the most frequent known word that is close to the input. First, if the
input is a known word, the checker returns it. Second, it makes every string
that is one edit away from the input, and it returns the most frequent known
word among them. An edit is a deletion, a swap of two adjacent letters, a
change of one letter, or an insertion of one letter. Third, it tries the strings
that are two edits away in the same way. If no known word is found, the checker
returns the input unchanged.

The checker matches words without regard to case, and it returns the correction
in the case of the input. If the input is all capitals, the correction is all
capitals, so `PEOTRYY` becomes `POETRY`. If the input starts with a capital, the
correction starts with one, so `Peotryy` becomes `Poetry`. Otherwise the
correction is lowercase.

## Evaluation

The repository includes Peter Norvig's two test sets of misspelled words in
`data/`. To measure accuracy and speed, run this command from the repository
root:

```sh
cargo run --release --example eval
```

The current results match the numbers Norvig reports for his own version. The
checker corrects 75% of the 270 words in the first set and 68% of the 400 words
in the second set. Add `-- -v` to print each wrong correction.

CI runs the evaluation on every push and pull request. The run fails if either
test set has fewer correct words than the minimum in `examples/eval.rs`, which
is the current result. When a change improves accuracy, raise the minimum.

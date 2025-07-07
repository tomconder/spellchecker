# spellchecker

A spell checker based on the algorithm described by Peter Norvig, Director of Research at Google.

## Features

* Can be trained one or more times with a large corpus of words
* Probabilistic model for determining the most likely correct spelling

## Installation and Running

To use the library, add the following to your `Cargo.toml` manifest file:

```toml
[dependencies]
regex = { git = "https://github.com/tomconder/spellchecker.git" }
```

Then in your code, you can use the library as follows:

```rust
extern crate spellchecker;
use spellchecker::Checker;
```

Now you can create a `Checker` instance and use it to check spelling:

```rust
let mut spellchecker = Checker::new();

// train the spellchecker with a large text file 
let contents = fs::read_to_string("big.txt").expect("Something went wrong");
spellchecker.train(&contents);

// the expected value is "poetry"
println!("{} -> {}", "peotryy", spellchecker.correct("peotryy"));
```

## Algorithm

### Norvig's algorithm

[Peter Norvig's spell checker](http://norvig.com/spell-correct.html) uses a probabilistic model based on a large corpus
of words to determine the most likely correct spelling. It works by generating a set of possible corrections, or edits,
for a misspelled word. Then it ranks those corrections based on their likelihood of being the correct spelling.

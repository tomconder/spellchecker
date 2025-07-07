//! A spell checker based on the probabilistic algorithm described by Peter Norvig
// in http://norvig.com/spell-correct.html
//!
//! Using the checker involves two steps:
//! 1) call speller.train() with a large text string to train the language model
//! 2) call speller.correct(word) to retrieve the correction for a given word

use std::collections::HashMap;
use regex::Regex;

pub struct Checker {
    /// The letters of the alphabet
    letters: String,
    /// frequency map of words
    freq_words: HashMap<String, u32>
}

impl Checker {
    /// Creates a new `Checker` instance with the alphabet and an empty frequency map.
    pub fn new() -> Self {
        Checker {
            letters: "abcdefghijklmnopqrstuvwxyz".to_string(),
            freq_words: HashMap::new(),
        }
    }

    /// A function to train the spell checker with the given text
    pub fn train(&mut self, text: &str) {
        // split the text into words and add them to the frequency map
        let re = Regex::new(r"[a-z]+").unwrap();
        for m in re.find_iter(&text.to_lowercase()) {
            *self.freq_words.entry(m.as_str().to_string()).or_insert(0) += 1;
        }
    }

    /// A function to correct a word based on the frequency map
    pub fn correct(&mut self, word: &str) -> String {
        // find word in the frequency map
        if self.freq_words.contains_key(word) {
            return word.to_string();
        }

        let mut candidates: HashMap<u32, String> = HashMap::new();
        let list = self.edits(word);

        // find candidates in the edits of the word
        for edit in &list {
            if let Some(value) = self.freq_words.get(edit) {
                candidates.insert(*value, edit.to_string());
            }
        }

        if let Some(c) = candidates.iter().max_by_key(|&entry| entry.0) {
            return c.1.to_string();
        }

        // find candidates in the edits of the edits
        for edit in &list {
            for w in self.edits(&edit) {
                if let Some(value) = self.freq_words.get(&w) {
                    candidates.insert(*value, w);
                }
            }
        }

        if let Some(c) = candidates.iter().max_by_key(|&entry| entry.0) {
            return c.1.to_string();
        }

        // return the input unchanged if no candidates found in the frequency map
        word.to_string()
    }

    fn edits(&mut self, word: &str) -> Vec<String> {
        let mut edits = Vec::new();

        // Generate edits by deleting, transposing, replacing, and inserting letters

        // deletion
        for i in 0 .. word.len() {
            let (first, last) = word.split_at(i);
            edits.push([first, &last[1..]].concat());
        }

        // transposition
        for i in 0 .. word.len() - 1 {
            let (first, last) = word.split_at(i);
            edits.push([first, &last[1..2], &last[..1], &last[2..]].concat());
        }

        // alteration
        for i in 0 .. word.len() {
            for c in self.letters.chars() {
                let (first, last) = word.split_at(i);
                let mut buffer = [0; 1];
                let result = c.encode_utf8(&mut buffer);
                edits.push([first, result, &last[1..]].concat());
            }
        }

        // insertion
        for i in 0 .. word.len() + 1 {
            for c in self.letters.chars() {
                let (first, last) = word.split_at(i);
                let mut buffer = [0; 1];
                let result = c.encode_utf8(&mut buffer);
                edits.push([first, result, last].concat());
            }
        }

        edits
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::Checker;

    #[test]
    fn test_deletion() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("speling"), "spelling");
    }

    #[test]
    fn test_transposition() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spellign"), "spelling");
    }

    #[test]
    fn test_alteration() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spellang"), "spelling");
        assert_eq!(spellchecker.correct("spelleng"), "spelling");
        assert_eq!(spellchecker.correct("spulling"), "spelling");
    }

    #[test]
    fn test_insertion() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spelliing"), "spelling");
        assert_eq!(spellchecker.correct("speelling"), "spelling");
    }
}

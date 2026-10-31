//! A spell checker based on the probabilistic algorithm described by Peter Norvig
//! in <http://norvig.com/spell-correct.html>
//!
//! Using the checker involves two steps:
//! 1) call speller.train() with a large text string to train the language model
//! 2) call speller.correct(word) to retrieve the correction for a given word

use regex::Regex;
use std::collections::HashMap;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";

pub struct Checker {
    /// frequency map of words
    freq_words: HashMap<String, u32>,
    /// Cached regex for word extraction
    word_regex: Regex,
}

impl Checker {
    /// Creates a new `Checker` instance with the alphabet and an empty frequency map.
    pub fn new() -> Self {
        Checker {
            freq_words: HashMap::new(),
            word_regex: Regex::new(r"[a-z]+").unwrap(),
        }
    }

    /// A function to train the spell checker with the given text
    pub fn train(&mut self, text: &str) {
        // split the text into words and add them to the frequency map
        for m in self.word_regex.find_iter(&text.to_lowercase()) {
            *self.freq_words.entry(m.as_str().to_string()).or_insert(0) += 1;
        }
    }

    /// Returns true if the word appears in the training text
    pub fn is_known(&self, word: &str) -> bool {
        self.freq_words.contains_key(&word.to_lowercase())
    }

    /// A function to correct a word based on the frequency map
    pub fn correct(&self, word: &str) -> String {
        let word = &word.to_lowercase();

        // find word in the frequency map
        if self.freq_words.contains_key(word) {
            return word.to_string();
        }

        let possible_edits = self.edits(word);

        // find candidates in the edits of the word
        if let Some(candidate) = self.find_best_candidate(&possible_edits) {
            return candidate;
        }

        // find candidates in the edits of the edits
        let edits_of_edits: Vec<String> = possible_edits
            .iter()
            .flat_map(|edit| self.edits(edit))
            .collect();

        self.find_best_candidate(&edits_of_edits)
            .unwrap_or_else(|| word.to_string())
    }

    fn find_best_candidate(&self, candidates: &[String]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|word| self.freq_words.get(word).map(|freq| (freq, word)))
            .max_by_key(|(freq, _)| *freq)
            .map(|(_, word)| word.clone())
    }

    fn edits(&self, word: &str) -> Vec<String> {
        // index by char, not byte, so non-ASCII input can't split a character
        let chars: Vec<char> = word.chars().collect();
        let word_len = chars.len();
        let join = |parts: &[&[char]]| parts.concat().into_iter().collect::<String>();
        let mut edits = Vec::with_capacity(word_len * 54 + 26);

        // deletion
        for i in 0..word_len {
            edits.push(join(&[&chars[..i], &chars[i + 1..]]));
        }

        // transposition
        for i in 0..word_len.saturating_sub(1) {
            edits.push(join(&[
                &chars[..i],
                &[chars[i + 1], chars[i]],
                &chars[i + 2..],
            ]));
        }

        // alteration
        for i in 0..word_len {
            for c in LETTERS.chars() {
                edits.push(join(&[&chars[..i], &[c], &chars[i + 1..]]));
            }
        }

        // insertion
        for i in 0..=word_len {
            for c in LETTERS.chars() {
                edits.push(join(&[&chars[..i], &[c], &chars[i..]]));
            }
        }

        edits
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Checker;

    #[test]
    fn test_deletion() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spelling"), "spelling");
    }

    #[test]
    fn test_transposition() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spelling"), "spelling");
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
        assert_eq!(spellchecker.correct("spelling"), "spelling");
    }

    #[test]
    fn test_correct_lowercases_input() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("Spelling"), "spelling");
    }

    #[test]
    fn test_non_ascii_input() {
        let mut spellchecker = Checker::new();
        spellchecker.train("cafe");
        assert_eq!(spellchecker.correct("café"), "cafe");
        assert_eq!(spellchecker.correct("ñ"), "ñ");
    }
}

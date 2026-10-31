//! A spell checker based on the probabilistic algorithm described by Peter Norvig
//! in <http://norvig.com/spell-correct.html>
//!
//! Using the checker involves two steps:
//! 1) call speller.train() with a large text string to train the language model
//! 2) call speller.correct(word) to retrieve the correction for a given word

use std::collections::HashMap;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Default)]
pub struct Checker {
    /// frequency map of words
    freq_words: HashMap<String, u32>,
}

impl Checker {
    /// Creates a new `Checker` instance with an empty frequency map.
    pub fn new() -> Self {
        Self::default()
    }

    /// A function to train the spell checker with the given text
    pub fn train(&mut self, text: &str) {
        // words are runs of a-z, the same as the regex [a-z]+
        let text = text.to_lowercase();
        for word in text
            .split(|c: char| !c.is_ascii_lowercase())
            .filter(|w| !w.is_empty())
        {
            *self.freq_words.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    /// Returns true if the word appears in the training text
    pub fn is_known(&self, word: &str) -> bool {
        self.freq_words.contains_key(&word.to_lowercase())
    }

    /// Corrects a word based on the frequency map, keeping the input's case:
    /// ALL CAPS stays all caps, a capital first letter stays capital, and
    /// anything else is lowercase. A word with no correction is returned unchanged.
    pub fn correct(&self, word: &str) -> String {
        match self.find_correction(&word.to_lowercase()) {
            Some(correction) => match_case(word, correction),
            None => word.to_string(),
        }
    }

    fn find_correction(&self, word: &str) -> Option<String> {
        // find word in the frequency map
        if self.freq_words.contains_key(word) {
            return Some(word.to_string());
        }

        let possible_edits = self.edits(word);

        // find candidates in the edits of the word
        if let Some(candidate) = self.find_best_candidate(&possible_edits) {
            return Some(candidate);
        }

        // find candidates in the edits of the edits
        let edits_of_edits: Vec<String> = possible_edits
            .iter()
            .flat_map(|edit| self.edits(edit))
            .collect();

        self.find_best_candidate(&edits_of_edits)
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
        let mut edits = Vec::new();

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

/// Gives the lowercase `word` the case pattern of `original`.
fn match_case(original: &str, word: String) -> String {
    let has_upper = original.chars().any(char::is_uppercase);
    if has_upper && !original.chars().any(char::is_lowercase) {
        return word.to_uppercase();
    }
    if original.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            return first.to_uppercase().chain(chars).collect();
        }
    }
    word
}

#[cfg(test)]
mod tests {
    use super::Checker;

    #[test]
    fn test_deletion() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spellng"), "spelling");
    }

    #[test]
    fn test_transposition() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("spellnig"), "spelling");
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
    }

    #[test]
    fn test_preserves_case() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("Spelling"), "Spelling");
        assert_eq!(spellchecker.correct("Spellng"), "Spelling");
        assert_eq!(spellchecker.correct("SPELLNG"), "SPELLING");
        assert_eq!(spellchecker.correct("sPELLNG"), "spelling");
    }

    #[test]
    fn test_unknown_word_unchanged() {
        let mut spellchecker = Checker::new();
        spellchecker.train("spelling");
        assert_eq!(spellchecker.correct("Xyz"), "Xyz");
    }

    #[test]
    fn test_non_ascii_input() {
        let mut spellchecker = Checker::new();
        spellchecker.train("cafe");
        assert_eq!(spellchecker.correct("café"), "cafe");
        assert_eq!(spellchecker.correct("ñ"), "ñ");
    }
}

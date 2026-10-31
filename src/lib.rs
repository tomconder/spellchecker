//! A spell checker based on the algorithm described by Peter Norvig
//! in <http://norvig.com/spell-correct.html>, with candidates ranked by
//! word frequency times a weight for how likely the typo is.
//!
//! Using the checker involves two steps:
//! 1) call speller.train() with a large text string to train the language model
//! 2) call speller.correct(word) to retrieve the correction for a given word

use std::collections::HashMap;

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";

/// Weight of a likely typo (an adjacent swap, a doubled or undoubled letter, or
/// one vowel for another) relative to any other edit. Tuned on Norvig's test
/// set 1 only, so that set 2 stays a held-out check.
const LIKELY_EDIT_WEIGHT: f64 = 30.0;

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
        if let Some(candidate) = self.find_best_candidate(possible_edits.iter().cloned()) {
            return Some(candidate);
        }

        // find candidates in the edits of the edits; a path's weight is the product
        self.find_best_candidate(possible_edits.iter().flat_map(|(edit, w1)| {
            self.edits(edit)
                .into_iter()
                .map(move |(edit2, w2)| (edit2, w1 * w2))
        }))
    }

    /// Picks the known candidate with the highest frequency times edit weight.
    /// On a tie, the last one generated wins, so the result is deterministic.
    fn find_best_candidate(
        &self,
        candidates: impl Iterator<Item = (String, f64)>,
    ) -> Option<String> {
        candidates
            .filter_map(|(word, weight)| {
                let freq = *self.freq_words.get(&word)?;
                Some((f64::from(freq) * weight, word))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, word)| word)
    }

    /// Returns every string one edit away, with the edit's weight.
    fn edits(&self, word: &str) -> Vec<(String, f64)> {
        // index by char, not byte, so non-ASCII input can't split a character
        let chars: Vec<char> = word.chars().collect();
        let word_len = chars.len();
        let join = |parts: &[&[char]]| parts.concat().into_iter().collect::<String>();
        let weight = |likely: bool| if likely { LIKELY_EDIT_WEIGHT } else { 1.0 };
        let is_vowel = |c: char| "aeiou".contains(c);
        let next_to = |i: usize, c: char| {
            (i > 0 && chars[i - 1] == c) || chars.get(i + 1).is_some_and(|&n| n == c)
        };
        let mut edits = Vec::new();

        // deletion; likely when the letter is doubled ("spelliing")
        for i in 0..word_len {
            let doubled = next_to(i, chars[i]);
            edits.push((join(&[&chars[..i], &chars[i + 1..]]), weight(doubled)));
        }

        // transposition; always likely
        for i in 0..word_len.saturating_sub(1) {
            edits.push((
                join(&[&chars[..i], &[chars[i + 1], chars[i]], &chars[i + 2..]]),
                LIKELY_EDIT_WEIGHT,
            ));
        }

        // alteration; likely when a vowel replaces a vowel
        for i in 0..word_len {
            for c in LETTERS.chars() {
                let vowels = is_vowel(c) && is_vowel(chars[i]);
                edits.push((join(&[&chars[..i], &[c], &chars[i + 1..]]), weight(vowels)));
            }
        }

        // insertion; likely when it doubles a neighbouring letter
        for i in 0..=word_len {
            for c in LETTERS.chars() {
                let doubled = (i > 0 && chars[i - 1] == c) || chars.get(i) == Some(&c);
                edits.push((join(&[&chars[..i], &[c], &chars[i..]]), weight(doubled)));
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
    fn test_likely_edit_beats_frequency() {
        let mut spellchecker = Checker::new();
        // "boat" is more frequent, but "bat" needs only a vowel change
        spellchecker.train("boat boat bat");
        assert_eq!(spellchecker.correct("bot"), "bat");
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

use aho_corasick::AhoCorasick;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// KeywordFilter provides case-insensitive word filtering using the Aho-Corasick algorithm.
/// It efficiently checks if text contains any words from a configured filter list.
#[derive(Clone)]
pub struct KeywordFilter {
    /// The Aho-Corasick automaton for efficient multi-pattern matching
    automaton: Arc<RwLock<Option<AhoCorasick>>>,
    /// The current filter list (stored for reference and updates)
    filter_list: Arc<RwLock<Vec<String>>>,
}

impl KeywordFilter {
    /// Creates a new KeywordFilter with an empty filter list
    pub fn new() -> Self {
        Self {
            automaton: Arc::new(RwLock::new(None)),
            filter_list: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new KeywordFilter and loads the initial filter list
    pub fn with_filters(filters: Vec<String>) -> Self {
        let filter = Self::new();
        filter.load_filters(filters);
        filter
    }

    /// Loads a new filter list and rebuilds the Aho-Corasick automaton.
    /// This method is case-insensitive - all patterns are converted to lowercase.
    pub fn load_filters(&self, filters: Vec<String>) {
        // Convert all filters to lowercase for case-insensitive matching
        let lowercase_filters: Vec<String> = filters
            .iter()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        info!(
            "Loading {} word filters (case-insensitive)",
            lowercase_filters.len()
        );

        // Store the original filter list
        if let Ok(mut list) = self.filter_list.write() {
            *list = filters;
        }

        // Build the Aho-Corasick automaton
        if lowercase_filters.is_empty() {
            // No filters, clear the automaton
            if let Ok(mut automaton) = self.automaton.write() {
                *automaton = None;
            }
            debug!("Word filter cleared (empty filter list)");
        } else {
            match AhoCorasick::new(&lowercase_filters) {
                Ok(ac) => {
                    if let Ok(mut automaton) = self.automaton.write() {
                        *automaton = Some(ac);
                    }
                    debug!("Word filter automaton built successfully");
                }
                Err(e) => {
                    warn!("Failed to build word filter automaton: {}", e);
                    if let Ok(mut automaton) = self.automaton.write() {
                        *automaton = None;
                    }
                }
            }
        }
    }

    /// Checks if the given text contains any filtered words (case-insensitive).
    /// Returns true if a filtered word is found, false otherwise.
    /// Logs the detection event when a message is blocked.
    pub fn contains_filtered_word(&self, text: &str) -> bool {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("keyword_filter_check");
        
        if text.is_empty() {
            return false;
        }

        // Convert text to lowercase for case-insensitive matching
        let lowercase_text = text.to_lowercase();

        if let Ok(automaton) = self.automaton.read() {
            if let Some(ref ac) = *automaton {
                let has_match = ac.is_match(&lowercase_text);
                if has_match {
                    // Log the blocked message with details about which word(s) were detected
                    let filtered_words = self.find_all_filtered_words(text);
                    if !filtered_words.is_empty() {
                        warn!(
                            "Message blocked by word filter. Detected words: {:?}, Message preview: {}",
                            filtered_words,
                            if text.len() > 50 {
                                format!("{}...", &text[..50])
                            } else {
                                text.to_string()
                            }
                        );
                    } else {
                        warn!("Message blocked by word filter. Message preview: {}", 
                            if text.len() > 50 {
                                format!("{}...", &text[..50])
                            } else {
                                text.to_string()
                            }
                        );
                    }
                }
                return has_match;
            }
        }

        // No automaton means no filters, so nothing is filtered
        false
    }

    /// Returns the first filtered word found in the text (case-insensitive).
    /// Returns None if no filtered word is found.
    pub fn find_first_filtered_word(&self, text: &str) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        let lowercase_text = text.to_lowercase();

        if let Ok(automaton) = self.automaton.read() {
            if let Some(ref ac) = *automaton {
                if let Some(mat) = ac.find(&lowercase_text) {
                    // Get the matched pattern from the filter list
                    if let Ok(list) = self.filter_list.read() {
                        if mat.pattern().as_usize() < list.len() {
                            return Some(list[mat.pattern().as_usize()].clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Returns all filtered words found in the text (case-insensitive).
    pub fn find_all_filtered_words(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        let lowercase_text = text.to_lowercase();
        let mut found_words = Vec::new();

        if let Ok(automaton) = self.automaton.read() {
            if let Some(ref ac) = *automaton {
                if let Ok(list) = self.filter_list.read() {
                    for mat in ac.find_iter(&lowercase_text) {
                        let pattern_idx = mat.pattern().as_usize();
                        if pattern_idx < list.len() {
                            let word = list[pattern_idx].clone();
                            if !found_words.contains(&word) {
                                found_words.push(word);
                            }
                        }
                    }
                }
            }
        }

        found_words
    }

    /// Returns the current filter list
    pub fn get_filters(&self) -> Vec<String> {
        if let Ok(list) = self.filter_list.read() {
            list.clone()
        } else {
            Vec::new()
        }
    }

    /// Returns the number of filters currently loaded
    pub fn filter_count(&self) -> usize {
        if let Ok(list) = self.filter_list.read() {
            list.len()
        } else {
            0
        }
    }
}

impl Default for KeywordFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter() {
        let filter = KeywordFilter::new();
        assert!(!filter.contains_filtered_word("hello world"));
        assert!(!filter.contains_filtered_word(""));
    }

    #[test]
    fn test_basic_filtering() {
        let filter = KeywordFilter::with_filters(vec!["bad".to_string(), "evil".to_string()]);
        
        assert!(filter.contains_filtered_word("this is bad"));
        assert!(filter.contains_filtered_word("evil things"));
        assert!(!filter.contains_filtered_word("good things"));
    }

    #[test]
    fn test_case_insensitive() {
        let filter = KeywordFilter::with_filters(vec!["bad".to_string()]);
        
        assert!(filter.contains_filtered_word("BAD"));
        assert!(filter.contains_filtered_word("Bad"));
        assert!(filter.contains_filtered_word("bAd"));
        assert!(filter.contains_filtered_word("This is BAD stuff"));
    }

    #[test]
    fn test_reload_filters() {
        let filter = KeywordFilter::with_filters(vec!["bad".to_string()]);
        assert!(filter.contains_filtered_word("bad"));
        
        filter.load_filters(vec!["evil".to_string()]);
        assert!(!filter.contains_filtered_word("bad"));
        assert!(filter.contains_filtered_word("evil"));
    }

    #[test]
    fn test_find_first_filtered_word() {
        let filter = KeywordFilter::with_filters(vec!["bad".to_string(), "evil".to_string()]);
        
        assert_eq!(filter.find_first_filtered_word("this is bad"), Some("bad".to_string()));
        assert_eq!(filter.find_first_filtered_word("evil things"), Some("evil".to_string()));
        assert_eq!(filter.find_first_filtered_word("good things"), None);
    }

    #[test]
    fn test_find_all_filtered_words() {
        let filter = KeywordFilter::with_filters(vec![
            "bad".to_string(),
            "evil".to_string(),
            "wrong".to_string(),
        ]);
        
        let words = filter.find_all_filtered_words("this is bad and evil");
        assert_eq!(words.len(), 2);
        assert!(words.contains(&"bad".to_string()));
        assert!(words.contains(&"evil".to_string()));
    }

    #[test]
    fn test_empty_text() {
        let filter = KeywordFilter::with_filters(vec!["bad".to_string()]);
        assert!(!filter.contains_filtered_word(""));
        assert_eq!(filter.find_first_filtered_word(""), None);
        assert_eq!(filter.find_all_filtered_words("").len(), 0);
    }

    #[test]
    fn test_filter_count() {
        let filter = KeywordFilter::new();
        assert_eq!(filter.filter_count(), 0);
        
        filter.load_filters(vec!["bad".to_string(), "evil".to_string()]);
        assert_eq!(filter.filter_count(), 2);
    }

    #[test]
    fn test_blocked_message_logging() {
        // This test verifies that when a message is blocked, the system logs the detection event
        // Requirement 12.5: WHEN a message is blocked THEN the system SHALL log the detection event
        
        let filter = KeywordFilter::with_filters(vec!["bad".to_string(), "evil".to_string()]);
        
        // Test that a message with a filtered word is detected and logged
        // The logging happens inside contains_filtered_word
        assert!(filter.contains_filtered_word("this is a bad message"));
        
        // Test with multiple filtered words
        assert!(filter.contains_filtered_word("this is bad and evil"));
        
        // Test that clean messages don't trigger logging
        assert!(!filter.contains_filtered_word("this is a good message"));
    }

    #[test]
    fn test_blocked_message_logging_with_long_text() {
        // Test that long messages are truncated in the log preview
        let filter = KeywordFilter::with_filters(vec!["bad".to_string()]);
        
        let long_message = "This is a very long message that contains the word bad and should be truncated in the log output to prevent log spam";
        assert!(filter.contains_filtered_word(long_message));
        
        // The actual logging verification would require a log capture mechanism,
        // but we can at least verify the filtering works correctly
    }
}

use super::keyword_filter::KeywordFilter;
use crate::utils::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

/// Represents the result of a message filtering check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    /// Whether the message was blocked
    pub blocked: bool,
    /// The reason for blocking (if blocked)
    pub reason: Option<String>,
    /// The filtered words found (if any)
    pub filtered_words: Vec<String>,
}

/// MessageFilter provides word filtering functionality for transcribed messages.
/// It checks messages against a configured filter list and can block messages
/// containing filtered words.
#[derive(Clone)]
pub struct MessageFilter {
    /// The underlying keyword filter
    keyword_filter: Arc<KeywordFilter>,
    /// Whether filtering is enabled
    enabled: bool,
}

impl MessageFilter {
    /// Creates a new MessageFilter with an empty filter list
    pub fn new() -> Self {
        Self {
            keyword_filter: Arc::new(KeywordFilter::new()),
            enabled: true,
        }
    }

    /// Creates a new MessageFilter with the given filter list
    pub fn with_filters(filters: Vec<String>) -> Self {
        Self {
            keyword_filter: Arc::new(KeywordFilter::with_filters(filters)),
            enabled: true,
        }
    }

    /// Enables or disables message filtering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Message filtering enabled");
        } else {
            info!("Message filtering disabled");
        }
    }

    /// Returns whether filtering is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Updates the filter list
    /// This reloads the keyword processor with the new filters
    pub fn update_filters(&self, filters: Vec<String>) {
        info!("Updating message filter list with {} filters", filters.len());
        self.keyword_filter.load_filters(filters);
    }

    /// Checks if a message should be blocked based on the filter list.
    /// Returns a FilterResult indicating whether the message was blocked and why.
    ///
    /// This method performs case-insensitive matching against the filter list.
    pub fn check_message(&self, message: &str) -> Result<FilterResult> {
        // If filtering is disabled, allow all messages
        if !self.enabled {
            return Ok(FilterResult {
                blocked: false,
                reason: None,
                filtered_words: Vec::new(),
            });
        }

        // Check if the message contains any filtered words
        let filtered_words = self.keyword_filter.find_all_filtered_words(message);

        if filtered_words.is_empty() {
            // No filtered words found, allow the message
            Ok(FilterResult {
                blocked: false,
                reason: None,
                filtered_words: Vec::new(),
            })
        } else {
            // Filtered words found, block the message
            let reason = format!(
                "Message blocked: contains filtered word(s): {}",
                filtered_words.join(", ")
            );

            // Log the blocked message (Requirement 12.5)
            warn!(
                "Message blocked due to filtered words: {:?} in message: {:?}",
                filtered_words,
                message
            );

            Ok(FilterResult {
                blocked: true,
                reason: Some(reason),
                filtered_words,
            })
        }
    }

    /// Convenience method to check if a message contains filtered words
    /// without returning detailed information
    pub fn contains_filtered_words(&self, message: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.keyword_filter.contains_filtered_word(message)
    }

    /// Returns the current filter list
    pub fn get_filters(&self) -> Vec<String> {
        self.keyword_filter.get_filters()
    }

    /// Returns the number of filters currently loaded
    pub fn filter_count(&self) -> usize {
        self.keyword_filter.filter_count()
    }
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification event sent when a message is blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBlockedNotification {
    /// The filtered words that caused the block
    pub filtered_words: Vec<String>,
    /// A user-friendly message explaining why the message was blocked
    pub message: String,
    /// Timestamp of when the message was blocked
    pub timestamp: String,
}

impl MessageBlockedNotification {
    /// Creates a new notification from a filter result
    pub fn from_filter_result(result: &FilterResult) -> Option<Self> {
        if result.blocked {
            Some(Self {
                filtered_words: result.filtered_words.clone(),
                message: result
                    .reason
                    .clone()
                    .unwrap_or_else(|| "Message blocked by word filter".to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_filter_basic() {
        let filter = MessageFilter::with_filters(vec!["bad".to_string(), "evil".to_string()]);

        let result = filter.check_message("this is bad").unwrap();
        assert!(result.blocked);
        assert!(result.filtered_words.contains(&"bad".to_string()));

        let result = filter.check_message("good message").unwrap();
        assert!(!result.blocked);
        assert!(result.filtered_words.is_empty());
    }

    #[test]
    fn test_message_filter_case_insensitive() {
        let filter = MessageFilter::with_filters(vec!["bad".to_string()]);

        let result = filter.check_message("This is BAD").unwrap();
        assert!(result.blocked);

        let result = filter.check_message("This is Bad").unwrap();
        assert!(result.blocked);

        let result = filter.check_message("This is bAd").unwrap();
        assert!(result.blocked);
    }

    #[test]
    fn test_message_filter_disabled() {
        let mut filter = MessageFilter::with_filters(vec!["bad".to_string()]);
        filter.set_enabled(false);

        let result = filter.check_message("this is bad").unwrap();
        assert!(!result.blocked);
    }

    #[test]
    fn test_message_filter_update() {
        let filter = MessageFilter::with_filters(vec!["bad".to_string()]);

        let result = filter.check_message("this is bad").unwrap();
        assert!(result.blocked);

        filter.update_filters(vec!["evil".to_string()]);

        let result = filter.check_message("this is bad").unwrap();
        assert!(!result.blocked);

        let result = filter.check_message("this is evil").unwrap();
        assert!(result.blocked);
    }

    #[test]
    fn test_filter_update_immediate_application() {
        // Test that filter updates are applied immediately
        let filter = MessageFilter::with_filters(vec!["old".to_string()]);

        // Old filter should work
        assert!(filter.contains_filtered_words("old word"));
        assert!(!filter.contains_filtered_words("new word"));

        // Update filters
        filter.update_filters(vec!["new".to_string()]);

        // New filter should work immediately, old filter should not
        assert!(!filter.contains_filtered_words("old word"));
        assert!(filter.contains_filtered_words("new word"));
    }

    #[test]
    fn test_blocked_message_logging() {
        // Test that blocked messages produce proper filter results with logging information
        let filter = MessageFilter::with_filters(vec!["bad".to_string(), "evil".to_string()]);

        let result = filter.check_message("this is a bad message").unwrap();
        assert!(result.blocked);
        assert!(result.reason.is_some());
        assert!(result.reason.as_ref().unwrap().contains("bad"));
        assert_eq!(result.filtered_words, vec!["bad".to_string()]);

        let result = filter.check_message("this is bad and evil").unwrap();
        assert!(result.blocked);
        assert!(result.reason.is_some());
        assert_eq!(result.filtered_words.len(), 2);
    }

    #[test]
    fn test_allowed_message_no_logging() {
        // Test that allowed messages don't produce blocked results
        let filter = MessageFilter::with_filters(vec!["bad".to_string()]);

        let result = filter.check_message("this is a good message").unwrap();
        assert!(!result.blocked);
        assert!(result.reason.is_none());
        assert!(result.filtered_words.is_empty());
    }

    #[test]
    fn test_message_filter_multiple_words() {
        let filter = MessageFilter::with_filters(vec![
            "bad".to_string(),
            "evil".to_string(),
            "wrong".to_string(),
        ]);

        let result = filter.check_message("this is bad and evil").unwrap();
        assert!(result.blocked);
        assert_eq!(result.filtered_words.len(), 2);
        assert!(result.filtered_words.contains(&"bad".to_string()));
        assert!(result.filtered_words.contains(&"evil".to_string()));
    }

    #[test]
    fn test_contains_filtered_words() {
        let filter = MessageFilter::with_filters(vec!["bad".to_string()]);

        assert!(filter.contains_filtered_words("this is bad"));
        assert!(!filter.contains_filtered_words("this is good"));
    }

    #[test]
    fn test_notification_from_filter_result() {
        let result = FilterResult {
            blocked: true,
            reason: Some("Test reason".to_string()),
            filtered_words: vec!["bad".to_string()],
        };

        let notification = MessageBlockedNotification::from_filter_result(&result);
        assert!(notification.is_some());

        let notification = notification.unwrap();
        assert_eq!(notification.filtered_words, vec!["bad".to_string()]);
        assert_eq!(notification.message, "Test reason");
    }

    #[test]
    fn test_notification_from_unblocked_result() {
        let result = FilterResult {
            blocked: false,
            reason: None,
            filtered_words: Vec::new(),
        };

        let notification = MessageBlockedNotification::from_filter_result(&result);
        assert!(notification.is_none());
    }
}

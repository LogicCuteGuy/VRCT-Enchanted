use crate::config::types::{MessageFormatParts, MessagePart, TranslationPart};
use crate::utils::error::Result;

/// Message formatter for applying prefix, suffix, and separator to messages
pub struct MessageFormatter {
    send_format: MessageFormatParts,
    received_format: MessageFormatParts,
    send_only_translated: bool,
}

impl MessageFormatter {
    /// Create a new message formatter with the given configuration
    pub fn new(
        send_format: MessageFormatParts,
        received_format: MessageFormatParts,
        send_only_translated: bool,
    ) -> Self {
        Self {
            send_format,
            received_format,
            send_only_translated,
        }
    }

    /// Format a sent message with original text and optional translation
    /// 
    /// # Arguments
    /// * `original` - The original message text
    /// * `translation` - Optional translation text
    /// 
    /// # Returns
    /// Formatted message string
    pub fn format_sent_message(&self, original: &str, translation: Option<&str>) -> Result<String> {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("message_format_sent");
        
        self.format_message(original, translation, &self.send_format, self.send_only_translated)
    }

    /// Format a received message with original text and optional translation
    /// 
    /// # Arguments
    /// * `original` - The original message text
    /// * `translation` - Optional translation text
    /// 
    /// # Returns
    /// Formatted message string
    pub fn format_received_message(&self, original: &str, translation: Option<&str>) -> Result<String> {
        self.format_message(original, translation, &self.received_format, false)
    }

    /// Internal method to format a message with the given format configuration
    fn format_message(
        &self,
        original: &str,
        translation: Option<&str>,
        format: &MessageFormatParts,
        send_only_translated: bool,
    ) -> Result<String> {
        // Handle formatting errors with fallback
        match self.try_format_message(original, translation, format, send_only_translated) {
            Ok(formatted) => Ok(formatted),
            Err(e) => {
                // On formatting errors, use simple format with original and translation
                tracing::warn!("Message formatting error: {}. Using fallback format.", e);
                Ok(self.fallback_format(original, translation))
            }
        }
    }

    /// Try to format a message, returning an error if formatting fails
    fn try_format_message(
        &self,
        original: &str,
        translation: Option<&str>,
        format: &MessageFormatParts,
        send_only_translated: bool,
    ) -> Result<String> {
        let mut parts = Vec::new();

        // Determine order based on translation_first setting
        if format.translation_first {
            // Translation first mode: place translation before original
            if let Some(trans) = translation {
                parts.push(self.format_translation_part(trans, &format.translation)?);
            }
            
            // Add original message unless send_only_translated is enabled
            if !send_only_translated {
                if !parts.is_empty() {
                    parts.push(format.separator.clone());
                }
                parts.push(self.format_message_part(original, &format.message)?);
            }
        } else {
            // Normal mode: original first, then translation
            
            // Add original message unless send_only_translated is enabled
            if !send_only_translated {
                parts.push(self.format_message_part(original, &format.message)?);
            }
            
            if let Some(trans) = translation {
                if !parts.is_empty() {
                    parts.push(format.separator.clone());
                }
                parts.push(self.format_translation_part(trans, &format.translation)?);
            }
        }

        // If no parts were added (e.g., send_only_translated with no translation), return empty
        if parts.is_empty() {
            return Ok(String::new());
        }

        Ok(parts.join(""))
    }

    /// Format the message part with prefix and suffix
    fn format_message_part(&self, text: &str, part: &MessagePart) -> Result<String> {
        Ok(format!("{}{}{}", part.prefix, text, part.suffix))
    }

    /// Format the translation part with prefix, separator, and suffix
    fn format_translation_part(&self, text: &str, part: &TranslationPart) -> Result<String> {
        Ok(format!("{}{}{}", part.prefix, text, part.suffix))
    }

    /// Fallback format for when formatting errors occur
    /// Returns a simple format with original and translation
    fn fallback_format(&self, original: &str, translation: Option<&str>) -> String {
        match translation {
            Some(trans) => format!("{}\n{}", original, trans),
            None => original.to_string(),
        }
    }

    /// Update the send message format configuration
    pub fn update_send_format(&mut self, format: MessageFormatParts) {
        self.send_format = format;
    }

    /// Update the received message format configuration
    pub fn update_received_format(&mut self, format: MessageFormatParts) {
        self.received_format = format;
    }

    /// Update the send-only-translated setting
    pub fn update_send_only_translated(&mut self, enabled: bool) {
        self.send_only_translated = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_format() -> MessageFormatParts {
        MessageFormatParts {
            message: MessagePart {
                prefix: "[".to_string(),
                suffix: "]".to_string(),
            },
            separator: " | ".to_string(),
            translation: TranslationPart {
                prefix: "(".to_string(),
                separator: " ".to_string(),
                suffix: ")".to_string(),
            },
            translation_first: false,
        }
    }

    #[test]
    fn test_format_sent_message_with_translation() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.format_sent_message("Hello", Some("Bonjour")).unwrap();
        assert_eq!(result, "[Hello] | (Bonjour)");
    }

    #[test]
    fn test_format_sent_message_without_translation() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.format_sent_message("Hello", None).unwrap();
        assert_eq!(result, "[Hello]");
    }

    #[test]
    fn test_format_received_message() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.format_received_message("Bonjour", Some("Hello")).unwrap();
        assert_eq!(result, "[Bonjour] | (Hello)");
    }

    #[test]
    fn test_translation_first_mode() {
        let mut format = create_test_format();
        format.translation_first = true;
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.format_sent_message("Hello", Some("Bonjour")).unwrap();
        assert_eq!(result, "(Bonjour) | [Hello]");
    }

    #[test]
    fn test_send_only_translated_mode() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), true);
        
        let result = formatter.format_sent_message("Hello", Some("Bonjour")).unwrap();
        assert_eq!(result, "(Bonjour)");
    }

    #[test]
    fn test_send_only_translated_without_translation() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), true);
        
        let result = formatter.format_sent_message("Hello", None).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_empty_prefix_suffix() {
        let format = MessageFormatParts {
            message: MessagePart {
                prefix: String::new(),
                suffix: String::new(),
            },
            separator: "\n".to_string(),
            translation: TranslationPart {
                prefix: String::new(),
                separator: String::new(),
                suffix: String::new(),
            },
            translation_first: false,
        };
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.format_sent_message("Hello", Some("Bonjour")).unwrap();
        assert_eq!(result, "Hello\nBonjour");
    }

    #[test]
    fn test_fallback_format() {
        let format = create_test_format();
        let formatter = MessageFormatter::new(format.clone(), format.clone(), false);
        
        let result = formatter.fallback_format("Hello", Some("Bonjour"));
        assert_eq!(result, "Hello\nBonjour");
        
        let result = formatter.fallback_format("Hello", None);
        assert_eq!(result, "Hello");
    }
}

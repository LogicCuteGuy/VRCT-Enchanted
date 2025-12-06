// Transliteration subsystem module
// This module handles Japanese text transliteration (kana/romaji conversion)

pub mod kana;
pub mod romaji;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Represents a segment of transliterated text with multiple representations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransliterationSegment {
    /// Original text (may contain kanji, kana, or other characters)
    pub orig: String,
    /// Katakana reading
    pub kana: String,
    /// Hiragana reading
    pub hira: String,
    /// Hepburn romanization
    pub hepburn: String,
}

/// Settings for transliteration behavior
#[derive(Debug, Clone)]
pub struct TransliterationSettings {
    /// Whether to use macrons for long vowels (ā, ī, ū, ē, ō)
    pub use_macron: bool,
}

impl Default for TransliterationSettings {
    fn default() -> Self {
        Self { use_macron: false }
    }
}

/// Main transliterator for Japanese text
pub struct Transliterator {
    settings: Arc<std::sync::RwLock<TransliterationSettings>>,
}

impl Transliterator {
    /// Create a new transliterator with default settings
    pub fn new() -> Self {
        Self {
            settings: Arc::new(std::sync::RwLock::new(TransliterationSettings::default())),
        }
    }

    /// Update transliteration settings at runtime
    pub fn update_settings(&self, settings: TransliterationSettings) {
        if let Ok(mut s) = self.settings.write() {
            *s = settings;
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> TransliterationSettings {
        self.settings
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Analyze text and return transliteration segments
    /// 
    /// This method tokenizes the input text and produces transliteration
    /// information for each segment. On error, returns the original text
    /// as a single segment.
    pub fn analyze(&self, text: &str) -> Vec<TransliterationSegment> {
        use crate::utils::profiling::global_profiler;
        let _guard = global_profiler().start_timing("transliteration_analyze");
        
        // Get current settings
        let settings = self.get_settings();
        
        // Try to analyze the text
        match self.analyze_internal(text, settings.use_macron) {
            Ok(segments) => segments,
            Err(_) => {
                // On error, return original text unchanged
                vec![TransliterationSegment {
                    orig: text.to_string(),
                    kana: text.to_string(),
                    hira: text.to_string(),
                    hepburn: text.to_string(),
                }]
            }
        }
    }

    /// Internal analysis implementation
    fn analyze_internal(
        &self,
        text: &str,
        use_macron: bool,
    ) -> Result<Vec<TransliterationSegment>, Box<dyn std::error::Error>> {
        // Simple character-by-character conversion
        // This handles basic kana conversion without morphological analysis
        // A full implementation would use a morphological analyzer like lindera
        
        let mut segments = Vec::new();
        let mut current_segment = String::new();
        let mut is_kana_segment = false;
        
        for ch in text.chars() {
            let ch_is_kana = kana::is_kana(ch);
            
            // Check if we need to flush the current segment
            if !current_segment.is_empty() && is_kana_segment != ch_is_kana {
                // Flush the current segment
                if is_kana_segment {
                    // Process kana segment
                    let kana_text = if kana::is_hiragana(current_segment.chars().next().unwrap_or(' ')) {
                        kana::hiragana_to_katakana(&current_segment)
                    } else {
                        current_segment.clone()
                    };
                    
                    let hira_text = kana::katakana_to_hiragana(&kana_text);
                    let hepburn_text = romaji::katakana_to_hepburn(&kana_text, use_macron);
                    
                    segments.push(TransliterationSegment {
                        orig: current_segment.clone(),
                        kana: kana_text,
                        hira: hira_text,
                        hepburn: hepburn_text,
                    });
                } else {
                    // Non-kana segment - keep as is
                    segments.push(TransliterationSegment {
                        orig: current_segment.clone(),
                        kana: current_segment.clone(),
                        hira: current_segment.clone(),
                        hepburn: current_segment.clone(),
                    });
                }
                current_segment.clear();
            }
            
            current_segment.push(ch);
            is_kana_segment = ch_is_kana;
        }
        
        // Flush any remaining segment
        if !current_segment.is_empty() {
            if is_kana_segment {
                // Process kana segment
                let kana_text = if kana::is_hiragana(current_segment.chars().next().unwrap_or(' ')) {
                    kana::hiragana_to_katakana(&current_segment)
                } else {
                    current_segment.clone()
                };
                
                let hira_text = kana::katakana_to_hiragana(&kana_text);
                let hepburn_text = romaji::katakana_to_hepburn(&kana_text, use_macron);
                
                segments.push(TransliterationSegment {
                    orig: current_segment.clone(),
                    kana: kana_text,
                    hira: hira_text,
                    hepburn: hepburn_text,
                });
            } else {
                // Non-kana segment - keep as is
                segments.push(TransliterationSegment {
                    orig: current_segment.clone(),
                    kana: current_segment.clone(),
                    hira: current_segment.clone(),
                    hepburn: current_segment.clone(),
                });
            }
        }
        
        Ok(segments)
    }
}

impl Default for Transliterator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transliterator_creation() {
        let transliterator = Transliterator::new();
        let settings = transliterator.get_settings();
        assert!(!settings.use_macron);
    }

    #[test]
    fn test_settings_update() {
        let transliterator = Transliterator::new();
        transliterator.update_settings(TransliterationSettings { use_macron: true });
        let settings = transliterator.get_settings();
        assert!(settings.use_macron);
    }

    #[test]
    fn test_analyze_returns_original_on_error() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("test");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_analyze_hiragana() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("ひらがな");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].orig, "ひらがな");
        assert_eq!(result[0].hira, "ひらがな");
        assert_eq!(result[0].kana, "ヒラガナ");
        assert_eq!(result[0].hepburn, "hiragana");
    }

    #[test]
    fn test_analyze_katakana() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("カタカナ");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].orig, "カタカナ");
        assert_eq!(result[0].hira, "かたかな");
        assert_eq!(result[0].kana, "カタカナ");
        assert_eq!(result[0].hepburn, "katakana");
    }

    #[test]
    fn test_analyze_mixed_content() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("こんにちはworld");
        assert!(result.len() >= 2);
        // First segment should be kana
        assert_eq!(result[0].orig, "こんにちは");
        assert_eq!(result[0].hepburn, "konnichiha");
        // Second segment should be non-kana
        assert_eq!(result[1].orig, "world");
        assert_eq!(result[1].hepburn, "world");
    }

    #[test]
    fn test_analyze_with_macron() {
        let transliterator = Transliterator::new();
        transliterator.update_settings(TransliterationSettings { use_macron: true });
        let result = transliterator.analyze("トウキョウ");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].hepburn, "tōkyō");
    }

    #[test]
    fn test_analyze_empty_string() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_analyze_non_japanese() {
        let transliterator = Transliterator::new();
        let result = transliterator.analyze("Hello World");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].orig, "Hello World");
        assert_eq!(result[0].hepburn, "Hello World");
    }
}

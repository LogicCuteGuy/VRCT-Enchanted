// Language mappings for translation
//
// This module provides language code mappings for all supported translation engines.
// It loads language definitions from a YAML file and provides methods to:
// - Get language codes for specific translators
// - Validate source and target languages
// - Get lists of supported languages
//
// The language mappings support multiple translation backends including:
// - DeepL and DeepL API (with English/Portuguese variants)
// - Google Translate
// - Bing Translator
// - Papago
// - CTranslate2 (with multiple model variants)
// - Plamo API
// - Gemini API
// - OpenAI API
// - LM Studio
// - Ollama

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Once;
use std::sync::RwLock;

use crate::utils::error::{Result, VrctError};

/// Language mapping structure for a single backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageMapping {
    pub source: HashMap<String, String>,
    pub target: HashMap<String, String>,
}

/// CTranslate2 model-specific language mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTranslate2Mapping {
    #[serde(rename = "m2m100_418M-ct2-int8")]
    pub m2m100_418m: LanguageMapping,
    #[serde(rename = "m2m100_1.2B-ct2-int8")]
    pub m2m100_1_2b: LanguageMapping,
    #[serde(rename = "nllb-200-distilled-1.3B-ct2-int8")]
    pub nllb_200_distilled_1_3b: LanguageMapping,
    #[serde(rename = "nllb-200-3.3B-ct2-int8")]
    pub nllb_200_3_3b: LanguageMapping,
}

/// Root structure for all translation language mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationLanguages {
    #[serde(rename = "DeepL")]
    pub deepl: LanguageMapping,
    #[serde(rename = "DeepL_API")]
    pub deepl_api: LanguageMapping,
    #[serde(rename = "Google")]
    pub google: LanguageMapping,
    #[serde(rename = "Bing")]
    pub bing: LanguageMapping,
    #[serde(rename = "Papago")]
    pub papago: LanguageMapping,
    #[serde(rename = "CTranslate2")]
    pub ctranslate2: CTranslate2Mapping,
    #[serde(rename = "Plamo_API")]
    pub plamo_api: LanguageMapping,
    #[serde(rename = "Gemini_API")]
    pub gemini_api: LanguageMapping,
    #[serde(rename = "OpenAI_API")]
    pub openai_api: LanguageMapping,
    #[serde(rename = "LMStudio")]
    pub lmstudio: LanguageMapping,
    #[serde(rename = "Ollama")]
    pub ollama: LanguageMapping,
}

static TRANSLATION_LANGUAGES: RwLock<Option<TranslationLanguages>> = RwLock::new(None);
static INIT: Once = Once::new();

impl TranslationLanguages {
    /// Load translation language mappings from embedded YAML
    pub fn load() -> Result<&'static Self> {
        INIT.call_once(|| {
            // Embed the YAML file at compile time
            const LANGUAGES_YAML: &str = include_str!("../../resources/languages/languages.yml");
            
            match serde_yaml::from_str(LANGUAGES_YAML) {
                Ok(languages) => {
                    let mut lock = TRANSLATION_LANGUAGES.write().unwrap();
                    *lock = Some(languages);
                }
                Err(e) => {
                    eprintln!("Failed to parse language mappings: {}", e);
                }
            }
        });

        // Safety: After INIT.call_once, the value is guaranteed to be initialized
        let lock = TRANSLATION_LANGUAGES.read().unwrap();
        match lock.as_ref() {
            Some(languages) => {
                // This is safe because we never modify the value after initialization
                // and the reference lives as long as the static
                unsafe { Ok(&*(languages as *const TranslationLanguages)) }
            }
            None => Err(VrctError::Translation(
                "Failed to load translation languages".to_string(),
            )),
        }
    }

    /// Get supported source languages for a specific translator
    pub fn get_supported_source_languages(
        translator_name: &str,
        weight_type: Option<&str>,
    ) -> Result<Vec<String>> {
        let languages = Self::load()?;
        
        let mapping = match translator_name {
            "DeepL_API" => &languages.deepl_api.source,
            "DeepL" => &languages.deepl.source,
            "Google" => &languages.google.source,
            "Bing" => &languages.bing.source,
            "Papago" => &languages.papago.source,
            "Plamo_API" => &languages.plamo_api.source,
            "Gemini_API" => &languages.gemini_api.source,
            "OpenAI_API" => &languages.openai_api.source,
            "LMStudio" => &languages.lmstudio.source,
            "Ollama" => &languages.ollama.source,
            "CTranslate2" => {
                let weight_type = weight_type.ok_or_else(|| {
                    VrctError::Translation("Weight type required for CTranslate2".to_string())
                })?;
                
                let model_mapping = match weight_type {
                    "m2m100_418M-ct2-int8" => &languages.ctranslate2.m2m100_418m.source,
                    "m2m100_1.2B-ct2-int8" => &languages.ctranslate2.m2m100_1_2b.source,
                    "nllb-200-distilled-1.3B-ct2-int8" => {
                        &languages.ctranslate2.nllb_200_distilled_1_3b.source
                    }
                    "nllb-200-3.3B-ct2-int8" => &languages.ctranslate2.nllb_200_3_3b.source,
                    _ => {
                        return Err(VrctError::Translation(format!(
                            "Unknown CTranslate2 weight type: {}",
                            weight_type
                        )))
                    }
                };
                model_mapping
            }
            _ => {
                return Err(VrctError::Translation(format!(
                    "Unknown translator: {}",
                    translator_name
                )))
            }
        };
        
        Ok(mapping.keys().cloned().collect())
    }

    /// Get supported target languages for a specific translator
    pub fn get_supported_target_languages(
        translator_name: &str,
        weight_type: Option<&str>,
    ) -> Result<Vec<String>> {
        let languages = Self::load()?;
        
        let mapping = match translator_name {
            "DeepL_API" => &languages.deepl_api.target,
            "DeepL" => &languages.deepl.target,
            "Google" => &languages.google.target,
            "Bing" => &languages.bing.target,
            "Papago" => &languages.papago.target,
            "Plamo_API" => &languages.plamo_api.target,
            "Gemini_API" => &languages.gemini_api.target,
            "OpenAI_API" => &languages.openai_api.target,
            "LMStudio" => &languages.lmstudio.target,
            "Ollama" => &languages.ollama.target,
            "CTranslate2" => {
                let weight_type = weight_type.ok_or_else(|| {
                    VrctError::Translation("Weight type required for CTranslate2".to_string())
                })?;
                
                let model_mapping = match weight_type {
                    "m2m100_418M-ct2-int8" => &languages.ctranslate2.m2m100_418m.target,
                    "m2m100_1.2B-ct2-int8" => &languages.ctranslate2.m2m100_1_2b.target,
                    "nllb-200-distilled-1.3B-ct2-int8" => {
                        &languages.ctranslate2.nllb_200_distilled_1_3b.target
                    }
                    "nllb-200-3.3B-ct2-int8" => &languages.ctranslate2.nllb_200_3_3b.target,
                    _ => {
                        return Err(VrctError::Translation(format!(
                            "Unknown CTranslate2 weight type: {}",
                            weight_type
                        )))
                    }
                };
                model_mapping
            }
            _ => {
                return Err(VrctError::Translation(format!(
                    "Unknown translator: {}",
                    translator_name
                )))
            }
        };
        
        Ok(mapping.keys().cloned().collect())
    }

    /// Validate if a source language is supported by a translator
    pub fn is_valid_source_language(
        translator_name: &str,
        weight_type: Option<&str>,
        language: &str,
    ) -> Result<bool> {
        let languages = Self::load()?;
        
        let mapping = match translator_name {
            "DeepL_API" => &languages.deepl_api.source,
            "DeepL" => &languages.deepl.source,
            "Google" => &languages.google.source,
            "Bing" => &languages.bing.source,
            "Papago" => &languages.papago.source,
            "Plamo_API" => &languages.plamo_api.source,
            "Gemini_API" => &languages.gemini_api.source,
            "OpenAI_API" => &languages.openai_api.source,
            "LMStudio" => &languages.lmstudio.source,
            "Ollama" => &languages.ollama.source,
            "CTranslate2" => {
                let weight_type = weight_type.ok_or_else(|| {
                    VrctError::Translation("Weight type required for CTranslate2".to_string())
                })?;
                
                let model_mapping = match weight_type {
                    "m2m100_418M-ct2-int8" => &languages.ctranslate2.m2m100_418m.source,
                    "m2m100_1.2B-ct2-int8" => &languages.ctranslate2.m2m100_1_2b.source,
                    "nllb-200-distilled-1.3B-ct2-int8" => {
                        &languages.ctranslate2.nllb_200_distilled_1_3b.source
                    }
                    "nllb-200-3.3B-ct2-int8" => &languages.ctranslate2.nllb_200_3_3b.source,
                    _ => {
                        return Err(VrctError::Translation(format!(
                            "Unknown CTranslate2 weight type: {}",
                            weight_type
                        )))
                    }
                };
                model_mapping
            }
            _ => {
                return Err(VrctError::Translation(format!(
                    "Unknown translator: {}",
                    translator_name
                )))
            }
        };
        
        Ok(mapping.contains_key(language))
    }

    /// Validate if a target language is supported by a translator
    pub fn is_valid_target_language(
        translator_name: &str,
        weight_type: Option<&str>,
        language: &str,
    ) -> Result<bool> {
        let languages = Self::load()?;
        
        let mapping = match translator_name {
            "DeepL_API" => &languages.deepl_api.target,
            "DeepL" => &languages.deepl.target,
            "Google" => &languages.google.target,
            "Bing" => &languages.bing.target,
            "Papago" => &languages.papago.target,
            "Plamo_API" => &languages.plamo_api.target,
            "Gemini_API" => &languages.gemini_api.target,
            "OpenAI_API" => &languages.openai_api.target,
            "LMStudio" => &languages.lmstudio.target,
            "Ollama" => &languages.ollama.target,
            "CTranslate2" => {
                let weight_type = weight_type.ok_or_else(|| {
                    VrctError::Translation("Weight type required for CTranslate2".to_string())
                })?;
                
                let model_mapping = match weight_type {
                    "m2m100_418M-ct2-int8" => &languages.ctranslate2.m2m100_418m.target,
                    "m2m100_1.2B-ct2-int8" => &languages.ctranslate2.m2m100_1_2b.target,
                    "nllb-200-distilled-1.3B-ct2-int8" => {
                        &languages.ctranslate2.nllb_200_distilled_1_3b.target
                    }
                    "nllb-200-3.3B-ct2-int8" => &languages.ctranslate2.nllb_200_3_3b.target,
                    _ => {
                        return Err(VrctError::Translation(format!(
                            "Unknown CTranslate2 weight type: {}",
                            weight_type
                        )))
                    }
                };
                model_mapping
            }
            _ => {
                return Err(VrctError::Translation(format!(
                    "Unknown translator: {}",
                    translator_name
                )))
            }
        };
        
        Ok(mapping.contains_key(language))
    }

    /// Validate a language pair for a specific translator
    pub fn validate_language_pair(
        translator_name: &str,
        weight_type: Option<&str>,
        source_language: &str,
        target_language: &str,
    ) -> Result<()> {
        if !Self::is_valid_source_language(translator_name, weight_type, source_language)? {
            return Err(VrctError::Translation(format!(
                "Source language '{}' is not supported by {}",
                source_language, translator_name
            )));
        }
        
        if !Self::is_valid_target_language(translator_name, weight_type, target_language)? {
            return Err(VrctError::Translation(format!(
                "Target language '{}' is not supported by {}",
                target_language, translator_name
            )));
        }
        
        Ok(())
    }

    /// Get language code for a specific translator
    pub fn get_language_code(
        translator_name: &str,
        weight_type: Option<&str>,
        target_country: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<(String, String)> {
        let languages = Self::load()?;
        
        match translator_name {
            "DeepL_API" => {
                let mut target_lang = target_language.to_string();
                
                // Handle English variants
                if target_language == "English" {
                    target_lang = if matches!(
                        target_country,
                        "United States" | "Canada" | "Philippines"
                    ) {
                        "English (American)".to_string()
                    } else {
                        "English (British)".to_string()
                    };
                }
                // Handle Portuguese variants
                else if target_language == "Portuguese" {
                    target_lang = if target_country == "Portugal" {
                        "Portuguese (European)".to_string()
                    } else {
                        "Portuguese (Brazilian)".to_string()
                    };
                }
                
                let source_code = languages
                    .deepl_api
                    .source
                    .get(source_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Source language '{}' not found for DeepL_API",
                            source_language
                        ))
                    })?;
                
                let target_code = languages
                    .deepl_api
                    .target
                    .get(&target_lang)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Target language '{}' not found for DeepL_API",
                            target_lang
                        ))
                    })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "CTranslate2" => {
                let weight_type = weight_type.ok_or_else(|| {
                    VrctError::Translation("Weight type required for CTranslate2".to_string())
                })?;
                
                let model_mapping = match weight_type {
                    "m2m100_418M-ct2-int8" => &languages.ctranslate2.m2m100_418m,
                    "m2m100_1.2B-ct2-int8" => &languages.ctranslate2.m2m100_1_2b,
                    "nllb-200-distilled-1.3B-ct2-int8" => {
                        &languages.ctranslate2.nllb_200_distilled_1_3b
                    }
                    "nllb-200-3.3B-ct2-int8" => &languages.ctranslate2.nllb_200_3_3b,
                    _ => {
                        return Err(VrctError::Translation(format!(
                            "Unknown CTranslate2 weight type: {}",
                            weight_type
                        )))
                    }
                };
                
                let source_code = model_mapping.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for CTranslate2/{}",
                        source_language, weight_type
                    ))
                })?;
                
                let target_code = model_mapping.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for CTranslate2/{}",
                        target_language, weight_type
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "DeepL" => {
                let source_code = languages.deepl.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for DeepL",
                        source_language
                    ))
                })?;
                
                let target_code = languages.deepl.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for DeepL",
                        target_language
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Google" => {
                let source_code = languages.google.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for Google",
                        source_language
                    ))
                })?;
                
                let target_code = languages.google.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for Google",
                        target_language
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Bing" => {
                let source_code = languages.bing.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for Bing",
                        source_language
                    ))
                })?;
                
                let target_code = languages.bing.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for Bing",
                        target_language
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Papago" => {
                let source_code = languages.papago.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for Papago",
                        source_language
                    ))
                })?;
                
                let target_code = languages.papago.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for Papago",
                        target_language
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Plamo_API" => {
                let source_code = languages
                    .plamo_api
                    .source
                    .get(source_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Source language '{}' not found for Plamo_API",
                            source_language
                        ))
                    })?;
                
                let target_code = languages
                    .plamo_api
                    .target
                    .get(target_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Target language '{}' not found for Plamo_API",
                            target_language
                        ))
                    })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Gemini_API" => {
                let source_code = languages
                    .gemini_api
                    .source
                    .get(source_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Source language '{}' not found for Gemini_API",
                            source_language
                        ))
                    })?;
                
                let target_code = languages
                    .gemini_api
                    .target
                    .get(target_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Target language '{}' not found for Gemini_API",
                            target_language
                        ))
                    })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "OpenAI_API" => {
                let source_code = languages
                    .openai_api
                    .source
                    .get(source_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Source language '{}' not found for OpenAI_API",
                            source_language
                        ))
                    })?;
                
                let target_code = languages
                    .openai_api
                    .target
                    .get(target_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Target language '{}' not found for OpenAI_API",
                            target_language
                        ))
                    })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "LMStudio" => {
                let source_code = languages
                    .lmstudio
                    .source
                    .get(source_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Source language '{}' not found for LMStudio",
                            source_language
                        ))
                    })?;
                
                let target_code = languages
                    .lmstudio
                    .target
                    .get(target_language)
                    .ok_or_else(|| {
                        VrctError::Translation(format!(
                            "Target language '{}' not found for LMStudio",
                            target_language
                        ))
                    })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            "Ollama" => {
                let source_code = languages.ollama.source.get(source_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Source language '{}' not found for Ollama",
                        source_language
                    ))
                })?;
                
                let target_code = languages.ollama.target.get(target_language).ok_or_else(|| {
                    VrctError::Translation(format!(
                        "Target language '{}' not found for Ollama",
                        target_language
                    ))
                })?;
                
                Ok((source_code.clone(), target_code.clone()))
            }
            _ => Err(VrctError::Translation(format!(
                "Unknown translator: {}",
                translator_name
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_languages() {
        let languages = TranslationLanguages::load().expect("Failed to load languages");
        
        // Test that we can access some basic languages
        assert!(languages.deepl.source.contains_key("English"));
        assert!(languages.deepl.source.contains_key("Japanese"));
        assert!(languages.openai_api.source.contains_key("English"));
    }

    #[test]
    fn test_get_language_code_deepl_api() {
        let (source, target) = TranslationLanguages::get_language_code(
            "DeepL_API",
            None,
            "United States",
            "Japanese",
            "English",
        )
        .expect("Failed to get language codes");
        
        assert_eq!(source, "ja");
        assert_eq!(target, "en-US");
    }

    #[test]
    fn test_get_language_code_ctranslate2() {
        let (source, target) = TranslationLanguages::get_language_code(
            "CTranslate2",
            Some("m2m100_418M-ct2-int8"),
            "",
            "English",
            "Japanese",
        )
        .expect("Failed to get language codes");
        
        assert_eq!(source, "en");
        assert_eq!(target, "ja");
    }

    #[test]
    fn test_get_supported_source_languages() {
        let languages = TranslationLanguages::get_supported_source_languages("OpenAI_API", None)
            .expect("Failed to get supported languages");
        
        assert!(!languages.is_empty());
        assert!(languages.contains(&"English".to_string()));
        assert!(languages.contains(&"Japanese".to_string()));
    }

    #[test]
    fn test_get_supported_target_languages() {
        let languages = TranslationLanguages::get_supported_target_languages("Gemini_API", None)
            .expect("Failed to get supported languages");
        
        assert!(!languages.is_empty());
        assert!(languages.contains(&"English".to_string()));
        assert!(languages.contains(&"Japanese".to_string()));
    }

    #[test]
    fn test_get_supported_languages_ctranslate2() {
        let languages = TranslationLanguages::get_supported_source_languages(
            "CTranslate2",
            Some("m2m100_418M-ct2-int8"),
        )
        .expect("Failed to get supported languages");
        
        assert!(!languages.is_empty());
        assert!(languages.contains(&"English".to_string()));
    }

    #[test]
    fn test_is_valid_source_language() {
        assert!(TranslationLanguages::is_valid_source_language("OpenAI_API", None, "English")
            .expect("Failed to validate"));
        assert!(TranslationLanguages::is_valid_source_language("OpenAI_API", None, "Japanese")
            .expect("Failed to validate"));
        assert!(!TranslationLanguages::is_valid_source_language(
            "OpenAI_API",
            None,
            "NonexistentLanguage"
        )
        .expect("Failed to validate"));
    }

    #[test]
    fn test_is_valid_target_language() {
        assert!(TranslationLanguages::is_valid_target_language("DeepL_API", None, "English")
            .expect("Failed to validate"));
        assert!(TranslationLanguages::is_valid_target_language("DeepL_API", None, "Japanese")
            .expect("Failed to validate"));
        assert!(!TranslationLanguages::is_valid_target_language(
            "DeepL_API",
            None,
            "NonexistentLanguage"
        )
        .expect("Failed to validate"));
    }

    #[test]
    fn test_validate_language_pair_valid() {
        let result = TranslationLanguages::validate_language_pair(
            "OpenAI_API",
            None,
            "English",
            "Japanese",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_language_pair_invalid_source() {
        let result = TranslationLanguages::validate_language_pair(
            "OpenAI_API",
            None,
            "NonexistentLanguage",
            "Japanese",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source language"));
    }

    #[test]
    fn test_validate_language_pair_invalid_target() {
        let result = TranslationLanguages::validate_language_pair(
            "OpenAI_API",
            None,
            "English",
            "NonexistentLanguage",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Target language"));
    }

    #[test]
    fn test_validate_language_pair_ctranslate2() {
        let result = TranslationLanguages::validate_language_pair(
            "CTranslate2",
            Some("m2m100_418M-ct2-int8"),
            "English",
            "Japanese",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_translator() {
        let result = TranslationLanguages::get_supported_source_languages("UnknownTranslator", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown translator"));
    }

    #[test]
    fn test_ctranslate2_requires_weight_type() {
        let result = TranslationLanguages::get_supported_source_languages("CTranslate2", None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Weight type required"));
    }

    #[test]
    fn test_deepl_api_english_variants() {
        // Test US English
        let (_, target) = TranslationLanguages::get_language_code(
            "DeepL_API",
            None,
            "United States",
            "Japanese",
            "English",
        )
        .expect("Failed to get language codes");
        assert_eq!(target, "en-US");

        // Test British English
        let (_, target) = TranslationLanguages::get_language_code(
            "DeepL_API",
            None,
            "United Kingdom",
            "Japanese",
            "English",
        )
        .expect("Failed to get language codes");
        assert_eq!(target, "en-GB");
    }

    #[test]
    fn test_deepl_api_portuguese_variants() {
        // Test Brazilian Portuguese
        let (_, target) = TranslationLanguages::get_language_code(
            "DeepL_API",
            None,
            "Brazil",
            "English",
            "Portuguese",
        )
        .expect("Failed to get language codes");
        assert_eq!(target, "pt-BR");

        // Test European Portuguese
        let (_, target) = TranslationLanguages::get_language_code(
            "DeepL_API",
            None,
            "Portugal",
            "English",
            "Portuguese",
        )
        .expect("Failed to get language codes");
        assert_eq!(target, "pt-PT");
    }

    #[test]
    fn test_all_translators_have_mappings() {
        let languages = TranslationLanguages::load().expect("Failed to load languages");
        
        // Verify all translators have non-empty source and target mappings
        assert!(!languages.deepl.source.is_empty());
        assert!(!languages.deepl.target.is_empty());
        assert!(!languages.deepl_api.source.is_empty());
        assert!(!languages.deepl_api.target.is_empty());
        assert!(!languages.google.source.is_empty());
        assert!(!languages.google.target.is_empty());
        assert!(!languages.bing.source.is_empty());
        assert!(!languages.bing.target.is_empty());
        assert!(!languages.papago.source.is_empty());
        assert!(!languages.papago.target.is_empty());
        assert!(!languages.plamo_api.source.is_empty());
        assert!(!languages.plamo_api.target.is_empty());
        assert!(!languages.gemini_api.source.is_empty());
        assert!(!languages.gemini_api.target.is_empty());
        assert!(!languages.openai_api.source.is_empty());
        assert!(!languages.openai_api.target.is_empty());
        assert!(!languages.lmstudio.source.is_empty());
        assert!(!languages.lmstudio.target.is_empty());
        assert!(!languages.ollama.source.is_empty());
        assert!(!languages.ollama.target.is_empty());
        
        // Verify CTranslate2 models
        assert!(!languages.ctranslate2.m2m100_418m.source.is_empty());
        assert!(!languages.ctranslate2.m2m100_418m.target.is_empty());
        assert!(!languages.ctranslate2.m2m100_1_2b.source.is_empty());
        assert!(!languages.ctranslate2.m2m100_1_2b.target.is_empty());
        assert!(!languages.ctranslate2.nllb_200_distilled_1_3b.source.is_empty());
        assert!(!languages.ctranslate2.nllb_200_distilled_1_3b.target.is_empty());
        assert!(!languages.ctranslate2.nllb_200_3_3b.source.is_empty());
        assert!(!languages.ctranslate2.nllb_200_3_3b.target.is_empty());
    }
}

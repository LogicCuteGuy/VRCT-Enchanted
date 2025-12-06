// Translation engines module
// This module contains implementations for various translation engines

pub mod deepl;
pub mod openai;
pub mod gemini;
pub mod lmstudio;
pub mod ollama;
pub mod plamo;

pub use deepl::DeepLTranslator;
pub use openai::OpenAITranslator;
pub use gemini::GeminiTranslator;
pub use lmstudio::LMStudioTranslator;
pub use ollama::OllamaTranslator;
pub use plamo::PlamoTranslator;

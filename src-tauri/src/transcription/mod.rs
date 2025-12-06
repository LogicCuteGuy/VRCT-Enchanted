// Transcription subsystem module
// This module handles speech-to-text transcription using Whisper

pub mod whisper;
pub mod languages;

pub use whisper::{
    WhisperTranscriber, WhisperModelType, ComputeDevice,
    TranscriptionParams, TranscriptionResult,
    download_model, check_model_exists,
};
pub use languages::{
    LanguageMapping, get_transcription_languages,
    get_whisper_code, get_google_code,
};

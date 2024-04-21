use std::hash::{Hash as _, Hasher as _};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Language {
    pub code: String,
    pub display_name: String,
}

#[derive(Hash, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslationRequest {
    pub source_language_code: String,
    pub target_language_code: String,
    pub text: String,
}

impl TranslationRequest {
    pub fn generate_hash(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

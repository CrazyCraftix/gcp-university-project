#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Language {
    pub code: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslationRequest {
    pub source_language_code: String,
    pub target_language_code: String,
    pub text: String,
}

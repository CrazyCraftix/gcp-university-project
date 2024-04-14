#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Language {
    pub id: usize,
    pub display_string: String,
}

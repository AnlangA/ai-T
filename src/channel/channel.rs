#[derive(Debug, Clone)]
pub enum UiMessage {
    UpdateTranslation(String),
    Error(String),
    TranslationComplete,
}


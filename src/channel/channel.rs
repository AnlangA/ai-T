//! UI message channel for communicating between async tasks and the UI.
//!
//! This module defines message types used to communicate translation
//! progress and results from background tasks to the UI thread.

/// Messages sent from background tasks to the UI.
#[derive(Debug, Clone)]
pub enum UiMessage {
    /// A chunk of translation text has been received
    UpdateTranslation(String),
    /// An error occurred during translation
    Error(String),
    /// Translation has completed successfully
    TranslationComplete,
    /// Translation was cancelled by the user
    TranslationCancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_message_variants() {
        let msg1 = UiMessage::UpdateTranslation("test".to_string());
        assert!(matches!(msg1, UiMessage::UpdateTranslation(_)));

        let msg2 = UiMessage::Error("error".to_string());
        assert!(matches!(msg2, UiMessage::Error(_)));

        let msg3 = UiMessage::TranslationComplete;
        assert!(matches!(msg3, UiMessage::TranslationComplete));

        let msg4 = UiMessage::TranslationCancelled;
        assert!(matches!(msg4, UiMessage::TranslationCancelled));
    }

    #[test]
    fn test_ui_message_clone() {
        let msg = UiMessage::UpdateTranslation("test".to_string());
        let cloned = msg.clone();

        match (msg, cloned) {
            (UiMessage::UpdateTranslation(s1), UiMessage::UpdateTranslation(s2)) => {
                assert_eq!(s1, s2);
            }
            _ => panic!("Clone failed"),
        }
    }
}

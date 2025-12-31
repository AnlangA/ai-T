#[derive(Debug, Clone)]
pub enum UiMessage {
    UpdateTranslation(String),
    Error(String),
    TranslationComplete,
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

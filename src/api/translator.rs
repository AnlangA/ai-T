use crate::api::client::{ApiClient, ChatMessage};

pub struct Translator {
    client: ApiClient,
}

impl Translator {
    pub fn new(api_key: String) -> Self {
        Translator {
            client: ApiClient::new(api_key),
        }
    }

    pub fn translate(
        &self,
        text: String,
        target_language: String,
    ) -> tokio::sync::mpsc::UnboundedReceiver<
        Result<String, Box<dyn std::error::Error + Send + Sync>>,
    > {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let prompt = format!(
            "Translate the following text to {}. Only output the translation, nothing else:\n\n{}",
            target_language, text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let client = self.client.clone();

        tokio::spawn(async move {
            let mut stream_rx = client.stream_chat(messages).await;
            
            while let Some(result) = stream_rx.recv().await {
                let _ = tx.send(result);
            }
        });

        rx
    }
}

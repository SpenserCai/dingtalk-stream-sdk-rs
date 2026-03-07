//! 计算器机器人示例 - 对齐 Python calcbot
//!
//! 运行: cargo run --example calcbot

use async_trait::async_trait;
use dingtalk_stream::*;

struct CalcBotHandler {
    replier: ChatbotReplier,
}

#[async_trait]
impl CallbackHandler for CalcBotHandler {
    async fn process(&self, callback_message: &messages::frames::MessageBody) -> (u16, String) {
        let data: serde_json::Value =
            serde_json::from_str(&callback_message.data).unwrap_or_default();
        let incoming = ChatbotMessage::from_value(&data).unwrap_or_default();

        let expression = incoming
            .text
            .as_ref()
            .and_then(|t| t.content.as_deref())
            .unwrap_or("")
            .trim();

        let result = format!("Q: {expression}\nA: (expression received)");
        let _ = self.replier.reply_text(&result, &incoming).await;

        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let client_id =
        std::env::var("DINGTALK_CLIENT_ID").expect("DINGTALK_CLIENT_ID env var required");
    let client_secret =
        std::env::var("DINGTALK_CLIENT_SECRET").expect("DINGTALK_CLIENT_SECRET env var required");

    let credential = Credential::new(&client_id, &client_secret);
    let client_for_handler = DingTalkStreamClient::builder(credential.clone()).build();
    let replier = client_for_handler.chatbot_replier();

    let mut client = DingTalkStreamClient::builder(credential)
        .register_callback_handler(ChatbotMessage::TOPIC, CalcBotHandler { replier })
        .build();

    client.start_forever().unwrap();
}

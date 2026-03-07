//! Hello World 示例 - 最简单的聊天机器人
//!
//! 运行: cargo run --example helloworld

use async_trait::async_trait;
use dingtalk_stream::*;

struct MyHandler;

#[async_trait]
impl CallbackHandler for MyHandler {
    async fn process(&self, callback_message: &messages::frames::MessageBody) -> (u16, String) {
        let incoming = ChatbotMessage::from_value(
            &serde_json::from_str(&callback_message.data).unwrap_or_default(),
        );
        if let Ok(msg) = incoming {
            println!(
                "Received: type={:?}, sender={:?}",
                msg.message_type, msg.sender_nick
            );
        }
        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let client_id =
        std::env::var("DINGTALK_CLIENT_ID").expect("DINGTALK_CLIENT_ID env var required");
    let client_secret =
        std::env::var("DINGTALK_CLIENT_SECRET").expect("DINGTALK_CLIENT_SECRET env var required");

    let credential = Credential::new(client_id, client_secret);
    let mut client = DingTalkStreamClient::builder(credential)
        .register_callback_handler(ChatbotMessage::TOPIC, MyHandler)
        .build();

    client.start_forever().unwrap();
}

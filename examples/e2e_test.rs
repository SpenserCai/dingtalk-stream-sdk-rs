//! 端到端测试 - 启动机器人服务，等待用户通过钉钉发送消息进行测试
//!
//! 配置文件: examples/secret.toml (参考 examples/secret.example.toml)
//!
//! 运行: cargo run --example e2e_test
//!
//! 测试方式:
//! 1. 启动后，通过钉钉私聊机器人发送 "ping"，机器人回复 "pong"
//! 2. 通过群聊 @机器人 发送 "ping"，机器人回复 "pong"
//! 3. 发送 "echo <text>"，机器人回复相同文本
//! 4. 发送 "card"，机器人回复一个 Markdown 卡片
//! 5. 发送 "info"，机器人回复消息详情（用于验证字段解析）

use async_trait::async_trait;
use dingtalk_stream::*;
use std::collections::HashMap;

/// 从 TOML 配置文件读取凭证
fn load_config(path: &str) -> HashMap<String, String> {
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        panic!(
            "Failed to read config file: {path}\n\
             Please copy examples/secret.example.toml to examples/secret.toml and fill in your credentials."
        )
    });
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"');
            map.insert(key.trim().to_owned(), value.to_owned());
        }
    }
    map
}

/// 端到端测试 Handler
struct E2ETestHandler {
    replier: ChatbotReplier,
}

#[async_trait]
impl CallbackHandler for E2ETestHandler {
    async fn process(&self, callback_message: &messages::frames::MessageBody) -> (u16, String) {
        let data: serde_json::Value =
            serde_json::from_str(&callback_message.data).unwrap_or_default();
        let incoming = match ChatbotMessage::from_value(&data) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse ChatbotMessage: {e}");
                return (AckMessage::STATUS_OK, "OK".to_owned());
            }
        };

        let text = incoming
            .text
            .as_ref()
            .and_then(|t| t.content.as_deref())
            .unwrap_or("")
            .trim()
            .to_owned();

        let conv_type = incoming.conversation_type.as_deref().unwrap_or("unknown");
        println!(
            "[E2E] Received message: text='{text}', conv_type={conv_type}, sender={:?}",
            incoming.sender_nick
        );

        let result = match text.to_lowercase().as_str() {
            "ping" => self.replier.reply_text("pong", &incoming).await,
            t if t.starts_with("echo ") => {
                let echo_text = &text[5..];
                self.replier.reply_text(echo_text, &incoming).await
            }
            "card" => {
                match self
                    .replier
                    .reply_markdown_card(
                        "**E2E Test Card**\n\nThis is a test card from Rust SDK.",
                        &incoming,
                        "E2E Test",
                        "@lALPDfJ6V_FPDmvNAfTNAfQ",
                        false,
                        false,
                    )
                    .await
                {
                    Ok(instance) => {
                        println!(
                            "[E2E] Card sent, instance_id={:?}",
                            instance.card_instance_id
                        );
                        Ok(serde_json::json!({"status": "card sent"}))
                    }
                    Err(e) => Err(e),
                }
            }
            "info" => {
                let info = format!(
                    "**Message Info**\n\n\
                     - message_type: {:?}\n\
                     - sender_nick: {:?}\n\
                     - sender_id: {:?}\n\
                     - sender_staff_id: {:?}\n\
                     - conversation_type: {:?}\n\
                     - conversation_id: {:?}\n\
                     - conversation_title: {:?}\n\
                     - message_id: {:?}\n\
                     - robot_code: {:?}\n\
                     - is_admin: {:?}\n\
                     - at_users: {:?}",
                    incoming.message_type,
                    incoming.sender_nick,
                    incoming.sender_id,
                    incoming.sender_staff_id,
                    incoming.conversation_type,
                    incoming.conversation_id,
                    incoming.conversation_title,
                    incoming.message_id,
                    incoming.robot_code,
                    incoming.is_admin,
                    incoming.at_users,
                );
                self.replier
                    .reply_markdown("Message Info", &info, &incoming)
                    .await
            }
            _ => self
                .replier
                .reply_text(
                    &format!(
                        "E2E Test Bot - Commands: ping, echo <text>, card, info\nReceived: {text}"
                    ),
                    &incoming,
                )
                .await,
        };

        match result {
            Ok(_) => println!("[E2E] Reply sent successfully"),
            Err(e) => eprintln!("[E2E] Reply failed: {e}"),
        }

        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = load_config("examples/secret.toml");
    let client_id = config
        .get("client_id")
        .expect("client_id not found in examples/secret.toml");
    let client_secret = config
        .get("client_secret")
        .expect("client_secret not found in examples/secret.toml");

    println!("[E2E] Starting E2E test bot...");
    println!("[E2E] client_id: {client_id}");
    println!("[E2E] Commands: ping, echo <text>, card, info");
    println!("[E2E] Send messages to the bot via DingTalk to test.");

    let credential = Credential::new(client_id, client_secret);

    let temp_client = DingTalkStreamClient::builder(credential.clone()).build();
    let replier = temp_client.chatbot_replier();

    let handler = E2ETestHandler { replier };

    let mut client = DingTalkStreamClient::builder(credential)
        .register_callback_handler(ChatbotMessage::TOPIC, handler)
        .build();

    client.start_forever().unwrap();
}

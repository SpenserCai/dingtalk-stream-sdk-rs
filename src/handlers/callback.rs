//! 回调消息处理器，对齐 Python handlers.py CallbackHandler

use crate::messages::frames::{AckMessage, Headers, MessageBody};
use async_trait::async_trait;

/// 回调消息处理器 trait
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// 处理回调消息，返回 (状态码, 响应消息)
    async fn process(&self, callback_message: &MessageBody) -> (u16, String) {
        let _ = callback_message;
        (AckMessage::STATUS_NOT_IMPLEMENT, "not implement".to_owned())
    }

    /// 启动前的初始化
    fn pre_start(&self) {}

    /// 内部处理方法，封装 ACK 构建逻辑
    async fn raw_process(&self, callback_message: &MessageBody) -> AckMessage {
        let (code, message) = self.process(callback_message).await;
        AckMessage {
            code,
            headers: Headers {
                message_id: callback_message.headers.message_id.clone(),
                content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
                ..Default::default()
            },
            message: String::new(),
            data: serde_json::to_string(&serde_json::json!({"response": message}))
                .unwrap_or_default(),
        }
    }
}

impl CallbackHandler for () {}

/// 卡片回调主题常量
pub const TOPIC_CARD_CALLBACK: &str = "/v1.0/card/instances/callback";

//! 事件消息处理器，对齐 Python handlers.py EventHandler

use crate::messages::frames::{AckMessage, Headers, MessageBody};
use async_trait::async_trait;

/// 事件消息处理器 trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件消息，返回 (状态码, 响应消息)
    async fn process(&self, event_message: &MessageBody) -> (u16, String) {
        let _ = event_message;
        (AckMessage::STATUS_NOT_IMPLEMENT, "not implement".to_owned())
    }

    /// 启动前的初始化
    fn pre_start(&self) {}

    /// 内部处理方法，封装 ACK 构建逻辑
    async fn raw_process(&self, event_message: &MessageBody) -> AckMessage {
        let (code, message) = self.process(event_message).await;
        AckMessage {
            code,
            headers: Headers {
                message_id: event_message.headers.message_id.clone(),
                content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
                ..Default::default()
            },
            message,
            data: event_message.data.clone(),
        }
    }
}

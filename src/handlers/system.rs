//! 系统消息处理器，对齐 Python handlers.py SystemHandler

use crate::messages::frames::{AckMessage, Headers, MessageBody};
use async_trait::async_trait;

/// 系统消息处理器 trait
#[async_trait]
pub trait SystemHandler: Send + Sync {
    /// 处理系统消息，返回 (状态码, 响应消息)
    async fn process(&self, system_message: &MessageBody) -> (u16, String) {
        let _ = system_message;
        (AckMessage::STATUS_OK, "OK".to_owned())
    }

    /// 启动前的初始化
    fn pre_start(&self) {}

    /// 内部处理方法，封装 ACK 构建逻辑
    async fn raw_process(&self, system_message: &MessageBody) -> AckMessage {
        let (code, message) = self.process(system_message).await;
        AckMessage {
            code,
            headers: Headers {
                message_id: system_message.headers.message_id.clone(),
                content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
                ..Default::default()
            },
            message,
            data: system_message.data.clone(),
        }
    }
}

/// 默认系统消息处理器
pub struct DefaultSystemHandler;

#[async_trait]
impl SystemHandler for DefaultSystemHandler {}

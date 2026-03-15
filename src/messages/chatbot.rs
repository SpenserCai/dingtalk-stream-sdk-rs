//! 聊天机器人消息类型，对齐 Python chatbot.py

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 聊天机器人消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatbotMessage {
    /// 是否在 @列表中
    #[serde(rename = "isInAtList", skip_serializing_if = "Option::is_none")]
    pub is_in_at_list: Option<bool>,
    /// Session Webhook URL
    #[serde(rename = "sessionWebhook", skip_serializing_if = "Option::is_none")]
    pub session_webhook: Option<String>,
    /// 发送者昵称
    #[serde(rename = "senderNick", skip_serializing_if = "Option::is_none")]
    pub sender_nick: Option<String>,
    /// 机器人代码
    #[serde(rename = "robotCode", skip_serializing_if = "Option::is_none")]
    pub robot_code: Option<String>,
    /// Session Webhook 过期时间
    #[serde(
        rename = "sessionWebhookExpiredTime",
        skip_serializing_if = "Option::is_none"
    )]
    pub session_webhook_expired_time: Option<i64>,
    /// 消息 ID
    #[serde(rename = "msgId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// 发送者 ID
    #[serde(rename = "senderId", skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    /// 机器人用户 ID
    #[serde(rename = "chatbotUserId", skip_serializing_if = "Option::is_none")]
    pub chatbot_user_id: Option<String>,
    /// 会话 ID
    #[serde(rename = "conversationId", skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    /// 是否管理员
    #[serde(rename = "isAdmin", skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    /// 创建时间
    #[serde(rename = "createAt", skip_serializing_if = "Option::is_none")]
    pub create_at: Option<i64>,
    /// 会话类型: "1"=单聊, "2"=群聊
    #[serde(rename = "conversationType", skip_serializing_if = "Option::is_none")]
    pub conversation_type: Option<String>,
    /// @的用户列表
    #[serde(rename = "atUsers", skip_serializing_if = "Option::is_none")]
    pub at_users: Option<Vec<AtUser>>,
    /// 机器人所属企业 ID
    #[serde(rename = "chatbotCorpId", skip_serializing_if = "Option::is_none")]
    pub chatbot_corp_id: Option<String>,
    /// 发送者所属企业 ID
    #[serde(rename = "senderCorpId", skip_serializing_if = "Option::is_none")]
    pub sender_corp_id: Option<String>,
    /// 会话标题
    #[serde(rename = "conversationTitle", skip_serializing_if = "Option::is_none")]
    pub conversation_title: Option<String>,
    /// 消息类型: text, picture, richText
    #[serde(rename = "msgtype", skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    /// 文本内容
    #[serde(rename = "text", skip_serializing_if = "Option::is_none")]
    pub text: Option<TextContent>,
    /// 发送者员工 ID
    #[serde(rename = "senderStaffId", skip_serializing_if = "Option::is_none")]
    pub sender_staff_id: Option<String>,
    /// 托管上下文
    #[serde(rename = "hostingContext", skip_serializing_if = "Option::is_none")]
    pub hosting_context: Option<HostingContext>,
    /// 会话消息上下文
    #[serde(
        rename = "conversationMsgContext",
        skip_serializing_if = "Option::is_none"
    )]
    pub conversation_msg_context: Option<Vec<ConversationMessage>>,
    /// 图片内容（从 content 字段解析，msgtype=picture 时）
    #[serde(skip)]
    pub image_content: Option<ImageContent>,
    /// 富文本内容（从 content 字段解析，msgtype=richText 时）
    #[serde(skip)]
    pub rich_text_content: Option<RichTextContent>,
    // ── Rust SDK exclusive: audio message support ────────────────────
    // NOTE: This field is Rust-SDK-only and does NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this field.
    /// 语音内容（从 content 字段解析，msgtype=audio 时，仅单聊支持）
    #[serde(skip)]
    pub audio_content: Option<AudioContent>,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl ChatbotMessage {
    /// 机器人消息回调主题
    pub const TOPIC: &'static str = "/v1.0/im/bot/messages/get";
    /// 机器人消息委托主题
    pub const DELEGATE_TOPIC: &'static str = "/v1.0/im/bot/messages/delegate";

    /// 从 JSON Value 构造（处理 content 字段的特殊解析逻辑）
    pub fn from_value(value: &serde_json::Value) -> crate::Result<Self> {
        let mut msg: Self = serde_json::from_value(value.clone())?;

        // 根据 msgtype 解析 content 字段
        if let Some(msg_type) = &msg.message_type {
            if let Some(content) = value.get("content") {
                match msg_type.as_str() {
                    "picture" => {
                        msg.image_content = serde_json::from_value(content.clone()).ok();
                    }
                    "richText" => {
                        msg.rich_text_content = serde_json::from_value(content.clone()).ok();
                    }
                    // Rust SDK exclusive: audio message parsing
                    "audio" => {
                        msg.audio_content = serde_json::from_value(content.clone()).ok();
                    }
                    _ => {}
                }
            }
        }

        Ok(msg)
    }

    /// 获取文本列表
    pub fn get_text_list(&self) -> Option<Vec<String>> {
        match self.message_type.as_deref() {
            Some("text") => self
                .text
                .as_ref()
                .and_then(|t| t.content.clone())
                .map(|c| vec![c]),
            Some("richText") => self.rich_text_content.as_ref().map(|rtc| {
                rtc.rich_text_list
                    .iter()
                    .filter_map(|item| item.get("text").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            }),
            // Rust SDK exclusive: extract recognition text from audio messages
            Some("audio") => self
                .audio_content
                .as_ref()
                .and_then(|ac| ac.recognition.clone())
                .map(|r| vec![r]),
            _ => None,
        }
    }

    /// 获取图片下载码列表
    pub fn get_image_list(&self) -> Option<Vec<String>> {
        match self.message_type.as_deref() {
            Some("picture") => self
                .image_content
                .as_ref()
                .and_then(|ic| ic.download_code.clone())
                .map(|dc| vec![dc]),
            Some("richText") => self.rich_text_content.as_ref().map(|rtc| {
                rtc.rich_text_list
                    .iter()
                    .filter_map(|item| {
                        item.get("downloadCode")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
                    .collect()
            }),
            _ => None,
        }
    }
}

impl std::fmt::Display for ChatbotMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChatbotMessage(message_type={:?}, text={:?}, sender_nick={:?}, conversation_title={:?})",
            self.message_type, self.text, self.sender_nick, self.conversation_title
        )
    }
}

/// @用户信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AtUser {
    /// 钉钉 ID
    #[serde(rename = "dingtalkId", skip_serializing_if = "Option::is_none")]
    pub dingtalk_id: Option<String>,
    /// 员工 ID
    #[serde(rename = "staffId", skip_serializing_if = "Option::is_none")]
    pub staff_id: Option<String>,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 文本内容
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextContent {
    /// 文本内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl std::fmt::Display for TextContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextContent(content={:?})", self.content)
    }
}

/// 图片内容
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageContent {
    /// 下载码
    #[serde(rename = "downloadCode", skip_serializing_if = "Option::is_none")]
    pub download_code: Option<String>,
}

// ── Rust SDK exclusive: AudioContent ─────────────────────────────────
// NOTE: This struct is Rust-SDK-only and does NOT exist in the official
// Python SDK.  When syncing features from the Python SDK, do NOT remove
// this struct.

/// 语音消息内容（仅单聊场景下机器人可接收）
///
/// 钉钉服务端会自动进行语音识别（STT），识别结果通过 `recognition` 字段返回。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioContent {
    /// 语音识别后的文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recognition: Option<String>,
    /// 语音文件下载码
    #[serde(rename = "downloadCode", skip_serializing_if = "Option::is_none")]
    pub download_code: Option<String>,
    /// 语音时长（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}

/// 富文本内容
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RichTextContent {
    /// 富文本列表
    #[serde(rename = "richText", default)]
    pub rich_text_list: Vec<serde_json::Value>,
}

/// 托管上下文
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostingContext {
    /// 用户 ID
    #[serde(rename = "userId")]
    pub user_id: String,
    /// 昵称
    pub nick: String,
}

/// 会话消息上下文
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// 已读状态
    #[serde(rename = "readStatus", default)]
    pub read_status: String,
    /// 发送者用户 ID
    #[serde(rename = "senderUserId", default)]
    pub sender_user_id: String,
    /// 发送时间
    #[serde(rename = "sendTime", default)]
    pub send_time: i64,
}

impl ConversationMessage {
    /// 消息是否被我已读
    pub fn read_by_me(&self) -> bool {
        self.read_status == "2"
    }
}

/// 构造指定单聊的 `ChatbotMessage`（用于主动发送卡片到单聊）
pub fn reply_specified_single_chat(user_id: &str, user_nickname: &str) -> ChatbotMessage {
    let value = serde_json::json!({
        "senderId": user_id,
        "senderStaffId": user_id,
        "senderNick": user_nickname,
        "conversationType": "1",
        "msgId": uuid::Uuid::new_v4().to_string(),
    });
    serde_json::from_value(value).unwrap_or_default()
}

/// 构造指定群聊的 `ChatbotMessage`（用于主动发送卡片到群聊）
pub fn reply_specified_group_chat(open_conversation_id: &str) -> ChatbotMessage {
    let value = serde_json::json!({
        "conversationId": open_conversation_id,
        "conversationType": "2",
        "msgId": uuid::Uuid::new_v4().to_string(),
    });
    serde_json::from_value(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatbot_message_text() {
        let json = serde_json::json!({
            "msgtype": "text",
            "text": {"content": "hello world"},
            "senderNick": "test_user",
            "conversationType": "1",
            "senderId": "user_001",
            "senderStaffId": "staff_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("text"));
        assert_eq!(
            msg.text.as_ref().and_then(|t| t.content.as_deref()),
            Some("hello world")
        );
        let texts = msg.get_text_list().unwrap();
        assert_eq!(texts, vec!["hello world"]);
    }

    #[test]
    fn test_chatbot_message_picture() {
        let json = serde_json::json!({
            "msgtype": "picture",
            "content": {"downloadCode": "dc_001"},
            "senderId": "user_001",
            "msgId": "msg_002"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("picture"));
        assert_eq!(
            msg.image_content
                .as_ref()
                .and_then(|ic| ic.download_code.as_deref()),
            Some("dc_001")
        );
        let images = msg.get_image_list().unwrap();
        assert_eq!(images, vec!["dc_001"]);
    }

    #[test]
    fn test_chatbot_message_rich_text() {
        let json = serde_json::json!({
            "msgtype": "richText",
            "content": {
                "richText": [
                    {"text": "line1"},
                    {"downloadCode": "img_001"},
                    {"text": "line2"}
                ]
            },
            "senderId": "user_001",
            "msgId": "msg_003"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let texts = msg.get_text_list().unwrap();
        assert_eq!(texts, vec!["line1", "line2"]);
        let images = msg.get_image_list().unwrap();
        assert_eq!(images, vec!["img_001"]);
    }

    #[test]
    fn test_reply_specified_single_chat() {
        let msg = reply_specified_single_chat("user_001", "Test User");
        assert_eq!(msg.sender_id.as_deref(), Some("user_001"));
        assert_eq!(msg.sender_staff_id.as_deref(), Some("user_001"));
        assert_eq!(msg.conversation_type.as_deref(), Some("1"));
        assert!(msg.message_id.is_some());
    }

    #[test]
    fn test_reply_specified_group_chat() {
        let msg = reply_specified_group_chat("conv_001");
        assert_eq!(msg.conversation_id.as_deref(), Some("conv_001"));
        assert_eq!(msg.conversation_type.as_deref(), Some("2"));
        assert!(msg.message_id.is_some());
    }

    #[test]
    fn test_conversation_message_read_by_me() {
        let msg = ConversationMessage {
            read_status: "2".to_owned(),
            sender_user_id: "user_001".to_owned(),
            send_time: 1_690_000_000,
        };
        assert!(msg.read_by_me());

        let msg2 = ConversationMessage {
            read_status: "1".to_owned(),
            ..Default::default()
        };
        assert!(!msg2.read_by_me());
    }

    #[test]
    fn test_at_user_serde() {
        let json = r#"{"dingtalkId":"dt_001","staffId":"staff_001","extra":"val"}"#;
        let user: AtUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.dingtalk_id.as_deref(), Some("dt_001"));
        assert_eq!(user.staff_id.as_deref(), Some("staff_001"));
        assert!(user.extensions.contains_key("extra"));
    }

    // ── Rust SDK exclusive: audio message tests ──────────────────────

    #[test]
    fn test_chatbot_message_audio() {
        let json = serde_json::json!({
            "msgtype": "audio",
            "content": {
                "duration": 4000,
                "downloadCode": "dc_audio_001",
                "recognition": "钉钉，让进步发生"
            },
            "senderId": "user_001",
            "senderStaffId": "staff_001",
            "conversationType": "1",
            "msgId": "msg_audio_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("audio"));
        let ac = msg.audio_content.as_ref().unwrap();
        assert_eq!(ac.recognition.as_deref(), Some("钉钉，让进步发生"));
        assert_eq!(ac.download_code.as_deref(), Some("dc_audio_001"));
        assert_eq!(ac.duration, Some(4000));
        // get_text_list should return recognition text
        let texts = msg.get_text_list().unwrap();
        assert_eq!(texts, vec!["钉钉，让进步发生"]);
    }

    #[test]
    fn test_chatbot_message_audio_no_recognition() {
        let json = serde_json::json!({
            "msgtype": "audio",
            "content": {
                "duration": 2000,
                "downloadCode": "dc_audio_002"
            },
            "senderId": "user_001",
            "msgId": "msg_audio_002"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("audio"));
        assert!(msg.audio_content.is_some());
        // No recognition → get_text_list returns None
        assert!(msg.get_text_list().is_none());
    }
}

//! 聊天机器人消息类型，对齐 Python chatbot.py

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deserializes an `Option<i64>` that may come as either a JSON number or a
/// JSON string (DingTalk API sometimes sends timestamps as strings).
fn deserialize_opt_i64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i64),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        Some(StringOrNumber::String(s)) => match s.parse::<i64>() {
            Ok(n) => Ok(Some(n)),
            Err(_) => Err(de::Error::custom(format!(
                "failed to parse '{}' as i64",
                s
            ))),
        },
    }
}


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
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_i64_from_string_or_number"
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
    #[serde(
        rename = "createAt",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_i64_from_string_or_number"
    )]
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
    // ── Rust SDK exclusive: file message support ─────────────────────
    // NOTE: This field is Rust-SDK-only and does NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this field.
    /// 文件内容（从 content 字段解析，msgtype=file 时，仅单聊支持）
    #[serde(skip)]
    pub file_content: Option<FileContent>,
    // ── Rust SDK exclusive: video message support ────────────────────
    // NOTE: This field is Rust-SDK-only and does NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this field.
    /// 视频内容（从 content 字段解析，msgtype=video 时，仅单聊支持）
    #[serde(skip)]
    pub video_content: Option<VideoContent>,
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
                    // Rust SDK exclusive: file message parsing
                    "file" => {
                        msg.file_content = serde_json::from_value(content.clone()).ok();
                    }
                    // Rust SDK exclusive: video message parsing
                    "video" => {
                        msg.video_content = serde_json::from_value(content.clone()).ok();
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

    // ── Rust SDK exclusive: get_all_download_codes ───────────────────
    // NOTE: This method is Rust-SDK-only and does NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this method.

    /// 获取所有媒体文件的下载码列表
    ///
    /// 返回 `(media_type, download_code)` 元组列表，覆盖 picture、richText 图片、
    /// audio、video、file 五种消息类型。
    pub fn get_all_download_codes(&self) -> Vec<(String, String)> {
        let mut codes = Vec::new();
        match self.message_type.as_deref() {
            Some("picture") => {
                if let Some(dc) = self
                    .image_content
                    .as_ref()
                    .and_then(|ic| ic.download_code.as_ref())
                {
                    codes.push(("picture".to_owned(), dc.clone()));
                }
            }
            Some("richText") => {
                if let Some(rtc) = &self.rich_text_content {
                    for item in &rtc.rich_text_list {
                        if let Some(dc) = item.get("downloadCode").and_then(|v| v.as_str()) {
                            codes.push(("picture".to_owned(), dc.to_owned()));
                        }
                    }
                }
            }
            Some("audio") => {
                if let Some(dc) = self
                    .audio_content
                    .as_ref()
                    .and_then(|ac| ac.download_code.as_ref())
                {
                    codes.push(("audio".to_owned(), dc.clone()));
                }
            }
            Some("file") => {
                if let Some(dc) = self
                    .file_content
                    .as_ref()
                    .and_then(|fc| fc.download_code.as_ref())
                {
                    codes.push(("file".to_owned(), dc.clone()));
                }
            }
            Some("video") => {
                if let Some(dc) = self
                    .video_content
                    .as_ref()
                    .and_then(|vc| vc.download_code.as_ref())
                {
                    codes.push(("video".to_owned(), dc.clone()));
                }
            }
            _ => {}
        }
        codes
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

// ── Rust SDK exclusive: FileContent ──────────────────────────────────
// NOTE: This struct is Rust-SDK-only and does NOT exist in the official
// Python SDK.  When syncing features from the Python SDK, do NOT remove
// this struct.

/// 文件消息内容（仅单聊场景下机器人可接收）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileContent {
    /// 文件下载码
    #[serde(rename = "downloadCode", skip_serializing_if = "Option::is_none")]
    pub download_code: Option<String>,
    /// 文件名
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

// ── Rust SDK exclusive: VideoContent ─────────────────────────────────
// NOTE: This struct is Rust-SDK-only and does NOT exist in the official
// Python SDK.  When syncing features from the Python SDK, do NOT remove
// this struct.

/// 视频消息内容（仅单聊场景下机器人可接收）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VideoContent {
    /// 视频文件下载码
    #[serde(rename = "downloadCode", skip_serializing_if = "Option::is_none")]
    pub download_code: Option<String>,
    /// 视频时长（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    /// 视频类型
    #[serde(rename = "videoType", skip_serializing_if = "Option::is_none")]
    pub video_type: Option<String>,
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

    // ── Rust SDK exclusive: file message tests ───────────────────────

    #[test]
    fn test_chatbot_message_file() {
        let json = serde_json::json!({
            "msgtype": "file",
            "content": {
                "downloadCode": "dc_file_001",
                "fileName": "report.pdf"
            },
            "senderId": "user_001",
            "conversationType": "1",
            "msgId": "msg_file_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("file"));
        let fc = msg.file_content.as_ref().unwrap();
        assert_eq!(fc.download_code.as_deref(), Some("dc_file_001"));
        assert_eq!(fc.file_name.as_deref(), Some("report.pdf"));
    }

    #[test]
    fn test_chatbot_message_file_partial() {
        let json = serde_json::json!({
            "msgtype": "file",
            "content": { "downloadCode": "dc_file_002" },
            "senderId": "user_001",
            "msgId": "msg_file_002"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let fc = msg.file_content.as_ref().unwrap();
        assert_eq!(fc.download_code.as_deref(), Some("dc_file_002"));
        assert!(fc.file_name.is_none());
    }

    // ── Rust SDK exclusive: video message tests ──────────────────────

    #[test]
    fn test_chatbot_message_video() {
        let json = serde_json::json!({
            "msgtype": "video",
            "content": {
                "downloadCode": "dc_video_001",
                "duration": 15000,
                "videoType": "mp4"
            },
            "senderId": "user_001",
            "conversationType": "1",
            "msgId": "msg_video_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.message_type.as_deref(), Some("video"));
        let vc = msg.video_content.as_ref().unwrap();
        assert_eq!(vc.download_code.as_deref(), Some("dc_video_001"));
        assert_eq!(vc.duration, Some(15000));
        assert_eq!(vc.video_type.as_deref(), Some("mp4"));
    }

    #[test]
    fn test_chatbot_message_video_partial() {
        let json = serde_json::json!({
            "msgtype": "video",
            "content": { "downloadCode": "dc_video_002" },
            "senderId": "user_001",
            "msgId": "msg_video_002"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let vc = msg.video_content.as_ref().unwrap();
        assert_eq!(vc.download_code.as_deref(), Some("dc_video_002"));
        assert!(vc.duration.is_none());
        assert!(vc.video_type.is_none());
    }

    // ── Rust SDK exclusive: get_all_download_codes tests ─────────────

    #[test]
    fn test_get_all_download_codes_file() {
        let json = serde_json::json!({
            "msgtype": "file",
            "content": { "downloadCode": "dc_001", "fileName": "test.pdf" },
            "senderId": "user_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let codes = msg.get_all_download_codes();
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0], ("file".to_owned(), "dc_001".to_owned()));
    }

    #[test]
    fn test_get_all_download_codes_picture() {
        let json = serde_json::json!({
            "msgtype": "picture",
            "content": { "downloadCode": "dc_pic_001" },
            "senderId": "user_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let codes = msg.get_all_download_codes();
        assert_eq!(codes, vec![("picture".to_owned(), "dc_pic_001".to_owned())]);
    }

    #[test]
    fn test_get_all_download_codes_video() {
        let json = serde_json::json!({
            "msgtype": "video",
            "content": { "downloadCode": "dc_vid_001", "duration": 5000 },
            "senderId": "user_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let codes = msg.get_all_download_codes();
        assert_eq!(codes, vec![("video".to_owned(), "dc_vid_001".to_owned())]);
    }

    #[test]
    fn test_get_all_download_codes_text_empty() {
        let json = serde_json::json!({
            "msgtype": "text",
            "text": { "content": "hello" },
            "senderId": "user_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert!(msg.get_all_download_codes().is_empty());
    }

    #[test]
    fn test_get_all_download_codes_rich_text() {
        let json = serde_json::json!({
            "msgtype": "richText",
            "content": {
                "richText": [
                    { "text": "hello" },
                    { "downloadCode": "dc_rt_001" },
                    { "downloadCode": "dc_rt_002" }
                ]
            },
            "senderId": "user_001",
            "msgId": "msg_001"
        });
        let msg = ChatbotMessage::from_value(&json).unwrap();
        let codes = msg.get_all_download_codes();
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[0].0, "picture");
        assert_eq!(codes[1].0, "picture");
    }

    // ── Timestamp string deserialization tests ──────────────────────

    #[test]
    fn test_chatbot_message_timestamp_as_number() {
        let json: serde_json::Value = serde_json::from_str(r#"{
            "msgtype": "text",
            "text": {"content": "hello"},
            "senderId": "user_001",
            "msgId": "msg_001",
            "sessionWebhookExpiredTime": 1690106592000,
            "createAt": 1690106592000
        }"#).unwrap();
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.session_webhook_expired_time, Some(1_690_106_592_000));
        assert_eq!(msg.create_at, Some(1_690_106_592_000));
    }

    #[test]
    fn test_chatbot_message_timestamp_as_string() {
        let json: serde_json::Value = serde_json::from_str(r#"{
            "msgtype": "text",
            "text": {"content": "hello"},
            "senderId": "user_001",
            "msgId": "msg_001",
            "sessionWebhookExpiredTime": "1690106592000",
            "createAt": "1690106592000"
        }"#).unwrap();
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.session_webhook_expired_time, Some(1_690_106_592_000));
        assert_eq!(msg.create_at, Some(1_690_106_592_000));
    }

    #[test]
    fn test_chatbot_message_timestamp_missing() {
        let json: serde_json::Value = serde_json::from_str(r#"{
            "msgtype": "text",
            "text": {"content": "hello"},
            "senderId": "user_001",
            "msgId": "msg_001"
        }"#).unwrap();
        let msg = ChatbotMessage::from_value(&json).unwrap();
        assert_eq!(msg.session_webhook_expired_time, None);
        assert_eq!(msg.create_at, None);
    }
}

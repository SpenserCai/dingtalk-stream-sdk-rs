//! 消息帧结构，对齐 Python frames.py

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


/// 消息头
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Headers {
    /// 应用 ID
    #[serde(rename = "appId", skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    /// 连接 ID
    #[serde(rename = "connectionId", skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    /// 内容类型
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// 消息 ID
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// 时间戳
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// 主题
    #[serde(rename = "topic", skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    // Event 专用字段
    /// 事件产生时间
    #[serde(
        rename = "eventBornTime",
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_i64_from_string_or_number"
    )]
    pub event_born_time: Option<i64>,
    /// 事件所属企业 ID
    #[serde(rename = "eventCorpId", skip_serializing_if = "Option::is_none")]
    pub event_corp_id: Option<String>,
    /// 事件 ID
    #[serde(rename = "eventId", skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// 事件类型
    #[serde(rename = "eventType", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// 事件统一应用 ID
    #[serde(rename = "eventUnifiedAppId", skip_serializing_if = "Option::is_none")]
    pub event_unified_app_id: Option<String>,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Headers {
    /// JSON 内容类型常量
    pub const CONTENT_TYPE_APPLICATION_JSON: &'static str = "application/json";
}

/// WebSocket 入站消息（使用 serde tag 区分类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamMessage {
    /// 事件消息
    #[serde(rename = "EVENT")]
    Event(MessageBody),
    /// 回调消息
    #[serde(rename = "CALLBACK")]
    Callback(MessageBody),
    /// 系统消息
    #[serde(rename = "SYSTEM")]
    System(MessageBody),
}

/// 消息体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    /// 协议版本
    #[serde(rename = "specVersion", default)]
    pub spec_version: String,
    /// 消息头
    pub headers: Headers,
    /// 消息数据（JSON 字符串）
    #[serde(default)]
    pub data: String,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// ACK 响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckMessage {
    /// 状态码
    pub code: u16,
    /// 响应头
    pub headers: Headers,
    /// 响应消息
    #[serde(default)]
    pub message: String,
    /// 响应数据（JSON 字符串）
    #[serde(default)]
    pub data: String,
}

impl AckMessage {
    /// 成功
    pub const STATUS_OK: u16 = 200;
    /// 请求错误
    pub const STATUS_BAD_REQUEST: u16 = 400;
    /// 未实现
    pub const STATUS_NOT_IMPLEMENT: u16 = 404;
    /// 系统异常
    pub const STATUS_SYSTEM_EXCEPTION: u16 = 500;

    /// 创建一个成功的 ACK 消息
    pub fn ok(message_id: Option<String>, data: serde_json::Value) -> Self {
        Self {
            code: Self::STATUS_OK,
            headers: Headers {
                message_id,
                content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
                ..Default::default()
            },
            message: "OK".to_owned(),
            data: serde_json::to_string(&data).unwrap_or_default(),
        }
    }

    /// 创建一个未实现的 ACK 消息
    pub fn not_implemented(message_id: Option<String>) -> Self {
        Self {
            code: Self::STATUS_NOT_IMPLEMENT,
            headers: Headers {
                message_id,
                content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
                ..Default::default()
            },
            message: "not implement".to_owned(),
            data: String::new(),
        }
    }
}

/// 系统消息常量
pub struct SystemMessage;

impl SystemMessage {
    /// 断开连接主题
    pub const TOPIC_DISCONNECT: &'static str = "disconnect";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_message_deserialize_event() {
        let json = r#"{
            "specVersion": "1.0",
            "type": "EVENT",
            "headers": {
                "appId": "test_app",
                "messageId": "msg_001",
                "topic": "/v1.0/im/bot/messages/get",
                "eventId": "evt_001",
                "eventType": "chat_update_title",
                "eventBornTime": 1690106592000,
                "eventCorpId": "corp_001"
            },
            "data": "{\"key\":\"value\"}"
        }"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match &msg {
            StreamMessage::Event(body) => {
                assert_eq!(body.spec_version, "1.0");
                assert_eq!(body.headers.app_id.as_deref(), Some("test_app"));
                assert_eq!(body.headers.event_id.as_deref(), Some("evt_001"));
                assert_eq!(
                    body.headers.event_type.as_deref(),
                    Some("chat_update_title")
                );
                assert_eq!(body.headers.event_born_time, Some(1_690_106_592_000));
                assert_eq!(body.data, r#"{"key":"value"}"#);
            }
            _ => panic!("expected Event"),
        }
    }

    #[test]
    fn test_stream_message_deserialize_callback() {
        let json = r#"{
            "specVersion": "1.0",
            "type": "CALLBACK",
            "headers": {
                "messageId": "msg_002",
                "topic": "/v1.0/im/bot/messages/get"
            },
            "data": "{\"text\":\"hello\"}"
        }"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, StreamMessage::Callback(_)));
    }

    #[test]
    fn test_stream_message_deserialize_system() {
        let json = r#"{
            "specVersion": "1.0",
            "type": "SYSTEM",
            "headers": {
                "topic": "disconnect"
            },
            "data": ""
        }"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match &msg {
            StreamMessage::System(body) => {
                assert_eq!(body.headers.topic.as_deref(), Some("disconnect"));
            }
            _ => panic!("expected System"),
        }
    }

    #[test]
    fn test_ack_message_serialize() {
        let ack = AckMessage::ok(
            Some("msg_001".to_owned()),
            serde_json::json!({"response": "ok"}),
        );
        let json = serde_json::to_value(&ack).unwrap();
        assert_eq!(json["code"], 200);
        assert_eq!(json["headers"]["messageId"], "msg_001");
        assert_eq!(json["headers"]["contentType"], "application/json");
        assert_eq!(json["message"], "OK");
    }

    #[test]
    fn test_ack_not_implemented() {
        let ack = AckMessage::not_implemented(Some("msg_002".to_owned()));
        assert_eq!(ack.code, AckMessage::STATUS_NOT_IMPLEMENT);
        assert_eq!(ack.message, "not implement");
    }

    #[test]
    fn test_headers_extensions() {
        let json = r#"{
            "appId": "test",
            "customField": "custom_value"
        }"#;
        let headers: Headers = serde_json::from_str(json).unwrap();
        assert_eq!(headers.app_id.as_deref(), Some("test"));
        assert_eq!(
            headers.extensions.get("customField"),
            Some(&serde_json::Value::String("custom_value".to_owned()))
        );
    }

    // ── Timestamp string deserialization tests ──────────────────────

    #[test]
    fn test_event_born_time_as_string() {
        let json = r#"{
            "specVersion": "1.0",
            "type": "EVENT",
            "headers": {
                "appId": "test_app",
                "messageId": "msg_001",
                "topic": "/v1.0/im/bot/messages/get",
                "eventId": "evt_001",
                "eventType": "chat_update_title",
                "eventBornTime": "1690106592000",
                "eventCorpId": "corp_001"
            },
            "data": "{}"
        }"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match &msg {
            StreamMessage::Event(body) => {
                assert_eq!(body.headers.event_born_time, Some(1_690_106_592_000));
            }
            _ => panic!("expected Event"),
        }
    }

    #[test]
    fn test_event_born_time_missing() {
        let json = r#"{
            "specVersion": "1.0",
            "type": "EVENT",
            "headers": {
                "messageId": "msg_001",
                "topic": "test"
            },
            "data": "{}"
        }"#;
        let msg: StreamMessage = serde_json::from_str(json).unwrap();
        match &msg {
            StreamMessage::Event(body) => {
                assert_eq!(body.headers.event_born_time, None);
            }
            _ => panic!("expected Event"),
        }
    }
}

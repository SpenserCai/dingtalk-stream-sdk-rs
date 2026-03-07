//! 卡片回调消息，对齐 Python card_callback.py

use serde::{Deserialize, Serialize};

/// 卡片回调路由主题
pub const CARD_CALLBACK_ROUTER_TOPIC: &str = "/v1.0/card/instances/callback";

/// 卡片回调消息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardCallbackMessage {
    /// 扩展信息
    #[serde(default)]
    pub extension: serde_json::Value,
    /// 企业 ID
    #[serde(rename = "corpId", default)]
    pub corp_id: String,
    /// 用户 ID
    #[serde(rename = "userId", default)]
    pub user_id: String,
    /// 卡片实例 ID
    #[serde(rename = "outTrackId", default)]
    pub card_instance_id: String,
    /// 回调内容
    #[serde(default)]
    pub content: serde_json::Value,
    /// 回调类型
    #[serde(rename = "type", default)]
    pub callback_type: String,
    /// 空间类型
    #[serde(rename = "spaceType", default)]
    pub space_type: String,
    /// 用户 ID 类型
    #[serde(rename = "userIdType", default)]
    pub user_id_type: i32,
    /// 空间 ID
    #[serde(rename = "spaceId", default)]
    pub space_id: String,
}

impl CardCallbackMessage {
    /// 从 JSON 字符串解析（处理 extension 和 content 的二次解析）
    pub fn from_json_str(data: &str) -> crate::Result<Self> {
        let raw: serde_json::Value = serde_json::from_str(data)?;
        Self::from_value(&raw)
    }

    /// 从 JSON Value 解析（处理 extension 和 content 的字符串→对象转换）
    pub fn from_value(value: &serde_json::Value) -> crate::Result<Self> {
        let mut msg: Self = serde_json::from_value(value.clone())?;

        // Python 中 extension 和 content 是 json.loads(value) 的结果
        if let Some(ext_str) = value.get("extension").and_then(|v| v.as_str()) {
            if let Ok(parsed) = serde_json::from_str(ext_str) {
                msg.extension = parsed;
            }
        }
        if let Some(content_str) = value.get("content").and_then(|v| v.as_str()) {
            if let Ok(parsed) = serde_json::from_str(content_str) {
                msg.content = parsed;
            }
        }

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_callback_message_from_value() {
        let json = serde_json::json!({
            "extension": "{\"key\":\"value\"}",
            "corpId": "corp_001",
            "userId": "user_001",
            "outTrackId": "card_001",
            "content": "{\"action\":\"click\"}",
            "type": "actionCallback",
            "spaceType": "IM_GROUP",
            "userIdType": 1,
            "spaceId": "space_001"
        });
        let msg = CardCallbackMessage::from_value(&json).unwrap();
        assert_eq!(msg.corp_id, "corp_001");
        assert_eq!(msg.user_id, "user_001");
        assert_eq!(msg.card_instance_id, "card_001");
        assert_eq!(msg.callback_type, "actionCallback");
        assert_eq!(msg.space_type, "IM_GROUP");
        assert_eq!(msg.user_id_type, 1);
        assert_eq!(msg.extension["key"], "value");
        assert_eq!(msg.content["action"], "click");
    }

    #[test]
    fn test_card_callback_router_topic() {
        assert_eq!(CARD_CALLBACK_ROUTER_TOPIC, "/v1.0/card/instances/callback");
    }
}

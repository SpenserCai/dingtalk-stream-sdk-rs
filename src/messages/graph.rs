//! Graph API 消息类型，对齐 Python graph.py

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Graph 消息常量
pub struct GraphMessage;

impl GraphMessage {
    /// Graph API 调用主题
    pub const TOPIC: &'static str = "/v1.0/graph/api/invoke";
}

/// 请求行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLine {
    /// HTTP 方法
    #[serde(default = "default_method")]
    pub method: String,
    /// 请求 URI
    #[serde(default = "default_uri")]
    pub uri: String,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Default for RequestLine {
    fn default() -> Self {
        Self {
            method: "GET".to_owned(),
            uri: "/".to_owned(),
            extensions: HashMap::new(),
        }
    }
}

fn default_method() -> String {
    "GET".to_owned()
}

fn default_uri() -> String {
    "/".to_owned()
}

/// 状态行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusLine {
    /// 状态码
    #[serde(default = "default_status_code")]
    pub code: u16,
    /// 原因短语
    #[serde(rename = "reasonPhrase", default = "default_reason")]
    pub reason_phrase: String,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Default for StatusLine {
    fn default() -> Self {
        Self {
            code: 200,
            reason_phrase: "OK".to_owned(),
            extensions: HashMap::new(),
        }
    }
}

fn default_status_code() -> u16 {
    200
}

fn default_reason() -> String {
    "OK".to_owned()
}

/// Graph 请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphRequest {
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 请求行
    #[serde(rename = "requestLine", default)]
    pub request_line: RequestLine,
    /// 请求头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Graph 响应
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphResponse {
    /// 响应体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 响应头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 状态行
    #[serde(rename = "statusLine", default)]
    pub status_line: StatusLine,
    /// 扩展字段
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_request_deserialize() {
        let json = r#"{
            "body": "{\"key\":\"value\"}",
            "requestLine": {"method": "POST", "uri": "/api/test"},
            "headers": {"Content-Type": "application/json"}
        }"#;
        let req: GraphRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.request_line.method, "POST");
        assert_eq!(req.request_line.uri, "/api/test");
        assert_eq!(req.headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_graph_response_serialize() {
        let resp = GraphResponse {
            body: Some(r#"{"result":"ok"}"#.to_owned()),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_owned(), "application/json".to_owned());
                h
            },
            status_line: StatusLine::default(),
            extensions: HashMap::new(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["statusLine"]["code"], 200);
        assert_eq!(json["statusLine"]["reasonPhrase"], "OK");
    }

    #[test]
    fn test_graph_message_topic() {
        assert_eq!(GraphMessage::TOPIC, "/v1.0/graph/api/invoke");
    }

    #[test]
    fn test_request_line_defaults() {
        let rl = RequestLine::default();
        assert_eq!(rl.method, "GET");
        assert_eq!(rl.uri, "/");
    }
}

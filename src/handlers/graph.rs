//! Graph API 处理器，对齐 Python graph.py GraphHandler

use crate::handlers::callback::CallbackHandler;
use crate::messages::graph::{GraphRequest, GraphResponse, StatusLine};
use crate::transport::http::HttpClient;
use async_trait::async_trait;
use std::collections::HashMap;

/// Graph API 处理器 trait
#[async_trait]
pub trait GraphHandler: CallbackHandler {
    /// 处理 Graph 请求
    async fn on_graph_request(&self, request: &GraphRequest) -> crate::Result<GraphResponse>;
}

/// Graph 回复工具
pub struct GraphReplier {
    http_client: HttpClient,
}

impl GraphReplier {
    /// Graph Markdown 模板 ID
    pub const MARKDOWN_TEMPLATE_ID: &'static str = "d28e2ac5-fb34-4d93-94bc-cf5c580c2d4f.schema";

    /// 创建新的 `GraphReplier`
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// 通过 webhook 回复 markdown
    pub async fn reply_markdown(
        &self,
        webhook: &str,
        content: &str,
    ) -> crate::Result<(u16, String)> {
        let payload = serde_json::json!({
            "contentType": "ai_card",
            "content": {
                "templateId": Self::MARKDOWN_TEMPLATE_ID,
                "cardData": {
                    "content": content,
                }
            }
        });

        self.http_client.post_json_raw(webhook, &payload).await
    }

    /// 构建成功响应
    pub fn success_response(payload: Option<serde_json::Value>) -> GraphResponse {
        let body = payload.unwrap_or_default();
        GraphResponse {
            body: Some(serde_json::to_string(&body).unwrap_or_default()),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_owned(), "application/json".to_owned());
                h
            },
            status_line: StatusLine {
                code: 200,
                reason_phrase: "OK".to_owned(),
                extensions: HashMap::new(),
            },
            extensions: HashMap::new(),
        }
    }
}

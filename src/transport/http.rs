//! HTTP 客户端封装

use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use url::form_urlencoded;

/// 默认 OpenAPI 端点
const DEFAULT_OPENAPI_ENDPOINT: &str = "https://api.dingtalk.com";
/// 文件上传端点
const UPLOAD_ENDPOINT: &str = "https://oapi.dingtalk.com";

/// HTTP 客户端
#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    openapi_endpoint: String,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new() -> Self {
        let openapi_endpoint = std::env::var("DINGTALK_OPENAPI_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_OPENAPI_ENDPOINT.to_owned());

        Self {
            client: reqwest::Client::new(),
            openapi_endpoint,
        }
    }

    /// 获取 OpenAPI 端点
    pub fn openapi_endpoint(&self) -> &str {
        &self.openapi_endpoint
    }

    /// 获取 User-Agent 字符串
    fn user_agent() -> String {
        format!(
            "DingTalkStream/1.0 SDK/{} Rust/{}",
            env!("CARGO_PKG_VERSION"),
            rustc_version()
        )
    }

    /// 构建带 access_token 的请求头
    fn build_headers(access_token: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(&Self::user_agent())
                .unwrap_or_else(|_| HeaderValue::from_static("DingTalkStream/1.0")),
        );
        if let Some(token) = access_token {
            if let Ok(val) = HeaderValue::from_str(token) {
                headers.insert("x-acs-dingtalk-access-token", val);
            }
        }
        headers
    }

    /// POST JSON 请求，返回解析后的 JSON
    pub async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
        access_token: Option<&str>,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(url)
            .headers(Self::build_headers(access_token))
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(Error::Connection(format!(
                "POST {} failed with status {}: {}",
                url, status, text
            )));
        }

        serde_json::from_str(&text).map_err(Error::Json)
    }

    /// PUT JSON 请求，返回解析后的 JSON
    pub async fn put_json(
        &self,
        url: &str,
        body: &serde_json::Value,
        access_token: Option<&str>,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .put(url)
            .headers(Self::build_headers(access_token))
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(Error::Connection(format!(
                "PUT {} failed with status {}: {}",
                url, status, text
            )));
        }

        serde_json::from_str(&text).or_else(|_| Ok(serde_json::Value::Null))
    }

    /// POST JSON 请求，返回原始 (status_code, body_text)
    pub async fn post_json_raw(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<(u16, String)> {
        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        let text = resp.text().await?;
        Ok((status, text))
    }

    /// GET 请求，返回字节内容
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.client.get(url).send().await?;
        resp.error_for_status_ref()
            .map_err(|e| Error::Http(e.without_url()))?;
        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// 上传文件到钉钉
    pub async fn upload_file(
        &self,
        access_token: &str,
        content: &[u8],
        filetype: &str,
        filename: &str,
        mimetype: &str,
    ) -> Result<String> {
        let encoded_token: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("access_token", access_token)
            .finish();
        let url = format!("{}/media/upload?{}", UPLOAD_ENDPOINT, encoded_token);

        let part = reqwest::multipart::Part::bytes(content.to_vec())
            .file_name(filename.to_owned())
            .mime_str(mimetype)
            .map_err(|e| Error::Handler(format!("invalid mime type: {e}")))?;

        let form = reqwest::multipart::Form::new()
            .text("type", filetype.to_owned())
            .part("media", part);

        let resp = self.client.post(&url).multipart(form).send().await?;

        let status = resp.status();
        let text = resp.text().await?;

        if status.as_u16() == 401 {
            return Err(Error::Auth("upload returned 401".to_owned()));
        }

        if !status.is_success() {
            return Err(Error::Connection(format!(
                "upload failed with status {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        json.get("media_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| Error::Handler(format!("upload failed, response: {text}")))
    }

    /// 发送原始 POST 请求（用于 open_connection）
    pub async fn post_raw(&self, url: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let headers = Self::build_headers(None);
        let resp = self
            .client
            .post(url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(Error::Connection(format!(
                "POST {} failed with status {}: {}",
                url, status, text
            )));
        }

        serde_json::from_str(&text).map_err(Error::Json)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取 Rust 版本字符串
fn rustc_version() -> &'static str {
    // 编译时确定的版本
    env!("CARGO_PKG_RUST_VERSION")
}

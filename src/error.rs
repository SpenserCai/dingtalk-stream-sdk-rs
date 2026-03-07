//! 统一错误类型

/// SDK 统一错误类型
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// WebSocket 连接错误
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// HTTP 请求错误
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 连接失败
    #[error("Connection failed: {0}")]
    Connection(String),

    /// 认证失败
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Handler 处理错误
    #[error("Handler error: {0}")]
    Handler(String),

    /// 卡片操作错误
    #[error("Card operation failed: {0}")]
    Card(String),
}

/// SDK 统一 Result 类型
pub type Result<T> = std::result::Result<T, Error>;

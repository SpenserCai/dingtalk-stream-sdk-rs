//! DingTalk Stream 客户端，对齐 Python stream.py

use crate::credential::Credential;
use crate::error::{Error, Result};
use crate::handlers::callback::CallbackHandler;
use crate::handlers::chatbot::{AsyncChatbotHandler, ChatbotReplier, async_raw_process};
use crate::handlers::event::EventHandler;
use crate::handlers::system::{DefaultSystemHandler, SystemHandler};
use crate::messages::frames::{AckMessage, StreamMessage, SystemMessage};
use crate::transport::http::HttpClient;
use crate::transport::token::TokenManager;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::form_urlencoded;

/// 回调处理器条目
enum CallbackEntry {
    Sync(Arc<dyn CallbackHandler>),
    Async(Arc<dyn AsyncChatbotHandler>),
}

/// DingTalk Stream 客户端
pub struct DingTalkStreamClient {
    credential: Credential,
    event_handler: Option<Arc<dyn EventHandler>>,
    callback_handlers: HashMap<String, CallbackEntry>,
    system_handler: Arc<dyn SystemHandler>,
    http_client: HttpClient,
    token_manager: Arc<TokenManager>,
    is_event_required: bool,
    pre_started: bool,
}

impl DingTalkStreamClient {
    /// 创建 Builder
    pub fn builder(credential: Credential) -> ClientBuilder {
        ClientBuilder::new(credential)
    }

    /// 获取 access_token
    pub async fn get_access_token(&self) -> Result<String> {
        self.token_manager.get_access_token().await
    }

    /// 重置 access_token 缓存
    pub async fn reset_access_token(&self) {
        self.token_manager.reset().await;
    }

    /// 创建 `ChatbotReplier`
    pub fn chatbot_replier(&self) -> ChatbotReplier {
        ChatbotReplier::new(
            self.http_client.clone(),
            Arc::clone(&self.token_manager),
            self.credential.client_id.clone(),
        )
    }

    /// 上传文件到钉钉
    pub async fn upload_to_dingtalk(
        &self,
        image_content: &[u8],
        filetype: &str,
        filename: &str,
        mimetype: &str,
    ) -> Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        let result = self
            .http_client
            .upload_file(&access_token, image_content, filetype, filename, mimetype)
            .await;

        if let Err(Error::Auth(_)) = &result {
            self.token_manager.reset().await;
        }

        result
    }

    /// 启动客户端（异步，永久重连循环）
    pub async fn start(&mut self) -> Result<()> {
        self.pre_start();

        loop {
            match self.run_once().await {
                Ok(()) => {
                    tracing::info!("connection closed, reconnecting in 3s...");
                }
                Err(e) => {
                    tracing::error!(error = %e, "connection error, reconnecting in 10s...");
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    continue;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    /// 同步启动（阻塞当前线程）
    pub fn start_forever(&mut self) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Connection(format!("failed to create runtime: {e}")))?;
        rt.block_on(self.start())
    }

    fn pre_start(&mut self) {
        if self.pre_started {
            return;
        }
        self.pre_started = true;

        if let Some(ref handler) = self.event_handler {
            handler.pre_start();
        }
        self.system_handler.pre_start();
        for entry in self.callback_handlers.values() {
            match entry {
                CallbackEntry::Sync(h) => h.pre_start(),
                CallbackEntry::Async(h) => h.pre_start(),
            }
        }
    }

    /// 单次连接运行
    async fn run_once(&self) -> Result<()> {
        let connection = self.open_connection().await?;

        let endpoint = connection
            .get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Connection("endpoint not found".to_owned()))?;
        let ticket = connection
            .get("ticket")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Connection("ticket not found".to_owned()))?;

        let encoded_ticket: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("ticket", ticket)
            .finish();
        let uri = format!("{}?{}", endpoint, encoded_ticket);

        tracing::info!(endpoint = %endpoint, "connecting to WebSocket");

        let (ws_stream, _) =
            tokio::time::timeout(std::time::Duration::from_secs(30), connect_async(&uri))
                .await
                .map_err(|_| Error::Connection("WebSocket connect timeout".to_string()))?
                .map_err(Error::WebSocket)?;
        let (write, read) = ws_stream.split();

        let write = Arc::new(tokio::sync::Mutex::new(write));
        let write_keepalive = Arc::clone(&write);

        // Keepalive task
        let keepalive_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let mut w = write_keepalive.lock().await;
                if w.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        });

        // 消息处理
        let mut read = read;
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    let route_result = self.route_message(&text).await;
                    match route_result {
                        Ok((ack_opt, should_disconnect)) => {
                            if let Some(ack) = ack_opt {
                                let ack_json = serde_json::to_string(&ack).unwrap_or_default();
                                let mut w = write.lock().await;
                                if let Err(e) = w.send(Message::Text(ack_json.into())).await {
                                    tracing::error!(error = %e, "failed to send ack");
                                    break;
                                }
                            }
                            if should_disconnect {
                                tracing::info!("received disconnect, closing connection");
                                let mut w = write.lock().await;
                                let _ = w.close().await;
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "route message failed");
                        }
                    }
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    tracing::error!(error = %e, "WebSocket read error");
                    break;
                }
                _ => {}
            }
        }

        keepalive_handle.abort();
        Ok(())
    }

    /// 路由消息
    async fn route_message(&self, raw: &str) -> Result<(Option<AckMessage>, bool)> {
        let msg: StreamMessage = serde_json::from_str(raw)?;
        let mut should_disconnect = false;

        let ack = match msg {
            StreamMessage::System(body) => {
                let ack = self.system_handler.raw_process(&body).await;
                if body.headers.topic.as_deref() == Some(SystemMessage::TOPIC_DISCONNECT) {
                    should_disconnect = true;
                    tracing::info!(
                        topic = ?body.headers.topic,
                        "received disconnect"
                    );
                } else {
                    tracing::warn!(
                        topic = ?body.headers.topic,
                        "unknown system message topic"
                    );
                }
                Some(ack)
            }
            StreamMessage::Event(body) => {
                if let Some(ref handler) = self.event_handler {
                    Some(handler.raw_process(&body).await)
                } else {
                    tracing::warn!("no event handler registered");
                    None
                }
            }
            StreamMessage::Callback(body) => {
                let topic = body.headers.topic.as_deref().unwrap_or("");
                if let Some(entry) = self.callback_handlers.get(topic) {
                    match entry {
                        CallbackEntry::Sync(handler) => Some(handler.raw_process(&body).await),
                        CallbackEntry::Async(handler) => {
                            Some(async_raw_process(Arc::clone(handler), body).await)
                        }
                    }
                } else {
                    tracing::warn!(topic = %topic, "unknown callback topic");
                    None
                }
            }
        };

        Ok((ack, should_disconnect))
    }

    /// 打开连接
    async fn open_connection(&self) -> Result<serde_json::Value> {
        let url = format!(
            "{}/v1.0/gateway/connections/open",
            self.http_client.openapi_endpoint()
        );

        tracing::info!(url = %url, "opening connection");

        let mut topics: Vec<serde_json::Value> = Vec::new();
        if self.is_event_required {
            topics.push(serde_json::json!({"type": "EVENT", "topic": "*"}));
        }
        for topic in self.callback_handlers.keys() {
            topics.push(serde_json::json!({"type": "CALLBACK", "topic": topic}));
        }

        let body = serde_json::json!({
            "clientId": self.credential.client_id,
            "clientSecret": self.credential.client_secret,
            "subscriptions": topics,
            "ua": format!("dingtalk-sdk-rust/v{}-union", env!("CARGO_PKG_VERSION")),
            "localIp": get_host_ip(),
        });

        self.http_client.post_raw(&url, &body).await
    }
}

/// 客户端构建器
pub struct ClientBuilder {
    credential: Credential,
    event_handler: Option<Arc<dyn EventHandler>>,
    callback_handlers: HashMap<String, CallbackEntry>,
    system_handler: Option<Arc<dyn SystemHandler>>,
    connect_timeout_secs: Option<u64>,
    request_timeout_secs: Option<u64>,
}

impl ClientBuilder {
    /// 创建新的构建器
    pub fn new(credential: Credential) -> Self {
        Self {
            credential,
            event_handler: None,
            callback_handlers: HashMap::new(),
            system_handler: None,
            connect_timeout_secs: None,
            request_timeout_secs: None,
        }
    }

    /// 注册事件处理器
    pub fn register_all_event_handler(mut self, handler: impl EventHandler + 'static) -> Self {
        self.event_handler = Some(Arc::new(handler));
        self
    }

    /// 注册回调处理器
    pub fn register_callback_handler(
        mut self,
        topic: &str,
        handler: impl CallbackHandler + 'static,
    ) -> Self {
        self.callback_handlers
            .insert(topic.to_owned(), CallbackEntry::Sync(Arc::new(handler)));
        self
    }

    /// 注册异步聊天机器人处理器
    pub fn register_async_chatbot_handler(
        mut self,
        topic: &str,
        handler: impl AsyncChatbotHandler + 'static,
    ) -> Self {
        self.callback_handlers
            .insert(topic.to_owned(), CallbackEntry::Async(Arc::new(handler)));
        self
    }

    /// 注册系统消息处理器
    pub fn register_system_handler(mut self, handler: impl SystemHandler + 'static) -> Self {
        self.system_handler = Some(Arc::new(handler));
        self
    }

    /// 设置 HTTP 连接超时（秒），默认 10s
    pub fn connect_timeout_secs(mut self, secs: u64) -> Self {
        self.connect_timeout_secs = Some(secs);
        self
    }

    /// 设置 HTTP 请求超时（秒），默认 30s
    pub fn request_timeout_secs(mut self, secs: u64) -> Self {
        self.request_timeout_secs = Some(secs);
        self
    }

    /// 构建客户端
    pub fn build(self) -> DingTalkStreamClient {
        let http_client = match (self.connect_timeout_secs, self.request_timeout_secs) {
            (None, None) => HttpClient::new(),
            (ct, rt) => HttpClient::with_timeout(ct.unwrap_or(10), rt.unwrap_or(30)),
        };
        let token_manager = Arc::new(TokenManager::new(
            self.credential.clone(),
            http_client.clone(),
        ));

        let is_event_required = self.event_handler.is_some();

        DingTalkStreamClient {
            credential: self.credential,
            event_handler: self.event_handler,
            callback_handlers: self.callback_handlers,
            system_handler: self
                .system_handler
                .unwrap_or_else(|| Arc::new(DefaultSystemHandler)),
            http_client,
            token_manager,
            is_event_required,
            pre_started: false,
        }
    }
}

/// 获取本机 IP 地址
fn get_host_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default()
}

//! 聊天机器人处理器，对齐 Python chatbot.py

use crate::card::instances::{
    AIMarkdownCardInstance, CarouselCardInstance, MarkdownButtonCardInstance, MarkdownCardInstance,
    RPAPluginCardInstance,
};
use crate::card::replier::{AICardReplier, CardReplier};
use crate::card::templates::generate_multi_text_line_card_data;
use crate::handlers::callback::CallbackHandler;
use crate::messages::chatbot::ChatbotMessage;
use crate::messages::frames::{AckMessage, Headers, MessageBody};
use crate::transport::http::HttpClient;
use crate::transport::token::TokenManager;
use async_trait::async_trait;
use std::sync::Arc;

/// 聊天机器人处理器 trait
#[async_trait]
pub trait ChatbotHandler: CallbackHandler {}

/// 异步聊天机器人处理器（立即 ACK + 后台处理）
pub trait AsyncChatbotHandler: Send + Sync + 'static {
    /// 用户实现此方法处理消息（非 async）
    fn process(&self, callback_message: &MessageBody);
    /// 启动前的初始化
    fn pre_start(&self) {}
}

/// 为 `AsyncChatbotHandler` 生成立即 ACK + 后台处理
pub(crate) async fn async_raw_process(
    handler: Arc<dyn AsyncChatbotHandler>,
    callback_message: MessageBody,
) -> AckMessage {
    let message_id = callback_message.headers.message_id.clone();
    let data = callback_message.data.clone();

    tokio::spawn(async move {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.process(&callback_message);
        }));
        if let Err(e) = result {
            tracing::error!("AsyncChatbotHandler.process panicked: {:?}", e);
        }
    });

    AckMessage {
        code: AckMessage::STATUS_OK,
        headers: Headers {
            message_id,
            content_type: Some(Headers::CONTENT_TYPE_APPLICATION_JSON.to_owned()),
            ..Default::default()
        },
        message: "ok".to_owned(),
        data,
    }
}

/// 聊天机器人回复工具
#[derive(Clone)]
pub struct ChatbotReplier {
    http_client: HttpClient,
    token_manager: Arc<TokenManager>,
    client_id: String,
}

impl ChatbotReplier {
    /// 创建新的 `ChatbotReplier`
    pub fn new(
        http_client: HttpClient,
        token_manager: Arc<TokenManager>,
        client_id: String,
    ) -> Self {
        Self {
            http_client,
            token_manager,
            client_id,
        }
    }

    /// 通过 session webhook 回复文本消息
    pub async fn reply_text(
        &self,
        text: &str,
        incoming_message: &ChatbotMessage,
    ) -> crate::Result<serde_json::Value> {
        let webhook = incoming_message
            .session_webhook
            .as_deref()
            .ok_or_else(|| crate::Error::Handler("session_webhook is empty".to_owned()))?;
        let body = serde_json::json!({
            "msgtype": "text",
            "text": {"content": text},
            "at": {"atUserIds": [incoming_message.sender_staff_id.as_deref().unwrap_or("")]}
        });
        self.http_client.post_json(webhook, &body, None).await
    }

    /// 通过 session webhook 回复 markdown 消息
    pub async fn reply_markdown(
        &self,
        title: &str,
        text: &str,
        incoming_message: &ChatbotMessage,
    ) -> crate::Result<serde_json::Value> {
        let webhook = incoming_message
            .session_webhook
            .as_deref()
            .ok_or_else(|| crate::Error::Handler("session_webhook is empty".to_owned()))?;
        let body = serde_json::json!({
            "msgtype": "markdown",
            "markdown": {"title": title, "text": text},
            "at": {"atUserIds": [incoming_message.sender_staff_id.as_deref().unwrap_or("")]}
        });
        self.http_client.post_json(webhook, &body, None).await
    }

    /// 发送互动卡片 (OpenAPI)
    pub async fn reply_card(
        &self,
        card_data: &serde_json::Value,
        incoming_message: &ChatbotMessage,
        at_sender: bool,
        at_all: bool,
    ) -> crate::Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        let card_biz_id = CardReplier::gen_card_id(incoming_message);
        let mut body = serde_json::json!({
            "cardTemplateId": "StandardCard",
            "robotCode": self.client_id,
            "cardData": serde_json::to_string(card_data).unwrap_or_default(),
            "sendOptions": {},
            "cardBizId": card_biz_id,
        });
        let Some(body_obj) = body.as_object_mut() else {
            return Ok(card_biz_id);
        };
        if incoming_message.conversation_type.as_deref() == Some("2") {
            body_obj.insert(
                "openConversationId".to_owned(),
                serde_json::json!(incoming_message.conversation_id),
            );
        } else if incoming_message.conversation_type.as_deref() == Some("1") {
            let receiver = serde_json::json!({"userId": incoming_message.sender_staff_id});
            body_obj.insert(
                "singleChatReceiver".to_owned(),
                serde_json::Value::String(serde_json::to_string(&receiver).unwrap_or_default()),
            );
        }
        if let Some(send_options) = body_obj
            .get_mut("sendOptions")
            .and_then(|v| v.as_object_mut())
        {
            send_options.insert("atAll".to_owned(), serde_json::json!(at_all));
            if at_sender {
                let user_list = serde_json::json!([{"nickName": incoming_message.sender_nick, "userId": incoming_message.sender_staff_id}]);
                send_options.insert(
                    "atUserListJson".to_owned(),
                    serde_json::Value::String(
                        serde_json::to_string(&user_list).unwrap_or_default(),
                    ),
                );
            }
        }
        let url = format!(
            "{}/v1.0/im/v1.0/robot/interactiveCards/send",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &body, Some(&access_token))
            .await?;
        Ok(card_biz_id)
    }

    /// 更新互动卡片
    pub async fn update_card(
        &self,
        card_biz_id: &str,
        card_data: &serde_json::Value,
    ) -> crate::Result<serde_json::Value> {
        let access_token = self.token_manager.get_access_token().await?;
        let body = serde_json::json!({"cardBizId": card_biz_id, "cardData": serde_json::to_string(card_data).unwrap_or_default()});
        let url = format!(
            "{}/v1.0/im/robots/interactiveCards",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .put_json(&url, &body, Some(&access_token))
            .await
    }

    fn make_card_replier(&self, incoming_message: &ChatbotMessage) -> CardReplier {
        CardReplier::new(
            self.http_client.clone(),
            Arc::clone(&self.token_manager),
            self.client_id.clone(),
            incoming_message.clone(),
        )
    }

    fn make_ai_card_replier(&self, incoming_message: &ChatbotMessage) -> AICardReplier {
        AICardReplier::new(self.make_card_replier(incoming_message))
    }

    /// 创建 AI 卡片回复器，用于自定义模板场景
    pub fn create_ai_card_replier(&self, incoming_message: &ChatbotMessage) -> AICardReplier {
        self.make_ai_card_replier(incoming_message)
    }

    /// 回复 Markdown 卡片
    pub async fn reply_markdown_card(
        &self,
        markdown: &str,
        incoming_message: &ChatbotMessage,
        title: &str,
        logo: &str,
        at_sender: bool,
        at_all: bool,
    ) -> crate::Result<MarkdownCardInstance> {
        let mut instance = MarkdownCardInstance::new(self.make_card_replier(incoming_message));
        instance.set_title_and_logo(title, logo);
        instance
            .reply(markdown, at_sender, at_all, None, true)
            .await?;
        Ok(instance)
    }

    /// 回复 RPA 插件卡片
    #[allow(clippy::too_many_arguments)]
    pub async fn reply_rpa_plugin_card(
        &self,
        incoming_message: &ChatbotMessage,
        plugin_id: &str,
        plugin_version: &str,
        plugin_name: &str,
        ability_name: &str,
        plugin_args: &serde_json::Value,
        goal: &str,
        corp_id: &str,
        recipients: Option<&[String]>,
    ) -> crate::Result<RPAPluginCardInstance> {
        let mut instance = RPAPluginCardInstance::new(self.make_ai_card_replier(incoming_message));
        instance.set_goal(goal);
        instance.set_corp_id(corp_id);
        instance
            .reply(
                plugin_id,
                plugin_version,
                plugin_name,
                ability_name,
                plugin_args,
                recipients,
                true,
            )
            .await?;
        Ok(instance)
    }

    /// 回复带按钮的 Markdown 卡片
    pub async fn reply_markdown_button(
        &self,
        incoming_message: &ChatbotMessage,
        markdown: &str,
        button_list: Vec<serde_json::Value>,
        tips: &str,
        title: &str,
        logo: &str,
    ) -> crate::Result<MarkdownButtonCardInstance> {
        let mut instance =
            MarkdownButtonCardInstance::new(self.make_card_replier(incoming_message));
        instance.set_title_and_logo(title, logo);
        instance
            .reply(markdown, button_list, tips, None, true)
            .await?;
        Ok(instance)
    }

    /// 回复带按钮的 AI Markdown 卡片
    #[allow(clippy::too_many_arguments)]
    pub async fn reply_ai_markdown_button(
        &self,
        incoming_message: &ChatbotMessage,
        markdown: &str,
        button_list: Vec<serde_json::Value>,
        tips: &str,
        title: &str,
        logo: &str,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<AIMarkdownCardInstance> {
        let mut instance = AIMarkdownCardInstance::new(self.make_ai_card_replier(incoming_message));
        instance.set_title_and_logo(title, logo);
        instance.ai_start(recipients, support_forward).await?;
        instance.ai_streaming(markdown, true).await?;
        instance
            .ai_finish(Some(markdown), Some(button_list), tips)
            .await?;
        Ok(instance)
    }

    /// 回复轮播图卡片
    pub async fn reply_carousel_card(
        &self,
        incoming_message: &ChatbotMessage,
        markdown: &str,
        image_slider: &[(String, String)],
        button_text: &str,
        title: &str,
        logo: &str,
    ) -> crate::Result<CarouselCardInstance> {
        let mut instance = CarouselCardInstance::new(self.make_ai_card_replier(incoming_message));
        instance.set_title_and_logo(title, logo);
        instance
            .reply(markdown, image_slider, button_text, None, true)
            .await?;
        Ok(instance)
    }

    /// 发起 AI 卡片
    pub async fn ai_markdown_card_start(
        &self,
        incoming_message: &ChatbotMessage,
        title: &str,
        logo: &str,
        recipients: Option<&[String]>,
    ) -> crate::Result<AIMarkdownCardInstance> {
        let mut instance = AIMarkdownCardInstance::new(self.make_ai_card_replier(incoming_message));
        instance.set_title_and_logo(title, logo);
        instance.ai_start(recipients, true).await?;
        Ok(instance)
    }

    /// 从消息中提取文本列表
    pub fn extract_text(incoming_message: &ChatbotMessage) -> Option<Vec<String>> {
        incoming_message.get_text_list()
    }

    /// 从消息中提取图片，下载后重新上传到钉钉
    pub async fn extract_and_reupload_images(
        &self,
        incoming_message: &ChatbotMessage,
    ) -> crate::Result<Vec<String>> {
        let image_list = match incoming_message.get_image_list() {
            Some(list) if !list.is_empty() => list,
            _ => return Ok(Vec::new()),
        };
        let mut media_ids = Vec::new();
        for download_code in &image_list {
            let download_url = self.get_image_download_url(download_code).await?;
            let image_bytes = self.http_client.get_bytes(&download_url).await?;
            let media_id = self
                .upload_to_dingtalk(&image_bytes, "image", "image.png", "image/png")
                .await?;
            media_ids.push(media_id);
        }
        Ok(media_ids)
    }

    /// 根据 download_code 获取图片下载 URL
    pub async fn get_image_download_url(&self, download_code: &str) -> crate::Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        let body = serde_json::json!({"robotCode": self.client_id, "downloadCode": download_code});
        let url = format!(
            "{}/v1.0/robot/messageFiles/download",
            self.http_client.openapi_endpoint()
        );
        let resp: serde_json::Value = self
            .http_client
            .post_json(&url, &body, Some(&access_token))
            .await?;
        resp.get("downloadUrl")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| crate::Error::Handler("downloadUrl not found".to_owned()))
    }

    /// 上传文件到钉钉
    pub async fn upload_to_dingtalk(
        &self,
        content: &[u8],
        filetype: &str,
        filename: &str,
        mimetype: &str,
    ) -> crate::Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        self.http_client
            .upload_file(&access_token, content, filetype, filename, mimetype)
            .await
    }

    // ── Rust SDK exclusive: OTO message API ──────────────────────────
    // NOTE: This method is Rust-SDK-only and does NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this section.

    /// 通过 OpenAPI 向指定用户发送单聊消息（OTO = One-To-One）。
    ///
    /// 对应钉钉 OpenAPI：`POST /v1.0/robot/oToMessages/batchSend`
    ///
    /// # Arguments
    /// * `user_id`  – 接收者的 userId（即 staffId）
    /// * `msg_key`  – 消息模板 key，如 `"sampleFile"`, `"sampleText"` 等
    /// * `msg_param` – 消息模板参数的 JSON 字符串
    pub async fn send_oto_message(
        &self,
        user_id: &str,
        msg_key: &str,
        msg_param: &str,
    ) -> crate::Result<serde_json::Value> {
        let access_token = self.token_manager.get_access_token().await?;
        let body = serde_json::json!({
            "robotCode": self.client_id,
            "userIds": [user_id],
            "msgKey": msg_key,
            "msgParam": msg_param,
        });
        let url = format!(
            "{}/v1.0/robot/oToMessages/batchSend",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &body, Some(&access_token))
            .await
    }

    // ── Rust SDK exclusive: download helpers ─────────────────────────
    // NOTE: These methods are Rust-SDK-only and do NOT exist in the
    // official Python SDK.  When syncing features from the Python SDK,
    // do NOT remove this section.

    /// 下载文件字节内容（无大小限制）
    pub async fn download_bytes(&self, url: &str) -> crate::Result<Vec<u8>> {
        self.http_client.get_bytes(url).await
    }

    /// 下载文件字节内容（带大小限制）
    ///
    /// 先检查响应的 `Content-Length` 头，超过 `max_size` 则直接拒绝；
    /// 下载过程中累计检查已读字节数，超限则中止。
    pub async fn download_bytes_with_limit(
        &self,
        url: &str,
        max_size: u64,
    ) -> crate::Result<Vec<u8>> {
        self.http_client.get_bytes_with_limit(url, max_size).await
    }

    /// 设置离线提示词
    pub async fn set_off_duty_prompt(
        &self,
        text: &str,
        title: &str,
        logo: &str,
    ) -> crate::Result<serde_json::Value> {
        let access_token = self.token_manager.get_access_token().await?;
        let title = if title.is_empty() {
            "钉钉Stream机器人"
        } else {
            title
        };
        let logo = if logo.is_empty() {
            "@lALPDfJ6V_FPDmvNAfTNAfQ"
        } else {
            logo
        };
        let prompt_card_data = generate_multi_text_line_card_data(title, logo, &[text]);
        let body = serde_json::json!({
            "robotCode": self.client_id,
            "cardData": serde_json::to_string(&prompt_card_data).unwrap_or_default(),
            "cardTemplateId": "StandardCard",
        });
        let url = format!(
            "{}/v1.0/innerApi/robot/stream/away/template/update",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &body, Some(&access_token))
            .await
    }
}

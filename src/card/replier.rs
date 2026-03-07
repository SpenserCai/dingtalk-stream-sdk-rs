//! 卡片回复器，对齐 Python card_replier.py

use crate::messages::chatbot::ChatbotMessage;
use crate::transport::http::HttpClient;
use crate::transport::token::TokenManager;
use serde_repr::{Deserialize_repr, Serialize_repr};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// AI 卡片状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
#[non_exhaustive]
pub enum AICardStatus {
    /// 处理中
    Processing = 1,
    /// 输入中
    Inputing = 2,
    /// 完成
    Finished = 3,
    /// 执行中
    Executing = 4,
    /// 失败
    Failed = 5,
}

/// 卡片回复器（使用 Arc 共享所有权，避免生命周期问题）
#[derive(Clone)]
pub struct CardReplier {
    pub(crate) http_client: HttpClient,
    pub(crate) token_manager: Arc<TokenManager>,
    pub(crate) client_id: String,
    pub(crate) incoming_message: ChatbotMessage,
}

impl CardReplier {
    /// 创建新的卡片回复器
    pub fn new(
        http_client: HttpClient,
        token_manager: Arc<TokenManager>,
        client_id: String,
        incoming_message: ChatbotMessage,
    ) -> Self {
        Self {
            http_client,
            token_manager,
            client_id,
            incoming_message,
        }
    }

    /// 生成卡片 ID (SHA256)
    pub fn gen_card_id(msg: &ChatbotMessage) -> String {
        let factor = format!(
            "{}_{}_{}_{}_{}",
            msg.sender_id.as_deref().unwrap_or(""),
            msg.sender_corp_id.as_deref().unwrap_or(""),
            msg.conversation_id.as_deref().unwrap_or(""),
            msg.message_id.as_deref().unwrap_or(""),
            uuid::Uuid::new_v4()
        );
        let mut hasher = Sha256::new();
        hasher.update(factor.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 将 `cardParamMap` 中的所有值转为字符串（钉钉 API 要求）
    fn stringify_card_param_map(card_data: &serde_json::Value) -> serde_json::Value {
        match card_data {
            serde_json::Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(
                        k.clone(),
                        match v {
                            serde_json::Value::String(_) => v.clone(),
                            _ => serde_json::Value::String(v.to_string()),
                        },
                    );
                }
                serde_json::Value::Object(out)
            }
            other => other.clone(),
        }
    }

    /// 创建并发送卡片（两步：创建 + 投放）
    #[allow(clippy::too_many_arguments)]
    pub async fn create_and_send_card(
        &self,
        card_template_id: &str,
        card_data: &serde_json::Value,
        callback_type: &str,
        callback_route_key: &str,
        at_sender: bool,
        at_all: bool,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        let card_instance_id = Self::gen_card_id(&self.incoming_message);
        let param_map = Self::stringify_card_param_map(card_data);

        let mut create_body = serde_json::json!({
            "cardTemplateId": card_template_id,
            "outTrackId": card_instance_id,
            "cardData": {"cardParamMap": param_map},
            "callbackType": callback_type,
            "imGroupOpenSpaceModel": {"supportForward": support_forward},
            "imRobotOpenSpaceModel": {"supportForward": support_forward},
        });

        if callback_type == "HTTP" {
            create_body["callbackRouteKey"] = serde_json::json!(callback_route_key);
        }

        let url = format!(
            "{}/v1.0/card/instances",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &create_body, Some(&access_token))
            .await?;

        let mut deliver_body = serde_json::json!({
            "outTrackId": card_instance_id,
            "userIdType": 1,
        });

        self.build_deliver_body(&mut deliver_body, at_sender, at_all, recipients);

        let url = format!(
            "{}/v1.0/card/instances/deliver",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &deliver_body, Some(&access_token))
            .await?;

        Ok(card_instance_id)
    }

    /// 创建并投放卡片（一步到位）
    #[allow(clippy::too_many_arguments)]
    pub async fn create_and_deliver_card(
        &self,
        card_template_id: &str,
        card_data: &serde_json::Value,
        callback_type: &str,
        callback_route_key: &str,
        at_sender: bool,
        at_all: bool,
        recipients: Option<&[String]>,
        support_forward: bool,
        extra: Option<&serde_json::Value>,
    ) -> crate::Result<String> {
        let access_token = self.token_manager.get_access_token().await?;
        let card_instance_id = Self::gen_card_id(&self.incoming_message);
        let param_map = Self::stringify_card_param_map(card_data);

        let mut body = serde_json::json!({
            "cardTemplateId": card_template_id,
            "outTrackId": card_instance_id,
            "cardData": {"cardParamMap": param_map},
            "callbackType": callback_type,
            "imGroupOpenSpaceModel": {"supportForward": support_forward},
            "imRobotOpenSpaceModel": {"supportForward": support_forward},
        });

        if callback_type == "HTTP" {
            body["callbackRouteKey"] = serde_json::json!(callback_route_key);
        }

        self.build_deliver_body(&mut body, at_sender, at_all, recipients);

        if let Some(extra) = extra {
            if let (Some(body_obj), Some(extra_obj)) = (body.as_object_mut(), extra.as_object()) {
                for (k, v) in extra_obj {
                    body_obj.insert(k.clone(), v.clone());
                }
            }
        }

        let url = format!(
            "{}/v1.0/card/instances/createAndDeliver",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .post_json(&url, &body, Some(&access_token))
            .await?;

        Ok(card_instance_id)
    }

    /// 更新卡片数据
    pub async fn put_card_data(
        &self,
        card_instance_id: &str,
        card_data: &serde_json::Value,
        extra: Option<&serde_json::Value>,
    ) -> crate::Result<()> {
        let access_token = self.token_manager.get_access_token().await?;
        let param_map = Self::stringify_card_param_map(card_data);

        let mut body = serde_json::json!({
            "outTrackId": card_instance_id,
            "cardData": {"cardParamMap": param_map},
        });

        if let Some(extra) = extra {
            if let (Some(body_obj), Some(extra_obj)) = (body.as_object_mut(), extra.as_object()) {
                for (k, v) in extra_obj {
                    body_obj.insert(k.clone(), v.clone());
                }
            }
        }

        let url = format!(
            "{}/v1.0/card/instances",
            self.http_client.openapi_endpoint()
        );
        self.http_client
            .put_json(&url, &body, Some(&access_token))
            .await?;

        Ok(())
    }

    /// 构建投放请求体中的会话相关字段
    fn build_deliver_body(
        &self,
        body: &mut serde_json::Value,
        at_sender: bool,
        at_all: bool,
        recipients: Option<&[String]>,
    ) {
        let msg = &self.incoming_message;
        let Some(body_obj) = body.as_object_mut() else {
            return;
        };

        if msg.conversation_type.as_deref() == Some("2") {
            body_obj.insert(
                "openSpaceId".to_owned(),
                serde_json::json!(format!(
                    "dtv1.card//IM_GROUP.{}",
                    msg.conversation_id.as_deref().unwrap_or("")
                )),
            );

            let mut group_model = serde_json::json!({
                "robotCode": self.client_id,
            });

            if at_all {
                group_model["atUserIds"] = serde_json::json!({"@ALL": "@ALL"});
            } else if at_sender {
                let staff_id = msg.sender_staff_id.as_deref().unwrap_or("");
                let nick = msg.sender_nick.as_deref().unwrap_or("");
                group_model["atUserIds"] = serde_json::json!({staff_id: nick});
            }

            if let Some(recipients) = recipients {
                group_model["recipients"] = serde_json::json!(recipients);
            }

            if let Some(ref hosting) = msg.hosting_context {
                group_model["extension"] = serde_json::json!({
                    "hostingRepliedContext": serde_json::to_string(
                        &serde_json::json!({"userId": hosting.user_id})
                    ).unwrap_or_default()
                });
            }

            body_obj.insert("imGroupOpenDeliverModel".to_owned(), group_model);
        } else if msg.conversation_type.as_deref() == Some("1") {
            body_obj.insert(
                "openSpaceId".to_owned(),
                serde_json::json!(format!(
                    "dtv1.card//IM_ROBOT.{}",
                    msg.sender_staff_id.as_deref().unwrap_or("")
                )),
            );

            let mut robot_model = serde_json::json!({"spaceType": "IM_ROBOT"});

            if let Some(ref hosting) = msg.hosting_context {
                robot_model["extension"] = serde_json::json!({
                    "hostingRepliedContext": serde_json::to_string(
                        &serde_json::json!({"userId": hosting.user_id})
                    ).unwrap_or_default()
                });
            }

            body_obj.insert("imRobotOpenDeliverModel".to_owned(), robot_model);
        }
    }
}

/// AI 卡片回复器
#[derive(Clone)]
pub struct AICardReplier {
    inner: CardReplier,
}

impl AICardReplier {
    /// 创建新的 AI 卡片回复器
    pub fn new(inner: CardReplier) -> Self {
        Self { inner }
    }

    /// 获取内部 `CardReplier` 的引用
    pub fn inner(&self) -> &CardReplier {
        &self.inner
    }

    /// AI 卡片创建（flowStatus = PROCESSING）
    pub async fn start(
        &self,
        card_template_id: &str,
        card_data: &serde_json::Value,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<String> {
        let mut data = card_data.clone();
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "flowStatus".to_owned(),
                serde_json::json!(AICardStatus::Processing as u8),
            );
        }

        self.inner
            .create_and_send_card(
                card_template_id,
                &data,
                "STREAM",
                "",
                false,
                false,
                recipients,
                support_forward,
            )
            .await
    }

    /// AI 卡片完成（flowStatus = FINISHED）
    pub async fn finish(
        &self,
        card_instance_id: &str,
        card_data: &serde_json::Value,
    ) -> crate::Result<()> {
        let mut data = card_data.clone();
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "flowStatus".to_owned(),
                serde_json::json!(AICardStatus::Finished as u8),
            );
        }

        self.inner
            .put_card_data(card_instance_id, &data, None)
            .await
    }

    /// AI 卡片失败（flowStatus = FAILED）
    pub async fn fail(
        &self,
        card_instance_id: &str,
        card_data: &serde_json::Value,
    ) -> crate::Result<()> {
        let mut data = card_data.clone();
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "flowStatus".to_owned(),
                serde_json::json!(AICardStatus::Failed as u8),
            );
        }

        self.inner
            .put_card_data(card_instance_id, &data, None)
            .await
    }

    /// AI 卡片流式输出
    pub async fn streaming(
        &self,
        card_instance_id: &str,
        content_key: &str,
        content_value: &str,
        append: bool,
        finished: bool,
        failed: bool,
    ) -> crate::Result<()> {
        let access_token = self.inner.token_manager.get_access_token().await?;

        let body = serde_json::json!({
            "outTrackId": card_instance_id,
            "guid": uuid::Uuid::new_v4().to_string(),
            "key": content_key,
            "content": content_value,
            "isFull": !append,
            "isFinalize": finished,
            "isError": failed,
        });

        let url = format!(
            "{}/v1.0/card/streaming",
            self.inner.http_client.openapi_endpoint()
        );
        self.inner
            .http_client
            .put_json(&url, &body, Some(&access_token))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_card_id() {
        let msg = ChatbotMessage {
            sender_id: Some("user_001".to_owned()),
            sender_corp_id: Some("corp_001".to_owned()),
            conversation_id: Some("conv_001".to_owned()),
            message_id: Some("msg_001".to_owned()),
            ..Default::default()
        };
        let id = CardReplier::gen_card_id(&msg);
        assert_eq!(id.len(), 64);
    }

    #[test]
    fn test_ai_card_status_values() {
        assert_eq!(AICardStatus::Processing as u8, 1);
        assert_eq!(AICardStatus::Inputing as u8, 2);
        assert_eq!(AICardStatus::Finished as u8, 3);
        assert_eq!(AICardStatus::Executing as u8, 4);
        assert_eq!(AICardStatus::Failed as u8, 5);
    }

    #[test]
    fn test_ai_card_status_serialize() {
        let status = AICardStatus::Processing;
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json, 1);
    }
}

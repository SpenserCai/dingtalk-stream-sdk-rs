//! OAuth2 Access Token 管理（带缓存）

use crate::credential::Credential;
use crate::error::{Error, Result};
use crate::transport::http::HttpClient;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Token 缓存
struct TokenCache {
    access_token: String,
    expire_time: i64,
}

/// Token 管理器
pub struct TokenManager {
    credential: Credential,
    http_client: HttpClient,
    cache: RwLock<Option<TokenCache>>,
}

impl TokenManager {
    /// 创建新的 Token 管理器
    pub fn new(credential: Credential, http_client: HttpClient) -> Self {
        Self {
            credential,
            http_client,
            cache: RwLock::new(None),
        }
    }

    /// 获取当前时间戳（秒）
    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// 获取 access_token（带缓存）
    pub async fn get_access_token(&self) -> Result<String> {
        // 先尝试读缓存
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if Self::now() < c.expire_time {
                    return Ok(c.access_token.clone());
                }
            }
        }

        // 缓存过期或不存在，请求新 token
        let url = format!(
            "{}/v1.0/oauth2/accessToken",
            self.http_client.openapi_endpoint()
        );
        let body = serde_json::json!({
            "appKey": self.credential.client_id,
            "appSecret": self.credential.client_secret,
        });

        let result: serde_json::Value = self.http_client.post_json(&url, &body, None).await?;

        let access_token = result
            .get("accessToken")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Auth("accessToken not found in response".to_owned()))?
            .to_owned();

        let expire_in = result
            .get("expireIn")
            .and_then(|v| v.as_i64())
            .unwrap_or(7200);

        // 提前 5 分钟过期
        let expire_time = Self::now() + expire_in - 300;

        let mut cache = self.cache.write().await;
        *cache = Some(TokenCache {
            access_token: access_token.clone(),
            expire_time,
        });

        Ok(access_token)
    }

    /// 重置 token 缓存
    pub async fn reset(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}

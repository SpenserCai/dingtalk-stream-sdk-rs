//! 凭证管理

/// 钉钉应用凭证
#[derive(Clone)]
pub struct Credential {
    /// 应用的 client_id (appKey)
    pub client_id: String,
    /// 应用的 client_secret (appSecret)
    pub client_secret: String,
}

impl Credential {
    /// 创建新的凭证
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("client_id", &self.client_id)
            .field("client_secret", &"***")
            .finish()
    }
}

impl std::fmt::Display for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Credential(client_id={})", self.client_id)
    }
}

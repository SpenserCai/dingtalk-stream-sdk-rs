//! 卡片实例，对齐 Python card_instance.py

use crate::card::replier::{AICardReplier, AICardStatus, CardReplier};

const MARKDOWN_CARD_TEMPLATE_ID: &str = "589420e2-c1e2-46ef-a5ed-b8728e654da9.schema";
const MARKDOWN_BUTTON_CARD_TEMPLATE_ID: &str = "1366a1eb-bc54-4859-ac88-517c56a9acb1.schema";
const AI_MARKDOWN_CARD_TEMPLATE_ID: &str = "382e4302-551d-4880-bf29-a30acfab2e71.schema";
const RPA_PLUGIN_CARD_TEMPLATE_ID: &str = "7f538f6d-ebb7-4533-a9ac-61a32da094cf.schema";

/// Markdown 卡片实例
pub struct MarkdownCardInstance {
    replier: CardReplier,
    /// 卡片实例 ID
    pub card_instance_id: Option<String>,
    title: Option<String>,
    logo: Option<String>,
}

impl MarkdownCardInstance {
    /// 创建新的 Markdown 卡片实例
    pub fn new(replier: CardReplier) -> Self {
        Self {
            replier,
            card_instance_id: None,
            title: None,
            logo: None,
        }
    }

    /// 设置标题和 logo
    pub fn set_title_and_logo(&mut self, title: &str, logo: &str) {
        self.title = Some(title.to_owned());
        self.logo = Some(logo.to_owned());
    }

    fn get_card_data(&self, markdown: &str) -> serde_json::Value {
        let mut data = serde_json::json!({"markdown": markdown});
        if let Some(ref t) = self.title {
            if !t.is_empty() {
                data["title"] = serde_json::json!(t);
            }
        }
        if let Some(ref l) = self.logo {
            if !l.is_empty() {
                data["logo"] = serde_json::json!(l);
            }
        }
        data
    }

    /// 回复 markdown 内容
    pub async fn reply(
        &mut self,
        markdown: &str,
        at_sender: bool,
        at_all: bool,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<()> {
        let card_data = self.get_card_data(markdown);
        self.card_instance_id = Some(
            self.replier
                .create_and_send_card(
                    MARKDOWN_CARD_TEMPLATE_ID,
                    &card_data,
                    "STREAM",
                    "",
                    at_sender,
                    at_all,
                    recipients,
                    support_forward,
                )
                .await?,
        );
        Ok(())
    }

    /// 更新 markdown 内容
    pub async fn update(&self, markdown: &str) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not sent yet".to_owned()))?;
        self.replier
            .put_card_data(id, &self.get_card_data(markdown), None)
            .await
    }
}

/// Markdown + Button 卡片实例
pub struct MarkdownButtonCardInstance {
    replier: CardReplier,
    /// 卡片实例 ID
    pub card_instance_id: Option<String>,
    title: Option<String>,
    logo: Option<String>,
    button_list: Vec<serde_json::Value>,
}

impl MarkdownButtonCardInstance {
    /// 创建新实例
    pub fn new(replier: CardReplier) -> Self {
        Self {
            replier,
            card_instance_id: None,
            title: None,
            logo: None,
            button_list: Vec::new(),
        }
    }

    /// 设置标题和 logo
    pub fn set_title_and_logo(&mut self, title: &str, logo: &str) {
        self.title = Some(title.to_owned());
        self.logo = Some(logo.to_owned());
    }

    fn get_card_data(&self, markdown: &str, tips: &str) -> serde_json::Value {
        let mut data = serde_json::json!({"markdown": markdown, "tips": tips});
        if let Some(ref t) = self.title {
            if !t.is_empty() {
                data["title"] = serde_json::json!(t);
            }
        }
        if let Some(ref l) = self.logo {
            if !l.is_empty() {
                data["logo"] = serde_json::json!(l);
            }
        }
        if !self.button_list.is_empty() {
            data["sys_full_json_obj"] = serde_json::Value::String(
                serde_json::to_string(&serde_json::json!({"msgButtons": self.button_list}))
                    .unwrap_or_default(),
            );
        }
        data
    }

    /// 回复
    pub async fn reply(
        &mut self,
        markdown: &str,
        button_list: Vec<serde_json::Value>,
        tips: &str,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<()> {
        self.button_list = button_list;
        let card_data = self.get_card_data(markdown, tips);
        self.card_instance_id = Some(
            self.replier
                .create_and_send_card(
                    MARKDOWN_BUTTON_CARD_TEMPLATE_ID,
                    &card_data,
                    "STREAM",
                    "",
                    false,
                    false,
                    recipients,
                    support_forward,
                )
                .await?,
        );
        Ok(())
    }

    /// 更新
    pub async fn update(
        &mut self,
        markdown: &str,
        button_list: Vec<serde_json::Value>,
        tips: &str,
    ) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not sent yet".to_owned()))?;
        self.button_list = button_list;
        self.replier
            .put_card_data(id, &self.get_card_data(markdown, tips), None)
            .await
    }
}

/// AI Markdown 卡片实例
pub struct AIMarkdownCardInstance {
    replier: AICardReplier,
    /// 卡片实例 ID
    pub card_instance_id: Option<String>,
    title: Option<String>,
    logo: Option<String>,
    markdown: String,
    static_markdown: String,
    button_list: Option<Vec<serde_json::Value>>,
    inputing_status: bool,
    order: Vec<String>,
}

impl AIMarkdownCardInstance {
    /// 创建新实例
    pub fn new(replier: AICardReplier) -> Self {
        Self {
            replier,
            card_instance_id: None,
            title: None,
            logo: None,
            markdown: String::new(),
            static_markdown: String::new(),
            button_list: None,
            inputing_status: false,
            order: vec![
                "msgTitle".to_owned(),
                "msgContent".to_owned(),
                "staticMsgContent".to_owned(),
                "msgTextList".to_owned(),
                "msgImages".to_owned(),
                "msgSlider".to_owned(),
                "msgButtons".to_owned(),
            ],
        }
    }

    /// 设置标题和 logo
    pub fn set_title_and_logo(&mut self, title: &str, logo: &str) {
        self.title = Some(title.to_owned());
        self.logo = Some(logo.to_owned());
    }

    /// 设置排序
    pub fn set_order(&mut self, order: Vec<String>) {
        self.order = order;
    }

    /// 获取卡片数据
    pub fn get_card_data(&self, flow_status: Option<AICardStatus>) -> serde_json::Value {
        let mut data = serde_json::json!({
            "msgContent": self.markdown,
            "staticMsgContent": self.static_markdown,
        });
        if let Some(status) = flow_status {
            data["flowStatus"] = serde_json::json!(status as u8);
        }
        if let Some(ref t) = self.title {
            if !t.is_empty() {
                data["msgTitle"] = serde_json::json!(t);
            }
        }
        if let Some(ref l) = self.logo {
            if !l.is_empty() {
                data["logo"] = serde_json::json!(l);
            }
        }

        let mut sys_full = serde_json::json!({"order": self.order});
        if let Some(ref buttons) = self.button_list {
            if !buttons.is_empty() {
                sys_full["msgButtons"] = serde_json::json!(buttons);
            }
        }
        if let Some(ref hosting) = self.replier.inner().incoming_message.hosting_context {
            sys_full["source"] =
                serde_json::json!({"text": format!("由{}的数字助理回答", hosting.nick)});
        }
        data["sys_full_json_obj"] =
            serde_json::Value::String(serde_json::to_string(&sys_full).unwrap_or_default());
        data
    }

    /// 开始 AI 卡片
    pub async fn ai_start(
        &mut self,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<()> {
        if self.card_instance_id.is_some() {
            return Ok(());
        }
        self.card_instance_id = Some(
            self.replier
                .start(
                    AI_MARKDOWN_CARD_TEMPLATE_ID,
                    &serde_json::json!({}),
                    recipients,
                    support_forward,
                )
                .await?,
        );
        self.inputing_status = false;
        Ok(())
    }

    /// 流式输出
    pub async fn ai_streaming(&mut self, markdown: &str, append: bool) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not started yet".to_owned()))?
            .to_owned();
        if !self.inputing_status {
            let card_data = self.get_card_data(Some(AICardStatus::Inputing));
            self.replier
                .inner()
                .put_card_data(&id, &card_data, None)
                .await?;
            self.inputing_status = true;
        }
        if append {
            self.markdown.push_str(markdown);
        } else {
            self.markdown = markdown.to_owned();
        }
        self.replier
            .streaming(&id, "msgContent", &self.markdown, false, false, false)
            .await
    }

    /// 完成 AI 卡片
    pub async fn ai_finish(
        &mut self,
        markdown: Option<&str>,
        button_list: Option<Vec<serde_json::Value>>,
        _tips: &str,
    ) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not started yet".to_owned()))?
            .to_owned();
        if let Some(md) = markdown {
            self.markdown = md.to_owned();
        }
        if let Some(buttons) = button_list {
            self.button_list = Some(buttons);
        }
        self.replier.finish(&id, &self.get_card_data(None)).await
    }

    /// 更新非流式内容
    pub async fn update(
        &mut self,
        static_markdown: Option<&str>,
        button_list: Option<Vec<serde_json::Value>>,
        _tips: &str,
    ) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not started yet".to_owned()))?
            .to_owned();
        if let Some(buttons) = button_list {
            self.button_list = Some(buttons);
        }
        if let Some(sm) = static_markdown {
            self.static_markdown = sm.to_owned();
        }
        self.replier.finish(&id, &self.get_card_data(None)).await
    }

    /// 失败状态
    pub async fn ai_fail(&mut self) -> crate::Result<()> {
        let id = self
            .card_instance_id
            .as_deref()
            .ok_or_else(|| crate::Error::Card("card not started yet".to_owned()))?
            .to_owned();
        let mut card_data = serde_json::json!({});
        if let Some(ref t) = self.title {
            if !t.is_empty() {
                card_data["msgTitle"] = serde_json::json!(t);
            }
        }
        if let Some(ref l) = self.logo {
            if !l.is_empty() {
                card_data["logo"] = serde_json::json!(l);
            }
        }
        self.replier.fail(&id, &card_data).await
    }
}

/// 轮播图卡片实例
pub struct CarouselCardInstance {
    replier: AICardReplier,
    /// 卡片实例 ID
    pub card_instance_id: Option<String>,
    title: Option<String>,
    logo: Option<String>,
}

impl CarouselCardInstance {
    /// 创建新实例
    pub fn new(replier: AICardReplier) -> Self {
        Self {
            replier,
            card_instance_id: None,
            title: None,
            logo: None,
        }
    }

    /// 设置标题和 logo
    pub fn set_title_and_logo(&mut self, title: &str, logo: &str) {
        self.title = Some(title.to_owned());
        self.logo = Some(logo.to_owned());
    }

    /// 开始 AI 卡片
    pub async fn ai_start(&mut self) -> crate::Result<()> {
        self.card_instance_id = Some(
            self.replier
                .start(
                    AI_MARKDOWN_CARD_TEMPLATE_ID,
                    &serde_json::json!({}),
                    None,
                    true,
                )
                .await?,
        );
        Ok(())
    }

    /// 回复轮播图卡片
    pub async fn reply(
        &mut self,
        markdown: &str,
        image_slider_list: &[(String, String)],
        button_text: &str,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<()> {
        let slider: Vec<serde_json::Value> = image_slider_list
            .iter()
            .map(|(t, i)| serde_json::json!({"title": t, "image": i}))
            .collect();
        let sys_full = serde_json::json!({
            "order": ["msgTitle", "staticMsgContent", "msgSlider", "msgImages", "msgTextList", "msgButtons"],
            "msgSlider": slider,
            "msgButtons": [{"text": button_text, "color": "blue", "id": "image_slider_select_button", "request": true}]
        });
        let mut card_data = serde_json::json!({"staticMsgContent": markdown, "sys_full_json_obj": serde_json::to_string(&sys_full).unwrap_or_default()});
        if let Some(ref t) = self.title {
            if !t.is_empty() {
                card_data["msgTitle"] = serde_json::json!(t);
            }
        }
        if let Some(ref l) = self.logo {
            if !l.is_empty() {
                card_data["logo"] = serde_json::json!(l);
            }
        }

        self.card_instance_id = Some(
            self.replier
                .inner()
                .create_and_send_card(
                    AI_MARKDOWN_CARD_TEMPLATE_ID,
                    &serde_json::json!({"flowStatus": AICardStatus::Processing as u8}),
                    "STREAM",
                    "",
                    false,
                    false,
                    recipients,
                    support_forward,
                )
                .await?,
        );
        let id = self.card_instance_id.as_deref().unwrap_or_default();
        self.replier.finish(id, &card_data).await
    }
}

/// RPA 插件卡片实例
pub struct RPAPluginCardInstance {
    replier: AICardReplier,
    /// 卡片实例 ID
    pub card_instance_id: Option<String>,
    goal: String,
    corp_id: String,
}

impl RPAPluginCardInstance {
    /// 创建新实例
    pub fn new(replier: AICardReplier) -> Self {
        Self {
            replier,
            card_instance_id: None,
            goal: String::new(),
            corp_id: String::new(),
        }
    }

    /// 设置目标
    pub fn set_goal(&mut self, goal: &str) {
        self.goal = goal.to_owned();
    }

    /// 设置企业 ID
    pub fn set_corp_id(&mut self, corp_id: &str) {
        self.corp_id = corp_id.to_owned();
    }

    /// 回复 RPA 插件卡片
    #[allow(clippy::too_many_arguments)]
    pub async fn reply(
        &mut self,
        plugin_id: &str,
        plugin_version: &str,
        plugin_name: &str,
        ability_name: &str,
        plugin_args: &serde_json::Value,
        recipients: Option<&[String]>,
        support_forward: bool,
    ) -> crate::Result<()> {
        let plan = serde_json::json!({
            "corpId": self.corp_id, "goal": self.goal,
            "plan": format!("(function(){{dd.callPlugin({{'pluginName':'{}','abilityName':'{}','args':{} }});}})())", plugin_name, ability_name, serde_json::to_string(plugin_args).unwrap_or_default()),
            "planType": "jsCode",
            "pluginInstances": [{"id": format!("AGI-EXTENSION-{}", plugin_id), "version": plugin_version}]
        });
        let card_data = serde_json::json!({"goal": self.goal, "processFlag": "true", "plan": serde_json::to_string(&plan).unwrap_or_default()});

        self.card_instance_id = Some(
            self.replier
                .inner()
                .create_and_send_card(
                    RPA_PLUGIN_CARD_TEMPLATE_ID,
                    &serde_json::json!({"flowStatus": AICardStatus::Processing as u8}),
                    "STREAM",
                    "",
                    false,
                    false,
                    recipients,
                    support_forward,
                )
                .await?,
        );
        let id = self.card_instance_id.as_deref().unwrap_or_default();
        self.replier.finish(id, &card_data).await
    }
}

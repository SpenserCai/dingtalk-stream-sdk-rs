//! 端到端测试 - 启动机器人服务，等待用户通过钉钉发送消息进行测试
//!
//! 配置文件: examples/secret.toml (参考 examples/secret.example.toml)
//!
//! 运行: cargo run --example e2e_test
//! 带下载测试: cargo run --example e2e_test -- --download-dir /tmp/dingtalk_downloads
//!
//! 聊天机器人命令 (私聊或群聊 @机器人):
//!   ping          → 回复 pong (文本)
//!   echo <text>   → 回声 (文本)
//!   md            → Markdown 格式回复
//!   card          → 发送 Markdown 卡片，3 秒后自动更新内容
//!   buttons       → 发送带按钮的 Markdown 卡片
//!   ai            → AI 流式卡片 (start → streaming × 3 → finish)
//!   aibuttons     → AI Markdown 卡片 + 按钮
//!   carousel      → 轮播图卡片
//!   offduty       → 设置下班自动回复
//!   info          → 显示消息详情 (调试用)
//!   [语音消息]    → 回复语音识别结果 (仅单聊，Rust SDK 独有)
//!   [图片消息]    → 回复图片信息 (Rust SDK 独有)
//!   [文件消息]    → 回复文件信息 (仅单聊，Rust SDK 独有)
//!   [视频消息]    → 回复视频信息 (仅单聊，Rust SDK 独有)
//!
//! 卡片回调:
//!   点击带按钮的卡片 → 控制台打印回调详情
//!
//! 事件订阅:
//!   组织内事件 → 控制台打印事件类型和数据

use async_trait::async_trait;
use dingtalk_stream::*;
use std::collections::HashMap;

// ─── 配置加载 ───────────────────────────────────────────────────────────────

fn load_config(path: &str) -> HashMap<String, String> {
    let content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        panic!(
            "Failed to read config file: {path}\n\
             Please copy examples/secret.example.toml to examples/secret.toml and fill in your credentials."
        )
    });
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"');
            map.insert(key.trim().to_owned(), value.to_owned());
        }
    }
    map
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    ts.to_string()
}

// ─── 聊天机器人 Handler ─────────────────────────────────────────────────────

struct ChatbotTestHandler {
    replier: ChatbotReplier,
    download_dir: Option<String>,
}

#[async_trait]
impl CallbackHandler for ChatbotTestHandler {
    async fn process(&self, callback_message: &MessageBody) -> (u16, String) {
        let data: serde_json::Value =
            serde_json::from_str(&callback_message.data).unwrap_or_default();
        let incoming = match ChatbotMessage::from_value(&data) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("[Chatbot] Failed to parse message: {e}");
                return (AckMessage::STATUS_OK, "OK".to_owned());
            }
        };

        let text = ChatbotReplier::extract_text(&incoming)
            .and_then(|v| v.into_iter().next())
            .unwrap_or_default()
            .trim()
            .to_owned();

        let is_audio = incoming.message_type.as_deref() == Some("audio");
        let msg_type = incoming.message_type.as_deref().unwrap_or("unknown");
        let conv_type = incoming.conversation_type.as_deref().unwrap_or("unknown");
        println!(
            "[Chatbot] text='{text}', msgtype={msg_type}, conv={conv_type}, sender={:?}",
            incoming.sender_nick
        );

        // Rust SDK exclusive: echo back audio recognition result
        if is_audio {
            let ac = incoming.audio_content.as_ref();
            let duration_info = ac
                .and_then(|a| a.duration)
                .map(|ms| format!(" (时长 {:.1}s)", ms as f64 / 1000.0))
                .unwrap_or_default();
            let mut reply = format!(
                "🎤 语音识别结果{duration_info}:\n\n{}",
                if text.is_empty() {
                    "（未识别到内容）"
                } else {
                    &text
                },
            );
            if let Some(dir) = &self.download_dir {
                reply += &self.try_download(&incoming, dir).await;
            }
            let _ = self.replier.reply_text(&reply, &incoming).await;
            return (AckMessage::STATUS_OK, "OK".to_owned());
        }

        // Rust SDK exclusive: echo back file info
        if msg_type == "file" {
            let fc = incoming.file_content.as_ref();
            let name = fc.and_then(|f| f.file_name.as_deref()).unwrap_or("unknown");
            let dc = fc
                .and_then(|f| f.download_code.as_deref())
                .unwrap_or("none");
            let mut reply = format!("📄 收到文件\n\n文件名: {name}\n下载码: {dc}");
            if let Some(dir) = &self.download_dir {
                reply += &self.try_download(&incoming, dir).await;
            }
            let _ = self.replier.reply_text(&reply, &incoming).await;
            return (AckMessage::STATUS_OK, "OK".to_owned());
        }

        // Rust SDK exclusive: echo back picture info
        if msg_type == "picture" {
            let dc = incoming
                .image_content
                .as_ref()
                .and_then(|ic| ic.download_code.as_deref())
                .unwrap_or("none");
            let mut reply = format!("🖼️ 收到图片\n\n下载码: {dc}");
            if let Some(dir) = &self.download_dir {
                reply += &self.try_download(&incoming, dir).await;
            }
            let _ = self.replier.reply_text(&reply, &incoming).await;
            return (AckMessage::STATUS_OK, "OK".to_owned());
        }

        // Rust SDK exclusive: echo back video info
        if msg_type == "video" {
            let vc = incoming.video_content.as_ref();
            let duration = vc
                .and_then(|v| v.duration)
                .map(|ms| format!("{:.1}s", ms as f64 / 1000.0))
                .unwrap_or_else(|| "unknown".to_owned());
            let vtype = vc
                .and_then(|v| v.video_type.as_deref())
                .unwrap_or("unknown");
            let dc = vc
                .and_then(|v| v.download_code.as_deref())
                .unwrap_or("none");
            let mut reply = format!("🎬 收到视频\n\n时长: {duration}\n类型: {vtype}\n下载码: {dc}");
            if let Some(dir) = &self.download_dir {
                reply += &self.try_download(&incoming, dir).await;
            }
            let _ = self.replier.reply_text(&reply, &incoming).await;
            return (AckMessage::STATUS_OK, "OK".to_owned());
        }

        let result = self.handle_command(&text, &incoming).await;
        match &result {
            Ok(_) => println!("[Chatbot] Reply sent OK"),
            Err(e) => eprintln!("[Chatbot] Reply failed: {e}"),
        }

        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

impl ChatbotTestHandler {
    /// 尝试下载媒体文件到指定目录，返回追加到回复的文本
    async fn try_download(&self, incoming: &ChatbotMessage, dir: &str) -> String {
        let codes = incoming.get_all_download_codes();
        if codes.is_empty() {
            return String::new();
        }
        let mut result = String::from("\n\n── 下载测试 ──");
        for (media_type, download_code) in &codes {
            match self.replier.get_image_download_url(download_code).await {
                Ok(url) => {
                    let start = std::time::Instant::now();
                    match self.replier.download_bytes(&url).await {
                        Ok(bytes) => {
                            let elapsed = start.elapsed();
                            let ext = match media_type.as_str() {
                                "picture" => "png",
                                "audio" => "ogg",
                                "video" => "mp4",
                                "file" => incoming
                                    .file_content
                                    .as_ref()
                                    .and_then(|fc| fc.file_name.as_deref())
                                    .and_then(|n| n.rsplit('.').next())
                                    .unwrap_or("bin"),
                                _ => "bin",
                            };
                            let filename = format!("{}_{}.{ext}", media_type, chrono_timestamp());
                            let path = format!("{dir}/{filename}");
                            match std::fs::write(&path, &bytes) {
                                Ok(()) => {
                                    let size_kb = bytes.len() as f64 / 1024.0;
                                    result += &format!(
                                        "\n✅ {media_type}: {size_kb:.1}KB, {elapsed:.1?} → {path}"
                                    );
                                    println!(
                                        "[Download] {media_type} saved: {path} ({size_kb:.1}KB, {elapsed:.1?})"
                                    );
                                }
                                Err(e) => {
                                    result += &format!("\n❌ {media_type}: 写入失败: {e}");
                                }
                            }
                        }
                        Err(e) => result += &format!("\n❌ {media_type}: 下载失败: {e}"),
                    }
                }
                Err(e) => result += &format!("\n❌ {media_type}: 获取URL失败: {e}"),
            }
        }
        result
    }

    async fn handle_command(
        &self,
        text: &str,
        incoming: &ChatbotMessage,
    ) -> dingtalk_stream::Result<()> {
        match text.to_lowercase().as_str() {
            // ── 文本回复 ──
            "ping" => {
                self.replier.reply_text("pong 🏓", incoming).await?;
            }

            // ── 文本回声 ──
            t if t.starts_with("echo ") => {
                self.replier.reply_text(&text[5..], incoming).await?;
            }

            // ── Markdown 回复 ──
            "md" => {
                self.replier
                    .reply_markdown(
                        "Rust SDK 测试",
                        "### 钉钉 Stream SDK for Rust\n\n\
                         - **语言**: Rust 🦀\n\
                         - **运行时**: tokio async\n\
                         - **状态**: 正常运行中 ✅\n\n\
                         > 这是一条 Markdown 格式的测试消息",
                        incoming,
                    )
                    .await?;
            }

            // ── Markdown 卡片 + 延迟更新 ──
            "card" => {
                let instance = self
                    .replier
                    .reply_markdown_card(
                        "📦 **订单通知**\n\n您的订单 #RS-2026030701 已创建，正在处理中...",
                        incoming,
                        "订单助手",
                        "@lALPDfJ6V_FPDmvNAfTNAfQ",
                        false,
                        false,
                    )
                    .await?;
                println!(
                    "[Chatbot] Card sent, id={:?}, will update in 3s",
                    instance.card_instance_id
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                instance
                    .update("✅ **订单通知**\n\n您的订单 #RS-2026030701 已发货！\n\n快递单号: SF1234567890")
                    .await?;
                println!("[Chatbot] Card updated");
            }

            // ── 带按钮的 Markdown 卡片 ──
            "buttons" => {
                let buttons = vec![
                    serde_json::json!({"text": "👍 满意", "color": "blue", "id": "btn_good", "request": true}),
                    serde_json::json!({"text": "👎 不满意", "color": "red", "id": "btn_bad", "request": true}),
                ];
                self.replier
                    .reply_markdown_button(
                        incoming,
                        "### 服务评价\n\n请对本次服务进行评价：",
                        buttons,
                        "请点击按钮完成评价",
                        "服务评价",
                        "@lALPDfJ6V_FPDmvNAfTNAfQ",
                    )
                    .await?;
            }

            // ── AI 流式卡片 ──
            "ai" => {
                let mut instance = self
                    .replier
                    .ai_markdown_card_start(incoming, "AI 助手", "@lALPDfJ6V_FPDmvNAfTNAfQ", None)
                    .await?;
                instance.set_order(vec![
                    "msgTitle".to_owned(),
                    "msgContent".to_owned(),
                    "msgButtons".to_owned(),
                ]);
                println!(
                    "[Chatbot] AI card started, id={:?}",
                    instance.card_instance_id
                );

                // 第一阶段：思考中（后续会被全量替换）
                instance.ai_streaming("正在思考中...", true).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // 第二阶段：全量替换为正式内容，然后逐步追加
                let first = "根据您的问题，以下是我的分析：\n\n";
                instance.ai_streaming(first, false).await?;

                let chunks = [
                    "1. **Rust** 是一门注重安全和性能的系统编程语言\n",
                    "2. **tokio** 提供了高效的异步运行时\n",
                    "3. **serde** 实现了零成本的序列化/反序列化\n\n",
                    "> 以上信息由 Rust SDK E2E 测试生成",
                ];
                for chunk in &chunks {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    instance.ai_streaming(chunk, true).await?;
                }

                instance.ai_finish(None, None, "").await?;
                println!("[Chatbot] AI card finished");
            }

            // ── AI Markdown 卡片 + 按钮 ──
            "aibuttons" => {
                let buttons = vec![
                    serde_json::json!({"text": "📋 查看详情", "color": "blue", "id": "btn_detail", "request": true}),
                    serde_json::json!({"text": "🔄 重新生成", "color": "grey", "id": "btn_retry", "request": true}),
                ];
                let mut instance = self
                    .replier
                    .ai_markdown_card_start(incoming, "数据分析", "@lALPDfJ6V_FPDmvNAfTNAfQ", None)
                    .await?;
                instance.set_order(vec![
                    "msgTitle".to_owned(),
                    "msgContent".to_owned(),
                    "msgButtons".to_owned(),
                ]);
                let md = "### 智能分析报告\n\n\
                          本月销售数据分析完成：\n\
                          - 总销售额: ¥128,000\n\
                          - 环比增长: +15.3%\n\
                          - 最佳产品: Rust SDK 企业版";
                instance.ai_streaming(md, true).await?;
                instance
                    .ai_finish(Some(md), Some(buttons), "点击按钮查看更多")
                    .await?;
            }

            // ── 轮播图卡片 ──
            "carousel" => {
                let images: Vec<(String, String)> = vec![
                    (
                        "Rust 语言".to_owned(),
                        "https://www.rust-lang.org/static/images/rust-social-wide.jpg".to_owned(),
                    ),
                    (
                        "Ferris the Crab".to_owned(),
                        "https://rustacean.net/assets/rustacean-flat-happy.png".to_owned(),
                    ),
                ];
                self.replier
                    .reply_carousel_card(
                        incoming,
                        "### Rust 生态一览\n\n精选 Rust 社区图片",
                        &images,
                        "查看大图",
                        "图片浏览",
                        "@lALPDfJ6V_FPDmvNAfTNAfQ",
                    )
                    .await?;
            }

            // ── 设置下班提示 ──
            "offduty" => {
                self.replier
                    .set_off_duty_prompt(
                        "您好，当前为非工作时间（18:00 - 09:00），您的消息我已收到，将在工作时间第一时间回复。",
                        "值班助手",
                        "@lALPDfJ6V_FPDmvNAfTNAfQ",
                    )
                    .await?;
                self.replier
                    .reply_text("✅ 下班自动回复已设置", incoming)
                    .await?;
            }

            // ── 消息详情 ──
            "info" => {
                let texts = ChatbotReplier::extract_text(incoming);
                let info = format!(
                    "### 消息详情\n\n\
                     | 字段 | 值 |\n|---|---|\n\
                     | message_type | {:?} |\n\
                     | sender_nick | {:?} |\n\
                     | sender_id | {:?} |\n\
                     | sender_staff_id | {:?} |\n\
                     | conversation_type | {:?} |\n\
                     | conversation_id | {:?} |\n\
                     | conversation_title | {:?} |\n\
                     | message_id | {:?} |\n\
                     | robot_code | {:?} |\n\
                     | is_admin | {:?} |\n\
                     | at_users | {:?} |\n\
                     | extracted_texts | {:?} |\n\
                     | audio_content | {:?} |\n\
                     | file_content | {:?} |\n\
                     | video_content | {:?} |",
                    incoming.message_type,
                    incoming.sender_nick,
                    incoming.sender_id,
                    incoming.sender_staff_id,
                    incoming.conversation_type,
                    incoming.conversation_id,
                    incoming.conversation_title,
                    incoming.message_id,
                    incoming.robot_code,
                    incoming.is_admin,
                    incoming.at_users,
                    texts,
                    incoming.audio_content,
                    incoming.file_content,
                    incoming.video_content,
                );
                self.replier
                    .reply_markdown("消息详情", &info, incoming)
                    .await?;
            }

            // ── 帮助 ──
            _ => {
                self.replier
                    .reply_markdown(
                        "使用帮助",
                        "### 🤖 E2E 测试机器人\n\n\
                         | 命令 | 说明 |\n|---|---|\n\
                         | `ping` | 文本回复 pong |\n\
                         | `echo <文本>` | 文本回声 |\n\
                         | `md` | Markdown 格式回复 |\n\
                         | `card` | Markdown 卡片 (3s 后自动更新) |\n\
                         | `buttons` | 带按钮的卡片 |\n\
                         | `ai` | AI 流式卡片 |\n\
                         | `aibuttons` | AI 卡片 + 按钮 |\n\
                         | `carousel` | 轮播图卡片 |\n\
                         | `offduty` | 设置下班自动回复 |\n\
                         | `info` | 显示消息详情 |\n\
                         | 🎤 语音 | 回复识别结果 (仅单聊) |\n\
                         | 🖼️ 图片 | 回复图片信息 |\n\
                         | 📄 文件 | 回复文件信息 (仅单聊) |\n\
                         | 🎬 视频 | 回复视频信息 (仅单聊) |",
                        incoming,
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

// ─── 卡片回调 Handler ───────────────────────────────────────────────────────

struct CardCallbackTestHandler;

#[async_trait]
impl CallbackHandler for CardCallbackTestHandler {
    async fn process(&self, callback_message: &MessageBody) -> (u16, String) {
        match CardCallbackMessage::from_json_str(&callback_message.data) {
            Ok(msg) => {
                println!(
                    "[CardCallback] user={}, card={}, type={}, content={}",
                    msg.user_id, msg.card_instance_id, msg.callback_type, msg.content
                );
                // 返回卡片更新数据（对标 Python cardcallback 示例）
                let response = serde_json::json!({
                    "cardData": {
                        "cardParamMap": {
                            "markdown": format!("✅ 已收到您的反馈\n\n操作人: {}", msg.user_id),
                        }
                    }
                });
                (
                    AckMessage::STATUS_OK,
                    serde_json::to_string(&response).unwrap_or_else(|_| "OK".to_owned()),
                )
            }
            Err(e) => {
                eprintln!("[CardCallback] Parse failed: {e}");
                (AckMessage::STATUS_OK, "OK".to_owned())
            }
        }
    }
}

// ─── 事件 Handler ───────────────────────────────────────────────────────────

struct EventTestHandler;

#[async_trait]
impl EventHandler for EventTestHandler {
    async fn process(&self, event_message: &MessageBody) -> (u16, String) {
        println!(
            "[Event] type={:?}, id={:?}, born_time={:?}",
            event_message.headers.event_type,
            event_message.headers.event_id,
            event_message.headers.event_born_time,
        );
        println!("[Event] data={}", event_message.data);
        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

// ─── 主函数 ─────────────────────────────────────────────────────────────────

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // 解析 --download-dir 参数
    let args: Vec<String> = std::env::args().collect();
    let download_dir = args
        .windows(2)
        .find(|w| w[0] == "--download-dir")
        .map(|w| w[1].clone());
    if let Some(dir) = &download_dir {
        std::fs::create_dir_all(dir).expect("Failed to create download directory");
        println!("[Config] Download dir: {dir}");
    }

    let config = load_config("examples/secret.toml");
    let client_id = config
        .get("client_id")
        .expect("client_id not found in examples/secret.toml");
    let client_secret = config
        .get("client_secret")
        .expect("client_secret not found in examples/secret.toml");

    println!("╔══════════════════════════════════════════╗");
    println!("║   DingTalk Stream SDK - E2E Test Bot     ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║ client_id: {:<29}║", client_id);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Commands:                                ║");
    println!("║   ping / echo / md / card / buttons      ║");
    println!("║   ai / aibuttons / carousel / offduty    ║");
    println!("║   info / 🎤 voice / 🖼️ pic / 📄 file     ║");
    println!("║   🎬 video                               ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║ Also listening:                          ║");
    println!("║   Card callbacks, Events                 ║");
    println!("╠══════════════════════════════════════════╣");
    println!(
        "║ Download: {:<30}║",
        download_dir.as_deref().unwrap_or("disabled")
    );
    println!("╚══════════════════════════════════════════╝");

    let credential = Credential::new(client_id, client_secret);

    let temp_client = DingTalkStreamClient::builder(credential.clone()).build();
    let replier = temp_client.chatbot_replier();

    let mut client = DingTalkStreamClient::builder(credential)
        // 聊天机器人消息
        .register_callback_handler(
            ChatbotMessage::TOPIC,
            ChatbotTestHandler {
                replier,
                download_dir,
            },
        )
        // 卡片交互回调
        .register_callback_handler(CARD_CALLBACK_ROUTER_TOPIC, CardCallbackTestHandler)
        // 事件订阅
        .register_all_event_handler(EventTestHandler)
        .build();

    client.start_forever().unwrap();
}

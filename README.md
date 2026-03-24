# DingTalk Stream SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/dingtalk-stream.svg)](https://crates.io/crates/dingtalk-stream)
[![docs.rs](https://docs.rs/dingtalk-stream/badge.svg)](https://docs.rs/dingtalk-stream)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.85.0-orange.svg)](https://www.rust-lang.org)

钉钉 Stream 模式 SDK 的 Rust 实现，完整对齐 Python 版 [dingtalk-stream-sdk-python](https://github.com/open-dingtalk/dingtalk-stream-sdk-python)。

> 对齐的 Python SDK 版本: commit [`6381a7b`](https://github.com/open-dingtalk/dingtalk-stream-sdk-python/commit/6381a7bacb0353e1e0cd425688a02ff785fda44c)

## 功能特性

- 基于 WebSocket 的钉钉 Stream 协议连接（自动重连、60s Keepalive）
- 聊天机器人消息收发（文本、Markdown、卡片）
- 交互式卡片（Markdown / AI 流式 / 按钮 / 轮播 / RPA 插件）
- Graph API 支持
- 事件订阅
- OAuth2 Token 自动管理（带缓存与 5 分钟提前刷新）
- 统一 async/await（基于 tokio）

## 快速开始

将依赖添加到 `Cargo.toml`：

```toml
[dependencies]
dingtalk-stream = "0.1"
```

最小示例：

```rust
use async_trait::async_trait;
use dingtalk_stream::*;

struct MyHandler;

#[async_trait]
impl CallbackHandler for MyHandler {
    async fn process(&self, message: &messages::frames::MessageBody) -> (u16, String) {
        println!("Received: {}", message.data);
        (AckMessage::STATUS_OK, "OK".to_owned())
    }
}

fn main() {
    let credential = Credential::new("your_client_id", "your_client_secret");
    let handler = MyHandler;

    let mut client = DingTalkStreamClient::builder(credential)
        .register_callback_handler(ChatbotMessage::TOPIC, handler)
        .build();

    client.start_forever().unwrap();
}
```

更多示例见 [`examples/`](examples/) 目录。

## 项目结构

```
src/
├── lib.rs              # 公共 API 导出
├── client.rs           # DingTalkStreamClient + Builder
├── credential.rs       # Credential（Debug 遮蔽 secret）
├── error.rs            # 统一错误类型
├── handlers/           # Handler traits (Callback/Event/System/Chatbot/Graph)
├── messages/           # 消息类型 (Frames/Chatbot/CardCallback/Graph)
├── card/               # 卡片系统 (Replier/Instances/Templates)
└── transport/          # 传输层 (HTTP/Token/WebSocket)
```

## 文档

- [整体架构图](docs/architecture.md)
- [连接生命周期时序图](docs/connection-lifecycle.md)
- [卡片操作时序图](docs/card-operations.md)

## 端到端测试

1. 复制配置模板并填入凭证：
   ```bash
   cp examples/secret.example.toml examples/secret.toml
   # 编辑 examples/secret.toml，填入 client_id 和 client_secret
   ```

2. 运行测试机器人：
   ```bash
   cargo run --example e2e_test
   ```

3. 带媒体下载测试：
   ```bash
   cargo run --example e2e_test -- --download-dir /tmp/dingtalk_downloads
   ```

4. 通过钉钉向机器人发送消息测试：
   - `ping` → 回复 pong
   - `echo <text>` → 回声
   - `card` → 发送 Markdown 卡片
   - `info` → 显示消息详情

## 与 Python SDK 的设计差异

| Python | Rust | 说明 |
|---|---|---|
| sync + async 双版本 | 统一 async | 消除代码重复 |
| 类继承 | Trait + 组合 | Rust 惯用模式 |
| ThreadPoolExecutor | tokio::spawn | 更高效的异步调度 |
| dict 动态类型 | 强类型 struct + serde | 编译期类型安全 |

## Rust SDK 独有功能

以下功能为 Rust SDK 独有，Python SDK 中不存在。同步 Python SDK 时请勿删除（代码中均有 `Rust SDK exclusive` 标注）。

### 扩展消息类型

Rust SDK 支持解析钉钉机器人可接收的全部 6 种消息类型，Python SDK 仅支持 text / richText / picture：

| 消息类型 | 结构体 | 场景限制 |
|---------|--------|---------|
| audio 语音 | `AudioContent` | 仅单聊 |
| file 文件 | `FileContent` | 仅单聊 |
| video 视频 | `VideoContent` | 仅单聊 |

```rust
let msg = ChatbotMessage::from_value(&json)?;
if let Some(fc) = &msg.file_content {
    println!("文件名: {:?}, 下载码: {:?}", fc.file_name, fc.download_code);
}
```

### 统一下载码提取

`get_all_download_codes()` 一次性获取消息中所有媒体的下载码，覆盖 picture / richText 图片 / audio / file / video：

```rust
for (media_type, download_code) in msg.get_all_download_codes() {
    let url = replier.get_image_download_url(&download_code).await?;
    let bytes = replier.download_bytes(&url).await?;
}
```

### 文件下载

| 方法 | 说明 |
|------|------|
| `download_bytes(url)` | 下载文件字节内容 |
| `download_bytes_with_limit(url, max_size)` | 带大小限制的流式下载（Content-Length 预检 + 累计检查） |

### 主动单聊消息

`send_oto_message()` 通过 OpenAPI 向指定用户发送单聊消息（对应 `POST /v1.0/robot/oToMessages/batchSend`）：

```rust
replier.send_oto_message(user_id, "sampleText", r#"{"content":"hello"}"#).await?;
```

## 环境要求

- Rust ≥ 1.85.0（edition 2024）
- tokio 异步运行时

## License

MIT

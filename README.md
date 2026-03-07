# DingTalk Stream SDK for Rust

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
dingtalk-stream = { path = "." }
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

3. 通过钉钉向机器人发送消息测试：
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

## 环境要求

- Rust ≥ 1.85.0（edition 2024）
- tokio 异步运行时

## License

MIT

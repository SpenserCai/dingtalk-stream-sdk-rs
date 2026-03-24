//! DingTalk Stream SDK for Rust
//!
//! 钉钉 Stream 模式 SDK 的 Rust 实现，提供 WebSocket 长连接、消息处理、卡片系统等完整功能。

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod card;
pub mod client;
pub mod credential;
pub mod error;
pub mod handlers;
pub mod messages;
pub mod transport;

pub use client::{ClientBuilder, DingTalkStreamClient};
pub use credential::Credential;
pub use error::{Error, Result};

// Handlers
pub use handlers::callback::CallbackHandler;
pub use handlers::chatbot::{AsyncChatbotHandler, ChatbotHandler, ChatbotReplier};
pub use handlers::event::EventHandler;
pub use handlers::graph::{GraphHandler, GraphReplier};
pub use handlers::system::SystemHandler;

// Messages
pub use messages::card_callback::{CARD_CALLBACK_ROUTER_TOPIC, CardCallbackMessage};
pub use messages::chatbot::{
    AtUser, AudioContent, ChatbotMessage, ConversationMessage, FileContent, HostingContext,
    ImageContent, RichTextContent, TextContent, VideoContent,
};
pub use messages::frames::{AckMessage, Headers, MessageBody, StreamMessage};
pub use messages::graph::{GraphMessage, GraphRequest, GraphResponse, RequestLine, StatusLine};

// Cards
pub use card::instances::{
    AIMarkdownCardInstance, CarouselCardInstance, MarkdownButtonCardInstance, MarkdownCardInstance,
    RPAPluginCardInstance,
};
pub use card::replier::{AICardReplier, AICardStatus, CardReplier};
pub use card::templates::{
    generate_multi_text_image_card_data, generate_multi_text_line_card_data,
};

// Helper functions
pub use messages::chatbot::{reply_specified_group_chat, reply_specified_single_chat};

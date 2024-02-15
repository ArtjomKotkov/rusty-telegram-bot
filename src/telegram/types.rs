use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
    pub is_premium: Option<bool>,
    pub added_to_attachment_menu: Option<bool>,
    pub can_join_groups: Option<bool>,
    pub can_read_all_group_messages: Option<bool>,
    pub supports_inline_queries: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateResponse {
    pub update_id: i64,
    #[serde(flatten)]
    pub event: UpdateEvent
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FromBlock {
    pub id: u64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatBlock {
    pub id: u64
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub message_id: u64,
    pub from: FromBlock,
    pub chat: ChatBlock,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UpdateEvent {
    #[serde(rename = "message")]
    Message(Message)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyParameters {
    message_id: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRequest {
    chat_id: u64,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_parameters: Option<ReplyParameters>,
}

impl MessageRequest {
    pub fn new(chat_id: u64, text: String, reply_to: Option<u64>) -> Self {
        let reply_params = reply_to.map(|id| ReplyParameters {message_id: id});

        MessageRequest {
            chat_id,
            text,
            reply_parameters: reply_params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub chat_id: u64,
    pub message_id: u64,
    pub text: String,
}

pub struct Action {}
impl Action {
    pub const TYPING: &'static str = "typing";
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRequest {
    pub chat_id: u64,
    pub action: String
}



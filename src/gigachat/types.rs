use serde::{Deserialize, Serialize};
use serde_json::ser::CharEscape::Quote;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub access_token: String,
    pub expires_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
}

impl Question {
    pub fn from_string(message: String, model: String, max_tokens: u32) -> Self {
        Question {
            model,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: message,
                }
            ],
            max_tokens,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountQuestion {
    pub model: String,
    pub input: Vec<String>,
}

impl CountQuestion {
    pub fn from_string(message: String, model: String) -> Self {
        CountQuestion {
            model,
            input: vec![message],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    pub created: u64,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct CountAnswer {
    pub tokens: u32,
    pub characters: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub index: u32,
    pub finish_reason: String,
}
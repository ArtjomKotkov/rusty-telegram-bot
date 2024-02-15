use std::error::Error;
use std::time::Duration;

use async_stream::stream;
use futures_core::stream::Stream;
use tokio::time::sleep;
use reqwest;
use serde_json::{json, Value};
use serde_json::Value::{Bool, Null};

use super::types::{UserResponse, UpdateResponse, UpdateEvent, Message, MessageRequest, EditMessageRequest, ActionRequest};


enum Method {
    GET,
    POST,
}

pub struct TelegramClient {
    token: String,
    client: reqwest::Client,
    poll_refresh_rate: u64,
}

impl TelegramClient {
    const API_URL: &'static str = "https://api.telegram.org/bot";

    pub fn new(token: String, poll_refresh_rate: Option<u64>) -> Self {
        TelegramClient {
            token,
            client: reqwest::Client::new(),
            poll_refresh_rate: poll_refresh_rate.unwrap_or(1),
        }
    }


    pub async fn who_am_i(&self) -> Result<UserResponse, Box<dyn Error>> {
        let resp = self.make_request(Method::GET, "getMe", &Null).await?;

        Ok(serde_json::from_value::<UserResponse>(resp["result"].clone())?)
    }

    pub async fn send_message(&self, chat_id: u64, text: String, reply_to: Option<u64>) -> Result<Message, Box<dyn Error>> {
        let message_request = MessageRequest::new(chat_id, text, reply_to);
        let body = serde_json::to_value(message_request)?;

        let response = self.make_request(
            Method::POST,
            "sendMessage",
            &body,
        ).await?;

        Ok(serde_json::from_value::<Message>(response["result"].clone())?)
    }

    pub async fn update_message(&self, chat_id: u64, message_id: u64, text: String) -> Result<Message, Box<dyn Error>> {
        let message_request = EditMessageRequest {chat_id, message_id, text};
        let body = serde_json::to_value(message_request)?;

        let response = self.make_request(
            Method::POST,
            "editMessageText",
            &body,
        ).await?;

        Ok(serde_json::from_value::<Message>(response["result"].clone())?)
    }

    pub async fn send_action(&self, chat_id: u64, action: String) -> Result<(), Box<dyn Error>> {
        let request = ActionRequest {chat_id, action};
        let body = serde_json::to_value(request)?;

        let _ = self.make_request(
            Method::POST,
            "sendChatAction",
            &body,
        ).await?;

        Ok(())
    }

    pub async fn make_polling_stream(&self) -> impl Stream<Item = UpdateEvent> + '_ {
        let mut last_polling: Option<i64> = None;

        stream! {
            loop {
                let body = if let Some(value) = last_polling {
                    json!({"offset": value + 1, "allowed_updates": ["message"]})
                } else {
                    json!({"allowed_updates": ["message"]})
                };

                let response = self.make_request(Method::POST, "getUpdates", &body).await;

                if let Ok(value) = response {
                    if let Bool(true) = value["ok"] {
                        let result = value["result"].clone();
                        if let Ok(response) = serde_json::from_value::<Vec<UpdateResponse>>(result) {
                            if let Some(event) = response.last() {
                                last_polling = Some(event.update_id);
                            }

                            for item in response.iter() {
                                yield item.clone().event
                            }
                        }
                    } else {
                        println!(
                            "Error requesting [getUpdates] with offset {:?}, description:\n{:?}",
                            last_polling,
                            value.get("description")
                        );
                    }
                }

                sleep(Duration::from_secs(self.poll_refresh_rate)).await;
            }
        }
    }

    async fn make_request(
        &self,
        http_method: Method,
        method_name: &'static str,
        body: &Value,
    ) -> Result<Value, Box<dyn Error>> {
        let url = self.get_request_url(method_name);
        let resp: Value;

        match http_method {
            Method::GET => {
                resp = self.client.get(url).send().await?.json::<Value>().await?;
            }
            Method::POST => {
                resp = self.client.post(url).json(body).send().await?.json::<Value>().await?;
            }
        }

        Ok(resp)
    }

    fn get_request_url(&self, method_name: &str) -> String {
        TelegramClient::API_URL.to_string() + &self.token + "/" + method_name
    }
}
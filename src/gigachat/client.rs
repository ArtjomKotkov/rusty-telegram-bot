use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::time::SystemTime;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Value};
use uuid::Uuid;
use super::types::{Answer, CountAnswer, CountQuestion, Question, Token};


enum Method {
    GET,
    POST,
}


pub struct GigaChatClient {
    credentials: String,
    token: Option<Token>,
    client: reqwest::Client,
}

impl GigaChatClient {
    const AUTH_URL: &'static str = "https://ngw.devices.sberbank.ru:9443/api/v2/oauth";
    const BASE_URL: &'static str = " https://gigachat.devices.sberbank.ru/api/v1";

    pub fn new(credentials: String) -> Self{
        GigaChatClient {
            credentials,
            token: None,
            client: reqwest::Client::builder()
                // For not installing sber certs.
                .danger_accept_invalid_certs(true).build().unwrap(),
        }
    }

    pub async fn count(&mut self, question: CountQuestion) -> Result<Vec<CountAnswer>, Box<dyn Error>> {
        let url = GigaChatClient::BASE_URL.to_string() + "/tokens/count";
        let response = self.make_request(
            Method::POST,
            url,
            serde_json::to_value(question).unwrap(),
        ).await?;

        Ok(serde_json::from_value::<Vec<CountAnswer>>(response)?)
    }

    pub async fn ask(&mut self, question: Question) -> Result<Answer, Box<dyn Error>> {
        let url = GigaChatClient::BASE_URL.to_string() + "/chat/completions";
        let response = self.make_request(
            Method::POST,
            url,
            serde_json::to_value(question).unwrap(),
        ).await?;

        Ok(serde_json::from_value::<Answer>(response)?)
    }

    async fn get_token(&mut self) -> Result<Token, Box<dyn Error>> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        if let Some(token) = &self.token {
            if token.expires_at > now {
                return Ok(token.clone());
            }
        }

        let token = self.obtain_token(&self.credentials).await?;

        self.token.replace(token.clone());

        Ok(token)
    }

    async fn obtain_token(&self, credentials: &String) -> Result<Token, Box<dyn Error>> {
        let request_id = Uuid::new_v4();

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("RqUID", HeaderValue::from_str(&request_id.to_string()).unwrap());

        let mut auth_header_value = "Basic ".to_string();
        auth_header_value.push_str(credentials);
        headers.insert("Authorization", HeaderValue::from_str(auth_header_value.as_str()).unwrap());

        let mut params = HashMap::new();
        params.insert("scope", "GIGACHAT_API_PERS");

        let response = self.client
            .post(GigaChatClient::AUTH_URL.to_string())
            .headers(headers)
            .form(&params)
            .send().await?
            .json::<Value>().await?;

        Ok(serde_json::from_value::<Token>(response)?)
    }

    async fn make_request(
        &mut self,
        http_method: Method,
        url: String,
        body: Value,
    ) -> Result<Value, Box<dyn Error>> {
        let resp: Value;

        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let token = self.get_token().await?;

        match http_method {
            Method::GET => {
                resp = self.client
                    .get(url)
                    .bearer_auth(token.access_token)
                    .send().await?
                    .json::<Value>().await?;
            }
            Method::POST => {
                headers.insert("Content-Type", HeaderValue::from_static("application/json"));

                resp = self.client
                    .post(url)
                    .headers(headers)
                    .bearer_auth(token.access_token)
                    .json(&body)
                    .send().await?
                    .json::<Value>().await?;
            }
        }

        Ok(resp)
    }
}
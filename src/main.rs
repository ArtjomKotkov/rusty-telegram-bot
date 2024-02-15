mod telegram;
mod gigachat;

use std::env;

use futures_util::pin_mut;
use futures_util::stream::StreamExt;

use telegram::client::TelegramClient;
use telegram::types::{UpdateEvent, Message as TMessage, Action};

use gigachat::client::GigaChatClient;
use gigachat::types::{Question, CountQuestion};

use std::error::Error;
use std::str::FromStr;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let giga_chat_token = env::var("gigachat-secret")?;
    let tg_bot_token = env::var("tg-secret")?;

    let mut giga_client = GigaChatClient::new(giga_chat_token);
    let mut client: TelegramClient = TelegramClient::new(tg_bot_token, None);

    let resp = client.who_am_i().await?;
    println!("Bot id {}, {} is ready.", resp.id, resp.first_name);

    let stream = client.make_polling_stream().await;
    pin_mut!(stream);

    while let Some(value) = stream.next().await {
        handler(&client, &mut giga_client, value).await;
    }

    Ok(())
}

async fn handler(client: &TelegramClient, giga_client: &mut GigaChatClient, event: UpdateEvent) -> () {
    if let UpdateEvent::Message(message) = event {
        message_handler(client, giga_client, message).await;
    }
}

async fn message_handler(client: &TelegramClient, giga_client: &mut GigaChatClient, message: TMessage) -> () {
    if message.text.starts_with("/") {
        let items = message.text.split_once(" ");
        if let Some((command, full_args)) = items {
            command_handler(client, giga_client, &message, command, full_args).await;
        }
    } else {
        let _ = client.send_message(message.from.id.into(), "Какой-то ответ".to_string(), Some(message.message_id.into())).await;
    }
}

async fn command_handler(
    client: &TelegramClient,
    giga_client: &mut GigaChatClient,
    message: &TMessage,
    command: &str,
    full_args: &str,
) {
    match command {
        "/text" => {
            let question = Question::from_string(
                full_args.to_string(),
                "GigaChat".to_string(),
                512,
            );

            let _ = client.send_action(message.chat.id.into(), Action::TYPING.to_string()).await.unwrap();

            let response = giga_client.ask(question).await.unwrap();

            client.send_message(message.chat.id.into(), response.choices[0].message.content.clone(), Some(message.message_id.into())).await.unwrap();
        },
        "/count" => {
            let question = CountQuestion::from_string(
                full_args.to_string(),
                "GigaChat".to_string(),
            );

            let _ = client.send_action(message.chat.id.into(), Action::TYPING.to_string()).await.unwrap();

            let response = giga_client.count(question).await.unwrap()[0].clone();

            let text = format!("Tokens: {}\nCharacters: {}", response.tokens, response.characters);

            client.send_message(message.chat.id.into(), text, Some(message.message_id.into())).await.unwrap();
        },
        "/start" => {
            let text = "Help:\n/text {message} - ask a question\n/count {message} - count tokens in message";
            client.send_message(message.chat.id.into(), text.to_string(), None).await.unwrap();
        },
        _ => {

        }
    };
}


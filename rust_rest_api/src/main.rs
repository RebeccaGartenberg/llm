#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde::Deserialize;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW as GPT4;
use std::env;

#[derive(Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[post("/chat/completions", data = "<data>")]
fn index(data: Json<Message>) -> Result<(), Box<dyn std::error::Error>>{
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let role = data.role.clone();
    let content = data.content.clone();
    let user_role = "user".to_string();
    let system_role = "system".to_string();
    let mut message_role = chat_completion::MessageRole::user; // default role to user for now

    if role == user_role {
        message_role = chat_completion::MessageRole::user;
    } else if role == system_role {
        message_role = chat_completion::MessageRole::system;
    }

    let req = ChatCompletionRequest::new(
        GPT4.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: message_role.clone(),
            content: chat_completion::Content::Text(String::from(content)),
            name: None,
        }],
    );

    let result = client.chat_completion(req)?;
    println!("Content: {:?}", result.choices[0].message.content);
    println!("Response Headers: {:?}", result.headers);

    Ok(())
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}

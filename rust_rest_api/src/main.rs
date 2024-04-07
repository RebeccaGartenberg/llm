#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
use rocket::{get, post, routes, Rocket, Request, Data, Response};
use rocket::http::Status;
use rocket::http::ContentType;
use rocket::response::Responder;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW as GPT4;
use std::env;

#[post("/chat/completions", data = "<input>")]
fn index(input: String) -> Result<(), Box<dyn std::error::Error>>{
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let req = ChatCompletionRequest::new(
        GPT4.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(input)),
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

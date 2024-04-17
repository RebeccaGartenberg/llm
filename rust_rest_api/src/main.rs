#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde::Deserialize;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW as GPT4;
use std::env;
mod crawler;
mod embeddings;
use std::fs::read_to_string;
use postgres::Client as PostgresClient;
use postgres::NoTls;

#[derive(Deserialize)]
struct Message {
    role: String,
    content: String,
}

fn web_crawler(url: String) -> Result<(), Box<dyn std::error::Error>>{
    println!("Scraping webpage: \n");
    crawler::web_scraper_links(url.to_string());

    for subdomain in read_to_string("./data/urls.txt").unwrap().lines() {
        crawler::web_scraper_html(subdomain.to_string());
    }

    let data_path = "./data/articles";
    embeddings::create_embeddings(data_path.to_string());

    Ok(())
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

    let message_embedding = embeddings::fetch_embeddings(&[content.clone()]);
    let message_embedding_vec = &message_embedding.unwrap()[0];

    // compare message embedding to all article embeddings
    let mut postgres_client = PostgresClient::configure()
        .host("localhost")
        .dbname("chatbot_db")
        .user(std::env::var("USER").unwrap().as_str()) //
        .connect(NoTls)?;

    let docs = postgres_client.query("SELECT * from documents", &[])?;

    // Retrieve top 2 closest embeddings from db
    let mut query = "SELECT content, 1-(embedding <=> '<embedding_vector>') as cosine_similarity
    FROM documents
    ORDER BY cosine_similarity DESC
    LIMIT 2;"; // source: https://tembo.io/blog/pgvector-and-embedding-solutions-with-postgres

    let message_embedding_vec_str = serde_json::to_string(message_embedding_vec).unwrap();
    let query = query.replace("<embedding_vector>", &message_embedding_vec_str);
    let ordered_docs = postgres_client.query(&query, &[])?;

    for doc in ordered_docs{
        let article_content: String = doc.get("content");
        println!("{:?}",  article_content);
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
    let url = "https://www.artificialintelligence-news.com";
    web_crawler(url.to_string());
    rocket::ignite().mount("/", routes![index]).launch();
}

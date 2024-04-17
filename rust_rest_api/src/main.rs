#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde::Deserialize;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW as GPT4;
use std::env;
mod crawler;
use chrono;
mod embeddings;
use std::fs::read_to_string;
use postgres::Client as PostgresClient;
use postgres::NoTls;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Messages {
    model: String,
    messages: Vec<serde_json::Value>
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
fn index(data: Json<Messages>) -> Result<(), Box<dyn std::error::Error>>  {
    let model = data.model.clone();
    let messages = data.messages.clone();
    let mut input_messages = data.messages.clone();
    let api_key = std::env::var("OPENAI_API_KEY").or(Err("Set OPENAI_API_KEY"))?;
    let today = chrono::offset::Local::today().format("%B %d, %Y").to_string();
    let mut ordered_doc_strings: Vec<String> = Vec::new();
    let mut postgres_client = PostgresClient::configure()
        .host("localhost")
        .dbname("chatbot_db")
        .user(std::env::var("USER").unwrap().as_str())
        .connect(NoTls)?;

    // Get embeddings for all messages
    for message in messages{

        // Retrieve top 2 closest embeddings from db
        let mut query = "SELECT content, 1-(embedding <=> '<embedding_vector>') as cosine_similarity
        FROM documents
        ORDER BY cosine_similarity DESC
        LIMIT 2;"; // source: https://tembo.io/blog/pgvector-and-embedding-solutions-with-postgres

        let role: String = message.get("role").unwrap().to_string();
        let content: String = message.get("content").unwrap().to_string();
        let message_embedding = embeddings::fetch_embeddings(&[content]);
        let message_embedding_vec = &message_embedding.unwrap()[0];
        let message_embedding_vec_str = serde_json::to_string(message_embedding_vec).unwrap();
        let query = query.replace("<embedding_vector>", &message_embedding_vec_str);
        let ordered_docs = postgres_client.query(&query, &[])?;

        for doc in ordered_docs{
            let article_content: String = doc.get("content");
            ordered_doc_strings.push(article_content);
        }
    }

    input_messages.insert(1, json!({"role": "system", "content": "Today is ".to_string() + &today }));
    input_messages.insert(2, json!({"role": "system", "content": "Use the information provided here to answer the following questions."}));
    let mut i = 3;
    let mut j = 0;
    let test = ordered_doc_strings.clone();
    for doc in ordered_doc_strings{
        input_messages.insert(i, json!({"role": "system", "content": test[j]}));
        i+=1;
        j+=1;
    }

    let response: Value = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(ureq::json!({
            "model": model,
            "messages": input_messages,
        }))?
        .into_json()?;

    println!("{}", response);

    Ok(())
}

fn main() {
    let url = "https://www.artificialintelligence-news.com";
    web_crawler(url.to_string());
    rocket::ignite().mount("/", routes![index]).launch();
}

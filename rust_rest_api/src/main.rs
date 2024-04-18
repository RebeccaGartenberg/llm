#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
mod crawler;
mod embeddings;
use std::fs::read_to_string;
use postgres::Client as PostgresClient;
use postgres::NoTls;
use serde_json::{json, Value};
use rocket::request::Request;
use rocket::response::{Response, Responder};
use rocket::http::{ContentType, Status};


#[derive(Deserialize)]
struct Messages {
    model: String,
    messages: Vec<serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub choices : Value,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        Ok(Response::build()
            .sized_body(Cursor::new(serde_json::to_string(&self).unwrap()))
            .header(ContentType::JSON)
            .status(Status::Ok)
            .finalize())
    }
}

fn web_crawler(url: String) -> Result<(), Box<dyn std::error::Error>>{
    println!("Scraping webpage: \n");
    crawler::web_scraper_links(url.to_string());

    for subdomain in read_to_string("./data/urls.txt").unwrap().lines() {
        crawler::web_scraper_html(subdomain.to_string());
    }

    let data_path: &str = "./data/articles";
    embeddings::create_embeddings(data_path.to_string());

    Ok(())
}

#[post("/chat/completions", data = "<data>")]
fn index(data: Json<Messages>) -> Result<ApiResponse, Box<dyn std::error::Error>> {

    let model: String = data.model.clone();
    let mut messages: Vec<serde_json::Value> = data.messages.clone();
    let mut input_messages: Vec<serde_json::Value> = data.messages.clone();
    let api_key: String = std::env::var("OPENAI_API_KEY").or(Err("Set OPENAI_API_KEY"))?;
    let today: String = chrono::offset::Local::today().format("%B %d, %Y").to_string();
    let mut ordered_doc_strings: Vec<String> = Vec::new();
    let mut postgres_client = PostgresClient::configure()
        .host("localhost")
        .dbname("chatbot_db")
        .user(std::env::var("USER").unwrap().as_str())
        .connect(NoTls)?;

    messages.push(json!({"role": "system", "content": "Today is ".to_string() + &today }));

    // Get embeddings for all messages
    for message in messages{

        // Retrieve top 2 closest embeddings from db
        let mut query: &str = "SELECT content, 1-(embedding <=> '<embedding_vector>') as cosine_similarity
        FROM documents
        ORDER BY cosine_similarity DESC
        LIMIT 2;"; // source: https://tembo.io/blog/pgvector-and-embedding-solutions-with-postgres

        let content: String = message.get("content").unwrap().to_string();
        let message_embedding = embeddings::fetch_embeddings(&[content]);
        let message_embedding_vec: &Vec<f32> = &message_embedding.unwrap()[0];
        let message_embedding_vec_str: String = serde_json::to_string(message_embedding_vec).unwrap();
        let query = query.replace("<embedding_vector>", &message_embedding_vec_str);
        let ordered_docs: Vec<postgres::row::Row> = postgres_client.query(&query, &[])?;

        for doc in ordered_docs{
            let article_content: String = doc.get("content");
            if !ordered_doc_strings.contains(&article_content){
                ordered_doc_strings.push(article_content);
            }
        }
    }

    input_messages.insert(1, json!({"role": "system", "content": "Today is ".to_string() + &today }));
    input_messages.insert(2, json!({"role": "system", "content": "Use the information provided here to answer the following questions."}));
    let mut i = 3;
    let mut j = 0;
    let ordered_doc_content: Vec<String> = ordered_doc_strings.clone();
    for doc in ordered_doc_strings{
        input_messages.insert(i, json!({"role": "system", "content": ordered_doc_content.clone()[j]}));
        i+=1;
        j+=1;
    }

    let response: ApiResponse = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(ureq::json!({
            "model": model,
            "messages": input_messages,
        }))?
        .into_json()?;
    Ok(response)
}

fn main() {
    let url: &str = "https://www.artificialintelligence-news.com";
    web_crawler(url.to_string());
    rocket::ignite().mount("/", routes![index]).launch();
}

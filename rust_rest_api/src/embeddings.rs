use postgres::Client as PostgresClient;
use postgres::NoTls;
use std::env;
use std::fs;
use serde_json::Value;
use pgvector::Vector;

// source: https://github.com/pgvector/pgvector-rust/blob/master/examples/openai/src/main.rs
fn fetch_embeddings(input: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY").or(Err("Set OPENAI_API_KEY"))?;

    let response: Value = ureq::post("https://api.openai.com/v1/embeddings")
        .set("Authorization", &format!("Bearer {}", api_key))
        .send_json(ureq::json!({
            "input": input,
            "model": "text-embedding-ada-002",
        }))?
        .into_json()?;

    let embeddings = response["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            v["embedding"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect()
        })
        .collect();

    Ok(embeddings)
}

pub fn create_embeddings(data_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut postgres_client = PostgresClient::configure()
        .host("localhost")
        .dbname("chatbot_db")
        .user(std::env::var("USER").unwrap().as_str()) //
        .connect(NoTls)?;

    // Initialize db table
    postgres_client.execute("CREATE EXTENSION IF NOT EXISTS vector;", &[]).unwrap();
    postgres_client.execute("DROP TABLE IF EXISTS documents", &[]).unwrap();
    postgres_client.execute("CREATE TABLE IF NOT EXISTS documents (id serial PRIMARY KEY, content text, embedding vector(1536))", &[]).unwrap();

    // Insert article embeddings into db
    let entries = fs::read_dir(data_path)?;
    let mut count = 1;
    for entry in entries{
        println!("getting article {}", count);
        let article_path = entry?.path();
        let article = fs::read_to_string(article_path);
        let input = [article.unwrap()];
        println!("creating embedding");
        let embeddings = fetch_embeddings(&input)?;

        for (content, embedding) in input.iter().zip(embeddings) {
            let embedding = Vector::from(embedding);
            postgres_client.execute("INSERT INTO documents (content, embedding) VALUES ($1, $2)", &[&content, &embedding])?;
        }
        count += 1;
    }
    println!("Inserted all articles");
    let result = postgres_client.query("SELECT * from documents", &[])?;

    // Retrieve embeddings from db
    for row in result {
        let content: String = row.get("content");
        let embedding: Vector = row.get("embedding");
        // println!("content: {}", content);
        // println!("embedding: {:?}", embedding);
    }

    Ok(())
}

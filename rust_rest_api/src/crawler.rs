extern crate spider;

use spider::website::Website;
use spider::tokio;
use std::fs::File;
use std::io::Write;

// Gets list of subdomains from main url
#[tokio::main]
pub async fn web_crawler_urls() {
    let url = "https://www.artificialintelligence-news.com";
    let mut website = Website::new(&url);
    website.crawl().await;

    for link in website.get_links() {
        println!("- {:?}", link.as_ref());
    }
}

// Extracts links to articles from webpage
pub fn web_scraper_links(url: String) -> Result<(), Box<dyn std::error::Error>>{
    let url_copy = url.clone();
    let response = reqwest::blocking::get(url);
    let html_content = response.unwrap().text().unwrap();
    let document = scraper::Html::parse_document(&html_content);
    let selector = scraper::Selector::parse("a[href]").unwrap();
    let mut url_vec: Vec<String> = Vec::new();
    let mut url_file = File::create("data/urls.txt").expect("creation failed");

    // Include articles from 2024 (all articles appear to contain /<year>)
    let article_url = "https://www.artificialintelligence-news.com/2024";

    // Extract <a> tags that contain href
    for a in document.select(&selector) {
        if let Some(href) = a.value().attr("href") {
            let resolved_url = reqwest::Url::parse(&url_copy)?.join(href)?;

            // Check if url has already been saved so as not to store duplicates
            if (resolved_url.to_string()).contains(article_url) & !url_vec.contains(&resolved_url.to_string()){
                url_vec.push(resolved_url.to_string());
                // Write url to file
                url_file.write(resolved_url.to_string().as_bytes()).expect("write failed");
                url_file.write("\n".as_bytes()).expect("write failed");
            }
        }
    }

    Ok(())
}

// Extracts html content from webpage
pub fn web_scraper_html(url: String) -> Result<(), Box<dyn std::error::Error>>{
    let response = reqwest::blocking::get(url);
    let html_content = response.unwrap().text().unwrap();
    let document = scraper::Html::parse_document(&html_content);
    let selector = scraper::Selector::parse("p").unwrap(); // searching for <p> tag based on html structure of selected website

    // Extract <p> tags
    for p in document.select(&selector) {
        println!("{}", p.text().collect::<String>());
    }

    Ok(())
}

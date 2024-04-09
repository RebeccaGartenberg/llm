extern crate spider;

use spider::website::Website;
use spider::tokio;

// Gets list of subdomains from main url
#[tokio::main]
pub async fn web_crawler_urls() {
    let url = "https://www.artificialintelligence-news.com";
    let mut website = Website::new(&url);
    website.configuration.subdomains = false;
    website.crawl().await;

    for link in website.get_links() {
        println!("- {:?}", link.as_ref());
    }
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

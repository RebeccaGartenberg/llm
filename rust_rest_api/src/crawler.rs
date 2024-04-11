extern crate spider;
use chrono::NaiveDate;
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
    let title_selector = scraper::Selector::parse("title").unwrap();
    let article_selector = scraper::Selector::parse("p").unwrap(); // searching for <p> tag based on html structure of selected website
    let date_selector = scraper::Selector::parse("time[datetime]").unwrap();
    let mut date: String = String::new();
    let mut title: String = String::new();

    // Extract article title
    for t in document.select(&title_selector){
        title = t.text().collect::<String>();
    }

    // Extract article date
    for d in document.select(&date_selector){
        date = d.text().collect::<String>();
    }

    let formatted_date = NaiveDate::parse_from_str(&date, "%B %d, %Y").expect("Failed to parse date");
    let filename = format!("./data/article_{}_{}.txt", formatted_date, title);
    let mut file = File::create(filename).expect("creation failed");

    // Write title to file
    file.write(title.as_bytes()).expect("write failed");
    file.write("\n".as_bytes()).expect("write failed");

    // Write date to file
    file.write(date.as_bytes()).expect("write failed");
    file.write("\n".as_bytes()).expect("write failed");

    // Extract article content from <p> tags
    for p in document.select(&article_selector) {
        let line = p.text().collect::<String>();

        // Do not include info before article
        if line.contains("LOG IN") || line.contains("ARTICLE"){
            continue;
        }
        // Do not include info after article
        if line.contains("Want to learn more about AI and big data from industry leaders?"){
            break;
        } else if line.contains("You must be logged in to post a comment."){
            break;
        }

        // Write article contents to file
        file.write(line.as_bytes()).expect("write failed");
        file.write("\n".as_bytes()).expect("write failed");
    }

    Ok(())
}

use std::env::var;

use color_eyre::eyre::{OptionExt, Result};
use scraper::Html;
use tracing::instrument;

use crate::scraping::{BASE_URL, get};

#[instrument(skip(html))]
fn parse_html(html: &str) -> Html {
    Html::parse_document(html)
}

#[instrument(skip(html))]
fn extract_script_urls(html: &Html) -> Vec<String> {
    html.select(&scraper::Selector::parse("script[src]").unwrap())
        .filter_map(|el| el.value().attr("src"))
        .map(String::from)
        .collect()
}

#[instrument(skip(urls))]
fn pick_script_url(urls: Vec<String>) -> Option<String> {
    const URL_PREFIX: &str = "./assets/index-";
    urls.into_iter().find(|url| url.starts_with(URL_PREFIX))
}

#[instrument]
fn format_script_url(url: &str, base_url: &str) -> String {
    let url = url.trim_start_matches("./");
    format!("{base_url}/{url}")
}

#[instrument]
pub async fn scrape_js_url() -> Result<String> {
    let base_url = var("BASE_URL").unwrap_or_else(|_| BASE_URL.to_string());

    let html = get(&base_url).await?;
    let parsed_html = parse_html(&html);

    let script_urls = extract_script_urls(&parsed_html);

    let picked_url = pick_script_url(script_urls);
    let picked_url = picked_url.ok_or_eyre("No suitable script URL found")?;

    let formatted_url = format_script_url(&picked_url, &base_url);

    Ok(formatted_url)
}

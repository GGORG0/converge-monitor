use color_eyre::eyre::{Result, eyre};
use tracing::instrument;

use crate::{
    HTTP_CLIENT,
    scraping::{
        extract_data::ExtractedData,
        js_estree::{get_js_estree, print_diagnostics},
        js_url::scrape_js_url,
    },
};

pub mod extract_data;
pub mod js_estree;
pub mod js_url;

const BASE_URL: &str = "https://converge.hackclub.com";

#[instrument]
pub async fn get(url: &str) -> Result<String> {
    Ok(HTTP_CLIENT.get(url).send().await?.text().await?)
}

#[instrument]
pub async fn scrape() -> Result<ExtractedData> {
    let js_url = scrape_js_url().await?;
    let js = get(&js_url).await?;

    let js_binding = js.clone();
    let parsed = get_js_estree(&js_binding).await?;

    if !parsed.errors.is_empty() {
        print_diagnostics(parsed.errors, js);
    }
    if parsed.panicked {
        return Err(eyre!("Parsing JS panicked"));
    }

    let program = parsed.program;

    let data = ExtractedData::extract(&program)?;

    Ok(data)
}

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::HTTP_CLIENT;

pub mod extract_data;
pub mod js_estree;
pub mod js_url;

const BASE_URL: &str = "https://converge.hackclub.com";

#[instrument]
pub async fn get(url: &str) -> Result<String> {
    Ok(HTTP_CLIENT.get(url).send().await?.text().await?)
}

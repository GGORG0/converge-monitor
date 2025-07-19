use std::{env, sync::LazyLock};

use color_eyre::eyre::{OptionExt, Report, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::instrument;
use url::Url;

use crate::monitor::Reward;

pub static EMPORIUM_URL: LazyLock<Url> = LazyLock::new(|| {
    env::var("BASE_URL")
        .unwrap_or_else(|_| "https://emporium.hackclub.com".to_string())
        .parse()
        .expect("Invalid BASE_URL")
});

static ITEM_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("body > div > div > main > div > ul > div").unwrap());

#[instrument(skip(http_client))]
pub async fn scrape(http_client: &Client) -> Result<Vec<Reward>> {
    let html = http_client
        .get(EMPORIUM_URL.as_str())
        .send()
        .await?
        .text()
        .await?;

    let html = Html::parse_document(&html);

    let rewards = html
        .select(&ITEM_SELECTOR)
        .map(|item| {
            let image = item
                .select(&Selector::parse("img").unwrap())
                .next()
                .ok_or_eyre("Image child not found")?;
            let title = item
                .select(&Selector::parse("h3").unwrap())
                .next()
                .ok_or_eyre("Title tag not found")?;
            let description = item
                .select(&Selector::parse("p").unwrap())
                .next()
                .ok_or_eyre("Description tag not found")?;
            let price = item
                .select(&Selector::parse("div > span").unwrap())
                .next()
                .ok_or_eyre("Price tag not found")?;

            Ok::<_, Report>(Reward {
                title: title.text().collect(),
                description: description.text().collect(),
                tokens: price
                    .text()
                    .collect::<String>()
                    .split(' ')
                    .next()
                    .ok_or_eyre("Price string invalid")?
                    .parse()?,
                image_url: image
                    .attr("src")
                    .ok_or_eyre("Image tag doesn't have src attr")?
                    .parse()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(rewards)
}

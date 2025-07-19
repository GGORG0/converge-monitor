use std::{
    env::{self, var},
    path::Path,
    sync::Arc,
    time::Duration,
};

use color_eyre::eyre::{Result, eyre};
use dotenvy::dotenv;
use reqwest::{Client, cookie::Jar};
use rustls::crypto::aws_lc_rs;
use slack_morphism::{
    SlackApiToken, SlackApiTokenValue, SlackClient, prelude::SlackClientHyperConnector,
};
use tokio::time::{MissedTickBehavior, interval};
use tracing::{error, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{monitor::run, scraping::EMPORIUM_URL, updates::UsergroupPing};

mod monitor;
mod scraping;
mod updates;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv().ok();
    init_tracing()?;

    aws_lc_rs::default_provider().install_default().ok();
    let hyper_connector = SlackClientHyperConnector::new()?;
    let client = SlackClient::new(hyper_connector);

    let http_client = init_reqwest()?;

    let token_value: SlackApiTokenValue = var("SLACK_XOXB")?.into();
    let token = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let channel = var("SLACK_CHANNEL")?.into();

    let path = var("DATA_FILE").unwrap_or_else(|_| "data.json".to_string());
    let path = Path::new(&path);

    let update_interval = var("UPDATE_INTERVAL")
        .ok()
        .map(|s| s.parse::<u64>())
        .transpose()?
        .unwrap_or(60 * 5);

    let usergroup_ping = UsergroupPing::new();

    let mut timer = interval(Duration::from_secs(update_interval));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        timer.tick().await;

        if let Err(e) = run(path, &session, &channel, &usergroup_ping, &http_client).await {
            error!(error = ?e);
        }
    }
}

fn init_tracing() -> Result<()> {
    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .with(ErrorLayer::default())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .try_init()?;

    Ok(())
}

fn init_reqwest() -> Result<Client> {
    let jar = Jar::default();
    let cookie = env::var("COOKIE").map_err(|_| eyre!("COOKIE environment variable not set"))?;

    jar.add_cookie_str(&cookie, &EMPORIUM_URL);

    let jar = Arc::new(jar);

    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .cookie_provider(jar)
        .build()?;

    Ok(client)
}

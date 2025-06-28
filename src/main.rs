use std::{env::var, path::Path, sync::LazyLock, time::Duration};

use color_eyre::eyre::Result;
use dotenvy::dotenv;
use reqwest::Client;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{error, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

use crate::monitor::run;

mod monitor;
mod scraping;
mod updates;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to build HTTP client")
});

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv().ok();
    init_tracing()?;

    let path = var("DATA_FILE").unwrap_or_else(|_| "data.json".to_string());
    let path = Path::new(&path);

    let update_interval = var("UPDATE_INTERVAL")
        .ok()
        .map(|s| s.parse::<u64>())
        .transpose()?
        .unwrap_or(60 * 5);

    let mut timer = interval(Duration::from_secs(update_interval));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        timer.tick().await;

        if let Err(e) = run(path).await {
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

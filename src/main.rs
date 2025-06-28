use std::sync::LazyLock;

use color_eyre::eyre::{Result, eyre};
use reqwest::Client;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

use crate::scraping::{
    extract_data::{
        platforms::get_platforms, rewards::get_rewards, top_level_elements::extract_root_element,
    },
    get,
    js_estree::{get_js_estree, print_diagnostics},
    js_url::scrape_js_url,
};

mod scraping;

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
    init_tracing()?;

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

    let root_element = extract_root_element(&program)?;

    let platforms = get_platforms(&program, root_element)?;
    dbg!(platforms);

    let rewards = get_rewards(root_element)?;
    dbg!(rewards);

    Ok(())
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

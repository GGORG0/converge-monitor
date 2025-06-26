use std::sync::LazyLock;

use reqwest::Client;

mod scraping;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .expect("Failed to build HTTP client")
});

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dbg!(scraping::js_url::scrape_js_urls().await);
}

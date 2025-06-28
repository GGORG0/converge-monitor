use std::{env::var, path::Path};

use chrono::Utc;
use color_eyre::eyre::Result;
use slack_morphism::SlackBlocksTemplate;
use tracing::info;

use crate::{
    scraping::{load_data, save_data, scrape},
    updates::{Update, compare},
};

pub async fn run(path: &Path) -> Result<()> {
    let data = scrape().await?;

    if !tokio::fs::try_exists(&path).await? {
        info!("Data file does not exist, creating it at {:?}", &path);
        save_data(&data, path).await?;
    }

    let old_data = load_data(path).await?;

    if data == old_data {
        info!("No updates found.");
        return Ok(());
    }

    let platform_updates = compare(&old_data.platforms, &data.platforms);
    let reward_updates = compare(&old_data.rewards, &data.rewards);

    let updates: Vec<Update> = platform_updates
        .into_iter()
        .map(Into::into)
        .chain(reward_updates.into_iter().map(Into::into))
        .collect();

    save_data(&data, path).await?;

    info!("{} updates found.", updates.len());

    if let Ok(log_path) = var("LOG_DIR") {
        let now = Utc::now();

        let log_path = Path::new(&log_path);
        if !tokio::fs::try_exists(log_path).await? {
            info!("Creating log directory at {:?}", log_path);
            tokio::fs::create_dir_all(log_path).await?;
        }
        let log_path = log_path.join(format!("updates_{}.json", now.to_rfc3339()));

        info!("Saving updates to {:?}", log_path);
        let updates_json = serde_json::to_string_pretty(&updates)?;
        tokio::fs::write(log_path, updates_json).await?;
    }

    let notification_text = create_notification_text(&updates);

    info!("Updates: {notification_text}");

    let blocks = updates
        .iter()
        .flat_map(|update| update.render_template())
        .collect::<Vec<_>>();

    if let Ok(block_log_path) = var("BLOCK_LOG_DIR") {
        let now = Utc::now();

        let block_log_path = Path::new(&block_log_path);
        if !tokio::fs::try_exists(block_log_path).await? {
            info!("Creating block log directory at {:?}", block_log_path);
            tokio::fs::create_dir_all(block_log_path).await?;
        }
        let block_log_path = block_log_path.join(format!("blocks_{}.json", now.to_rfc3339()));

        info!("Saving blocks to {:?}", block_log_path);
        let blocks_json = serde_json::to_string_pretty(&blocks)?;
        tokio::fs::write(block_log_path, blocks_json).await?;
    }

    Ok(())
}

fn create_notification_text(updates: &[Update]) -> String {
    let mut notification_texts = Vec::new();

    let new_items = updates
        .iter()
        .filter_map(|update| {
            if update.is_new() {
                Some(update.item_name())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !new_items.is_empty() {
        notification_texts.push(format!("New items: {}", new_items.join(", ")));
    }

    let updated_items = updates
        .iter()
        .filter_map(|update| {
            if update.is_updated() {
                Some(update.item_name())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !updated_items.is_empty() {
        notification_texts.push(format!("Updated items: {}", updated_items.join(", ")));
    }

    let removed_items = updates
        .iter()
        .filter_map(|update| {
            if update.is_removed() {
                Some(update.item_name())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !removed_items.is_empty() {
        notification_texts.push(format!("Removed items: {}", removed_items.join(", ")));
    }

    notification_texts.join(" · ").trim().to_string()
}

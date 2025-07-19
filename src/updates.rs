use serde::{Deserialize, Serialize};
use slack_morphism::{
    blocks::{
        SlackBlock, SlackBlockMarkDownText, SlackBlockPlainText, SlackContextBlock, SlackHeaderBlock, SlackImageBlock, SlackSectionBlock
    }, SlackBlocksTemplate
};

use crate::monitor::Reward;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardUpdate {
    New(Reward),
    Updated { old: Reward, new: Reward },
    Removed(Reward),
}

impl RewardUpdate {
    pub fn item_name(&self) -> &str {
        match self {
            RewardUpdate::New(item) => &item.title,
            RewardUpdate::Updated { new, .. } => &new.title,
            RewardUpdate::Removed(item) => &item.title,
        }
    }

    fn emoji(&self) -> &str {
        match self {
            RewardUpdate::New(_) => ":new:",
            RewardUpdate::Updated { .. } => ":arrows_counterclockwise:",
            RewardUpdate::Removed(_) => ":win10-trash:",
        }
    }

    pub fn item(&self) -> &Reward {
        match self {
            RewardUpdate::New(item) => item,
            RewardUpdate::Updated { new, .. } => new,
            RewardUpdate::Removed(item) => item,
        }
    }

    pub fn old_item(&self) -> Option<&Reward> {
        match self {
            RewardUpdate::Updated { old, .. } => Some(old),
            _ => None,
        }
    }
}

pub fn compare(old: &[Reward], new: &[Reward]) -> Vec<RewardUpdate> {
    let mut updates = Vec::new();

    for new_item in new {
        if let Some(old_item) = old.iter().find(|old_item| old_item.title == new_item.title) {
            if old_item != new_item {
                updates.push(RewardUpdate::Updated {
                    old: old_item.clone(),
                    new: new_item.clone(),
                });
            }
        } else {
            updates.push(RewardUpdate::New(new_item.clone()));
        }
    }

    for old_item in old {
        if !new.iter().any(|new_item| new_item.title == old_item.title) {
            updates.push(RewardUpdate::Removed(old_item.clone()));
        }
    }

    updates
}

impl SlackBlocksTemplate for RewardUpdate {
    fn render_template(&self) -> Vec<SlackBlock> {
        let item = self.item();
        let old_item = self.old_item();

        let token_text = old_item
            .and_then(|old_item| {
                if old_item.tokens != item.tokens {
                    Some(format!("{} → {}", old_item.tokens, item.tokens))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| item.tokens.to_string());

        let description_text = old_item
            .and_then(|old_item| {
                if old_item.description != item.description {
                    let old_desc = if !old_item.description.is_empty() {
                        old_item.description.clone()
                    } else {
                        "_no description_".to_string()
                    };

                    let new_desc = if !item.description.is_empty() {
                        item.description.clone()
                    } else {
                        "_no description_".to_string()
                    };

                    Some(format!("{old_desc} → {new_desc}"))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| item.description.clone());

        let image_url = item.image_url.clone();
        let old_image_url = old_item
            .map(|old_item| old_item.image_url.clone())
            .filter(|old_image_url| old_image_url != &image_url);

        vec![
            Some(
                SlackHeaderBlock::new(
                    SlackBlockPlainText::new(format!(
                        "{} Reward: {} (:coin: {token_text})",
                        self.emoji(),
                        self.item_name()
                    ))
                    .into(),
                )
                .into(),
            ),
            Some(
                SlackSectionBlock::new()
                    .with_text(SlackBlockMarkDownText::new(description_text).into())
                    .into(),
            ),
            old_image_url.map(|old_image_url| {
                SlackImageBlock::new(
                    old_image_url.into(),
                    format!("Old {} logo", self.item_name()),
                )
                .into()
            }),
            Some(
                SlackImageBlock::new(
                    image_url.into(),
                    format!("{} logo", self.item_name())
                ).into()
            )
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

pub struct UsergroupPing {
    usergroup_id: String,
}

impl UsergroupPing {
    pub fn new() -> Option<Self> {
        let usergroup_id = std::env::var("SLACK_USERGROUP_ID").ok()?;
        Some(UsergroupPing { usergroup_id })
    }
}

impl SlackBlocksTemplate for UsergroupPing {
    fn render_template(&self) -> Vec<SlackBlock> {
        vec![
            SlackContextBlock::new(vec![
                SlackBlockMarkDownText::new(format!(
                    "pinging <!subteam^{}> · <{}|>",
                    self.usergroup_id,
                    env!("CARGO_PKG_REPOSITORY")
                ))
                .into(),
            ])
            .into(),
        ]
    }
}

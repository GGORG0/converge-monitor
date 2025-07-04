use serde::{Deserialize, Serialize};
use slack_morphism::{
    SlackBlocksTemplate,
    blocks::{
        SlackBlock, SlackBlockMarkDownText, SlackBlockPlainText, SlackContextBlock,
        SlackHeaderBlock, SlackImageBlock, SlackSectionBlock,
    },
};

use crate::scraping::extract_data::{Item, platforms::Platform, rewards::Reward};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    Platform(ItemUpdate<Platform>),
    Reward(ItemUpdate<Reward>),
}

impl Update {
    pub fn item_name(&self) -> &str {
        match self {
            Update::Platform(update) => update.item_name(),
            Update::Reward(update) => update.item_name(),
        }
    }

    pub fn is_new(&self) -> bool {
        matches!(
            self,
            Update::Platform(ItemUpdate::New(_)) | Update::Reward(ItemUpdate::New(_))
        )
    }

    pub fn is_updated(&self) -> bool {
        matches!(
            self,
            Update::Platform(ItemUpdate::Updated { .. })
                | Update::Reward(ItemUpdate::Updated { .. })
        )
    }

    pub fn is_removed(&self) -> bool {
        matches!(
            self,
            Update::Platform(ItemUpdate::Removed(_)) | Update::Reward(ItemUpdate::Removed(_))
        )
    }
}

impl From<ItemUpdate<Platform>> for Update {
    fn from(update: ItemUpdate<Platform>) -> Self {
        Update::Platform(update)
    }
}

impl From<ItemUpdate<Reward>> for Update {
    fn from(update: ItemUpdate<Reward>) -> Self {
        Update::Reward(update)
    }
}

impl SlackBlocksTemplate for Update {
    fn render_template(&self) -> Vec<SlackBlock> {
        match self {
            Update::Platform(update) => update.render_template(),
            Update::Reward(update) => update.render_template(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemUpdate<T: Item> {
    New(T),
    Updated { old: T, new: T },
    Removed(T),
}

impl<T: Item> ItemUpdate<T> {
    pub fn item_name(&self) -> &str {
        match self {
            ItemUpdate::New(item) => item.name(),
            ItemUpdate::Updated { new, .. } => new.name(),
            ItemUpdate::Removed(item) => item.name(),
        }
    }

    fn emoji(&self) -> &str {
        match self {
            ItemUpdate::New(_) => ":new:",
            ItemUpdate::Updated { .. } => ":arrows_counterclockwise:",
            ItemUpdate::Removed(_) => ":win10-trash:",
        }
    }

    pub fn item(&self) -> &T {
        match self {
            ItemUpdate::New(item) => item,
            ItemUpdate::Updated { new, .. } => new,
            ItemUpdate::Removed(item) => item,
        }
    }

    pub fn old_item(&self) -> Option<&T> {
        match self {
            ItemUpdate::Updated { old, .. } => Some(old),
            _ => None,
        }
    }
}

pub fn compare<T: Item>(old: &[T], new: &[T]) -> Vec<ItemUpdate<T>> {
    let mut updates = Vec::new();

    for new_item in new {
        if let Some(old_item) = old
            .iter()
            .find(|old_item| old_item.name() == new_item.name())
        {
            if old_item != new_item {
                updates.push(ItemUpdate::Updated {
                    old: old_item.clone(),
                    new: new_item.clone(),
                });
            }
        } else {
            updates.push(ItemUpdate::New(new_item.clone()));
        }
    }

    for old_item in old {
        if !new
            .iter()
            .any(|new_item| new_item.name() == old_item.name())
        {
            updates.push(ItemUpdate::Removed(old_item.clone()));
        }
    }

    updates
}

impl SlackBlocksTemplate for ItemUpdate<Platform> {
    fn render_template(&self) -> Vec<SlackBlock> {
        let image_url = self.item().image.clone();
        let old_image_url = self
            .old_item()
            .map(|item| item.image.clone())
            .filter(|old_image_url| old_image_url != &image_url);

        vec![
            Some(
                SlackHeaderBlock::new(
                    SlackBlockPlainText::new(format!(
                        "{} Platform: {}",
                        self.emoji(),
                        self.item_name()
                    ))
                    .into(),
                )
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
                SlackImageBlock::new(image_url.into(), format!("{} logo", self.item_name())).into(),
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl SlackBlocksTemplate for ItemUpdate<Reward> {
    fn render_template(&self) -> Vec<SlackBlock> {
        let item = self.item();
        let old_item = self.old_item();

        let icon_text = old_item
            .and_then(|old_item| {
                if old_item.icon != item.icon {
                    let old_icon = old_item
                        .icon
                        .map(|icon| icon.to_string())
                        .unwrap_or(":no_entry_sign:".to_string());
                    let new_icon = item
                        .icon
                        .map(|icon| icon.to_string())
                        .unwrap_or(":no_entry_sign:".to_string());
                    Some(format!("({old_icon} → {new_icon}) "))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| item.icon.map(|icon| format!("{icon} ")).unwrap_or_default());

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

        vec![
            Some(
                SlackHeaderBlock::new(
                    SlackBlockPlainText::new(format!(
                        "{} Reward: {}{} (:coin: {token_text})",
                        self.emoji(),
                        icon_text,
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
                    "pinging <!subteam^{}> ·<{}|>",
                    self.usergroup_id,
                    env!("CARGO_PKG_REPOSITORY")
                ))
                .into(),
            ])
            .into(),
        ]
    }
}

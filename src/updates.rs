use serde::{Deserialize, Serialize};

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

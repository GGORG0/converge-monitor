use color_eyre::eyre::Result;
use oxc_ast::ast::Program;

use crate::scraping::extract_data::{
    platforms::{Platform, get_platforms},
    rewards::{Reward, get_rewards},
    top_level_elements::extract_root_element,
};

pub mod platforms;
pub mod rewards;
pub mod top_level_elements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedData {
    pub platforms: Vec<Platform>,
    pub rewards: Vec<Reward>,
}

impl ExtractedData {
    pub fn extract(program: &Program) -> Result<Self> {
        let root_element = extract_root_element(program)?;

        let platforms = get_platforms(program, root_element)?;
        let rewards = get_rewards(root_element)?;

        Ok(Self { platforms, rewards })
    }
}

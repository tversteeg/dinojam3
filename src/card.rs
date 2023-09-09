use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;

pub const CARD_PATHS: [&str; 2] = ["cannon", "noop"];

#[derive(Default, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardTarget {
    #[default]
    Nothing,
    InitialSpeed,
}

#[derive(Default, Clone, Deserialize)]
pub struct Card {
    title: String,
    description: Vec<String>,
    target: CardTarget,
    #[serde(default)]
    value: f64,
    #[serde(default)]
    cost: usize,
}

impl Asset for Card {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

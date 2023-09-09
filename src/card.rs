use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::game::GameState;

pub const CARD_PATHS: [&str; 2] = ["cannon", "noop"];
const CARD_SIZE: Extent2<f64> = Extent2::new(88.0, 110.0);

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

impl Card {
    pub fn render(&self, offset: Vec2<f64>, canvas: &mut [u32]) {
        let font = crate::font();
        font.render_centered(
            &self.title,
            offset + (CARD_SIZE.w / 2.0, CARD_SIZE.h / 6.0),
            canvas,
        );
        for (i, d) in self.description.iter().enumerate() {
            font.render_centered(
                d,
                offset + (CARD_SIZE.w / 2.0, CARD_SIZE.h / 2.0 + i as f64 * 12.0),
                canvas,
            );
        }

        let disk = crate::sprite("disk-icon");
        let x = 32.0;
        let y = CARD_SIZE.h - disk.height() as f64 - 4.0;
        disk.render(canvas, offset + (x, y));
        font.render(
            &format!("{}", self.cost),
            offset + (disk.width() as f64 + x + 2.0, y + 1.0),
            canvas,
        );
    }

    pub fn apply(&self, game: &mut GameState) {
        if self.cost > game.money {
            return;
        }

        game.money -= self.cost;
        match self.target {
            CardTarget::Nothing => (),
            CardTarget::InitialSpeed => game.extra_initial_speed += self.value,
        }
    }
}

impl Asset for Card {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

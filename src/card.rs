use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::game::GameState;

pub const CARDS: usize = 3;
pub const CARD_PATHS: [&str; CARDS] = ["cannon", "noop", "wings"];
const CARD_SIZE: Extent2<f64> = Extent2::new(88.0, 110.0);

#[derive(Default, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardTarget {
    #[default]
    Nothing,
    InitialSpeed,
    Gravity,
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
    #[serde(default)]
    max_amount: usize,
    #[serde(skip)]
    index: usize,
}

impl Card {
    pub fn random(money: usize, selected_cards: &[usize]) -> Self {
        let cards = CARD_PATHS
            .iter()
            .enumerate()
            .map(|(i, path)| {
                crate::asset::<Card>(&format!("card.{path}"))
                    .clone()
                    .with_index(i)
            })
            .filter(|card| {
                card.cost <= money
                    && (card.max_amount == 0 || selected_cards[card.index] < card.max_amount)
            })
            .collect::<Vec<_>>();

        fastrand::choice(cards).unwrap()
    }

    pub fn render(&self, offset: Vec2<f64>, canvas: &mut [u32], selected_cards: &[usize]) {
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

        if self.max_amount > 0 {
            font.render_centered(
                &format!("{}/{}", selected_cards[self.index], self.max_amount),
                offset + (CARD_SIZE.w / 2.0, 3.0),
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
            CardTarget::Gravity => game.extra_gravity += self.value,
        }
        game.selected_cards[self.index] += 1;
    }

    fn with_index(mut self, index: usize) -> Self {
        self.index = index;

        self
    }
}

impl Asset for Card {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

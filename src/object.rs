use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::{Extent2, Rect, Vec2};

use crate::{random::RandomRangeF64, SIZE};

#[derive(Debug)]
pub struct Object {
    repeat_distance: f64,
    pub pos: Vec2<f64>,
    parallax: Vec2<f64>,
    sprite_path: String,
    lock_x: Option<f64>,
    lock_y: Option<f64>,
    start_at: Vec2<f64>,
    collider: Extent2<f64>,
}

impl Object {
    pub fn reset(&mut self, pos: Vec2<f64>) {
        if pos.x == 0.0 {
            self.pos.x =
                self.start_at.x + SIZE.w as f64 * 3.0 * fastrand::f64() * self.repeat_distance;
        } else if pos.x > self.start_at.x {
            self.pos.x =
                SIZE.w as f64 * 2.0 + SIZE.w as f64 * fastrand::f64() * self.repeat_distance;
        }

        if self.lock_y.is_none() {
            if pos.y.abs() > self.start_at.y {
                self.pos.y = -(SIZE.h as f64 / 2.0) + fastrand::f64() * SIZE.h as f64;
            } else {
                self.pos.y = -self.start_at.y;
            }
        }
    }

    pub fn update(&mut self, pos: Vec2<f64>, vel: Vec2<f64>, player_offset: Vec2<f64>, dt: f64) {
        if let Some(lock_x) = self.lock_x {
            self.pos.x = lock_x - pos.x + player_offset.x + self.start_at.x;
        } else {
            self.pos.x -= vel.x * dt * (1.0 - self.parallax.x);
        }
        if let Some(lock_y) = self.lock_y {
            self.pos.y =
                lock_y - pos.y * (1.0 - self.parallax.y) + player_offset.y + self.start_at.y;
        } else {
            self.pos.y -= vel.y * dt * (1.0 - self.parallax.y);
        }

        if self.lock_x.is_none() && self.pos.x < -(SIZE.w as f64) {
            self.reset(pos);
        }
    }

    pub fn collides_user(&self, player_rect: Rect<f64, f64>) -> bool {
        Rect::new(self.pos.x, self.pos.y, self.collider.w, self.collider.h)
            .collides_with_rect(player_rect)
    }

    pub fn render(&self, canvas: &mut [u32]) {
        /*
        let aabr = Rect::new(self.pos.x, self.pos.y, self.collider.w, self.collider.h).into_aabr();
        crate::render_aabr(aabr, canvas, 0xFFFF0000);
        */

        crate::sprite(&self.sprite_path).render(canvas, self.pos);
    }
}

#[derive(Debug, Deserialize)]
pub struct ObjectsSpawner {
    repeat_distance: f64,
    amount: usize,
    sprite_path: String,
    #[serde(default)]
    parallax_x: RandomRangeF64,
    #[serde(default)]
    parallax_y_factor: f64,
    #[serde(default)]
    lock_x: Option<f64>,
    #[serde(default)]
    lock_y: Option<f64>,
    #[serde(default)]
    start_at: Vec2<f64>,
    #[serde(default)]
    collider: Extent2<f64>,
}

impl ObjectsSpawner {
    pub fn to_objects(&self) -> Vec<Object> {
        let settings = crate::settings();

        let mut objects = (0..self.amount)
            .map(|_| {
                let parallax_x = self.parallax_x.value();
                let mut obj = Object {
                    pos: Vec2::zero(),
                    repeat_distance: self.repeat_distance,
                    parallax: Vec2::new(parallax_x, parallax_x * self.parallax_y_factor),
                    sprite_path: self.sprite_path.clone(),
                    lock_x: self.lock_x,
                    lock_y: self.lock_y,
                    start_at: self.start_at,
                    collider: self.collider,
                };

                obj.reset(Vec2::zero());

                obj
            })
            .collect::<Vec<_>>();

        objects.sort_by_key(|obj| ((1.0 - obj.parallax.x) * 10000.0) as u32);

        objects
    }
}

impl Asset for ObjectsSpawner {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

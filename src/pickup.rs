use vek::Vec2;

use crate::{camera::Camera, SIZE};

pub struct Pickup {
    pub sprite: &'static str,
    pub spawn_rate: f64,
    pub active: Vec<Vec2<f64>>,
}

impl Pickup {
    pub fn new(sprite: &'static str, spawn_rate: f64, amount: usize) -> Self {
        let active = (0..amount)
            .map(|_| {
                Vec2::new(
                    SIZE.w as f64 + fastrand::f64() * SIZE.w as f64,
                    fastrand::f64() * SIZE.h as f64,
                )
            })
            .collect();

        Self {
            sprite,
            spawn_rate,
            active,
        }
    }

    pub fn update(&mut self, player: Vec2<f64>, dt: f64) {
        for item in self.active.iter_mut() {
            if item.x < player.x - SIZE.w as f64 {
                item.x = player.x + SIZE.w as f64;
                item.y = player.y + fastrand::f64() * SIZE.h as f64 * 2.0 - SIZE.h as f64
            }
        }
    }

    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        let sprite = crate::sprite(self.sprite);
        for item in self.active.iter() {
            sprite.render(canvas, camera, *item);
        }
    }
}

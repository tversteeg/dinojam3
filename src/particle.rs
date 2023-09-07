use vek::Vec2;

use crate::{graphics::Color, SIZE};

pub struct Particle {
    pos: Vec2<f64>,
    vel: Vec2<f64>,
    color: Color,
}

impl Particle {
    pub fn new(pos: Vec2<f64>, mut vel: Vec2<f64>, force: f64, color: Color) -> Self {
        debug_assert!(pos.x >= 0.0);
        debug_assert!(pos.y >= 0.0);
        debug_assert!(pos.x < SIZE.w as f64);
        debug_assert!(pos.y < SIZE.h as f64);

        vel += Vec2::new(
            fastrand::f64() * force * 2.0 - force,
            fastrand::f64() * force * 2.0 - force,
        );

        Self { pos, vel, color }
    }

    pub fn update(&mut self, vel: Vec2<f64>, gravity: f64, dt: f64) -> bool {
        self.pos += self.vel * dt;
        self.pos -= vel * dt;
        self.vel.y += gravity;

        !(self.pos.x < 1.0
            || self.pos.y < 1.0
            || self.pos.x >= SIZE.w as f64 - 1.0
            || self.pos.y >= SIZE.h as f64 - 1.0)
    }

    pub fn render(&self, canvas: &mut [u32]) {
        canvas[self.pos.x as usize + self.pos.y as usize * SIZE.w] = self.color.as_u32();
    }
}

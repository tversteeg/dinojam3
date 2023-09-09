use vek::Vec2;

use crate::{graphics::Color, SIZE};

pub struct Particle {
    pos: Vec2<f64>,
    vel: Vec2<f64>,
    color: Color,
    affected_by_world: bool,
    life: f64,
}

impl Particle {
    pub fn new(
        pos: Vec2<f64>,
        mut vel: Vec2<f64>,
        force: f64,
        color: Color,
        affected_by_world: bool,
        life: f64,
    ) -> Self {
        debug_assert!(pos.x >= 0.0);
        debug_assert!(pos.y >= 0.0);
        debug_assert!(pos.x < SIZE.w as f64);
        debug_assert!(pos.y < SIZE.h as f64);

        vel += Vec2::new(
            fastrand::f64() * force * 2.0 - force,
            fastrand::f64() * force * 2.0 - force,
        );

        Self {
            pos,
            vel,
            color,
            affected_by_world,
            life,
        }
    }

    pub fn update(&mut self, vel: Vec2<f64>, gravity: f64, dt: f64) -> bool {
        self.pos += self.vel * dt;
        if self.affected_by_world {
            self.pos -= vel * dt;
        }
        self.vel.y += gravity;
        self.life -= dt;

        !(self.pos.x < 1.0
            || self.pos.y < 1.0
            || self.pos.x >= SIZE.w as f64 - 1.0
            || self.pos.y >= SIZE.h as f64 - 1.0)
            && self.life > 0.0
    }

    pub fn render(&self, canvas: &mut [u32]) {
        let index = self.pos.x as usize + self.pos.y as usize * SIZE.w;
        canvas[index - SIZE.w] = self.color.as_u32();
        canvas[(index - 1)..(index + 2)].fill(self.color.as_u32());
        canvas[index + SIZE.w] = self.color.as_u32();
    }
}

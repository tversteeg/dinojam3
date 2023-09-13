use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::{Extent2, Rect, Vec2};

use crate::{
    card::{Card, CARDS, CARD_PATHS},
    graphics::Color,
    input::Input,
    math::Iso,
    object::Object,
    particle::Particle,
    timer::Timer,
    SIZE,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Phase {
    Buy,
    LaunchSetAngle,
    LaunchSetSpeed,
    Fly,
    Dead,
}

/// Handles everything related to the game.
pub struct GameState {
    phase: Phase,
    pub initial_angle: f64,
    pub sign: f64,
    pub initial_speed: f64,
    pub pos: Vec2<f64>,
    pub vel: Vec2<f64>,
    pub rot: f64,
    pub money: usize,
    pub boost: f64,
    pub boost_sign: f64,
    pub boost_delay: f64,
    pub buy_timeout: f64,
    pub buy_item: f64,
    pub trees: Vec<Object>,
    pub clouds: Vec<Object>,
    pub disks: Vec<Object>,
    pub rocks: Vec<Object>,
    pub particles: Vec<Particle>,
    pub card_options: [Card; 3],
    pub selected_cards: [usize; CARDS],
    pub extra_initial_speed: f64,
    pub extra_gravity: f64,
    pub dead_timeout: f64,
    pub max_distance: f64,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let settings = crate::settings();
        let trees = crate::objects("palm").to_objects();
        let clouds = crate::objects("cloud").to_objects();
        let disks = crate::objects("disk").to_objects();
        let rocks = crate::objects("rock").to_objects();

        let mut state = Self {
            phase: Phase::Buy,
            initial_angle: settings.min_angle,
            initial_speed: settings.min_speed,
            extra_initial_speed: 0.0,
            buy_timeout: settings.buy_time,
            particles: Vec::new(),
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            rot: 0.0,
            money: 0,
            sign: 1.0,
            boost: 0.0,
            boost_sign: 1.0,
            boost_delay: 0.0,
            buy_item: 0.0,
            trees,
            clouds,
            disks,
            rocks,
            card_options: [Card::default(), Card::default(), Card::default()],
            selected_cards: [0; CARDS],
            extra_gravity: 0.0,
            dead_timeout: 0.0,
            max_distance: 0.0,
        };

        state.switch_to_buy();

        state
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f64) {
        let settings = crate::settings();

        if self.phase != Phase::Dead {
            self.clouds
                .iter_mut()
                .chain(self.trees.iter_mut())
                .chain(self.disks.iter_mut())
                .chain(self.rocks.iter_mut())
                .for_each(|obj| obj.update(self.pos, self.vel, settings.player_offset, dt));

            self.particles
                .retain_mut(|particle| particle.update(self.vel, settings.particle_gravity, dt));

            self.disks.iter_mut().for_each(|disk| {
                if disk.collides_user(settings.player_collider) {
                    self.money += 1;

                    for _ in 0..settings.particle_amount {
                        self.particles.push(Particle::new(
                            disk.pos,
                            self.vel * settings.particle_vel_multiplier,
                            settings.particle_force,
                            Color::Brown,
                            true,
                            settings.particle_life,
                        ));
                    }
                    for _ in 0..settings.topleft_particle_amount {
                        self.particles.push(Particle::new(
                            Vec2::new(33.0, 10.0),
                            Vec2::zero(),
                            settings.topleft_particle_force,
                            Color::White,
                            false,
                            settings.topleft_particle_life,
                        ));
                    }

                    disk.reset(self.pos, self.vel);
                }
            });
        }

        match self.phase {
            Phase::Buy => {
                self.buy_timeout -= dt;
                if self.buy_timeout <= 0.0 {
                    self.phase = Phase::LaunchSetAngle;
                } else if input.left_mouse.is_released() {
                    self.phase = Phase::LaunchSetAngle;

                    let index = self.buy_item.floor().clamp(0.0, 3.0) as usize;
                    let card = self.card_options[index].clone();
                    card.apply(self);
                }

                self.pos = Vec2::zero();
                self.vel = Vec2::zero();
                self.rot = 0.0;

                self.buy_item += settings.buy_speed * dt;
                if self.buy_item >= 3.0 {
                    self.buy_item -= 3.0;
                }
            }
            Phase::LaunchSetAngle => {
                if input.left_mouse.is_released() {
                    self.phase = Phase::LaunchSetSpeed;
                    self.sign = 1.0;
                }

                self.initial_angle += settings.angle_delta * self.sign * dt;
                if self.initial_angle > settings.max_angle {
                    self.sign = -1.0;
                } else if self.initial_angle < settings.min_angle {
                    self.sign = 1.0;
                }

                self.pos = Vec2::zero();
                self.vel = Vec2::zero();
                self.rot = 0.0;
            }
            Phase::LaunchSetSpeed => {
                if input.left_mouse.is_released() {
                    self.phase = Phase::Fly;
                    self.sign = 1.0;

                    self.vel = Vec2::new(self.initial_angle.cos(), self.initial_angle.sin())
                        * (self.initial_speed + self.extra_initial_speed);
                    self.boost_delay = settings.boost_delay;
                }

                self.initial_speed += settings.speed_delta * self.sign * dt;
                if self.initial_speed > settings.max_speed {
                    self.sign = -1.0;
                } else if self.initial_speed < settings.min_speed {
                    self.sign = 1.0;
                }
            }
            Phase::Fly => {
                self.pos += self.vel * dt;
                self.vel.y += (settings.gravity + self.extra_gravity) * dt;
                self.vel *= settings.air_friction;
                self.rot += (self.vel.x * settings.rot_factor.x
                    + self
                        .vel
                        .y
                        .clamp(-settings.rot_y_clamp, settings.rot_y_clamp)
                        * settings.rot_factor.y)
                    * dt;

                self.boost += self.boost_sign * settings.boost_meter_speed * dt;
                if self.boost_delay > 0.0 {
                    self.boost_delay -= dt;
                }
                if self.boost
                    >= settings.boost_meter_penalty_area
                        + settings.boost_meter_safe_area
                        + settings.boost_meter_crit_area
                {
                    self.boost_sign = -1.0;
                    self.boost = settings.boost_meter_penalty_area
                        + settings.boost_meter_safe_area
                        + settings.boost_meter_crit_area;
                } else if self.boost <= 0.0 {
                    self.boost_sign = 1.0;
                    self.boost = 0.0;
                }

                if self.boost_delay <= 0.0 && input.left_mouse.is_released() {
                    let boost = if self.boost
                        > settings.boost_meter_safe_area + settings.boost_meter_penalty_area
                    {
                        for _ in 0..settings.crit_particle_amount {
                            self.particles.push(Particle::new(
                                settings.player_offset,
                                Vec2::zero(),
                                settings.crit_particle_force,
                                Color::DarkGreen,
                                false,
                                settings.crit_particle_life,
                            ));
                        }

                        settings.boost_crit
                    } else if self.boost > settings.boost_meter_penalty_area {
                        settings.boost_safe
                    } else {
                        settings.boost_penalty
                    };

                    self.vel.x = (self.vel.x * boost.x).min(settings.max_boost_velocity.x);
                    self.vel.y = (self.vel.y * boost.y).min(settings.max_boost_velocity.y);

                    if boost.x > 1.0
                        && self.vel.magnitude() > settings.static_velocity_boost_treshold
                    {
                        self.vel.x += settings.static_velocity_boost.x;
                        self.vel.y += settings.static_velocity_boost.y * self.vel.y.signum();
                    }

                    self.boost = fastrand::f64()
                        * (settings.boost_meter_penalty_area
                            + settings.boost_meter_safe_area
                            + settings.boost_meter_crit_area);
                    self.boost_delay = settings.boost_delay;
                }

                if self.pos.y > 0.0 {
                    if self.vel.x.abs() < settings.halting_velocity.x
                        && self.vel.y.abs() < settings.halting_velocity.y
                    {
                        self.phase = Phase::Dead;
                        self.dead_timeout = settings.dead_wait_time;
                        self.max_distance = self.max_distance.max(self.pos.x);
                    } else {
                        self.pos.y = 0.0;
                        self.vel.x *= settings.restitution.x;
                        self.vel.y = -self.vel.y.abs() * settings.restitution.y;
                        for _ in 0..settings.bounce_particle_amount {
                            self.particles.push(Particle::new(
                                settings.player_offset,
                                self.vel * settings.bounce_particle_vel_multiplier,
                                settings.bounce_particle_force,
                                Color::Green,
                                true,
                                settings.bounce_particle_life,
                            ));
                        }
                    }
                }
            }
            Phase::Dead => {
                self.dead_timeout -= dt;
                if self.dead_timeout <= 0.0 {
                    self.switch_to_buy();
                }
            }
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], _frame_time: f64) {
        let settings = crate::settings();

        self.clouds
            .iter_mut()
            .chain(self.trees.iter_mut())
            .chain(self.disks.iter_mut())
            .for_each(|obj| obj.render(canvas));

        let ground_height =
            (settings.player_offset.y - self.pos.y).clamp(0.0, SIZE.h as f64) as usize;
        canvas[(ground_height * SIZE.w)..].fill(Color::LightGreen.as_u32());
        let edge_ground_height =
            (settings.player_offset.y - self.pos.y - 3.0).clamp(0.0, SIZE.h as f64) as usize;
        canvas[(edge_ground_height * SIZE.w)..(ground_height * SIZE.w)].fill(Color::Green.as_u32());

        self.rocks.iter_mut().for_each(|obj| obj.render(canvas));

        self.particles
            .iter()
            .for_each(|particle| particle.render(canvas));

        //crate::render_aabr(settings.player_collider.into_aabr(), canvas, 0xFFFF0000);

        match self.phase {
            Phase::Buy => {
                crate::sprite("buy-screen").render(canvas, Vec2::zero());
                let font = crate::font();
                let item_str = "Skipped in:";
                font.render(
                    item_str,
                    Vec2::new(
                        SIZE.w / 2 - item_str.len() * font.char_size.w as usize / 2,
                        2,
                    )
                    .as_(),
                    canvas,
                );
                font.render(
                    &format!("{}", self.buy_timeout.round()),
                    Vec2::new(SIZE.w / 2 - font.char_size.w as usize / 2, 18).as_(),
                    canvas,
                );

                let index = self.buy_item.floor().clamp(0.0, 3.0) as usize;
                crate::sprite("buy-screen-selected-card")
                    .render(canvas, Vec2::new([20, 116, 212][index] as f64, 64.0));

                let buy_offset: Vec2<usize> = (settings.buy_meter_offset).as_();
                for y in buy_offset.y..(buy_offset.y + settings.buy_meter_size.h as usize - 4) {
                    let start = y * SIZE.w + buy_offset.x;
                    let buy_frac = self.buy_item / 3.0;
                    let x4 = start + (buy_frac * (settings.buy_meter_size.w - 4.0)) as usize;
                    canvas[x4..(x4 + 3)].fill(Color::White.as_u32());
                }

                self.card_options[0].render(Vec2::new(20.0, 64.0), canvas, &self.selected_cards);
                self.card_options[1].render(Vec2::new(116.0, 64.0), canvas, &self.selected_cards);
                self.card_options[2].render(Vec2::new(212.0, 64.0), canvas, &self.selected_cards);

                let pos = Vec2::new(3, 3).as_();
                crate::font().render(&format!("  {}", self.money,), pos, canvas);

                let disk = crate::sprite("disk-icon");
                disk.render(canvas, pos - (1.0, 1.0));
            }
            Phase::LaunchSetAngle => {
                crate::font().render("Click to set the angle!", Vec2::new(10, 10).as_(), canvas);
            }
            Phase::LaunchSetSpeed => {
                crate::font().render("Click to set the speed!", Vec2::new(10, 10).as_(), canvas);
                let speed_offset: Vec2<usize> = (settings.speed_meter_offset).as_();

                let speed_bar = crate::sprite("speed-bar");
                speed_bar.render(canvas, speed_offset.as_() - (2.0, 2.0));
                for y in speed_offset.y..(speed_offset.y + speed_bar.height() as usize - 4) {
                    let start = y * SIZE.w + speed_offset.x;
                    let x4 = start
                        + ((self.initial_speed - settings.min_speed)
                            / (settings.max_speed - settings.min_speed)
                            * (speed_bar.width() - 4) as f64) as usize;
                    canvas[x4..(x4 + 3)].fill(Color::White.as_u32());
                }
            }
            Phase::Dead | Phase::Fly => {
                crate::rotatable_sprite("dino1")
                    .render(Iso::new(settings.player_offset, self.rot), canvas);

                let pos = Vec2::new(3, 3).as_();
                crate::font().render(
                    &format!(
                        "  {:<4} [ {:<7} ] {}",
                        self.money,
                        self.pos.x.round(),
                        self.pos.y.abs().round(),
                    ),
                    pos,
                    canvas,
                );

                let disk = crate::sprite("disk-icon");
                disk.render(canvas, pos - (1.0, 1.0));
            }
        }

        if self.phase == Phase::Fly && self.boost_delay <= 0.0 {
            let boost_offset: Vec2<usize> = (settings.boost_meter_offset).as_();

            let boost_bar = crate::sprite("boost-bar");
            boost_bar.render(canvas, boost_offset.as_() - (2.0, 2.0));
            for y in boost_offset.y..(boost_offset.y + boost_bar.height() as usize - 4) {
                let start = y * SIZE.w + boost_offset.x;
                let boost_frac = self.boost
                    / (settings.boost_meter_safe_area
                        + settings.boost_meter_penalty_area
                        + settings.boost_meter_crit_area);
                let x4 = start + (boost_frac * (boost_bar.width() as f64 - 4.0)) as usize;
                canvas[x4..(x4 + 3)].fill(Color::White.as_u32());
            }
        }

        if self.phase == Phase::Dead {
            crate::font().render_centered(
                &format!("Distance: {}", self.pos.x.round()),
                Vec2::new(SIZE.w as f64 / 2.0, SIZE.h as f64 / 2.0 - 50.0),
                canvas,
            );
            crate::font().render_centered(
                &format!("Max Distance: {}", self.max_distance.round()),
                Vec2::new(SIZE.w as f64 / 2.0, SIZE.h as f64 / 2.0 - 30.0),
                canvas,
            );
        }

        if self.phase != Phase::Buy && self.pos.x < SIZE.w as f64 {
            crate::rotatable_sprite("cannon").render(
                Iso::new(
                    -self.pos + settings.cannon_offset,
                    self.initial_angle + std::f64::consts::FRAC_PI_2,
                ),
                canvas,
            );
        }
    }

    fn switch_to_buy(&mut self) {
        let settings = crate::settings();

        self.phase = Phase::Buy;
        self.buy_timeout = settings.buy_time;
        self.buy_item = fastrand::f64() * 3.0;

        self.initial_angle = settings.min_angle;
        self.initial_speed = settings.min_speed;

        self.clouds
            .iter_mut()
            .chain(self.trees.iter_mut())
            .for_each(|obj| obj.reset(Vec2::zero(), Vec2::zero()));

        self.card_options.iter_mut().for_each(|card| {
            *card = Card::random(self.money, &self.selected_cards);
        });
    }
}

/// Game settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    pub min_angle: f64,
    pub max_angle: f64,
    pub angle_delta: f64,
    pub min_speed: f64,
    pub max_speed: f64,
    pub speed_delta: f64,
    pub gravity: f64,
    pub cannon_offset: Vec2<f64>,
    pub player_offset: Vec2<f64>,
    pub boost_meter_offset: Vec2<f64>,
    pub speed_meter_offset: Vec2<f64>,
    pub buy_meter_offset: Vec2<f64>,
    pub buy_meter_size: Extent2<f64>,
    pub rot_factor: Vec2<f64>,
    pub rot_y_clamp: f64,
    pub air_friction: f64,
    pub restitution: Vec2<f64>,
    pub halting_velocity: Vec2<f64>,
    pub tree_amount: usize,
    pub rock_amount: usize,
    pub boost_meter_speed: f64,
    pub boost_meter_penalty_area: f64,
    pub boost_meter_safe_area: f64,
    pub boost_meter_crit_area: f64,
    pub boost_penalty: Vec2<f64>,
    pub boost_crit: Vec2<f64>,
    pub boost_safe: Vec2<f64>,
    pub boost_delay: f64,
    pub max_boost_velocity: Vec2<f64>,
    pub static_velocity_boost: Vec2<f64>,
    pub static_velocity_boost_treshold: f64,
    pub player_collider: Rect<f64, f64>,
    pub particle_amount: usize,
    pub particle_gravity: f64,
    pub particle_force: f64,
    pub particle_vel_multiplier: Vec2<f64>,
    pub particle_life: f64,
    pub topleft_particle_amount: usize,
    pub topleft_particle_gravity: f64,
    pub topleft_particle_force: f64,
    pub topleft_particle_life: f64,
    pub crit_particle_amount: usize,
    pub crit_particle_gravity: f64,
    pub crit_particle_force: f64,
    pub crit_particle_life: f64,
    pub bounce_particle_amount: usize,
    pub bounce_particle_gravity: f64,
    pub bounce_particle_force: f64,
    pub bounce_particle_vel_multiplier: Vec2<f64>,
    pub bounce_particle_life: f64,
    pub buy_time: f64,
    pub buy_speed: f64,
    pub dead_wait_time: f64,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

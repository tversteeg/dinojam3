use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    camera::Camera,
    graphics::Color,
    input::Input,
    math::Iso,
    physics::{Physics, Settings as PhysicsSettings},
    terrain::Settings as TerrainSettings,
    terrain::Terrain,
    timer::Timer,
    unit::{Unit, UnitType},
    SIZE,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Phase {
    LaunchSetAngle,
    LaunchSetSpeed,
    Fly,
}

/// Handles everything related to the game.
pub struct GameState {
    /// Enemies on the map.
    enemies: Vec<Unit>,
    /// Camera position based on the cursor.
    camera: Camera,
    /// Physics engine.
    ///
    /// Size of the grid is the maximum size of any map.
    physics: Physics,
    phase: Phase,
    initial_angle: f64,
    sign: f64,
    initial_speed: f64,
    pos: Vec2<f64>,
    vel: Vec2<f64>,
    rot: f64,
    boost: f64,
    boost_sign: f64,
    boost_delay: f64,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let enemies = Vec::new();
        let mut unit_spawner = Timer::new(crate::settings().unit_spawn_interval);
        unit_spawner.trigger();
        let camera = Camera::default();
        let mut physics = Physics::new();

        Self {
            enemies,
            camera,
            physics,
            phase: Phase::LaunchSetAngle,
            initial_angle: crate::settings().min_angle,
            initial_speed: crate::settings().min_speed,
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            rot: 0.0,
            sign: 1.0,
            boost: 0.0,
            boost_sign: 1.0,
            boost_delay: 0.0,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], _frame_time: f64) {
        let settings = crate::settings();

        let mut player_pos = settings.player_offset + (0.0, SIZE.h as f64 / 2.0);
        if self.phase != Phase::Fly {
            player_pos += settings.initial_camera;
            self.camera
                .pan(settings.initial_camera.x, settings.initial_camera.y, 0.0);
        }

        let tree = crate::sprite("palm");
        for i in 0..(settings.tree_amount * 2) {
            tree.render(
                canvas,
                &Camera::default(),
                Vec2::new(
                    (-(self.camera.x * 0.8) % SIZE.w as f64)
                        + i as f64 * (SIZE.w / settings.tree_amount) as f64,
                    -(self.camera.y * 0.95) + tree.height() as f64 - tree.height() as f64 / 4.0
                        + 10.0,
                ),
            );
        }
        for i in 0..((settings.tree_amount - 1) * 2) {
            tree.render(
                canvas,
                &Camera::default(),
                Vec2::new(
                    (-(self.camera.x * 0.9) % SIZE.w as f64)
                        + i as f64 * (SIZE.w / (settings.tree_amount - 1)) as f64
                        + 5.0,
                    -self.camera.y + tree.height() as f64 - tree.height() as f64 / 4.0 - 3.0,
                ),
            );
        }

        let ground_height = SIZE.h - ((self.pos.y + player_pos.y).max(0.0) as usize).min(SIZE.h);
        canvas[(ground_height * SIZE.w)..].fill(Color::LightGreen.as_u32());
        let edge_ground_height =
            SIZE.h - ((self.pos.y + player_pos.y + 3.0).max(0.0) as usize).min(SIZE.h);
        canvas[(edge_ground_height * SIZE.w)..(ground_height * SIZE.w)].fill(Color::Green.as_u32());

        let rock = crate::sprite("rock");
        rock.render(
            canvas,
            &Camera::default(),
            Vec2::new(
                (-self.camera.x % (SIZE.w as f64 * 2.3)) + SIZE.w as f64,
                -self.camera.y + player_pos.y - rock.height() as f64 / 2.0,
            ),
        );

        let cloud = crate::sprite("cloud");
        for i in 0..20 {
            cloud.render(
                canvas,
                &Camera::default(),
                Vec2::new(
                    (-(self.camera.x * 0.5) % (SIZE.w as f64 * 2.3 + i as f64 * 130.0))
                        + SIZE.w as f64,
                    -(self.camera.y * 0.5) - i as f64 * 200.0 - 100.0,
                ),
            );
        }

        match self.phase {
            Phase::LaunchSetAngle => {
                crate::font().render("Click to set the angle!", Vec2::new(10, 10).as_(), canvas);
            }
            Phase::LaunchSetSpeed => {
                crate::font().render("Click to set the speed!", Vec2::new(10, 10).as_(), canvas);
                let speed_offset: Vec2<usize> = (settings.speed_meter_offset + player_pos).as_();
                for y in speed_offset.y..(speed_offset.y + settings.speed_meter_size.h) {
                    let start = y * SIZE.w + speed_offset.x;
                    let x = start + settings.speed_meter_size.w / 2;
                    canvas[start..x].fill(Color::Salmon.as_u32());

                    let x2 = x + settings.speed_meter_size.w / 2;

                    canvas[x..x2].fill(Color::LightGreen.as_u32());

                    let x4 = start
                        + ((self.initial_speed - settings.min_speed)
                            / (settings.max_speed - settings.min_speed)
                            * settings.speed_meter_size.w as f64)
                            as usize;
                    canvas[x4..(x4 + 3)].fill(Color::White.as_u32());
                }
            }
            Phase::Fly => {
                crate::rotatable_sprite("dino1").render(
                    Iso::new(player_pos, self.rot),
                    canvas,
                    &Default::default(),
                );

                if self.boost_delay <= 0.0 {
                    let boost_offset: Vec2<usize> =
                        (settings.boost_meter_offset + player_pos).as_();
                    for y in boost_offset.y..(boost_offset.y + settings.boost_meter_height) {
                        let start = y * SIZE.w + boost_offset.x;
                        let x = start + settings.boost_meter_penalty_area as usize;
                        canvas[start..x].fill(Color::Salmon.as_u32());

                        let x2 = x + settings.boost_meter_safe_area as usize;

                        canvas[x..x2].fill(Color::LightGreen.as_u32());

                        let x3 = x2 + settings.boost_meter_crit_area as usize;

                        canvas[x2..x3].fill(Color::Green.as_u32());

                        let x4 = start + self.boost as usize;
                        canvas[x4..(x4 + 3)].fill(Color::White.as_u32());
                    }
                }

                crate::font().render(
                    &format!(
                        "Distance: {:<7} Height: {}",
                        self.pos.x.round(),
                        self.pos.y.abs().round()
                    ),
                    Vec2::new(0, 0).as_(),
                    canvas,
                );
            }
        }

        if self.pos.x < SIZE.w as f64 {
            crate::rotatable_sprite("cannon").render(
                Iso::new(
                    player_pos + settings.cannon_offset,
                    self.initial_angle + std::f64::consts::FRAC_PI_2,
                ),
                canvas,
                &self.camera,
            );
        }

        // Render all units
        self.enemies
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera));
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f64) {
        let settings = crate::settings();

        self.camera.pan(self.pos.x, self.pos.y, 0.0);

        match self.phase {
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
                        * self.initial_speed;
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
                self.vel.y += settings.gravity * dt;
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
                        self.phase = Phase::LaunchSetAngle;

                        self.initial_angle = settings.min_angle;
                        self.initial_speed = settings.min_speed;
                    }
                    self.pos.y = 0.0;
                    self.vel.x *= settings.restitution.x;
                    self.vel.y = -self.vel.y.abs() * settings.restitution.y;
                }
            }
        }
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
    pub initial_camera: Vec2<f64>,
    pub cannon_offset: Vec2<f64>,
    pub player_offset: Vec2<f64>,
    pub boost_meter_offset: Vec2<f64>,
    pub boost_meter_height: usize,
    pub speed_meter_offset: Vec2<f64>,
    pub speed_meter_size: Extent2<usize>,
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
    /// Distance from the edge at which the camera will pan.
    pub pan_edge_offset: i32,
    /// How many pixels per second the camera will pan.
    pub pan_speed: f64,
    /// Interval in seconds for when a unit spawns.
    pub unit_spawn_interval: f64,
    /// Interval in seconds for when an enemy unit spawns.
    pub enemy_unit_spawn_interval: f64,
    /// Physics settings.
    pub physics: PhysicsSettings,
    /// Terrain settings.
    pub terrain: TerrainSettings,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

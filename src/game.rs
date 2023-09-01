use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::Vec2;

use crate::{
    camera::Camera,
    input::Input,
    physics::{Physics, Settings as PhysicsSettings},
    terrain::Settings as TerrainSettings,
    terrain::Terrain,
    timer::Timer,
    unit::{Unit, UnitType},
    SIZE,
};

/// Handles everything related to the game.
pub struct GameState {
    /// First level ground.
    terrain: Terrain,
    /// Enemies on the map.
    enemies: Vec<Unit>,
    /// Camera position based on the cursor.
    camera: Camera,
    /// Physics engine.
    ///
    /// Size of the grid is the maximum size of any map.
    physics: Physics,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let enemies = Vec::new();
        let mut unit_spawner = Timer::new(crate::settings().unit_spawn_interval);
        unit_spawner.trigger();
        let enemy_unit_spawner = Timer::new(crate::settings().enemy_unit_spawn_interval);
        let camera = Camera::default();
        let mut physics = Physics::new();
        let terrain = Terrain::new(&mut physics);

        Self {
            terrain,
            enemies,
            camera,
            physics,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], _frame_time: f64) {
        self.terrain.render(canvas, &self.camera);

        // Render all units
        self.enemies
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera));
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f64) {
        let settings = crate::settings();

        // Move the camera based on the mouse position
        if input.mouse_pos.x <= settings.pan_edge_offset {
            self.camera.pan(
                -settings.pan_speed * dt,
                0.0,
                0.0,
                (settings.terrain.width - SIZE.w as u32) as f64,
            );
        } else if input.mouse_pos.x >= SIZE.w as i32 - settings.pan_edge_offset {
            self.camera.pan(
                settings.pan_speed * dt,
                0.0,
                0.0,
                (settings.terrain.width - SIZE.w as u32) as f64,
            );
        }

        // Simulate the physics
        self.physics.step(dt);

        // Update all units
        self.enemies.iter_mut().for_each(|unit| {
            if let Some(projectile) = unit.update(&self.terrain, dt, &mut self.physics) {
                todo!()
            }
        });
    }
}

/// Game settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
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

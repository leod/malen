use malen::DrawParams;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use malen::nalgebra::{Point2, Vector2};

use malen::{
    geometry::{ColorRect, ColorTriangleBatch},
    pass::Matrices,
    Camera, Color4, Context, Error, InitError, InputState, Key, Rect, UniformBuffer,
};

struct Wall {
    center: Point2<f32>,
    size: Vector2<f32>,
}

struct Enemy {
    center: Point2<f32>,
    angle: f32,
}

struct State {
    walls: Vec<Wall>,
    enemies: Vec<Enemy>,
    player_pos: Point2<f32>,
    last_timestamp_secs: Option<f64>,
}

impl State {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(200.0, 150.0).unwrap();
        let walls = (0..50)
            .map(|_| {
                let center =
                    Point2::new(rng.gen(), rng.gen()) * 4096.0 - Vector2::new(1.0, 1.0) * 2048.0;
                let size = Vector2::new(normal.sample(&mut rng), normal.sample(&mut rng));

                Wall { center, size }
            })
            .collect();

        let num_enemies = 30;
        let enemies = (0..num_enemies)
            .map(|_| {
                let center =
                    Point2::new(rng.gen(), rng.gen()) * 4096.0 - Vector2::new(1.0, 1.0) * 2048.0;

                Enemy {
                    center,
                    angle: rng.gen::<f32>() * std::f32::consts::PI,
                }
            })
            .collect();

        Self {
            walls,
            enemies,
            player_pos: Point2::origin(),
            last_timestamp_secs: None,
        }
    }

    pub fn update(&mut self, timestamp_secs: f64, input_state: &InputState) {
        let dt_secs = self
            .last_timestamp_secs
            .map_or(0.0, |last_timestamp_secs| {
                timestamp_secs - last_timestamp_secs
            })
            .max(0.0) as f32;
        self.last_timestamp_secs = Some(timestamp_secs);

        let mut player_dir = Vector2::zeros();
        if input_state.key(Key::W) {
            player_dir.y -= 1.0;
        }
        if input_state.key(Key::S) {
            player_dir.y += 1.0;
        }
        if input_state.key(Key::A) {
            player_dir.x -= 1.0;
        }
        if input_state.key(Key::D) {
            player_dir.x += 1.0;
        }
        if player_dir.norm_squared() > 0.0 {
            let player_dir = player_dir.normalize();
            self.player_pos += dt_secs * 500.0 * player_dir;
        }

        for (i, thingy) in self.enemies.iter_mut().enumerate() {
            let mut delta = 0.2 * std::f32::consts::PI * dt_secs;
            if i % 2 == 0 {
                delta *= -1.0;
            }
            thingy.angle += delta;
        }
    }
}

struct Game {
    state: State,

    matrices: UniformBuffer<Matrices>,
    color_triangles: ColorTriangleBatch,
}

impl Game {
    pub fn new(context: &Context) -> Result<Game, InitError> {
        let state = State::new();
        let matrices = UniformBuffer::new(context.gl(), Matrices::default())?;
        let color_triangles = ColorTriangleBatch::new(context.gl())?;

        Ok(Game {
            state,
            matrices,
            color_triangles,
        })
    }

    pub fn render(&mut self) {
        self.color_triangles.clear();

        for wall in &self.state.walls {
            self.color_triangles.push(ColorRect {
                rect: Rect {
                    center: wall.center,
                    size: wall.size,
                },
                z: 0.0,
                color: Color4::new(0.2, 0.2, 0.8, 1.0),
            });
        }

        for enemy in &self.state.enemies {
            self.color_triangles.push(ColorRect {
                rect: Rect {
                    center: enemy.center,
                    size: Vector2::new(30.0, 30.0),
                },
                z: 0.0,
                color: Color4::new(0.2, 0.8, 0.2, 1.0),
            })
        }
    }

    pub fn draw(&mut self, context: &Context) -> Result<(), Error> {
        let screen = context.screen();
        let camera = Camera {
            center: self.state.player_pos,
            zoom: 1.0,
            angle: 0.0,
        };

        self.matrices.set_data(Matrices {
            view: camera.to_matrix(&screen),
            projection: screen.orthographic_projection(),
        });

        context.clear(Color4::new(0.0, 0.0, 0.0, 1.0));
        context.draw_colors(
            &self.matrices,
            self.color_triangles.draw_unit(),
            &DrawParams::default(),
        );

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Starting the malen example");

    let mut context = Context::from_canvas_element_id("canvas").unwrap();
    log::info!("Initialized malen context");

    let mut game = Game::new(&context).unwrap();

    malen::main_loop(move |timestamp_secs, _running| {
        use malen::Event::*;

        while let Some(event) = context.pop_event() {
            match event {
                Focused => {
                    log::info!("Canvas got focus");
                }
                Unfocused => {
                    log::info!("Canvas lost focus");
                }
                _ => (),
            }
        }

        context.resize_fill();

        game.state.update(timestamp_secs, context.input_state());
        game.render();
        game.draw(&context).unwrap();
    });
}

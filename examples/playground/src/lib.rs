use rand::Rng;
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{
    geometry::{ColorRect, ColorRotatedRect, ColorTriangleBatch},
    gl::{self, DepthTest},
    nalgebra::{Point2, Vector2},
    pass::Matrices,
    Camera, Color4, Context, DrawParams, Error, InitError, InputState, Key, Rect, UniformBuffer,
};

struct Wall {
    center: Point2<f32>,
    size: Vector2<f32>,
}

struct Enemy {
    pos: Point2<f32>,
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
        let map_size = 2048.0;
        let num_walls = 50;
        let num_enemies = 30;

        let mut rng = rand::thread_rng();
        let walls = (0..num_walls)
            .map(|_| {
                let center = Point2::new(rng.gen(), rng.gen()) * 2.0 * map_size
                    - Vector2::new(1.0, 1.0) * map_size;

                let choice = rng.gen_range(0, 3);
                let size = match choice {
                    0 => {
                        let x = rng.gen_range(50.0, 500.0);
                        Vector2::new(x, x)
                    }
                    1 => Vector2::new(50.0, rng.gen_range(100.0, 1000.0)),
                    2 => Vector2::new(rng.gen_range(100.0, 1000.0), 50.0),
                    _ => unreachable!(),
                };

                Wall { center, size }
            })
            .collect();

        let enemies = (0..num_enemies)
            .map(|_| {
                let pos = Point2::new(rng.gen(), rng.gen()) * 2.0 * map_size
                    - Vector2::new(1.0, 1.0) * map_size;

                Enemy {
                    pos,
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
                z: 1.0,
                color: Color4::new(0.2, 0.2, 0.8, 1.0),
            });
        }

        for enemy in &self.state.enemies {
            self.color_triangles.push(ColorRect {
                rect: Rect {
                    center: enemy.pos,
                    size: Vector2::new(30.0, 30.0),
                },
                z: 0.0,
                color: Color4::new(0.8, 0.2, 0.2, 1.0),
            })
        }

        self.color_triangles.push(ColorRect {
            rect: Rect {
                center: self.state.player_pos,
                size: Vector2::new(50.0, 50.0),
            },
            z: 0.5,
            color: Color4::new(0.2, 0.8, 0.2, 1.0),
        });
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
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
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

    let mut gl_timer = gl::Timer::new(context.gl(), 60);

    malen::main_loop(move |timestamp_secs, _running| {
        coarse_prof::profile!("frame");

        while let Some(event) = context.pop_event() {
            use malen::Event::*;

            match event {
                Focused => {
                    log::info!("Canvas got focus");
                }
                Unfocused => {
                    log::info!("Canvas lost focus");
                }
                KeyPressed(Key::P) => {
                    let mut buffer = std::io::Cursor::new(Vec::new());
                    coarse_prof::write(&mut buffer).unwrap();
                    coarse_prof::reset();
                    log::info!(
                        "Profiling: {}",
                        std::str::from_utf8(buffer.get_ref()).unwrap()
                    );
                    log::info!("GL timer: {:?}", gl_timer.timing_info(),);
                }
                _ => (),
            }
        }

        context.resize_fill();

        game.state.update(timestamp_secs, context.input_state());
        game.render();

        gl_timer.start_draw();
        game.draw(&context).unwrap();
        gl_timer.end_draw();
    });
}

use nalgebra::{Point2, Vector2};
use rand::Rng;
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{
    geometry::{ColorRect, ColorRotatedRect, ColorTriangleBatch, Sprite, SpriteBatch},
    gl::{DepthTest, DrawParams, FrameTimer, Texture, TextureParams, UniformBuffer},
    Camera, CanvasSizeConfig, Color4, Config, Context, Error, InitError, InputState, Key,
    MatrixBlock, Rect, Screen,
};

struct Wall {
    center: Point2<f32>,
    size: Vector2<f32>,
}

struct Enemy {
    pos: Point2<f32>,
    angle: f32,
}

struct Player {
    pos: Point2<f32>,
    angle: f32,
}

struct State {
    walls: Vec<Wall>,
    enemies: Vec<Enemy>,
    player: Player,
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
            player: Player {
                pos: Point2::origin(),
                angle: 0.0,
            },
            last_timestamp_secs: None,
        }
    }

    pub fn camera(&self) -> Camera {
        Camera {
            center: self.player.pos,
            zoom: 1.0,
            angle: 0.0,
        }
    }

    pub fn update(&mut self, timestamp_secs: f64, screen: Screen, input_state: &InputState) {
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
            self.player.pos += dt_secs * 500.0 * player_dir;
        }

        self.player.angle = {
            let mouse_logical_pos = input_state.mouse_logical_pos().cast::<f32>();
            let mouse_world_pos = self
                .camera()
                .inverse_matrix(screen)
                .transform_point(&mouse_logical_pos);
            let offset = mouse_world_pos - self.player.pos;
            offset.y.atan2(offset.x)
        };

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

    wall_texture: Texture,
    matrix_buffer: UniformBuffer<MatrixBlock>,
    color_batch: ColorTriangleBatch,
    wall_sprite_batch: SpriteBatch,
}

impl Game {
    pub fn new(context: &Context) -> Result<Game, InitError> {
        let state = State::new();

        let wall_texture = Texture::from_encoded_bytes(
            context.gl(),
            include_bytes!("../resources/04muroch256.png"),
            TextureParams::default(),
        )?;
        let matrix_buffer = UniformBuffer::new(context.gl(), MatrixBlock::default())?;
        let color_batch = ColorTriangleBatch::new(context.gl())?;
        let wall_sprite_batch = SpriteBatch::new(context.gl())?;

        Ok(Game {
            state,
            wall_texture,
            matrix_buffer,
            color_batch,
            wall_sprite_batch,
        })
    }

    pub fn render(&mut self) {
        self.color_batch.clear();
        self.wall_sprite_batch.clear();

        for wall in &self.state.walls {
            self.wall_sprite_batch.push(Sprite {
                rect: Rect {
                    center: wall.center,
                    size: wall.size,
                },
                tex_rect: Rect::from_bottom_left(Point2::origin(), wall.size),
                z: 0.2,
            });
        }

        for enemy in &self.state.enemies {
            self.color_batch.push(ColorRect {
                rect: Rect {
                    center: enemy.pos,
                    size: Vector2::new(30.0, 30.0),
                },
                z: 0.3,
                color: Color4::new(0.8, 0.2, 0.2, 1.0),
            })
        }

        self.color_batch.push(ColorRotatedRect {
            rect: Rect {
                center: self.state.player.pos,
                size: Vector2::new(50.0, 50.0),
            }
            .rotate(self.state.player.angle),
            z: 0.4,
            color: Color4::new(0.2, 0.8, 0.2, 1.0),
        });
    }

    pub fn draw(&mut self, context: &Context) -> Result<(), Error> {
        let screen = context.screen();
        self.matrix_buffer.set_data(MatrixBlock {
            view: self.state.camera().matrix(screen),
            projection: screen.orthographic_projection(),
        });

        context.clear(Color4::new(1.0, 1.0, 1.0, 1.0));
        context.draw_colors(
            &self.matrix_buffer,
            self.color_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );
        context.draw_sprites(
            &self.matrix_buffer,
            &self.wall_texture,
            self.wall_sprite_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        )?;

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Starting the malen example");

    let config = Config {
        canvas_size: CanvasSizeConfig::Fill,
        //canvas_size: CanvasSizeConfig::LogicalSize(Vector2::new(640, 480)),
    };

    let mut context = Context::from_canvas_element_id("canvas", config).unwrap();
    log::info!("Initialized malen context");

    let mut game = Game::new(&context).unwrap();

    let mut frame_timer = FrameTimer::new(context.gl(), 60);

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
                    log::info!("Frame timer: {:?}", frame_timer.timing_info(),);
                }
                _ => (),
            }
        }

        game.state
            .update(timestamp_secs, context.screen(), context.input_state());
        game.render();

        frame_timer.start_draw();
        game.draw(&context).unwrap();
        frame_timer.end_draw();
    });
}

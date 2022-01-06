use std::{collections::VecDeque, time::Duration};

use coarse_prof::profile;
use instant::Instant;
use nalgebra::{Matrix3, Point2, Vector2};
use rand::Rng;
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{
    geometry::{ColorRect, ColorRotatedRect, ColorTriangleBatch, Sprite, SpriteBatch},
    gl::{DepthTest, DrawParams, DrawTimer, Texture, TextureParams, UniformBuffer},
    pass::MatricesBlock,
    plot::{Axis, LineGraph, Plot, PlotBatch, PlotStyle},
    text::{Font, Text, TextBatch},
    Camera, CanvasSizeConfig, Color4, Config, Context, FrameError, InitError, InputState, Key,
    Rect, Screen,
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
        profile!("update");

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

    font: Font,
    wall_texture: Texture,

    camera_matrices: UniformBuffer<MatricesBlock>,
    screen_matrices: UniformBuffer<MatricesBlock>,

    color_batch: ColorTriangleBatch,
    wall_sprite_batch: SpriteBatch,
    text_batch: TextBatch,
}

impl Game {
    pub fn new(context: &Context) -> Result<Game, InitError> {
        let state = State::new();

        let font = Font::load(
            context,
            include_bytes!("../resources/RobotoMono-Regular.ttf"),
            40.0,
        )?;
        let wall_texture = Texture::load(
            context.gl(),
            include_bytes!("../resources/04muroch256.png"),
            TextureParams::default(),
        )?;

        let camera_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;
        let screen_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;

        let color_batch = ColorTriangleBatch::new(context.gl())?;
        let wall_sprite_batch = SpriteBatch::new(context.gl())?;
        let text_batch = TextBatch::new(context.gl())?;

        Ok(Game {
            state,
            font,
            wall_texture,
            camera_matrices,
            screen_matrices,
            color_batch,
            wall_sprite_batch,
            text_batch,
        })
    }

    pub fn render(&mut self) -> Result<(), FrameError> {
        profile!("render");

        self.color_batch.clear();
        self.wall_sprite_batch.clear();
        self.text_batch.clear();

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

        Ok(())
    }

    pub fn draw(&mut self, context: &Context) -> Result<(), FrameError> {
        profile!("draw");

        let screen = context.screen();

        self.camera_matrices.set_data(MatricesBlock {
            view: self.state.camera().matrix(screen),
            projection: screen.orthographic_projection(),
        });
        self.screen_matrices.set_data(MatricesBlock {
            view: Matrix3::identity(),
            projection: screen.orthographic_projection(),
        });

        context.canvas().clear(Color4::new(1.0, 1.0, 1.0, 1.0));
        context.color_pass().draw(
            &self.camera_matrices,
            self.color_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );
        context.sprite_pass().draw(
            &self.camera_matrices,
            &self.wall_texture,
            self.wall_sprite_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        )?;
        self.font
            .draw(&self.screen_matrices, &mut self.text_batch)?;

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

    let plot_secs = 5;
    let mut draw_timer = DrawTimer::new(context.gl(), Duration::from_secs(plot_secs));
    let mut frame_times = VecDeque::<(Instant, Duration)>::new();
    let mut plot_batch = PlotBatch::new(context.gl()).unwrap();
    let mut show_profiling = false;

    malen::main_loop(move |timestamp_secs, _running| {
        profile!("frame");

        let start_time = Instant::now();

        while let Some(event) = context.pop_event() {
            profile!("event");

            use malen::Event::*;

            match event {
                Focused => {
                    log::info!("Canvas got focus");
                }
                Unfocused => {
                    log::info!("Canvas lost focus");
                }
                KeyPressed(Key::P) => {
                    log::info!("Profiling:\n{}", coarse_prof::to_string());
                    log::info!("Frame timer: {:?}", draw_timer.timing_info());
                    show_profiling = !show_profiling;
                }
                KeyPressed(Key::R) => {
                    coarse_prof::reset();
                }
                _ => (),
            }
        }

        game.state
            .update(timestamp_secs, context.screen(), context.input_state());
        game.render().unwrap();

        plot_batch.clear();
        if let Some((last_time, _)) = frame_times.back().filter(|_| show_profiling) {
            profile!("render_profiling");

            let prof_string = coarse_prof::to_string();
            let prof_size = game.font.text_size(17.0, &prof_string) + Vector2::new(30.0, 30.0);
            let prof_pos = Point2::from(context.canvas().logical_size())
                - prof_size
                - Vector2::new(10.0, 10.0);

            let text_end = game
                .font
                .write(
                    Text {
                        pos: prof_pos + Vector2::new(10.0, 10.0),
                        size: 17.0,
                        z: 0.0,
                        color: Color4::new(0.0, 0.0, 0.0, 1.0),
                        text: &prof_string,
                    },
                    &mut plot_batch.text_batch,
                )
                .unwrap();

            plot_batch.triangle_batch.push(ColorRect {
                rect: Rect::from_top_left(prof_pos, prof_size),
                z: 0.0,
                color: PlotStyle::default().background_color.unwrap(),
            });

            let plot = Plot {
                rect: Rect::from_top_left(
                    Point2::new(10.0, context.canvas().logical_size().y - 210.0),
                    Vector2::new(700.0, 200.0),
                ),
                x_axis: Axis {
                    label: "dt[s]".to_owned(),
                    range: Some((-(plot_secs as f32), 0.0)),
                    tics: 1.0,
                },
                y_axis: Axis {
                    label: "dur[ms]".to_owned(),
                    range: Some((0.0, 30.0)),
                    tics: 15.0,
                },
                line_graphs: vec![
                    LineGraph {
                        caption: "frame[ms]".to_owned(),
                        color: Color4::new(1.0, 0.0, 0.0, 1.0),
                        points: frame_times
                            .iter()
                            .map(|(time, dur)| {
                                (
                                    -last_time.duration_since(*time).as_secs_f32(),
                                    dur.as_secs_f32() * 1000.0,
                                )
                            })
                            .collect(),
                    },
                    LineGraph {
                        caption: "draw[ms]".to_owned(),
                        color: Color4::new(0.0, 0.0, 1.0, 1.0),
                        points: draw_timer
                            .samples()
                            .iter()
                            .map(|(time, dur)| {
                                (
                                    -last_time.duration_since(*time).as_secs_f32(),
                                    dur.as_secs_f32() * 1000.0,
                                )
                            })
                            .collect(),
                    },
                ],
            };

            plot_batch
                .push(&mut game.font, plot, PlotStyle::default())
                .unwrap();
        }

        draw_timer.start_draw();
        game.draw(&context).unwrap();
        context
            .plot_pass()
            .draw(&game.screen_matrices, &mut game.font, &mut plot_batch)
            .unwrap();
        draw_timer.end_draw();

        while frame_times.front().map_or(false, |(time, _)| {
            start_time.duration_since(*time) > Duration::from_secs(plot_secs)
        }) {
            frame_times.pop_front();
        }

        frame_times.push_back((start_time, Instant::now().duration_since(start_time)));
    });
}

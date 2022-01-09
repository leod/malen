use std::{collections::VecDeque, time::Duration};

use coarse_prof::profile;
use instant::Instant;
use nalgebra::{Matrix3, Point2, Vector2};
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{
    data::{
        ColorCircle, ColorLineBatch, ColorRect, ColorRotatedRect, ColorTriangleBatch, ColorVertex,
        InstanceBatch, Mesh, SpriteBatch, TriangleTag,
    },
    geom::{Circle, Rect},
    gl::{DepthTest, DrawParams, DrawTimer, Texture, TextureParams, UniformBuffer},
    light::{
        GlobalLightParams, Light, LightPipeline, LightPipelineParams, OccluderBatch,
        OccluderCircle, OccluderRect, OccluderRotatedRect,
    },
    pass::{ColorInstance, MatricesBlock},
    plot::{Axis, LineGraph, Plot, PlotBatch, PlotStyle},
    text::{Font, Text, TextBatch},
    CanvasSizeConfig, Color3, Color4, Config, Context, FrameError, InitError, Key,
};

mod state;

use state::State;

struct Game {
    state: State,

    font: Font,
    wall_texture: Texture,

    camera_matrices: UniformBuffer<MatricesBlock>,
    screen_matrices: UniformBuffer<MatricesBlock>,

    circle_instances: InstanceBatch<ColorVertex, ColorInstance>,
    color_batch: ColorTriangleBatch,
    shaded_color_batch: ColorTriangleBatch,
    wall_sprite_batch: SpriteBatch,
    outline_batch: ColorLineBatch,
    text_batch: TextBatch,

    light_pipeline: LightPipeline,
    occluder_batch: OccluderBatch,
    lights: Vec<Light>,
}

impl Game {
    pub async fn new(context: &Context) -> Result<Game, InitError> {
        let state = State::new();

        let font = Font::load(context, "resources/RobotoMono-Regular.ttf", 40.0).await?;
        let wall_texture = Texture::load(
            context.gl(),
            "resources/04muroch256.png",
            TextureParams::default(),
        )
        .await?;

        let camera_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;
        let screen_matrices = UniformBuffer::new(context.gl(), MatricesBlock::default())?;

        let circle_mesh = Mesh::from_geometry::<TriangleTag, _>(
            context.gl(),
            ColorCircle {
                circle: Circle {
                    center: Point2::origin(),
                    radius: 20.0,
                },
                z: 0.0,
                angle: 0.0,
                num_segments: 64,
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            },
        )?;

        let circle_instances = InstanceBatch::from_mesh(circle_mesh)?;
        let color_batch = ColorTriangleBatch::new(context.gl())?;
        let shaded_color_batch = ColorTriangleBatch::new(context.gl())?;
        let wall_sprite_batch = SpriteBatch::new(context.gl())?;
        let outline_batch = ColorLineBatch::new(context.gl())?;
        let text_batch = TextBatch::new(context.gl())?;

        let light_pipeline = LightPipeline::new(
            context,
            LightPipelineParams {
                shadow_map_resolution: 2048,
                max_num_lights: 300,
            },
        )?;
        let occluder_batch = light_pipeline.new_occluder_batch()?;
        let lights = Vec::new();

        Ok(Game {
            state,
            font,
            wall_texture,
            camera_matrices,
            screen_matrices,
            circle_instances,
            color_batch,
            shaded_color_batch,
            wall_sprite_batch,
            outline_batch,
            text_batch,
            light_pipeline,
            occluder_batch,
            lights,
        })
    }

    pub fn render(&mut self) -> Result<(), FrameError> {
        profile!("render");

        self.circle_instances.clear();
        self.color_batch.clear();
        self.shaded_color_batch.clear();
        self.wall_sprite_batch.clear();
        self.outline_batch.clear();
        self.text_batch.clear();
        self.occluder_batch.clear();
        self.lights.clear();

        self.shaded_color_batch.push(ColorRect {
            rect: Rect {
                center: Point2::origin(),
                size: 2.0 * Vector2::new(state::MAP_SIZE, state::MAP_SIZE),
            },
            z: 0.8,
            color: Color3::from_u8(183, 182, 193)
                .to_linear()
                .scale(0.5)
                .to_color4(),
        });

        for wall in &self.state.walls {
            /*self.wall_sprite_batch.push(Sprite {
                rect,
                tex_rect: Rect::from_top_left(Point2::origin(), wall.size),
                z: 0.2,
            });*/
            let color = Color3::from_u8(88, 80, 74).to_linear();
            self.color_batch.push(ColorRect {
                rect: wall.rect(),
                z: 0.2,
                color: color.to_color4(),
            });
            self.outline_batch.push(ColorRect {
                rect: wall.rect(),
                z: 0.2,
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            });
            self.occluder_batch.push(OccluderRect {
                rect: wall.rect(),
                color: Color3::from_u8(69, 157, 69),
                ignore_light_index: None,
            });
        }

        for enemy in &self.state.enemies {
            let color = Color3::from_u8(240, 101, 67).to_linear();
            /*self.circle_instances.push(ColorInstance {
                position: enemy.pos,
                angle: enemy.angle,
                color: color.to_color4(),
                z: 0.3,
                ..ColorInstance::default()
            });*/
            self.color_batch.push(ColorCircle {
                circle: enemy.circle(),
                angle: enemy.angle,
                z: 0.3,
                num_segments: 16,
                color: color.to_color4(),
            });
            self.outline_batch.push(ColorCircle {
                circle: enemy.circle(),
                angle: 0.0,
                z: 0.0,
                num_segments: 64,
                color: Color4::from_u8(255, 255, 255, 255),
            });

            self.occluder_batch.push(OccluderCircle {
                circle: enemy.circle(),
                angle: 0.0,
                num_segments: 16,
                color: color,
                ignore_light_index: Some(self.lights.len() as u32),
            });
            self.lights.push(Light {
                position: enemy.pos,
                radius: 800.0,
                angle: enemy.angle,
                angle_size: std::f32::consts::PI / 2.5,
                start: 13.0,
                //color: color.scale(0.5),
                color: Color3::from_u8(44, 110, 73).to_linear().scale(2.0),
            });
        }

        for ball in &self.state.balls {
            let color = Color3::from_u8(240, 101, 67).to_linear();
            self.color_batch.push(ColorCircle {
                circle: ball.circle(),
                angle: 0.0,
                z: 0.3,
                num_segments: 64,
                color: color.to_color4(),
            });
            self.outline_batch.push(ColorCircle {
                circle: ball.circle(),
                angle: 0.0,
                z: 0.0,
                num_segments: 64,
                color: Color4::from_u8(255, 255, 255, 255),
            });

            self.occluder_batch.push(OccluderCircle {
                circle: ball.circle(),
                angle: 0.0,
                num_segments: 16,
                color: color,
                ignore_light_index: None,
            });
        }

        let color = Color3::from_u8(255, 209, 102).to_linear();
        self.color_batch.push(ColorRotatedRect {
            rect: self.state.player.rotated_rect(),
            z: 0.4,
            color: color.to_color4(),
        });
        self.outline_batch.push(ColorRotatedRect {
            rect: self.state.player.rotated_rect(),
            z: 0.4,
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        });
        self.occluder_batch.push(OccluderRotatedRect {
            rect: self.state.player.rotated_rect(),
            color: color,
            ignore_light_index: Some(self.lights.len() as u32),
        });
        self.lights.push(Light {
            position: self.state.player.pos,
            radius: 1200.0,
            angle: self.state.player.angle,
            angle_size: std::f32::consts::PI / 8.0,
            start: 22.0,
            color: Color3::from_u8(134, 187, 189).to_linear().scale(5.0),
        });
        /*self.lights.push(Light {
            position: self.state.player.pos,
            radius: 100.0,
            angle: self.state.player.angle,
            angle_size: 2.0 * std::f32::consts::PI,
            color: Color3::new(0.8, 0.8, 2.0),
        });*/

        Ok(())
    }

    pub fn draw(&mut self, context: &mut Context) -> Result<(), FrameError> {
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

        context.clear_color_and_depth(Color4::new(1.0, 1.0, 1.0, 1.0), 1.0);

        self.light_pipeline
            .build_screen_light(
                &self.camera_matrices,
                GlobalLightParams {
                    ambient: Color3::new(0.3, 0.3, 0.3),
                },
                &self.lights,
            )?
            .draw_occluders(&mut self.occluder_batch)
            .finish_screen_light()
            //.draw_occluder_glow(&mut self.occluder_batch)
            .draw_shaded_colors(self.shaded_color_batch.draw_unit(), &DrawParams::default())
            .draw_shaded_colors(self.color_batch.draw_unit(), &DrawParams::default())
            .finish();

        /*context.color_pass().draw(
            &self.camera_matrices,
            self.color_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );*/
        context.sprite_pass().draw(
            &self.camera_matrices,
            &self.wall_texture,
            self.wall_sprite_batch.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        )?;
        context.instanced_color_pass().draw(
            &self.camera_matrices,
            self.circle_instances.draw_unit(),
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );

        context.color_pass().draw(
            &self.camera_matrices,
            self.outline_batch.draw_unit(),
            &DrawParams::default(),
        );

        self.font
            .draw(&self.screen_matrices, &mut self.text_batch)?;

        /*context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 10.0), Vector2::new(640.0, 480.0)),
            &self.light_pipeline.shadow_map(),
        )?;
        context.draw_debug_texture(
            Rect::from_top_left(Point2::new(10.0, 500.0), Vector2::new(640.0, 480.0)),
            &self.light_pipeline.screen_light(),
        )?;*/

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Starting the malen example");

    let config = Config {
        canvas_size: CanvasSizeConfig::Fill,
        //canvas_size: CanvasSizeConfig::LogicalSize(Vector2::new(640, 480)),
    };

    let mut context = Context::from_canvas_element_id("canvas", config).unwrap();
    log::info!("Initialized malen context");

    let mut game = Game::new(&context).await.unwrap();

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
            let prof_pos =
                Point2::from(context.logical_size()) - prof_size - Vector2::new(10.0, 10.0);

            game.font
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
                    Point2::new(10.0, context.logical_size().y - 210.0),
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
        game.draw(&mut context).unwrap();
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

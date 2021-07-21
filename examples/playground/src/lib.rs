use std::time::Duration;

use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use malen::nalgebra::{Point2, Point3, Vector2};

use malen::{
    draw::{
        ColPass, ColVertex, Font, Light, LineBatch, OccluderBatch, Quad, ShadowColPass, ShadowMap,
        TextBatch, TriBatch,
    },
    golem::depth::{DepthTestFunction, DepthTestMode},
    Camera, Canvas, Color3, Color4, Error, InputState, Key,
};

struct Wall {
    center: Point2<f32>,
    size: Vector2<f32>,
}

struct Thingy {
    center: Point2<f32>,
    angle: f32,
}

struct Game {
    occluder_batch: OccluderBatch,
    tri_shadowed_batch: TriBatch<ColVertex>,
    tri_plain_batch: TriBatch<ColVertex>,
    line_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,

    shadow_map: ShadowMap,
    shadow_col_pass: ShadowColPass,
    color_pass: ColPass,
    font: Font,

    walls: Vec<Wall>,
    thingies: Vec<Thingy>,
    player_pos: Point2<f32>,

    last_timestamp_secs: Option<f64>,
}

impl Game {
    pub fn new(canvas: &Canvas) -> Result<Game, Error> {
        let num_thingies = 30;
        let shadow_map = ShadowMap::new(canvas, 512, 1 + num_thingies)?;

        let font = Font::from_bytes(
            canvas,
            include_bytes!("../resources/Roboto-Regular.ttf").to_vec(),
            400.0,
        )?;

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

        let thingies = (0..num_thingies)
            .map(|_| {
                let center =
                    Point2::new(rng.gen(), rng.gen()) * 4096.0 - Vector2::new(1.0, 1.0) * 2048.0;

                Thingy {
                    center,
                    angle: rng.gen::<f32>() * std::f32::consts::PI,
                }
            })
            .collect();

        Ok(Game {
            occluder_batch: OccluderBatch::new(canvas)?,
            tri_shadowed_batch: TriBatch::new(canvas)?,
            tri_plain_batch: TriBatch::new(canvas)?,
            line_batch: LineBatch::new(canvas)?,
            text_batch: TextBatch::new(canvas)?,
            shadow_map,
            shadow_col_pass: ShadowColPass::new(canvas)?,
            color_pass: ColPass::new(canvas)?,
            font,
            walls,
            thingies,
            player_pos: Point2::origin(),
            last_timestamp_secs: None,
        })
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

        for (i, thingy) in self.thingies.iter_mut().enumerate() {
            let mut delta = 0.2 * std::f32::consts::PI * dt_secs;
            if i % 2 == 0 {
                delta *= -1.0;
            }
            thingy.angle += delta;
        }
    }

    pub fn push_quad_with_occluder(
        &mut self,
        center: Point2<f32>,
        size: Vector2<f32>,
        color: Color4,
        ignore_light_offset: Option<f32>,
    ) {
        let quad = Quad::axis_aligned(center, size);

        let z = 0.5;
        self.tri_plain_batch.push_quad(&quad, z, color);
        self.occluder_batch
            .push_occluder_quad(&quad, ignore_light_offset);
        self.line_batch
            .push_quad_outline(&quad, z, Color4::new(0.0, 0.0, 0.0, 1.0));
    }

    pub fn draw(&mut self, canvas: &mut Canvas) -> Result<(), Error> {
        self.tri_plain_batch.clear();
        self.tri_shadowed_batch.clear();
        self.line_batch.clear();
        self.occluder_batch.clear();
        self.text_batch.clear();

        // Floor
        self.tri_shadowed_batch.push_quad(
            &Quad::axis_aligned(Point2::new(0.0, 0.0), Vector2::new(4096.0, 4096.0)),
            0.0,
            Color4::new(0.4, 0.9, 0.9, 1.0),
        );

        self.font.write(
            10.0,
            Point3::new(10.0, 10.0, 0.0),
            Color4::new(1.0, 0.0, 1.0, 1.0),
            &format!("{:?}", canvas.screen()),
            &mut self.text_batch,
        );

        self.font.write(
            60.0,
            Point3::new(150.0, 150.0, 0.0),
            Color4::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        self.font.write(
            50.0,
            Point3::new(150.0, 300.0, 0.0),
            Color4::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        self.font.write(
            40.0,
            Point3::new(150.0, 450.0, 0.0),
            Color4::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        let mut lights = vec![Light {
            world_pos: self.player_pos,
            radius: 1024.0,
            angle: 0.0,
            angle_size: std::f32::consts::PI * 2.0,
            color: Color3::new(0.6, 0.6, 0.6),
        }];

        for i in 0..self.walls.len() {
            self.push_quad_with_occluder(
                self.walls[i].center,
                self.walls[i].size,
                Color4::new(0.2, 0.2, 0.8, 1.0),
                None,
            )
        }

        for i in 0..self.thingies.len() {
            self.push_quad_with_occluder(
                self.thingies[i].center,
                Vector2::new(30.0, 30.0),
                Color4::new(0.2, 0.8, 0.2, 1.0),
                Some(self.shadow_map.light_offset(i + 1)),
            );

            lights.push(Light {
                world_pos: self.thingies[i].center,
                radius: 1024.0,
                angle: self.thingies[i].angle,
                angle_size: 0.2 * std::f32::consts::PI,
                color: Color3::new(0.1, 0.25, 0.1),
            });
        }

        self.push_quad_with_occluder(
            self.player_pos,
            Vector2::new(30.0, 30.0),
            Color4::new(0.7, 0.2, 0.2, 1.0),
            Some(self.shadow_map.light_offset(0)),
        );

        let screen = canvas.screen();
        let camera = Camera {
            center: self.player_pos,
            zoom: 1.0,
            angle: 0.0,
        };
        let view = camera.to_matrix(&screen);
        let transform = screen.orthographic_projection() * view;

        self.shadow_map
            .build(canvas, &view, &lights)?
            .draw_occluders(&self.occluder_batch.draw_unit())?
            .finish()?;

        canvas.clear(Color4::new(0.0, 0.0, 0.0, 1.0));

        self.shadow_col_pass.draw(
            &transform,
            Color3::new(0.025, 0.025, 0.025),
            &self.shadow_map,
            &self.tri_shadowed_batch.draw_unit(),
        )?;

        /*self.color_pass.draw(
            &(screen.orthographic_projection() * view),
            &self.tri_shadowed_batch.draw_unit(),
        )?;*/

        canvas.golem_ctx().set_depth_test_mode(Some(DepthTestMode {
            function: DepthTestFunction::Less,
            ..Default::default()
        }));
        self.color_pass
            .draw(&transform, &self.tri_plain_batch.draw_unit())?;
        self.color_pass
            .draw(&transform, &self.line_batch.draw_unit())?;
        canvas.golem_ctx().set_depth_test_mode(None);

        self.font.draw(
            canvas,
            &screen.orthographic_projection(),
            &self.text_batch.draw_unit(),
        )?;

        /*unsafe {
            canvas.debug_tex(
                Point2::new(100.0, 100.0),
                Vector2::new(320.0, 240.0),
                self.shadow_map.light_surface().borrow_texture().unwrap(),
            )?;
            canvas.debug_tex(
                Point2::new(10.0, 100.0),
                Vector2::new(320.0, 240.0),
                self.shadow_map.shadow_map().borrow_texture().unwrap(),
            )?;
        }*/

        /*canvas.debug_tex(
            Point2::new(100.0, 100.0),
            Vector2::new(320.0, 240.0),
            self.font.texture(),
        )?;*/

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let mut canvas = Canvas::from_element_id("canvas").unwrap();
    log::info!("Initialized malen context");

    let mut game = Game::new(&canvas).unwrap();

    malen::main_loop(move |timestamp_secs, _running| {
        use malen::Event::*;

        while let Some(event) = canvas.pop_event() {
            match event {
                Focused => {
                    log::info!("got focus");
                }
                Unfocused => {
                    log::info!("lost focus");
                }
                _ => (),
            }
        }

        canvas.resize_full();

        game.update(timestamp_secs, canvas.input_state());
        game.draw(&mut canvas).unwrap();
    })
    .unwrap();
}

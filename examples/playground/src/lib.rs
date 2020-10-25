use std::time::Duration;

use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;
use webglee::{
    draw::{
        ColPass, ColVertex, Font, Light, LineBatch, OccluderBatch, Quad, ShadowMap,
        ShadowedColorPass, TextBatch, TriBatch,
    },
    golem::depth::{DepthTestFunction, DepthTestMode},
    Camera, Color, Context, Error, InputState, Point2, Point3, Vector2, VirtualKeyCode,
};

struct Wall {
    center: Point2,
    size: Vector2,
}

struct Thingy {
    center: Point2,
    angle: f32,
}

struct Game {
    occluder_batch: OccluderBatch,
    tri_shadowed_batch: TriBatch<ColVertex>,
    tri_plain_batch: TriBatch<ColVertex>,
    line_batch: LineBatch<ColVertex>,
    text_batch: TextBatch,

    shadow_map: ShadowMap,
    shadowed_color_pass: ShadowedColorPass,
    color_pass: ColPass,
    font: Font,

    walls: Vec<Wall>,
    thingies: Vec<Thingy>,
    player_pos: Point2,
}

impl Game {
    pub fn new(ctx: &Context) -> Result<Game, Error> {
        let num_thingies = 32;
        let shadow_map = ShadowMap::new(ctx, 512, 1 + num_thingies)?;

        let font = Font::from_bytes(
            ctx,
            include_bytes!("../resources/Roboto-Regular.ttf").to_vec(),
            60.0,
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
            occluder_batch: OccluderBatch::new(ctx)?,
            tri_shadowed_batch: TriBatch::new(ctx)?,
            tri_plain_batch: TriBatch::new(ctx)?,
            line_batch: LineBatch::new(ctx)?,
            text_batch: TextBatch::new(ctx)?,
            shadow_map,
            shadowed_color_pass: ShadowedColorPass::new(ctx)?,
            color_pass: ColPass::new(ctx)?,
            font,
            walls,
            thingies,
            player_pos: Point2::origin(),
        })
    }

    pub fn update(&mut self, dt: Duration, input_state: &InputState) {
        let dt_secs = dt.as_secs_f32();

        let mut player_dir = Vector2::zeros();
        if input_state.is_key_pressed(VirtualKeyCode::W) {
            player_dir.y -= 1.0;
        }
        if input_state.is_key_pressed(VirtualKeyCode::S) {
            player_dir.y += 1.0;
        }
        if input_state.is_key_pressed(VirtualKeyCode::A) {
            player_dir.x -= 1.0;
        }
        if input_state.is_key_pressed(VirtualKeyCode::D) {
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
        center: Point2,
        size: Vector2,
        color: Color,
        ignore_light_offset: Option<f32>,
    ) {
        let quad = Quad::axis_aligned(center, size);

        let z = 0.5;
        self.tri_plain_batch.push_quad(&quad, z, color);
        self.occluder_batch
            .push_occluder_quad(&quad, ignore_light_offset);
        self.line_batch
            .push_quad_outline(&quad, z, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    pub fn draw(&mut self, ctx: &mut Context) -> Result<(), Error> {
        let screen = ctx.draw().screen();

        self.tri_plain_batch.clear();
        self.tri_shadowed_batch.clear();
        self.line_batch.clear();
        self.occluder_batch.clear();
        self.text_batch.clear();

        // Floor
        self.tri_shadowed_batch.push_quad(
            &Quad::axis_aligned(Point2::new(0.0, 0.0), Vector2::new(4096.0, 4096.0)),
            0.0,
            Color::new(0.4, 0.9, 0.9, 1.0),
        );

        self.font.write(
            30.0,
            Point3::new(150.0, 150.0, 0.0),
            Color::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        self.font.write(
            20.0,
            Point3::new(150.0, 300.0, 0.0),
            Color::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        self.font.write(
            10.0,
            Point3::new(150.0, 450.0, 0.0),
            Color::new(1.0, 0.0, 1.0, 1.0),
            "Hello world! What's up?",
            &mut self.text_batch,
        );

        /*self.font.write(
            30.0,
            Point3::new(10.0, 10.0, 0.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
            &format!("Screen: {:?}", screen),
            &mut self.text_batch,
        );

        self.font.write(
            10.0,
            Point3::new(10.0, 100.0, 0.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
            &format!("This is a long blabla of text life: {:?}", screen),
            &mut self.text_batch,
        );*/

        let mut lights = vec![Light {
            world_pos: self.player_pos,
            radius: 1024.0,
            angle: 0.0,
            angle_size: std::f32::consts::PI * 2.0,
            color: Color::new(0.6, 0.6, 0.6, 1.0),
        }];

        for i in 0..self.walls.len() {
            self.push_quad_with_occluder(
                self.walls[i].center,
                self.walls[i].size,
                Color::new(0.2, 0.2, 0.8, 1.0),
                None,
            )
        }

        for i in 0..self.thingies.len() {
            self.push_quad_with_occluder(
                self.thingies[i].center,
                Vector2::new(30.0, 30.0),
                Color::new(0.2, 0.8, 0.2, 1.0),
                Some(self.shadow_map.light_offset(i + 1)),
            );

            lights.push(Light {
                world_pos: self.thingies[i].center,
                radius: 2048.0,
                angle: self.thingies[i].angle,
                angle_size: 0.2 * std::f32::consts::PI,
                color: Color::new(0.1, 0.25, 0.1, 1.0),
            });
        }

        self.push_quad_with_occluder(
            self.player_pos,
            Vector2::new(30.0, 30.0),
            Color::new(0.7, 0.2, 0.2, 1.0),
            Some(self.shadow_map.light_offset(0)),
        );

        let view = Camera {
            center: self.player_pos,
            zoom: 0.4,
            angle: 0.0,
        }
        .to_matrix(&screen);

        self.shadow_map
            .build(ctx, &(screen.orthographic_projection() * view), &lights)?
            .draw_occluders(&self.occluder_batch.draw_unit())?
            .finish()?;

        ctx.golem_ctx()
            .set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        ctx.golem_ctx().set_clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.golem_ctx().clear();

        self.shadowed_color_pass.draw(
            &(screen.orthographic_projection() * view),
            Color::new(0.025, 0.025, 0.025, 1.0),
            &self.shadow_map,
            &self.tri_shadowed_batch.draw_unit(),
        )?;

        ctx.golem_ctx().set_depth_test_mode(Some(DepthTestMode {
            function: DepthTestFunction::Less,
            ..Default::default()
        }));
        self.color_pass.draw(
            &(screen.orthographic_projection() * view),
            &self.tri_plain_batch.draw_unit(),
        )?;
        self.color_pass.draw(
            &(screen.orthographic_projection() * view),
            &self.line_batch.draw_unit(),
        )?;
        ctx.golem_ctx().set_depth_test_mode(None);

        self.font.draw(
            ctx,
            &screen.orthographic_projection(),
            &self.text_batch.draw_unit(),
        )?;

        ctx.debug_tex(Point2::new(400.0, 400.0), self.font.texture())?;

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let ctx = Context::from_canvas_id("canvas").unwrap();
    log::info!("Initialized webglee context");

    let mut game = Game::new(&ctx).unwrap();

    ctx.main_loop(move |mut ctx, dt, events, _running| {
        for event in events {
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

        game.update(dt, ctx.input_state());
        game.draw(&mut ctx).unwrap();
    })
    .unwrap();
}

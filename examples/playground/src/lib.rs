use std::time::Duration;

use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;
use webglee::{
    draw::{
        shadow::{Light, LineSegment},
        Batch, ColorPass, ColorVertex, Quad, ShadowMap, ShadowedColorPass,
    },
    golem::depth::{DepthTestFunction, DepthTestMode},
    Camera, Color, Context, Error, InputState, Matrix3, Point2, Point3, Vector2, VirtualKeyCode,
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
    shadow_map: ShadowMap,
    occluder_batch: Batch<LineSegment>,
    shadowed_color_pass: ShadowedColorPass,

    color_pass: ColorPass,
    tri_batch_shadowed: Batch<ColorVertex>,
    tri_batch_plain: Batch<ColorVertex>,
    line_batch: Batch<ColorVertex>,

    walls: Vec<Wall>,

    thingies: Vec<Thingy>,

    player_pos: Point2,
}

impl Game {
    pub fn new(ctx: &Context) -> Result<Game, Error> {
        let num_thingies = 32;
        let shadow_map = ShadowMap::new(ctx, 1024, 1 + num_thingies)?;
        let occluder_batch = Batch::new_lines(ctx)?;
        let shadowed_color_pass = ShadowedColorPass::new(ctx)?;

        let color_pass = ColorPass::new(ctx)?;
        let tri_batch_shadowed = Batch::new_triangles(ctx)?;
        let tri_batch_plain = Batch::new_triangles(ctx)?;
        let line_batch = Batch::new_lines(ctx)?;

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
            shadow_map,
            occluder_batch,
            shadowed_color_pass,
            color_pass,
            tri_batch_shadowed,
            tri_batch_plain,
            line_batch,
            walls,
            thingies,
            player_pos: Point2::origin(),
        })
    }

    pub fn render_quad_with_outline(&mut self, center: Point2, size: Vector2, color: Color) {
        let quad = Quad::axis_aligned(Point3::new(center.x, center.y, 0.5), size);

        self.tri_batch_plain.push_quad(&quad, color);
        self.line_batch
            .push_quad_outline(&quad, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    pub fn render_quad_with_occluder(&mut self, center: Point2, size: Vector2, color: Color) {
        let quad = Quad::axis_aligned(Point3::new(center.x, center.y, 0.5), size);

        self.tri_batch_plain.push_quad(&quad, color);
        self.occluder_batch.push_occluder_quad(&quad);
        self.line_batch
            .push_quad_outline(&quad, Color::new(0.0, 0.0, 0.0, 1.0));
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
            self.player_pos += dt_secs * 1000.0 * player_dir;
        }

        for (i, thingy) in self.thingies.iter_mut().enumerate() {
            let mut delta = 0.2 * std::f32::consts::PI * dt_secs;
            if i % 2 == 0 {
                delta *= -1.0;
            }
            thingy.angle += delta;
        }
    }

    pub fn draw(&mut self, ctx: &Context) -> Result<(), Error> {
        let screen = ctx.screen();

        self.tri_batch_shadowed.clear();
        self.tri_batch_plain.clear();
        self.line_batch.clear();
        self.occluder_batch.clear();

        self.tri_batch_shadowed.push_quad(
            &Quad::axis_aligned(Point3::new(0.0, 0.0, 0.0), Vector2::new(4096.0, 4096.0)),
            Color::new(0.4, 0.9, 0.9, 1.0),
        );

        let mut lights = vec![Light {
            world_pos: self.player_pos,
            radius: 1024.0,
            angle: 0.0,
            angle_size: std::f32::consts::PI * 2.0,
            color: Color::new(0.6, 0.6, 0.6, 1.0),
        }];

        for i in 0..self.walls.len() {
            self.render_quad_with_occluder(
                self.walls[i].center,
                self.walls[i].size,
                Color::new(0.2, 0.2, 0.8, 1.0),
            )
        }

        for i in 0..self.thingies.len() {
            self.render_quad_with_outline(
                self.thingies[i].center,
                Vector2::new(30.0, 30.0),
                Color::new(0.2, 0.8, 0.2, 1.0),
            );

            lights.push(Light {
                world_pos: self.thingies[i].center,
                radius: 2048.0,
                angle: self.thingies[i].angle,
                angle_size: 0.2 * std::f32::consts::PI,
                color: Color::new(0.2, 0.3, 0.2, 1.0),
            });
        }

        self.render_quad_with_outline(
            self.player_pos,
            Vector2::new(30.0, 30.0),
            Color::new(0.7, 0.2, 0.2, 1.0),
        );

        let view = Camera {
            center: self.player_pos,
            zoom: 0.4,
            angle: 0.0,
        }
        .to_matrix(&screen);

        self.shadow_map
            .build(ctx, &screen.orthographic_projection(), &view, &lights)?
            .draw_occluder_batch(&mut self.occluder_batch)?
            .finish()?;

        ctx.golem_context()
            .set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        ctx.golem_context().set_clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.golem_context().clear();

        self.shadowed_color_pass.draw_batch(
            &screen.orthographic_projection(),
            &view,
            Color::new(0.1, 0.1, 0.1, 1.0),
            &self.shadow_map,
            &mut self.tri_batch_shadowed,
        )?;

        ctx.golem_context().set_depth_test_mode(Some(DepthTestMode {
            function: DepthTestFunction::Less,
            ..Default::default()
        }));
        self.color_pass.draw_batch(
            &screen.orthographic_projection(),
            &view,
            &mut self.tri_batch_plain,
        )?;
        self.color_pass.draw_batch(
            &screen.orthographic_projection(),
            &view,
            &mut self.line_batch,
        )?;
        ctx.golem_context().set_depth_test_mode(None);

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

    ctx.main_loop(move |ctx, dt, events, _running| {
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
        game.draw(&ctx).unwrap();
    })
    .unwrap();
}

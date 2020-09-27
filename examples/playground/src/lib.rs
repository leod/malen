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
    Camera, Color, Context, Error, InputState, Matrix3, Point2, Point3, Vector2, VirtualKeyCode,
};

struct Wall {
    center: Point2,
    size: Vector2,
}

struct Game {
    shadow_map: ShadowMap,
    occluder_batch: Batch<LineSegment>,
    shadowed_color_pass: ShadowedColorPass,

    color_pass: ColorPass,
    tri_batch: Batch<ColorVertex>,
    line_batch: Batch<ColorVertex>,

    walls: Vec<Wall>,
    player_pos: Point2,
}

impl Game {
    pub fn new(ctx: &Context) -> Result<Game, Error> {
        let shadow_map = ShadowMap::new(ctx, 1024, 1)?;
        let occluder_batch = Batch::new_lines(ctx)?;
        let shadowed_color_pass = ShadowedColorPass::new(ctx)?;

        let color_pass = ColorPass::new(ctx)?;
        let tri_batch = Batch::new_triangles(ctx)?;
        let line_batch = Batch::new_lines(ctx)?;

        let mut rng = rand::thread_rng();
        let normal = Normal::new(550.0, 50.0).unwrap();
        let walls = (0..1)
            .map(|_| {
                let center =
                    Point2::new(rng.gen(), rng.gen()) * 4096.0 - Vector2::new(1.0, 1.0) * 2048.0;
                let size = Vector2::new(normal.sample(&mut rng), normal.sample(&mut rng));

                Wall { center, size }
            })
            .collect();

        Ok(Game {
            shadow_map,
            occluder_batch,
            shadowed_color_pass,
            color_pass,
            tri_batch,
            line_batch,
            walls,
            player_pos: Point2::origin(),
        })
    }

    pub fn render_quad_with_outline(&mut self, center: Point2, size: Vector2, color: Color) {
        let quad = Quad::axis_aligned(Point3::new(center.x, center.y, 0.5), size);

        self.tri_batch.push_quad(&quad, color);
        self.line_batch
            .push_quad_outline(&quad, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    pub fn render_quad_with_occluder(&mut self, center: Point2, size: Vector2, color: Color) {
        let quad = Quad::axis_aligned(Point3::new(center.x, center.y, 0.5), size);

        self.tri_batch.push_quad(&quad, color);
        self.occluder_batch.push_occluder_quad(&quad);
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
    }

    pub fn draw(&mut self, ctx: &Context) -> Result<(), Error> {
        let screen = ctx.screen();

        self.tri_batch.clear();
        self.line_batch.clear();
        self.occluder_batch.clear();

        self.tri_batch.push_quad(
            &Quad::axis_aligned(Point3::new(0.0, 0.0, 0.0), Vector2::new(4096.0, 4096.0)),
            Color::new(0.9, 0.9, 0.9, 1.0),
        );

        for i in 0..self.walls.len() {
            self.render_quad_with_occluder(
                self.walls[i].center,
                self.walls[i].size,
                Color::new(0.8, 0.8, 0.8, 1.0),
            )
        }

        self.render_quad_with_outline(
            self.player_pos,
            Vector2::new(30.0, 30.0),
            Color::new(1.0, 0.0, 0.0, 1.0),
        );

        let view = Camera {
            center: self.player_pos,
            zoom: 0.4,
            angle: 0.0,
        }
        .to_matrix(&screen);

        let lights = vec![Light {
            world_pos: self.player_pos,
        }];
        self.shadow_map
            .draw_occluder_batch(ctx, &mut self.occluder_batch, &lights)?;

        ctx.golem_context()
            .set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        ctx.golem_context().set_clear_color(1.0, 1.0, 0.0, 1.0);
        ctx.golem_context().clear();

        self.shadowed_color_pass.draw_batch(
            &screen.orthographic_projection(),
            &view,
            &lights,
            &self.shadow_map,
            &mut self.tri_batch,
        )?;
        self.color_pass.draw_batch(
            &screen.orthographic_projection(),
            &view,
            &mut self.line_batch,
        )?;

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

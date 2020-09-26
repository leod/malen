use std::time::Duration;

use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;
use webglee::{
    draw::{Batch, ColorPass, ColorVertex, Quad},
    Camera, Color, Context, Error, InputState, Matrix3, Point2, Point3, Vector2, VirtualKeyCode,
};

struct Wall {
    center: Point2,
    size: Vector2,
}

struct Game {
    color_pass: ColorPass,
    tri_batch: Batch<ColorVertex>,
    line_batch: Batch<ColorVertex>,

    walls: Vec<Wall>,
    player_pos: Point2,
}

impl Game {
    pub fn new(ctx: &Context) -> Result<Game, Error> {
        let color_pass = ColorPass::new(ctx)?;
        let tri_batch = Batch::<ColorVertex>::new_triangles(ctx)?;
        let line_batch = Batch::<ColorVertex>::new_lines(ctx)?;

        let mut rng = rand::thread_rng();
        let normal = Normal::new(150.0, 50.0).unwrap();
        let walls = (0..100)
            .map(|_| {
                let center =
                    Point2::new(rng.gen(), rng.gen()) * 4096.0 - Vector2::new(1.0, 1.0) * 2048.0;
                let size = Vector2::new(normal.sample(&mut rng), normal.sample(&mut rng));

                Wall { center, size }
            })
            .collect();

        Ok(Game {
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
            self.player_pos += dt_secs * 300.0 * player_dir;
        }
    }

    pub fn draw(&mut self, ctx: &Context) {
        let screen = ctx.screen();
        let golem_ctx = ctx.golem_context();

        golem_ctx.set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        golem_ctx.set_clear_color(1.0, 1.0, 0.0, 1.0);
        golem_ctx.clear();

        self.tri_batch.clear();
        self.line_batch.clear();

        for i in 0..self.walls.len() {
            self.render_quad_with_outline(
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

        self.color_pass
            .draw_batch(
                &screen.orthographic_projection(),
                &view,
                &mut self.tri_batch,
            )
            .unwrap();
        self.color_pass
            .draw_batch(
                &screen.orthographic_projection(),
                &view,
                &mut self.line_batch,
            )
            .unwrap();
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
        game.draw(&ctx);
    })
    .unwrap();
}

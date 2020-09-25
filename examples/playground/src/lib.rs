use rand::Rng;
use rand_distr::{Distribution, Normal};
use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;
use webglee::{
    draw::{Batch, ColorPass, ColorVertex, Quad},
    Color, Context, Error, Matrix3, Point2, Point3, Vector2,
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
}

impl Game {
    pub fn new(ctx: &Context) -> Result<Game, Error> {
        let color_pass = ColorPass::new(ctx)?;
        let tri_batch = Batch::<ColorVertex>::new_triangles(ctx)?;
        let line_batch = Batch::<ColorVertex>::new_lines(ctx)?;

        let mut rng = rand::thread_rng();
        let normal = Normal::new(150.0, 50.0).unwrap();
        let walls = (0..2)
            .map(|_| {
                let center = Point2::new(rng.gen(), rng.gen()) * 320.0;
                let size = Vector2::new(normal.sample(&mut rng), normal.sample(&mut rng));

                Wall { center, size }
            })
            .collect();

        Ok(Game {
            color_pass,
            tri_batch,
            line_batch,
            walls,
        })
    }

    pub fn render_quad_with_outline(&mut self, center: Point2, size: Vector2, color: Color) {
        let quad = Quad::axis_aligned(Point3::new(center.x, center.y, 0.5), size);

        self.tri_batch.push_quad(&quad, color);
        self.line_batch
            .push_quad_outline(&quad, Color::new(0.0, 0.0, 0.0, 1.0));
    }

    pub fn draw(&mut self, ctx: &Context) {
        let screen = ctx.screen();
        let golem_ctx = ctx.golem_context();

        golem_ctx.set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        golem_ctx.set_clear_color(1.0, 1.0, 0.0, 1.0);
        golem_ctx.clear();

        self.tri_batch.clear();
        self.line_batch.clear();

        self.render_quad_with_outline(
            Point2::new(320.0, 240.0),
            Vector2::new(100.0, 100.0),
            Color::new(1.0, 0.0, 0.0, 1.0),
        );

        for i in 0..self.walls.len() {
            self.render_quad_with_outline(
                self.walls[i].center,
                self.walls[i].size,
                Color::new(0.8, 0.8, 0.8, 1.0),
            )
        }

        self.color_pass
            .draw_batch(
                &screen.orthographic_projection(),
                &Matrix3::identity(),
                &mut self.tri_batch,
            )
            .unwrap();
        self.color_pass
            .draw_batch(
                &screen.orthographic_projection(),
                &Matrix3::identity(),
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

    ctx.main_loop(move |ctx, _dt, events, _running| {
        for event in events {
            match event {
                Focused => {
                    log::info!("got focus");
                }
                Unfocused => {
                    log::info!("lost focus");
                }
                KeyPressed(key) => {
                    log::info!("key pressed: {:?}", key);
                }
                WindowResized(size) => {
                    log::info!("window resized to: {:?}", size);
                }
                _ => (),
            }
        }

        game.draw(&ctx);
    })
    .unwrap();
}

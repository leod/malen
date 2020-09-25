use wasm_bindgen::prelude::wasm_bindgen;
use webglee::Event::*;
use webglee::{
    draw::{ColorSprite, SpriteBatch, SpriteList, SpritePass},
    na, Camera, Color, Matrix3, Point3, Vector2, Vector3,
};

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let ctx = webglee::Context::from_canvas_id("canvas").unwrap();
    log::info!("Initialized webglee context");

    let mut sprite_pass = SpritePass::new(ctx.golem_context()).unwrap();
    let sprite_list = vec![ColorSprite::axis_aligned(
        Point3::new(320.0, 240.0, 0.5),
        Vector2::new(100.0, 100.0),
        Color::new(1.0, 0.0, 0.0, 1.0),
    )]
    .into();
    let sprite_batch = SpriteBatch::from_list(ctx.golem_context(), &sprite_list).unwrap();

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

        let screen = ctx.screen();
        let golem_ctx = ctx.golem_context();

        golem_ctx.set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        golem_ctx.set_clear_color(1.0, 1.0, 0.0, 1.0);
        golem_ctx.clear();

        sprite_pass
            .draw(
                &screen.orthographic_projection(),
                &Matrix3::identity(),
                &sprite_batch,
            )
            .unwrap();
    })
    .unwrap();
}

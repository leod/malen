use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("Hi, starting the example");

    let mut context = webglee::Context::from_canvas_id("canvas").unwrap();
    log::info!("Initialized webglee context");

    webglee::main_loop(move |dt, _running| {
        while let Some(event) = context.input_mut().pop_event() {
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

        let golem_ctx = context.golem_context();
        let screen = context.screen();

        golem_ctx.set_viewport(0, 0, screen.size.x as u32, screen.size.y as u32);
        golem_ctx.set_clear_color(1.0, 1.0, 0.0, 1.0);
        golem_ctx.clear();
    });
}

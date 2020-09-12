use wasm_bindgen::prelude::wasm_bindgen;

use webglee::Event::*;

#[wasm_bindgen(start)]
pub fn main() {
    web_logger::init();
    log::info!("Hi, starting the example");

    let mut context = webglee::Context::from_canvas_id("canvas").unwrap();
    log::info!("Initialized webglee context");

    webglee::main_loop(move |dt, _running| {
        for event in context.input_mut().pop_event() {
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
                _ => (),
            }
        }
    })
}

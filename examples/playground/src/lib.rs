use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn main() {
    web_logger::init();
    log::info!("Hi, starting the example");

    let context = webglee::Context::from_canvas_id("canvas").unwrap();
    log::info!("Initialized webglee context");
}

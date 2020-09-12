use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn main() {
    let context = webglee::draw::Context::from_canvas_id("canvas");
}

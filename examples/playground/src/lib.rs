use nalgebra::Vector2;
use wasm_bindgen::prelude::wasm_bindgen;

use malen::{CanvasSizeConfig, Config, Context};

mod draw;
mod game;
mod state;

#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Starting malen playground example");

    let config = Config {
        canvas_size: CanvasSizeConfig::Fill,
    };

    let context = Context::from_canvas_element_id("canvas", config).unwrap();
    log::info!("Initialized malen::Context");

    let mut game = game::Game::new(context).await.unwrap();
    log::info!("Created Game");

    malen::main_loop(move |timestamp_secs, _running| {
        game.frame(timestamp_secs).unwrap();
    });
}

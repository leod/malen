use wasm_bindgen::prelude::wasm_bindgen;

use malen::{CanvasSizeConfig, Config, Context};

use crate::game::Game;

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
        //canvas_size: CanvasSizeConfig::LogicalSize(Vector2::new(640, 480)),
    };

    let mut context = Context::from_canvas_element_id("canvas", config).unwrap();
    log::info!("Initialized malen::Context");

    let mut game = Game::new(context).await.unwrap();
    log::info!("Created Game");

    malen::main_loop(move |timestamp_secs, _running| {
        game.frame(timestamp_secs).unwrap();
    });
}

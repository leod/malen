use thiserror::Error;
use wasm_bindgen::JsValue;

use super::{Context, Sound};

#[derive(Debug, Error)]
pub enum PlayError {
    #[error("failed to create buffer source: {0:?}")]
    CreateBufferSource(JsValue),

    #[error("failed to start playing: {0:?}")]
    Start(JsValue),
}

pub fn play(al: &Context, sound: &Sound) -> Result<(), PlayError> {
    let buffer_source = al
        .context
        .create_buffer_source()
        .map_err(PlayError::CreateBufferSource)?;
    buffer_source.set_buffer(Some(sound.buffer()));
    buffer_source.start().map_err(PlayError::Start)?;

    Ok(())
}

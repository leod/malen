use thiserror::Error;
use wasm_bindgen::JsValue;
use web_sys::AudioBufferSourceNode;

use super::Sound;

#[derive(Debug, Error)]
pub enum PlayError {
    #[error("failed to create buffer source: {0:?}")]
    CreateBufferSource(JsValue),

    #[error("failed to connect: {0:?}")]
    Connect(JsValue),

    #[error("failed to start playing: {0:?}")]
    Start(JsValue),
}

type SoundSourceNode = AudioBufferSourceNode;

pub fn play(sound: &Sound) -> Result<SoundSourceNode, PlayError> {
    let al = &sound.al().context;
    let buffer_source = al
        .create_buffer_source()
        .map_err(PlayError::CreateBufferSource)?;
    buffer_source.set_buffer(Some(sound.buffer()));
    buffer_source
        .connect_with_audio_node(&al.destination())
        .map_err(PlayError::Connect)?;
    buffer_source.start().map_err(PlayError::Start)?;

    Ok(buffer_source)
}

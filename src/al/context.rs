use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::AudioContext;

#[derive(Error, Debug)]
pub enum NewContextError {
    #[error("failed to create AudioContext: {0:?}")]
    NewAudioContext(JsValue),
}

pub struct Context {
    pub(super) context: AudioContext,
}

impl Context {
    pub fn new() -> Result<Self, NewContextError> {
        let audio = AudioContext::new().map_err(NewContextError::NewAudioContext)?;

        Ok(Context { context: audio })
    }
}

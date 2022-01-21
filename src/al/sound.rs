use std::rc::Rc;

use thiserror::Error;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::AudioBuffer;

use crate::FetchError;

use super::Context;

#[derive(Debug, Error)]
pub enum LoadSoundError {
    #[error("fetch error: {0}")]
    Fetch(#[from] FetchError),

    #[error("failed to create buffer source: {0:?}")]
    CreateBufferSource(JsValue),

    #[error("failed to decode audio data: {0:?}")]
    DecodeAudioData(JsValue),

    #[error("failed to await audio data: {0:?}")]
    AwaitDecodeAudioData(JsValue),
}

pub struct Sound {
    al: Rc<Context>,
    buffer: AudioBuffer,
}

impl Sound {
    pub async fn load(al: Rc<Context>, path: &str) -> Result<Sound, LoadSoundError> {
        let buffer: AudioBuffer = {
            let array_buffer = crate::fetch_array_buffer(path).await?;
            let promise = al
                .context
                .decode_audio_data(&array_buffer)
                .map_err(LoadSoundError::DecodeAudioData)?;
            let value = JsFuture::from(promise)
                .await
                .map_err(LoadSoundError::AwaitDecodeAudioData)?;
            assert!(value.is_instance_of::<AudioBuffer>());
            value.dyn_into().unwrap()
        };

        Ok(Sound { al, buffer })
    }

    pub fn al(&self) -> Rc<Context> {
        self.al.clone()
    }

    pub fn buffer(&self) -> &AudioBuffer {
        &self.buffer
    }
}

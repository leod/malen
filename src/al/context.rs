use nalgebra::{Point3, Vector3};
use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::AudioContext;

#[derive(Error, Debug)]
pub enum NewContextError {
    #[error("failed to create AudioContext: {0:?}")]
    NewAudioContext(JsValue),
}

pub struct Context {
    context: AudioContext,
}

impl Context {
    pub fn new() -> Result<Self, NewContextError> {
        let audio = AudioContext::new().map_err(NewContextError::NewAudioContext)?;

        Ok(Context { context: audio })
    }

    pub fn context(&self) -> &AudioContext {
        &self.context
    }

    pub fn set_listener_pos(&self, p: Point3<f32>) {
        self.context
            .listener()
            .set_position(p.x as f64, p.y as f64, p.z as f64);
    }

    pub fn set_listener_vel(&self, v: Point3<f32>) {
        self.context
            .listener()
            .set_velocity(v.x as f64, v.y as f64, v.z as f64);
    }
}

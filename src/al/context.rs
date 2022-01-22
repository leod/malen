use nalgebra::Point3;
use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::{AudioContext, AudioNode, DynamicsCompressorNode};

#[derive(Error, Debug)]
pub enum NewContextError {
    #[error("failed to create audio context: {0:?}")]
    NewAudioContext(JsValue),

    #[error("failed to create dynamics compressor: {0:?}")]
    CreateDynamicsCompressor(JsValue),

    #[error("failed to connect node: {0:?}")]
    Connect(JsValue),
}

pub struct Context {
    context: AudioContext,
    compressor_node: DynamicsCompressorNode,
}

impl Context {
    pub fn new() -> Result<Self, NewContextError> {
        let context = AudioContext::new().map_err(NewContextError::NewAudioContext)?;

        let compressor_node = context
            .create_dynamics_compressor()
            .map_err(NewContextError::CreateDynamicsCompressor)?;

        compressor_node
            .connect_with_audio_node(&context.destination())
            .map_err(NewContextError::Connect)?;

        Ok(Context {
            context,
            compressor_node,
        })
    }

    pub fn context(&self) -> &AudioContext {
        &self.context
    }

    pub fn destination(&self) -> &AudioNode {
        &self.compressor_node
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

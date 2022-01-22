use nalgebra::Point3;
use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::{AudioBufferSourceNode, DistanceModelType, PanningModelType};

use super::Sound;

#[derive(Debug, Error)]
pub enum PlayError {
    #[error("failed to create buffer source: {0:?}")]
    CreateBufferSource(JsValue),

    #[error("failed to create panner: {0:?}")]
    CreatePanner(JsValue),

    #[error("failed to connect: {0:?}")]
    Connect(JsValue),

    #[error("failed to start playing: {0:?}")]
    Start(JsValue),
}

pub type SoundSourceNode = AudioBufferSourceNode;

pub fn sound_source_node(sound: &Sound) -> Result<SoundSourceNode, PlayError> {
    let al = sound.al();
    let source_node = al
        .context()
        .create_buffer_source()
        .map_err(PlayError::CreateBufferSource)?;
    source_node.set_buffer(Some(sound.buffer()));

    Ok(source_node)
}

pub fn play(sound: &Sound) -> Result<SoundSourceNode, PlayError> {
    let al = sound.al();
    let source_node = sound_source_node(sound)?;
    source_node
        .connect_with_audio_node(&al.context().destination())
        .map_err(PlayError::Connect)?;
    source_node.start().map_err(PlayError::Start)?;

    Ok(source_node)
}

pub fn play_spatial(sound: &Sound, pos: Point3<f32>) -> Result<SoundSourceNode, PlayError> {
    let al = sound.al();
    let source_node = sound_source_node(sound)?;
    source_node
        .connect_with_audio_node(&al.context().destination())
        .map_err(PlayError::Connect)?;
    source_node.start().map_err(PlayError::Start)?;

    let panner_node = al
        .context()
        .create_panner()
        .map_err(PlayError::CreatePanner)?;
    panner_node.set_cone_inner_angle(60.0);
    panner_node.set_cone_outer_angle(90.0);
    panner_node.set_cone_outer_gain(0.3);
    panner_node.set_panning_model(PanningModelType::Hrtf);
    panner_node.set_distance_model(DistanceModelType::Linear);
    panner_node.set_max_distance(1000.0);
    panner_node.set_ref_distance(1.0);
    panner_node.set_rolloff_factor(10.0);
    panner_node.set_position(pos.x as f64, pos.y as f64, pos.z as f64);

    Ok(source_node)
}

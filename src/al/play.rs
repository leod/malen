use nalgebra::{Point3, Vector3};
use thiserror::Error;

use wasm_bindgen::JsValue;
use web_sys::{AudioBufferSourceNode, DistanceModelType, PannerNode, PanningModelType};

use super::Sound;

#[derive(Debug, Error)]
pub enum PlayError {
    #[error("failed to create buffer source: {0:?}")]
    CreateBufferSource(JsValue),

    #[error("failed to create panner: {0:?}")]
    CreatePanner(JsValue),

    #[error("failed to create gain: {0:?}")]
    CreateGain(JsValue),

    #[error("failed to connect: {0:?}")]
    Connect(JsValue),

    #[error("failed to start playing: {0:?}")]
    Start(JsValue),
}

#[derive(Debug, Clone)]
pub struct SpatialPlayParams {
    pub cone_inner_angle: f32,
    pub cone_outer_angle: f32,
    pub cone_outer_gain: f32,
    pub panning_model: PanningModelType,
    pub distance_model: DistanceModelType,
    pub max_distance: f32,
    pub ref_distance: f32,
    pub rolloff_factor: f32,
    pub orientation: Vector3<f32>,
    pub pos: Point3<f32>,
    pub gain: f32,
}

impl Default for SpatialPlayParams {
    fn default() -> Self {
        Self {
            cone_inner_angle: 360.0,
            cone_outer_angle: 360.0,
            cone_outer_gain: 0.3,
            panning_model: PanningModelType::Hrtf,
            distance_model: DistanceModelType::Linear,
            max_distance: 1000.0,
            ref_distance: 1.0,
            rolloff_factor: 10.0,
            orientation: Vector3::new(1.0, 0.0, 0.0),
            pos: Point3::origin(),
            gain: 1.0,
        }
    }
}

pub type PlayNode = AudioBufferSourceNode;

pub struct SpatialPlayNode {
    pub source: AudioBufferSourceNode,
    pub panner: PannerNode,
}

pub fn play_node(sound: &Sound) -> Result<PlayNode, PlayError> {
    let al = sound.al();
    let source = al
        .context()
        .create_buffer_source()
        .map_err(PlayError::CreateBufferSource)?;
    source.set_buffer(Some(sound.buffer()));

    Ok(source)
}

pub fn play(sound: &Sound) -> Result<PlayNode, PlayError> {
    let al = sound.al();
    let source = play_node(sound)?;

    source
        .connect_with_audio_node(al.destination())
        .map_err(PlayError::Connect)?;
    source.start().map_err(PlayError::Start)?;

    Ok(source)
}

pub fn play_spatial(
    sound: &Sound,
    params: &SpatialPlayParams,
) -> Result<SpatialPlayNode, PlayError> {
    let al = sound.al();
    let source = play_node(sound)?;

    let panner = al
        .context()
        .create_panner()
        .map_err(PlayError::CreatePanner)?;
    panner.set_cone_inner_angle(params.cone_inner_angle as f64);
    panner.set_cone_outer_angle(params.cone_outer_angle as f64);
    panner.set_cone_outer_gain(params.cone_outer_gain as f64);
    panner.set_panning_model(params.panning_model);
    panner.set_distance_model(params.distance_model);
    panner.set_max_distance(params.max_distance as f64);
    panner.set_ref_distance(params.ref_distance as f64);
    panner.set_rolloff_factor(params.rolloff_factor as f64);
    panner.set_orientation(
        params.orientation.x as f64,
        params.orientation.y as f64,
        params.orientation.z as f64,
    );
    panner.set_position(
        params.pos.x as f64,
        params.pos.y as f64,
        params.pos.z as f64,
    );

    let gain = al.context().create_gain().map_err(PlayError::CreateGain)?;
    gain.gain().set_value(params.gain);

    source
        .connect_with_audio_node(&panner)
        .map_err(PlayError::Connect)?;
    panner
        .connect_with_audio_node(&gain)
        .map_err(PlayError::Connect)?;
    gain.connect_with_audio_node(al.destination())
        .map_err(PlayError::Connect)?;
    source.start().map_err(PlayError::Start)?;

    Ok(SpatialPlayNode { source, panner })
}

impl SpatialPlayNode {
    pub fn set_pos(&self, p: Point3<f32>) {
        self.panner.set_position(p.x as f64, p.y as f64, p.z as f64);
    }

    pub fn set_loop(&self, value: bool) {
        self.source.set_loop(value);
    }
}

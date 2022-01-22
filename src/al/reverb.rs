use web_sys::{AudioNode, ConvolverNode};

use super::{PlayError, Sound};

#[derive(Debug, Clone)]
pub struct ReverbParams {
    pub pre_delay_secs: f32,
    pub reverb_time_secs: f32,
    pub num_taps: usize,
    pub convolver_gain: f32,
    pub taps_gain: f32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            pre_delay_secs: 0.1,
            reverb_time_secs: 2.0,
            num_taps: 2,
            convolver_gain: 0.2,
            taps_gain: 0.7,
        }
    }
}

pub fn convolver(impulse: &Sound) -> Result<ConvolverNode, PlayError> {
    let al = impulse.al();
    let convolver = al
        .context()
        .create_convolver()
        .map_err(PlayError::WebAudio)?;
    convolver.set_buffer(Some(impulse.buffer()));

    Ok(convolver)
}

pub fn reverb(
    impulse: &Sound,
    dest: &AudioNode,
    params: &ReverbParams,
) -> Result<AudioNode, PlayError> {
    let al = impulse.al();

    let input = al.context().create_gain().map_err(PlayError::WebAudio)?;
    let pre_delay = al.context().create_delay().map_err(PlayError::WebAudio)?;
    let taps = (0..params.num_taps)
        .map(|_| Ok((al.context().create_delay()?, al.context().create_gain()?)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(PlayError::WebAudio)?;
    let taps_gain = al.context().create_gain().map_err(PlayError::WebAudio)?;
    let convolver = convolver(impulse)?;
    let convolver_gain = al.context().create_gain().map_err(PlayError::WebAudio)?;
    let output = al.context().create_gain().map_err(PlayError::WebAudio)?;

    pre_delay.delay_time().set_value(params.pre_delay_secs);
    for (i, (tap_delay, tap_gain)) in taps.iter().enumerate() {
        tap_delay
            .delay_time()
            .set_value(0.001 + i as f32 * 0.5 * params.pre_delay_secs);
        tap_gain.gain().set_value(params.taps_gain);
    }
    taps_gain.gain().set_value(params.taps_gain);
    convolver_gain.gain().set_value(params.convolver_gain);

    input
        .connect_with_audio_node(&pre_delay)
        .map_err(PlayError::WebAudio)?;
    input
        .connect_with_audio_node(&output)
        .map_err(PlayError::WebAudio)?;
    if !taps.is_empty() {
        input
            .connect_with_audio_node(&taps[0].0)
            .map_err(PlayError::WebAudio)?;
    }
    for ((tap1_delay, tap1_gain), (tap2_delay, _)) in taps.iter().zip(taps.iter().skip(1)) {
        tap1_delay
            .connect_with_audio_node(tap1_gain)
            .map_err(PlayError::WebAudio)?;
        tap1_gain
            .connect_with_audio_node(tap2_delay)
            .map_err(PlayError::WebAudio)?;
        tap1_gain
            .connect_with_audio_node(&taps_gain)
            .map_err(PlayError::WebAudio)?;
    }
    taps_gain
        .connect_with_audio_node(&output)
        .map_err(PlayError::WebAudio)?;
    pre_delay
        .connect_with_audio_node(&convolver)
        .map_err(PlayError::WebAudio)?;
    convolver
        .connect_with_audio_node(&convolver_gain)
        .map_err(PlayError::WebAudio)?;
    convolver_gain
        .connect_with_audio_node(&output)
        .map_err(PlayError::WebAudio)?;
    output
        .connect_with_audio_node(dest)
        .map_err(PlayError::WebAudio)?;

    Ok(input.into())
}

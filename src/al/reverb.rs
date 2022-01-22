use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::{AudioNode, AudioParam, ConvolverNode, DelayNode, GainNode};

use super::{Context, PlayError, Sound};

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
            pre_delay_secs: 0.01,
            reverb_time_secs: 2.0,
            num_taps: 3,
            convolver_gain: 0.2,
            taps_gain: 0.4,
        }
    }
}

pub struct ReverbNode {
    al: Rc<Context>,
    input: GainNode,
    pre_delay: DelayNode,
    taps: Vec<(DelayNode, GainNode)>,
    taps_gain: GainNode,
    convolver_gain: GainNode,
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
) -> Result<ReverbNode, PlayError> {
    // Inspired by <https://blog.gskinner.com/archives/2019/02/reverb-web-audio-api.html>.

    let al = impulse.al();

    let convolver = convolver(impulse)?;

    let reverb = (|| {
        let input = al.context().create_gain()?;
        let pre_delay = al.context().create_delay()?;
        let taps = (0..params.num_taps)
            .map(|_| Ok((al.context().create_delay()?, al.context().create_gain()?)))
            .collect::<Result<Vec<_>, JsValue>>()?;
        let taps_gain = al.context().create_gain()?;
        let convolver_gain = al.context().create_gain()?;
        let output = al.context().create_gain()?;

        input.connect_with_audio_node(&dest)?;
        input.connect_with_audio_node(&pre_delay)?;
        if !taps.is_empty() {
            input.connect_with_audio_node(&taps[0].0)?;
        }
        for ((tap1_delay, tap1_gain), (tap2_delay, _)) in taps.iter().zip(taps.iter().skip(1)) {
            tap1_delay.connect_with_audio_node(tap1_gain)?;
            tap1_gain.connect_with_audio_node(tap2_delay)?;
            tap1_gain.connect_with_audio_node(&taps_gain)?;
        }
        taps_gain.connect_with_audio_node(&output)?;
        pre_delay.connect_with_audio_node(&convolver)?;
        convolver.connect_with_audio_node(&convolver_gain)?;
        convolver_gain.connect_with_audio_node(&output)?;
        output.connect_with_audio_node(dest)?;

        Ok(ReverbNode {
            al,
            input,
            pre_delay,
            taps,
            taps_gain,
            convolver_gain,
        })
    })()
    .map_err(PlayError::WebAudio)?;

    reverb.set_params(params)?;
    Ok(reverb)
}

impl ReverbNode {
    pub fn input(&self) -> &AudioNode {
        &self.input
    }

    pub fn set_params(&self, params: &ReverbParams) -> Result<(), PlayError> {
        assert!(self.taps.len() == params.num_taps);

        let set = |audio_param: AudioParam, value: f32| {
            audio_param.set_value(value);
            Ok(())
        };
        self.set_params_generic(params, set)
            .map_err(PlayError::WebAudio)
    }

    pub fn linear_ramp_to_params(&self, params: &ReverbParams, secs: f32) -> Result<(), PlayError> {
        assert!(self.taps.len() == params.num_taps);

        let end_time = self.al.context().current_time() + secs as f64;

        let set = |audio_param: AudioParam, value: f32| {
            audio_param
                .linear_ramp_to_value_at_time(value, end_time)
                .map(|_| ())
        };
        self.set_params_generic(params, set)
            .map_err(PlayError::WebAudio)
    }

    fn set_params_generic(
        &self,
        params: &ReverbParams,
        set: impl Fn(AudioParam, f32) -> Result<(), JsValue>,
    ) -> Result<(), JsValue> {
        assert!(self.taps.len() == params.num_taps);

        set(self.pre_delay.delay_time(), params.pre_delay_secs)?;
        for (i, (tap_delay, tap_gain)) in self.taps.iter().enumerate() {
            set(
                tap_delay.delay_time(),
                0.001 + i as f32 * 0.5 * params.pre_delay_secs,
            )?;
            set(tap_gain.gain(), params.taps_gain)?;
        }
        set(self.taps_gain.gain(), params.taps_gain)?;
        set(self.convolver_gain.gain(), params.convolver_gain)?;

        Ok(())
    }
}

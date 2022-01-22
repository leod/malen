use web_sys::{AudioNode, ConvolverNode};

use super::{PlayError, Sound};

pub fn convolver(impulse: &Sound) -> Result<ConvolverNode, PlayError> {
    let al = impulse.al();
    let convolver = al
        .context()
        .create_convolver()
        .map_err(PlayError::WebAudio)?;
    convolver.set_buffer(Some(impulse.buffer()));

    Ok(convolver)
}

pub fn reverb(impulse: &Sound, dest: &AudioNode) -> Result<AudioNode, PlayError> {
    let al = impulse.al();
    let input = al.context().create_gain().map_err(PlayError::WebAudio)?;
    let convolver = convolver(impulse)?;
    let mix = al.context().create_gain().map_err(PlayError::WebAudio)?;

    input
        .connect_with_audio_node(&convolver)
        .map_err(PlayError::WebAudio)?;
    convolver
        .connect_with_audio_node(&mix)
        .map_err(PlayError::WebAudio)?;
    input
        .connect_with_audio_node(&mix)
        .map_err(PlayError::WebAudio)?;
    mix.connect_with_audio_node(dest)
        .map_err(PlayError::WebAudio)?;

    Ok(input.into())
}

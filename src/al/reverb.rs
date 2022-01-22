use web_sys::AudioNode;

use super::{PlayError, Sound};

fn convolver(impulse: &Sound) -> Result<AudioNode, PlayError> {
    let al = impulse.al();
    let node = al
        .context()
        .create_convolver()
        .map_err(PlayError::WebAudio)?;

    Ok(node.into())
}

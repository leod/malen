use thiserror::Error;

use crate::{
    gl::{
        Framebuffer, NewFramebufferError, NewTextureError, Texture, TextureMagFilter,
        TextureMinFilter, TextureParams, TextureValueType, TextureWrap,
    },
    Context,
};

#[derive(Debug, Clone)]
pub struct LightPipelineParams {
    pub max_num_lights: usize,
}

pub struct LightPipeline {
    params: LightPipelineParams,
    shadow_map: Framebuffer,
}

#[derive(Debug, Error)]
pub enum NewLightPipelineError {
    #[error("texture error: {0}")]
    NewTexture(#[from] NewTextureError),

    #[error("framebuffer error: {0}")]
    NewFramebuffer(#[from] NewFramebufferError),
}

impl LightPipeline {
    pub fn new(
        context: &Context,
        params: LightPipelineParams,
    ) -> Result<LightPipeline, NewLightPipelineError> {
        let initial_size = context.canvas().physical_size();
        let shadow_map = Framebuffer::new(
            context.gl(),
            vec![Texture::new(
                context.gl(),
                initial_size,
                TextureParams {
                    value_type: TextureValueType::Depth,
                    min_filter: TextureMinFilter::Nearest,
                    mag_filter: TextureMagFilter::Nearest,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?],
        )?;

        Ok(Self { params, shadow_map })
    }
}

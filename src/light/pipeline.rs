use std::{cell::RefCell, rc::Rc};

use nalgebra::Vector2;
use thiserror::Error;

use crate::{
    gl::{
        Framebuffer, NewFramebufferError, NewTextureError, Texture, TextureMagFilter,
        TextureMinFilter, TextureParams, TextureValueType, TextureWrap,
    },
    Canvas, Context,
};

#[derive(Debug, Clone)]
pub struct LightPipelineParams {
    pub shadow_map_resolution: u32,
    pub max_num_lights: u32,
}

pub struct LightPipeline {
    canvas: Rc<RefCell<Canvas>>,
    params: LightPipelineParams,
    shadow_map: Framebuffer,
    screen_light: Framebuffer,
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
        let canvas = context.canvas();

        let shadow_map = Framebuffer::new(
            context.gl(),
            vec![Texture::new(
                context.gl(),
                Vector2::new(params.shadow_map_resolution, params.max_num_lights),
                TextureParams {
                    value_type: TextureValueType::Depth,
                    min_filter: TextureMinFilter::Nearest,
                    mag_filter: TextureMagFilter::Nearest,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?],
        )?;

        let screen_light = Framebuffer::new(
            context.gl(),
            vec![Texture::new(
                context.gl(),
                canvas.borrow().physical_size(),
                TextureParams {
                    value_type: TextureValueType::RgbaF32,
                    min_filter: TextureMinFilter::Nearest,
                    mag_filter: TextureMagFilter::Nearest,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?],
        )?;

        Ok(Self {
            canvas,
            params,
            shadow_map,
            screen_light,
        })
    }
}

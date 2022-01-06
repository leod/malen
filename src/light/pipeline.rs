use std::{cell::RefCell, rc::Rc};

use nalgebra::Vector2;
use thiserror::Error;

use crate::{
    gl::{
        self, Framebuffer, NewFramebufferError, NewTextureError, Texture, TextureMagFilter,
        TextureMinFilter, TextureParams, TextureValueType, TextureWrap, UniformBuffer,
        VertexBuffer,
    },
    pass::MatricesBlock,
    Canvas, Context, FrameError,
};

use super::{data::LightInstance, Light, OccluderBatch};

#[derive(Debug, Clone)]
pub struct LightPipelineParams {
    pub shadow_map_resolution: u32,
    pub max_num_lights: u32,
}

pub struct LightPipeline {
    canvas: Rc<RefCell<Canvas>>,
    params: LightPipelineParams,

    light_instances: Rc<VertexBuffer<LightInstance>>,
    shadow_map: Framebuffer,
    screen_light: Framebuffer,
}

#[derive(Debug, Error)]
pub enum NewLightPipelineError {
    #[error("OpenGL error: {0}")]
    OpenGL(#[from] gl::Error),

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

        let light_instances = Rc::new(VertexBuffer::new(context.gl())?);

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

        let screen_light = new_screen_light(canvas.clone())?;

        Ok(Self {
            canvas,
            params,
            light_instances,
            shadow_map,
            screen_light,
        })
    }

    pub fn new_occluder_batch(&self) -> Result<OccluderBatch, gl::Error> {
        OccluderBatch::new(self.light_instances.clone())
    }

    pub fn start<'a>(
        &'a mut self,
        matrices: &'a UniformBuffer<MatricesBlock>,
        lights: &'a [Light],
    ) -> Result<BuildShadowMapPipelineStep, FrameError> {
        if self.screen_light.textures()[0].size() != screen_light_size(self.canvas.clone()) {
            self.screen_light = new_screen_light(self.canvas.clone())?;
        }

        Ok(BuildShadowMapPipelineStep {
            pipeline: self,
            lights,
        })
    }
}

pub struct BuildShadowMapPipelineStep<'a> {
    pipeline: &'a LightPipeline,
    lights: &'a [Light],
}

fn screen_light_size(canvas: Rc<RefCell<Canvas>>) -> Vector2<u32> {
    let canvas = canvas.borrow();
    let physical_size = canvas.physical_size();
    let max_size = Texture::max_size(&*canvas.gl());

    Vector2::new(physical_size.x.min(max_size), physical_size.y.min(max_size))
}

fn new_screen_light(canvas: Rc<RefCell<Canvas>>) -> Result<Framebuffer, NewFramebufferError> {
    Framebuffer::new(
        canvas.borrow().gl(),
        vec![Texture::new(
            canvas.borrow().gl(),
            screen_light_size(canvas.clone()),
            TextureParams {
                value_type: TextureValueType::RgbaF32,
                min_filter: TextureMinFilter::Nearest,
                mag_filter: TextureMagFilter::Nearest,
                wrap_vertical: TextureWrap::ClampToEdge,
                wrap_horizontal: TextureWrap::ClampToEdge,
            },
        )?],
    )
}

//! This implementation follows the following with some modifications:
//! https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php

use std::{cell::RefCell, rc::Rc};

use nalgebra::{Vector2, Vector3};
use thiserror::Error;

use crate::{
    data::{ColorVertex, TriangleBatch},
    gl::{
        self, DrawParams, DrawUnit, Framebuffer, NewFramebufferError, NewTextureError, Texture,
        TextureMagFilter, TextureMinFilter, TextureParams, TextureValueType, TextureWrap,
        UniformBuffer, VertexBuffer,
    },
    pass::MatricesBlock,
    Canvas, Color4, Context, FrameError,
};

use super::{
    data::{LightAreaVertex, LightCircleSegment, LightInstance, LightRect},
    screen_light_pass::ScreenLightPass,
    shadow_map_pass::ShadowMapPass,
    ColorPass, GlobalLightParams, GlobalLightParamsBlock, Light, OccluderBatch,
};

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

    shadow_map_pass: ShadowMapPass,
    screen_light_pass: ScreenLightPass,
    light_area_batch: TriangleBatch<LightAreaVertex>,

    color_pass: ColorPass,
    global_light_params: UniformBuffer<GlobalLightParamsBlock>,
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
                    value_type: TextureValueType::RgbaF32,
                    min_filter: TextureMinFilter::Nearest,
                    mag_filter: TextureMagFilter::Nearest,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?],
        )?;
        let screen_light = new_screen_light(canvas.clone())?;

        let shadow_map_pass = ShadowMapPass::new(context.gl(), params.max_num_lights)?;
        let screen_light_pass = ScreenLightPass::new(
            context.gl(),
            params.shadow_map_resolution,
            params.max_num_lights,
        )?;
        let light_area_batch = TriangleBatch::new(context.gl())?;

        let color_pass = ColorPass::new(context.gl())?;
        let global_light_params = UniformBuffer::new(
            context.gl(),
            GlobalLightParamsBlock {
                ambient: Vector3::zeros(),
            },
        )?;

        Ok(Self {
            canvas,
            params,
            light_instances,
            shadow_map,
            screen_light,
            shadow_map_pass,
            screen_light_pass,
            light_area_batch,
            color_pass,
            global_light_params,
        })
    }

    pub fn shadow_map(&self) -> &Texture {
        &self.shadow_map.textures()[0]
    }

    pub fn screen_light(&self) -> &Texture {
        &self.screen_light.textures()[0]
    }

    pub fn new_occluder_batch(&self) -> Result<OccluderBatch, gl::Error> {
        OccluderBatch::new(self.light_instances.clone())
    }

    pub fn build_screen_light<'a>(
        &'a mut self,
        matrices: &'a UniformBuffer<MatricesBlock>,
        global_light_params: GlobalLightParams,
        lights: &'a [Light],
    ) -> Result<BuildScreenLightPipelineStep, FrameError> {
        if self.screen_light.textures()[0].size() != screen_light_size(self.canvas.clone()) {
            self.screen_light = new_screen_light(self.canvas.clone())?;
        }

        self.light_instances.set_data(
            &lights
                .iter()
                .cloned()
                .map(LightInstance::from_light)
                .collect::<Vec<_>>(),
        );

        self.global_light_params
            .set_data(global_light_params.into());

        gl::with_framebuffer(&self.shadow_map, || {
            gl::clear_color(&*self.shadow_map.gl(), Color4::new(1.0, 1.0, 1.0, 1.0));
        });

        Ok(BuildScreenLightPipelineStep {
            pipeline: self,
            matrices,
            lights,
        })
    }
}

#[must_use]
pub struct BuildScreenLightPipelineStep<'a> {
    pipeline: &'a mut LightPipeline,
    matrices: &'a UniformBuffer<MatricesBlock>,
    lights: &'a [Light],
}

impl<'a> BuildScreenLightPipelineStep<'a> {
    pub fn draw_occluders(self, batch: &mut OccluderBatch) -> Self {
        gl::with_framebuffer(&self.pipeline.shadow_map, || {
            self.pipeline.shadow_map_pass.draw(batch.draw_unit())
        });

        self
    }

    pub fn finish_screen_light(self) -> DrawShadedPipelineStep<'a> {
        /*self.pipeline
            .light_area_batch
            .reset(
                self.lights
                    .iter()
                    .enumerate()
                    .map(|(light_index, light)| LightRect {
                        light_index: light_index as i32,
                        light: light.clone(),
                        rect: light.rect(),
                    }),
            );*/
        self.pipeline
            .light_area_batch
            .reset(
                self.lights
                    .iter()
                    .enumerate()
                    .map(|(light_index, light)| LightCircleSegment {
                        light_index: light_index as i32,
                        light: light.clone(),
                        num_segments: 16,
                    }),
            );

        gl::with_framebuffer(&self.pipeline.screen_light, || {
            gl::clear_color(
                &*self.pipeline.screen_light.gl(),
                Color4::new(0.0, 0.0, 0.0, 1.0),
            );

            self.pipeline.screen_light_pass.draw(
                self.matrices,
                &self.pipeline.shadow_map.textures()[0],
                self.pipeline.light_area_batch.draw_unit(),
            );
        });

        DrawShadedPipelineStep {
            pipeline: self.pipeline,
            matrices: self.matrices,
        }
    }
}

#[must_use]
pub struct DrawShadedPipelineStep<'a> {
    pipeline: &'a mut LightPipeline,
    matrices: &'a UniformBuffer<MatricesBlock>,
}

impl<'a> DrawShadedPipelineStep<'a> {
    pub fn draw_shaded_colors(
        self,
        draw_unit: DrawUnit<ColorVertex>,
        draw_params: &DrawParams,
    ) -> Self {
        self.pipeline.color_pass.draw(
            self.matrices,
            &self.pipeline.global_light_params,
            &self.pipeline.screen_light.textures()[0],
            draw_unit,
            draw_params,
        );

        self
    }

    pub fn finish(self) {}
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

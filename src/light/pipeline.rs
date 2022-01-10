//! This implementation follows the following with some modifications:
//! https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php

use std::{cell::RefCell, rc::Rc};

use nalgebra::{Vector2, Vector3};
use thiserror::Error;

use crate::{
    data::{ColorVertex, SpriteVertex, TriangleBatch},
    gl::{
        self, DrawUnit, Element, Framebuffer, NewFramebufferError, NewTextureError, Texture,
        TextureMagFilter, TextureMinFilter, TextureParams, TextureValueType, TextureWrap,
        UniformBuffer, VertexBuffer,
    },
    pass::MatricesBlock,
    Canvas, Color4, Context, FrameError,
};

use super::{
    compose_pass::ComposePass,
    geometry_color_pass::GeometryColorPass,
    geometry_sprite_normal_pass::GeometrySpriteNormalPass,
    light_area::{LightAreaVertex, LightCircleSegment},
    screen_light_pass::ScreenLightPass,
    shadow_map_pass::ShadowMapPass,
    GlobalLightParams, GlobalLightParamsBlock, Light, OccluderBatch,
};

#[derive(Debug, Clone)]
pub struct LightPipelineParams {
    pub shadow_map_resolution: u32,
    pub max_num_lights: u32,
}

pub struct LightPipeline {
    canvas: Rc<RefCell<Canvas>>,

    light_instances: Rc<VertexBuffer<Light>>,
    light_area_batch: TriangleBatch<LightAreaVertex>,
    global_light_params: UniformBuffer<GlobalLightParamsBlock>,

    screen_geometry: Framebuffer,
    shadow_map: Framebuffer,
    screen_light: Framebuffer,

    geometry_color_pass: GeometryColorPass,
    geometry_sprite_normal_pass: GeometrySpriteNormalPass,
    shadow_map_pass: ShadowMapPass,
    screen_light_pass: ScreenLightPass,
    compose_pass: ComposePass,
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
        let light_area_batch = TriangleBatch::new(context.gl())?;
        let global_light_params = UniformBuffer::new(
            context.gl(),
            GlobalLightParamsBlock {
                ambient: Vector3::zeros(),
            },
        )?;

        let screen_geometry = new_screen_framebuffer(canvas.clone(), 2, true)?;
        let shadow_map = Framebuffer::from_textures(
            context.gl(),
            vec![Texture::new(
                context.gl(),
                Vector2::new(params.shadow_map_resolution, params.max_num_lights),
                TextureParams {
                    value_type: TextureValueType::RgbaF32,
                    min_filter: TextureMinFilter::Linear,
                    mag_filter: TextureMagFilter::Linear,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?],
        )?;
        let screen_light = new_screen_framebuffer(canvas.clone(), 1, false)?;

        let geometry_color_pass = GeometryColorPass::new(context.gl())?;
        let geometry_sprite_normal_pass = GeometrySpriteNormalPass::new(context.gl())?;
        let shadow_map_pass = ShadowMapPass::new(context.gl(), params.max_num_lights)?;
        let screen_light_pass = ScreenLightPass::new(context.gl(), params.clone())?;
        let compose_pass = ComposePass::new(context.gl())?;

        Ok(Self {
            canvas,
            light_instances,
            light_area_batch,
            global_light_params,
            screen_geometry,
            shadow_map,
            screen_light,
            geometry_color_pass,
            geometry_sprite_normal_pass,
            shadow_map_pass,
            screen_light_pass,
            compose_pass,
        })
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.shadow_map.gl()
    }

    pub fn shadow_map(&self) -> &Texture {
        &self.shadow_map.textures()[0]
    }

    pub fn screen_albedo(&self) -> &Texture {
        &self.screen_geometry.textures()[0]
    }

    pub fn screen_normals(&self) -> &Texture {
        &self.screen_geometry.textures()[1]
    }

    pub fn screen_light(&self) -> &Texture {
        &self.screen_light.textures()[0]
    }

    pub fn new_occluder_batch(&self) -> Result<OccluderBatch, gl::Error> {
        OccluderBatch::new(self.light_instances.clone())
    }

    pub fn geometry_phase<'a>(
        &'a mut self,
        matrices: &'a UniformBuffer<MatricesBlock>,
    ) -> Result<GeometryPhase<'a>, FrameError> {
        if self.screen_geometry.textures()[0].size() != screen_light_size(self.canvas.clone()) {
            self.screen_geometry = new_screen_framebuffer(self.canvas.clone(), 2, true)?;
            self.screen_light = new_screen_framebuffer(self.canvas.clone(), 1, false)?;
        }

        Ok(GeometryPhase::new(self, Input { matrices }))
    }
}

struct Input<'a> {
    matrices: &'a UniformBuffer<MatricesBlock>,
}

#[must_use]
pub struct GeometryPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: Input<'a>,
}

#[must_use]
pub struct ShadowMapPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: Input<'a>,
    lights: &'a [Light],
}

pub struct ComposePhase<'a> {
    pipeline: &'a mut LightPipeline,
}

impl<'a> GeometryPhase<'a> {
    fn new(pipeline: &'a mut LightPipeline, input: Input<'a>) -> Self {
        gl::with_framebuffer(&pipeline.screen_geometry, || {
            gl::clear_color_and_depth(&pipeline.gl(), Color4::new(0.0, 0.0, 0.0, 1.0), 1.0);
        });

        Self { pipeline, input }
    }

    pub fn draw_geometry_colors<E>(self, draw_unit: DrawUnit<ColorVertex, E>) -> Self
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline
                .geometry_color_pass
                .draw(self.input.matrices, draw_unit);
        });

        self
    }

    pub fn draw_geometry_sprite_normals<E>(
        self,
        texture: &Texture,
        normal_map: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
    ) -> Result<Self, FrameError>
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline.geometry_sprite_normal_pass.draw(
                self.input.matrices,
                texture,
                normal_map,
                draw_unit,
            )
        })?;

        Ok(self)
    }

    pub fn shadow_map_phase(self, lights: &'a [Light]) -> ShadowMapPhase<'a> {
        ShadowMapPhase::new(self, lights)
    }
}

impl<'a> ShadowMapPhase<'a> {
    fn new(phase: GeometryPhase<'a>, lights: &'a [Light]) -> Self {
        phase.pipeline.light_instances.set_data(lights);

        gl::with_framebuffer(&phase.pipeline.shadow_map, || {
            gl::clear_color(
                &*phase.pipeline.shadow_map.gl(),
                Color4::new(1.0, 1.0, 1.0, 1.0),
            );
        });

        Self {
            pipeline: phase.pipeline,
            input: phase.input,
            lights,
        }
    }

    pub fn draw_occluders(self, batch: &mut OccluderBatch) -> Self {
        gl::with_framebuffer(&self.pipeline.shadow_map, || {
            self.pipeline.shadow_map_pass.draw(batch.draw_unit())
        });

        self
    }

    pub fn build_screen_light(self) -> ComposePhase<'a> {
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
                        light: *light,
                        num_segments: 16,
                    }),
            );

        gl::with_framebuffer(&self.pipeline.screen_light, || {
            gl::clear_color(
                &*self.pipeline.screen_light.gl(),
                Color4::new(0.0, 0.0, 0.0, 1.0),
            );

            self.pipeline.screen_light_pass.draw(
                self.input.matrices,
                &self.pipeline.shadow_map.textures()[0],
                &self.pipeline.screen_geometry.textures()[1],
                self.pipeline.light_area_batch.draw_unit(),
            );
        });

        ComposePhase::new(self)
    }
}

impl<'a> ComposePhase<'a> {
    fn new(phase: ShadowMapPhase<'a>) -> Self {
        Self {
            pipeline: phase.pipeline,
        }
    }

    pub fn compose(self, global_light_params: GlobalLightParams) {
        self.pipeline
            .global_light_params
            .set_data(global_light_params.into());

        self.pipeline.compose_pass.draw(
            &self.pipeline.global_light_params,
            &self.pipeline.screen_geometry.textures()[0],
            &self.pipeline.screen_light.textures()[0],
        );
    }
}

#[must_use]
fn screen_light_size(canvas: Rc<RefCell<Canvas>>) -> Vector2<u32> {
    let canvas = canvas.borrow();
    let physical_size = canvas.physical_size();
    let max_size = Texture::max_size(&*canvas.gl());

    Vector2::new(physical_size.x.min(max_size), physical_size.y.min(max_size))
}

fn new_screen_framebuffer(
    canvas: Rc<RefCell<Canvas>>,
    num_textures: usize,
    depth: bool,
) -> Result<Framebuffer, NewFramebufferError> {
    let mut textures = (0..num_textures)
        .map(|_| {
            Texture::new(
                canvas.borrow().gl(),
                screen_light_size(canvas.clone()),
                TextureParams {
                    value_type: TextureValueType::RgbaF32,
                    min_filter: TextureMinFilter::Linear,
                    mag_filter: TextureMagFilter::Linear,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    if depth {
        textures.push(Texture::new(
            canvas.borrow().gl(),
            screen_light_size(canvas.clone()),
            TextureParams {
                value_type: TextureValueType::Depth,
                min_filter: TextureMinFilter::Linear,
                mag_filter: TextureMagFilter::Linear,
                wrap_vertical: TextureWrap::ClampToEdge,
                wrap_horizontal: TextureWrap::ClampToEdge,
            },
        )?);
    }

    Framebuffer::from_textures(canvas.borrow().gl(), textures)
}

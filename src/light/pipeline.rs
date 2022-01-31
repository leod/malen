//! This implementation follows the following with some modifications:
//! https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php

use std::{cell::RefCell, rc::Rc};

use nalgebra::Vector2;
use thiserror::Error;

use crate::{
    data::{ColorVertex, SpriteVertex, TriangleBatch},
    gl::{
        self, DrawParams, DrawUnit, Element, Framebuffer, NewFramebufferError, NewTextureError,
        Texture, TextureMagFilter, TextureMinFilter, TextureParams, TextureValueType, TextureWrap,
        Uniform, VertexBuffer,
    },
    pass::{ColorPass, MatricesBlock},
    Canvas, Color4, Context, FrameError,
};

use super::{
    light_area::{LightAreaVertex, LightCircleSegment},
    pass::{
        compose::ComposePass, compose_with_indirect::ComposeWithIndirectPass,
        geometry_color::GeometryColorPass, geometry_sprite::GeometrySpritePass,
        geometry_sprite_with_normals::GeometrySpriteWithNormalsPass, screen_light::ScreenLightPass,
        shaded_color::ShadedColorPass, shaded_sprite::ShadedSpritePass, shadow_map::ShadowMapPass,
    },
    GlobalLightParams, GlobalLightParamsBlock, Light, LightPipelineParams, ObjectLightParams,
    OccluderBatch,
};

pub struct LightPipeline {
    canvas: Rc<RefCell<Canvas>>,

    light_instances: Rc<VertexBuffer<Light>>,
    light_area_batch: TriangleBatch<LightAreaVertex>,
    global_light_params: Uniform<GlobalLightParamsBlock>,

    screen_geometry: Framebuffer,
    screen_reflectors: Framebuffer,
    shadow_map: Framebuffer,
    screen_light: Framebuffer,

    color_pass: Rc<ColorPass>,
    geometry_color_pass: GeometryColorPass,
    geometry_sprite_pass: GeometrySpritePass,
    geometry_sprite_normal_pass: GeometrySpriteWithNormalsPass,
    shadow_map_pass: ShadowMapPass,
    screen_light_pass: ScreenLightPass,
    shaded_color_pass: ShadedColorPass,
    shaded_sprite_pass: ShadedSpritePass,
    compose_pass: ComposePass,
    compose_with_indirect_pass: ComposeWithIndirectPass,
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

const SCREEN_ALBEDO_LOCATION: usize = 0;
const SCREEN_NORMALS_LOCATION: usize = 1;
const SCREEN_OCCLUSION_LOCATION: usize = 2;

impl LightPipeline {
    pub fn new(
        context: &Context,
        params: LightPipelineParams,
    ) -> Result<LightPipeline, NewLightPipelineError> {
        let canvas = context.canvas();

        let light_instances = Rc::new(VertexBuffer::new(context.gl())?);
        let light_area_batch = TriangleBatch::new(context.gl())?;
        let global_light_params = Uniform::new(
            context.gl(),
            GlobalLightParamsBlock::new(GlobalLightParams::default()),
        )?;

        let screen_geometry = new_screen_geometry(canvas.clone())?;
        let screen_reflectors = new_screen_reflectors(&screen_geometry)?;
        let shadow_map = new_shadow_map(context.gl(), &params)?;
        let screen_light = new_screen_light(canvas.clone())?;

        let color_pass = context.color_pass();
        let geometry_color_pass = GeometryColorPass::new(context.gl())?;
        let geometry_sprite_pass = GeometrySpritePass::new(context.gl())?;
        let geometry_sprite_normal_pass = GeometrySpriteWithNormalsPass::new(context.gl())?;
        let shadow_map_pass = ShadowMapPass::new(context.gl(), params.max_num_lights)?;
        let screen_light_pass = ScreenLightPass::new(context.gl(), params.clone())?;
        let shaded_color_pass = ShadedColorPass::new(context.gl())?;
        let shaded_sprite_pass = ShadedSpritePass::new(context.gl())?;
        let compose_pass = ComposePass::new(context.gl())?;
        let compose_with_indirect_pass = ComposeWithIndirectPass::new(context.gl(), params)?;

        Ok(Self {
            canvas,
            light_instances,
            light_area_batch,
            global_light_params,
            screen_geometry,
            screen_reflectors,
            shadow_map,
            screen_light,
            color_pass,
            geometry_color_pass,
            geometry_sprite_pass,
            geometry_sprite_normal_pass,
            shadow_map_pass,
            screen_light_pass,
            shaded_color_pass,
            shaded_sprite_pass,
            compose_pass,
            compose_with_indirect_pass,
        })
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.shadow_map.gl()
    }

    pub fn shadow_map_framebuffer(&self) -> &Framebuffer {
        &self.shadow_map
    }

    pub fn shadow_map(&self) -> &Texture {
        &self.shadow_map.textures()[0]
    }

    pub fn screen_albedo(&self) -> &Texture {
        &self.screen_geometry.textures()[SCREEN_ALBEDO_LOCATION]
    }

    pub fn screen_normals(&self) -> &Texture {
        &self.screen_geometry.textures()[SCREEN_NORMALS_LOCATION]
    }

    pub fn screen_occlusion(&self) -> &Texture {
        &self.screen_geometry.textures()[SCREEN_OCCLUSION_LOCATION]
    }

    pub fn screen_light(&self) -> &Texture {
        &self.screen_light.textures()[0]
    }

    pub fn new_occluder_batch(&self) -> Result<OccluderBatch, gl::Error> {
        OccluderBatch::new(self.light_instances.clone())
    }

    pub fn geometry_phase<'a>(
        &'a mut self,
        matrices: &'a Uniform<MatricesBlock>,
    ) -> Result<GeometryPhase<'a>, FrameError> {
        if self.screen_geometry.textures()[0].size() != screen_light_size(self.canvas.clone()) {
            self.screen_geometry = new_screen_geometry(self.canvas.clone())?;
            self.screen_reflectors = new_screen_reflectors(&self.screen_geometry)?;
            self.screen_light = new_screen_light(self.canvas.clone())?;
        }

        {
            #[cfg(feature = "coarse-prof")]
            coarse_prof::profile!("clear_geometry");

            gl::with_framebuffer(&self.screen_geometry, || {
                gl::clear_color_and_depth(&self.gl(), Color4::new(0.0, 0.0, 0.0, 1.0), 1.0);
            });
        }

        Ok(GeometryPhase {
            pipeline: self,
            input: PhaseInput { matrices },
            #[cfg(feature = "coarse-prof")]
            guard: coarse_prof::enter("geometry"),
        })
    }
}

struct PhaseInput<'a> {
    matrices: &'a Uniform<MatricesBlock>,
}

#[must_use]
pub struct GeometryPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: PhaseInput<'a>,
    #[cfg(feature = "coarse-prof")]
    guard: coarse_prof::Guard,
}

#[must_use]
pub struct ShadowMapPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: PhaseInput<'a>,
    lights: &'a [Light],
    #[cfg(feature = "coarse-prof")]
    guard: coarse_prof::Guard,
}

pub struct BuiltScreenLightPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: PhaseInput<'a>,
    #[cfg(feature = "coarse-prof")]
    guard: coarse_prof::Guard,
}

pub struct IndirectLightPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: PhaseInput<'a>,
    #[cfg(feature = "coarse-prof")]
    guard: coarse_prof::Guard,
}

pub struct ComposeWithIndirectPhase<'a> {
    pipeline: &'a mut LightPipeline,
    #[cfg(feature = "coarse-prof")]
    _guard: coarse_prof::Guard,
}

impl<'a> GeometryPhase<'a> {
    pub fn draw_colors<E>(
        self,
        object_light_params: &Uniform<ObjectLightParams>,
        draw_unit: DrawUnit<ColorVertex, E>,
        draw_params: &DrawParams,
    ) -> Self
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline.geometry_color_pass.draw(
                self.input.matrices,
                object_light_params,
                draw_unit,
                draw_params,
            );
        });

        self
    }

    pub fn draw_sprites<E>(
        self,
        object_light_params: &Uniform<ObjectLightParams>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        draw_params: &DrawParams,
    ) -> Self
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline.geometry_sprite_pass.draw(
                self.input.matrices,
                object_light_params,
                texture,
                draw_unit,
                draw_params,
            );
        });

        self
    }

    pub fn draw_sprites_with_normals<E>(
        self,
        object_light_params: &Uniform<ObjectLightParams>,
        texture: &Texture,
        normal_map: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        draw_params: &DrawParams,
    ) -> Self
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline.geometry_sprite_normal_pass.draw(
                self.input.matrices,
                object_light_params,
                texture,
                normal_map,
                draw_unit,
                draw_params,
            );
        });

        self
    }

    pub fn shadow_map_phase(self, lights: &'a [Light]) -> ShadowMapPhase<'a> {
        self.pipeline.light_instances.set(lights);

        gl::with_framebuffer(&self.pipeline.shadow_map, || {
            gl::clear_color(
                &*self.pipeline.shadow_map.gl(),
                Color4::new(1.0, 1.0, 1.0, 1.0),
            );
        });

        drop(self.guard);

        ShadowMapPhase {
            pipeline: self.pipeline,
            input: self.input,
            lights,
            #[cfg(feature = "coarse-prof")]
            guard: coarse_prof::enter("shadow_map"),
        }
    }
}

impl<'a> ShadowMapPhase<'a> {
    pub fn draw_occluders(self, batch: &mut OccluderBatch) -> Self {
        gl::with_framebuffer(&self.pipeline.shadow_map, || {
            self.pipeline
                .shadow_map_pass
                .draw(batch.light_instanced_draw_unit());
        });

        self
    }

    pub fn build_screen_light(
        self,
        global_light_params: GlobalLightParams,
    ) -> BuiltScreenLightPhase<'a> {
        self.pipeline
            .global_light_params
            .set(GlobalLightParamsBlock::new(global_light_params));

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

            let draw_unit = self.pipeline.light_area_batch.draw_unit();
            self.pipeline.screen_light_pass.draw(
                self.input.matrices,
                &self.pipeline.global_light_params,
                &self.pipeline.shadow_map.textures()[0],
                &self.pipeline.screen_geometry.textures()[SCREEN_NORMALS_LOCATION],
                draw_unit,
            );
        });

        //self.pipeline.shadow_map.invalidate();

        drop(self.guard);

        BuiltScreenLightPhase {
            pipeline: self.pipeline,
            input: self.input,
            #[cfg(feature = "coarse-prof")]
            guard: coarse_prof::enter("screen_light"),
        }
    }
}

impl<'a> BuiltScreenLightPhase<'a> {
    pub fn indirect_light_phase(self) -> IndirectLightPhase<'a> {
        drop(self.guard);

        IndirectLightPhase {
            pipeline: self.pipeline,
            input: self.input,
            #[cfg(feature = "coarse-prof")]
            guard: coarse_prof::enter("indirect_light"),
        }
    }

    pub fn compose(self) {
        self.pipeline.compose_pass.draw(
            &self.pipeline.global_light_params,
            &self.pipeline.screen_geometry.textures()[SCREEN_ALBEDO_LOCATION],
            &self.pipeline.screen_light.textures()[0],
        );

        /*self.pipeline.screen_geometry.invalidate();
        self.pipeline.screen_reflectors.invalidate();
        self.pipeline.screen_light.invalidate();*/
    }
}

impl<'a> IndirectLightPhase<'a> {
    fn draw_params(draw_params: &DrawParams) -> DrawParams {
        DrawParams {
            color_mask: (true, true, true, false),
            ..draw_params.clone()
        }
    }

    pub fn draw_color_reflectors(
        self,
        draw_unit: DrawUnit<ColorVertex>,
        draw_params: &DrawParams,
    ) -> Self {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            self.pipeline.shaded_color_pass.draw(
                self.input.matrices,
                &self.pipeline.screen_light.textures()[0],
                draw_unit,
                &Self::draw_params(draw_params),
            );
        });

        self
    }

    pub fn draw_sprite_reflectors(
        self,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex>,
        draw_params: &DrawParams,
    ) -> Self {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            self.pipeline.shaded_sprite_pass.draw(
                self.input.matrices,
                texture,
                &self.pipeline.screen_light.textures()[0],
                draw_unit,
                &Self::draw_params(draw_params),
            )
        });

        self
    }

    pub fn draw_color_sources(self, draw_unit: DrawUnit<ColorVertex>) -> Self {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            self.pipeline.color_pass.draw(
                self.input.matrices,
                draw_unit,
                &Self::draw_params(&DrawParams::default()),
            );
        });

        self
    }

    pub fn prepare_cone_tracing(self) -> ComposeWithIndirectPhase<'a> {
        self.pipeline.screen_occlusion().generate_mipmap();

        drop(self.guard);

        ComposeWithIndirectPhase {
            pipeline: self.pipeline,
            #[cfg(feature = "coarse-prof")]
            _guard: coarse_prof::enter("compose_with_indirect"),
        }
    }
}

impl<'a> ComposeWithIndirectPhase<'a> {
    pub fn compose(self) {
        self.pipeline.compose_with_indirect_pass.draw(
            &self.pipeline.global_light_params,
            &self.pipeline.screen_geometry.textures()[SCREEN_ALBEDO_LOCATION],
            &self.pipeline.screen_geometry.textures()[SCREEN_NORMALS_LOCATION],
            &self.pipeline.screen_geometry.textures()[SCREEN_OCCLUSION_LOCATION],
            &self.pipeline.screen_light.textures()[0],
        );

        /*self.pipeline.screen_geometry.invalidate();
        self.pipeline.screen_reflectors.invalidate();
        self.pipeline.screen_light.invalidate();*/
    }
}

fn screen_light_size(canvas: Rc<RefCell<Canvas>>) -> Vector2<u32> {
    let canvas = canvas.borrow();
    let physical_size = canvas.physical_size();
    let max_size = Texture::max_size(&*canvas.gl());

    Vector2::new(physical_size.x.min(max_size), physical_size.y.min(max_size))
}

fn new_shadow_map(
    gl: Rc<gl::Context>,
    params: &LightPipelineParams,
) -> Result<Framebuffer, NewFramebufferError> {
    Framebuffer::from_textures(
        gl.clone(),
        vec![Texture::new(
            gl,
            Vector2::new(params.shadow_map_resolution, params.max_num_lights),
            TextureParams {
                value_type: TextureValueType::RgF16,
                min_filter: TextureMinFilter::Linear,
                mag_filter: TextureMagFilter::Linear,
                wrap_vertical: TextureWrap::ClampToEdge,
                wrap_horizontal: TextureWrap::ClampToEdge,
            },
        )?],
    )
}

fn new_screen_geometry(canvas: Rc<RefCell<Canvas>>) -> Result<Framebuffer, NewFramebufferError> {
    let size = screen_light_size(canvas.clone());
    let albedo = Texture::new(
        canvas.borrow().gl(),
        size,
        TextureParams {
            value_type: TextureValueType::RgbaU8,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Nearest,
            wrap_vertical: TextureWrap::ClampToEdge,
            wrap_horizontal: TextureWrap::ClampToEdge,
        },
    )?;
    let normals = Texture::new(
        canvas.borrow().gl(),
        size,
        TextureParams {
            value_type: TextureValueType::RgbaU8,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Nearest,
            wrap_vertical: TextureWrap::ClampToEdge,
            wrap_horizontal: TextureWrap::ClampToEdge,
        },
    )?;
    let occluder = Texture::new(
        canvas.borrow().gl(),
        size,
        TextureParams {
            value_type: TextureValueType::RgbaU8,
            min_filter: TextureMinFilter::LinearMipmapLinear,
            mag_filter: TextureMagFilter::Linear,
            wrap_vertical: TextureWrap::ClampToEdge,
            wrap_horizontal: TextureWrap::ClampToEdge,
        },
    )?;
    let depth = Texture::new(
        canvas.borrow().gl(),
        size,
        TextureParams {
            value_type: TextureValueType::Depth,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Nearest,
            wrap_vertical: TextureWrap::ClampToEdge,
            wrap_horizontal: TextureWrap::ClampToEdge,
        },
    )?;

    // Texture order corresponds to SCREEN_ALBEDO_LOCATION, etc.
    Framebuffer::from_textures(canvas.borrow().gl(), vec![albedo, normals, occluder, depth])
}

fn new_screen_reflectors(
    screen_geometry: &Framebuffer,
) -> Result<Framebuffer, NewFramebufferError> {
    Framebuffer::new(
        screen_geometry.gl(),
        vec![screen_geometry.textures()[2].clone()],
    )
}

fn new_screen_light(canvas: Rc<RefCell<Canvas>>) -> Result<Framebuffer, NewFramebufferError> {
    let size = screen_light_size(canvas.clone());
    let light = Texture::new(
        canvas.borrow().gl(),
        size,
        TextureParams {
            value_type: TextureValueType::RgbaF32,
            min_filter: TextureMinFilter::Nearest,
            mag_filter: TextureMagFilter::Nearest,
            wrap_vertical: TextureWrap::ClampToEdge,
            wrap_horizontal: TextureWrap::ClampToEdge,
        },
    )?;

    Framebuffer::from_textures(canvas.borrow().gl(), vec![light])
}

//! This implementation follows the following with some modifications:
//! https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php

use std::{cell::RefCell, rc::Rc};

use nalgebra::Vector2;
use thiserror::Error;

use crate::{
    data::{ColorVertex, SpriteVertex, TriangleBatch},
    gl::{
        self, DrawUnit, Element, Framebuffer, NewFramebufferError, NewTextureError, Texture,
        TextureMagFilter, TextureMinFilter, TextureParams, TextureValueType, TextureWrap, Uniform,
        VertexBuffer,
    },
    pass::MatricesBlock,
    Canvas, Color4, Context, FrameError,
};

use super::{
    light_area::{LightAreaVertex, LightCircleSegment},
    pass::{
        compose::ComposePass, geometry_color::GeometryColorPass,
        geometry_sprite_with_normals::GeometrySpriteWithNormalsPass, reflector::ReflectorPass,
        screen_light::ScreenLightPass, shaded_color::ShadedColorPass,
        shaded_sprite::ShadedSpritePass, shadow_map::ShadowMapPass,
    },
    GlobalLightParams, GlobalLightParamsBlock, Light, ObjectLightParams, OccluderBatch,
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
    global_light_params: Uniform<GlobalLightParamsBlock>,

    screen_geometry: Framebuffer,
    shadow_map: Framebuffer,
    screen_light: Framebuffer,
    screen_reflectors: Framebuffer,

    geometry_color_pass: GeometryColorPass,
    geometry_sprite_normal_pass: GeometrySpriteWithNormalsPass,
    shadow_map_pass: ShadowMapPass,
    screen_light_pass: ScreenLightPass,
    reflector_pass: ReflectorPass,
    shaded_sprite_pass: ShadedSpritePass,
    shaded_color_pass: ShadedColorPass,
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
        let global_light_params = Uniform::new(context.gl(), GlobalLightParams::default().into())?;

        let screen_geometry = new_screen_framebuffer(canvas.clone(), 3, true, true)?;
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
        let screen_light = new_screen_framebuffer(canvas.clone(), 1, false, false)?;
        let screen_reflectors = new_screen_framebuffer(canvas.clone(), 1, false, true)?;

        let geometry_color_pass = GeometryColorPass::new(context.gl())?;
        let geometry_sprite_normal_pass = GeometrySpriteWithNormalsPass::new(context.gl())?;
        let shadow_map_pass = ShadowMapPass::new(context.gl(), params.max_num_lights)?;
        let screen_light_pass = ScreenLightPass::new(context.gl(), params.clone())?;
        let reflector_pass = ReflectorPass::new(context.gl())?;
        let shaded_sprite_pass = ShadedSpritePass::new(context.gl())?;
        let shaded_color_pass = ShadedColorPass::new(context.gl())?;
        let compose_pass = ComposePass::new(context.gl())?;

        Ok(Self {
            canvas,
            light_instances,
            light_area_batch,
            global_light_params,
            screen_geometry,
            shadow_map,
            screen_light,
            screen_reflectors,
            geometry_color_pass,
            geometry_sprite_normal_pass,
            shadow_map_pass,
            screen_light_pass,
            reflector_pass,
            shaded_sprite_pass,
            shaded_color_pass,
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

    pub fn screen_occlusion(&self) -> &Texture {
        &self.screen_geometry.textures()[2]
    }

    pub fn screen_light(&self) -> &Texture {
        &self.screen_light.textures()[0]
    }

    pub fn screen_reflectors(&self) -> &Texture {
        &self.screen_reflectors.textures()[0]
    }

    pub fn new_occluder_batch(&self) -> Result<OccluderBatch, gl::Error> {
        OccluderBatch::new(self.light_instances.clone())
    }

    pub fn geometry_phase<'a>(
        &'a mut self,
        matrices: &'a Uniform<MatricesBlock>,
    ) -> Result<GeometryPhase<'a>, FrameError> {
        if self.screen_geometry.textures()[0].size() != screen_light_size(self.canvas.clone()) {
            self.screen_geometry = new_screen_framebuffer(self.canvas.clone(), 3, true, true)?;
            self.screen_light = new_screen_framebuffer(self.canvas.clone(), 1, false, false)?;
            self.screen_reflectors = new_screen_framebuffer(self.canvas.clone(), 1, false, true)?;
        }

        #[cfg(feature = "coarse-prof")]
        coarse_prof::profile!("clear_geometry");

        gl::with_framebuffer(&self.screen_geometry, || {
            gl::clear_color_and_depth(&self.gl(), Color4::new(0.0, 0.0, 0.0, 1.0), 1.0);
        });

        Ok(GeometryPhase {
            pipeline: self,
            input: Input { matrices },
        })
    }
}

struct Input<'a> {
    matrices: &'a Uniform<MatricesBlock>,
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

pub struct BuiltScreenLightPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: Input<'a>,
}

pub struct IndirectLightPhase<'a> {
    pipeline: &'a mut LightPipeline,
    input: Input<'a>,
}

pub struct ComposePhase<'a> {
    pipeline: &'a mut LightPipeline,
}

impl<'a> GeometryPhase<'a> {
    pub fn draw_colors<E>(
        self,
        object_light_params: &Uniform<ObjectLightParams>,
        draw_unit: DrawUnit<ColorVertex, E>,
    ) -> Self
    where
        E: Element,
    {
        gl::with_framebuffer(&self.pipeline.screen_geometry, || {
            self.pipeline.geometry_color_pass.draw(
                self.input.matrices,
                object_light_params,
                draw_unit,
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
    ) -> Result<Self, FrameError>
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
            )
        })?;

        Ok(self)
    }

    pub fn shadow_map_phase(self, lights: &'a [Light]) -> ShadowMapPhase<'a> {
        self.pipeline.light_instances.set(lights);

        gl::with_framebuffer(&self.pipeline.shadow_map, || {
            gl::clear_color(
                &*self.pipeline.shadow_map.gl(),
                Color4::new(1.0, 1.0, 1.0, 1.0),
            );
        });

        ShadowMapPhase {
            pipeline: self.pipeline,
            input: self.input,
            lights,
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
            .set(global_light_params.into());

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
                &self.pipeline.global_light_params,
                &self.pipeline.shadow_map.textures()[0],
                &self.pipeline.screen_geometry.textures()[1],
                self.pipeline.light_area_batch.draw_unit(),
            );
        });

        BuiltScreenLightPhase {
            pipeline: self.pipeline,
            input: self.input,
        }
    }
}

impl<'a> BuiltScreenLightPhase<'a> {
    pub fn indirect_light_phase(self) -> IndirectLightPhase<'a> {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            gl::clear_color(&self.pipeline.gl(), Color4::new(0.0, 0.0, 0.0, 1.0));
        });

        IndirectLightPhase {
            pipeline: self.pipeline,
            input: self.input,
        }
    }

    pub fn compose(self) {
        compose(self.pipeline);
    }
}

impl<'a> IndirectLightPhase<'a> {
    pub fn draw_color_reflectors(self, draw_unit: DrawUnit<ColorVertex>) -> Self {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            self.pipeline.shaded_color_pass.draw(
                self.input.matrices,
                &self.pipeline.screen_light.textures()[0],
                draw_unit,
            );
        });

        self
    }

    pub fn draw_sprite_reflectors(
        self,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex>,
    ) -> Result<Self, FrameError> {
        gl::with_framebuffer(&self.pipeline.screen_reflectors, || {
            self.pipeline.shaded_sprite_pass.draw(
                self.input.matrices,
                texture,
                &self.pipeline.screen_light.textures()[0],
                draw_unit,
            )
        })?;

        Ok(self)
    }

    pub fn prepare_cone_tracing(self) -> ComposePhase<'a> {
        self.pipeline.screen_occlusion().generate_mipmap();
        self.pipeline.screen_reflectors().generate_mipmap();

        ComposePhase {
            pipeline: self.pipeline,
        }
    }
}

impl<'a> ComposePhase<'a> {
    pub fn compose(self) {
        compose(self.pipeline);
    }
}

fn compose(pipeline: &mut LightPipeline) {
    pipeline.compose_pass.draw(
        &pipeline.global_light_params,
        &pipeline.screen_geometry.textures()[0],
        &pipeline.screen_geometry.textures()[1],
        &pipeline.screen_geometry.textures()[2],
        &pipeline.screen_light.textures()[0],
        &pipeline.screen_reflectors.textures()[0],
    );
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
    mip_map: bool,
) -> Result<Framebuffer, NewFramebufferError> {
    let mut textures = (0..num_textures)
        .map(|i| {
            Texture::new(
                canvas.borrow().gl(),
                screen_light_size(canvas.clone()),
                TextureParams {
                    value_type: TextureValueType::RgbaF32,
                    min_filter: if mip_map && i + 1 == num_textures {
                        TextureMinFilter::LinearMipmapLinear
                    } else {
                        TextureMinFilter::Nearest
                    },
                    mag_filter: TextureMagFilter::Nearest,
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
                min_filter: TextureMinFilter::Nearest,
                mag_filter: TextureMagFilter::Nearest,
                wrap_vertical: TextureWrap::ClampToEdge,
                wrap_horizontal: TextureWrap::ClampToEdge,
            },
        )?);
    }

    Framebuffer::from_textures(canvas.borrow().gl(), textures)
}

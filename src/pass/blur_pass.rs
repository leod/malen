use std::rc::Rc;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{
        self, DrawParams, Framebuffer, NewFramebufferError, Texture, TextureMagFilter,
        TextureMinFilter, TextureParams, TextureWrap, Uniform, UniformBlock,
    },
    program, Color4,
};

use super::BLUR_PROPS_BLOCK_BINDING;

#[derive(Debug, Clone)]
pub struct BlurParams {
    pub weights: Vec<f32>,
}

impl Default for BlurParams {
    fn default() -> Self {
        Self {
            weights: vec![0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216],
        }
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct BlurProps {
    direction: f32,
}

impl UniformBlock for BlurProps {}

program! {
    program BlurProgram
    params {
        params: BlurParams,
    }
    uniforms {
        blur_props: BlurProps = BLUR_PROPS_BLOCK_BINDING,
    }
    samplers {
        tex: Sampler2,
    }
    attributes {
        a: SpriteVertex,
    }
    defines [
        num_weights => params.weights.len(),
        weights => params
            .weights
            .iter()
            .map(f32::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    ]
    vertex glsl! {
        out vec2 v_uv;
        void main() {
            gl_Position = vec4(a_position.xyz, 1.0);
            v_uv = a_tex_coords;
        }
    }
    fragment glsl! {
        in vec2 v_uv;
        out vec4 f_color;

        const float weight[{{num_weights}}] = float[] ({{weights}});

        void main() {
            vec2 texel = 1.0 / vec2(textureSize(tex, 0));

            vec3 result = texture(tex, v_uv).rgb * weight[0];

            if (blur_props.direction == 0.0) {
                for (int i = 1; i < {{num_weights}}; i += 1) {
                    result += texture(tex, v_uv + vec2(texel.x * float(i), 0.0)).rgb * weight[i];
                    result += texture(tex, v_uv - vec2(texel.x * float(i), 0.0)).rgb * weight[i];
                }
            } else {
                for (int i = 1; i < {{num_weights}}; i += 1) {
                    result += texture(tex, v_uv + vec2(0.0, texel.y * float(i))).rgb * weight[i];
                    result += texture(tex, v_uv - vec2(0.0, texel.y * float(i))).rgb * weight[i];
                }
            }

            f_color = vec4(result, 1.0);
        }
    }
}

pub struct BlurBuffer {
    back: Option<Framebuffer>,
}

impl BlurBuffer {
    pub fn new(_: Rc<gl::Context>) -> Result<Self, NewFramebufferError> {
        Ok(Self { back: None })
    }
}

pub struct BlurPass {
    screen_rect: Mesh<SpriteVertex>,
    horizontal_props: Uniform<BlurProps>,
    vertical_props: Uniform<BlurProps>,
    program: BlurProgram,
}

impl BlurPass {
    pub fn new(gl: Rc<gl::Context>, params: BlurParams) -> Result<Self, gl::Error> {
        let screen_rect = Mesh::from_geometry(
            gl.clone(),
            Sprite {
                rect: Rect {
                    center: Point2::origin(),
                    size: Vector2::new(2.0, 2.0),
                },
                depth: 0.0,
                tex_rect: Rect::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0)),
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            },
        )?;
        let horizontal_props = Uniform::new(gl.clone(), BlurProps { direction: 0.0 })?;
        let vertical_props = Uniform::new(gl.clone(), BlurProps { direction: 1.0 })?;
        let program = BlurProgram::new(gl, params)?;

        Ok(Self {
            screen_rect,
            horizontal_props,
            vertical_props,
            program,
        })
    }

    pub fn blur(
        &self,
        iters: usize,
        texture: &Texture,
        buffer: &mut BlurBuffer,
        output: &Framebuffer,
    ) -> Result<(), NewFramebufferError> {
        if buffer
            .back
            .as_ref()
            .map_or(true, |b| texture.size() != b.textures()[0].size())
        {
            let back = Texture::new(
                texture.gl(),
                texture.size(),
                TextureParams {
                    value_type: output.textures()[0].params().value_type,
                    min_filter: TextureMinFilter::Nearest,
                    mag_filter: TextureMagFilter::Nearest,
                    wrap_vertical: TextureWrap::ClampToEdge,
                    wrap_horizontal: TextureWrap::ClampToEdge,
                },
            )?;
            buffer.back = Some(Framebuffer::from_textures(texture.gl(), vec![back])?);
        }

        for i in 0..iters {
            gl::with_framebuffer(&buffer.back.as_ref().unwrap(), || {
                gl::draw(
                    &self.program,
                    &self.horizontal_props,
                    [if i == 0 {
                        texture
                    } else {
                        &output.textures()[0]
                    }],
                    self.screen_rect.draw_unit(),
                    &DrawParams::default(),
                );
            });
            gl::with_framebuffer(&output, || {
                gl::draw(
                    &self.program,
                    &self.vertical_props,
                    [&buffer.back.as_ref().unwrap().textures()[0]],
                    self.screen_rect.draw_unit(),
                    &DrawParams::default(),
                );
            });
        }

        Ok(())
    }
}

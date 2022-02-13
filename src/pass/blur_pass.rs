use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{
        self, DrawParams, DrawUnit, Framebuffer, NewFramebufferError, Texture, TextureParams,
        Uniform, UniformBlock,
    },
    program, Color4, FrameError,
};

use super::BLUR_PROPS_BLOCK_BINDING;

#[derive(Debug, Clone)]
pub struct BlurParams {
    pub weights: Vec<f32>,
    pub offsets: Vec<f32>,
}

impl Default for BlurParams {
    fn default() -> Self {
        Self {
            //weights: vec![0.99999],
            //weights: vec![0.2270270270, 0.3162162162, 0.0702702703],
            //offsets: vec![0.0, 1.3846153846, 3.2307692308],
            weights: vec![0.29411764705882354, 0.35294117647058826],
            offsets: vec![0.0, 1.3333333333333333],
        }
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct BlurProps {
    direction: f32,
    level: f32,
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
            .map(|x| format!("{:.8}", x))
            .collect::<Vec<_>>()
            .join(", "),
        offsets => params
            .offsets
            .iter()
            .map(|x| format!("{:.8}", x))
            .collect::<Vec<_>>()
            .join(", "),
    ]
    vertex glsl! {
        out vec2 v_tex_coords;
        void main() {
            gl_Position = vec4(a_position.xyz, 1.0);
            v_tex_coords = a_tex_coords;
        }
    }
    fragment glsl! {
        in vec2 v_tex_coords;
        out vec4 f_color;

        const float weight[{{num_weights}}] = float[] ({{weights}});
        const float offset[{{num_weights}}] = float[] ({{offsets}});

        void main() {
            vec2 texel = 1.0 / vec2(textureSize(tex, 0));
            /*f_color = vec4(textureLod(tex, v_tex_coords, blur_props.level).rgb, 1.0);
            return;*/

            vec3 result = weight[0] * textureLod(tex, v_tex_coords, blur_props.level).rgb;

            if (blur_props.direction == 0.0) {
                for (int i = 1; i < {{num_weights}}; i += 1) {
                    result += weight[i] * textureLod(
                        tex,
                        v_tex_coords + vec2(texel.x * offset[i], 0.0),
                        blur_props.level
                    ).rgb;
                    result += weight[i] * textureLod(
                        tex,
                        v_tex_coords - vec2(texel.x * offset[i], 0.0),
                        blur_props.level
                    ).rgb;
                }
            } else {
                for (int i = 1; i < {{num_weights}}; i += 1) {
                    result += weight[i] * textureLod(
                        tex,
                        v_tex_coords + vec2(0.0, texel.y * offset[i]),
                        blur_props.level
                    ).rgb;
                    result += weight[i] * textureLod(
                        tex,
                        v_tex_coords - vec2(0.0, texel.y * offset[i]),
                        blur_props.level
                    ).rgb;
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
    program: BlurProgram,
    blur_props: RefCell<HashMap<u32, (Uniform<BlurProps>, Uniform<BlurProps>)>>,
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
        let program = BlurProgram::new(gl, params)?;

        Ok(Self {
            screen_rect,
            program,
            blur_props: RefCell::new(HashMap::new()),
        })
    }

    pub fn screen_rect(&self) -> DrawUnit<SpriteVertex> {
        self.screen_rect.draw_unit()
    }

    pub fn blur(
        &self,
        iters: usize,
        texture: &Texture,
        mipmap_level: u32,
        buffer: &mut BlurBuffer,
        output: &Framebuffer,
    ) -> Result<(), FrameError> {
        let input_size = Vector2::new(
            texture.size().x / 2u32.pow(mipmap_level),
            texture.size().y / 2u32.pow(mipmap_level),
        );

        if buffer
            .back
            .as_ref()
            .map_or(true, |b| input_size != b.textures()[0].size())
        {
            let back = Texture::new(
                texture.gl(),
                input_size,
                TextureParams::linear(output.textures()[0].params().value_type),
            )?;
            buffer.back = Some(Framebuffer::from_textures(vec![back])?);
        }

        if !self.blur_props.borrow().contains_key(&mipmap_level) {
            let horizontal_props = Uniform::new(
                self.program.gl(),
                BlurProps {
                    direction: 0.0,
                    level: mipmap_level as f32,
                },
            )?;
            let vertical_props = Uniform::new(
                self.program.gl(),
                BlurProps {
                    direction: 1.0,
                    level: mipmap_level as f32,
                },
            )?;
            self.blur_props
                .borrow_mut()
                .insert(mipmap_level, (horizontal_props, vertical_props));
        }

        let blur_props = self.blur_props.borrow();
        let (horizontal_props, vertical_props) = blur_props.get(&mipmap_level).as_ref().unwrap();

        for i in 0..iters {
            gl::with_framebuffer_invalidating(&buffer.back.as_ref().unwrap(), || {
                gl::draw(
                    &self.program,
                    horizontal_props,
                    [if i == 0 {
                        texture
                    } else {
                        &output.textures()[0]
                    }],
                    self.screen_rect.draw_unit(),
                    &DrawParams::default(),
                );
            });
            gl::with_framebuffer_invalidating(&output, || {
                gl::draw(
                    &self.program,
                    vertical_props,
                    [&buffer.back.as_ref().unwrap().textures()[0]],
                    self.screen_rect.draw_unit(),
                    &DrawParams::default(),
                );
            });
        }

        Ok(())
    }
}

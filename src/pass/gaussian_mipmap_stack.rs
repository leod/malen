use std::rc::Rc;

use nalgebra::Vector2;

use crate::{
    data::SpriteVertex,
    gl::{
        self, DrawParams, Framebuffer, NewFramebufferError, Texture, TextureParams,
        TextureValueType,
    },
    program, FrameError,
};

use super::{BlurBuffer, BlurPass};

pub struct GaussianMipmapStack {
    blur_pass: Rc<BlurPass>,
    downsample_program: DownsampleProgram,
    texture: Rc<Texture>,
    blur_buffers: Vec<BlurLevel>,
    mipmap_targets: Vec<Framebuffer>,
}

impl GaussianMipmapStack {
    pub fn new(blur_pass: Rc<BlurPass>, texture: Rc<Texture>) -> Result<Self, NewFramebufferError> {
        // TODO: Reuse program
        let downsample_program = DownsampleProgram::new(texture.gl())?;

        let mut size = texture.size();
        let levels = (size.x as f32).max(size.y as f32).log2() as u32 + 1;
        assert!(levels > 0);

        let mut blur_buffers = Vec::new();
        let mut mipmap_targets = Vec::new();

        for level in 0..levels - 1 {
            if level + 1 < levels {
                let blur_level = BlurLevel::new(texture.gl(), texture.params().value_type, size)?;
                blur_buffers.push(blur_level);
            }

            if level > 0 {
                let mipmap_target =
                    Framebuffer::new_with_mipmap_levels(vec![(texture.clone(), level)])?;
                mipmap_targets.push(mipmap_target);
            }

            size.x = (size.x / 2).max(1);
            size.y = (size.y / 2).max(1);
        }

        Ok(Self {
            blur_pass,
            downsample_program,
            texture,
            blur_buffers,
            mipmap_targets,
        })
    }

    pub fn texture(&self) -> Rc<Texture> {
        self.texture.clone()
    }

    pub fn create_mipmaps(&mut self) -> Result<(), FrameError> {
        let it = self
            .blur_buffers
            .iter_mut()
            .zip(self.mipmap_targets.iter())
            .enumerate();
        for (mipmap_level, (blur_level, mipmap_target)) in it {
            self.blur_pass.blur(
                5,
                &self.texture,
                mipmap_level as u32,
                &mut blur_level.buffer,
                &blur_level.output,
            )?;
            gl::with_framebuffer(mipmap_target, || {
                gl::draw(
                    &self.downsample_program,
                    (),
                    [&blur_level.output.textures()[0]],
                    self.blur_pass.screen_rect(),
                    &DrawParams::default(),
                );
            });
        }

        Ok(())
    }
}

struct BlurLevel {
    buffer: BlurBuffer,
    output: Framebuffer,
}

impl BlurLevel {
    pub fn new(
        gl: Rc<gl::Context>,
        value_type: TextureValueType,
        size: Vector2<u32>,
    ) -> Result<Self, NewFramebufferError> {
        let blur_buffer = BlurBuffer::new(gl.clone())?;
        let blur_target_texture = Texture::new(gl, size, TextureParams::linear(value_type))?;
        let blur_target = Framebuffer::from_textures(vec![blur_target_texture])?;

        Ok(Self {
            buffer: blur_buffer,
            output: blur_target,
        })
    }
}

program! {
    program DownsampleProgram
    samplers {
        tex: Sampler2,
    }
    attributes {
        a: SpriteVertex,
    }
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

        void main() {
            vec2 uv = v_uv + 0.5 / vec2(textureSize(tex, 0));
            f_color = textureLod(tex, uv, 0.0);
        }
    }
}

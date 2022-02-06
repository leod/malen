use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Texture, Uniform},
    program,
};

use super::{ViewMatrices, MATRICES_BLOCK_BINDING};

program! {
    program SpriteProgram
    uniforms {
        matrices: ViewMatrices = MATRICES_BLOCK_BINDING,
    }
    samplers {
        sprite: Sampler2,
    }
    attributes {
        a: SpriteVertex,
    }
    vertex glsl! {
        out vec2 v_uv;
        out vec4 v_color;

        void main() {
            vec3 position = matrices.projection
                * matrices.view
                * vec3(a_position.xy, 1.0);

            gl_Position = vec4(position.xy, a_position.z, 1.0);

            v_uv = a_tex_coords / vec2(textureSize(sprite, 0));
            v_uv.y = 1.0 - v_uv.y;
            v_color = a_color;
        }
    }
    fragment glsl! {
        in vec2 v_uv;
        in vec4 v_color;
        out vec4 f_color;

        void main() {
            f_color = texture(sprite, v_uv) * v_color;
        }
    }
}

pub struct SpritePass {
    program: SpriteProgram,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = SpriteProgram::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<ViewMatrices>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, [texture], draw_unit, params);
    }
}

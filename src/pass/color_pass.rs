use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Uniform},
    program,
};

use super::{ViewMatrices, MATRICES_BLOCK_BINDING};

program! {
    program ColorProgram
    uniforms {
        matrices: ViewMatrices = MATRICES_BLOCK_BINDING,
    }
    attributes {
        a: ColorVertex,
    }
    vertex glsl!{
        out vec4 v_color;

        void main() {
            vec3 position = matrices.projection
                * matrices.view
                * vec3(a_position.xy, 1.0);

            gl_Position = vec4(position.xy, a_position.z, 1.0);

            v_color = a_color;
        }
    }
    fragment glsl!{
        in vec4 v_color;
        out vec4 f_color;

        void main() {
            f_color = v_color;
        }
    }
}

pub struct ColorPass {
    program: ColorProgram,
}

impl ColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = ColorProgram::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<ViewMatrices>,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, [], draw_unit, params);
    }
}

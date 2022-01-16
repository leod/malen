use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Uniform},
};

use super::{MatricesBlock, MATRICES_BLOCK_BINDING};

pub struct ColorPass {
    program: Program<MatricesBlock, ColorVertex, 0>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("matrices", MATRICES_BLOCK_BINDING)];

const VERTEX_SOURCE: &str = r#"
out vec4 v_color;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);

    v_color = a_color;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec4 v_color;
out vec4 f_color;

void main() {
    f_color = v_color;
}
"#;

impl ColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: UNIFORM_BLOCKS,
            samplers: [],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<MatricesBlock>,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, [], draw_unit, params);
    }
}

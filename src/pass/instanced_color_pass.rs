use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, Element, InstancedDrawUnit, Program, ProgramDef, Uniform},
};

use super::{ColorInstance, MatricesBlock, MATRICES_BLOCK_BINDING};

pub struct InstancedColorPass {
    program: Program<MatricesBlock, (ColorVertex, ColorInstance), 0>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("matrices", MATRICES_BLOCK_BINDING)];

const VERTEX_SOURCE: &str = r#"
out vec4 v_color;

void main() {
    mat2 i_rot = mat2(
        cos(i_angle), -sin(i_angle),
        sin(i_angle), cos(i_angle)
    );
    vec2 world_pos = i_rot * (i_scale * a_position.xy) + i_position;

    vec3 position = matrices.projection
        * matrices.view
        * vec3(world_pos, 1.0);

    gl_Position = vec4(position.xy, a_position.z + i_z, 1.0);

    v_color = a_color * i_color;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
    in vec4 v_color;
    out vec4 f_color;

    void main() {
        f_color = v_color;
    }
"#;

impl InstancedColorPass {
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
        draw_unit: InstancedDrawUnit<(ColorVertex, ColorInstance), E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw_instanced(&self.program, matrices, [], draw_unit, params);
    }
}

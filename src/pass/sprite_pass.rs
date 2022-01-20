use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
};

use super::{MatricesBlock, MATRICES_BLOCK_BINDING};

pub struct SpritePass {
    program: Program<MatricesBlock, SpriteVertex, 1>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("matrices", MATRICES_BLOCK_BINDING)];

const SAMPLERS: [&str; 1] = ["sprite"];

const VERTEX_SOURCE: &str = r#"
    out vec2 v_uv;

    void main() {
        vec3 position = matrices.projection
            * matrices.view
            * vec3(a_position.xy, 1.0);

        gl_Position = vec4(position.xy, a_position.z, 1.0);

        v_uv = a_tex_coords / vec2(textureSize(sprite, 0));
    }
"#;

const FRAGMENT_SOURCE: &str = r#"
    in vec2 v_uv;
    out vec4 f_color;

    void main() {
        f_color = texture(sprite, v_uv);
    }
"#;

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: UNIFORM_BLOCKS,
            samplers: SAMPLERS,
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<MatricesBlock>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, [texture], draw_unit, params);
    }
}

use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

pub struct ShadedColorPass {
    program: Program<MatricesBlock, ColorVertex, 1>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("matrices", MATRICES_BLOCK_BINDING)];

const SAMPLERS: [&str; 1] = ["screen_light"];

const VERTEX_SOURCE: &str = r#"
out vec3 v_color;
out vec2 v_screen_uv;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);
    v_color = vec3(a_color);
    v_screen_uv = vec2(position.xy) * 0.5 + 0.5;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec3 v_color;
in vec2 v_screen_uv;
out vec4 f_color;

void main() {
    vec3 light = texture(screen_light, v_screen_uv).rgb;
    f_color = vec4(v_color * light, 1.0);
}
"#;

impl ShadedColorPass {
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
        screen_light: &Texture,
        draw_unit: DrawUnit<ColorVertex, E>,
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            matrices,
            [screen_light],
            draw_unit,
            draw_params,
        );
    }
}

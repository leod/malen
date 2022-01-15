use std::rc::Rc;

use crate::{
    gl::{self, DrawParams, DrawUnit, Program, ProgramDef, Texture, Uniform},
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{GlobalLightParamsBlock, OccluderLineVertex};

const VERTEX_SOURCE: &str = r#"
out vec2 v_tex_coords;

void main() {
    if (a_order == 2 || a_order == 3) {
        gl_Position = vec4(-10.0, -10.0, -10.0, 1.0);
        return;
    }

    vec3 p = matrices.projection * matrices.view * vec3(a_line_0, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_tex_coords = p.xy * 0.5 + 0.5;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    vec3 albedo = texture(screen_albedo, v_tex_coords).rgb;
    vec3 light = texture(screen_light, v_tex_coords).rgb;

    f_color = vec4(light, 1.0);
}
"#;

pub struct ReflectorPass {
    program: Program<MatricesBlock, OccluderLineVertex, 2>,
}

impl ReflectorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [("matrices", MATRICES_BLOCK_BINDING)],
            samplers: ["screen_albedo", "screen_light"],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw(
        &self,
        matrices: &Uniform<MatricesBlock>,
        screen_albedo: &Texture,
        screen_light: &Texture,
        draw_unit: DrawUnit<OccluderLineVertex>,
    ) {
        gl::draw(
            &self.program,
            matrices,
            [screen_albedo, screen_light],
            draw_unit,
            &DrawParams {
                line_width: 3.0,
                ..DrawParams::default()
            },
        );
    }
}

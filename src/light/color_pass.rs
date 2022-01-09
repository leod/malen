use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, UniformBuffer},
};

use crate::pass::{MatricesBlock, MATRICES_BLOCK_BINDING};

use super::def::GlobalLightParamsBlock;

const VERTEX_SOURCE: &str = r#"
out vec4 v_color;
out vec2 v_tex_coords;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);

    v_color = a_color;
    v_tex_coords = (gl_Position.xy + vec2(1.0, 1.0)) / 2.0;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec4 v_color;
in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    vec3 light = texture(screen_light, v_tex_coords).rgb;
    vec3 diffuse = (global_light_params.ambient + light) * v_color.rgb;

    f_color = vec4(
        pow(diffuse, vec3(1.0 / 2.2)),
        v_color.a
    );
}
"#;

pub struct ColorPass {
    program: Program<(MatricesBlock, GlobalLightParamsBlock), ColorVertex, 1>,
}

impl ColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("global_light_params", 1),
            ],
            samplers: ["screen_light"],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        global_light_params: &UniformBuffer<GlobalLightParamsBlock>,
        screen_light: &Texture,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        #[cfg(feature = "coarse-prof")]
        coarse_prof::profile!("light::ColorPass::draw");

        gl::draw(
            &self.program,
            (matrices, global_light_params),
            [screen_light],
            draw_unit,
            params,
        );
    }
}

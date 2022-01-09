use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DepthTest, DrawParams, DrawUnit, Element, Program, ProgramDef, UniformBuffer},
};

use crate::pass::{MatricesBlock, MATRICES_BLOCK_BINDING};

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
layout (location = 0) out vec4 f_albedo;
layout (location = 1) out vec4 f_normal;

void main() {
    f_albedo = v_color;
    f_normal = vec4(0.0, 0.0, 0.0, 1.0);
}
"#;

pub struct GeometryColorPass {
    program: Program<MatricesBlock, ColorVertex, 0>,
}

impl GeometryColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [("matrices", MATRICES_BLOCK_BINDING)],
            samplers: [],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        draw_unit: DrawUnit<ColorVertex, E>,
    ) where
        E: Element,
    {
        #[cfg(feature = "coarse-prof")]
        coarse_prof::profile!("light::GeometryColorPass::draw");

        gl::draw(
            &self.program,
            matrices,
            [],
            draw_unit,
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );
    }
}

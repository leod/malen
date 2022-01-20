use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DepthTest, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{super::ObjectLightParams, OBJECT_LIGHT_PARAMS_BLOCK_BINDING};

pub struct GeometrySpriteWithNormalsPass {
    program: Program<(MatricesBlock, ObjectLightParams), SpriteVertex, 2>,
}

const UNIFORM_BLOCKS: [(&str, u32); 2] = [
    ("matrices", MATRICES_BLOCK_BINDING),
    ("object_params", OBJECT_LIGHT_PARAMS_BLOCK_BINDING),
];

const SAMPLERS: [&str; 2] = ["sprite", "normal_map"];

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
layout (location = 0) out vec4 f_albedo;
layout (location = 1) out vec4 f_normal;
layout (location = 2) out vec4 f_occlusion;

void main() {
    f_albedo = vec4(pow(texture(sprite, v_uv).rgb, vec3(2.2)), object_params.ambient_scale);
    f_normal = texture(normal_map, v_uv);
    f_occlusion.a = object_params.occlusion;
}
"#;

impl GeometrySpriteWithNormalsPass {
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
        object_params: &Uniform<ObjectLightParams>,
        texture: &Texture,
        normal_map: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            (matrices, object_params),
            [texture, normal_map],
            draw_unit,
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );
    }
}

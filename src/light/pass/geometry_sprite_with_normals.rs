use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
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
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_uv;
in vec4 v_color;
layout (location = 0) out vec4 f_albedo;
layout (location = 1) out vec4 f_normal;
layout (location = 2) out vec4 f_occlusion;

float myatan2(vec2 dir) {
    float angle = asin(dir.x) > 0.0 ? acos(dir.y) : -acos(dir.y);
    return angle;
}

void main() {
    vec4 albedo = texture(sprite, v_uv);
    f_albedo = v_color * vec4(pow(albedo.rgb, vec3(2.2)), albedo.a);

    float angle = -myatan2(vec2(-dFdy(5.0 * v_uv.x), dFdx(5.0 * v_uv.x)));
    mat2 rot = mat2(cos(angle), -sin(angle), sin(angle), cos(angle));
    vec3 normal = texture(normal_map, v_uv).rgb * 2.0 - 1.0;
    normal.xy = rot * normal.xy;

    f_normal = vec4((normal + 1.0) / 2.0, f_albedo.a);
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
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            (matrices, object_params),
            [texture, normal_map],
            draw_unit,
            draw_params,
        );
    }
}

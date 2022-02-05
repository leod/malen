use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Texture, Uniform},
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
    program,
};

use super::{super::ObjectLightParams, OBJECT_LIGHT_PARAMS_BLOCK_BINDING};

program! {
    Program [
        (
            matrices: MatricesBlock = MATRICES_BLOCK_BINDING,
            object_params: ObjectLightParams = OBJECT_LIGHT_PARAMS_BLOCK_BINDING,
        ),
        (sprite),
        (a: SpriteVertex),
    ]
    => (VERTEX_SOURCE, FRAGMENT_SOURCE)
}

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

void main() {
    vec4 albedo = texture(sprite, v_uv);
    f_albedo = v_color * vec4(pow(albedo.rgb, vec3(2.2)), albedo.a);
    f_normal = vec4(0.0, 0.0, 1.0, f_albedo.a);
    f_occlusion = vec4(object_params.occlusion, 0.0, 0.0, f_albedo.a);
}
"#;

pub struct GeometrySpritePass {
    program: Program,
}

impl GeometrySpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<MatricesBlock>,
        object_params: &Uniform<ObjectLightParams>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            (matrices, object_params),
            [texture],
            draw_unit,
            draw_params,
        );
    }
}

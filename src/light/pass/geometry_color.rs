use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Uniform},
    program,
};

use crate::pass::{MatricesBlock, MATRICES_BLOCK_BINDING};

use super::{super::ObjectLightParams, OBJECT_LIGHT_PARAMS_BLOCK_BINDING};

program! {
    Program [
        (
            matrices: MatricesBlock = MATRICES_BLOCK_BINDING,
            object_params: ObjectLightParams = OBJECT_LIGHT_PARAMS_BLOCK_BINDING,
        ),
        (),
        (a: ColorVertex),
    ]
    => (VERTEX_SOURCE, FRAGMENT_SOURCE)
}

const VERTEX_SOURCE: &str = r#"
out vec4 v_color;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);

    v_color = vec4(pow(a_color.rgb, vec3(2.2)), a_color.a);
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec4 v_color;
layout (location = 0) out vec4 f_albedo;
layout (location = 1) out vec4 f_normal;
layout (location = 2) out vec4 f_occlusion;

void main() {
    f_albedo = v_color;
    f_normal = vec4(0.5, 0.5, 1.0, f_albedo.a);
    f_occlusion = vec4(object_params.occlusion, 0.0, 0.0, f_albedo.a);
}
"#;

pub struct GeometryColorPass {
    program: Program,
}

impl GeometryColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<MatricesBlock>,
        object_light_params: &Uniform<ObjectLightParams>,
        draw_unit: DrawUnit<ColorVertex, E>,
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            (matrices, object_light_params),
            [],
            draw_unit,
            draw_params,
        );
    }
}

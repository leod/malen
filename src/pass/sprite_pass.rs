use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Texture, Uniform},
    program,
};

use super::{MatricesBlock, MATRICES_BLOCK_BINDING};

program! {
    Program [
        (matrices: MatricesBlock = MATRICES_BLOCK_BINDING),
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
out vec4 f_color;

void main() {
    f_color = texture(sprite, v_uv) * v_color;
}
"#;

pub struct SpritePass {
    program: Program,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(gl)?;

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

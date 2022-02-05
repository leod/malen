use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Texture, Uniform},
    pass::{ViewMatrices, MATRICES_BLOCK_BINDING},
    program,
};

program! {
    Program [
        matrices: ViewMatrices = MATRICES_BLOCK_BINDING;
        sprite: Sampler2,
        screen_light: Sampler2;
        a: SpriteVertex,
    ]
    -> (VERTEX_SOURCE, FRAGMENT_SOURCE)
}

const VERTEX_SOURCE: &str = r#"
out vec2 v_sprite_uv;
out vec2 v_screen_uv;
out vec4 v_color;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);
    v_sprite_uv = a_tex_coords / vec2(textureSize(sprite, 0));
    v_screen_uv = vec2(position.xy) * 0.5 + 0.5;
    v_color = a_color;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_sprite_uv;
in vec2 v_screen_uv;
in vec4 v_color;
out vec4 f_color;

void main() {
    vec4 data = texture(sprite, v_sprite_uv);
    vec3 albedo = pow(data.rgb, vec3(2.2));
    vec3 light = texture(screen_light, v_screen_uv).rgb;
    f_color = vec4(v_color.rgb * albedo * light, v_color.a * data.a);
}
"#;

pub struct ShadedSpritePass {
    program: Program,
}

impl ShadedSpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<ViewMatrices>,
        texture: &Texture,
        screen_light: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            matrices,
            [texture, screen_light],
            draw_unit,
            draw_params,
        );
    }
}

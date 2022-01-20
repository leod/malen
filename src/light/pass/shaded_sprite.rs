use std::rc::Rc;

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

pub struct ShadedSpritePass {
    program: Program<MatricesBlock, SpriteVertex, 2>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("matrices", MATRICES_BLOCK_BINDING)];

const SAMPLERS: [&str; 2] = ["sprite", "screen_light"];

const VERTEX_SOURCE: &str = r#"
out vec2 v_sprite_uv;
out vec2 v_screen_uv;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);
    v_sprite_uv = a_tex_coords / vec2(textureSize(sprite, 0));
    v_screen_uv = vec2(position.xy) * 0.5 + 0.5;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_sprite_uv;
in vec2 v_screen_uv;
out vec4 f_color;

void main() {
    vec3 albedo = pow(texture(sprite, v_sprite_uv).rgb, vec3(2.2));
    vec3 light = texture(screen_light, v_screen_uv).rgb;
    f_color = vec4(albedo * light, 1.0);
}
"#;

impl ShadedSpritePass {
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

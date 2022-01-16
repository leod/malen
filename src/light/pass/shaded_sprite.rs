use std::{cell::RefCell, rc::Rc};

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
    pass::{
        MatricesBlock, SpriteInfoBlock, SpriteInfos, MATRICES_BLOCK_BINDING,
        SPRITE_INFO_BLOCK_BINDING,
    },
};

const VERTEX_SOURCE: &str = r#"
out vec2 v_sprite_uv;
out vec2 v_screen_uv;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);
    v_sprite_uv = a_tex_coords / sprite_info.size;
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

pub struct ShadedSpritePass {
    program: Program<(MatricesBlock, SpriteInfoBlock), SpriteVertex, 2>,
    sprite_infos: RefCell<SpriteInfos>,
}

impl ShadedSpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("sprite_info", SPRITE_INFO_BLOCK_BINDING),
            ],
            samplers: ["sprite", "screen_light"],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            program,
            sprite_infos: RefCell::new(SpriteInfos::new()),
        })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<MatricesBlock>,
        texture: &Texture,
        screen_light: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        let mut sprite_infos = self.sprite_infos.borrow_mut();
        let sprite_info = sprite_infos.get(texture)?;

        gl::draw(
            &self.program,
            (matrices, sprite_info),
            [texture, screen_light],
            draw_unit,
            &DrawParams::default(),
        );

        Ok(())
    }
}

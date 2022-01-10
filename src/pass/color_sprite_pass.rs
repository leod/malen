use std::{cell::RefCell, rc::Rc};

use crate::{
    data::ColorSpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, UniformBuffer},
};

use super::{
    sprite_info::{SpriteInfoBlock, SpriteInfos},
    MatricesBlock, MATRICES_BLOCK_BINDING, SPRITE_INFO_BLOCK_BINDING,
};

pub struct ColorSpritePass {
    program: Program<(MatricesBlock, SpriteInfoBlock), ColorSpriteVertex, 1>,
    sprite_infos: RefCell<SpriteInfos>,
}

impl ColorSpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("sprite_info", SPRITE_INFO_BLOCK_BINDING),
            ],
            samplers: ["sprite"],
            vertex_source: r#"
                out vec2 v_tex_coords;
                out vec4 v_color;

                void main() {
                    vec3 position = matrices.projection
                        * matrices.view
                        * vec3(a_position.xy, 1.0);

                    gl_Position = vec4(position.xy, a_position.z, 1.0);

                    v_tex_coords = a_tex_coords;
                    v_color = a_color;
                }
            "#,
            fragment_source: r#"
                in vec2 v_tex_coords;
                in vec4 v_color;
                out vec4 f_color;

                void main() {
                    vec2 uv = v_tex_coords / sprite_info.size;
                    f_color = texture(sprite, uv) * v_color;
                }
            "#,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            program,
            sprite_infos: RefCell::new(SpriteInfos::new()),
        })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        texture: &Texture,
        draw_unit: DrawUnit<ColorSpriteVertex, E>,
        params: &DrawParams,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        //#[cfg(feature = "coarse-prof")]
        //coarse_prof::profile!("ColorSpritePass::draw");

        let mut sprite_infos = self.sprite_infos.borrow_mut();
        let sprite_info = sprite_infos.get(texture)?;

        gl::draw(
            &self.program,
            (matrices, sprite_info),
            [texture],
            draw_unit,
            params,
        );

        Ok(())
    }
}

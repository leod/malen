use std::{cell::RefCell, rc::Rc};

use crate::{
    data::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
};

use super::{
    sprite_info::{SpriteInfoBlock, SpriteInfos},
    MatricesBlock, MATRICES_BLOCK_BINDING, SPRITE_INFO_BLOCK_BINDING,
};

pub struct SpritePass {
    program: Program<(MatricesBlock, SpriteInfoBlock), SpriteVertex, 1>,
    sprite_infos: RefCell<SpriteInfos>,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("sprite_info", SPRITE_INFO_BLOCK_BINDING),
            ],
            samplers: ["sprite"],
            vertex_source: r#"
                out vec2 v_uv;

                void main() {
                    vec3 position = matrices.projection
                        * matrices.view
                        * vec3(a_position.xy, 1.0);

                    gl_Position = vec4(position.xy, a_position.z, 1.0);

                    v_uv = a_tex_coords / sprite_info.size;
                }
            "#,
            fragment_source: r#"
                in vec2 v_uv;
                out vec4 f_color;

                void main() {
                    f_color = texture(sprite, v_uv);
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
        matrices: &Uniform<MatricesBlock>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        //#[cfg(feature = "coarse-prof")]
        //coarse_prof::profile!("SpritePass::draw");

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

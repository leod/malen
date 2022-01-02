use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::Vector2;

use crate::{
    geometry::SpriteVertex,
    gl::{
        self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, UniformBlock,
        UniformBuffer,
    },
};

use super::MatrixBlock;

pub struct SpritePass {
    program: Program<(MatrixBlock, SpriteInfoBlock), SpriteVertex, 1>,

    sprite_infos: RefCell<BTreeMap<glow::Texture, UniformBuffer<SpriteInfoBlock>>>,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            samplers: ["sprite"],
            vertex_source: r#"
                out vec2 v_tex_coords;

                void main() {
                    vec3 position = matrices.projection
                        * matrices.view
                        * vec3(a_position.xy, 1.0);

                    gl_Position = vec4(position.xy, a_position.z, 1.0);

                    v_tex_coords = a_tex_coords;
                }
            "#,
            fragment_source: r#"
                in vec2 v_tex_coords;
                out vec4 f_color;

                void main() {
                    vec2 uv = v_tex_coords / sprite_info.size;
                    f_color = texture(sprite, uv);
                }
            "#,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            program,
            sprite_infos: RefCell::new(BTreeMap::new()),
        })
    }

    pub fn draw<E>(
        &self,
        matrix_buffer: &UniformBuffer<MatrixBlock>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        if !self.sprite_infos.borrow().contains_key(&texture.id()) {
            // TODO: Fails if OpenGL reuses ids -- need to introduce our own unique IDs
            // TODO: Max size for sprite info cache
            let buffer = UniformBuffer::new(
                self.program.gl(),
                SpriteInfoBlock {
                    size: texture.size().cast::<f32>(),
                },
            )?;
            self.sprite_infos.borrow_mut().insert(texture.id(), buffer);
        }

        let sprite_infos_borrow = self.sprite_infos.borrow();
        let sprite_info = sprite_infos_borrow.get(&texture.id()).unwrap();

        gl::draw(
            &self.program,
            (matrix_buffer, sprite_info),
            [texture],
            draw_unit,
            params,
        );

        Ok(())
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
struct SpriteInfoBlock {
    size: Vector2<f32>,
}

impl UniformBlock for SpriteInfoBlock {
    const INSTANCE_NAME: &'static str = "sprite_info";
}

use std::rc::Rc;

use crate::{
    geometry::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, UniformBuffer},
};

use super::Matrices;

pub struct SpritePass {
    program: Program<Matrices, SpriteVertex, 1>,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            samplers: ["sprite"],
            vertex_source: r#"
                void main() {
                }
            "#,
            fragment_source: r#"
                void main() {
                }
            "#,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<Matrices>,
        texture: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, [texture], draw_unit, params);
    }
}

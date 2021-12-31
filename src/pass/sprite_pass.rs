use std::rc::Rc;

use crate::{
    geometry::SpriteVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, UniformBuffer},
};

use super::Matrices;

pub struct SpritePass {
    program: Program<Matrices, SpriteVertex>,
}

impl SpritePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(
            gl,
            r#"
            "#,
            r#"
            "#,
        )?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<Matrices>,
        draw_unit: DrawUnit<SpriteVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, draw_unit, params);
    }
}

use std::rc::Rc;

use crate::{
    geometry::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Program, UniformBuffer},
};

use super::Matrices;

pub struct ColorPass {
    program: Program<Matrices, ColorVertex>,
}

impl ColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = Program::new(
            gl,
            r#"
            out vec4 v_color;

            void main() {
                vec3 position = matrices.projection * matrices.view * vec3(a_position.xy, 1.0);
                gl_Position = vec4(position.xy, a_position.z, 1.0);

                v_color = a_color;
            }"#,
            r#"
            in vec4 v_color;
            out vec4 f_color;

            void main() {
                f_color = v_color;
            }
            "#,
        )?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<Matrices>,
        draw_unit: DrawUnit<ColorVertex, E>,
        params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(&self.program, matrices, draw_unit, params);
    }
}

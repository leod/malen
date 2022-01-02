use std::rc::Rc;

use glow::HasContext;

use super::{
    draw_params::set_draw_params, uniform_block::UniformBuffers, DrawParams, DrawUnit, Element,
    Program, Texture, Vertex,
};

pub fn draw<U, V, E, const S: usize>(
    program: &Program<U::UniformBlocks, V, S>,
    uniforms: U,
    samplers: [&Texture; S],
    draw_unit: DrawUnit<V, E>,
    draw_params: &DrawParams,
) where
    U: UniformBuffers,
    V: Vertex,
    E: Element,
{
    assert!(Rc::ptr_eq(&program.gl(), &draw_unit.gl()));

    let gl = program.gl();

    set_draw_params(&*gl, draw_params);

    program.bind();
    uniforms.bind();
    draw_unit.bind();

    let range = draw_unit.element_range();

    let mode = draw_unit.primitive_mode().to_gl();
    let count = range.end - range.start;
    let element_type = E::to_gl();
    let offset = range.start * std::mem::size_of::<E>();

    unsafe {
        gl.draw_elements(mode, count as i32, element_type, offset as i32);
    }
}

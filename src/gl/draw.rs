use std::rc::Rc;

use glow::HasContext;

use super::{draw_params::set_draw_params, DrawParams, DrawUnit, Element, Program, Vertex};

pub fn draw<V, E>(program: &Program<V>, draw_unit: &DrawUnit<V, E>, draw_params: &DrawParams)
where
    V: Vertex,
    E: Element,
{
    assert!(Rc::ptr_eq(&program.gl(), &draw_unit.gl()));

    let gl = program.gl();

    set_draw_params(&*gl, draw_params);

    program.bind();
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

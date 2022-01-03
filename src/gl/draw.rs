use std::rc::Rc;

use glow::HasContext;

use super::{
    draw_params::set_draw_params, uniform_block::UniformBuffers, Context, DrawParams, DrawUnit,
    Element, Program, Texture, Vertex,
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

    let range = draw_unit.element_range();
    if range.start == range.end {
        return;
    }

    let gl = program.gl();

    set_draw_params(&*gl, draw_params);

    program.bind();
    uniforms.bind();
    bind_samplers(gl.clone(), samplers);
    draw_unit.bind();

    let mode = draw_unit.primitive_mode().to_gl();
    let count = range.end - range.start;
    let element_type = E::to_gl();
    let offset = range.start * std::mem::size_of::<E>();

    unsafe {
        gl.draw_elements(mode, count as i32, element_type, offset as i32);

        // It is important to unbind the vertex array here, so that buffer
        // bindings later on (e.g. for mutating buffer contents) do not change
        // the vertex array bindings.
        gl.bind_vertex_array(None);
    }
}

fn bind_samplers<const S: usize>(gl: Rc<Context>, samplers: [&Texture; S]) {
    for (i, sampler) in samplers.iter().enumerate() {
        assert!(Rc::ptr_eq(&sampler.gl(), &gl));

        unsafe {
            gl.active_texture(glow::TEXTURE0 + i as u32);
            gl.bind_texture(glow::TEXTURE_2D, Some(sampler.id()));
        }
    }

    unsafe {
        gl.active_texture(glow::TEXTURE0);
    }
}

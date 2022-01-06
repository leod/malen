use std::rc::Rc;

use glow::HasContext;

use super::{
    draw_params::set_draw_params, uniform_block::UniformBuffers, Context, DrawParams, DrawUnit,
    Element, InstancedDrawUnit, Program, Texture, Vertex, VertexDecls,
};

pub fn draw<U, V, E, const S: usize>(
    program: &Program<U::UniformBlockDecls, V, S>,
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
    uniforms.bind(program.uniform_block_bindings());
    bind_samplers(gl.clone(), samplers);
    draw_unit.bind();

    let count = range.end - range.start;
    let offset = range.start * std::mem::size_of::<E>();

    unsafe {
        gl.draw_elements(
            draw_unit.primitive_mode().to_gl(),
            i32::try_from(count).unwrap(),
            E::to_gl(),
            i32::try_from(offset).unwrap(),
        );

        // It is important to unbind the vertex array here, so that buffer
        // bindings later on (e.g. for mutating buffer contents) do not change
        // the vertex array bindings.
        gl.bind_vertex_array(None);
    }
}

pub fn draw_instanced<U, V, E, const S: usize>(
    program: &Program<U::UniformBlockDecls, V, S>,
    uniforms: U,
    samplers: [&Texture; S],
    draw_unit: InstancedDrawUnit<V, E>,
    draw_params: &DrawParams,
) where
    U: UniformBuffers,
    V: VertexDecls,
    E: Element,
{
    assert!(Rc::ptr_eq(&program.gl(), &draw_unit.gl()));

    let range = draw_unit.element_range();
    if range.start == range.end || draw_unit.num_instances() == 0 {
        return;
    }

    let gl = program.gl();

    set_draw_params(&*gl, draw_params);

    program.bind();
    uniforms.bind(program.uniform_block_bindings());
    bind_samplers(gl.clone(), samplers);
    draw_unit.bind();

    let count = range.end - range.start;
    let offset = range.start * std::mem::size_of::<E>();

    unsafe {
        gl.draw_elements_instanced(
            draw_unit.primitive_mode().to_gl(),
            i32::try_from(count).unwrap(),
            E::to_gl(),
            i32::try_from(offset).unwrap(),
            i32::try_from(draw_unit.num_instances()).unwrap(),
        );

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

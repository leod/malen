use std::rc::Rc;

use glow::HasContext;

use crate::Color4;

use super::{
    draw_params::set_draw_params, uniform_block::UniformBuffers, Context, DrawParams, DrawUnit,
    Element, Framebuffer, InstancedDrawUnit, Program, Texture, Vertex, VertexDecls,
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

    // FIXME: We need to re-verify the element range here, since the buffers
    //        references by the DrawUnit could have changed since its creation.

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

    // FIXME: We need to re-verify the element range here, since the buffers
    //        references by the DrawUnit could have changed since its creation.

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

pub fn with_framebuffer<F, R>(framebuffer: &Framebuffer, f: F) -> R
where
    F: FnOnce() -> R,
{
    let gl = framebuffer.gl();

    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer.id()));
        gl.viewport(
            0,
            0,
            i32::try_from(framebuffer.textures()[0].size().x).unwrap(),
            i32::try_from(framebuffer.textures()[0].size().y).unwrap(),
        );
    }

    let result = f();

    // TODO: We should be able to reduce state changes by delaying the unbind.
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.viewport(
            gl.main_viewport.get()[0],
            gl.main_viewport.get()[1],
            gl.main_viewport.get()[2],
            gl.main_viewport.get()[3],
        );
    }

    result
}

pub fn clear_color_and_depth(gl: &Context, color: Color4, depth: f32) {
    unsafe {
        gl.color_mask(true, true, true, true);
        gl.depth_mask(true);
        gl.clear_color(color.r, color.g, color.b, color.a);
        gl.clear_depth_f32(depth);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }
}

pub fn clear_color(gl: &Context, color: Color4) {
    unsafe {
        gl.color_mask(true, true, true, true);
        gl.clear_color(color.r, color.g, color.b, color.a);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}

pub fn clear_depth(gl: &Context, depth: f32) {
    unsafe {
        gl.depth_mask(true);
        gl.clear_depth_f32(depth);
        gl.clear(glow::DEPTH_BUFFER_BIT);
    }
}

fn bind_samplers<const S: usize>(gl: Rc<Context>, samplers: [&Texture; S]) {
    // FIXME: We need to verify that we are not sampling from the current
    //        framebuffer.

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

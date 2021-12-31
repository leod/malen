use std::rc::Rc;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use glow::HasContext;

use super::{Context, UniformBuffer};

pub trait UniformBlock: AsStd140 + GlslStruct {
    const INSTANCE_NAME: &'static str;
}

pub trait UniformBuffers {
    type UniformBlocks: UniformBlocks;

    fn bind(&self);
}

pub trait UniformBlocks {
    const NUM_BLOCKS: usize;

    fn glsl_definitions() -> String;
    fn bind_to_program(gl: &Context, program: <glow::Context as HasContext>::Program);
}

impl UniformBlocks for () {
    const NUM_BLOCKS: usize = 0;

    fn glsl_definitions() -> String {
        String::new()
    }

    fn bind_to_program(_gl: &Context, _program: <glow::Context as HasContext>::Program) {}
}

impl<U> UniformBlocks for U
where
    U: UniformBlock,
{
    const NUM_BLOCKS: usize = 1;

    fn glsl_definitions() -> String {
        glsl_definition::<U>()
    }

    fn bind_to_program(gl: &Context, program: <glow::Context as HasContext>::Program) {
        unsafe {
            gl.uniform_block_binding(
                program,
                gl.get_uniform_block_index(program, U::NAME).unwrap(),
                0,
            );
        }
    }
}

impl<U1, U2> UniformBlocks for (U1, U2)
where
    U1: UniformBlock,
    U2: UniformBlock,
{
    const NUM_BLOCKS: usize = 2;

    fn glsl_definitions() -> String {
        U1::glsl_definitions() + &glsl_definition::<U2>()
    }

    fn bind_to_program(gl: &Context, program: <glow::Context as HasContext>::Program) {
        unsafe {
            U1::bind_to_program(gl, program);
            gl.uniform_block_binding(
                program,
                gl.get_uniform_block_index(program, U2::INSTANCE_NAME)
                    .unwrap(),
                1,
            );
        }
    }
}

impl<U1, U2, U3> UniformBlocks for (U1, U2, U3)
where
    U1: UniformBlock,
    U2: UniformBlock,
    U3: UniformBlock,
{
    const NUM_BLOCKS: usize = 2;

    fn glsl_definitions() -> String {
        <(U1, U2)>::glsl_definitions() + &glsl_definition::<U3>()
    }

    fn bind_to_program(gl: &Context, program: <glow::Context as HasContext>::Program) {
        unsafe {
            <(U1, U2)>::bind_to_program(gl, program);
            gl.uniform_block_binding(
                program,
                gl.get_uniform_block_index(program, U2::INSTANCE_NAME)
                    .unwrap(),
                2,
            );
        }
    }
}

impl UniformBuffers for () {
    type UniformBlocks = ();

    fn bind(&self) {}
}

impl<'a, U> UniformBuffers for &'a UniformBuffer<U>
where
    U: UniformBlock,
{
    type UniformBlocks = U;

    fn bind(&self) {
        unsafe {
            self.gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(self.buffer));
        }
    }
}

impl<'a, U1, U2> UniformBuffers for (&'a UniformBuffer<U1>, &'a UniformBuffer<U2>)
where
    U1: UniformBlock,
    U2: UniformBlock,
{
    type UniformBlocks = (U1, U2);

    fn bind(&self) {
        assert!(Rc::ptr_eq(&self.0.gl(), &self.1.gl()));

        unsafe {
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(self.0.buffer));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 1, Some(self.1.buffer));
        }
    }
}

impl<'a, U1, U2, U3> UniformBuffers
    for (
        &'a UniformBuffer<U1>,
        &'a UniformBuffer<U2>,
        &'a UniformBuffer<U3>,
    )
where
    U1: UniformBlock,
    U2: UniformBlock,
    U3: UniformBlock,
{
    type UniformBlocks = (U1, U2, U3);

    fn bind(&self) {
        assert!(Rc::ptr_eq(&self.0.gl(), &self.1.gl()));
        assert!(Rc::ptr_eq(&self.0.gl(), &self.2.gl()));

        unsafe {
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(self.0.buffer));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 1, Some(self.1.buffer));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, 2, Some(self.2.buffer));
        }
    }
}

fn glsl_definition<U: UniformBlock>() -> String {
    let mut output = String::new();

    output.push_str("uniform ");
    output.push_str(U::NAME);
    output.push_str(" {\n");

    for field in U::FIELDS {
        output.push('\t');
        output.push_str(field.ty);
        output.push(' ');
        output.push_str(field.name);
        output.push_str(";\n");
    }

    output.push_str("} ");
    output.push_str(U::INSTANCE_NAME);
    output.push_str(";\n");

    output
}

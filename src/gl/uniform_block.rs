use std::rc::Rc;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use glow::HasContext;

use super::{Context, UniformBuffer};

pub trait UniformBlock: AsStd140 + GlslStruct {}

pub trait UniformBuffers {
    type UniformBlocks: UniformBlocks;

    fn bind(&self, bindings: &[u32]);
}

pub trait UniformBlocks {
    const N: usize;

    fn glsl_definitions(instance_names: &[&str]) -> String;
    fn bind_to_program(gl: &Context, id: glow::Program, uniform_blocks: &[(&str, u32)]);
}

impl UniformBlocks for () {
    const N: usize = 0;

    fn glsl_definitions(_: &[&str]) -> String {
        String::new()
    }

    fn bind_to_program(_: &Context, _: glow::Program, _: &[(&str, u32)]) {}
}

impl<U> UniformBlocks for U
where
    U: UniformBlock,
{
    const N: usize = 1;

    fn glsl_definitions(instance_names: &[&str]) -> String {
        let mut output = String::new();

        output.push_str("uniform ");
        output.push_str(&block_name(instance_names[0]));
        output.push_str(" {\n");

        for field in U::FIELDS {
            output.push('\t');
            output.push_str(field.ty);
            output.push(' ');
            output.push_str(field.name);
            output.push_str(";\n");
        }

        output.push_str("} ");
        output.push_str(instance_names[0]);
        output.push_str(";\n");

        output
    }

    fn bind_to_program(gl: &Context, id: glow::Program, uniform_blocks: &[(&str, u32)]) {
        if let Some(index) =
            unsafe { gl.get_uniform_block_index(id, &block_name(uniform_blocks[0].0)) }
        {
            unsafe {
                gl.uniform_block_binding(id, index, uniform_blocks[0].1);
            }
        } else {
            log::info!("Uniform block `{}` is unused", uniform_blocks[0].0);
        }
    }
}

impl<U0, U1> UniformBlocks for (U0, U1)
where
    U0: UniformBlock,
    U1: UniformBlock,
{
    const N: usize = 2;

    fn glsl_definitions(instance_names: &[&str]) -> String {
        U0::glsl_definitions(&[instance_names[0]]) + &U1::glsl_definitions(&[instance_names[1]])
    }

    fn bind_to_program(gl: &Context, id: glow::Program, uniform_blocks: &[(&str, u32)]) {
        U0::bind_to_program(gl, id, &[uniform_blocks[0]]);
        U1::bind_to_program(gl, id, &[uniform_blocks[1]]);
    }
}

impl<U0, U1, U2> UniformBlocks for (U0, U1, U2)
where
    U0: UniformBlock,
    U1: UniformBlock,
    U2: UniformBlock,
{
    const N: usize = 3;

    fn glsl_definitions(instance_names: &[&str]) -> String {
        U0::glsl_definitions(&[instance_names[0]])
            + &U1::glsl_definitions(&[instance_names[1]])
            + &U2::glsl_definitions(&[instance_names[2]])
    }

    fn bind_to_program(gl: &Context, id: glow::Program, uniform_blocks: &[(&str, u32)]) {
        U0::bind_to_program(gl, id, &[uniform_blocks[0]]);
        U1::bind_to_program(gl, id, &[uniform_blocks[1]]);
        U2::bind_to_program(gl, id, &[uniform_blocks[2]]);
    }
}

impl UniformBuffers for () {
    type UniformBlocks = ();

    fn bind(&self, bindings: &[u32]) {}
}

impl<'a, U> UniformBuffers for &'a UniformBuffer<U>
where
    U: UniformBlock,
{
    type UniformBlocks = U;

    fn bind(&self, bindings: &[u32]) {
        unsafe {
            self.gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[0], Some(self.id()));
        }
    }
}

impl<'a, U0, U1> UniformBuffers for (&'a UniformBuffer<U0>, &'a UniformBuffer<U1>)
where
    U0: UniformBlock,
    U1: UniformBlock,
{
    type UniformBlocks = (U0, U1);

    fn bind(&self, bindings: &[u32]) {
        assert!(Rc::ptr_eq(&self.0.gl(), &self.1.gl()));

        unsafe {
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[0], Some(self.0.id()));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[1], Some(self.1.id()));
        }
    }
}

impl<'a, U0, U1, U2> UniformBuffers
    for (
        &'a UniformBuffer<U0>,
        &'a UniformBuffer<U1>,
        &'a UniformBuffer<U2>,
    )
where
    U0: UniformBlock,
    U1: UniformBlock,
    U2: UniformBlock,
{
    type UniformBlocks = (U0, U1, U2);

    fn bind(&self, bindings: &[u32]) {
        assert!(Rc::ptr_eq(&self.0.gl(), &self.1.gl()));
        assert!(Rc::ptr_eq(&self.0.gl(), &self.2.gl()));

        unsafe {
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[0], Some(self.0.id()));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[1], Some(self.1.id()));
            self.0
                .gl()
                .bind_buffer_base(glow::UNIFORM_BUFFER, bindings[2], Some(self.2.id()));
        }
    }
}

fn block_name(instance_name: &str) -> String {
    format!("_{}_", instance_name)
}

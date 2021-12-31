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

impl<U> UniformBuffers for UniformBuffer<U>
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

impl<U1, U2> UniformBuffers for (UniformBuffer<U1>, UniformBuffer<U2>)
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

impl<U1, U2, U3> UniformBuffers for (UniformBuffer<U1>, UniformBuffer<U2>, UniformBuffer<U3>)
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

pub trait UniformBlocks {
    const NUM_BLOCKS: usize;

    type UniformBuffers: UniformBuffers;

    fn glsl_definitions() -> String;
}

impl<U> UniformBlocks for U
where
    U: UniformBlock,
{
    const NUM_BLOCKS: usize = 1;

    type UniformBuffers = UniformBuffer<U>;

    fn glsl_definitions() -> String {
        glsl_definition::<U>()
    }
}

impl<U1, U2> UniformBlocks for (U1, U2)
where
    U1: UniformBlock,
    U2: UniformBlock,
{
    const NUM_BLOCKS: usize = 2;

    type UniformBuffers = (UniformBuffer<U1>, UniformBuffer<U2>);

    fn glsl_definitions() -> String {
        glsl_definition::<U1>() + &glsl_definition::<U2>()
    }
}

impl<U1, U2, U3> UniformBlocks for (U1, U2, U3)
where
    U1: UniformBlock,
    U2: UniformBlock,
    U3: UniformBlock,
{
    const NUM_BLOCKS: usize = 2;

    type UniformBuffers = (UniformBuffer<U1>, UniformBuffer<U2>, UniformBuffer<U3>);

    fn glsl_definitions() -> String {
        glsl_definition::<U1>() + &glsl_definition::<U2>() + &glsl_definition::<U3>()
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

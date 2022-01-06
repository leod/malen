use std::{ops::Range, rc::Rc};

use super::{Context, ElementBuffer, VertexArray, VertexBuffer, VertexDecls};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    Triangle,
    Line,
}

pub struct DrawUnit<'a, V, E>
where
    V: VertexDecls,
{
    vertex_array: &'a VertexArray<V, E>,
    primitive_mode: PrimitiveMode,
    element_range: Range<usize>,
}

impl<'a, V, E> DrawUnit<'a, V, E>
where
    V: VertexDecls,
{
    pub fn new(
        vertex_array: &'a VertexArray<V, E>,
        primitive_mode: PrimitiveMode,
        element_range: Range<usize>,
    ) -> Self {
        assert!(element_range.start <= element_range.end);
        assert!(element_range.end <= vertex_array.element_buffer().len());

        Self {
            vertex_array,
            primitive_mode,
            element_range,
        }
    }

    pub fn gl(&self) -> Rc<Context> {
        self.vertex_array.gl()
    }

    pub fn vertex_array(&self) -> &'a VertexArray<V, E> {
        self.vertex_array
    }

    pub fn vertex_buffers(&self) -> V::RcVertexBufferTuple {
        self.vertex_array.vertex_buffers()
    }

    pub fn element_buffer(&self) -> Rc<ElementBuffer<E>> {
        self.vertex_array.element_buffer()
    }

    pub fn primitive_mode(&self) -> PrimitiveMode {
        self.primitive_mode
    }

    pub fn element_range(&self) -> Range<usize> {
        self.element_range.clone()
    }

    pub fn bind(&self) {
        self.vertex_array.bind();
    }
}

impl PrimitiveMode {
    pub fn to_gl(self) -> u32 {
        use PrimitiveMode::*;

        match self {
            Line => glow::LINES,
            Triangle => glow::TRIANGLES,
        }
    }
}

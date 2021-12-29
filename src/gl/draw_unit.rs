use std::{rc::Rc, ops::Range};

use super::{Context, ElementBuffer, VertexBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    Triangle,
    Line,
}

pub struct DrawUnit<'a, V, E> {
    vertex_buffer: &'a VertexBuffer<V>,
    element_buffer: &'a ElementBuffer<E>,
    primitive_mode: PrimitiveMode,
    element_range: Range<usize>,
}

impl<'a, V, E> DrawUnit<'a, V, E> {
    pub fn new(
        vertex_buffer: &'a VertexBuffer<V>,
        element_buffer: &'a ElementBuffer<E>,
        primitive_mode: PrimitiveMode,
        element_range: Range<usize>,
    ) -> Self {
        assert!(Rc::ptr_eq(&vertex_buffer.gl(), &element_buffer.gl()));
        assert!(element_range.start <= element_range.end);
        assert!(element_range.end <= element_buffer.len());

        Self {
            vertex_buffer,
            element_buffer,
            primitive_mode,
            element_range,
        }
    }

    pub fn gl(&self) -> Rc<Context> {
        self.vertex_buffer.gl()
    }

    pub fn vertex_buffer(&self) -> &'a VertexBuffer<V> {
        self.vertex_buffer
    }

    pub fn element_buffer(&self) -> &'a ElementBuffer<E> {
        self.element_buffer
    }

    pub fn primitive_mode(&self) -> PrimitiveMode {
        self.primitive_mode
    }

    pub fn element_range(&self) -> Range<usize> {
        self.element_range
    }

    pub fn bind(&self) {
        self.vertex_buffer.bind();
        self.element_buffer.bind();
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
use std::{marker::PhantomData, ops::Range};

use golem::{ElementBuffer, GeometryMode, ShaderProgram, VertexBuffer};

use crate::{Context, Error};

pub struct Buffers<V> {
    pub(crate) vertices: VertexBuffer,
    pub(crate) elements: ElementBuffer,
    pub(crate) num_elements: usize,
    _phantom: PhantomData<V>,
}

pub struct BuffersSlice<'a, V> {
    pub(crate) vertices: &'a VertexBuffer,
    pub(crate) elements: &'a ElementBuffer,
    pub(crate) first_element: usize,
    pub(crate) num_elements: usize,
    _phantom: PhantomData<V>,
}

pub trait AsBuffersSlice<V> {
    fn as_buffers_slice<'a>(&'a self) -> BuffersSlice<'a, V>;
}

impl<V> Buffers<V> {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let vertices = VertexBuffer::new(ctx.golem_context())?;
        let elements = ElementBuffer::new(ctx.golem_context())?;

        Ok(Self {
            vertices,
            elements,
            num_elements: 0,
            _phantom: PhantomData,
        })
    }

    pub fn from_buffers_unchecked(
        vertices: VertexBuffer,
        elements: ElementBuffer,
        num_elements: usize,
    ) -> Self {
        Self {
            vertices,
            elements,
            num_elements,
            _phantom: PhantomData,
        }
    }

    pub fn num_elements(&self) -> usize {
        self.num_elements
    }

    pub fn slice_elements(&self, range: Range<usize>) -> BuffersSlice<V> {
        assert!(range.start < range.end);
        assert!(range.end < self.num_elements);

        BuffersSlice {
            vertices: &self.vertices,
            elements: &self.elements,
            first_element: range.start,
            num_elements: range.end - range.start,
            _phantom: PhantomData,
        }
    }
}

impl<V> AsBuffersSlice<V> for Buffers<V> {
    fn as_buffers_slice<'a>(&'a self) -> BuffersSlice<'a, V> {
        BuffersSlice {
            vertices: &self.vertices,
            elements: &self.elements,
            first_element: 0,
            num_elements: self.num_elements,
            _phantom: PhantomData,
        }
    }
}

impl<'a, V> BuffersSlice<'a, V> {
    pub fn from_buffers_unchecked(
        vertices: &'a VertexBuffer,
        elements: &'a ElementBuffer,
        num_elements: usize,
    ) -> Self {
        Self {
            vertices,
            elements,
            first_element: 0,
            num_elements,
            _phantom: PhantomData,
        }
    }

    pub fn num_elements(&self) -> usize {
        self.num_elements
    }

    pub unsafe fn draw(
        &self,
        shader: &ShaderProgram,
        geometry_mode: GeometryMode,
    ) -> Result<(), Error> {
        Ok(shader.draw(
            self.vertices,
            self.elements,
            self.first_element..self.num_elements,
            geometry_mode,
        )?)
    }
}

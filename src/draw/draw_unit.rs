use std::marker::PhantomData;

use golem::{ElementBuffer, GeometryMode, ShaderProgram, VertexBuffer};

use crate::Error;

pub struct DrawUnit<'a, V> {
    vertices: &'a VertexBuffer,
    elements: &'a ElementBuffer,
    first_element: usize,
    num_elements: usize,
    geometry_mode: GeometryMode,
    _phantom: PhantomData<V>,
}

impl<'a, V> DrawUnit<'a, V> {
    pub unsafe fn from_buffers_unchecked(
        vertices: &'a VertexBuffer,
        elements: &'a ElementBuffer,
        first_element: usize,
        num_elements: usize,
        geometry_mode: GeometryMode,
    ) -> Self {
        Self {
            vertices,
            elements,
            first_element,
            num_elements,
            geometry_mode,
            _phantom: PhantomData,
        }
    }

    pub fn draw(&self, shader: &ShaderProgram) -> Result<(), Error> {
        // TODO: I believe this is safe, because Batch in its construction
        // makes sure that each element points to a valid index in the vertex
        // buffer. We need to verify this though. We also need to verify if
        // golem::ShaderProgram::draw has any additional requirements for
        // safety.
        Ok(unsafe {
            shader.draw(
                self.vertices,
                self.elements,
                self.first_element..self.num_elements,
                self.geometry_mode,
            )
        }?)
    }

    pub fn vertices(&self) -> &'a VertexBuffer {
        self.vertices
    }

    pub fn elements(&self) -> &'a ElementBuffer {
        self.elements
    }

    pub fn first_element(&self) -> usize {
        self.first_element
    }

    pub fn num_elements(&self) -> usize {
        self.num_elements
    }

    pub fn geometry_mode(&self) -> GeometryMode {
        self.geometry_mode
    }
}

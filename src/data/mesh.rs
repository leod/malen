use std::{ops::Range, rc::Rc};

use crate::gl::{
    self, DrawUnit, ElementBuffer, PrimitiveMode, Vertex, VertexArray, VertexBuffer, VertexDecls,
};

use super::{Geometry, GeometryBatch, PrimitiveTag};

pub struct Mesh<V>
where
    V: Vertex,
{
    vertex_array: Rc<VertexArray<V>>,
    primitive_mode: PrimitiveMode,
    element_range: Range<usize>,
}

impl<V> Mesh<V>
where
    V: Vertex,
{
    pub fn new(
        vertex_array: Rc<VertexArray<V>>,
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

    pub fn from_geometry<P, G>(gl: Rc<gl::Context>, geometry: G) -> Result<Self, gl::Error>
    where
        P: PrimitiveTag,
        G: Geometry<P, Vertex = V>,
    {
        let mut batch = GeometryBatch::<P, V>::new(gl)?;
        batch.push(geometry);

        Ok(batch.into_mesh())
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.vertex_array.gl()
    }

    pub fn vertex_array(&self) -> Rc<VertexArray<V>> {
        self.vertex_array.clone()
    }

    pub fn vertex_buffer(&self) -> Rc<VertexBuffer<V>> {
        self.vertex_array.vertex_buffers()
    }

    pub fn element_buffer(&self) -> Rc<ElementBuffer> {
        self.vertex_array.element_buffer()
    }

    pub fn primitive_mode(&self) -> PrimitiveMode {
        self.primitive_mode
    }

    pub fn element_range(&self) -> Range<usize> {
        self.element_range.clone()
    }

    pub fn draw_unit(&self) -> DrawUnit<V, u32> {
        DrawUnit::new(
            &*self.vertex_array,
            self.primitive_mode,
            self.element_range.clone(),
        )
    }
}

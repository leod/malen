use std::marker::PhantomData;

use crate::gl::{ElementBuffer, PrimitiveMode, Vertex, VertexBuffer};

use super::{Geometry, PrimitiveTag};

#[derive(Debug, Clone, Default)]
pub struct GeometryBuffer<P, V> {
    elements: Vec<u32>,
    vertices: Vec<V>,
    _phantom: PhantomData<P>,
}

impl<P, V> GeometryBuffer<P, V> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            vertices: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.vertices.clear();
    }
}

impl<P, V> GeometryBuffer<P, V>
where
    P: PrimitiveTag,
{
    pub fn primitive_mode() -> PrimitiveMode {
        P::primitive_mode()
    }
}

impl<P, V> GeometryBuffer<P, V>
where
    P: PrimitiveTag,
    V: Vertex,
{
    pub fn push<G: Geometry<P, Vertex = V>>(&mut self, geometry: G) {
        geometry.write(&mut self.elements, &mut self.vertices);
    }
}

impl<P, V> GeometryBuffer<P, V>
where
    V: Vertex,
{
    pub fn upload(&mut self, element_buffer: &ElementBuffer<u32>, vertex_buffer: &VertexBuffer<V>) {
        /*#[cfg(feature = "coarse-prof")]
        coarse_prof::profile_string_name!(format!(
            "<{}, {}>::upload",
            std::any::type_name::<P>().split("::").last().unwrap(),
            std::any::type_name::<V>().split("::").last().unwrap(),
        ));*/

        element_buffer.set_data(&self.elements);
        vertex_buffer.set_data(&self.vertices);
    }
}

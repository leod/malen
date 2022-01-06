use std::marker::PhantomData;

use crate::gl::{ElementBuffer, PrimitiveMode, Vertex, VertexBuffer};

use super::{Geometry, PrimitiveTag};

#[derive(Debug, Clone, Default)]
pub struct GeometryBuffer<P, V> {
    vertices: Vec<V>,
    elements: Vec<u32>,
    _phantom: PhantomData<P>,
}

impl<P, V> GeometryBuffer<P, V> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            elements: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.elements.clear();
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
        geometry.write(&mut self.vertices, &mut self.elements);
    }
}

impl<P, V> GeometryBuffer<P, V>
where
    V: Vertex,
{
    pub fn upload(&mut self, vertex_buffer: &VertexBuffer<V>, element_buffer: &ElementBuffer<u32>) {
        #[cfg(feature = "coarse-prof")]
        coarse_prof::profile_string_name!(format!(
            "GeometryBuffer<{}, {}>::upload()",
            std::any::type_name::<P>().split("::").last().unwrap(),
            std::any::type_name::<V>().split("::").last().unwrap(),
        ));

        vertex_buffer.set_data(&self.vertices);
        element_buffer.set_data(&self.elements);
    }
}

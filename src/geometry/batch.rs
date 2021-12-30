use crate::gl::Vertex;

use super::{Geometry, GeometryBuffer, LineTag, PrimitiveTag, TriangleTag};

pub struct GeometryBatch<P, V> {
    buffer: GeometryBuffer<P, V>,
}

pub type TriangleBatch<V> = GeometryBatch<TriangleTag, V>;
pub type LineBatch<V> = GeometryBatch<LineTag, V>;

impl<P, V> GeometryBatch<P, V>
where
    P: PrimitiveTag,
    V: Vertex,
{
    pub fn push<G: Geometry<P, Vertex = V>>(&mut self, geometry: G) {
        self.buffer.push(geometry);
    }
}

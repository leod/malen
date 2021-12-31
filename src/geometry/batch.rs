use std::rc::Rc;

use crate::gl::{self, DrawUnit, ElementBuffer, Vertex, VertexArray, VertexBuffer};

use super::{
    ColorSpriteVertex, ColorVertex, Geometry, GeometryBuffer, LineTag, PrimitiveTag, SpriteVertex,
    TriangleTag,
};

pub struct GeometryBatch<P, V> {
    buffer: GeometryBuffer<P, V>,
    vertex_array: VertexArray<V, u32>,
    dirty: bool,
}

pub type TriangleBatch<V> = GeometryBatch<TriangleTag, V>;
pub type LineBatch<V> = GeometryBatch<LineTag, V>;

pub type SpriteBatch = TriangleBatch<SpriteVertex>;
pub type ColorSpriteBatch = TriangleBatch<ColorSpriteVertex>;
pub type ColorTriangleBatch = TriangleBatch<ColorVertex>;
pub type ColorLineBatch = LineBatch<ColorVertex>;

impl<P, V> GeometryBatch<P, V>
where
    V: Vertex,
{
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let vertex_buffer = VertexBuffer::new_dynamic(gl.clone())?;
        let element_buffer = ElementBuffer::new_dynamic(gl)?;
        let vertex_array = VertexArray::new(Rc::new(vertex_buffer), Rc::new(element_buffer))?;

        Ok(GeometryBatch {
            buffer: GeometryBuffer::new(),
            vertex_array,
            dirty: false,
        })
    }
}

impl<P, V> GeometryBatch<P, V>
where
    P: PrimitiveTag,
    V: Vertex,
{
    pub fn push<G: Geometry<P, Vertex = V>>(&mut self, geometry: G) {
        self.buffer.push(geometry);
        self.dirty = true;
    }

    pub fn draw_unit(&mut self) -> DrawUnit<V, u32> {
        if self.dirty {
            self.buffer.upload(
                &*self.vertex_array.vertex_buffer(),
                &*self.vertex_array.element_buffer(),
            );
            self.dirty = false;
        }

        DrawUnit::new(
            &self.vertex_array,
            P::primitive_mode(),
            0..self.vertex_array.element_buffer().len(),
        )
    }
}

impl<P, V> GeometryBatch<P, V> {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.dirty = true;
    }
}

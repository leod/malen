use std::rc::Rc;

use crate::gl::{
    self, DrawUnit, ElementBuffer, InstancedDrawUnit, Vertex, VertexArray, VertexBuffer,
};

use super::{
    ColorSpriteVertex, ColorVertex, Geometry, GeometryBuffer, LineTag, Mesh, PrimitiveTag,
    SpriteVertex, TriangleTag,
};

pub struct GeometryBatch<P, V>
where
    V: Vertex,
{
    buffer: GeometryBuffer<P, V>,
    vertex_array: VertexArray<V>,
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
        let element_buffer = ElementBuffer::new(gl.clone())?;
        let vertex_buffer = VertexBuffer::new(gl)?;
        let vertex_array = VertexArray::new(Rc::new(element_buffer), Rc::new(vertex_buffer))?;

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

    pub fn flush(&mut self) {
        if self.dirty {
            self.buffer.upload(
                &*self.vertex_array.element_buffer(),
                &*self.vertex_array.vertex_buffers(),
            );
            self.dirty = false;
        }
    }

    pub fn reset<G, I>(&mut self, iter: I)
    where
        G: Geometry<P, Vertex = V>,
        I: IntoIterator<Item = G>,
    {
        self.clear();
        self.extend(iter);
    }

    pub fn into_mesh(mut self) -> Mesh<V> {
        self.flush();
        let element_range = 0..self.vertex_array.element_buffer().len();
        Mesh::new(
            Rc::new(self.vertex_array),
            P::primitive_mode(),
            element_range,
        )
    }

    pub fn draw_unit(&mut self) -> DrawUnit<V> {
        self.flush();
        DrawUnit::new(
            &self.vertex_array,
            P::primitive_mode(),
            0..self.vertex_array.element_buffer().len(),
        )
    }
}

impl<G, P, V> Extend<G> for GeometryBatch<P, V>
where
    P: PrimitiveTag,
    V: Vertex,
    G: Geometry<P, Vertex = V>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = G>,
    {
        for geometry in iter.into_iter() {
            self.push(geometry);
        }
    }
}

impl<P, V> GeometryBatch<P, V>
where
    V: Vertex,
{
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.dirty = true;
    }
}

pub struct InstanceBatch<V, I>
where
    V: Vertex,
    I: Vertex,
{
    mesh: Mesh<V>,
    vertex_array: VertexArray<(V, I)>,
    instances: Vec<I>,
    dirty: bool,
}

impl<V, I> InstanceBatch<V, I>
where
    V: Vertex,
    I: Vertex,
{
    pub fn from_mesh(mesh: Mesh<V>) -> Result<Self, gl::Error> {
        let instance_buffer = Rc::new(VertexBuffer::new(mesh.element_buffer().gl())?);
        let vertex_array = VertexArray::new_instanced(
            mesh.element_buffer(),
            (mesh.vertex_buffer(), instance_buffer),
            &[0, 1],
        )?;

        Ok(Self {
            mesh,
            vertex_array,
            instances: Vec::new(),
            dirty: false,
        })
    }

    pub fn push(&mut self, instance: I) {
        self.instances.push(instance);
        self.dirty = true;
    }

    pub fn flush(&mut self) {
        if self.dirty {
            self.vertex_array
                .vertex_buffers()
                .1
                .set_data(&self.instances);
            self.dirty = false;
        }
    }

    pub fn draw_unit(&mut self) -> InstancedDrawUnit<(V, I)> {
        self.flush();
        InstancedDrawUnit::new(
            &self.vertex_array,
            self.mesh.primitive_mode(),
            self.mesh.element_range(),
            self.vertex_array.vertex_buffers().1.len(),
        )
    }

    pub fn clear(&mut self) {
        self.instances.clear();
        self.dirty = true;
    }
}

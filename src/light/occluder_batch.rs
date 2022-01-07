use std::rc::Rc;

use crate::{
    data::{Geometry, GeometryBuffer, LineTag, PrimitiveTag},
    gl::{self, ElementBuffer, InstancedDrawUnit, VertexArray, VertexBuffer},
};

use super::{Light, OccluderLineVertex};

pub struct OccluderBatch {
    buffer: GeometryBuffer<LineTag, OccluderLineVertex>,
    vertex_array: VertexArray<(OccluderLineVertex, Light)>,
    dirty: bool,
}

impl OccluderBatch {
    pub(super) fn new(light_instances: Rc<VertexBuffer<Light>>) -> Result<Self, gl::Error> {
        let buffer = GeometryBuffer::new();
        let element_buffer = Rc::new(ElementBuffer::new(light_instances.gl())?);
        let vertex_buffer = Rc::new(VertexBuffer::new(light_instances.gl())?);
        let vertex_array =
            VertexArray::new_instanced(element_buffer, (vertex_buffer, light_instances), &[0, 1])?;

        Ok(Self {
            buffer,
            vertex_array,
            dirty: false,
        })
    }

    pub fn push<G: Geometry<LineTag, Vertex = OccluderLineVertex>>(&mut self, geometry: G) {
        self.buffer.push(geometry);
        self.dirty = true;
    }

    pub fn flush(&mut self) {
        if self.dirty {
            self.buffer.upload(
                &*self.vertex_array.element_buffer(),
                &*self.vertex_array.vertex_buffers().0,
            );
            self.dirty = false;
        }
    }

    pub(super) fn draw_unit(&mut self) -> InstancedDrawUnit<(OccluderLineVertex, Light)> {
        self.flush();
        InstancedDrawUnit::new(
            &self.vertex_array,
            LineTag::primitive_mode(),
            0..self.vertex_array.vertex_buffers().0.len(),
            self.vertex_array.vertex_buffers().1.len(),
        )
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.dirty = true;
    }
}

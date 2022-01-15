use std::rc::Rc;

use crate::{
    data::{Geometry, LineBatch, LineTag, PrimitiveTag},
    gl::{self, DrawUnit, InstancedDrawUnit, VertexArray, VertexBuffer},
};

use super::{Light, OccluderLineVertex};

pub struct OccluderBatch {
    batch: LineBatch<OccluderLineVertex>,
    light_instanced_vertex_array: VertexArray<(OccluderLineVertex, Light)>,
}

impl OccluderBatch {
    pub(super) fn new(light_instances: Rc<VertexBuffer<Light>>) -> Result<Self, gl::Error> {
        let batch = LineBatch::new(light_instances.gl())?;
        let element_buffer = batch.vertex_array().element_buffer();
        let vertex_buffer = batch.vertex_array().vertex_buffers();
        let light_instanced_vertex_array =
            VertexArray::new_instanced(element_buffer, (vertex_buffer, light_instances), &[0, 1])?;

        Ok(Self {
            batch,
            light_instanced_vertex_array,
        })
    }

    pub fn push<G: Geometry<LineTag, Vertex = OccluderLineVertex>>(&mut self, geometry: G) {
        self.batch.push(geometry);
    }

    pub fn flush(&mut self) {
        self.batch.flush();
    }

    pub(super) fn light_instanced_draw_unit(
        &mut self,
    ) -> InstancedDrawUnit<(OccluderLineVertex, Light)> {
        self.flush();

        InstancedDrawUnit::new(
            &self.light_instanced_vertex_array,
            LineTag::primitive_mode(),
            0..self.light_instanced_vertex_array.vertex_buffers().0.len(),
            self.light_instanced_vertex_array.vertex_buffers().1.len(),
        )
    }

    pub(super) fn draw_unit(&mut self) -> DrawUnit<OccluderLineVertex> {
        self.batch.draw_unit()
    }

    pub fn clear(&mut self) {
        self.batch.clear();
    }
}

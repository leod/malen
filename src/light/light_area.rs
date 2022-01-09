use nalgebra::{Point2, Vector2, Vector4};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    attributes,
    data::{quad_triangle_indices, Geometry, TriangleTag},
    geom::Rect,
    gl::{Attribute, Vertex},
    Color3,
};

use super::Light;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LightAreaVertex {
    pub position: Point2<f32>,
    pub light_index: f32,
    pub light_position: Point2<f32>,
    pub light_params: Vector4<f32>,
    pub light_color: Color3,
}

impl Vertex for LightAreaVertex {
    fn attributes() -> Vec<Attribute> {
        attributes![
            "a_": position,
            light_index,
            light_position,
            light_params,
            light_color
        ]
    }
}

#[derive(Debug, Clone)]
pub struct LightRect {
    pub light_index: i32,
    pub light: Light,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct LightCircleSegment {
    pub light_index: i32,
    pub light: Light,
    pub num_segments: usize,
}

impl Geometry<TriangleTag> for LightRect {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(self.light.to_light_vertex(p, self.light_index));
        }
    }
}

impl Geometry<TriangleTag> for LightCircleSegment {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let start_index = vertices.len() as u32;

        vertices.push(
            self.light
                .to_light_vertex(self.light.position, self.light_index),
        );

        for i in 0..=self.num_segments {
            let angle = self.light.angle - self.light.angle_size / 2.0
                + i as f32 / self.num_segments as f32 * self.light.angle_size;
            let p =
                self.light.position + Vector2::new(angle.cos(), angle.sin()) * self.light.radius;
            vertices.push(self.light.to_light_vertex(p, self.light_index));

            if i != self.num_segments {
                elements.extend_from_slice(&[
                    start_index,
                    start_index + 1 + ((i + 1) % (self.num_segments + 1)) as u32,
                    start_index + 1 + i as u32,
                ]);
            }
        }
    }
}

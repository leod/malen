use nalgebra::Point3;

use crate::{math::Circle, Color4};

use super::{ColorVertex, Geometry, TriangleTag};

pub struct ColorCircle {
    pub circle: Circle,
    pub z: f32,
    pub angle: f32,
    pub num_segments: usize,
    pub color: Color4,
}

impl Geometry<TriangleTag> for ColorCircle {
    type Vertex = ColorVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let start_index = elements.len() as u32;

        vertices.push(ColorVertex {
            position: Point3::new(self.circle.center.x, self.circle.center.y, self.z),
            color: self.color,
        });

        for (i, p) in self
            .circle
            .points(self.angle, self.num_segments)
            .enumerate()
        {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });

            elements.extend_from_slice(&[
                start_index,
                start_index + 1 + ((i + 1) % self.num_segments) as u32,
                start_index + 1 + i as u32,
            ]);
        }
    }
}

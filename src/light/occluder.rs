use nalgebra::Point2;

use bytemuck::{Pod, Zeroable};

use crate::{
    attributes,
    data::{Geometry, LineTag},
    geom::{Circle, Line, Rect, RotatedRect},
    gl::{Attribute, Vertex},
};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct OccluderLineVertex {
    pub line_0: Point2<f32>,
    pub line_1: Point2<f32>,
    pub order: i32,
    pub height: f32,
    pub ignore_light_index1: i32,
    pub ignore_light_index2: i32,
}

impl Vertex for OccluderLineVertex {
    fn attributes() -> Vec<Attribute> {
        attributes![
            "a_": line_0,
            line_1,
            order,
            height,
            ignore_light_index1,
            ignore_light_index2
        ]
    }
}

#[derive(Debug, Clone)]
pub struct OccluderLine {
    pub line: Line,
    pub height: f32,
    pub ignore_light_index1: Option<u32>,
    pub ignore_light_index2: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderRect {
    pub rect: Rect,
    pub height: f32,
    pub ignore_light_index1: Option<u32>,
    pub ignore_light_index2: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderRotatedRect {
    pub rect: RotatedRect,
    pub height: f32,
    pub ignore_light_index1: Option<u32>,
    pub ignore_light_index2: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderCircle {
    pub circle: Circle,
    pub angle: f32,
    pub num_segments: usize,
    pub height: f32,
    pub ignore_light_index1: Option<u32>,
    pub ignore_light_index2: Option<u32>,
}

impl Geometry<LineTag> for OccluderLine {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let ignore_light_index1 = self
            .ignore_light_index1
            .map_or(-1, |i| i32::try_from(i).unwrap());
        let ignore_light_index2 = self
            .ignore_light_index2
            .map_or(-1, |i| i32::try_from(i).unwrap());

        let start_index = elements.len() as u32;
        elements.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
            start_index + 3,
        ]);

        vertices.extend_from_slice(&[
            OccluderLineVertex {
                line_0: self.line.0,
                line_1: self.line.1,
                order: 0,
                height: self.height,
                ignore_light_index1,
                ignore_light_index2,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 1,
                height: self.height,
                ignore_light_index1,
                ignore_light_index2,
            },
            OccluderLineVertex {
                line_0: self.line.0,
                line_1: self.line.1,
                order: 2,
                height: self.height,
                ignore_light_index1,
                ignore_light_index2,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 3,
                height: self.height,
                ignore_light_index1,
                ignore_light_index2,
            },
        ]);
    }
}

impl Geometry<LineTag> for OccluderRect {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for line in self.rect.edges().iter().copied() {
            OccluderLine {
                line,
                height: self.height,
                ignore_light_index1: self.ignore_light_index1,
                ignore_light_index2: self.ignore_light_index2,
            }
            .write(elements, vertices);
        }
    }
}

impl Geometry<LineTag> for OccluderRotatedRect {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for line in self.rect.edges().iter().copied() {
            OccluderLine {
                line,
                height: self.height,
                ignore_light_index1: self.ignore_light_index1,
                ignore_light_index2: self.ignore_light_index2,
            }
            .write(elements, vertices);
        }
    }
}

impl Geometry<LineTag> for OccluderCircle {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        // TODO: Alloc in OccluderCircle::write
        let points = self
            .circle
            .points(self.angle, self.num_segments)
            .collect::<Vec<_>>();

        for i in 0..points.len() - 1 {
            OccluderLine {
                line: Line(points[i], points[(i + 1) % points.len()]),
                height: self.height,
                ignore_light_index1: self.ignore_light_index1,
                ignore_light_index2: self.ignore_light_index2,
            }
            .write(elements, vertices);
        }

        OccluderLine {
            line: Line(points[points.len() - 1], points[0]),
            height: self.height,
            ignore_light_index1: self.ignore_light_index1,
            ignore_light_index2: self.ignore_light_index2,
        }
        .write(elements, vertices);
    }
}

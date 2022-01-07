use nalgebra::Point2;

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    attributes,
    data::{Geometry, LineTag},
    gl::{Attribute, Vertex},
    math::{Circle, Line},
    Rect, RotatedRect,
};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct OccluderLineVertex {
    pub line_0: Point2<f32>,
    pub line_1: Point2<f32>,
    pub order: i32,
    pub ignore_light_index: i32,
}

impl Vertex for OccluderLineVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": line_0, line_1, order, ignore_light_index]
    }
}

#[derive(Debug, Clone)]
pub struct OccluderLine {
    pub line: Line,
    pub ignore_light_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderRect {
    pub rect: Rect,
    pub ignore_light_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderRotatedRect {
    pub rect: RotatedRect,
    pub ignore_light_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderCircle {
    pub circle: Circle,
    pub angle: f32,
    pub num_segments: usize,
    pub ignore_light_index: Option<u32>,
}

impl From<Line> for OccluderLine {
    fn from(line: Line) -> Self {
        OccluderLine {
            line,
            ignore_light_index: None,
        }
    }
}

impl From<Rect> for OccluderRect {
    fn from(rect: Rect) -> Self {
        OccluderRect {
            rect,
            ignore_light_index: None,
        }
    }
}

impl Geometry<LineTag> for OccluderLine {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let ignore_light_index = self
            .ignore_light_index
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
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 1,
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.0,
                line_1: self.line.1,
                order: 2,
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 3,
                ignore_light_index,
            },
        ]);
    }
}

impl Geometry<LineTag> for OccluderRect {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for line in self
            .rect
            .lines()
            .iter()
            .chain(self.rect.caps().iter())
            .copied()
        {
            OccluderLine {
                line,
                ignore_light_index: self.ignore_light_index,
            }
            .write(elements, vertices);
        }
    }
}

impl Geometry<LineTag> for OccluderRotatedRect {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for line in self
            .rect
            .lines()
            .iter()
            .chain(self.rect.caps().iter())
            .copied()
        {
            OccluderLine {
                line,
                ignore_light_index: self.ignore_light_index,
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
            .chain(
                self.circle
                    .points(std::f32::consts::PI / 10.0, self.num_segments),
            )
            .collect::<Vec<_>>();

        for i in 0..points.len() {
            OccluderLine {
                line: Line(points[i], points[(i + 1) % points.len()]),
                ignore_light_index: self.ignore_light_index,
            }
            .write(elements, vertices);
        }
    }
}

use std::marker::PhantomData;

use golem::GeometryMode;

use crate::{
    draw::{shadow, Buffers, ColorVertex, Quad, TexVertex, Vertex},
    Color, Context, Error, Point2, Point3, Vector2,
};

pub struct Batch<V> {
    geometry_mode: GeometryMode,

    vertices: Vec<f32>,
    elements: Vec<u32>,
    num_vertices: usize,

    buffers: Buffers<V>,
    is_dirty: bool,

    _phantom: PhantomData<V>,
}

impl<V: Vertex> Batch<V> {
    pub fn new_triangles(ctx: &Context) -> Result<Self, Error> {
        Self::new(ctx, GeometryMode::Triangles)
    }

    pub fn new_lines(ctx: &Context) -> Result<Self, Error> {
        Self::new(ctx, GeometryMode::Lines)
    }

    pub fn new(ctx: &Context, geometry_mode: GeometryMode) -> Result<Self, Error> {
        let buffers = Buffers::new(ctx)?;

        Ok(Self {
            geometry_mode,
            vertices: Vec::new(),
            elements: Vec::new(),
            num_vertices: 0,
            buffers,
            is_dirty: false,
            _phantom: PhantomData,
        })
    }

    pub fn geometry_mode(&self) -> GeometryMode {
        self.geometry_mode
    }

    pub fn num_vertices(&self) -> usize {
        self.num_vertices
    }

    pub fn num_elements(&self) -> usize {
        self.elements.len()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.elements.clear();
        self.num_vertices = 0;
        self.is_dirty = true;
    }

    pub fn push_element(&mut self, element: u32) {
        assert!(element < self.vertices.len() as u32);

        self.elements.push(element);
        self.is_dirty = true;
    }

    pub fn push_vertex(&mut self, vertex: &V) {
        vertex.append(&mut self.vertices);
        self.num_vertices += 1;
        self.is_dirty = true;
    }

    pub fn push_triangle(&mut self, a: &V, b: &V, c: &V) {
        assert!(self.geometry_mode() == GeometryMode::Triangles);

        let first_idx = self.num_vertices() as u32;

        self.push_vertex(a);
        self.push_vertex(b);
        self.push_vertex(c);

        self.push_element(first_idx + 0);
        self.push_element(first_idx + 1);
        self.push_element(first_idx + 2);
    }

    pub fn flush(&mut self) {
        if self.is_dirty {
            self.buffers.vertices.set_data(&self.vertices);
            self.buffers.elements.set_data(&self.elements);
            self.buffers.num_elements = self.elements.len();

            self.is_dirty = false;
        }
    }

    pub fn buffers(&self) -> &Buffers<V> {
        &self.buffers
    }
}

impl Batch<ColorVertex> {
    pub fn push_quad(&mut self, quad: &Quad, color: Color) {
        assert!(self.geometry_mode == GeometryMode::Triangles);

        let first_idx = self.num_vertices() as u32;

        for corner in &quad.corners {
            self.push_vertex(&ColorVertex {
                world_pos: Point3::new(corner.x, corner.y, quad.z),
                color,
            });
        }

        self.elements.extend_from_slice(&[
            first_idx + 0,
            first_idx + 1,
            first_idx + 2,
            first_idx + 2,
            first_idx + 3,
            first_idx + 0,
        ]);
    }

    pub fn push_quad_outline(&mut self, quad: &Quad, color: Color) {
        assert!(self.geometry_mode == GeometryMode::Lines);

        let first_idx = self.num_vertices() as u32;

        for corner in &quad.corners {
            self.push_vertex(&ColorVertex {
                world_pos: Point3::new(corner.x, corner.y, quad.z),
                color,
            });
        }

        self.elements.extend_from_slice(&[
            first_idx + 0,
            first_idx + 1,
            first_idx + 1,
            first_idx + 2,
            first_idx + 2,
            first_idx + 3,
            first_idx + 3,
            first_idx + 0,
        ]);
    }
}

impl Batch<TexVertex> {
    pub fn push_quad(&mut self, quad: &Quad, tex_start: Point2, tex_size: Vector2) {
        assert!(self.geometry_mode == GeometryMode::Triangles);

        let first_idx = self.num_vertices() as u32;

        let tex_center = tex_start + tex_size / 2.0;

        for corner_idx in 0..4 {
            self.push_vertex(&TexVertex {
                world_pos: Point3::new(
                    quad.corners[corner_idx].x,
                    quad.corners[corner_idx].y,
                    quad.z,
                ),
                tex_coords: tex_center + Quad::corners()[corner_idx].component_mul(&tex_size),
            })
        }

        self.elements.extend_from_slice(&[
            first_idx + 0,
            first_idx + 1,
            first_idx + 2,
            first_idx + 2,
            first_idx + 3,
            first_idx + 0,
        ]);
    }
}

impl Batch<shadow::LineSegment> {
    pub fn push_occluder_line(
        &mut self,
        line_p: Point2,
        line_q: Point2,
        ignore_light_offset: Option<f32>,
    ) {
        assert!(self.geometry_mode == GeometryMode::Lines);

        let first_idx = self.num_vertices() as u32;

        let ignore_light_offset = ignore_light_offset.unwrap_or(-1.0);

        self.push_vertex(&shadow::LineSegment {
            world_pos_p: line_p,
            world_pos_q: line_q,
            order: 0.0,
            ignore_light_offset,
        });
        self.push_vertex(&shadow::LineSegment {
            world_pos_p: line_q,
            world_pos_q: line_p,
            order: 1.0,
            ignore_light_offset,
        });

        self.push_vertex(&shadow::LineSegment {
            world_pos_p: line_p,
            world_pos_q: line_q,
            order: 2.0,
            ignore_light_offset,
        });
        self.push_vertex(&shadow::LineSegment {
            world_pos_p: line_q,
            world_pos_q: line_p,
            order: 3.0,
            ignore_light_offset,
        });

        self.elements.extend_from_slice(&[
            first_idx + 0,
            first_idx + 1,
            first_idx + 2,
            first_idx + 3,
        ]);
    }

    pub fn push_occluder_quad(&mut self, quad: &Quad, ignore_light_offset: Option<f32>) {
        self.push_occluder_line(quad.corners[0], quad.corners[1], ignore_light_offset);
        self.push_occluder_line(quad.corners[1], quad.corners[2], ignore_light_offset);
        self.push_occluder_line(quad.corners[2], quad.corners[3], ignore_light_offset);
        self.push_occluder_line(quad.corners[3], quad.corners[0], ignore_light_offset);
    }
}

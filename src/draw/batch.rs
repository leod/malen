use std::marker::PhantomData;

use golem::{ElementBuffer, ShaderProgram, VertexBuffer};
use nalgebra::{Point2, Point3};

use crate::{
    Canvas, Color4, Error, Rect,
};
use super::{DrawUnit, ColVertex, Geometry, Line, Quad, TexColVertex, TexVertex, Triangle, Vertex};

#[derive(Default)]
struct Scratch {
    vertices: Vec<f32>,
    elements: Vec<u32>,
    num_vertices: usize,
    dirty: bool,
}

pub struct Batch<G: Geometry> {
    scratch: Scratch,

    vertices: VertexBuffer,
    elements: ElementBuffer,

    _phantom: PhantomData<G>,
}

pub type TriBatch<V> = Batch<Triangle<V>>;
pub type LineBatch<V> = Batch<Line<V>>;

impl<G: Geometry> Batch<G> {
    pub fn new_golem(ctx: &golem::Context) -> Result<Self, Error> {
        Ok(Self {
            scratch: Scratch::default(),
            vertices: VertexBuffer::new(ctx)?,
            elements: ElementBuffer::new(ctx)?,
            _phantom: PhantomData,
        })
    }

    pub fn new(ctx: &Canvas) -> Result<Self, Error> {
        Self::new_golem(ctx.golem_ctx())
    }

    pub fn vertices(&self) -> &VertexBuffer {
        &self.vertices
    }

    pub fn elements(&self) -> &ElementBuffer {
        &self.elements
    }

    pub fn num_vertices(&self) -> usize {
        self.scratch.vertices.len()
    }

    pub fn next_index(&self) -> u32 {
        self.scratch.num_vertices as u32
    }

    pub fn num_elements(&self) -> usize {
        self.scratch.num_vertices
    }

    pub fn push_element(&mut self, element: u32) {
        assert!(element < self.next_index());

        self.scratch.elements.push(element);
        self.scratch.dirty = true;
    }

    pub fn push_vertex(&mut self, vertex: &G::Vertex) {
        vertex.write(&mut self.scratch.vertices);
        self.scratch.num_vertices += 1;
        self.scratch.dirty = true;
    }

    pub fn clear(&mut self) {
        self.scratch.vertices.clear();
        self.scratch.elements.clear();
        self.scratch.num_vertices = 0;
        self.scratch.dirty = true;
    }

    pub fn draw_unit(&mut self) -> DrawUnit<'_, G::Vertex> {
        if self.scratch.dirty {
            self.vertices.set_data(&self.scratch.vertices);
            self.elements.set_data(&self.scratch.elements);
            self.scratch.dirty = false;
        }

        unsafe {
            DrawUnit::from_buffers_unchecked(
                &self.vertices,
                &self.elements,
                0,
                self.scratch.elements.len(),
                G::mode(),
            )
        }
    }

    pub fn draw(&mut self, shader: &ShaderProgram) -> Result<(), Error> {
        self.draw_unit().draw(shader)
    }
}

impl<V: Vertex> Batch<Triangle<V>> {
    pub fn push_triangle(&mut self, a: &V, b: &V, c: &V) {
        let first_idx = self.next_index();

        self.push_vertex(a);
        self.push_vertex(b);
        self.push_vertex(c);

        self.scratch
            .elements
            .extend_from_slice(&[first_idx + 0, first_idx + 1, first_idx + 2]);
    }
}

impl Batch<Triangle<ColVertex>> {
    pub fn push_quad(&mut self, quad: &Quad, z: f32, color: Color4) {
        let first_idx = self.next_index();

        for corner in &quad.corners {
            self.push_vertex(&ColVertex {
                world_pos: Point3::new(corner.x, corner.y, z),
                color,
            });
        }

        self.scratch
            .elements
            .extend_from_slice(&Quad::triangle_indices(first_idx));
    }
}

impl Batch<Line<ColVertex>> {
    pub fn push_quad_outline(&mut self, quad: &Quad, z: f32, color: Color4) {
        let first_idx = self.next_index();

        for corner in &quad.corners {
            self.push_vertex(&ColVertex {
                world_pos: Point3::new(corner.x, corner.y, z),
                color,
            });
        }

        self.scratch.elements.extend_from_slice(&[
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

    pub fn push_line(&mut self, p: Point2<f32>, q: Point2<f32>, z: f32, color: Color4) {
        let first_idx = self.next_index();

        self.push_vertex(&ColVertex {
            world_pos: Point3::new(p.x, p.y, z),
            color,
        });
        self.push_vertex(&ColVertex {
            world_pos: Point3::new(q.x, q.y, z),
            color,
        });

        self.scratch
            .elements
            .extend_from_slice(&[first_idx + 0, first_idx + 1]);
    }
}

impl TriBatch<TexVertex> {
    pub fn push_quad(&mut self, quad: &Quad, z: f32, uv_rect: Rect) {
        let first_idx = self.next_index();

        for corner_idx in 0..4 {
            self.push_vertex(&TexVertex {
                world_pos: Point3::new(quad.corners[corner_idx].x, quad.corners[corner_idx].y, z),
                tex_coords: uv_rect.center
                    + Quad::corners()[corner_idx].component_mul(&uv_rect.size),
            })
        }

        self.scratch
            .elements
            .extend_from_slice(&Quad::triangle_indices(first_idx));
    }
}

impl TriBatch<TexColVertex> {
    pub fn push_quad(&mut self, quad: &Quad, z: f32, uv_rect: Rect, color: Color4) {
        let first_idx = self.next_index();

        for corner_idx in 0..4 {
            self.push_vertex(&TexColVertex {
                world_pos: Point3::new(quad.corners[corner_idx].x, quad.corners[corner_idx].y, z),
                tex_coords: uv_rect.center
                    + Quad::corners()[corner_idx].component_mul(&uv_rect.size),
                color,
            })
        }

        self.scratch
            .elements
            .extend_from_slice(&Quad::triangle_indices(first_idx));
    }
}

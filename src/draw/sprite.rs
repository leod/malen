use std::marker::PhantomData;

use nalgebra as na;

use golem::{
    Attribute, AttributeType, Dimension, ElementBuffer, GeometryMode, ShaderDescription,
    ShaderProgram, Uniform, UniformType, UniformValue, VertexBuffer,
};

use crate::{
    geom::matrix3_to_flat_array,
    Color, Error, Matrix3, Point2, Point3, Vector2,
};

pub trait Sprite {
    fn attributes() -> Vec<Attribute>;
    fn write_vertices(&self, out: &mut Vec<f32>);
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub corners: [Point2; 4],
    pub z: f32,
}

impl Quad {
    pub fn new(transform: &Matrix3) -> Self {
        // We apply the model transformation on CPU. This seems to be the
        // easiest way to render moderate amounts of sprites in a somewhat 
        // performant way with WebGL 1: We don't have an easy way to send the
        // per-sprite data to GPU, since we don't have access to UBOs and SSBOs.
        Self {
            corners: [
                (transform * Point3::new(-0.5, -0.5, 1.0)).xy(),
                (transform * Point3::new(-0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, -0.5, 1.0)).xy(),
            ],
            z: transform[(2, 2)],
        }
    }

    pub fn axis_aligned(pos: Point3, size: Vector2) -> Self {
        Self {
            corners: [
                // Top left
                pos.xy() + Vector2::new(-0.5, -0.5).component_mul(&size),
                // Bottom left
                pos.xy() + Vector2::new(-0.5, 0.5).component_mul(&size),
                // Bottom right
                pos.xy() + Vector2::new(0.5, 0.5).component_mul(&size),
                // Top right
                pos.xy() + Vector2::new(0.5, -0.5).component_mul(&size),
            ],
            z: pos.z,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorSprite {
    pub quad: Quad,
    pub color: Color,
}

impl ColorSprite {
    pub fn new(transform: &Matrix3, color: Color) -> Self {
        Self {
            quad: Quad::new(transform),
            color,
        }
    }

    pub fn axis_aligned(pos: Point3, size: Vector2, color: Color) -> Self {
        Self {
            quad: Quad::axis_aligned(pos, size),
            color,
        }
    }
}

impl Sprite for ColorSprite {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn write_vertices(&self, out: &mut Vec<f32>) {
        out.reserve(4 * (3 + 4));

        // Quad corners in counter-clockwise order. Order is important here
        // because of backface culling.
        for corner in &self.quad.corners {
            out.extend_from_slice(&[
                corner.x,
                corner.y,
                self.quad.z,
                self.color.x,
                self.color.y,
                self.color.z,
                self.color.w,
            ]);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpriteList<S> {
    vertices: Vec<f32>,
    elements: Vec<u32>,
    _phantom: PhantomData<S>,
}

impl<S> Default for SpriteList<S> {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            elements: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S, T> From<T> for SpriteList<S>
where
    T: IntoIterator<Item = S>,
    S: Sprite,
{
    fn from(sprites: T) -> Self {
        let mut list = SpriteList::new();
        list.extend(sprites);
        list
    }
}

impl<S: Sprite> SpriteList<S> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, sprite: &S) {
        let first_idx = self.vertices.len() as u32;

        sprite.write_vertices(&mut self.vertices);

        // Add two triangles, again being careful about the order because of
        // backface culling.
        self.elements.reserve(6);

        for &offset in &[0, 1, 2, 2, 3, 0] {
            self.elements.push(first_idx + offset);
        }
    }

    pub fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        // TODO: Use size hint in SpriteBuffer::extend?
        for sprite in iter {
            self.push(&sprite);
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.elements.clear();
    }
}

pub struct SpriteBatch<S> {
    vertices: VertexBuffer,
    elements: ElementBuffer,
    num_elements: usize,
    _phantom: PhantomData<S>,
}

impl<S: Sprite> SpriteBatch<S> {
    pub fn new(ctx: &golem::Context) -> Result<Self, Error> {
        Ok(Self {
            vertices: VertexBuffer::new(ctx)?,
            elements: ElementBuffer::new(ctx)?,
            num_elements: 0,
            _phantom: PhantomData,
        })
    }

    pub fn from_list(ctx: &golem::Context, data: &SpriteList<S>) -> Result<Self, Error> {
        let mut result = Self::new(ctx)?;
        result.set_data(data);
        Ok(result)
    }

    pub fn set_data(&mut self, data: &SpriteList<S>) {
        self.vertices.set_data(&data.vertices);
        self.elements.set_data(&data.elements);

        self.num_elements = data.elements.len();
    }
}

pub struct SpritePass<S> {
    shader: ShaderProgram,
    _phantom: PhantomData<S>,
}

impl SpritePass<ColorSprite> {
    pub fn new(ctx: &golem::Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx,
            ShaderDescription {
                vertex_input: &ColorSprite::attributes(),
                fragment_input: &[Attribute::new(
                    "v_color",
                    AttributeType::Vector(Dimension::D4),
                )],
                uniforms: &[Uniform::new(
                    "mat_projection_view",
                    UniformType::Matrix(Dimension::D3),
                )],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos.xy, 1.0);
                    gl_Position = vec4(p.xy, a_world_pos.z, 1.0);
                    v_color = a_color;
                }
                "#,
                fragment_shader: r#"
                void main() {
                    gl_FragColor = v_color;
                }
                "#,
            },
        )?;

        Ok(SpritePass {
            shader,
            _phantom: PhantomData,
        })
    }

    pub fn draw(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        batch: &SpriteBatch<ColorSprite>,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;

        unsafe {
            self.shader.draw(
                &batch.vertices,
                &batch.elements,
                0..batch.num_elements,
                GeometryMode::Triangles,
            )?;
        }

        Ok(())
    }
}

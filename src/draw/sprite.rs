use std::marker::PhantomData;

use golem::{
    Attribute, AttributeType, Dimension, ElementBuffer, GeometryMode, ShaderDescription,
    ShaderProgram, Uniform, UniformType, VertexBuffer,
};

use crate::{Color, Error, Matrix3, Vector2, Vector3};

pub trait Sprite {
    fn attributes() -> Vec<Attribute>;
    fn write_vertices(&self, out: &mut Vec<f32>);
}

#[derive(Debug, Clone)]
pub struct ColorSprite {
    pub transform: Matrix3,
    pub color: Color,
}

impl Sprite for ColorSprite {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn write_vertices(&self, out: &mut Vec<f32>) {
        // Quad corners in counter-clockwise order. Order is important here
        // because of backface culling.
        let corners = [
            // Top left
            Vector2::new(-0.5, -0.5),
            // Bottom left
            Vector2::new(-0.5, 0.5),
            // Bottom right
            Vector2::new(0.5, 0.5),
            // Top right
            Vector2::new(0.5, -0.5),
        ];

        // Apply the model transformation on CPU. This seems to be the easiest
        // way to render moderate amounts of sprites in a somewhat performant
        // way with WebGL 1: We don't have an easy way to send the per-sprite
        // data to GPU, since we don't have access to UBOs and SSBOs.
        for corner in &corners {
            let corner_world = self.transform * Vector3::new(corner.x, corner.y, 1.0);

            out.push(corner_world.x);
            out.push(corner_world.y);
            out.push(corner_world.z);
        }

        out.push(self.color.x);
        out.push(self.color.y);
        out.push(self.color.z);
        out.push(self.color.w);
    }
}

#[derive(Default)]
pub struct SpriteStage<S> {
    vertices: Vec<f32>,
    elements: Vec<u32>,
    _phantom: PhantomData<S>,
}

impl<S: Sprite> SpriteStage<S> {
    pub fn push(&mut self, sprite: S) {
        let first_idx = self.vertices.len() as u32;

        sprite.write_vertices(&mut self.vertices);

        // Add two triangles, again being careful about the order because of
        // backface culling.
        for &offset in &[0, 1, 2, 2, 3, 0] {
            self.elements.push(first_idx + offset);
        }
    }

    pub fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        // TODO: Use size hint in SpriteBuffer::extend?
        for sprite in iter {
            self.push(sprite);
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
    _phantom: PhantomData<S>,
}

impl<S: Sprite> SpriteBatch<S> {
    pub fn new(ctx: &golem::Context) -> Result<Self, Error> {
        Ok(Self {
            vertices: VertexBuffer::new(ctx)?,
            elements: ElementBuffer::new(ctx)?,
            _phantom: PhantomData,
        })
    }

    pub fn set_data(&mut self, data: &SpriteStage<S>) {
        self.vertices.set_data(&data.vertices);
        self.elements.set_data(&data.elements);
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
                    vec3 p = mat_projection_view * vec3(a_world_pos);
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

    pub fn draw(&self, batch: &SpriteBatch<ColorSprite>) -> Result<(), Error> {
        unsafe {
            self.shader.draw(
                &batch.vertices,
                &batch.elements,
                0..batch.elements.size(),
                GeometryMode::TriangleStrip,
            )?;
        }

        Ok(())
    }
}

use std::rc::Rc;

use bytemuck::Pod;
use nalgebra::{Matrix2, Matrix3, Matrix4, Point2, Point3, Point4, Vector2, Vector3, Vector4};

use glow::HasContext;

use crate::Color4;

use super::{Context, VertexBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeValueType {
    Float,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: &'static str,
    pub offset: usize,
    pub glsl_type_name: &'static str,
    pub element_type: AttributeValueType,
    pub num_elements: usize,
}

pub trait Vertex: Pod {
    fn attributes() -> Vec<Attribute>;

    unsafe fn bind_to_vertex_array(
        gl: &Context,
        vertex_buffer: &VertexBuffer<Self>,
        mut index: usize,
    ) -> usize {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer.id()));

        for attribute in Self::attributes().iter() {
            assert!(
                attribute.offset + attribute.num_elements * attribute.element_type.size()
                    <= std::mem::size_of::<Self>()
            );

            gl.enable_vertex_attrib_array(index as u32);

            match attribute.element_type {
                AttributeValueType::Float => gl.vertex_attrib_pointer_f32(
                    index as u32,
                    attribute.num_elements as i32,
                    attribute.element_type.to_gl(),
                    false,
                    std::mem::size_of::<Self>() as i32,
                    attribute.offset as i32,
                ),
                AttributeValueType::Int => gl.vertex_attrib_pointer_i32(
                    index as u32,
                    attribute.num_elements as i32,
                    attribute.element_type.to_gl(),
                    std::mem::size_of::<Self>() as i32,
                    attribute.offset as i32,
                ),
            }

            index += 1;
        }

        index
    }
}

pub trait VertexDecls {
    const N: usize;

    type RcVertexBufferTuple: Clone;

    fn attributes() -> Vec<Attribute>;

    unsafe fn bind_to_vertex_array(
        gl: &Context,
        buffers: Self::RcVertexBufferTuple,
        index: usize,
    ) -> usize;
}

impl<V> VertexDecls for V
where
    V: Vertex,
{
    const N: usize = 1;

    type RcVertexBufferTuple = Rc<VertexBuffer<V>>;

    fn attributes() -> Vec<Attribute> {
        V::attributes()
    }

    unsafe fn bind_to_vertex_array(
        gl: &Context,
        buffers: Self::RcVertexBufferTuple,
        index: usize,
    ) -> usize {
        V::bind_to_vertex_array(gl, &*buffers, index)
    }
}

impl<V0, V1> VertexDecls for (V0, V1)
where
    V0: Vertex,
    V1: Vertex,
{
    const N: usize = 2;

    type RcVertexBufferTuple = (Rc<VertexBuffer<V0>>, Rc<VertexBuffer<V1>>);

    fn attributes() -> Vec<Attribute> {
        [&V0::attributes()[..], &V1::attributes()[..]].concat()
    }

    unsafe fn bind_to_vertex_array(
        gl: &Context,
        buffers: Self::RcVertexBufferTuple,
        index: usize,
    ) -> usize {
        V1::bind_to_vertex_array(
            gl,
            &*buffers.1,
            V0::bind_to_vertex_array(gl, &*buffers.0, index),
        )
    }
}

impl<V0, V1, V2> VertexDecls for (V0, V1, V2)
where
    V0: Vertex,
    V1: Vertex,
    V2: Vertex,
{
    const N: usize = 3;

    type RcVertexBufferTuple = (
        Rc<VertexBuffer<V0>>,
        Rc<VertexBuffer<V1>>,
        Rc<VertexBuffer<V2>>,
    );

    fn attributes() -> Vec<Attribute> {
        [
            &V0::attributes()[..],
            &V1::attributes()[..],
            &V2::attributes()[..],
        ]
        .concat()
    }

    unsafe fn bind_to_vertex_array(
        gl: &Context,
        buffers: Self::RcVertexBufferTuple,
        index: usize,
    ) -> usize {
        V2::bind_to_vertex_array(
            gl,
            &*buffers.2,
            V1::bind_to_vertex_array(
                gl,
                &*buffers.1,
                V0::bind_to_vertex_array(gl, &*buffers.0, index),
            ),
        )
    }
}

pub fn attribute<T: DataType>(name: &'static str, offset: usize) -> Attribute {
    Attribute {
        name,
        offset,
        glsl_type_name: T::glsl_name(),
        element_type: T::element_type(),
        num_elements: T::num_elements(),
    }
}

impl AttributeValueType {
    pub fn to_gl(self) -> u32 {
        match self {
            AttributeValueType::Float => glow::FLOAT,
            AttributeValueType::Int => glow::INT,
        }
    }

    pub fn size(self) -> usize {
        match self {
            AttributeValueType::Float => std::mem::size_of::<f32>(),
            AttributeValueType::Int => std::mem::size_of::<i32>(),
        }
    }
}

impl Attribute {
    pub fn glsl_string(&self) -> String {
        format!("in {} {};\n", self.glsl_type_name, self.name)
    }
}

pub trait DataType {
    fn glsl_name() -> &'static str;
    fn element_type() -> AttributeValueType;
    fn num_elements() -> usize;
}

macro_rules! impl_data_type {
    ($type:ty, $name:ident, $element_type:ident, $num_elements:expr) => {
        impl crate::gl::DataType for $type {
            fn glsl_name() -> &'static str {
                stringify!($name)
            }

            fn element_type() -> crate::gl::AttributeValueType {
                crate::gl::AttributeValueType::$element_type
            }

            fn num_elements() -> usize {
                $num_elements
            }
        }
    };
}

impl_data_type!(f32, float, Float, 1);
impl_data_type!(i32, int, Int, 1);

impl_data_type!(Vector2<f32>, vec2, Float, 2);
impl_data_type!(Point2<f32>, vec2, Float, 2);
impl_data_type!(Vector2<i32>, ivec2, Int, 2);
impl_data_type!(Point2<i32>, ivec2, Int, 2);

impl_data_type!(Vector3<f32>, vec3, Float, 3);
impl_data_type!(Point3<f32>, vec3, Float, 3);
impl_data_type!(Vector3<i32>, ivec3, Int, 3);
impl_data_type!(Point3<i32>, ivec3, Int, 3);

impl_data_type!(Vector4<f32>, vec4, Float, 4);
impl_data_type!(Point4<f32>, vec4, Float, 4);
impl_data_type!(Vector4<i32>, ivec4, Int, 4);
impl_data_type!(Point4<i32>, ivec4, Int, 4);

impl_data_type!(Matrix2<f32>, mat2, Float, 2 * 2);
impl_data_type!(Matrix2<i32>, imat2, Int, 2 * 2);

impl_data_type!(Matrix3<f32>, mat3, Float, 3 * 3);
impl_data_type!(Matrix3<i32>, imat3, Int, 3 * 3);

impl_data_type!(Matrix4<f32>, mat4, Float, 4 * 4);
impl_data_type!(Matrix4<i32>, imat4, Int, 4 * 4);

impl_data_type!(Color4, vec4, Float, 4);

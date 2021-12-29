use bytemuck::Pod;
use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Float,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: &'static str,
    pub offset: usize,
    pub glsl_type_name: &'static str,
    pub element_type: ValueType,
    pub num_elements: usize,
}

pub trait Vertex: Pod {
    fn attributes() -> Vec<Attribute>;
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

impl ValueType {
    pub fn to_gl(self) -> u32 {
        match self {
            ValueType::Float => glow::FLOAT,
            ValueType::Int => glow::INT,
        }
    }

    pub fn size(self) -> usize {
        match self {
            ValueType::Float => std::mem::size_of::<f32>(),
            ValueType::Int => std::mem::size_of::<i32>(),
        }
    }
}

impl Attribute {
    pub fn glsl_string(&self) -> String {
        format!("in {} {}", self.glsl_type_name, self.name)
    }
}

pub trait DataType {
    fn glsl_name() -> &'static str;
    fn element_type() -> ValueType;
    fn num_elements() -> usize;
}

macro_rules! impl_data_type {
    ($type:ty, $name:ident, $element_type:ident, $num_elements:expr) => {
        impl crate::gl::DataType for $type {
            fn glsl_name() -> &'static str {
                stringify!($name)
            }

            fn element_type() -> crate::gl::ValueType {
                crate::gl::ValueType::$element_type
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
impl_data_type!(Vector2<i32>, ivec2, Int, 2);

impl_data_type!(Vector3<f32>, vec3, Float, 3);
impl_data_type!(Vector3<i32>, ivec3, Int, 3);

impl_data_type!(Vector4<f32>, vec4, Float, 4);
impl_data_type!(Vector4<i32>, ivec4, Int, 4);

impl_data_type!(Matrix2<f32>, mat2, Float, 2 * 2);
impl_data_type!(Matrix2<i32>, imat2, Int, 2 * 2);

impl_data_type!(Matrix3<f32>, mat3, Float, 3 * 3);
impl_data_type!(Matrix3<i32>, imat3, Int, 3 * 3);

impl_data_type!(Matrix4<f32>, mat4, Float, 4 * 4);
impl_data_type!(Matrix4<i32>, imat4, Int, 4 * 4);

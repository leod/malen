use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::Matrix4;

use crate::gl::UniformBlock;

#[derive(Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct Matrices {
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
}

impl UniformBlock for Matrices {
    const INSTANCE_NAME: &'static str = "matrices";
}

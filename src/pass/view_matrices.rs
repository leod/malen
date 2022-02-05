use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::Matrix3;

use crate::gl::UniformBlock;

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct ViewMatrices {
    pub projection: Matrix3<f32>,
    pub view: Matrix3<f32>,
}

impl UniformBlock for ViewMatrices {}

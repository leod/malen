use nalgebra::{Matrix3, Vector2, Vector3};

pub fn matrix3_to_array(m: &Matrix3<f32>) -> [f32; 9] {
    [
        m[(0, 0)],
        m[(1, 0)],
        m[(2, 0)],
        m[(0, 1)],
        m[(1, 1)],
        m[(2, 1)],
        m[(0, 2)],
        m[(1, 2)],
        m[(2, 2)],
    ]
}

/// Build a 3x3 matrix that applies transformations in this order: first
/// scaling, then translation.
pub fn scale_translate3(scale: Vector2<f32>, offset: Vector3<f32>) -> Matrix3<f32> {
    #[rustfmt::skip]
    let transform = Matrix3::new(
        scale.x, 0.0,     offset.x,
        0.0,     scale.y, offset.y,
        0.0,     0.0,     offset.z,
    );

    transform
}

/// Build a 3x3 matrix that applies transformations in this order: first
/// scaling, then translation.
pub fn scale_translate(scale: Vector2<f32>, offset: Vector2<f32>) -> Matrix3<f32> {
    scale_translate3(scale, Vector3::new(offset.x, offset.y, 0.0))
}

/// Build a 3x3 matrix that applies transformations in this order: first
/// translation, then scaling and rotation.
pub fn translate_rotate_scale(
    offset: Vector2<f32>,
    angle: f32,
    scale: Vector2<f32>,
) -> Matrix3<f32> {
    // TODO: The point is that we can write this without using matrix
    //       multiplications.
    Matrix3::new_nonuniform_scaling(&scale)
        * Matrix3::new_rotation(angle)
        * Matrix3::new_translation(&offset)
}

/// Build a 3x3 matrix that applies transformations in this order: first
/// scaling, then rotation, and finally translation.
pub fn scale_rotate_translate(
    scale: Vector2<f32>,
    angle: f32,
    offset: Vector2<f32>,
) -> Matrix3<f32> {
    // TODO: The point is that we can write this without using matrix
    //       multiplications.
    Matrix3::new_translation(&offset)
        * Matrix3::new_rotation(angle)
        * Matrix3::new_nonuniform_scaling(&scale)
}

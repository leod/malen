use nalgebra as na;

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;
pub type Matrix2 = na::Matrix2<f32>;
pub type Matrix3 = na::Matrix3<f32>;

/// Build a 3x3 matrix that applies transformations in this order: first
/// translation, then scaling and rotation.
pub fn translate_rotate_scale(offset: Vector2, angle: f32, scale: Vector2) -> Matrix3 {
    // TODO: The point is that we can write this without using matrix
    //       multiplications.
    na::Matrix3::new_nonuniform_scaling(&scale)
        * na::Matrix3::new_rotation(angle)
        * na::Matrix3::new_translation(&offset)
}

#[derive(Debug, Clone)]
/// Parameters that define a two-dimensional camera transformation.
pub struct Camera {
    /// The center position of the camera.
    pub center: Point2,

    /// Zoom factor. For values larger than one, things will look larger, while
    /// for values smaller than one they will look smaller.
    /// Using nonpositive values will probably lead to funny (or frustrating)
    /// results.
    pub zoom: f32,

    /// Angle of rotation around the center in radians.
    pub angle: f32,
}

impl Camera {
    /// Build a 3x3 matrix with homogeneous coordinates to represent the
    /// transformation from world space to camera space.
    pub fn to_matrix(&self) -> Matrix3 {
        // It's a bit easier to first consider the camera space -> world space
        // transformation C2W and then take the inverse to get W2C. For C2W, we
        // first need to scale with S / rotate with R (order shouldn't matter
        // since we scale all axes the same), and then translate with T:
        //
        //     C2W = T(center) * S(1.0 / zoom) * R(angle)
        //
        //     => W2C = C2W^-1 = S(zoom) * R(-angle) * T(-center)
        //
        // (Using S(x)^-1 = S(1/x),
        //        R(x)^-1 = R(-x),
        //        T(x)^-1 = T(-x).)

        translate_rotate_scale(
            -self.center.coords,
            -self.angle,
            na::Vector2::new(self.zoom, self.zoom),
        )
    }
}

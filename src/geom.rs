use nalgebra::{Matrix3, Point2, Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct AaRect {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
}

impl AaRect {
    pub fn zero_to_one() -> Self {
        Self::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0))
    }

    pub fn from_top_left(top_left: Point2<f32>, size: Vector2<f32>) -> Self {
        Self {
            center: top_left + size / 2.0,
            size,
        }
    }

    pub fn from_bottom_left(bottom_left: Point2<f32>, size: Vector2<f32>) -> Self {
        Self {
            center: bottom_left + Vector2::new(size.x, -size.y) / 2.0,
            size,
        }
    }
}

pub fn matrix3_to_flat_array(m: &Matrix3<f32>) -> [f32; 9] {
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
/// scaling, then translation
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
/// scaling, then translation
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

#[derive(Debug, Clone, Copy)]
pub struct ScreenGeom {
    /// The screen size in pixels.
    pub size: Vector2<u32>,

    pub device_pixel_ratio: f64,
}

impl ScreenGeom {
    /// Returns an orthographic projection matrix.
    ///
    /// The returned matrix maps `[0..width] x [0..height]` to
    /// `[-1..1] x [-1..1]` (i.e. the OpenGL normalized device coordinates).
    ///
    /// Notes:
    /// - This projection also flips the Y axis, so that (0,0) is at the
    ///   top-left of your screen.
    /// - We assume the Z coordinate of the input vector to be set to 1.
    pub fn orthographic_projection(&self) -> Matrix3<f32> {
        let ratio = self.device_pixel_ratio as f32;
        let scale_to_unit = Matrix3::new_nonuniform_scaling(&Vector2::new(
            ratio / self.size.x as f32,
            ratio / self.size.y as f32,
        ));
        let shift = Matrix3::new_translation(&Vector2::new(-0.5, -0.5));
        let scale_and_flip_y = Matrix3::new_nonuniform_scaling(&Vector2::new(2.0, -2.0));

        scale_and_flip_y * shift * scale_to_unit
    }
}

#[derive(Debug, Clone)]
/// Parameters that define a two-dimensional camera transformation.
pub struct Camera {
    /// The center position of the camera.
    pub center: Point2<f32>,

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
    pub fn to_matrix(&self, screen: &ScreenGeom) -> Matrix3<f32> {
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

        Matrix3::new_translation(&Vector2::new(
            screen.size.x as f32 / (2.0 * screen.device_pixel_ratio as f32),
            screen.size.y as f32 / (2.0 * screen.device_pixel_ratio as f32),
        )) * translate_rotate_scale(
            -self.center.coords,
            -self.angle,
            Vector2::new(self.zoom, self.zoom),
        )
    }
}

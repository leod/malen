use std::rc::Rc;

use glow::HasContext;

use super::{Blend, Context, DepthTest};

#[derive(Clone, Debug, PartialEq)]
pub struct DrawParams {
    blend: Option<Blend>,
    depth_test: Option<DepthTest>,
}

pub fn set_draw_params(gl: Rc<Context>, draw_params: &DrawParams) {
    // TODO: We may eventually need to implement some caching here.

    set_blend(gl, draw_params.blend);
    set_depth_test(gl, draw_params.depth_test);
}

fn set_blend(gl: Rc<Context>, blend: Option<Blend>) {
    match blend {
        None => unsafe {
            gl.disable(glow::BLEND);
        },
        Some(Blend {
            equation,
            func,
            constant_color,
        }) => {
            unsafe {
                gl.enable(glow::BLEND);
            }

            if equation.is_same() {
                unsafe {
                    gl.blend_equation(equation.color.to_gl());
                }
            } else {
                unsafe {
                    gl.blend_equation_separate(equation.color.to_gl(), equation.alpha.to_gl());
                }
            }

            if func.is_same() {
                unsafe {
                    gl.blend_func(func.src_color.to_gl(), func.dst_color.to_gl());
                }
            } else {
                unsafe {
                    gl.blend_func_separate(
                        func.src_color.to_gl(),
                        func.src_alpha.to_gl(),
                        func.dst_color.to_gl(),
                        func.dst_alpha.to_gl(),
                    )
                }
            }

            unsafe {
                gl.blend_color(
                    constant_color.r,
                    constant_color.g,
                    constant_color.b,
                    constant_color.a,
                );
            }
        }
    }
}

fn set_depth_test(gl: Rc<Context>, depth_test: Option<DepthTest>) {
    match depth_test {
        None => unsafe {
            gl.disable(glow::DEPTH_TEST);
        },
        Some(DepthTest {
            func,
            range_near,
            range_far,
            write,
        }) => unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(func.to_gl());
            gl.depth_range_f32(range_near, range_far);
            gl.depth_mask(write);
        },
    }
}

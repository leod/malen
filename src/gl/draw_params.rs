use std::rc::Rc;

use glow::HasContext;

use crate::gl::{self, Blend};

// Borrowing *heavily* from the awesome glium and golem libraries here

pub struct DrawParams {}

pub fn set_blend(gl: Rc<gl::Context>, blend: Option<Blend>) {
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

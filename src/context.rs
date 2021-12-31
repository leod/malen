use std::rc::Rc;

use super::{gl, Canvas};

pub struct Context {
    canvas: Canvas,
    gl: Rc<gl::Context>,
}

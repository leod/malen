use std::ops::Deref;

pub struct Context {
    gl: glow::Context,
}

impl Context {
    pub fn new(gl: glow::Context) -> Self {
        Context { gl }
    }
}

impl Deref for Context {
    type Target = glow::Context;

    fn deref(&self) -> &glow::Context {
        &self.gl
    }
}

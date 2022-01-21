use std::ops::Deref;

pub struct Context {
    context: glow::Context,
}

impl Context {
    pub fn new(gl: glow::Context) -> Self {
        Context { context: gl }
    }
}

impl Deref for Context {
    type Target = glow::Context;

    fn deref(&self) -> &glow::Context {
        &self.context
    }
}

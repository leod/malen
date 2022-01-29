use std::{cell::Cell, ops::Deref};

use glow::HasContext;

pub struct Context {
    context: glow::Context,
    pub(super) main_viewport: Cell<[i32; 4]>,
}

impl Context {
    pub fn new(context: glow::Context) -> Self {
        let mut main_viewport = [0, 0, 0, 0];

        unsafe {
            context.get_parameter_i32_slice(glow::VIEWPORT, &mut main_viewport);
        }

        Context {
            context,
            main_viewport: Cell::new(main_viewport),
        }
    }

    pub fn set_main_viewport(&self, viewport: [i32; 4]) {
        self.main_viewport.set(viewport);
        unsafe {
            self.context
                .viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
        }
    }
}

impl Deref for Context {
    type Target = glow::Context;

    fn deref(&self) -> &glow::Context {
        &self.context
    }
}

mod batch;
mod buffers;
mod pass;
mod primitive;
pub mod shadow;
mod text;

use crate::{
    geom::{Screen, Vector2},
    Error,
};
use web_sys::HtmlCanvasElement;

pub use batch::Batch;
pub use buffers::{AsBuffersSlice, Buffers, BuffersSlice};
pub use pass::ColorPass;
pub use primitive::{ColorVertex, GeometryMode, Quad, TexVertex, Vertex};
pub use shadow::{ShadowMap, ShadowedColorPass};

pub struct Draw {
    canvas: HtmlCanvasElement,
    golem_ctx: golem::Context,
}

impl Draw {
    pub fn new(canvas: HtmlCanvasElement, golem_ctx: golem::Context) -> Result<Self, Error> {
        Ok(Draw { canvas, golem_ctx })
    }
    pub fn screen(&self) -> Screen {
        Screen {
            size: Vector2::new(self.canvas.width() as f32, self.canvas.height() as f32),
        }
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }
}

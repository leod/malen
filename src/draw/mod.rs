mod batch;
mod buffers;
mod pass;
mod primitive;
pub mod shadow;
mod text;

use nalgebra as na;
use web_sys::HtmlCanvasElement;

use crate::{geom::Screen, Error};

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
            size: na::Vector2::new(self.canvas.width(), self.canvas.height()),
        }
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }
}

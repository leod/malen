use web_sys::HtmlCanvasElement;

use nalgebra::{Point2, Vector2};

use crate::{AaRect, Color4, Error, ScreenGeom, draw::{util, Texture, TriBatch, TexColVertex, TexColPass}};

pub struct Draw {
    canvas: HtmlCanvasElement,
    golem_ctx: golem::Context,

    debug_tex_batch: Option<TriBatch<TexColVertex>>,
    debug_tex_pass: Option<TexColPass>,
}

impl Draw {
    pub fn new(canvas: HtmlCanvasElement, golem_ctx: golem::Context) -> Result<Self, Error> {
        util::set_canvas_size(&canvas, Vector2::new(canvas.width(), canvas.height()));

        Ok(Draw {
            canvas,
            golem_ctx,
            debug_tex_batch: None,
            debug_tex_pass: None,
        })
    }

    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    pub fn resize(&self, logical_size: Vector2<u32>) {
        util::set_canvas_size(&self.canvas, logical_size);
    }

    pub fn resize_full(&self) {
        // A collection of anti-patterns [1] recommends using
        // clientWidth/clientHeight and CSS for resizing. I have not been able
        // to get this to work yet.
        //
        // [1] https://webglfundamentals.org/webgl/lessons/webgl-anti-patterns.html
        //let width = self.canvas.client_width().max(0) as u32;
        //let height = self.canvas.client_height().max(0) as u32;

        let window = web_sys::window().expect("Failed to obtain window");
        let width = window
            .inner_width()
            .expect("Failed to obtain innerWidth")
            .as_f64()
            .unwrap_or(640.0) as u32;
        let height = window
            .inner_height()
            .expect("Failed to obtain innerHeight")
            .as_f64()
            .unwrap_or(480.0) as u32;

        util::set_canvas_size(&self.canvas, Vector2::new(width, height));
    }

    pub fn screen(&self) -> ScreenGeom {
        ScreenGeom {
            size: Vector2::new(self.canvas.width(), self.canvas.height()),
            device_pixel_ratio: util::device_pixel_ratio(),
        }
    }

    pub fn debug_tex(&mut self, pos: Point2<f32>, tex: &Texture) -> Result<(), Error> {
        let screen = self.screen();
        let size = Vector2::new(tex.width() as f32, tex.height() as f32);

        // We initialize debug batches and shaders lazily, so that they don't
        // have any impact on startup time when not debugging.
        if self.debug_tex_batch.is_none() {
            self.debug_tex_batch = Some(TriBatch::new_golem(&self.golem_ctx)?);
        }
        if self.debug_tex_pass.is_none() {
            self.debug_tex_pass = Some(TexColPass::new_golem(&self.golem_ctx)?);
        }

        let debug_tex_batch = self.debug_tex_batch.as_mut().unwrap();
        let debug_tex_pass = self.debug_tex_pass.as_mut().unwrap();

        debug_tex_batch.clear();
        debug_tex_batch.push_quad(
            &AaRect::from_top_left(pos, size).into(),
            0.0,
            AaRect::zero_to_one(),
            Color4::new(0.0, 1.0, 1.0, 1.0),
        );
        debug_tex_pass.draw(
            &screen.orthographic_projection(),
            tex,
            &debug_tex_batch.draw_unit(),
        )?;

        Ok(())
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }
}

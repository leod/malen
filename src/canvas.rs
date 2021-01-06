use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use golem::{glow, GolemError, Texture};
use nalgebra::{Point2, Vector2};

use crate::input::EventHandlers;
use crate::{
    draw::{TexColPass, TexColVertex, TriBatch},
    util, AaRect, Color4, Error, Event, InputState, ScreenGeom,
};

pub struct Canvas {
    canvas: HtmlCanvasElement,
    golem_ctx: golem::Context,
    event_handlers: EventHandlers,
    input_state: InputState,

    debug_tex_batch: Option<TriBatch<TexColVertex>>,
    debug_tex_pass: Option<TexColPass>,
}

impl Canvas {
    pub fn from_element_id(id: &str) -> Result<Self, Error> {
        let canvas = web_sys::window()
            .ok_or(Error::NoWindow)?
            .document()
            .ok_or(Error::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| Error::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| Error::ElementIsNotCanvas(id.into()))?;

        Self::from_element(canvas)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_element(_: HtmlCanvasElement) -> Result<Self, Error> {
        // This is only in here as a workaround for the fact that Visual Studio
        // Code ignores our target setting in .cargo/config.toml for some
        // reason. Then, `glow::Context::from_webgl1_context` is not defined
        // and we lose e.g. inline error display.
        unreachable!("malen only works on web platforms")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_element(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let event_handlers = EventHandlers::new(canvas.clone())?;
        let input_state = InputState::default();

        let webgl_ctx = canvas
            .get_context("webgl")
            .map_err(|e| Error::GetContext(e.as_string().unwrap_or("error".into())))?
            .ok_or(Error::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| Error::InitializeWebGl)?;
        let glow_ctx = glow::Context::from_webgl1_context(webgl_ctx);
        let golem_ctx = golem::Context::from_glow(glow_ctx)?;

        // Make the canvas focusable.
        canvas.set_attribute("tabIndex", "1").unwrap();

        let mut canvas = Self {
            canvas,
            golem_ctx,
            event_handlers,
            input_state,
            debug_tex_batch: None,
            debug_tex_pass: None,
        };

        // Make sure that the canvas size is correct for the screen's DPI.
        canvas.resize(canvas.screen_geom().size);

        Ok(canvas)
    }

    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn resize(&self, logical_size: Vector2<u32>) {
        util::set_canvas_size(&self.canvas, logical_size);
        self.set_viewport(Point2::origin(), logical_size);
    }

    pub fn screen_geom(&self) -> ScreenGeom {
        ScreenGeom {
            size: Vector2::new(self.canvas.width(), self.canvas.height()),
            device_pixel_ratio: util::device_pixel_ratio(),
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        if let Some(event) = self.event_handlers.pop_event() {
            self.on_event(&event);

            Some(event)
        } else {
            None
        }
    }

    pub fn clear(&self, color: Color4) {
        self.golem_ctx
            .set_clear_color(color.r, color.g, color.b, color.a);
        self.golem_ctx.clear();
    }

    pub fn set_viewport(&self, lower_left: Point2<u32>, size: Vector2<u32>) {
        self.golem_ctx
            .set_viewport(lower_left.x, lower_left.y, size.x, size.y);
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

        self.resize(Vector2::new(width, height));
    }

    pub fn debug_tex(&mut self, pos: Point2<f32>, tex: &Texture) -> Result<(), Error> {
        let screen = self.screen_geom();
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

    fn on_event(&mut self, event: &Event) {
        self.input_state.on_event(event);
    }
}

impl From<GolemError> for Error {
    fn from(e: GolemError) -> Self {
        Error::Golem(e)
    }
}

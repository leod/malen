use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use glow::HasContext;
use golem::{GolemError, Texture};
use nalgebra::{Point2, Vector2};

use crate::input::EventHandlers;
use crate::{
    draw::{TexColPass, TexColVertex, TriBatch},
    error::CanvasInitError,
    gl, util, Color4, Error, Event, InputState, Rect, Screen,
};

#[derive(Debug, Clone)]
pub struct CanvasCaps {
    pub max_texture_size: u32,
}

impl CanvasCaps {
    fn new(gl: Rc<gl::Context>) -> Self {
        let max_texture_size = unsafe {
            (glow::MAX_TEXTURE_SIZE - 1).min(gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32)
        };

        CanvasCaps { max_texture_size }
    }
}

pub struct Canvas {
    html_element: HtmlCanvasElement,
    gl: Rc<gl::Context>,
    caps: CanvasCaps,
    golem_ctx: golem::Context,
    event_handlers: EventHandlers,
    input_state: InputState,
    logical_size: Vector2<u32>,

    debug_tex_batch: Option<TriBatch<TexColVertex>>,
    debug_tex_pass: Option<TexColPass>,
}

impl Canvas {
    pub fn from_element_id(id: &str) -> Result<Self, CanvasInitError> {
        let canvas = web_sys::window()
            .ok_or(CanvasInitError::NoWindow)?
            .document()
            .ok_or(CanvasInitError::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| CanvasInitError::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| CanvasInitError::ElementIsNotCanvas(id.into()))?;

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
    pub fn from_element(html_element: HtmlCanvasElement) -> Result<Self, CanvasInitError> {
        let event_handlers = EventHandlers::new(html_element.clone())?;
        let input_state = InputState::default();

        let webgl_context = html_element
            .get_context("webgl")
            .map_err(|e| CanvasInitError::GetContext(e.as_string().unwrap_or("error".into())))?
            .ok_or(CanvasInitError::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| CanvasInitError::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let gl = Rc::new(gl::Context::new(glow_context));
        let caps = CanvasCaps::new(gl.clone());

        let webgl_context_tmp = html_element
            .get_context("webgl")
            .map_err(|e| CanvasInitError::GetContext(e.as_string().unwrap_or("error".into())))?
            .ok_or(CanvasInitError::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| CanvasInitError::InitializeWebGl)?;
        let glow_context_tmp = golem::glow::Context::from_webgl1_context(webgl_context_tmp);
        let golem_ctx =
            golem::Context::from_glow(glow_context_tmp).map_err(CanvasInitError::Golem)?;

        // Make the canvas focusable.
        html_element.set_attribute("tabIndex", "1").unwrap();

        let logical_size = Vector2::new(html_element.width(), html_element.height());

        let mut canvas = Self {
            html_element,
            gl,
            caps,
            golem_ctx,
            event_handlers,
            input_state,
            logical_size,
            debug_tex_batch: None,
            debug_tex_pass: None,
        };

        // Make sure that the physical canvas size is correct (adjusting for the
        // screen's DPI).
        canvas.resize(logical_size);

        Ok(canvas)
    }

    pub fn html_element(&self) -> HtmlCanvasElement {
        self.html_element.clone()
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.gl.clone()
    }

    pub fn caps(&self) -> &CanvasCaps {
        &self.caps
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        &self.golem_ctx
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn screen(&self) -> Screen {
        Screen {
            logical_size: self.logical_size,
            physical_size: Vector2::new(self.html_element.width(), self.html_element.height()),
            device_pixel_ratio: util::device_pixel_ratio(),
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.resize(self.logical_size);

        if let Some(event) = self.event_handlers.pop_event() {
            self.on_event(&event);

            Some(event)
        } else {
            None
        }
    }

    pub fn has_focus(&self) -> bool {
        let active_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .active_element()
            .unwrap();

        &active_element == AsRef::<web_sys::Element>::as_ref(&self.html_element)
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

    pub fn resize(&mut self, logical_size: Vector2<u32>) {
        util::set_canvas_size(&self.html_element, logical_size);
        self.set_viewport(Point2::origin(), self.screen().physical_size);
        self.logical_size = logical_size;
    }

    pub fn resize_full(&mut self) {
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

    pub fn debug_tex(
        &mut self,
        pos: Point2<f32>,
        size: Vector2<f32>,
        tex: &Texture,
    ) -> Result<(), Error> {
        let screen = self.screen();

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
            &Rect::from_top_left(pos, size).into(),
            0.0,
            Rect::zero_to_one(),
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

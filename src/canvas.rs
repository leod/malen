use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use glow::HasContext;
use nalgebra::{Point2, Vector2};

use crate::input::EventHandlers;
use crate::{error::InitError, gl, util, Color4, Event, Screen};

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
    element: HtmlCanvasElement,
    gl: Rc<gl::Context>,
    caps: CanvasCaps,
    event_handlers: EventHandlers,
    logical_size: Vector2<u32>,
}

impl Canvas {
    pub fn from_element_id(id: &str) -> Result<Self, InitError> {
        let canvas = web_sys::window()
            .ok_or(InitError::NoWindow)?
            .document()
            .ok_or(InitError::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| InitError::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| InitError::ElementIsNotCanvas(id.into()))?;

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
    pub fn from_element(element: HtmlCanvasElement) -> Result<Self, InitError> {
        let event_handlers = EventHandlers::new(element.clone())?;

        let webgl_context = element
            .get_context("webgl2")
            .map_err(|e| InitError::GetContext(e.as_string().unwrap_or("error".into())))?
            .ok_or(InitError::InitializeWebGl)?
            .dyn_into::<WebGl2RenderingContext>()
            .map_err(|_| InitError::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl2_context(webgl_context);
        let gl = Rc::new(gl::Context::new(glow_context));
        let caps = CanvasCaps::new(gl.clone());

        // Make the canvas focusable.
        element.set_attribute("tabIndex", "1").unwrap();

        let logical_size = Vector2::new(element.width(), element.height());

        let mut canvas = Self {
            element,
            gl,
            caps,
            event_handlers,
            logical_size,
        };

        // Make sure that the physical canvas size is correct (adjusting for the
        // screen's DPI).
        canvas.resize(logical_size);

        Ok(canvas)
    }

    pub fn element(&self) -> HtmlCanvasElement {
        self.element.clone()
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.gl.clone()
    }

    pub fn caps(&self) -> &CanvasCaps {
        &self.caps
    }

    pub fn screen(&self) -> Screen {
        Screen {
            logical_size: self.logical_size,
            physical_size: Vector2::new(self.element.width(), self.element.height()),
            device_pixel_ratio: util::device_pixel_ratio(),
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        // FIXME: Resize handling
        self.resize(self.logical_size);

        self.event_handlers.pop_event()
    }

    pub fn has_focus(&self) -> bool {
        let active_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .active_element()
            .unwrap();

        &active_element == AsRef::<web_sys::Element>::as_ref(&self.element)
    }

    pub fn clear(&self, color: Color4) {
        //unimplemented!();
    }

    pub fn set_viewport(&self, lower_left: Point2<u32>, size: Vector2<u32>) {
        //unimplemented!();
    }

    pub fn resize(&mut self, logical_size: Vector2<u32>) {
        util::set_canvas_size(&self.element, logical_size);
        self.set_viewport(Point2::origin(), self.screen().physical_size);
        self.logical_size = logical_size;
    }

    pub fn resize_fill(&mut self) {
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
}

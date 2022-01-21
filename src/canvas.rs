use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlContextAttributes};

use glow::HasContext;
use nalgebra::{Point2, Vector2};

use crate::input::EventHandlers;
use crate::{error::InitError, geom::Screen, gl, util, Color4, Event};

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

#[derive(Debug, Clone)]
pub enum CanvasSizeConfig {
    LogicalSize(Vector2<u32>),
    Fill,
}

pub struct Canvas {
    element: HtmlCanvasElement,
    gl: Rc<gl::Context>,
    caps: CanvasCaps,
    event_handlers: EventHandlers,
    logical_size: Vector2<f32>,
}

impl Canvas {
    pub fn from_element_id(id: &str, size_config: CanvasSizeConfig) -> Result<Self, InitError> {
        let canvas = web_sys::window()
            .ok_or(InitError::NoWindow)?
            .document()
            .ok_or(InitError::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| InitError::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| InitError::ElementIsNotCanvas(id.into()))?;

        Self::from_element(canvas, size_config)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_element(_: HtmlCanvasElement, _: CanvasSize) -> Result<Self, Error> {
        // This is only in here as a workaround for the fact that Visual Studio
        // Code ignores our target setting in .cargo/config.toml for some
        // reason. Then, `glow::Context::from_webgl1_context` is not defined
        // and we lose e.g. inline error display.
        unreachable!("malen only works on web platforms")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_element(
        element: HtmlCanvasElement,
        size_config: CanvasSizeConfig,
    ) -> Result<Self, InitError> {
        use web_sys::WebGlPowerPreference;

        let event_handlers = EventHandlers::new(element.clone())?;

        let mut context_attributes = WebGlContextAttributes::new();
        context_attributes.alpha(false);
        context_attributes.depth(true);
        context_attributes.stencil(true);
        context_attributes.antialias(false);
        context_attributes.power_preference(WebGlPowerPreference::HighPerformance);
        context_attributes.preserve_drawing_buffer(false);
        context_attributes.xr_compatible(false);

        let webgl_context = element
            .get_context_with_context_options("webgl2", &context_attributes.into())
            .map_err(|e| InitError::GetContext(e.as_string().unwrap_or_else(|| "error".into())))?
            .ok_or(InitError::InitializeWebGl)?
            .dyn_into::<WebGl2RenderingContext>()
            .map_err(|_| InitError::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl2_context(webgl_context);
        let gl = Rc::new(gl::Context::new(glow_context));
        let caps = CanvasCaps::new(gl.clone());

        util::make_canvas_focusable(&element);

        // Prevent scrollbar.
        util::set_canvas_style_property(&element, "display", "block");

        let initial_logical_size = Vector2::new(element.width(), element.height());

        let mut canvas = Self {
            element,
            gl,
            caps,
            event_handlers,
            logical_size: nalgebra::convert(initial_logical_size),
        };

        canvas.set_size_config(size_config);

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

    pub fn logical_size(&self) -> Vector2<f32> {
        self.logical_size
    }

    pub fn physical_size(&self) -> Vector2<u32> {
        Vector2::new(self.element.width(), self.element.height())
    }

    pub fn screen(&self) -> Screen {
        Screen {
            logical_size: self.logical_size,
            physical_size: self.physical_size(),
            device_pixel_ratio: util::device_pixel_ratio(),
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.adjust_sizes();

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

    pub fn clear_color_and_depth(&self, color: Color4, depth: f32) {
        gl::clear_color_and_depth(&*self.gl, color, depth);
    }

    pub fn clear_color(&self, color: Color4) {
        gl::clear_color(&*self.gl, color);
    }

    pub fn clear_depth(&self, depth: f32) {
        gl::clear_depth(&*self.gl, depth);
    }

    pub fn set_size_config(&mut self, size_config: CanvasSizeConfig) {
        match size_config {
            CanvasSizeConfig::LogicalSize(logical_size) => {
                util::set_canvas_logical_size(&self.element, logical_size);
                util::set_canvas_physical_size(
                    &self.element,
                    util::logical_to_physical_size(logical_size),
                );

                self.logical_size = nalgebra::convert(logical_size);

                log::info!(
                    "Set CanvasSizeConfig::LogicalSize [logical_size={}, physical_size={}]",
                    self.logical_size,
                    self.screen().physical_size
                );
            }
            CanvasSizeConfig::Fill => {
                util::set_canvas_logical_size_fill(&self.element);

                self.adjust_sizes();

                log::info!(
                    "Set CanvasSizeConfig::Fill [logical_size={}, physical_size={}]",
                    self.logical_size,
                    self.screen().physical_size
                );
            }
        }

        self.set_viewport(Point2::origin(), self.screen().physical_size);
    }

    fn set_viewport(&self, lower_left: Point2<u32>, size: Vector2<u32>) {
        unsafe {
            self.gl.viewport(
                lower_left.x as i32,
                lower_left.y as i32,
                size.x as i32,
                size.y as i32,
            );
        }
    }

    fn adjust_sizes(&mut self) {
        // https://webgl2fundamentals.org/webgl/lessons/webgl-resizing-the-canvas.html

        let device_pixel_ratio = util::device_pixel_ratio();
        let bounding_rect = self.element.get_bounding_client_rect();

        let logical_size = Vector2::new(bounding_rect.width(), bounding_rect.height());
        let physical_size = Vector2::new(
            (logical_size.x * device_pixel_ratio).round() as u32,
            (logical_size.y * device_pixel_ratio).round() as u32,
        );
        self.logical_size = logical_size.cast::<f32>();

        let need_resize =
            self.element.width() != physical_size.x || self.element.height() != physical_size.y;

        if need_resize {
            util::set_canvas_physical_size(&self.element, physical_size);
            self.set_viewport(Point2::origin(), physical_size);

            log::info!(
                "Resized canvas physical size [logical_size={}, physical_size={}]",
                self.logical_size,
                physical_size,
            );
        }
    }
}

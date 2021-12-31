use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::{error::InitError, gl, input::InputState, Canvas};

pub struct Context {
    canvas: Canvas,
    input_state: InputState,
}

impl Context {
    pub fn from_canvas_element_id(id: &str) -> Result<Self, InitError> {
        Self::from_canvas(Canvas::from_element_id(id)?)
    }

    pub fn from_canvas_element(html_element: HtmlCanvasElement) -> Result<Self, InitError> {
        Self::from_canvas(Canvas::from_element(html_element)?)
    }

    pub fn from_canvas(canvas: Canvas) -> Result<Self, InitError> {
        let input_state = InputState::default();

        Ok(Context {
            canvas,
            input_state,
        })
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn gl(&self) -> Rc<gl::Context> {
        self.canvas.gl()
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }
}

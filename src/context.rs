use std::{cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use golem::{glow, GolemError};
use nalgebra as na;

use crate::input::EventHandlers;
use crate::{Error, Event, Matrix3, Vector2};

#[derive(Debug, Clone)]
pub struct Screen {
    /// The screen size in pixels.
    pub size: Vector2,
}

impl Screen {
    /// Returns an orthographic projection matrix.
    ///
    /// The returned matrix maps `[0..width] x [0..height]` to
    /// `[-1..1] x [-1..1]` (i.e. the OpenGL normalized device coordinates).
    ///
    /// Notes:
    /// - This projection also flips the Y axis, so that (0,0) is at the
    ///   top-left of your screen.
    /// - We assume the Z coordinate of the input vector to be set to 1.
    pub fn orthographic_projection(&self) -> Matrix3 {
        let scale_to_unit = na::Matrix3::new_nonuniform_scaling(&Vector2::new(
            1.0 / self.size.x,
            1.0 / self.size.y,
        ));
        let shift = na::Matrix3::new_translation(&Vector2::new(-0.5, -0.5));
        let scale_and_flip_y = na::Matrix3::new_nonuniform_scaling(&Vector2::new(2.0, -2.0));

        scale_and_flip_y * shift * scale_to_unit
    }
}

pub struct Context {
    canvas: HtmlCanvasElement,
    event_handlers: EventHandlers,
    golem_context: golem::Context,
}

impl Context {
    pub fn from_canvas_id(id: &str) -> Result<Self, Error> {
        let canvas = web_sys::window()
            .ok_or(Error::NoWindow)?
            .document()
            .ok_or(Error::NoDocument)?
            .get_element_by_id(id)
            .ok_or_else(|| Error::InvalidElementId(id.into()))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| Error::ElementIsNotCanvas(id.into()))?;

        Self::from_canvas_element(canvas)
    }

    pub fn from_canvas_element(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let event_handlers = EventHandlers::new(canvas.clone())?;
        let webgl_context = canvas
            .get_context("webgl")
            .map_err(Error::GetContext)?
            .ok_or(Error::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| Error::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let golem_context = golem::Context::from_glow(glow_context)?;

        Ok(Context {
            canvas,
            event_handlers,
            golem_context,
        })
    }

    pub fn screen(&self) -> Screen {
        Screen {
            size: Vector2::new(self.canvas.width() as f32, self.canvas.height() as f32),
        }
    }

    pub fn golem_context(&self) -> &golem::Context {
        &self.golem_context
    }

    /// Run the `webglee` main loop.
    ///
    /// The callback is called once per frame, and it is passed the following
    /// arguments:
    /// 1. A reference to the webglee `Context`. The context can be used to
    ///    draw things.
    /// 2. The time that has elapsed since the last frame.
    /// 3. The input events that occured since the last frame.
    /// 4. A boolean which can be set to true in order to terminate the main
    ///    loop.
    ///
    /// The callback should be used to do the following things:
    /// - Handle input events.
    /// - Update the game state, relying on the elapsed time as given to the
    ///   callback.
    ///
    ///   We recommend that you do *not* do your own time measurements for delta
    ///   time, since the time that most browsers give us with e.g.
    ///   [`performance.now()`](https://developer.mozilla.org/en-US/docs/Web/API/Performance/now)
    ///   is limited in resolution to mitigate potential security threats.
    /// - Render the game.
    pub fn main_loop<F>(self, mut callback: F) -> Result<(), Error>
    where
        F: FnMut(&mut Context, Duration, &[Event], &mut bool) + 'static,
    {
        // Source:
        // https://github.com/grovesNL/glow/blob/2d42c5b105d979efe764191b5b1ce78fab99ffcf/src/web_sys.rs#L3258

        fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
            web_sys::window()
                .unwrap()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .unwrap();
        }

        let f = Rc::new(RefCell::new(None));

        let mut last_timestamp = None;
        let mut running = true;
        let mut context = self;

        *f.borrow_mut() = Some(Closure::wrap(Box::new({
            let f = f.clone();

            move |timestamp: f64| {
                let dt = last_timestamp.map_or(Duration::from_secs(0), |last_timestamp: f64| {
                    let dt_ms = (timestamp - last_timestamp).max(0.0);
                    Duration::from_secs_f64(dt_ms / 1000.0)
                });
                last_timestamp = Some(timestamp);

                let events = context.event_handlers.take_events();
                callback(&mut context, dt, &events, &mut running);
                if !running {
                    let _ = f.borrow_mut().take();
                    return;
                }
                request_animation_frame(f.borrow().as_ref().unwrap());
            }
        }) as Box<dyn FnMut(f64)>));

        request_animation_frame(f.borrow().as_ref().unwrap());

        Ok(())
    }
}

impl From<GolemError> for Error {
    fn from(e: GolemError) -> Self {
        Error::Golem(e)
    }
}

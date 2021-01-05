use std::{cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use golem::{glow, GolemError, Texture};
use nalgebra::{Point2, Vector2};

use crate::input::EventHandlers;
use crate::{Draw, Error, Event, InputState};

pub struct Context {
    event_handlers: EventHandlers,
    input_state: InputState,
    draw: Draw,
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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_canvas_element(_: HtmlCanvasElement) -> Result<Self, Error> {
        // This is only in here as a workaround for the fact that Visual Studio
        // Code ignores our target setting in .cargo/config.toml for some
        // reason. Then, `glow::Context::from_webgl1_context` is not defined
        // and we lose e.g. inline error display.
        unreachable!("malen only works on web platforms")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas_element(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let event_handlers = EventHandlers::new(canvas.clone())?;
        let input_state = InputState::default();
        let webgl_context = canvas
            .get_context("webgl")
            .map_err(|e| Error::GetContext(e.as_string().unwrap_or("error".into())))?
            .ok_or(Error::InitializeWebGl)?
            .dyn_into::<WebGlRenderingContext>()
            .map_err(|_| Error::InitializeWebGl)?;
        let glow_context = glow::Context::from_webgl1_context(webgl_context);
        let golem_context = golem::Context::from_glow(glow_context)?;
        let draw = Draw::new(canvas, golem_context)?;

        Ok(Context {
            event_handlers,
            input_state,
            draw,
        })
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn draw(&self) -> &Draw {
        &self.draw
    }

    pub fn draw_mut(&mut self) -> &mut Draw {
        &mut self.draw
    }

    pub fn golem_ctx(&self) -> &golem::Context {
        self.draw.golem_ctx()
    }

    pub fn debug_tex(&mut self, pos: Point2<f32>, tex: &Texture) -> Result<(), Error> {
        self.draw.debug_tex(pos, tex)
    }

    pub fn resize(&self, logical_size: Vector2<u32>) {
        self.draw.resize(logical_size);
    }

    pub fn resize_full(&self) {
        self.draw.resize_full();
    }

    /// Run the `malen` main loop.
    ///
    /// The callback is called once per frame, and it is passed the following
    /// arguments:
    /// 1. A reference to the malen `Context`. The context can be used to
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

                for event in &events {
                    context.input_state.on_event(event);
                }

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

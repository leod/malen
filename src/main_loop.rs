use std::{cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};

use crate::Error;

/// Run the `malen` main loop.
///
/// The callback is called once per frame, and it is passed the following
/// arguments:
/// 1. The time that has elapsed since the last frame.
/// 2. A boolean which can be set to true in order to terminate the main
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
pub fn main_loop<F>(mut callback: F) -> Result<(), Error>
where
    F: FnMut(Duration, &mut bool) + 'static,
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

    *f.borrow_mut() = Some(Closure::wrap(Box::new({
        let f = f.clone();

        move |timestamp: f64| {
            let dt = last_timestamp.map_or(Duration::from_secs(0), |last_timestamp: f64| {
                let dt_ms = (timestamp - last_timestamp).max(0.0);
                let dt_secs = dt_ms / 1000.0;

                Duration::from_secs_f64(dt_secs)
            });
            last_timestamp = Some(timestamp);

            callback(dt, &mut running);

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

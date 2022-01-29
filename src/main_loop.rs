use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast};

/// Run the `malen` main loop.
///
/// The callback is called once per frame, and it is passed the following
/// arguments:
/// 1. The time that has elapsed since the last frame.
/// 2. A boolean which can be set to true in order to terminate the main
///    loop.
///
/// The callback should be used to do the following things:
/// - Consume input events.
/// - Update the game state.
/// - Render the game.
///
/// For updating the game state, we recommend that you do *not* do your own time
/// measurements for delta time, since the time that most browsers give us with
/// e.g.
/// [`performance.now()`](https://developer.mozilla.org/en-US/docs/Web/API/Performance/now)
/// is limited in resolution to mitigate potential security threats.
pub fn main_loop<F>(mut callback: F)
where
    F: FnMut(f64, &mut bool) + 'static,
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

    #[cfg(feature = "coarse-prof")]
    let guard = Rc::new(RefCell::new(None));

    let mut running = true;

    *f.borrow_mut() = Some(Closure::wrap(Box::new({
        let f = f.clone();

        move |timestamp_millis: f64| {
            {
                #[cfg(feature = "coarse-prof")]
                {
                    *guard.borrow_mut() = None;
                }

                #[cfg(feature = "coarse-prof")]
                coarse_prof::profile!("animation_frame");

                callback(timestamp_millis / 1000.0f64, &mut running);

                if !running {
                    let _ = f.borrow_mut().take();
                    return;
                }

                request_animation_frame(f.borrow().as_ref().unwrap());
            }

            #[cfg(feature = "coarse-prof")]
            {
                *guard.borrow_mut() = Some(coarse_prof::enter("other"));
            }
        }
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(f.borrow().as_ref().unwrap());
}

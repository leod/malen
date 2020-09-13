use std::{
    cell::RefCell,
    rc::Rc,
    time::Duration,
};

use wasm_bindgen::{closure::Closure, JsCast};

pub fn main_loop(mut callback: impl FnMut(Duration, &mut bool) + 'static) {
    // Source:
    // https://github.com/grovesNL/glow/blob/2d42c5b105d979efe764191b5b1ce78fab99ffcf/src/web_sys.rs#L3258

    fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut last_timestamp = None;
    let mut running = true;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let dt = last_timestamp.map_or(Duration::from_secs(0), |last_timestamp: f64| {
            let dt_ms = (timestamp - last_timestamp).max(0.0);
            Duration::from_secs_f64(dt_ms / 1000.0)
        });
        last_timestamp = Some(timestamp);

        callback(dt, &mut running);
        if !running {
            let _ = f.borrow_mut().take();
            return;
        }
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

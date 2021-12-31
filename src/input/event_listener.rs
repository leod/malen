use wasm_bindgen::{closure::Closure, convert::FromWasmAbi, JsCast};

/// Event handlers with automatic clean up, inspired by
/// <https://github.com/rustwasm/gloo/issues/30>.
pub struct EventListener<T> {
    element: web_sys::EventTarget,
    kind: &'static str,
    callback: Closure<dyn FnMut(T)>,
}

impl<T> EventListener<T>
where
    T: 'static + AsRef<web_sys::Event> + FromWasmAbi,
{
    pub fn new<F>(element: &web_sys::EventTarget, kind: &'static str, f: F) -> Self
    where
        F: 'static + FnMut(T),
    {
        let callback = Closure::wrap(Box::new(f) as Box<dyn FnMut(T)>);

        element
            .add_event_listener_with_callback(kind, &callback.as_ref().unchecked_ref())
            .expect(&format!("Failed to add event listener for kind {}", kind));

        Self {
            element: element.clone(),
            kind,
            callback,
        }
    }

    pub fn new_consume<F>(element: &web_sys::EventTarget, kind: &'static str, mut f: F) -> Self
    where
        F: 'static + FnMut(T),
    {
        Self::new(element, kind, move |event| {
            {
                let event_ref = event.as_ref();
                event_ref.stop_propagation();
                event_ref.cancel_bubble();
            }

            f(event);
        })
    }
}

impl<T> Drop for EventListener<T> {
    fn drop(&mut self) {
        self.element
            .remove_event_listener_with_callback(self.kind, self.callback.as_ref().unchecked_ref())
            .expect(&format!(
                "Failed to remove event listener for kind {}",
                self.kind
            ));
    }
}

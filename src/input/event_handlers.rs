use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use nalgebra::Point2;
use web_sys::{FocusEvent, HtmlCanvasElement, KeyboardEvent, MouseEvent};

use crate::error::InitError;

use super::{Button, Event, EventListener, Key};

#[derive(Default, Clone)]
struct SharedState {
    events: VecDeque<Event>,
}

pub struct EventHandlers {
    state: Rc<RefCell<SharedState>>,

    _on_focus: EventListener<FocusEvent>,
    _on_blur: EventListener<FocusEvent>,
    _on_context_menu: EventListener<MouseEvent>,
    _on_key_down: EventListener<KeyboardEvent>,
    _on_key_release: EventListener<KeyboardEvent>,
    _on_mouse_down: EventListener<MouseEvent>,
    _on_mouse_release: EventListener<MouseEvent>,
    _on_mouse_move: EventListener<MouseEvent>,
}

impl EventHandlers {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, InitError> {
        let state = Rc::new(RefCell::new(SharedState::default()));

        let on_focus = EventListener::new_consuming(&canvas, "focus", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.events.push_back(Event::Focused);
            }
        });

        let on_blur = EventListener::new_consuming(&canvas, "blur", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.events.push_back(Event::Unfocused);
            }
        });

        let on_context_menu = EventListener::new_consuming(&canvas, "contextmenu", {
            let state = state.clone();
            move |_: MouseEvent| {
                let mut state = state.borrow_mut();
                state.events.push_back(Event::Unfocused);
            }
        });

        let on_key_down = EventListener::new_consuming(&canvas, "keydown", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if event.repeat() {
                    return;
                }

                if let Some(key) = Key::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyPressed(key));
                }
            }
        });

        let on_key_release = EventListener::new_consuming(&canvas, "keyup", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if let Some(key) = Key::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyReleased(key));
                }
            }
        });

        let on_mouse_down = EventListener::new_consuming(&canvas, "mousedown", {
            let state = state.clone();
            move |event: MouseEvent| {
                if let Some(button) = Button::from_mouse_event(&event) {
                    state
                        .borrow_mut()
                        .events
                        .push_back(Event::MousePressed(button));
                }
            }
        });

        let on_mouse_release = EventListener::new_consuming(&canvas, "mouseup", {
            let state = state.clone();
            move |event: MouseEvent| {
                if let Some(button) = Button::from_mouse_event(&event) {
                    state
                        .borrow_mut()
                        .events
                        .push_back(Event::MouseReleased(button));
                }
            }
        });

        let on_mouse_move = EventListener::new_consuming(&canvas, "mousemove", {
            let state = state.clone();
            let canvas = canvas.clone();
            move |event: MouseEvent| {
                // https://stackoverflow.com/a/42315942
                let bounding_rect = canvas.get_bounding_client_rect();
                let logical_pos = Point2::new(
                    event.client_x() as f64 - bounding_rect.left(),
                    event.client_y() as f64 - bounding_rect.top(),
                );
                state
                    .borrow_mut()
                    .events
                    .push_back(Event::MouseMoved(logical_pos));
            }
        });

        Ok(Self {
            state,
            _on_focus: on_focus,
            _on_blur: on_blur,
            _on_context_menu: on_context_menu,
            _on_key_down: on_key_down,
            _on_key_release: on_key_release,
            _on_mouse_down: on_mouse_down,
            _on_mouse_release: on_mouse_release,
            _on_mouse_move: on_mouse_move,
        })
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.state.borrow_mut().events.pop_front()
    }
}

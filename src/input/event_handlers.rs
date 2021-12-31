use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use web_sys::{FocusEvent, HtmlCanvasElement, KeyboardEvent};

use crate::error::InitError;

use super::{Event, EventListener, Key};

#[derive(Default, Debug, Clone)]
struct SharedState {
    events: VecDeque<Event>,
}

pub struct EventHandlers {
    state: Rc<RefCell<SharedState>>,

    _on_focus: EventListener<FocusEvent>,
    _on_blur: EventListener<FocusEvent>,
    _on_key_down: EventListener<KeyboardEvent>,
    _on_key_release: EventListener<KeyboardEvent>,
}

impl EventHandlers {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, InitError> {
        let state = Rc::new(RefCell::new(SharedState::default()));

        let on_focus = EventListener::new_consume(&canvas, "focus", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.events.push_back(Event::Focused);
            }
        });

        let on_blur = EventListener::new_consume(&canvas, "blur", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.events.push_back(Event::Unfocused);
            }
        });

        let on_key_down = EventListener::new_consume(&canvas, "keydown", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if let Some(key) = Key::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyPressed(key));
                }
            }
        });

        let on_key_release = EventListener::new_consume(&canvas, "keyup", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if let Some(key) = Key::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyReleased(key));
                }
            }
        });

        Ok(Self {
            state,
            _on_focus: on_focus,
            _on_blur: on_blur,
            _on_key_down: on_key_down,
            _on_key_release: on_key_release,
        })
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.state.borrow_mut().events.pop_front()
    }
}

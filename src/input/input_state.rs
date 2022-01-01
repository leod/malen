use std::collections::BTreeSet;

use nalgebra::Point2;

use super::{Event, Key};

#[derive(Debug, Clone)]
pub struct InputState {
    pressed_keys: BTreeSet<Key>,
    mouse_logical_pos: Point2<f64>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed_keys: BTreeSet::new(),
            mouse_logical_pos: Point2::origin(),
        }
    }
}

impl InputState {
    pub(crate) fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Unfocused => {
                self.pressed_keys.clear();
            }
            Event::KeyPressed(key) => {
                self.pressed_keys.insert(*key);
            }
            Event::KeyReleased(key) => {
                self.pressed_keys.remove(key);
            }
            Event::MouseMoved(logical_pos) => {
                self.mouse_logical_pos = *logical_pos;
            }
            _ => (),
        }
    }

    pub fn key(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn pressed_keys(&self) -> &BTreeSet<Key> {
        &self.pressed_keys
    }

    pub fn mouse_logical_pos(&self) -> Point2<f64> {
        self.mouse_logical_pos
    }
}

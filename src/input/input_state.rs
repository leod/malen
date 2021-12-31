use std::collections::BTreeSet;

use super::{Event, Key};

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pressed_keys: BTreeSet<Key>,
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
            _ => (),
        }
    }

    pub fn key(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn pressed_keys(&self) -> &BTreeSet<Key> {
        &self.pressed_keys
    }
}

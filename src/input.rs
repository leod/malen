use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    rc::Rc,
};

use wasm_bindgen::{closure::Closure, convert::FromWasmAbi, JsCast};
use web_sys::{FocusEvent, HtmlCanvasElement, KeyboardEvent};

use nalgebra as na;

use crate::Error;

#[derive(Debug, Clone)]
pub enum Event {
    Focused,
    Unfocused,
    KeyPressed(VirtualKeyCode),
    KeyReleased(VirtualKeyCode),
    WindowResized(na::Vector2<f64>),
}

type EventHandler<T> = Closure<dyn FnMut(T)>;

#[derive(Debug, Clone, Default)]
pub struct KeysState {
    pressed_keys: BTreeSet<VirtualKeyCode>,
}

impl KeysState {
    pub fn on_event(&mut self, event: &Event) {
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
}

#[derive(Default, Debug, Clone)]
struct State {
    is_focused: bool,
    events: VecDeque<Event>,
}

pub struct Input {
    state: Rc<RefCell<State>>,
    _on_focus: EventHandler<FocusEvent>,
    _on_blur: EventHandler<FocusEvent>,
    _on_key_down: EventHandler<KeyboardEvent>,
    _on_key_release: EventHandler<KeyboardEvent>,
    _on_resize: EventHandler<web_sys::Event>,
}

impl Input {
    pub fn new(_canvas: &HtmlCanvasElement) -> Result<Self, Error> {
        let state = Rc::new(RefCell::new(State::default()));

        let window = web_sys::window().ok_or(Error::NoWindow)?;

        let on_focus = set_handler(&window, "focus", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.is_focused = true;
                state.events.push_back(Event::Focused);
            }
        });

        let on_blur = set_handler(&window, "blur", {
            let state = state.clone();
            move |_: FocusEvent| {
                let mut state = state.borrow_mut();
                state.is_focused = false;
                state.events.push_back(Event::Unfocused);
            }
        });

        let on_key_down = set_handler(&window, "keydown", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if let Some(key) = VirtualKeyCode::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyPressed(key));
                }
            }
        });

        let on_key_release = set_handler(&window, "keyrelease", {
            let state = state.clone();
            move |event: KeyboardEvent| {
                if let Some(key) = VirtualKeyCode::from_keyboard_event(&event) {
                    state.borrow_mut().events.push_back(Event::KeyReleased(key));
                }
            }
        });

        let on_resize = set_handler(&window.clone(), "resize", {
            let state = state.clone();
            move |_| {
                let width = window.inner_width().map(|w| w.as_f64());
                let height = window.inner_height().map(|w| w.as_f64());
                if let (Ok(Some(width)), Ok(Some(height))) = (&width, &height) {
                    state
                        .borrow_mut()
                        .events
                        .push_back(Event::WindowResized(na::Vector2::new(*width, *height)));
                } else {
                    log::warn!(
                        "Failed to read innerWidth/innerHeight from window. Got: {:?}, {:?}",
                        width,
                        height
                    );
                }
            }
        });

        Ok(Self {
            state,
            _on_focus: on_focus,
            _on_blur: on_blur,
            _on_key_down: on_key_down,
            _on_key_release: on_key_release,
            _on_resize: on_resize,
        })
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.state.borrow_mut().events.pop_front()
    }
}

fn set_handler<T, E, F>(target: T, event_name: &str, mut handler: F) -> EventHandler<E>
where
    T: AsRef<web_sys::EventTarget>,
    E: 'static + AsRef<web_sys::Event> + FromWasmAbi,
    F: 'static + FnMut(E),
{
    // Source:
    // https://github.com/rust-windowing/winit/blob/e4754999b7e7f27786092a62eda5275672d74130/src/platform_impl/web/web_sys/canvas.rs#L295

    let closure = Closure::wrap(Box::new(move |event: E| {
        {
            let event_ref = event.as_ref();
            event_ref.stop_propagation();
            event_ref.cancel_bubble();
        }

        handler(event);
    }) as Box<dyn FnMut(E)>);

    target
        .as_ref()
        .add_event_listener_with_callback(event_name, &closure.as_ref().unchecked_ref())
        .expect("Failed to add event listener with callback");

    closure
}

/// A key that can be pressed.
///
/// This enum has been copied almost exactly from winit.
/// Source: https://github.com/rust-windowing/winit/blob/a2db4c0a320aafc10d240c432fe5ef4e4d84629d/src/event.rs#L774
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VirtualKeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,  // also called "Prior"
    NavigateBackward, // also called "Next"
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl VirtualKeyCode {
    pub fn from_keyboard_event(event: &KeyboardEvent) -> Option<Self> {
        // Source:
        // https://github.com/rust-windowing/winit/blob/e4754999b7e7f27786092a62eda5275672d74130/src/platform_impl/web/web_sys/event.rs#L64
        Some(match &event.code()[..] {
            "Digit1" => VirtualKeyCode::Key1,
            "Digit2" => VirtualKeyCode::Key2,
            "Digit3" => VirtualKeyCode::Key3,
            "Digit4" => VirtualKeyCode::Key4,
            "Digit5" => VirtualKeyCode::Key5,
            "Digit6" => VirtualKeyCode::Key6,
            "Digit7" => VirtualKeyCode::Key7,
            "Digit8" => VirtualKeyCode::Key8,
            "Digit9" => VirtualKeyCode::Key9,
            "Digit0" => VirtualKeyCode::Key0,
            "KeyA" => VirtualKeyCode::A,
            "KeyB" => VirtualKeyCode::B,
            "KeyC" => VirtualKeyCode::C,
            "KeyD" => VirtualKeyCode::D,
            "KeyE" => VirtualKeyCode::E,
            "KeyF" => VirtualKeyCode::F,
            "KeyG" => VirtualKeyCode::G,
            "KeyH" => VirtualKeyCode::H,
            "KeyI" => VirtualKeyCode::I,
            "KeyJ" => VirtualKeyCode::J,
            "KeyK" => VirtualKeyCode::K,
            "KeyL" => VirtualKeyCode::L,
            "KeyM" => VirtualKeyCode::M,
            "KeyN" => VirtualKeyCode::N,
            "KeyO" => VirtualKeyCode::O,
            "KeyP" => VirtualKeyCode::P,
            "KeyQ" => VirtualKeyCode::Q,
            "KeyR" => VirtualKeyCode::R,
            "KeyS" => VirtualKeyCode::S,
            "KeyT" => VirtualKeyCode::T,
            "KeyU" => VirtualKeyCode::U,
            "KeyV" => VirtualKeyCode::V,
            "KeyW" => VirtualKeyCode::W,
            "KeyX" => VirtualKeyCode::X,
            "KeyY" => VirtualKeyCode::Y,
            "KeyZ" => VirtualKeyCode::Z,
            "Escape" => VirtualKeyCode::Escape,
            "F1" => VirtualKeyCode::F1,
            "F2" => VirtualKeyCode::F2,
            "F3" => VirtualKeyCode::F3,
            "F4" => VirtualKeyCode::F4,
            "F5" => VirtualKeyCode::F5,
            "F6" => VirtualKeyCode::F6,
            "F7" => VirtualKeyCode::F7,
            "F8" => VirtualKeyCode::F8,
            "F9" => VirtualKeyCode::F9,
            "F10" => VirtualKeyCode::F10,
            "F11" => VirtualKeyCode::F11,
            "F12" => VirtualKeyCode::F12,
            "F13" => VirtualKeyCode::F13,
            "F14" => VirtualKeyCode::F14,
            "F15" => VirtualKeyCode::F15,
            "F16" => VirtualKeyCode::F16,
            "F17" => VirtualKeyCode::F17,
            "F18" => VirtualKeyCode::F18,
            "F19" => VirtualKeyCode::F19,
            "F20" => VirtualKeyCode::F20,
            "F21" => VirtualKeyCode::F21,
            "F22" => VirtualKeyCode::F22,
            "F23" => VirtualKeyCode::F23,
            "F24" => VirtualKeyCode::F24,
            "PrintScreen" => VirtualKeyCode::Snapshot,
            "ScrollLock" => VirtualKeyCode::Scroll,
            "Pause" => VirtualKeyCode::Pause,
            "Insert" => VirtualKeyCode::Insert,
            "Home" => VirtualKeyCode::Home,
            "Delete" => VirtualKeyCode::Delete,
            "End" => VirtualKeyCode::End,
            "PageDown" => VirtualKeyCode::PageDown,
            "PageUp" => VirtualKeyCode::PageUp,
            "ArrowLeft" => VirtualKeyCode::Left,
            "ArrowUp" => VirtualKeyCode::Up,
            "ArrowRight" => VirtualKeyCode::Right,
            "ArrowDown" => VirtualKeyCode::Down,
            "Backspace" => VirtualKeyCode::Backspace,
            "Enter" => VirtualKeyCode::Return,
            "Space" => VirtualKeyCode::Space,
            "Compose" => VirtualKeyCode::Compose,
            "Caret" => VirtualKeyCode::Caret,
            "NumLock" => VirtualKeyCode::Numlock,
            "Numpad0" => VirtualKeyCode::Numpad0,
            "Numpad1" => VirtualKeyCode::Numpad1,
            "Numpad2" => VirtualKeyCode::Numpad2,
            "Numpad3" => VirtualKeyCode::Numpad3,
            "Numpad4" => VirtualKeyCode::Numpad4,
            "Numpad5" => VirtualKeyCode::Numpad5,
            "Numpad6" => VirtualKeyCode::Numpad6,
            "Numpad7" => VirtualKeyCode::Numpad7,
            "Numpad8" => VirtualKeyCode::Numpad8,
            "Numpad9" => VirtualKeyCode::Numpad9,
            "AbntC1" => VirtualKeyCode::AbntC1,
            "AbntC2" => VirtualKeyCode::AbntC2,
            "NumpadAdd" => VirtualKeyCode::Add,
            "Quote" => VirtualKeyCode::Apostrophe,
            "Apps" => VirtualKeyCode::Apps,
            "At" => VirtualKeyCode::At,
            "Ax" => VirtualKeyCode::Ax,
            "Backslash" => VirtualKeyCode::Backslash,
            "Calculator" => VirtualKeyCode::Calculator,
            "Capital" => VirtualKeyCode::Capital,
            "Semicolon" => VirtualKeyCode::Semicolon,
            "Comma" => VirtualKeyCode::Comma,
            "Convert" => VirtualKeyCode::Convert,
            "NumpadDecimal" => VirtualKeyCode::Decimal,
            "NumpadDivide" => VirtualKeyCode::Divide,
            "Equal" => VirtualKeyCode::Equals,
            "Backquote" => VirtualKeyCode::Grave,
            "Kana" => VirtualKeyCode::Kana,
            "Kanji" => VirtualKeyCode::Kanji,
            "AltLeft" => VirtualKeyCode::LAlt,
            "BracketLeft" => VirtualKeyCode::LBracket,
            "ControlLeft" => VirtualKeyCode::LControl,
            "ShiftLeft" => VirtualKeyCode::LShift,
            "MetaLeft" => VirtualKeyCode::LWin,
            "Mail" => VirtualKeyCode::Mail,
            "MediaSelect" => VirtualKeyCode::MediaSelect,
            "MediaStop" => VirtualKeyCode::MediaStop,
            "Minus" => VirtualKeyCode::Minus,
            "NumpadMultiply" => VirtualKeyCode::Multiply,
            "Mute" => VirtualKeyCode::Mute,
            "LaunchMyComputer" => VirtualKeyCode::MyComputer,
            "NavigateForward" => VirtualKeyCode::NavigateForward,
            "NavigateBackward" => VirtualKeyCode::NavigateBackward,
            "NextTrack" => VirtualKeyCode::NextTrack,
            "NoConvert" => VirtualKeyCode::NoConvert,
            "NumpadComma" => VirtualKeyCode::NumpadComma,
            "NumpadEnter" => VirtualKeyCode::NumpadEnter,
            "NumpadEquals" => VirtualKeyCode::NumpadEquals,
            "OEM102" => VirtualKeyCode::OEM102,
            "Period" => VirtualKeyCode::Period,
            "PlayPause" => VirtualKeyCode::PlayPause,
            "Power" => VirtualKeyCode::Power,
            "PrevTrack" => VirtualKeyCode::PrevTrack,
            "AltRight" => VirtualKeyCode::RAlt,
            "BracketRight" => VirtualKeyCode::RBracket,
            "ControlRight" => VirtualKeyCode::RControl,
            "ShiftRight" => VirtualKeyCode::RShift,
            "MetaRight" => VirtualKeyCode::RWin,
            "Slash" => VirtualKeyCode::Slash,
            "Sleep" => VirtualKeyCode::Sleep,
            "Stop" => VirtualKeyCode::Stop,
            "Sysrq" => VirtualKeyCode::Sysrq,
            "Tab" => VirtualKeyCode::Tab,
            "Underline" => VirtualKeyCode::Underline,
            "Unlabeled" => VirtualKeyCode::Unlabeled,
            "AudioVolumeDown" => VirtualKeyCode::VolumeDown,
            "AudioVolumeUp" => VirtualKeyCode::VolumeUp,
            "Wake" => VirtualKeyCode::Wake,
            "WebBack" => VirtualKeyCode::WebBack,
            "WebFavorites" => VirtualKeyCode::WebFavorites,
            "WebForward" => VirtualKeyCode::WebForward,
            "WebHome" => VirtualKeyCode::WebHome,
            "WebRefresh" => VirtualKeyCode::WebRefresh,
            "WebSearch" => VirtualKeyCode::WebSearch,
            "WebStop" => VirtualKeyCode::WebStop,
            "Yen" => VirtualKeyCode::Yen,
            _ => return None,
        })
    }
}

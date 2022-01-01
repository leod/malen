use web_sys::KeyboardEvent;

use nalgebra::Point2;

#[derive(Debug, Clone)]
pub enum Event {
    Focused,
    Unfocused,
    KeyPressed(Key),
    KeyReleased(Key),
    MouseMoved(Point2<f64>),
}

/// A key that can be pressed.
///
/// This enum has been copied almost exactly from winit.
/// Source: https://github.com/rust-windowing/winit/blob/a2db4c0a320aafc10d240c432fe5ef4e4d84629d/src/event.rs#L774
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
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

impl Key {
    pub fn from_keyboard_event(event: &KeyboardEvent) -> Option<Self> {
        // Source:
        // https://github.com/rust-windowing/winit/blob/e4754999b7e7f27786092a62eda5275672d74130/src/platform_impl/web/web_sys/event.rs#L64
        Some(match &event.code()[..] {
            "Digit1" => Key::Key1,
            "Digit2" => Key::Key2,
            "Digit3" => Key::Key3,
            "Digit4" => Key::Key4,
            "Digit5" => Key::Key5,
            "Digit6" => Key::Key6,
            "Digit7" => Key::Key7,
            "Digit8" => Key::Key8,
            "Digit9" => Key::Key9,
            "Digit0" => Key::Key0,
            "KeyA" => Key::A,
            "KeyB" => Key::B,
            "KeyC" => Key::C,
            "KeyD" => Key::D,
            "KeyE" => Key::E,
            "KeyF" => Key::F,
            "KeyG" => Key::G,
            "KeyH" => Key::H,
            "KeyI" => Key::I,
            "KeyJ" => Key::J,
            "KeyK" => Key::K,
            "KeyL" => Key::L,
            "KeyM" => Key::M,
            "KeyN" => Key::N,
            "KeyO" => Key::O,
            "KeyP" => Key::P,
            "KeyQ" => Key::Q,
            "KeyR" => Key::R,
            "KeyS" => Key::S,
            "KeyT" => Key::T,
            "KeyU" => Key::U,
            "KeyV" => Key::V,
            "KeyW" => Key::W,
            "KeyX" => Key::X,
            "KeyY" => Key::Y,
            "KeyZ" => Key::Z,
            "Escape" => Key::Escape,
            "F1" => Key::F1,
            "F2" => Key::F2,
            "F3" => Key::F3,
            "F4" => Key::F4,
            "F5" => Key::F5,
            "F6" => Key::F6,
            "F7" => Key::F7,
            "F8" => Key::F8,
            "F9" => Key::F9,
            "F10" => Key::F10,
            "F11" => Key::F11,
            "F12" => Key::F12,
            "F13" => Key::F13,
            "F14" => Key::F14,
            "F15" => Key::F15,
            "F16" => Key::F16,
            "F17" => Key::F17,
            "F18" => Key::F18,
            "F19" => Key::F19,
            "F20" => Key::F20,
            "F21" => Key::F21,
            "F22" => Key::F22,
            "F23" => Key::F23,
            "F24" => Key::F24,
            "PrintScreen" => Key::Snapshot,
            "ScrollLock" => Key::Scroll,
            "Pause" => Key::Pause,
            "Insert" => Key::Insert,
            "Home" => Key::Home,
            "Delete" => Key::Delete,
            "End" => Key::End,
            "PageDown" => Key::PageDown,
            "PageUp" => Key::PageUp,
            "ArrowLeft" => Key::Left,
            "ArrowUp" => Key::Up,
            "ArrowRight" => Key::Right,
            "ArrowDown" => Key::Down,
            "Backspace" => Key::Backspace,
            "Enter" => Key::Return,
            "Space" => Key::Space,
            "Compose" => Key::Compose,
            "Caret" => Key::Caret,
            "NumLock" => Key::Numlock,
            "Numpad0" => Key::Numpad0,
            "Numpad1" => Key::Numpad1,
            "Numpad2" => Key::Numpad2,
            "Numpad3" => Key::Numpad3,
            "Numpad4" => Key::Numpad4,
            "Numpad5" => Key::Numpad5,
            "Numpad6" => Key::Numpad6,
            "Numpad7" => Key::Numpad7,
            "Numpad8" => Key::Numpad8,
            "Numpad9" => Key::Numpad9,
            "AbntC1" => Key::AbntC1,
            "AbntC2" => Key::AbntC2,
            "NumpadAdd" => Key::Add,
            "Quote" => Key::Apostrophe,
            "Apps" => Key::Apps,
            "At" => Key::At,
            "Ax" => Key::Ax,
            "Backslash" => Key::Backslash,
            "Calculator" => Key::Calculator,
            "Capital" => Key::Capital,
            "Semicolon" => Key::Semicolon,
            "Comma" => Key::Comma,
            "Convert" => Key::Convert,
            "NumpadDecimal" => Key::Decimal,
            "NumpadDivide" => Key::Divide,
            "Equal" => Key::Equals,
            "Backquote" => Key::Grave,
            "Kana" => Key::Kana,
            "Kanji" => Key::Kanji,
            "AltLeft" => Key::LAlt,
            "BracketLeft" => Key::LBracket,
            "ControlLeft" => Key::LControl,
            "ShiftLeft" => Key::LShift,
            "MetaLeft" => Key::LWin,
            "Mail" => Key::Mail,
            "MediaSelect" => Key::MediaSelect,
            "MediaStop" => Key::MediaStop,
            "Minus" => Key::Minus,
            "NumpadMultiply" => Key::Multiply,
            "Mute" => Key::Mute,
            "LaunchMyComputer" => Key::MyComputer,
            "NavigateForward" => Key::NavigateForward,
            "NavigateBackward" => Key::NavigateBackward,
            "NextTrack" => Key::NextTrack,
            "NoConvert" => Key::NoConvert,
            "NumpadComma" => Key::NumpadComma,
            "NumpadEnter" => Key::NumpadEnter,
            "NumpadEquals" => Key::NumpadEquals,
            "OEM102" => Key::OEM102,
            "Period" => Key::Period,
            "PlayPause" => Key::PlayPause,
            _ => return None,
        })
    }
}

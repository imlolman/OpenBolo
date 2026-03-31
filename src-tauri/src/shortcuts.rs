use parking_lot::RwLock;
use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// --- CGEventTap FFI ---

type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);
    fn CFRunLoopRun();
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    static kCFRunLoopCommonModes: *const c_void;
}

const KCG_HID_EVENT_TAP: u32 = 0;
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const KCG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

const CG_EVENT_KEY_DOWN: u32 = 10;
const CG_EVENT_KEY_UP: u32 = 11;
const CG_EVENT_FLAGS_CHANGED: u32 = 12;
const CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
const CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
const CG_EVENT_OTHER_MOUSE_DOWN: u32 = 25;
const CG_EVENT_OTHER_MOUSE_UP: u32 = 26;
const CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;

const KCG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const KCG_MOUSE_EVENT_BUTTON_NUMBER: u32 = 3;

// Modifier flag masks
const FLAG_SHIFT: u64 = 0x00020000;
const FLAG_CONTROL: u64 = 0x00040000;
const FLAG_OPTION: u64 = 0x00080000;
const FLAG_COMMAND: u64 = 0x00100000;

// macOS virtual key codes
const VK_A: u16 = 0x00;
const VK_S: u16 = 0x01;
const VK_D: u16 = 0x02;
const VK_F: u16 = 0x03;
const VK_H: u16 = 0x04;
const VK_G: u16 = 0x05;
const VK_Z: u16 = 0x06;
const VK_X: u16 = 0x07;
const VK_C: u16 = 0x08;
const VK_V: u16 = 0x09;
const VK_B: u16 = 0x0B;
const VK_Q: u16 = 0x0C;
const VK_W: u16 = 0x0D;
const VK_E: u16 = 0x0E;
const VK_R: u16 = 0x0F;
const VK_Y: u16 = 0x10;
const VK_T: u16 = 0x11;
const VK_1: u16 = 0x12;
const VK_2: u16 = 0x13;
const VK_3: u16 = 0x14;
const VK_4: u16 = 0x15;
const VK_6: u16 = 0x16;
const VK_5: u16 = 0x17;
const VK_9: u16 = 0x19;
const VK_7: u16 = 0x1A;
const VK_8: u16 = 0x1C;
const VK_0: u16 = 0x1D;
const VK_O: u16 = 0x1F;
const VK_U: u16 = 0x20;
const VK_I: u16 = 0x22;
const VK_P: u16 = 0x23;
const VK_L: u16 = 0x25;
const VK_J: u16 = 0x26;
const VK_K: u16 = 0x28;
const VK_N: u16 = 0x2D;
const VK_M: u16 = 0x2E;
const VK_RETURN: u16 = 0x24;
const VK_TAB: u16 = 0x30;
const VK_SPACE: u16 = 0x31;
const VK_DELETE: u16 = 0x33;
const VK_ESCAPE: u16 = 0x35;
const VK_LCOMMAND: u16 = 0x37;
const VK_LSHIFT: u16 = 0x38;
const VK_CAPSLOCK: u16 = 0x39;
const VK_LOPTION: u16 = 0x3A;
const VK_LCONTROL: u16 = 0x3B;
const VK_RCOMMAND: u16 = 0x36;
const VK_RSHIFT: u16 = 0x3C;
const VK_ROPTION: u16 = 0x3D;
const VK_RCONTROL: u16 = 0x3E;
const VK_FORWARD_DELETE: u16 = 0x75;
const VK_F1: u16 = 0x7A;
const VK_F2: u16 = 0x78;
const VK_F3: u16 = 0x63;
const VK_F4: u16 = 0x76;
const VK_F5: u16 = 0x60;
const VK_F6: u16 = 0x61;
const VK_F7: u16 = 0x62;
const VK_F8: u16 = 0x64;
const VK_F9: u16 = 0x65;
const VK_F10: u16 = 0x6D;
const VK_F11: u16 = 0x67;
const VK_F12: u16 = 0x6F;

// --- Public types (unchanged API) ---

#[derive(Debug, Clone)]
pub enum ShortcutEvent {
    HoldPress,
    HoldRelease,
    TogglePress,
    PastePress,
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutConfig {
    pub hold: ParsedShortcut,
    pub toggle: ParsedShortcut,
    pub paste: ParsedShortcut,
}

#[derive(Debug, Clone)]
pub enum ParsedShortcut {
    None,
    SingleKey(u16),
    MouseButton(u8),
    Combo(Vec<u16>),
}

impl Default for ParsedShortcut {
    fn default() -> Self {
        ParsedShortcut::None
    }
}

// --- Name ↔ key code mapping ---

fn key_code_from_name(name: &str) -> Option<u16> {
    match name {
        "Alt_L" => Some(VK_LOPTION),
        "Alt_R" => Some(VK_ROPTION),
        "Shift_L" => Some(VK_LSHIFT),
        "Shift_R" => Some(VK_RSHIFT),
        "Control_L" => Some(VK_LCONTROL),
        "Control_R" => Some(VK_RCONTROL),
        "Super_L" => Some(VK_LCOMMAND),
        "Super_R" => Some(VK_RCOMMAND),
        "space" => Some(VK_SPACE),
        "Return" => Some(VK_RETURN),
        "Escape" => Some(VK_ESCAPE),
        "BackSpace" => Some(VK_DELETE),
        "Tab" => Some(VK_TAB),
        "Caps_Lock" => Some(VK_CAPSLOCK),
        "Delete" => Some(VK_FORWARD_DELETE),
        "F1" => Some(VK_F1),
        "F2" => Some(VK_F2),
        "F3" => Some(VK_F3),
        "F4" => Some(VK_F4),
        "F5" => Some(VK_F5),
        "F6" => Some(VK_F6),
        "F7" => Some(VK_F7),
        "F8" => Some(VK_F8),
        "F9" => Some(VK_F9),
        "F10" => Some(VK_F10),
        "F11" => Some(VK_F11),
        "F12" => Some(VK_F12),
        s if s.len() == 1 => {
            let c = s.chars().next()?;
            match c.to_ascii_lowercase() {
                'a' => Some(VK_A),
                'b' => Some(VK_B),
                'c' => Some(VK_C),
                'd' => Some(VK_D),
                'e' => Some(VK_E),
                'f' => Some(VK_F),
                'g' => Some(VK_G),
                'h' => Some(VK_H),
                'i' => Some(VK_I),
                'j' => Some(VK_J),
                'k' => Some(VK_K),
                'l' => Some(VK_L),
                'm' => Some(VK_M),
                'n' => Some(VK_N),
                'o' => Some(VK_O),
                'p' => Some(VK_P),
                'q' => Some(VK_Q),
                'r' => Some(VK_R),
                's' => Some(VK_S),
                't' => Some(VK_T),
                'u' => Some(VK_U),
                'v' => Some(VK_V),
                'w' => Some(VK_W),
                'x' => Some(VK_X),
                'y' => Some(VK_Y),
                'z' => Some(VK_Z),
                '0' => Some(VK_0),
                '1' => Some(VK_1),
                '2' => Some(VK_2),
                '3' => Some(VK_3),
                '4' => Some(VK_4),
                '5' => Some(VK_5),
                '6' => Some(VK_6),
                '7' => Some(VK_7),
                '8' => Some(VK_8),
                '9' => Some(VK_9),
                _ => None,
            }
        }
        _ => None,
    }
}

#[allow(dead_code)]
fn key_name_from_code(code: u16) -> Option<&'static str> {
    match code {
        VK_LOPTION => Some("Alt_L"),
        VK_ROPTION => Some("Alt_R"),
        VK_LSHIFT => Some("Shift_L"),
        VK_RSHIFT => Some("Shift_R"),
        VK_LCONTROL => Some("Control_L"),
        VK_RCONTROL => Some("Control_R"),
        VK_LCOMMAND => Some("Super_L"),
        VK_RCOMMAND => Some("Super_R"),
        VK_SPACE => Some("space"),
        VK_RETURN => Some("Return"),
        VK_ESCAPE => Some("Escape"),
        VK_DELETE => Some("BackSpace"),
        VK_TAB => Some("Tab"),
        VK_CAPSLOCK => Some("Caps_Lock"),
        VK_FORWARD_DELETE => Some("Delete"),
        VK_F1 => Some("F1"),
        VK_F2 => Some("F2"),
        VK_F3 => Some("F3"),
        VK_F4 => Some("F4"),
        VK_F5 => Some("F5"),
        VK_F6 => Some("F6"),
        VK_F7 => Some("F7"),
        VK_F8 => Some("F8"),
        VK_F9 => Some("F9"),
        VK_F10 => Some("F10"),
        VK_F11 => Some("F11"),
        VK_F12 => Some("F12"),
        VK_A => Some("a"),
        VK_B => Some("b"),
        VK_C => Some("c"),
        VK_D => Some("d"),
        VK_E => Some("e"),
        VK_F => Some("f"),
        VK_G => Some("g"),
        VK_H => Some("h"),
        VK_I => Some("i"),
        VK_J => Some("j"),
        VK_K => Some("k"),
        VK_L => Some("l"),
        VK_M => Some("m"),
        VK_N => Some("n"),
        VK_O => Some("o"),
        VK_P => Some("p"),
        VK_Q => Some("q"),
        VK_R => Some("r"),
        VK_S => Some("s"),
        VK_T => Some("t"),
        VK_U => Some("u"),
        VK_V => Some("v"),
        VK_W => Some("w"),
        VK_X => Some("x"),
        VK_Y => Some("y"),
        VK_Z => Some("z"),
        VK_0 => Some("0"),
        VK_1 => Some("1"),
        VK_2 => Some("2"),
        VK_3 => Some("3"),
        VK_4 => Some("4"),
        VK_5 => Some("5"),
        VK_6 => Some("6"),
        VK_7 => Some("7"),
        VK_8 => Some("8"),
        VK_9 => Some("9"),
        _ => None,
    }
}

fn is_modifier(code: u16) -> bool {
    matches!(
        code,
        VK_LOPTION
            | VK_ROPTION
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_LCOMMAND
            | VK_RCOMMAND
    )
}

fn modifier_flag_for_keycode(code: u16) -> u64 {
    match code {
        VK_LSHIFT | VK_RSHIFT => FLAG_SHIFT,
        VK_LCONTROL | VK_RCONTROL => FLAG_CONTROL,
        VK_LOPTION | VK_ROPTION => FLAG_OPTION,
        VK_LCOMMAND | VK_RCOMMAND => FLAG_COMMAND,
        _ => 0,
    }
}

fn button_code_from_name(name: &str) -> Option<u8> {
    match name {
        "left" => Some(0),
        "right" => Some(1),
        "middle" => Some(2),
        "back" => Some(3),
        "forward" => Some(4),
        _ => None,
    }
}

fn button_name_from_code(code: u8) -> &'static str {
    match code {
        0 => "left",
        1 => "right",
        2 => "middle",
        3 => "back",
        4 => "forward",
        _ => "unknown",
    }
}

// --- Shortcut parsing (public, same API) ---

pub fn parse_shortcut(s: &str) -> ParsedShortcut {
    if s.is_empty() {
        return ParsedShortcut::None;
    }
    if let Some(mouse) = s.strip_prefix("mouse:") {
        return match button_code_from_name(mouse) {
            Some(b) => ParsedShortcut::MouseButton(b),
            None => ParsedShortcut::None,
        };
    }
    if let Some(combo_str) = s.strip_prefix("combo:") {
        let keys: Vec<u16> = combo_str
            .split('+')
            .filter_map(key_code_from_name)
            .collect();
        if keys.len() >= 2 {
            return ParsedShortcut::Combo(keys);
        }
        return ParsedShortcut::None;
    }
    if let Some(key_name) = s.strip_prefix("key:") {
        return match key_code_from_name(key_name) {
            Some(k) => ParsedShortcut::SingleKey(k),
            None => ParsedShortcut::None,
        };
    }
    ParsedShortcut::None
}

pub fn button_to_name(code: u8) -> String {
    format!("mouse:{}", button_name_from_code(code))
}

fn keys_match_combo(pressed: &HashSet<u16>, combo: &[u16]) -> bool {
    combo.iter().all(|k| pressed.contains(k))
}

// --- GrabHandle (unchanged API) ---

pub struct GrabHandle {
    pub enabled: Arc<AtomicBool>,
    pub config: Arc<RwLock<ShortcutConfig>>,
    pub capture_mode: Arc<AtomicBool>,
    pub capture_tx: Arc<parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
}

// --- Internal grab state (accessed only from the CGEventTap callback thread) ---

struct GrabState {
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
    pressed: HashSet<u16>,
    tap_port: CFMachPortRef,
    combo_hold_active: bool,
    combo_toggle_active: bool,
    combo_paste_active: bool,
    paste_pending: bool,
}

// --- CGEventTap callback ---

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tap_callback_inner(_proxy, event_type, event, user_info)
    }));
    match result {
        Ok(ev) => ev,
        Err(_) => {
            eprintln!("[shortcuts] PANIC in tap_callback, returning event as-is");
            event
        }
    }
}

unsafe fn tap_callback_inner(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    let state = &mut *(user_info as *mut GrabState);

    if event_type == CG_EVENT_TAP_DISABLED_BY_TIMEOUT {
        eprintln!("[shortcuts] tap disabled by timeout, re-enabling");
        CGEventTapEnable(state.tap_port, true);
        return event;
    }

    if state.handle.capture_mode.load(Ordering::SeqCst) {
        let btn = match event_type {
            CG_EVENT_LEFT_MOUSE_DOWN => Some(0u8),
            CG_EVENT_RIGHT_MOUSE_DOWN => Some(1u8),
            CG_EVENT_OTHER_MOUSE_DOWN => {
                Some(CGEventGetIntegerValueField(event, KCG_MOUSE_EVENT_BUTTON_NUMBER) as u8)
            }
            _ => None,
        };
        if let Some(b) = btn {
            let name = button_to_name(b);
            if let Some(tx) = state.handle.capture_tx.lock().take() {
                tx.send(name).ok();
            }
            return std::ptr::null_mut();
        }
        return event;
    }

    let keycode = CGEventGetIntegerValueField(event, KCG_KEYBOARD_EVENT_KEYCODE) as u16;
    let flags = CGEventGetFlags(event);

    if !state.handle.enabled.load(Ordering::SeqCst) {
        match event_type {
            CG_EVENT_KEY_UP => { state.pressed.remove(&keycode); }
            CG_EVENT_FLAGS_CHANGED => {
                if state.pressed.contains(&keycode) {
                    state.pressed.remove(&keycode);
                }
            }
            _ => {}
        }
        return event;
    }

    let config = state.handle.config.read().clone();

    match event_type {
        CG_EVENT_KEY_DOWN => {
            if !state.pressed.insert(keycode) {
                return event;
            }
            let suppress = process_key_press(state, &config, keycode);
            if suppress { std::ptr::null_mut() } else { event }
        }

        CG_EVENT_KEY_UP => {
            state.pressed.remove(&keycode);
            let suppress = process_key_release(state, &config, keycode);
            if suppress { std::ptr::null_mut() } else { event }
        }

        CG_EVENT_FLAGS_CHANGED => {
            let in_pressed = state.pressed.contains(&keycode);
            if is_modifier(keycode) {
                let flag = modifier_flag_for_keycode(keycode);
                let flag_set = flag != 0 && (flags & flag) != 0;

                if flag_set && !in_pressed {
                    state.pressed.insert(keycode);
                    let suppress = process_key_press(state, &config, keycode);
                    if suppress { return std::ptr::null_mut(); }
                } else if !flag_set && in_pressed {
                    state.pressed.remove(&keycode);
                    let suppress = process_key_release(state, &config, keycode);
                    if suppress { return std::ptr::null_mut(); }
                }
            }
            event
        }

        CG_EVENT_LEFT_MOUSE_DOWN | CG_EVENT_RIGHT_MOUSE_DOWN | CG_EVENT_OTHER_MOUSE_DOWN => {
            let btn = match event_type {
                CG_EVENT_LEFT_MOUSE_DOWN => 0u8,
                CG_EVENT_RIGHT_MOUSE_DOWN => 1u8,
                _ => CGEventGetIntegerValueField(event, KCG_MOUSE_EVENT_BUTTON_NUMBER) as u8,
            };
            let suppress = process_button_press(state, &config, btn);
            if suppress { std::ptr::null_mut() } else { event }
        }

        CG_EVENT_LEFT_MOUSE_UP | CG_EVENT_RIGHT_MOUSE_UP | CG_EVENT_OTHER_MOUSE_UP => {
            let btn = match event_type {
                CG_EVENT_LEFT_MOUSE_UP => 0u8,
                CG_EVENT_RIGHT_MOUSE_UP => 1u8,
                _ => CGEventGetIntegerValueField(event, KCG_MOUSE_EVENT_BUTTON_NUMBER) as u8,
            };
            let suppress = process_button_release(state, &config, btn);
            if suppress { std::ptr::null_mut() } else { event }
        }

        _ => event,
    }
}

fn process_key_press(state: &mut GrabState, config: &ShortcutConfig, keycode: u16) -> bool {
    let mut suppress = false;

    if let ParsedShortcut::SingleKey(k) = &config.hold {
        if keycode == *k {
            state.event_tx.send(ShortcutEvent::HoldPress).ok();
            suppress = true;
        }
    }
    if let ParsedShortcut::SingleKey(k) = &config.toggle {
        if keycode == *k {
            state.event_tx.send(ShortcutEvent::TogglePress).ok();
            suppress = true;
        }
    }
    if let ParsedShortcut::SingleKey(k) = &config.paste {
        if keycode == *k {
            state.paste_pending = true;
            suppress = true;
        }
    }

    if let ParsedShortcut::Combo(ref combo) = config.hold {
        if !state.combo_hold_active && keys_match_combo(&state.pressed, combo) {
            state.combo_hold_active = true;
            state.event_tx.send(ShortcutEvent::HoldPress).ok();
            if !is_modifier(keycode) { suppress = true; }
        }
    }
    if let ParsedShortcut::Combo(ref combo) = config.toggle {
        if !state.combo_toggle_active && keys_match_combo(&state.pressed, combo) {
            state.combo_toggle_active = true;
            state.event_tx.send(ShortcutEvent::TogglePress).ok();
            if !is_modifier(keycode) { suppress = true; }
        }
    }
    if let ParsedShortcut::Combo(ref combo) = config.paste {
        if !state.combo_paste_active && keys_match_combo(&state.pressed, combo) {
            state.combo_paste_active = true;
            state.paste_pending = true;
            if !is_modifier(keycode) { suppress = true; }
        }
    }

    suppress
}

fn process_key_release(state: &mut GrabState, config: &ShortcutConfig, keycode: u16) -> bool {
    let mut suppress = false;

    if let ParsedShortcut::SingleKey(k) = &config.hold {
        if keycode == *k {
            state.event_tx.send(ShortcutEvent::HoldRelease).ok();
            suppress = true;
        }
    }

    if let ParsedShortcut::SingleKey(k) = &config.paste {
        if keycode == *k && state.paste_pending {
            state.paste_pending = false;
            state.event_tx.send(ShortcutEvent::PastePress).ok();
            suppress = true;
        }
    }

    if state.combo_hold_active {
        if let ParsedShortcut::Combo(ref combo) = config.hold {
            if !keys_match_combo(&state.pressed, combo) {
                state.combo_hold_active = false;
                state.event_tx.send(ShortcutEvent::HoldRelease).ok();
            }
        }
    }
    if state.combo_toggle_active {
        if let ParsedShortcut::Combo(ref combo) = config.toggle {
            if !keys_match_combo(&state.pressed, combo) {
                state.combo_toggle_active = false;
            }
        }
    }
    if state.combo_paste_active {
        if let ParsedShortcut::Combo(ref combo) = config.paste {
            if !keys_match_combo(&state.pressed, combo) {
                state.combo_paste_active = false;
                if state.paste_pending {
                    state.paste_pending = false;
                    state.event_tx.send(ShortcutEvent::PastePress).ok();
                }
            }
        }
    }

    suppress
}

fn process_button_press(state: &mut GrabState, config: &ShortcutConfig, btn: u8) -> bool {
    let mut suppress = false;
    if let ParsedShortcut::MouseButton(b) = &config.hold {
        if btn == *b { state.event_tx.send(ShortcutEvent::HoldPress).ok(); suppress = true; }
    }
    if let ParsedShortcut::MouseButton(b) = &config.toggle {
        if btn == *b { state.event_tx.send(ShortcutEvent::TogglePress).ok(); suppress = true; }
    }
    if let ParsedShortcut::MouseButton(b) = &config.paste {
        if btn == *b { state.paste_pending = true; suppress = true; }
    }
    suppress
}

fn process_button_release(state: &mut GrabState, config: &ShortcutConfig, btn: u8) -> bool {
    let mut suppress = false;
    if let ParsedShortcut::MouseButton(b) = &config.hold {
        if btn == *b {
            state.event_tx.send(ShortcutEvent::HoldRelease).ok();
            suppress = true;
        }
    }
    if let ParsedShortcut::MouseButton(b) = &config.paste {
        if btn == *b && state.paste_pending {
            state.paste_pending = false;
            state.event_tx.send(ShortcutEvent::PastePress).ok();
            suppress = true;
        }
    }
    suppress
}

// --- Public start_grab (same API) ---

pub fn start_grab(
    handle: GrabHandle,
    event_tx: tokio::sync::mpsc::UnboundedSender<ShortcutEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let event_mask: u64 = (1 << CG_EVENT_KEY_DOWN)
            | (1 << CG_EVENT_KEY_UP)
            | (1 << CG_EVENT_FLAGS_CHANGED)
            | (1 << CG_EVENT_LEFT_MOUSE_DOWN)
            | (1 << CG_EVENT_LEFT_MOUSE_UP)
            | (1 << CG_EVENT_RIGHT_MOUSE_DOWN)
            | (1 << CG_EVENT_RIGHT_MOUSE_UP)
            | (1 << CG_EVENT_OTHER_MOUSE_DOWN)
            | (1 << CG_EVENT_OTHER_MOUSE_UP);

        let mut grab_state = Box::new(GrabState {
            handle,
            event_tx,
            pressed: HashSet::new(),
            tap_port: std::ptr::null_mut(),
            combo_hold_active: false,
            combo_toggle_active: false,
            combo_paste_active: false,
            paste_pending: false,
        });

        let state_ptr = &mut *grab_state as *mut GrabState as *mut c_void;

        unsafe {
            let tap = CGEventTapCreate(
                KCG_HID_EVENT_TAP,
                KCG_HEAD_INSERT_EVENT_TAP,
                KCG_EVENT_TAP_OPTION_DEFAULT,
                event_mask,
                tap_callback,
                state_ptr,
            );

            if tap.is_null() {
                eprintln!("[wisperflow] Failed to create CGEventTap. Grant Accessibility permission in System Settings > Privacy & Security > Accessibility.");
                return;
            }

            grab_state.tap_port = tap;

            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            if source.is_null() {
                eprintln!("[wisperflow] Failed to create run loop source for CGEventTap");
                return;
            }

            let run_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);

            // This blocks forever, processing events
            CFRunLoopRun();
        }

        // prevent drop until CFRunLoopRun returns
        drop(grab_state);
    })
}

// --- Display functions (unchanged API) ---

pub fn shortcut_display(s: &str) -> String {
    if s.is_empty() {
        return "Not set".into();
    }
    if let Some(mouse) = s.strip_prefix("mouse:") {
        return match mouse {
            "left" => "Left Click",
            "right" => "Right Click",
            "middle" => "Middle Click",
            "back" => "Back Button",
            "forward" => "Forward Button",
            _ => mouse,
        }
        .to_string();
    }
    if let Some(combo) = s.strip_prefix("combo:") {
        return combo
            .split('+')
            .map(key_display_name)
            .collect::<Vec<_>>()
            .join(" + ");
    }
    if let Some(key_name) = s.strip_prefix("key:") {
        return key_display_name(key_name);
    }
    s.to_string()
}

fn key_display_name(name: &str) -> String {
    match name {
        "Alt_L" => "Left Option \u{2325}".into(),
        "Alt_R" => "Right Option \u{2325}".into(),
        "Shift_L" => "Left Shift \u{21e7}".into(),
        "Shift_R" => "Right Shift \u{21e7}".into(),
        "Control_L" => "Left Control \u{2303}".into(),
        "Control_R" => "Right Control \u{2303}".into(),
        "Super_L" => "Left Command \u{2318}".into(),
        "Super_R" => "Right Command \u{2318}".into(),
        "space" => "Space".into(),
        "Return" => "Return \u{21a9}".into(),
        "Escape" => "Esc".into(),
        "BackSpace" => "Delete \u{232b}".into(),
        "Tab" => "Tab \u{21e5}".into(),
        s if s.len() == 1 => s.to_uppercase(),
        s => s.to_string(),
    }
}

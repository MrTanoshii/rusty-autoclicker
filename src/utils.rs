use std::{env, thread, time::Duration};

use device_query::Keycode;
use eframe::emath::Numeric;
use rand::{RngExt, prelude::ThreadRng};
use rdev::{EventType, Key, SimulateError, simulate};
use sanitizer::prelude::StringSanitizer;

use crate::{
    defines::{
        APP_ICON, DURATION_CLICK_MAX, DURATION_CLICK_MIN, DURATION_DOUBLE_CLICK_MAX,
        DURATION_DOUBLE_CLICK_MIN, MOUSE_TWEEN_CURVE_MAX_PX, MOUSE_TWEEN_CURVE_RATIO_MAX,
        MOUSE_TWEEN_CURVE_RATIO_MIN, MOUSE_TWEEN_DELAY_JITTER_FRAC, MOUSE_TWEEN_MIN_STEPS,
        MOUSE_TWEEN_STEP_PX, MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX, MOUSE_TWEEN_TREMOR_MAX_FAR,
        MOUSE_TWEEN_TREMOR_MAX_NEAR, MOUSE_TWEEN_TREMOR_PX,
    },
    types::{AppMode, ClickButton, ClickInfo, ClickPosition},
};

/// Abbreviated, ≤8-character display form of an `rdev::Key`, for compact
/// display in the click-button dropdown, capture feedback/error messages,
/// and anywhere else a click-button key is shown. This is the single
/// source of truth for that formatting — don't use `{:?}` / `stringify!`
/// on `Key` directly, or the dropdown and the hotkey bar will drift apart
/// again (which is the whole reason this exists: "KeyF" in the dropdown vs
/// "F" in the hotkey bar for what is conceptually the same key).
///
/// Kept in sync 1:1 with `abbreviate_keycode` below for every key that
/// exists in both enums, so e.g. the F key always reads "F" whether it
/// came from a hotkey capture or the click-button dropdown.
pub fn abbreviate_key(key: Key) -> &'static str {
    use Key as K;
    match key {
        K::KeyA => "A", K::KeyB => "B", K::KeyC => "C", K::KeyD => "D", K::KeyE => "E",
        K::KeyF => "F", K::KeyG => "G", K::KeyH => "H", K::KeyI => "I", K::KeyJ => "J",
        K::KeyK => "K", K::KeyL => "L", K::KeyM => "M", K::KeyN => "N", K::KeyO => "O",
        K::KeyP => "P", K::KeyQ => "Q", K::KeyR => "R", K::KeyS => "S", K::KeyT => "T",
        K::KeyU => "U", K::KeyV => "V", K::KeyW => "W", K::KeyX => "X", K::KeyY => "Y",
        K::KeyZ => "Z",

        K::Num0 => "0", K::Num1 => "1", K::Num2 => "2", K::Num3 => "3", K::Num4 => "4",
        K::Num5 => "5", K::Num6 => "6", K::Num7 => "7", K::Num8 => "8", K::Num9 => "9",

        K::F1 => "F1", K::F2 => "F2", K::F3 => "F3", K::F4 => "F4", K::F5 => "F5",
        K::F6 => "F6", K::F7 => "F7", K::F8 => "F8", K::F9 => "F9", K::F10 => "F10",
        K::F11 => "F11", K::F12 => "F12",

        K::Escape => "Escape",
        K::Space => "Space",

        K::ControlLeft => "LCtrl",
        K::ControlRight => "RCtrl",
        K::ShiftLeft => "LShift",
        K::ShiftRight => "RShift",
        K::Alt => "Alt",
        K::AltGr => "AltGr",
        K::MetaLeft => "LMeta",
        K::MetaRight => "RMeta",
        K::Function => "Fn",

        K::Return => "Enter",
        K::UpArrow => "Up",
        K::DownArrow => "Down",
        K::LeftArrow => "Left",
        K::RightArrow => "Right",

        K::CapsLock => "CapsLock",
        K::Tab => "Tab",
        K::Home => "Home",
        K::End => "End",
        K::PageUp => "PageUp",
        K::PageDown => "PageDown",
        K::Insert => "Insert",
        K::Delete => "Delete",
        K::Backspace => "Backspc",

        K::PrintScreen => "PrtSc",
        K::ScrollLock => "ScrLk",
        K::Pause => "Pause",
        K::NumLock => "NumLock",

        K::BackQuote => "`",
        K::Minus => "-",
        K::Equal => "=",
        K::LeftBracket => "[",
        K::RightBracket => "]",
        K::SemiColon => ";",
        K::Quote => "'",
        K::BackSlash => "\\",
        K::IntlBackslash => "IntlBS",
        K::Comma => ",",
        K::Dot => ".",
        K::Slash => "/",

        K::KpReturn => "NumEnt",
        K::KpMinus => "Num-",
        K::KpPlus => "Num+",
        K::KpMultiply => "Num*",
        K::KpDivide => "Num/",
        K::Kp0 => "Num0", K::Kp1 => "Num1", K::Kp2 => "Num2", K::Kp3 => "Num3",
        K::Kp4 => "Num4", K::Kp5 => "Num5", K::Kp6 => "Num6", K::Kp7 => "Num7",
        K::Kp8 => "Num8", K::Kp9 => "Num9",
        K::KpDelete => "Num.",

        // Anything outside the dropdown's selectable set (click_btn can
        // only ever hold a value the UI let the user pick).
        _ => "?",
    }
}

/// Abbreviated, ≤8-character display form of a `device_query::Keycode`,
/// for the top hotkey bar and the Hotkeys window. Kept 1:1 in sync with
/// `abbreviate_key` above for every key both enums share — see that
/// function's doc comment for why this matters.
///
/// Unlike `abbreviate_key`, this is exhaustive over every `Keycode`
/// variant rather than falling back to a placeholder, since a hotkey field
/// can genuinely hold any of them (including ones with no dropdown/rdev
/// counterpart, like F13-F20 or the Mac Command/Option keys).
pub fn abbreviate_keycode(keycode: Keycode) -> &'static str {
    use Keycode as K;
    match keycode {
        K::Key0 => "0", K::Key1 => "1", K::Key2 => "2", K::Key3 => "3", K::Key4 => "4",
        K::Key5 => "5", K::Key6 => "6", K::Key7 => "7", K::Key8 => "8", K::Key9 => "9",

        K::A => "A", K::B => "B", K::C => "C", K::D => "D", K::E => "E", K::F => "F",
        K::G => "G", K::H => "H", K::I => "I", K::J => "J", K::K => "K", K::L => "L",
        K::M => "M", K::N => "N", K::O => "O", K::P => "P", K::Q => "Q", K::R => "R",
        K::S => "S", K::T => "T", K::U => "U", K::V => "V", K::W => "W", K::X => "X",
        K::Y => "Y", K::Z => "Z",

        K::F1 => "F1", K::F2 => "F2", K::F3 => "F3", K::F4 => "F4", K::F5 => "F5",
        K::F6 => "F6", K::F7 => "F7", K::F8 => "F8", K::F9 => "F9", K::F10 => "F10",
        K::F11 => "F11", K::F12 => "F12", K::F13 => "F13", K::F14 => "F14",
        K::F15 => "F15", K::F16 => "F16", K::F17 => "F17", K::F18 => "F18",
        K::F19 => "F19", K::F20 => "F20",

        K::Escape => "Escape",
        K::Space => "Space",

        K::LControl => "LCtrl",
        K::RControl => "RCtrl",
        K::LShift => "LShift",
        K::RShift => "RShift",
        K::LAlt => "Alt",
        K::RAlt => "AltGr",
        // Mac-only, no rdev/dropdown counterpart.
        K::Command => "Cmd",
        K::RCommand => "RCmd",
        K::LOption => "LOpt",
        K::ROption => "ROpt",
        K::LMeta => "LMeta",
        K::RMeta => "RMeta",

        K::Enter => "Enter",
        K::Up => "Up",
        K::Down => "Down",
        K::Left => "Left",
        K::Right => "Right",

        K::Backspace => "Backspc",

        K::CapsLock => "CapsLock",
        K::Tab => "Tab",
        K::Home => "Home",
        K::End => "End",
        K::PageUp => "PageUp",
        K::PageDown => "PageDown",
        K::Insert => "Insert",
        K::Delete => "Delete",

        K::Numpad0 => "Num0", K::Numpad1 => "Num1", K::Numpad2 => "Num2",
        K::Numpad3 => "Num3", K::Numpad4 => "Num4", K::Numpad5 => "Num5",
        K::Numpad6 => "Num6", K::Numpad7 => "Num7", K::Numpad8 => "Num8",
        K::Numpad9 => "Num9",
        K::NumpadSubtract => "Num-",
        K::NumpadAdd => "Num+",
        K::NumpadDivide => "Num/",
        K::NumpadMultiply => "Num*",
        // No rdev/dropdown counterpart used in this app.
        K::NumpadEquals => "Num=",
        K::NumpadEnter => "NumEnt",
        K::NumpadDecimal => "Num.",

        K::Grave => "`",
        K::Minus => "-",
        K::Equal => "=",
        K::LeftBracket => "[",
        K::RightBracket => "]",
        K::BackSlash => "\\",
        K::Semicolon => ";",
        K::Apostrophe => "'",
        K::Comma => ",",
        K::Dot => ".",
        K::Slash => "/",
    }
}

/// Map a globally-polled `device_query::Keycode` (used for hotkeys) to the
/// `rdev::Key` it represents (used for the simulated click button), so a
/// physically-pressed key can be used to select a click button directly
/// instead of scrolling the dropdown in `gui/sections/buttons.rs`.
///
/// Returns `None` for keys that either have no `rdev::Key` counterpart
/// offered in the click-button dropdown, or that `device_query` can't
/// report at all (e.g. PrintScreen/ScrollLock/Pause/NumLock aren't polled
/// by `device_query`, so they can never reach this function from a live
/// key-press — the dropdown remains the only way to select those).
/// Callers should fall back to the manual dropdown when this returns `None`.
pub fn keycode_to_rdev_key(keycode: Keycode) -> Option<Key> {
    use Keycode as K;
    Some(match keycode {
        K::Key0 => Key::Num0,
        K::Key1 => Key::Num1,
        K::Key2 => Key::Num2,
        K::Key3 => Key::Num3,
        K::Key4 => Key::Num4,
        K::Key5 => Key::Num5,
        K::Key6 => Key::Num6,
        K::Key7 => Key::Num7,
        K::Key8 => Key::Num8,
        K::Key9 => Key::Num9,

        K::A => Key::KeyA,
        K::B => Key::KeyB,
        K::C => Key::KeyC,
        K::D => Key::KeyD,
        K::E => Key::KeyE,
        K::F => Key::KeyF,
        K::G => Key::KeyG,
        K::H => Key::KeyH,
        K::I => Key::KeyI,
        K::J => Key::KeyJ,
        K::K => Key::KeyK,
        K::L => Key::KeyL,
        K::M => Key::KeyM,
        K::N => Key::KeyN,
        K::O => Key::KeyO,
        K::P => Key::KeyP,
        K::Q => Key::KeyQ,
        K::R => Key::KeyR,
        K::S => Key::KeyS,
        K::T => Key::KeyT,
        K::U => Key::KeyU,
        K::V => Key::KeyV,
        K::W => Key::KeyW,
        K::X => Key::KeyX,
        K::Y => Key::KeyY,
        K::Z => Key::KeyZ,

        K::F1 => Key::F1,
        K::F2 => Key::F2,
        K::F3 => Key::F3,
        K::F4 => Key::F4,
        K::F5 => Key::F5,
        K::F6 => Key::F6,
        K::F7 => Key::F7,
        K::F8 => Key::F8,
        K::F9 => Key::F9,
        K::F10 => Key::F10,
        K::F11 => Key::F11,
        K::F12 => Key::F12,

        K::Escape => Key::Escape,
        K::Space => Key::Space,

        K::LControl => Key::ControlLeft,
        K::RControl => Key::ControlRight,
        K::LShift => Key::ShiftLeft,
        K::RShift => Key::ShiftRight,
        K::LAlt => Key::Alt,
        K::RAlt => Key::AltGr,
        K::LMeta => Key::MetaLeft,
        K::RMeta => Key::MetaRight,

        K::Enter => Key::Return,
        K::Up => Key::UpArrow,
        K::Down => Key::DownArrow,
        K::Left => Key::LeftArrow,
        K::Right => Key::RightArrow,

        K::CapsLock => Key::CapsLock,
        K::Tab => Key::Tab,
        K::Home => Key::Home,
        K::End => Key::End,
        K::PageUp => Key::PageUp,
        K::PageDown => Key::PageDown,
        K::Insert => Key::Insert,
        K::Delete => Key::Delete,
        // Was previously missing from this table entirely, which is what
        // caused a captured Backspace to wrongly report "not supported for
        // auto-capture" — device_query DOES report Backspace fine; it's
        // rdev::Key::Backspace that just wasn't in the dropdown yet either.
        // Both are now wired up (see the dropdown/key_names! list and the
        // abbreviation tables).
        K::Backspace => Key::Backspace,

        K::Numpad0 => Key::Kp0,
        K::Numpad1 => Key::Kp1,
        K::Numpad2 => Key::Kp2,
        K::Numpad3 => Key::Kp3,
        K::Numpad4 => Key::Kp4,
        K::Numpad5 => Key::Kp5,
        K::Numpad6 => Key::Kp6,
        K::Numpad7 => Key::Kp7,
        K::Numpad8 => Key::Kp8,
        K::Numpad9 => Key::Kp9,
        K::NumpadSubtract => Key::KpMinus,
        K::NumpadAdd => Key::KpPlus,
        K::NumpadDivide => Key::KpDivide,
        K::NumpadMultiply => Key::KpMultiply,
        K::NumpadEnter => Key::KpReturn,
        // rdev has no separate "decimal" numpad key — on most keyboards
        // this physical key reports as Delete when Num Lock is off and as
        // a digit/decimal-point when it's on, and rdev only models the
        // Delete side of that (`KpDelete`). This was previously missing
        // too, causing the same false "not supported" error as Backspace.
        K::NumpadDecimal => Key::KpDelete,

        K::Grave => Key::BackQuote,
        K::Minus => Key::Minus,
        K::Equal => Key::Equal,
        K::LeftBracket => Key::LeftBracket,
        K::RightBracket => Key::RightBracket,
        K::BackSlash => Key::BackSlash,
        K::Semicolon => Key::SemiColon,
        K::Apostrophe => Key::Quote,
        K::Comma => Key::Comma,
        K::Dot => Key::Dot,
        K::Slash => Key::Slash,

        // No rdev::Key counterpart offered in the dropdown (NumpadEquals,
        // Command/RCommand/LOption/ROption, F13+): fall back to manual
        // selection. These genuinely have no equivalent, unlike Backspace
        // and NumpadDecimal above.
        _ => return None,
    })
}

/// Load icon from memory and return it
pub fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(APP_ICON)
            .expect("Failed to open icon path")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

/// Sanitize string
///
/// # Arguments
///
/// * `string` - String to sanitize
/// * `max_length` - Maximum length of string
pub fn sanitize_string(string: &mut String, max_length: usize) {
    // Accept numeric only
    let s_slice = string.as_str();
    let mut sanitizer = StringSanitizer::from(s_slice);
    sanitizer.numeric();
    *string = sanitizer.get();

    // Remove leading 0
    while string.len() > 1 && string.starts_with('0') {
        string.remove(0);
    }

    truncate_string(string, max_length);
}

/// Sanitize string of expected i64 type
///
/// # Arguments
///
/// * `string` - String to sanitize
/// * `max_length` - Maximum length of string
pub fn sanitize_i64_string(string: &mut String, max_length: usize) {
    // Remove leading & trailing whitespaces
    // Parse to i64 or return default of 0
    *string = string.trim().parse::<i64>().unwrap_or_default().to_string();

    truncate_string(string, max_length);
}

/// Truncate string to specified length
///
/// # Arguments
///
/// * `string` - String to be truncated
/// * `max_length` - Maximum length of string
fn truncate_string(string: &mut String, max_length: usize) {
    // Allow max size of `max_length` characters
    if string.len() >= max_length {
        string.truncate(max_length)
    };
}

/// Click interval in milliseconds from the hour/minute/second/millisecond values.
pub const fn interval_ms(hr: u64, min: u64, sec: u64, ms: u64) -> u64 {
    (hr * 3_600_000) + (min * 60_000) + (sec * 1_000) + ms
}

/// Send the simulated event (`rdev` crate)
///
/// # Arguments
///
/// * `event_type` - The event type to simulate
fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {event_type:?}");
        }
    }

    // Let the OS catchup (at least MacOS); mouse moves are exempt, as the
    // extra delay would cap the humanlike glide speed
    if env::consts::OS == "macos" && !matches!(event_type, EventType::MouseMove { .. }) {
        thread::sleep(Duration::from_millis(20u64));
    }
}

/// Evaluate a cubic Bezier curve at parameter `t` in [0, 1].
fn cubic_bezier(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    (
        w0 * p0.0 + w1 * p1.0 + w2 * p2.0 + w3 * p3.0,
        w0 * p0.1 + w1 * p1.1 + w2 * p2.1 + w3 * p3.1,
    )
}

/// Smoothstep ease-in-out: slow start, fast middle, slow end.
fn ease_in_out(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Maximum number of extra mid-tween hand tremors for a glide of `distance` px.
fn mid_tremor_max(distance: f64) -> u64 {
    if distance < MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX {
        MOUSE_TWEEN_TREMOR_MAX_NEAR
    } else {
        MOUSE_TWEEN_TREMOR_MAX_FAR
    }
}

/// [Humanlike] Glide the mouse from `start_coords` to `click_coord` along a
/// randomized cubic Bezier curve with ease-in-out timing and occasional hand
/// tremor (at the start and end of the glide, plus up to one or two random
/// mid-tween steps depending on distance), mimicking human movement. The
/// final event lands exactly on `click_coord`.
///
/// # Arguments
///
/// * `click_coord` - The click coordinates
/// * `start_coords` - The starting mouse coordinates
/// * `speed_range` - The (min, max) glide speed in px/s; the actual speed is
///   randomized within it
/// * `rng_thread` - The random number generator
fn move_to(
    click_coord: (f64, f64),
    start_coords: (f64, f64),
    speed_range: (f64, f64),
    rng_thread: &mut ThreadRng,
) {
    let dx = click_coord.0 - start_coords.0;
    let dy = click_coord.1 - start_coords.1;
    let distance = dx.hypot(dy);
    if distance < f64::EPSILON {
        send(&EventType::MouseMove {
            x: click_coord.0,
            y: click_coord.1,
        });
        return;
    }

    let steps = ((distance / MOUSE_TWEEN_STEP_PX).ceil() as u64).max(MOUSE_TWEEN_MIN_STEPS);

    // Randomized control points: bow the curve perpendicular to the straight
    // line, independently per control point (yields C-arcs and S-curves)
    let perp = (-dy / distance, dx / distance);
    let bow = (distance
        * rng_thread.random_range(MOUSE_TWEEN_CURVE_RATIO_MIN..=MOUSE_TWEEN_CURVE_RATIO_MAX))
    .min(MOUSE_TWEEN_CURVE_MAX_PX);
    let offset1 = bow * rng_thread.random_range(-1.0..=1.0);
    let offset2 = bow * rng_thread.random_range(-1.0..=1.0);
    let p1 = (
        start_coords.0 + 0.30 * dx + perp.0 * offset1,
        start_coords.1 + 0.30 * dy + perp.1 * offset1,
    );
    let p2 = (
        start_coords.0 + 0.70 * dx + perp.0 * offset2,
        start_coords.1 + 0.70 * dy + perp.1 * offset2,
    );

    // Hand tremor at the start and end of the glide (the final step stays
    // exact), plus up to one (short move) or two (long move) tremors at
    // random mid-tween steps
    let mut tremor_steps = vec![1, steps - 1];
    for _ in 0..rng_thread.random_range(0..=mid_tremor_max(distance)) {
        tremor_steps.push(rng_thread.random_range(2..=steps - 2));
    }

    // Random glide speed within the configured range
    let speed = rng_thread.random_range(speed_range.0..=speed_range.1.max(speed_range.0));
    let step_delay_s = distance / speed / steps as f64;

    #[cfg(debug_assertions)]
    println!(
        "Tweening from {start_coords:?} to {click_coord:?}: distance {distance:.1}px, speed {speed:.0}px/s, {steps} steps, bow {bow:.1}px, tremors at {tremor_steps:?}"
    );

    for i in 1..=steps {
        let t = ease_in_out(i as f64 / steps as f64);
        let (x, y) = if i == steps {
            // land exactly on the target, a click follows
            click_coord
        } else {
            let (x, y) = cubic_bezier(start_coords, p1, p2, click_coord, t);
            if tremor_steps.contains(&i) {
                (
                    x + rng_thread.random_range(-MOUSE_TWEEN_TREMOR_PX..=MOUSE_TWEEN_TREMOR_PX),
                    y + rng_thread.random_range(-MOUSE_TWEEN_TREMOR_PX..=MOUSE_TWEEN_TREMOR_PX),
                )
            } else {
                (x, y)
            }
        };
        send(&EventType::MouseMove { x, y });

        if i < steps {
            thread::sleep(Duration::from_secs_f64(
                step_delay_s
                    * rng_thread.random_range(
                        (1.0 - MOUSE_TWEEN_DELAY_JITTER_FRAC)
                            ..=(1.0 + MOUSE_TWEEN_DELAY_JITTER_FRAC),
                    ),
            ));
        }
    }
}

/// Move the mouse to `click_coord`, honoring the app mode: an instant jump in
/// [`Bot`](AppMode::Bot), or a step-wise humanlike glide in
/// [`Humanlike`](AppMode::Humanlike) (skipped when already at the target).
/// Shared by autoclick and click-and-hold.
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_coord` - The target coordinates
/// * `start_coords` - The current mouse coordinates (humanlike start point)
/// * `speed_range` - The (min, max) humanlike glide speed in px/s
/// * `rng_thread` - The random number generator (humanlike tweening)
pub fn move_mouse_to(
    app_mode: AppMode,
    click_coord: (f64, f64),
    start_coords: (i32, i32),
    speed_range: (f64, f64),
    rng_thread: &mut ThreadRng,
) {
    match app_mode {
        AppMode::Bot => send(&EventType::MouseMove {
            x: click_coord.0,
            y: click_coord.1,
        }),
        AppMode::Humanlike => {
            let start = (start_coords.0.to_f64(), start_coords.1.to_f64());
            // only move if start pos and click pos are not identical
            if click_coord.0 != start.0 || click_coord.1 != start.1 {
                move_to(click_coord, start, speed_range, rng_thread);
            }
        }
    }
}

/// Press the button/key down without releasing it (used by click-and-hold).
pub fn press_button(button: ClickButton) {
    match button {
        ClickButton::Mouse(button) => send(&EventType::ButtonPress(button)),
        ClickButton::Key(key) => send(&EventType::KeyPress(key)),
    }
}

/// Release a previously pressed button/key (used by click-and-hold).
pub fn release_button(button: ClickButton) {
    match button {
        ClickButton::Mouse(button) => send(&EventType::ButtonRelease(button)),
        ClickButton::Key(key) => send(&EventType::KeyRelease(key)),
    }
}

fn click_once(button: ClickButton, hold: Option<Duration>) {
    press_button(button);
    if let Some(hold) = hold {
        thread::sleep(hold);
    }
    release_button(button);
}

/// Autoclick the mouse
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_info` - The click information
/// * `mouse_coord` - The mouse coordinates
/// * `speed_range` - The (min, max) humanlike glide speed in px/s
/// * `rng_thread` - The random number generator thread
pub fn autoclick(
    app_mode: AppMode,
    click_info: ClickInfo,
    mouse_coord: (i32, i32),
    speed_range: (f64, f64),
    mut rng_thread: ThreadRng,
) {
    // Number of press/release cycles required
    let run_amount = click_info.click_type.run_count();

    // Autoclick as fast as possible
    if app_mode == AppMode::Bot {
        for _n in 1..=run_amount {
            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                move_mouse_to(
                    app_mode,
                    click_info.click_coord,
                    mouse_coord,
                    speed_range,
                    &mut rng_thread,
                );
            }
            click_once(click_info.click_btn, None);
        }
    // Autoclick to emulate a humanlike clicks
    } else if app_mode == AppMode::Humanlike {
        // move to target
        #[cfg(debug_assertions)]
        println!(
            "Moving from {:?}/{:?} towards: {:?}/{:?}",
            mouse_coord.0.to_f64(),
            mouse_coord.1.to_f64(),
            click_info.click_coord.0,
            click_info.click_coord.1
        );

        // perform clicks
        for n in 1..=run_amount {
            // Sleep between clicks
            if n % 2 == 0 {
                thread::sleep(Duration::from_millis(
                    rng_thread.random_range(DURATION_DOUBLE_CLICK_MIN..DURATION_DOUBLE_CLICK_MAX),
                ));
            }

            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                move_mouse_to(
                    app_mode,
                    click_info.click_coord,
                    mouse_coord,
                    speed_range,
                    &mut rng_thread,
                );
            }

            // Press, hold for a randomized human-like duration, then release
            let hold = Duration::from_millis(
                rng_thread.random_range(DURATION_CLICK_MIN..DURATION_CLICK_MAX),
            );
            click_once(click_info.click_btn, Some(hold));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_string_keeps_digits_only() {
        let mut s = String::from("1a2b3");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "123");
    }

    #[test]
    fn sanitize_string_strips_leading_zeros() {
        let mut s = String::from("007");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "7");
    }

    #[test]
    fn sanitize_string_keeps_a_single_zero() {
        let mut s = String::from("0");
        sanitize_string(&mut s, 10);
        assert_eq!(s, "0");
    }

    #[test]
    fn sanitize_string_truncates_to_max_length() {
        let mut s = String::from("123456789");
        sanitize_string(&mut s, 5);
        assert_eq!(s, "12345");
    }

    #[test]
    fn sanitize_i64_string_trims_and_keeps_sign() {
        let mut s = String::from("  -5 ");
        sanitize_i64_string(&mut s, 7);
        assert_eq!(s, "-5");
    }

    #[test]
    fn sanitize_i64_string_falls_back_to_zero_on_garbage() {
        let mut s = String::from("abc");
        sanitize_i64_string(&mut s, 7);
        assert_eq!(s, "0");
    }

    #[test]
    fn truncate_string_trims_when_too_long() {
        let mut s = String::from("123456");
        truncate_string(&mut s, 5);
        assert_eq!(s, "12345");
    }

    #[test]
    fn truncate_string_leaves_short_strings_untouched() {
        let mut s = String::from("12");
        truncate_string(&mut s, 5);
        assert_eq!(s, "12");
    }

    #[test]
    fn interval_ms_combines_units() {
        assert_eq!(interval_ms(0, 0, 0, 100), 100);
        assert_eq!(interval_ms(1, 0, 0, 0), 3_600_000);
        assert_eq!(interval_ms(0, 1, 1, 1), 61_001);
    }

    #[test]
    fn cubic_bezier_hits_endpoints() {
        let p0 = (10.0, 20.0);
        let p1 = (50.0, -30.0);
        let p2 = (120.0, 80.0);
        let p3 = (200.0, 40.0);
        assert_eq!(cubic_bezier(p0, p1, p2, p3, 0.0), p0);
        assert_eq!(cubic_bezier(p0, p1, p2, p3, 1.0), p3);
    }

    #[test]
    fn cubic_bezier_midpoint_known_value() {
        // At t = 0.5 the weights are 1/8, 3/8, 3/8, 1/8
        let (x, y) = cubic_bezier((0.0, 0.0), (8.0, 0.0), (0.0, 8.0), (8.0, 8.0), 0.5);
        assert!((x - 4.0).abs() < 1e-9);
        assert!((y - 4.0).abs() < 1e-9);
    }

    #[test]
    fn mid_tremor_max_depends_on_distance() {
        assert_eq!(mid_tremor_max(50.0), 1);
        assert_eq!(mid_tremor_max(479.9), 1);
        assert_eq!(mid_tremor_max(480.0), 2);
        assert_eq!(mid_tremor_max(1500.0), 2);
    }

    #[test]
    fn ease_in_out_boundaries_and_symmetry() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-9);
        assert!(ease_in_out(0.25) < ease_in_out(0.5));
        assert!(ease_in_out(0.5) < ease_in_out(0.75));
    }
}

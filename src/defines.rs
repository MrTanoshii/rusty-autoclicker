use device_query::Keycode;
use eframe::egui::FontFamily;

pub const APP_NAME: &str = "Rusty AutoClicker";
pub const APP_ICON: &[u8] = include_bytes!("../assets/icon-64x64.ico");

// Font
pub const FONT_SIZE: f32 = 12.0;
pub const FONT_FAMILY: FontFamily = FontFamily::Monospace;

// Default window size (logical pixels) — the original GUI dimensions.
// This stays a fixed constant: it's what Reset restores and what a
// fresh install starts at. It is NOT what the responsive layout computes
// or targets — the layout scales to whatever size the window actually is,
// between WINDOW_MIN_WIDTH/HEIGHT below and however large the user grows it.
//
// WINDOW_HEIGHT was bumped from the original 340 to fit the Random Offset
// row added alongside Click Interval, then bumped again (380 -> 374) for a
// SMALL, EQUAL amount of breathing room both ABOVE AND BELOW the Start/Hold
// buttons (see `show_autoclicker` in `gui/sections/mod.rs`) — just enough
// that row doesn't sit flush against the separator above it or the bottom
// bar below. `CentralPanel`'s and the bottom panel's own fixed (unscaled)
// frame margins are zeroed out on the sides that face the buttons (see
// those frames in `gui/mod.rs::ui()` and `show_bottombar`), so
// `show_autoclicker`'s `gap` is the ONLY source of that spacing on both
// sides — otherwise the fixed frame margins stacked on top of it and made
// the bottom gap visibly bigger than the top one. At 1:1 scale the content
// column needs ~316px and the topbar+bottombar+panel margins eat another
// ~56px regardless of window size, so anything under ~372 no longer had
// Default window height (logical pixels). Chosen so the default view
// renders at (approximately) `ui_scale == 1.0` in `gui/mod.rs` — i.e.
// pixel-parity with the original fixed-size design — rather than the
// content needing to shrink below scale 1.0 just to fit inside a shorter
// default window. Leaves a small, roughly symmetric gap above and below
// the Start/Hold button row at the default size.
pub const WINDOW_WIDTH: f32 = 580.0;
pub const WINDOW_HEIGHT: f32 = 372.0;

// Minimum resizable window size (logical pixels), enforced both by
// `ViewportBuilder::with_min_inner_size` (so the OS/eframe refuses to let
// the user drag it any smaller) and by clamping any saved/loaded window
// size on startup (see `app.rs::new()`) so a corrupted or degenerate saved
// size can never soft-brick the app the way it used to. Chosen small enough
// to stay flexible while still keeping the title bar controls plus one full
// row of the most important controls (Click Interval) usable and unclipped.
//
// 465 (not 420) specifically: below ~452, `ui_scale` is still WIDTH-bound
// (`w / WINDOW_WIDTH` is the smaller ratio), and in that regime the topbar's
// required content width scales up in exact lockstep with the window width
// — so growing the width alone never gained any slack, and "App Mode: Bot
// Human" stayed clipped no matter how close to 452 the width got. Past
// ~452, HEIGHT (`WINDOW_MIN_HEIGHT`) becomes the binding ratio instead, so
// `ui_scale` stops climbing and extra width turns into real usable slack.
// 465 sits a bit past that crossover so the full "Human" label (including
// its final "n") reliably clears the window edge instead of clipping.
pub const WINDOW_MIN_WIDTH: f32 = 465.0;
// Minimum window height, coupled to `WINDOW_MIN_WIDTH` above through
// `ui_scale`'s `min(width_ratio, height_ratio)` in `gui/mod.rs` — raising
// one can shift which axis is binding and change how much vertical room
// content actually needs at the minimum, so the two aren't independent.
// If `WINDOW_MIN_WIDTH` changes, this should be re-checked rather than
// assumed to still be correct. Leaves a small amount of slack below the
// Start/Hold button row at the enforced minimum size, matching the same
// gap the default view has.
pub const WINDOW_MIN_HEIGHT: f32 = 307.0;

// Uniform UI scale bounds. The whole UI — font size, spacing, padding,
// button/field sizes — scales together as one unit by the SAME factor,
// derived each frame from how the current window size compares to the
// default WINDOW_WIDTH/WINDOW_HEIGHT (see `ui_scale` in `app.rs` and its
// computation in `gui/mod.rs::ui()`). The floor keeps text legible at the
// enforced minimum window size instead of shrinking indefinitely; the
// ceiling stops it from growing unreasonably large on a very big window.
pub const UI_SCALE_MIN: f32 = 0.7;
pub const UI_SCALE_MAX: f32 = 1.5;

// Default window position — used by Reset and as the first-run fallback.
// Adjust these to wherever you want the window to appear on a fresh install.
pub const WINDOW_DEFAULT_X: f32 = 100.0;
pub const WINDOW_DEFAULT_Y: f32 = 100.0;

// ranges for click durations
pub const DURATION_CLICK_MIN: u64 = 20;
pub const DURATION_CLICK_MAX: u64 = 40;
pub const DURATION_DOUBLE_CLICK_MIN: u64 = 30;
pub const DURATION_DOUBLE_CLICK_MAX: u64 = 60;

// humanlike mouse tweening
pub const MOUSE_TWEEN_STEP_PX: f64 = 10.0;
pub const MOUSE_TWEEN_MIN_STEPS: u64 = 4;
pub const MOUSE_TWEEN_CURVE_RATIO_MIN: f64 = 0.05;
pub const MOUSE_TWEEN_CURVE_RATIO_MAX: f64 = 0.18;
pub const MOUSE_TWEEN_CURVE_MAX_PX: f64 = 120.0;
pub const MOUSE_TWEEN_TREMOR_PX: f64 = 1.5;
pub const MOUSE_TWEEN_TREMOR_DIST_THRESHOLD_PX: f64 = 480.0;
pub const MOUSE_TWEEN_TREMOR_MAX_NEAR: u64 = 1;
pub const MOUSE_TWEEN_TREMOR_MAX_FAR: u64 = 2;
pub const MOUSE_TWEEN_DELAY_JITTER_FRAC: f64 = 0.5;
pub const MOUSE_TWEEN_SPEED_MIN_PX_S: f64 = 1500.0;
pub const MOUSE_TWEEN_SPEED_MAX_PX_S: f64 = 4000.0;

// Default input values (click coords, not window position)
pub const DEFAULT_HR_STR: &str = "0";
pub const DEFAULT_MIN_STR: &str = "0";
pub const DEFAULT_SEC_STR: &str = "0";
pub const DEFAULT_MS_STR: &str = "200";
pub const DEFAULT_CLICK_AMOUNT_STR: &str = "0";
pub const DEFAULT_CLICK_X_STR: &str = "0";
pub const DEFAULT_CLICK_Y_STR: &str = "0";
/// Default +/- range (ms) shown in the Random Offset field. Only actually
/// applied to the click interval while `random_offset_enabled` is on.
pub const DEFAULT_RANDOM_OFFSET_STR: &str = "40";

// Maximum lengths for sanitized numeric inputs
pub const INPUT_LEN_TIME: usize = 5;
pub const INPUT_LEN_COORD: usize = 7;

// Hotkeys
pub const HOTKEY_AUTOCLICK: Option<Keycode> = Some(Keycode::F6);
pub const HOTKEY_OPEN_SET_COORD: Option<Keycode> = Some(Keycode::F10);
pub const HOTKEY_SET_COORD: Option<Keycode> = Some(Keycode::Escape);
pub const HOTKEY_HOLD: Option<Keycode> = Some(Keycode::F7);

/// How long the green "Set to X" / red conflict-error message under the
/// Buttons row stays visible before auto-clearing.
pub const CLICK_KEY_FEEDBACK_TIMEOUT_SECS: u64 = 5;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use device_query::{DeviceState, Keycode, MouseState};
use eframe::{egui, epaint::FontId};
use rand::{RngExt, prelude::ThreadRng, rng};
use rdev::{Button, Key};

use crate::{
    defines::*,
    settings::{self, Settings},
    types::{AppMode, ClickButton, ClickPosition, ClickType, InteractionMode},
    utils::{
        interval_ms, move_mouse_to, press_button, release_button, sanitize_i64_string,
        sanitize_string,
    },
};

/// Live data the coordinate-picker's independent (deferred) viewport reads
/// every frame to know where to draw itself. Shared via `Arc<Mutex<..>>`
/// because `show_viewport_deferred` requires a `Send + Sync + 'static`
/// callback — unlike `show_viewport_immediate`, it cannot borrow `&mut
/// self` directly, since it is designed to run independently of the
/// call site's frame lifecycle (which is exactly what makes it safe to
/// drive from `logic()`, and safe under rapid F10 spam).
#[derive(Default, Clone, Copy)]
pub struct CoordPickerShared {
    pub pos: (i32, i32),
    pub confirm_key: Option<Keycode>,
}

pub struct RustyAutoClickerApp {
    // Text input strings
    pub hr_str: String,
    pub min_str: String,
    pub sec_str: String,
    pub ms_str: String,
    pub click_amount_str: String,
    pub click_x_str: String,
    pub click_y_str: String,
    pub speed_min_str: String,
    pub speed_max_str: String,

    // Random offset (+/- jitter applied to the click interval)
    pub random_offset_enabled: bool,
    pub random_offset_str: String,

    /// The actual interval (ms) the current wait cycle is timing against —
    /// the base Click Interval, jittered by `random_offset_str` when
    /// `random_offset_enabled` is on. Rolled to a fresh random value once
    /// per cycle (`start_autoclick`, and after every successful click in
    /// `dispatch_click`) rather than every frame, so a click fires at one
    /// consistent randomly-chosen interval instead of effectively always
    /// firing at whatever the smallest interval sampled across polling
    /// frames happened to be.
    pub current_interval_ms: u64,

    // Time
    pub last_now: Instant,
    pub frame_start: Instant,

    // Counter
    pub click_counter: u64,

    // Hotkeys
    pub key_autoclick: Option<Keycode>,
    pub key_open_set_coord: Option<Keycode>,
    pub key_set_coord: Option<Keycode>,
    pub key_hold: Option<Keycode>,

    // Interaction state (mutually exclusive)
    pub mode: InteractionMode,

    // The button currently held down (click-and-hold), if any
    pub held_button: Option<ClickButton>,

    // App mode
    pub app_mode: AppMode,

    // Window state
    pub hotkey_window_open: bool,

    /// True only when THIS app auto-minimized the main window itself upon
    /// entering coordinate-setting mode (because it was visible at the
    /// time). Used so `exit_coordinate_setting` only un-minimizes the
    /// window if we're the ones who minimized it — if the window was
    /// already minimized before F10 was pressed, it correctly stays
    /// minimized after confirming coordinates too.
    pub auto_minimized_for_picker: bool,

    /// Last known inner window size [w, h] in logical pixels, for the MAIN
    /// window only. Updated every frame. The coordinate picker lives in its
    /// own viewport and never writes to this field, so it can never leak
    /// its own (much smaller) size into the persisted/main geometry.
    pub last_window_size: [f32; 2],

    /// The uniform scale factor derived from `last_window_size` vs the
    /// default [`WINDOW_WIDTH`]/[`WINDOW_HEIGHT`], recomputed once per
    /// frame at the top of `ui()` and applied globally to font size and
    /// `egui::Style::spacing` there. Section functions in
    /// `gui/sections/*.rs` also read it directly to scale their own
    /// explicit field/button sizes (which aren't covered by `Style`) by
    /// the same factor, so the whole UI — text, padding, controls, field
    /// widths — shrinks and grows together as one unit instead of each
    /// piece scaling independently.
    pub ui_scale: f32,

    /// Last known outer window position [x, y] in logical pixels, for the
    /// MAIN window only. Same "never touched by the picker" guarantee as
    /// `last_window_size` above.
    pub last_window_pos: egui::Pos2,

    // Key-down edge-detection flags
    pub key_pressed_autoclick: bool,
    pub key_pressed_open_set_coord: bool,
    pub key_pressed_esc: bool,
    pub key_pressed_hold: bool,
    pub keys_pressed: Option<Vec<Keycode>>,

    // Mouse snapshot (polled in `logic`, displayed in `ui`)
    pub mouse: MouseState,

    // Enums
    pub click_btn: ClickButton,
    pub click_type: ClickType,
    pub click_position: ClickPosition,

    /// Set while `mode == InteractionMode::SettingClickKey` when the last
    /// captured key couldn't be used — either it's already bound to one of
    /// the F6/F7/F10/etc. hotkeys, or it has no `rdev::Key` counterpart in
    /// the click-button dropdown (see `utils::keycode_to_rdev_key`).
    /// Cleared on entering capture mode and on a successful capture.
    pub click_key_capture_error: Option<String>,

    /// Set after a SUCCESSFUL capture, shown until the next capture starts.
    /// Exists so setting a key that happens to match the one already
    /// selected still gives visible confirmation — without this, that case
    /// produces no visible change anywhere (the dropdown text is identical
    /// to before), which otherwise looks exactly like nothing happened.
    pub click_key_capture_feedback: Option<String>,

    /// When `click_key_capture_feedback` was set, so it can auto-clear
    /// after `CLICK_KEY_FEEDBACK_TIMEOUT_SECS` instead of sitting there
    /// forever (or, worse, still showing the PREVIOUS key's message after
    /// you've since switched to Mouse mode or picked a different key).
    pub click_key_capture_feedback_set_at: Option<Instant>,

    /// The last `rdev::Key` selected in Keyboard mode (via the dropdown OR
    /// Press-Key capture), remembered independently of `click_btn` so that
    /// toggling Mouse -> Keyboard restores it instead of always resetting
    /// to Space. Session-only by default; persisted to disk like every
    /// other setting only when the Save button is used, and reset back to
    /// `Key::Space` by the Reset button — same rules as everything else.
    pub last_keyboard_key: Key,

    /// `click_btn` as of the end of the PREVIOUS frame, used by
    /// `show_buttons` to detect "did the key actually change" for the
    /// "Set to X" feedback message.
    ///
    /// This must be a field that persists ACROSS frames, not a local
    /// variable snapshotted at the top of `show_buttons` within the same
    /// frame — Press-Key capture mutates `click_btn` inside `logic()`,
    /// which runs BEFORE `ui()` in the same frame, so a same-frame local
    /// "before" snapshot taken in `ui()` would already equal "after" and
    /// never detect the change. A manual dropdown pick mutates `click_btn`
    /// live inside `ui()` itself, so it happened to work with a same-frame
    /// snapshot — which is exactly why an earlier version of this feature
    /// showed the confirmation for dropdown picks but not for captures.
    pub last_seen_click_btn: ClickButton,

    // RNG
    pub rng_thread: ThreadRng,

    /// Cached device-state poller — created once and reused every frame
    /// instead of being constructed anew in every `logic()` call.
    pub device_state: DeviceState,

    /// Shared with the coordinate-picker's deferred viewport (see
    /// `gui/windows.rs::show_coord_picker_viewport`).
    pub coord_picker_shared: Arc<Mutex<CoordPickerShared>>,
}

impl Default for RustyAutoClickerApp {
    fn default() -> Self {
        Self {
            hr_str: DEFAULT_HR_STR.to_owned(),
            min_str: DEFAULT_MIN_STR.to_owned(),
            sec_str: DEFAULT_SEC_STR.to_owned(),
            ms_str: DEFAULT_MS_STR.to_owned(),
            click_amount_str: DEFAULT_CLICK_AMOUNT_STR.to_owned(),
            click_x_str: DEFAULT_CLICK_X_STR.to_owned(),
            click_y_str: DEFAULT_CLICK_Y_STR.to_owned(),
            speed_min_str: MOUSE_TWEEN_SPEED_MIN_PX_S.to_string(),
            speed_max_str: MOUSE_TWEEN_SPEED_MAX_PX_S.to_string(),

            random_offset_enabled: false,
            random_offset_str: DEFAULT_RANDOM_OFFSET_STR.to_owned(),
            current_interval_ms: 0,

            last_now: Instant::now(),
            frame_start: Instant::now(),

            click_counter: 0u64,

            key_autoclick: HOTKEY_AUTOCLICK,
            key_open_set_coord: HOTKEY_OPEN_SET_COORD,
            key_set_coord: HOTKEY_SET_COORD,
            key_hold: HOTKEY_HOLD,

            mode: InteractionMode::Idle,
            held_button: None,
            app_mode: AppMode::Bot,

            hotkey_window_open: false,
            auto_minimized_for_picker: false,
            last_window_size: [WINDOW_WIDTH, WINDOW_HEIGHT],
            ui_scale: 1.0,
            last_window_pos: egui::pos2(WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y),

            key_pressed_autoclick: false,
            key_pressed_open_set_coord: false,
            key_pressed_esc: false,
            key_pressed_hold: false,
            keys_pressed: None,

            mouse: MouseState::default(),

            click_btn: ClickButton::Mouse(Button::Left),
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,
            click_key_capture_error: None,
            click_key_capture_feedback: None,
            click_key_capture_feedback_set_at: None,
            last_keyboard_key: Key::Space,
            last_seen_click_btn: ClickButton::Mouse(Button::Left),

            rng_thread: rng(),
            device_state: DeviceState::new(),
            coord_picker_shared: Arc::new(Mutex::new(CoordPickerShared::default())),
        }
    }
}

impl RustyAutoClickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        let mut style = (*ctx.global_style()).clone();
        style.override_font_id = Some(FontId { size: FONT_SIZE, family: FONT_FAMILY });
        ctx.set_global_style(style);

        let mut app = Self::default();

        let mut had_saved_position = false;

        if let Some(loaded) = settings::load_settings() {
            loaded.apply_to(&mut app);

            // Restore window size — sent as a viewport command here; the OS
            // will apply it before the first frame is painted. This is a
            // one-time startup restore, not a "resize" in the runtime sense
            // the user cares about, so it's exempt from the
            // "only user/Reset may resize" rule.
            //
            // Clamped to at least [`WINDOW_MIN_WIDTH`]/[`WINDOW_MIN_HEIGHT`]
            // before being applied: a saved size can be corrupted or
            // degenerate (hand-edited settings.json, a bug in an older
            // version, etc.), and restoring it verbatim used to be able to
            // soft-brick the app — shrinking the window down to something
            // with no usable controls and no way to resize it back up
            // without manually deleting settings.json. Clamping here means
            // that can never happen again, regardless of what's on disk.
            if let Some([w, h]) = loaded.window_size {
                let w = w.max(WINDOW_MIN_WIDTH);
                let h = h.max(WINDOW_MIN_HEIGHT);
                app.last_window_size = [w, h];
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            }

            // Restore window position. We send this command here so the
            // OS can position the window before the first paint, eliminating
            // the startup flicker where the window briefly appears at the
            // default position.
            if let Some([x, y]) = loaded.window_position {
                let pos = egui::pos2(x, y);
                app.last_window_pos = pos;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                had_saved_position = true;
            }
        }

        // Fresh install / no saved position yet: center on the actual
        // screen resolution, same as the Reset button does, instead of the
        // old hardcoded WINDOW_DEFAULT_X/Y fallback. Still sent before the
        // first frame paints, so there's no flicker on first launch either.
        if !had_saved_position
            && let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx)
        {
            if let egui::ViewportCommand::OuterPosition(pos) = cmd {
                app.last_window_pos = pos;
            }
            ctx.send_viewport_cmd(cmd);
        }

        app
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    /// Write the current settings to disk immediately (Save button).
    pub fn save_settings_now(&self) {
        settings::save_settings(&Settings::from_app(self));
    }

    /// Reset every user-configurable setting to its default value,
    /// resize the window back to the default size, and move it to the
    /// default position. Persists immediately so the reset survives the next launch.
    ///
    /// Resizing the main window at runtime now happens in exactly two
    /// places: this reset, and a manual drag by the user (handled directly
    /// by egui/the OS, never goes through this code). The layout itself
    /// reflows to whatever size the window actually is (see
    /// `gui/sections/*.rs`), so there is no longer a third,
    /// content-driven auto-resize mechanism fighting for control of the
    /// window size.
    pub fn reset_to_defaults(&mut self, ctx: &egui::Context) {
        self.hr_str = DEFAULT_HR_STR.to_owned();
        self.min_str = DEFAULT_MIN_STR.to_owned();
        self.sec_str = DEFAULT_SEC_STR.to_owned();
        self.ms_str = DEFAULT_MS_STR.to_owned();
        self.click_amount_str = DEFAULT_CLICK_AMOUNT_STR.to_owned();
        self.click_x_str = DEFAULT_CLICK_X_STR.to_owned();
        self.click_y_str = DEFAULT_CLICK_Y_STR.to_owned();
        self.speed_min_str = MOUSE_TWEEN_SPEED_MIN_PX_S.to_string();
        self.speed_max_str = MOUSE_TWEEN_SPEED_MAX_PX_S.to_string();

        self.random_offset_enabled = false;
        self.random_offset_str = DEFAULT_RANDOM_OFFSET_STR.to_owned();

        self.key_autoclick = HOTKEY_AUTOCLICK;
        self.key_open_set_coord = HOTKEY_OPEN_SET_COORD;
        self.key_set_coord = HOTKEY_SET_COORD;
        self.key_hold = HOTKEY_HOLD;

        self.click_btn = ClickButton::Mouse(Button::Left);
        self.last_seen_click_btn = self.click_btn;
        self.click_type = ClickType::Single;
        self.click_position = ClickPosition::Mouse;
        self.app_mode = AppMode::Bot;

        self.last_keyboard_key = Key::Space;
        self.click_key_capture_error = None;
        self.click_key_capture_feedback = None;
        self.click_key_capture_feedback_set_at = None;

        // Resize back to the original window size and move it back to the
        // true center of the screen it's currently on. We use egui's own
        // `ViewportCommand::center_on_screen` helper rather than manually
        // reading `viewport().monitor_size` — that field is documented as
        // commonly `None`/unreliable depending on platform and timing, which
        // is the most likely reason centering silently fell back to
        // WINDOW_DEFAULT_X/Y in practice. `center_on_screen` is egui's own
        // built-in, more robust implementation of exactly this.
        // WINDOW_DEFAULT_X/Y remains as the last-resort fallback for the
        // rare case egui itself can't determine placement.
        self.last_window_size = [WINDOW_WIDTH, WINDOW_HEIGHT];
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )));
        match egui::ViewportCommand::center_on_screen(ctx) {
            Some(cmd) => {
                if let egui::ViewportCommand::OuterPosition(pos) = cmd {
                    self.last_window_pos = pos;
                }
                ctx.send_viewport_cmd(cmd);
            }
            None => {
                let fallback = egui::pos2(WINDOW_DEFAULT_X, WINDOW_DEFAULT_Y);
                self.last_window_pos = fallback;
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(fallback));
            }
        }

        self.save_settings_now();
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    pub fn is_autoclicking(&self) -> bool { matches!(self.mode, InteractionMode::Autoclicking) }
    pub fn is_holding(&self) -> bool { matches!(self.mode, InteractionMode::Holding) }
    pub fn is_setting_coord(&self) -> bool { matches!(self.mode, InteractionMode::SettingCoord) }
    pub fn is_setting_click_key(&self) -> bool { matches!(self.mode, InteractionMode::SettingClickKey) }
    pub fn is_busy(&self) -> bool {
        // NOTE: `is_setting_click_key()` is deliberately NOT included here.
        // Capturing a click key only needs to lock the Mouse/Keyboard radio
        // (handled directly in `gui/sections/buttons.rs` via
        // `add_enabled_ui(!self.is_setting_click_key(), ...)`) — it doesn't
        // need to freeze Click Interval/Amount/Position/Movement Speed too.
        // An earlier version included it here, which dimmed all of those
        // for the capture's duration; when the captured key happened to
        // match what was already selected (no visible dropdown-text
        // change), that dimming-then-undimming was the ONLY visible thing
        // that happened, which read as "everything went black with no
        // feedback." See `click_key_capture_feedback` for the actual fix
        // (an explicit confirmation message on every successful capture).
        self.is_autoclicking() || self.is_holding() || self.hotkey_window_open || self.is_setting_coord()
    }
    pub fn is_idle(&self) -> bool { matches!(self.mode, InteractionMode::Idle) }

    pub fn disable_if_busy(&self, ui: &mut egui::Ui) {
        if self.is_busy() { ui.disable(); }
    }

    // -----------------------------------------------------------------------
    // Labels
    // -----------------------------------------------------------------------

    pub fn autoclick_button_label(&self) -> String {
        let verb = if self.is_autoclicking() { "STOP" } else { "START" };
        match self.key_autoclick {
            Some(k) => format!("🖱 {verb} ({})", crate::utils::abbreviate_keycode(k)),
            None => format!("🖱 {verb}"),
        }
    }

    pub fn hold_button_label(&self) -> String {
        let verb = if self.is_holding() { "RELEASE" } else { "HOLD" };
        match self.key_hold {
            Some(k) => format!("{verb} ({})", crate::utils::abbreviate_keycode(k)),
            None => verb.to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Input sanitization / parsing
    // -----------------------------------------------------------------------

    pub fn sanitize_inputs(&mut self) {
        sanitize_string(&mut self.hr_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.min_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.sec_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.ms_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.click_amount_str, INPUT_LEN_TIME);
        sanitize_i64_string(&mut self.click_x_str, INPUT_LEN_COORD);
        sanitize_i64_string(&mut self.click_y_str, INPUT_LEN_COORD);
        sanitize_string(&mut self.speed_min_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.speed_max_str, INPUT_LEN_TIME);
        sanitize_string(&mut self.random_offset_str, INPUT_LEN_TIME);
    }

    pub fn parsed_interval_ms(&self) -> u64 {
        interval_ms(
            self.hr_str.parse().unwrap_or_default(),
            self.min_str.parse().unwrap_or_default(),
            self.sec_str.parse().unwrap_or_default(),
            self.ms_str.parse().unwrap_or_default(),
        )
    }

    pub fn parsed_speed_range(&self) -> (f64, f64) {
        let min = self.speed_min_str.parse().unwrap_or(MOUSE_TWEEN_SPEED_MIN_PX_S).max(1.0);
        let max = self.speed_max_str.parse().unwrap_or(MOUSE_TWEEN_SPEED_MAX_PX_S).max(min);
        (min, max)
    }

    pub fn parsed_click_amount(&self) -> u64 { self.click_amount_str.parse().unwrap_or_default() }

    pub fn parsed_random_offset_ms(&self) -> u64 { self.random_offset_str.parse().unwrap_or_default() }

    pub fn parsed_click_coord(&self) -> (f64, f64) {
        (
            self.click_x_str.parse().unwrap_or_default(),
            self.click_y_str.parse().unwrap_or_default(),
        )
    }

    // -----------------------------------------------------------------------
    // Coordinate-setting mode
    // -----------------------------------------------------------------------
    //
    // The coordinate picker is ALWAYS rendered in its own independent egui
    // viewport (see `gui/windows.rs::show_coord_picker_viewport`) at a fixed
    // 500x32 size — it is never resized, in any code path, ever.
    //
    // The MAIN window itself is never resized here either. The only thing
    // this code does to the main window is minimize/restore it:
    //   * If the main window is VISIBLE when F10 fires, we minimize it so
    //     only the picker is on screen while picking, then restore it (un-
    //     minimize, back to its exact prior size/position — the OS remembers
    //     that) once coordinates are confirmed.
    //   * If the main window is ALREADY minimized when F10 fires, we leave
    //     it alone entirely — it stays minimized before, during, and after
    //     picking. No restore, no animation, no pop-up.
    // `auto_minimized_for_picker` is what makes this distinction: it's only
    // `true` when THIS code performed the minimize, so only then do we
    // reverse it on exit.

    /// Returns `true` when the OS reports the main window as minimized.
    pub fn is_window_minimized(ctx: &egui::Context) -> bool {
        ctx.input(|i| i.viewport().minimized).unwrap_or(false)
    }

    /// Enter coordinate-setting mode. If the main window is currently
    /// visible, minimize it (so only the fixed-size picker is on screen);
    /// if it's already minimized, leave it exactly as-is.
    pub fn enter_coordinate_setting(&mut self, ctx: &egui::Context) {
        if !self.is_idle() {
            return;
        }
        self.mode = InteractionMode::SettingCoord;

        if Self::is_window_minimized(ctx) {
            self.auto_minimized_for_picker = false;
        } else {
            self.auto_minimized_for_picker = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }

    /// Exit coordinate-setting mode. Restores the main window from minimized
    /// only if `enter_coordinate_setting` was the one that minimized it.
    pub fn exit_coordinate_setting(&mut self, ctx: &egui::Context) {
        self.mode = InteractionMode::Idle;
        self.click_position = ClickPosition::Coord;

        if self.auto_minimized_for_picker {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        }
        self.auto_minimized_for_picker = false;
    }

    // -----------------------------------------------------------------------
    // Autoclick / hold
    // -----------------------------------------------------------------------

    pub fn start_autoclick(&mut self, negative_click_start_offset: u64) {
        self.click_counter = 0u64;
        self.mode = InteractionMode::Autoclicking;
        self.rng_thread = rng();
        self.last_now = Instant::now()
            .checked_sub(Duration::from_millis(negative_click_start_offset))
            .unwrap();
        self.current_interval_ms = self.roll_next_interval(self.parsed_interval_ms());
    }

    /// The interval (ms) to actually wait for THIS click cycle: the base
    /// Click Interval as-is when Random Offset is off (or its field is
    /// "0"/blank), otherwise the base interval jittered by a fresh random
    /// amount in `[-offset, +offset]`, floored at 0 so a large offset can
    /// shrink the wait but never produce a negative one.
    pub fn roll_next_interval(&mut self, base_interval: u64) -> u64 {
        if !self.random_offset_enabled {
            return base_interval;
        }
        let offset = self.parsed_random_offset_ms();
        if offset == 0 {
            return base_interval;
        }
        let offset = offset as i64;
        let jitter = self.rng_thread.random_range(-offset..=offset);
        (base_interval as i64 + jitter).max(0) as u64
    }

    pub fn start_hold(&mut self) {
        if self.click_position == ClickPosition::Coord {
            move_mouse_to(
                self.app_mode,
                self.parsed_click_coord(),
                self.mouse.coords,
                self.parsed_speed_range(),
                &mut self.rng_thread,
            );
        }
        press_button(self.click_btn);
        self.held_button = Some(self.click_btn);
        self.mode = InteractionMode::Holding;
    }

    pub fn stop_hold(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
        self.mode = InteractionMode::Idle;
    }

    // -----------------------------------------------------------------------
    // Click-key capture ("press the key you want the autoclicker to use")
    // -----------------------------------------------------------------------

    /// Enter click-key capture mode. Called from the "Press Key" button in
    /// `gui/sections/buttons.rs`. Clears any stale error from a previous
    /// capture attempt.
    pub fn enter_click_key_capture(&mut self) {
        if !self.is_idle() {
            return;
        }
        self.click_key_capture_error = None;
        self.click_key_capture_feedback = None;
        self.mode = InteractionMode::SettingClickKey;
    }

    /// Cancel click-key capture without changing `click_btn`.
    pub fn cancel_click_key_capture(&mut self) {
        self.click_key_capture_error = None;
        self.mode = InteractionMode::Idle;
    }

    /// Auto-clear the "Set to X" feedback message after
    /// `CLICK_KEY_FEEDBACK_TIMEOUT_SECS`. Called once per frame from
    /// `logic()`. Deliberately does NOT clear `click_key_capture_error` —
    /// an unresolved conflict/unsupported-key error should stay visible
    /// until the user acts on it (switches to Mouse, picks another key, or
    /// cancels), not silently disappear on a timer.
    pub fn tick_click_key_feedback_timeout(&mut self) {
        let expired = self
            .click_key_capture_feedback_set_at
            .is_some_and(|set_at| set_at.elapsed() >= Duration::from_secs(CLICK_KEY_FEEDBACK_TIMEOUT_SECS));
        if expired {
            self.click_key_capture_feedback = None;
            self.click_key_capture_feedback_set_at = None;
        }
    }

    /// Look for a key that was pressed last frame and is no longer pressed
    /// this frame (same released-edge pattern as `capture_key` in
    /// `gui/mod.rs`), and try to use it as the click button.
    ///
    /// Rejects (sets `click_key_capture_error`, stays in capture mode so the
    /// user can try again) when:
    ///   - the key is already bound to one of the global hotkeys
    ///     (start/stop, open-coord-picker, confirm-coord, hold), since a
    ///     shared key would make it impossible to tell which action fired,
    ///   - the key has no `rdev::Key` counterpart in the dropdown, in which
    ///     case only the manual dropdown can select it.
    pub fn capture_click_key(&mut self, last_keys: Option<&[Keycode]>, keys: &[Keycode]) {
        let Some(last_keys) = last_keys else { return };
        for pressed_key in last_keys {
            if keys.contains(pressed_key) {
                continue;
            }
            let pressed_key = *pressed_key;

            let conflict = [
                (self.key_autoclick, "Start/Stop"),
                (self.key_open_set_coord, "Set Coords"),
                (self.key_set_coord, "Confirm Coords"),
                (self.key_hold, "Click & Hold"),
            ]
            .into_iter()
            .find_map(|(hotkey, label)| (hotkey == Some(pressed_key)).then_some(label));

            if let Some(label) = conflict {
                self.click_key_capture_error = Some(format!(
                    "\"{}\" is already the \"{label}\" hotkey — pick a different key",
                    crate::utils::abbreviate_keycode(pressed_key)
                ));
                continue;
            }

            match crate::utils::keycode_to_rdev_key(pressed_key) {
                Some(key) => {
                    self.click_btn = ClickButton::Key(key);
                    self.click_key_capture_error = None;
                    self.mode = InteractionMode::Idle;
                }
                None => {
                    self.click_key_capture_error = Some(format!(
                        "\"{}\" isn't supported for auto-capture — pick it from the dropdown instead",
                        crate::utils::abbreviate_keycode(pressed_key)
                    ));
                }
            }
            break;
        }
    }
}

impl Drop for RustyAutoClickerApp {
    /// On close: release any held button, then write the final window
    /// geometry (size + position) to disk without touching other settings.
    fn drop(&mut self) {
        if let Some(button) = self.held_button.take() {
            release_button(button);
        }
        settings::save_window_geometry(
            self.last_window_size[0],
            self.last_window_size[1],
            self.last_window_pos.x,
            self.last_window_pos.y,
        );
    }
}

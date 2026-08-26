use std::time::{Duration, Instant};

use device_query::{DeviceQuery, Keycode, MouseState};
use eframe::{egui, epaint::FontId};

use crate::{
    RustyAutoClickerApp,
    defines::{FONT_FAMILY, FONT_SIZE, UI_SCALE_MAX, UI_SCALE_MIN, WINDOW_HEIGHT, WINDOW_WIDTH},
    types::{ClickInfo, InteractionMode},
    utils::autoclick,
};

mod sections;
mod windows;

impl RustyAutoClickerApp {
    /// Poll mouse and keyboard from the cached `DeviceState` (created once in
    /// `Default::default()`, never reconstructed). This avoids the overhead of
    /// re-initializing the OS device handle on every frame.
    fn poll_devices(&mut self) -> (MouseState, Vec<Keycode>) {
        let mouse = self.device_state.get_mouse();
        let keys = self.device_state.get_keys();
        self.mouse = mouse.clone();
        (mouse, keys)
    }

    fn handle_hotkey_window_escape(&mut self, keys: &[Keycode]) {
        if !self.hotkey_window_open { return; }
        if keys.contains(&Keycode::Escape) {
            self.key_pressed_esc = true;
        } else if self.key_pressed_esc {
            if self.is_idle() { self.hotkey_window_open = false; }
            self.key_pressed_esc = false;
        }
    }

    /// Fires the toggle on the PRESS edge (not release). This needs only
    /// ONE poll to land — the one that happens to catch the key already
    /// down — instead of needing a poll to catch it going down AND a later
    /// poll to catch it coming back up. `device_query` polls key state
    /// rather than receiving true OS key-down/key-up events, and the poll
    /// rate can drop noticeably while the window is minimized (Windows can
    /// throttle background windows' event loops, which this app's own
    /// `logic()`/`ui()` split can't fully escape since both still share
    /// the same underlying event-loop wakeup) — so halving how many polls
    /// a single tap needs to register meaningfully cuts the miss rate.
    fn handle_autoclick_toggle(&mut self, keys: &[Keycode], interval: u64) {
        let is_pressed = self.key_autoclick.is_some() && keys.contains(&self.key_autoclick.unwrap());
        if is_pressed && !self.key_pressed_autoclick {
            self.key_pressed_autoclick = true;
            if self.is_autoclicking() {
                self.mode = InteractionMode::Idle;
            } else if self.is_idle() && !self.hotkey_window_open {
                self.start_autoclick(interval);
            }
        } else if !is_pressed {
            self.key_pressed_autoclick = false;
        }
    }

    /// Same press-edge fix as `handle_autoclick_toggle` above, for the same
    /// reason — see that function's doc comment.
    fn handle_hold_toggle(&mut self, keys: &[Keycode]) {
        let is_pressed = self.key_hold.is_some() && keys.contains(&self.key_hold.unwrap());
        if is_pressed && !self.key_pressed_hold {
            self.key_pressed_hold = true;
            if self.is_holding() {
                self.stop_hold();
            } else if self.is_idle() && !self.hotkey_window_open {
                self.start_hold();
            }
        } else if !is_pressed {
            self.key_pressed_hold = false;
        }
    }

    /// Handle the "open coordinate picker" hotkey (F10).
    ///
    /// If the main window is visible, this minimizes it (so only the
    /// picker is on screen) via `enter_coordinate_setting`; if it's already
    /// minimized, it's left untouched. Entering the mode is a plain,
    /// idempotent state transition guarded by `is_idle()`, so holding/
    /// spamming F10 can never double-fire, crash, or fight with itself.
    fn handle_open_set_coord_toggle(&mut self, ctx: &egui::Context, keys: &[Keycode]) {
        if self.key_open_set_coord.is_some()
            && keys.contains(&self.key_open_set_coord.unwrap())
        {
            self.key_pressed_open_set_coord = true;
        } else if self.key_pressed_open_set_coord {
            self.key_pressed_open_set_coord = false;
            if self.is_idle() && !self.hotkey_window_open {
                self.enter_coordinate_setting(ctx);
            }
        }
    }

    fn dispatch_click(
        &mut self,
        now: Instant,
        interval: u64,
        click_amount: u64,
        click_coord: (f64, f64),
        speed_range: (f64, f64),
        mouse_coord: (i32, i32),
    ) -> bool {
        let since_last = now
            .checked_duration_since(self.last_now)
            .unwrap_or(Duration::ZERO);
        // Gate on `current_interval_ms`, not the raw base `interval` — see
        // that field's doc comment in `app.rs`. It's the base interval,
        // possibly jittered by Random Offset, rolled fresh once per cycle
        // rather than re-sampled every frame.
        if !self.is_autoclicking() || (since_last.as_millis() as u64) < self.current_interval_ms {
            return false;
        }

        #[cfg(debug_assertions)]
        println!("{:?} {:?} Click: {since_last:?}", self.click_type, self.click_btn);
        self.last_now = Instant::now();

        autoclick(
            self.app_mode,
            ClickInfo {
                click_btn: self.click_btn,
                click_coord,
                click_position: self.click_position,
                click_type: self.click_type,
            },
            mouse_coord,
            speed_range,
            self.rng_thread.clone(),
        );

        self.click_counter += 1u64;
        if click_amount != 0u64 && self.click_counter >= click_amount {
            self.mode = InteractionMode::Idle;
        } else {
            // Roll the next cycle's (possibly jittered) interval now, off
            // the latest live base `interval`, so Random Offset re-rolls a
            // fresh value every click rather than reusing one draw for the
            // whole session.
            self.current_interval_ms = self.roll_next_interval(interval);
        }
        true
    }

    fn capture_key(
        target_key: &mut Option<Keycode>,
        mode: &mut InteractionMode,
        last_keys: Option<&[Keycode]>,
        keys: &[Keycode],
    ) {
        let Some(last_keys) = last_keys else { return };
        for pressed_key in last_keys {
            if !keys.contains(pressed_key) {
                *target_key = Some(*pressed_key);
                *mode = InteractionMode::Idle;
                break;
            }
        }
    }

    fn ensure_hotkey_window(&mut self) {
        if !self.hotkey_window_open
            && (self.key_autoclick.is_none()
                || self.key_open_set_coord.is_none()
                || self.key_set_coord.is_none()
                || self.key_hold.is_none())
        {
            self.hotkey_window_open = true;
        }
    }

    /// Track window size and position every frame so `Drop` can persist the
    /// final geometry on close.
    ///
    /// **Guarded to the ROOT viewport only.** `show_coord_picker_viewport`
    /// (called just above this, at line ~215) registers/drives the picker's
    /// deferred viewport via the SAME shared `egui::Context`. Under rapid
    /// F10 + confirm spam, egui's internal "current viewport" bookkeeping
    /// can transiently still be set to the picker's ID at the moment this
    /// function runs afterward in the same `logic()` call — in which case
    /// `ctx.input(|i| i.viewport())` would report the picker's tiny
    /// inner_rect/outer_rect instead of the main window's, and we'd persist
    /// THAT as the main window's saved geometry. That is the exact
    /// mechanism behind "main window shrinks to picker size" under rapid
    /// spam. Explicitly checking `ctx.viewport_id() == ViewportId::ROOT`
    /// makes it structurally impossible to record anything but the real
    /// main-window geometry here, regardless of timing.
    fn track_window_geometry(&mut self, ctx: &egui::Context) {
        if ctx.viewport_id() != egui::ViewportId::ROOT {
            return;
        }
        ctx.input(|i| {
            let vp = i.viewport();
            if let Some(rect) = vp.inner_rect {
                self.last_window_size = [rect.width(), rect.height()];
            }
            if let Some(rect) = vp.outer_rect {
                self.last_window_pos = rect.min;
            }
        });
    }

    /// Recompute `ui_scale` from the current window size and apply it
    /// globally: font size (via `override_font_id`, the single knob that
    /// controls every widget's text since `app.rs::new()` set it once at
    /// startup) and `egui::Style::spacing` (padding, control sizes, gaps).
    /// This is what makes text, buttons, and spacing all shrink or grow
    /// together as ONE unit as the window is resized, rather than only
    /// the whitespace between them compressing while text/controls stay a
    /// fixed size (which is what clipped text and still needed a
    /// scrollbar at the enforced minimum in an earlier version of this).
    ///
    /// `egui::widgets::Separator`'s own thickness isn't part of `Style`
    /// (it's a hardcoded constant inside egui itself), so section code
    /// that wants a separator to also scale needs to pass
    /// `.spacing(6.0 * self.ui_scale)` explicitly rather than using the
    /// bare `ui.separator()` shorthand — see `gui/mod.rs::ui()`'s content
    /// list below.
    fn update_ui_scale(&mut self, ctx: &egui::Context) {
        // Simple ratio against the default size on both axes. This only
        // lands at (approximately) 1.0 — i.e. pixel-parity with the
        // original, pre-scaling design — at the actual default window
        // size because WINDOW_HEIGHT itself was chosen (see its doc
        // comment in `defines.rs`) to be tall enough for this content at
        // 1:1 scale; without that, a plain size ratio would have silently
        // scaled everything down below 1.0 even at the "normal" size to
        // compensate for content that didn't actually fit.
        let [w, h] = self.last_window_size;
        self.ui_scale = ((w / WINDOW_WIDTH).min(h / WINDOW_HEIGHT))
            .clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        let scale = self.ui_scale;

        ctx.global_style_mut(|style| {
            style.override_font_id =
                Some(FontId { size: (FONT_SIZE * scale).max(1.0), family: FONT_FAMILY });

            // `egui::style::Spacing::default()` as the fixed baseline that
            // gets scaled — NOT the current live `style.spacing` — so this
            // is idempotent frame to frame instead of compounding.
            let base = egui::style::Spacing::default();
            style.spacing.item_spacing = base.item_spacing * scale;
            style.spacing.button_padding = base.button_padding * scale;
            style.spacing.interact_size = base.interact_size * scale;
            style.spacing.icon_width = base.icon_width * scale;
            style.spacing.icon_width_inner = base.icon_width_inner * scale;
            style.spacing.icon_spacing = base.icon_spacing * scale;
            style.spacing.combo_width = base.combo_width * scale;
            style.spacing.text_edit_width = base.text_edit_width * scale;
            style.spacing.indent = base.indent * scale;
        });
    }
}

impl eframe::App for RustyAutoClickerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        println!(
            "Frame delta: {:?}",
            Instant::now().checked_duration_since(self.frame_start).unwrap()
        );
        self.frame_start = Instant::now();

        let (mouse, keys) = self.poll_devices();

        self.sanitize_inputs();
        let interval    = self.parsed_interval_ms();
        let speed_range = self.parsed_speed_range();
        let click_amount = self.parsed_click_amount();
        let click_coord = self.parsed_click_coord();

        self.handle_hotkey_window_escape(&keys);

        let update_now = Instant::now();
        self.handle_autoclick_toggle(&keys, interval);
        self.handle_hold_toggle(&keys);
        self.handle_open_set_coord_toggle(ctx, &keys);

        if !self.dispatch_click(update_now, interval, click_amount, click_coord, speed_range, mouse.coords) {
            if matches!(self.mode, InteractionMode::SettingAutoclickKey) && self.keys_pressed.is_some() {
                Self::capture_key(&mut self.key_autoclick, &mut self.mode, self.keys_pressed.as_deref(), &keys);
            } else if matches!(self.mode, InteractionMode::SettingOpenCoordKey) && self.keys_pressed.is_some() {
                Self::capture_key(&mut self.key_open_set_coord, &mut self.mode, self.keys_pressed.as_deref(), &keys);
            } else if matches!(self.mode, InteractionMode::SettingSetCoordKey) && self.keys_pressed.is_some() {
                Self::capture_key(&mut self.key_set_coord, &mut self.mode, self.keys_pressed.as_deref(), &keys);
            } else if matches!(self.mode, InteractionMode::SettingHoldKey) && self.keys_pressed.is_some() {
                Self::capture_key(&mut self.key_hold, &mut self.mode, self.keys_pressed.as_deref(), &keys);
            } else if matches!(self.mode, InteractionMode::SettingClickKey) {
                if keys.contains(&Keycode::Escape) {
                    self.cancel_click_key_capture();
                } else if let Some(last_keys) = self.keys_pressed.clone() {
                    self.capture_click_key(Some(&last_keys), &keys);
                }
            }
        }

        // Drive the coordinate-picker viewport from `logic()`, NOT `ui()`.
        // `ui()` is skipped by the OS while the main window is minimized,
        // but `logic()` always runs — this is what lets F10 keep working
        // (opening, following the cursor, and confirming) even while the
        // main window stays minimized the entire time, with no restore.
        if self.is_setting_coord() {
            self.show_coord_picker_viewport(ctx, &mouse, &keys);
        }

        self.keys_pressed = Some(keys);
        self.ensure_hotkey_window();
        self.track_window_geometry(ctx);
        self.tick_click_key_feedback_timeout();

        // Always request a repaint so hotkeys are polled every frame.
        // egui already throttles to ~60 fps so this does not cause excessive
        // CPU usage — but it ensures F6/F7/F10 are never delayed by a sleep.
        ctx.request_repaint();

        #[cfg(debug_assertions)]
        println!(
            "Frame time: {:?}",
            Instant::now().checked_duration_since(self.frame_start).unwrap()
        );
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.update_ui_scale(&ctx);
        let sep_spacing = 6.0 * self.ui_scale;
        let separator = |ui: &mut egui::Ui| {
            ui.add(egui::Separator::default().spacing(sep_spacing));
        };

        // The main window is ALWAYS rendered at its normal size — the
        // coordinate picker never replaces or resizes it. While picking
        // (`is_setting_coord()`), the main UI is simply disabled via
        // `is_busy()`/`disable_if_busy()`, exactly like autoclicking or
        // holding disable it.
        self.show_topbar(ui);
        self.show_bottombar(ui);

        let click_amount = self.parsed_click_amount();

        // `CentralPanel::default()`'s frame has a fixed (unscaled) 8px
        // margin on every side, and `show_bottombar`'s panel has its own
        // fixed 2px top margin on top of that — combined, that's ~10px of
        // margin below the Start/Hold buttons that's entirely independent
        // of the small, deliberate `gap` added in `show_autoclicker`,
        // which is why the bottom gap looked noticeably bigger than the
        // top one even though the button code adds equal space on both
        // sides. Zeroing just the bottom inset here (leaving left/right/
        // top at the normal 8px) puts `show_autoclicker`'s `gap` back in
        // sole control of both the top AND bottom spacing around the
        // buttons.
        // No `ScrollArea` here — deliberately removed. Since the whole UI
        // already scales down to fit the window (see `update_ui_scale`
        // above), a scrollbar was redundant with that mechanism and just
        // added its own gutter (clipping the rightmost pixels of every
        // row) whenever content came up even slightly short. Instead,
        // `WINDOW_MIN_HEIGHT` (see `defines.rs`) was calibrated so this
        // content always fits at the enforced minimum window size without
        // needing to scroll at all.
        let mut central_frame = egui::Frame::central_panel(&ctx.global_style());
        central_frame.inner_margin.bottom = 0;
        egui::CentralPanel::default().frame(central_frame).show_inside(ui, |ui| {
            self.show_click_interval(ui);
            separator(ui);
            self.show_random_offset(ui);
            separator(ui);
            self.show_movement_speed(ui);
            separator(ui);
            self.show_buttons(ui);
            separator(ui);
            self.show_click_type(ui);
            separator(ui);
            self.show_click_amount(ui, click_amount);
            separator(ui);
            self.show_click_position(ui);
            separator(ui);
            self.show_infos(ui, &self.mouse, self.keys_pressed.as_deref().unwrap_or(&[]));
            separator(ui);
            self.show_autoclicker(ui);
        });

        if self.hotkey_window_open {
            self.show_hotkeys_window(&ctx);
        }
    }
}

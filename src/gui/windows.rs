use std::sync::Arc;

use device_query::{Keycode, MouseState};
use eframe::egui::{self, Context};

use crate::{RustyAutoClickerApp, types::InteractionMode};

/// Fixed size for the coordinate-picker viewport. This is intentionally
/// hardcoded and NEVER changes at runtime — not by the user, not by any
/// button, not by any setting. `.with_resizable(false)` below additionally
/// makes it impossible for the user to drag-resize it even if they tried.
const COORD_PICKER_SIZE: egui::Vec2 = egui::vec2(400.0, 65.0);
/// Vertical gap between the cursor tip and the top of the picker window.
/// Horizontally the picker is centered under the cursor instead of offset
/// to the side — see `clamped_picker_pos` below for why this alone isn't
/// enough to keep it fully on-screen.
const COORD_PICKER_VERTICAL_GAP: f32 = 16.0;

/// Compute where the picker window should sit for a given (logical) cursor
/// position: horizontally centered under the cursor, `COORD_PICKER_VERTICAL_GAP`
/// below it (so the cursor never overlaps the window), then clamped to stay
/// fully within the current monitor so it can't get cut off against the
/// right/bottom edge the way a fixed offset could.
///
/// Note: `ViewportInfo::monitor_size` only gives the monitor's *size*, not
/// its origin, so this assumes the monitor's origin is (0, 0) — correct for
/// the primary monitor / single-monitor setups. On a multi-monitor layout
/// where the picker is on a secondary monitor positioned to the left of or
/// above the primary, the clamp can be slightly off; a real fix needs
/// per-monitor origin info egui doesn't currently expose here.
fn clamped_picker_pos(monitor_size: Option<egui::Vec2>, cursor: egui::Pos2) -> egui::Pos2 {
    let mut x = cursor.x - COORD_PICKER_SIZE.x / 2.0;
    let mut y = cursor.y + COORD_PICKER_VERTICAL_GAP;
    if let Some(size) = monitor_size {
        x = x.clamp(0.0, (size.x - COORD_PICKER_SIZE.x).max(0.0));
        y = y.clamp(0.0, (size.y - COORD_PICKER_SIZE.y).max(0.0));
    }
    egui::pos2(x, y)
}

impl RustyAutoClickerApp {
    pub fn show_hotkeys_window(&mut self, ctx: &Context) {
        let idle = self.is_idle();
        egui::Window::new("Hotkeys")
            .fixed_size(egui::vec2(260f32, 190f32))
            .anchor(egui::Align2::CENTER_CENTER, [0f32, 0f32])
            .collapsible(false)
            .open(&mut self.hotkey_window_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [120.0f32, 32.0f32],
                            egui::widgets::Button::new("Start/Stop"),
                        )
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.mode = InteractionMode::SettingAutoclickKey;
                            self.key_autoclick = None;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_autoclick {
                            crate::utils::abbreviate_keycode(pressed_keys).to_string()
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([130.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [120.0f32, 32.0f32],
                            egui::widgets::Button::new("Set Coords"),
                        )
                        .on_hover_text("Opens the same coordinate picker as the \"Set Coords\" button")
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.key_open_set_coord = None;
                            self.mode = InteractionMode::SettingOpenCoordKey;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_open_set_coord {
                            crate::utils::abbreviate_keycode(pressed_keys).to_string()
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([130.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [120.0f32, 32.0f32],
                            egui::widgets::Button::new("Confirm Coords"),
                        )
                        .on_hover_text("Note: L Click cannot be changed")
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.key_set_coord = None;
                            self.mode = InteractionMode::SettingSetCoordKey;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_set_coord {
                            format!("{} / L Click", crate::utils::abbreviate_keycode(pressed_keys))
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([130.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [120.0f32, 32.0f32],
                            egui::widgets::Button::new("Click & Hold"),
                        )
                        .clicked()
                    {
                        // Allow keybind only if app is not busy
                        if idle {
                            self.key_hold = None;
                            self.mode = InteractionMode::SettingHoldKey;
                        }
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.disable();
                        let text: String = if let Some(pressed_keys) = self.key_hold {
                            crate::utils::abbreviate_keycode(pressed_keys).to_string()
                        } else {
                            "PRESS ANY KEY".to_string()
                        };
                        ui.add_sized([130.0f32, 32.0f32], egui::widgets::Button::new(text));
                    });
                });
            });
    }

    /// Render the coordinate picker as its own independent, borderless egui
    /// viewport that follows the mouse cursor.
    ///
    /// **Appearance (Issue #1 fix):** rebuilt to match the original v2.5.1
    /// in-window overlay exactly — `egui::Panel::top` + `egui::MenuBar` +
    /// `horizontal_wrapped` in a `right_to_left` layout, with the SAME
    /// widget order 2.5.1 used: Y-edit, "Y", X-edit, "X", separator,
    /// "Set with ..." label. No manual `Frame::fill(BLACK)` override — the
    /// panel uses the app's normal dark-theme panel styling (set via
    /// `picker_ctx.set_visuals(egui::Visuals::dark())` below), which is what
    /// produced the near-black bar in 2.5.1 in the first place. This is
    /// visually and structurally identical to the original, just hosted in
    /// its own OS-level window instead of the main window's top panel.
    ///
    /// **Real-time X/Y (Issue #2 fix):** the previous version relied solely
    /// on the PARENT calling `ctx.request_repaint_of(viewport_id)` once per
    /// frame to nudge this viewport awake. That's a best-effort hint, not a
    /// guarantee, and in practice the picker would paint once on creation
    /// and then go dormant — frozen until the next F10 press recreated it.
    /// The fix: the picker's own closure now also calls
    /// `picker_ctx.request_repaint()` on itself every time it runs, the same
    /// way the main window sustains its own loop via `ctx.request_repaint()`
    /// in `gui/mod.rs`. This makes the picker a self-sustaining repaint loop
    /// instead of depending on an external nudge, restoring true real-time
    /// X/Y tracking with no added CPU cost (it's still one lightweight
    /// repaint of a tiny single-row bar per frame, exactly like before).
    pub fn show_coord_picker_viewport(&mut self, ctx: &Context, mouse: &MouseState, keys: &[Keycode]) {
        let (mx, my) = mouse.coords;

        // Publish this frame's data for the deferred viewport to read.
        {
            let mut shared = self.coord_picker_shared.lock().unwrap();
            shared.pos = (mx, my);
            shared.confirm_key = self.key_set_coord;
        }

        // `device_query` reports mouse coords in PHYSICAL screen pixels, but
        // egui's viewport positioning APIs (`with_position`,
        // `ViewportCommand::OuterPosition`) expect LOGICAL points. On any
        // display scale other than 100% these are not the same number, and
        // the picker drifts further from the real cursor the further it is
        // from the monitor's origin. Divide by `pixels_per_point()` to
        // convert physical -> logical before positioning.
        let ppp = ctx.pixels_per_point();
        let cursor_logical = egui::pos2(mx as f32 / ppp, my as f32 / ppp);
        let monitor_size = ctx.input(|i| i.viewport().monitor_size);
        let target_pos = clamped_picker_pos(monitor_size, cursor_logical);
        let viewport_id = egui::ViewportId::from_hash_of("rusty_autoclicker_coord_picker");

        // Request the repaint BEFORE (re-)registering the viewport this
        // frame, not after. Ordering matters for deferred viewports: queuing
        // the repaint first ensures it's honored on the same pass rather
        // than potentially landing after this frame's viewport bookkeeping
        // has already been finalized. This is one of a few redundant nudges
        // below aimed at making the real-time X/Y update reliable — deferred
        // viewports are a fairly new part of this eframe version, and relying
        // on a single repaint call proved unreliable while the app is
        // windowed (foreground), where the always-on-top main window appears
        // to dominate the event loop's attention.
        ctx.request_repaint_of(viewport_id);

        // Always-on-top so the picker overlays whatever else is on screen —
        // any other app's window, just like v2.5.1. This is now safe to
        // enable: the main window auto-minimizes itself the moment picking
        // starts (see `enter_coordinate_setting` in app.rs), so there is
        // never a second always-on-top window visible at the same time to
        // fight over z-order. `.with_resizable(false)` ensures the fixed
        // 516x85 size can never be changed by the user, even by attempting
        // to drag an edge.
        let builder = egui::ViewportBuilder::default()
            .with_inner_size(COORD_PICKER_SIZE)
            .with_position(target_pos)
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top();

        let shared = Arc::clone(&self.coord_picker_shared);
        ctx.show_viewport_deferred(viewport_id, builder, move |picker_ctx, _class| {
            // Match the main window's theme so the panel's fill color is
            // identical to the original in-window overlay's background.
            picker_ctx.set_visuals(egui::Visuals::dark());

            let (pos, confirm_key) = {
                let s = shared.lock().unwrap();
                (s.pos, s.confirm_key)
            };
            let (mx, my) = pos;

            // Reposition every frame unconditionally (no dedup). The earlier
            // "only move if changed" optimization existed to avoid
            // compositor churn from TWO always-on-top windows fighting for
            // z-order; since only the main window is always-on-top now,
            // that concern doesn't apply here, and always repositioning
            // removes one more variable from the "why isn't this updating"
            // question.
            // Same physical -> logical conversion as the parent side above.
            // Use THIS viewport's own `pixels_per_point()`, not the root's —
            // if the picker is dragged onto a monitor with a different scale
            // factor than the main window's monitor, the two can legitimately
            // differ.
            let picker_ppp = picker_ctx.pixels_per_point();
            let cursor_logical = egui::pos2(mx as f32 / picker_ppp, my as f32 / picker_ppp);
            let monitor_size = picker_ctx.input(|i| i.viewport().monitor_size);
            let target = clamped_picker_pos(monitor_size, cursor_logical);
            picker_ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(target));

            let mut y_str = my.to_string();
            let mut x_str = mx.to_string();

            // Same container + widget order as the original v2.5.1 overlay:
            // Panel::top -> MenuBar -> horizontal_wrapped -> right_to_left
            // layout of [Y edit, "Y", X edit, "X", separator, "Set with..."].
            egui::Panel::top("coord_picker_panel").show_inside(picker_ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut y_str)
                                    .desired_width(50.0)
                                    .hint_text("0"),
                            );
                            ui.label("Y");
                            ui.add(
                                egui::TextEdit::singleline(&mut x_str)
                                    .desired_width(50.0)
                                    .hint_text("0"),
                            );
                            ui.label("X");
                            ui.separator();
                            let key_label = confirm_key
                                .map(|k| crate::utils::abbreviate_keycode(k).to_string())
                                .unwrap_or_else(|| "no key set".to_string());
                            ui.label(format!(" Set with \"{key_label}\" / \"L Click\""));
                        });
                    });
                });
            });

            // Keep this viewport repainting continuously on its OWN
            // schedule. Two redundant calls here on purpose: `request_repaint`
            // schedules the next frame ASAP, and `request_repaint_after` with
            // a zero duration is a slightly different code path in some
            // eframe versions. Belt-and-suspenders against the deferred-
            // viewport repaint scheduling proving unreliable in this
            // specific eframe version while the app is windowed.
            picker_ctx.request_repaint();
            picker_ctx.request_repaint_after(std::time::Duration::ZERO);
        });

        // Nudge again from the parent after registration too, in addition to
        // the nudge sent before `show_viewport_deferred` above.
        ctx.request_repaint_of(viewport_id);

        // Confirmation detection stays here in `logic()`, driven by the
        // same global device-state poll used everywhere else in the app —
        // it does not depend on the picker viewport having painted at all,
        // so it keeps working even while minimized.
        let confirm_key_down = self.key_set_coord.is_some_and(|k| keys.contains(&k));
        // device_query's mouse.button_pressed is 1-indexed; index 1 = Left click.
        let confirm_click = mouse.button_pressed.get(1).copied().unwrap_or(false);

        self.click_x_str = mx.to_string();
        self.click_y_str = my.to_string();

        if confirm_click || confirm_key_down {
            self.exit_coordinate_setting(ctx);
        }
    }
}

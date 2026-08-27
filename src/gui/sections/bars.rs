use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    settings::{self, Settings},
    types::{AppMode, InteractionMode},
};

impl RustyAutoClickerApp {
    pub fn show_topbar(&mut self, ui: &mut egui::Ui) {
        // `Panel::top`'s default frame has a FIXED (unscaled) 2px top/
        // bottom inner margin — fine back when the whole UI was a fixed
        // size, but now that everything else (font, buttons, spacing)
        // scales with `self.ui_scale`, that unscaled 2px stopped keeping
        // pace: at default/smaller window sizes the row reads as
        // artificially cramped/raised against the window's title bar
        // compared to the pre-scaling layout, which always had comfortable
        // breathing room here.
        //
        // ONLY the top inset is scaled — the bottom is deliberately left at
        // 0. The row's content (the "|" separators especially) should sit
        // flush against the thin border line `Panel::top`'s frame draws
        // along its own bottom edge, not float above it with a gap on both
        // sides. Padding only above (pushing the whole row down, away from
        // the title bar) combined with zero padding below is what makes
        // the separators' bottom tips actually touch that line instead of
        // stopping short of it, matching the pre-scaling look.
        //
        // Simply shrinking this margin (tried at 3.0, then 0.0) was WRONG:
        // since the bottom margin is fixed at 0, the border line `Panel::
        // top`'s frame draws along its own bottom edge sits at exactly
        // `vmargin + row_h` — shrinking `vmargin` alone drags that border
        // line up right along with the row, instead of leaving it in
        // place. The border (and the "|" separators, which stretch to
        // fill the full row height and must keep touching it) has to stay
        // exactly where it was; only the row's TEXT/ICONS should move up.
        //
        // Fix: move a fixed budget of pixels (`RAISE_PX`) OUT of the top
        // margin and INTO `row_h` (see below) in equal amounts. Their SUM
        // — and therefore `vmargin + row_h`, i.e. the border's position —
        // stays exactly what it was at the original `vmargin = 4.0`
        // baseline. The separator still stretches to fill the (now taller)
        // `row_h`, so its bottom tip still lands exactly on the
        // unmoved border. But the row's content is vertically CENTERED
        // within `row_h`, and that center point shifts up by half of
        // whatever was moved out of the margin — moving the budget from
        // margin into row_h pulls the text up without moving the border
        // or the separators' bottom tips at all. `RAISE_PX = 4.0` is the
        // maximum this technique supports without going past a 0.0 margin
        // (going further would need a different approach) — it yields a
        // ~2px upward shift of the text at scale 1.0.
        const BASE_MARGIN_PX: f32 = 4.0;
        const RAISE_PX: f32 = 4.0;
        let vmargin =
            ((BASE_MARGIN_PX - RAISE_PX) * self.ui_scale).round().clamp(0.0, 127.0) as i8;
        let mut top_frame = egui::Frame::side_top_panel(ui.style());
        top_frame.inner_margin.top = vmargin;
        top_frame.inner_margin.bottom = 0;
        egui::Panel::top("top_panel").frame(top_frame).show_inside(ui, |ui| {
            // Tried switching this to `horizontal_wrapped` so "App Mode:
            // Bot Humanlike" could drop to its own line instead of
            // clipping at the enforced minimum window size — reverted:
            // `horizontal_wrapped`'s spacing is looser than `MenuBar`'s, so
            // it wrapped even at the DEFAULT window width, adding an
            // unwanted second line there and re-triggering the vertical
            // scrollbar/clipping bug for every row below it. Left as
            // `MenuBar` (non-wrapping) for now; the topbar can still clip a
            // little at the exact enforced minimum size, a smaller and
            // separate issue from the content-row clipping this session
            // fixed.
            egui::MenuBar::new().ui(ui, |ui| {
                // Reserve the row's FINAL height up front, before adding
                // any widget. MenuBar itself already reserves
                // `interact_size.y` internally before it hands control to
                // this closure — but that's only tall enough for plain
                // buttons/labels, not the bigger Save/Reset icon glyphs
                // added below (see that section's note). Widgets already
                // placed in a row are NOT retroactively re-centered if a
                // later widget turns out to need more height — only the
                // container's reported size grows — so if the row's true
                // height isn't known until partway through (previously:
                // by the Save/Reset buttons), everything added before that
                // point centers against the wrong, smaller height while
                // everything after centers correctly. That mismatch is
                // exactly what made Start/Settings/HotKeys look "raised"
                // relative to App Mode. Reserving the real final height
                // here, before the first widget, is what actually fixes
                // it: `+6.0 * self.ui_scale` is the measured amount the
                // "↺" reset glyph needs beyond the plain interact_size.y
                // floor at its original (bigger, preferred) 20pt size.
                //
                // `+ RAISE_PX * self.ui_scale` on top of that is the other
                // half of the margin/row_h trade described above `vmargin`
                // — it makes this row taller by exactly what was removed
                // from the top margin, so the border position (margin +
                // row_h) is unchanged, while the row's vertically-centered
                // content shifts up within the extra room.
                let row_h =
                    ui.spacing().interact_size.y + (6.0 + RAISE_PX) * self.ui_scale;
                ui.set_min_height(row_h);

                // ── Start / Stop button ──────────────────────────────────
                if self.is_autoclicking() {
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.mode = InteractionMode::Idle;
                    }
                } else {
                    if self.hotkey_window_open || self.is_holding() {
                        ui.disable();
                    }
                    if ui.button(self.autoclick_button_label()).clicked() {
                        self.start_autoclick(0u64);
                    }
                }

                ui.separator();
                ui.label("Settings:");

                // ── Hotkeys window ───────────────────────────────────────
                if ui
                    .add_enabled(
                        !self.is_autoclicking() && !self.is_holding(),
                        egui::Button::new("⌨ HotKeys"),
                    )
                    .clicked()
                {
                    self.hotkey_window_open = true;
                }

                // ── Save / Reset icon buttons ────────────────────────────
                // Before the responsive-scaling work, these buttons used a
                // FIXED `min_size(vec2(28.0, 24.0))` — a height of exactly
                // 24px, independent of the row's own reserved height. That
                // 24px (scaled) is what actually reproduces "how it looked
                // before" — using `row_h` here instead (as an earlier
                // version of this fix did) subtly changed the button's
                // proportions and vertical centering versus the original.
                // `row_h` (reserved via `ui.set_min_height` above) is what
                // keeps this row's OTHER widgets — Start/Settings/HotKeys —
                // aligned with App Mode; it doesn't need to also be this
                // button's own height for that alignment to hold, since
                // `MenuBar`'s row centers each widget within the row's
                // overall (already-reserved) height regardless of that
                // widget's own min_size.
                let icon_size = egui::vec2(28.0 * self.ui_scale, 24.0 * self.ui_scale);
                if ui
                    .add_enabled(
                        !self.is_busy(),
                        egui::Button::new(egui::RichText::new("💾").size(16.0 * self.ui_scale))
                            .min_size(icon_size),
                    )
                    .on_hover_text("Save current settings to disk")
                    .clicked()
                {
                    settings::save_settings(&Settings::from_app(self));
                }

                let ctx = ui.ctx().clone();
                if ui
                    .add_enabled(
                        !self.is_busy(),
                        egui::Button::new(egui::RichText::new("↺").size(20.0 * self.ui_scale))
                            .min_size(icon_size),
                    )
                    .on_hover_text("Reset all settings to defaults")
                    .clicked()
                {
                    self.reset_to_defaults(&ctx);
                }

                ui.separator();
                ui.label("App Mode:");

                self.disable_if_busy(ui);
                ui.selectable_value(&mut self.app_mode, AppMode::Bot, "🖥 Bot")
                    .on_hover_text("Autoclick as fast as possible");
                ui.selectable_value(&mut self.app_mode, AppMode::Humanlike, "😆 Human")
                    .on_hover_text("Autoclick emulating human clicking");
            });
        });
    }

    pub fn show_bottombar(&mut self, ui: &mut egui::Ui) {
        // Zero the panel's own (fixed, unscaled) top inner margin — see the
        // matching note on `CentralPanel`'s frame in `gui/mod.rs::ui()`.
        // Without this, that fixed margin stacked with `CentralPanel`'s own
        // bottom margin made the gap below the Start/Hold buttons visibly
        // bigger than the deliberate `gap` added above them, even though
        // the button code adds equal space on both sides.
        let mut bottom_frame = egui::Frame::side_top_panel(ui.style());
        bottom_frame.inner_margin.top = 0;
        egui::Panel::bottom("bottom_panel").frame(bottom_frame).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.weak("⚠ Hold HotKeys*")
                    .on_hover_text(
                        "While the window is minimized, a quick tap of a hotkey may not \
                         always register — holding it down briefly is more reliable in \
                         that case. This doesn't apply while the window is just behind \
                         other windows, only while actually minimized."
                    );

                // right_to_left reverses visual order relative to add
                // order — the item added FIRST ends up RIGHTMOST. To read
                // left-to-right as "rusty-autoclicker powered by egui and
                // eframe [separator] [debug warning]", they have to be
                // added in exactly the reverse of that: debug warning,
                // separator, eframe, " and ", egui, "powered by ",
                // rusty-autoclicker. (An earlier version of this had that
                // reversal backwards, which is why it used to render as
                // "eframe and egui powered by rusty-autoclicker" instead.)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    egui::warn_if_debug_build(ui);
                    ui.separator();
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/eframe",
                    );
                    ui.label(" and ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label("powered by ");
                    ui.hyperlink_to(
                        "rusty-autoclicker",
                        "https://github.com/MrTanoshii/rusty-autoclicker",
                    );
                });
            });
        });
    }
}

# Rusty AutoClicker v2.5.2 — Changelog

This release adds full settings persistence, a completely reworked coordinate
picker (crash-safe under minimize/rapid input, matches other apps as an
overlay), a Save/Reset workflow, and several correctness/performance fixes.
Version bumped from 2.5.1 → 2.5.2.

## New: Settings Persistence

- Added `src/settings.rs` with a `Settings` struct that captures every
  user-configurable value: hotkeys, click coordinates, timing fields
  (hr/min/sec/ms), tween speed range, click amount, click button/type/
  position, app mode, and window geometry (size + position).
- Settings are stored as JSON at `%APPDATA%/rusty-autoclicker/settings.json`
  (via the `dirs` crate for cross-platform config-dir resolution).
- Hotkeys round-trip through `device_query::Keycode`'s own `Display`/
  `FromStr`. `ClickButton` gets a small custom string mapping
  (`to_setting_string` / `from_setting_string`) since `rdev::Button`/`Key`
  don't implement `serde` traits directly.
- New dependencies: `serde`, `serde_json`, `dirs`.

## New: Manual Save / Reset Workflow

- Removed continuous "save every frame if changed" auto-save entirely.
- Added a **Save** button (💾) in the top bar — writes all current settings
  to disk immediately, on demand.
- Added a **Reset** button (↺) in the top bar — restores every setting to
  its documented default, resizes the window back to its default size,
  re-centers it on the actual screen resolution, and saves immediately.
- **Window size and position are the one exception that still auto-saves**,
  but only once, in `Drop` (on app close) — not continuously. This keeps a
  manually-resized/moved window remembered across restarts without
  reintroducing per-frame disk writes.
- Fixed startup flicker: the saved window position/size (or, on first
  launch, a computed screen-center position) is now passed directly into
  `ViewportBuilder` in `main.rs` *before* the window is created, so the
  window never flashes at a default position and then jumps.
- Fixed true screen-centering: switched from manually computing center via
  `viewport().monitor_size` (documented as commonly `None`/unreliable) to
  egui's own `ViewportCommand::center_on_screen`. This now runs both on
  **Reset** and on **first launch** (previously first launch fell back to a
  hardcoded `(100, 100)` corner).

## New: "Set Coords" Hotkey

- Added a dedicated hotkey (default **F10**) to open the coordinate picker,
  as a shortcut for the existing manual "Set Coords" button — both now go
  through the same code path.
- Added to the Hotkeys window UI, positioned below Start/Stop and above
  Confirm Coords, with its own "PRESS ANY KEY" capture flow matching the
  other three hotkeys.
- Default hotkeys (all rebindable): Start/Stop = `F6`, Set Coords = `F10`,
  Confirm Coords = `Escape` / Left Click, Click & Hold = `F7`.

## Reworked: Coordinate Picker Architecture

The single biggest change this release. The old picker worked by resizing
the **main window itself** down to a small borderless overlay (400×30),
then restoring it after confirmation. This caused several serious bugs when
combined with the new F10 hotkey and minimize support:

- Main window would sometimes shrink permanently to the picker's tiny size
  under rapid F10 + confirm spam.
- Using F10 while the app was minimized would pop the main window back up,
  with a visible OS minimize/restore animation.
- Under rapid input, this could crash.

**Fix:** the coordinate picker is now rendered in its own **independent
egui viewport** (`ctx.show_viewport_deferred`, driven from `logic()` so it
keeps working even while the main window is minimized), completely separate
from the main window:

- Fixed size, currently **400×65**, defined as `COORD_PICKER_SIZE` in
  `src/gui/windows.rs` — never resized by any code path, and
  `.with_resizable(false)` prevents the user from dragging it larger/
  smaller either.
- `.with_always_on_top()` so it overlays other applications on screen, like
  the original picker behavior.
- Visual style rebuilt to match the original overlay exactly: `Panel::top`
  + `MenuBar` + `horizontal_wrapped` in a `right_to_left` layout, with the
  original widget order (Y-edit, "Y", X-edit, "X", separator, "Set with…"
  label), using the app's normal dark theme rather than a manually painted
  black box.
- Real-time X/Y tracking fixed: the picker's own viewport closure now calls
  `request_repaint()` on itself every frame (a self-sustaining repaint
  loop), rather than relying solely on the parent nudging it — the latter
  proved unreliable and caused the coordinates to appear frozen.
- Data is shared between the main app and the picker's viewport via
  `Arc<Mutex<CoordPickerShared>>` (mouse position + confirm hotkey), since
  deferred-viewport callbacks must be `Send + Sync + 'static` and can't
  borrow `&mut self`.
- New behavior on entering coordinate-setting mode: if the main window is
  currently **visible**, it's automatically minimized so only the picker is
  on screen while picking, then restored to its exact prior state once
  coordinates are confirmed. If the main window is **already minimized**
  when F10 fires, it's left untouched throughout — no pop-up, no animation,
  no resize, no matter how rapidly F10 or confirm are pressed.

## Performance

- `device_query::DeviceState` is now created **once** (in
  `Default::default()`) and reused every frame, instead of being
  reconstructed on every single `logic()` call.
- `track_window_geometry` (which persists the main window's size/position)
  is now explicitly guarded to only ever read the **root** viewport's
  geometry (`ctx.viewport_id() == egui::ViewportId::ROOT`), preventing a
  rare timing edge case where it could transiently read the picker
  viewport's tiny geometry instead and persist that as the main window's
  size.
- `ctx.request_repaint()` is called unconditionally every frame so hotkeys
  are never delayed by a sleep — egui already throttles this to the
  display's refresh rate, so it doesn't increase CPU usage beyond normal.

## Defaults Changed

- Click interval default: `100ms` → `200ms`.
- Main window default size: `550×341` → `580×340`.
- Coordinate picker fixed size: `400×65`.
- Click coordinate defaults remain `0, 0` (unrelated to window position).

## Cleanup

- Removed an unused/dead field (`key_pressed_set_coord`) that was written
  but never read anywhere.
- Removed unused imports left over from earlier iterations of the
  coordinate-picker rework.

---

### Files touched
`Cargo.toml`, `src/main.rs`, `src/defines.rs`, `src/types.rs`, `src/app.rs`,
`src/settings.rs` (new), `src/gui/mod.rs`, `src/gui/windows.rs`,
`src/gui/sections/bars.rs`, `src/gui/sections/click_config.rs`

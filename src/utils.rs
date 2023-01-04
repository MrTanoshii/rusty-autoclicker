use std::{env, thread, time::Duration};

use eframe::emath::Numeric;
use rand::{prelude::ThreadRng, Rng};
use rdev::{simulate, EventType, SimulateError};
use sanitizer::prelude::StringSanitizer;

use crate::{
    defines::{
        APP_ICON, DURATION_CLICK_MAX, DURATION_CLICK_MIN, DURATION_DOUBLE_CLICK_MAX,
        DURATION_DOUBLE_CLICK_MIN, MOUSE_STEP_NEG_X, MOUSE_STEP_NEG_Y, MOUSE_STEP_POS_X,
        MOUSE_STEP_POS_Y,
    },
    types::{AppMode, ClickInfo, ClickPosition, ClickType},
};

/// Load icon from memory and return it
pub fn load_icon() -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(APP_ICON)
            .expect("Failed to open icon path")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
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

/// Send the simulated event (`rdev` crate)
///
/// # Arguments
///
/// * `event_type` - The event type to simulate
fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }

    // Let the OS catchup (at least MacOS)
    if env::consts::OS == "macos" {
        thread::sleep(Duration::from_millis(20u64));
    }
}

/// Move the mouse to the specified coordinates
/// Work if app is in "Humandlike" mode only
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_position` - The click position type
/// * `click_coord` - The click coordinates
/// * `start_coords` - The starting mouse coordinates
/// * `movement_delay_in_ms` - The delay between mouse movements in milliseconds
fn move_to(
    app_mode: AppMode,
    click_position: ClickPosition,
    click_coord: (f64, f64),
    start_coords: (f64, f64),
    movement_delay_in_ms: u64,
) {
    if app_mode == AppMode::Humanlike && click_position == ClickPosition::Coord {
        // Move mouse slowly to saved coordinates if requested
        let mut current_x = start_coords.0;
        let mut current_y = start_coords.1;
        loop {
            // horizontal movement: determine whether we need to move left, right or not at all
            let delta_x: f64 = if current_x < click_coord.0 {
                MOUSE_STEP_POS_X.min(click_coord.0 - current_x)
            } else if current_x > click_coord.0 {
                MOUSE_STEP_NEG_X.max(click_coord.0 - current_x)
            } else {
                0.0
            };

            // vertical movement: determine whether we need to move up, down or not at all
            let delta_y: f64 = if current_y < click_coord.1 {
                MOUSE_STEP_POS_Y.min(click_coord.1 - current_y)
            } else if current_y > click_coord.1 {
                MOUSE_STEP_NEG_Y.max(click_coord.1 - current_y)
            } else {
                0.0
            };

            current_x += delta_x;
            current_y += delta_y;

            #[cfg(debug_assertions)]
            println!(
                "Moving by {:?} / {:?}, new pos: {:?} / {:?}",
                delta_x, delta_y, current_x, current_y
            );
            send(&EventType::MouseMove {
                x: current_x,
                y: current_y,
            });

            thread::sleep(Duration::from_millis(movement_delay_in_ms));
            if current_x == click_coord.0 && current_y == click_coord.1 {
                return;
            }
        }
    }
}

/// Autoclick the mouse
///
/// # Arguments
///
/// * `app_mode` - The app mode
/// * `click_info` - The click information
/// * `mouse_coord` - The mouse coordinates
/// * `movement_delay_in_ms` - The delay between mouse movements in milliseconds
/// * `rng_thread` - The random number generator thread
pub fn autoclick(
    app_mode: AppMode,
    click_info: ClickInfo,
    mouse_coord: (i32, i32),
    movement_delay_in_ms: u64,
    mut rng_thread: ThreadRng,
) {
    // Set the amount of runs/clicks required
    let run_amount: u8 = if click_info.click_type == ClickType::Single {
        1
    } else if click_info.click_type == ClickType::Double {
        2
    } else {
        0
    };

    // Autoclick as fast as possible
    if app_mode == AppMode::Bot {
        for _n in 1..=run_amount {
            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                send(&EventType::MouseMove {
                    x: click_info.click_coord.0,
                    y: click_info.click_coord.1,
                })
            }

            send(&EventType::ButtonPress(click_info.click_btn));
            send(&EventType::ButtonRelease(click_info.click_btn));
        }
    // Autoclick to emulate a humanlike clicks
    } else if app_mode == AppMode::Humanlike {
        let click_x = click_info.click_coord.0;
        let click_y = click_info.click_coord.1;
        // move to target
        #[cfg(debug_assertions)]
        println!(
            "Moving from {:?}/{:?} towards: {:?}/{:?}",
            mouse_coord.0.to_f64(),
            mouse_coord.1.to_f64(),
            click_x,
            click_y
        );

        // perform clicks
        for n in 1..=run_amount {
            // Sleep between clicks
            if n % 2 == 0 {
                thread::sleep(Duration::from_millis(
                    rng_thread.gen_range(DURATION_DOUBLE_CLICK_MIN..DURATION_DOUBLE_CLICK_MAX),
                ));
            }

            // Move mouse to saved coordinates if requested
            if click_info.click_position == ClickPosition::Coord {
                // only move if start pos and click pos are not identical
                if click_x != mouse_coord.0.to_f64() || click_y != mouse_coord.1.to_f64() {
                    move_to(
                        app_mode,
                        click_info.click_position,
                        (click_x, click_y),
                        (mouse_coord.0.to_f64(), mouse_coord.1.to_f64()),
                        movement_delay_in_ms,
                    );
                }
            }

            send(&EventType::ButtonPress(click_info.click_btn));
            thread::sleep(Duration::from_millis(
                rng_thread.gen_range(DURATION_CLICK_MIN..DURATION_CLICK_MAX),
            ));
            send(&EventType::ButtonRelease(click_info.click_btn));
        }
    }
}

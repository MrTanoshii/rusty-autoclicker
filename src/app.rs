use eframe::{
    egui,
    epaint::{FontFamily, FontId},
    epi,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    value: f32,
    hr_str: String,
    min_str: String,
    sec_str: String,
    ms_str: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            hr_str: "0".to_owned(),
            min_str: "0".to_owned(),
            sec_str: "0".to_owned(),
            ms_str: "0".to_owned(),
        }
    }
}

// Sanitation to only allow numbers required
fn sanitize_time(string: &mut String) -> &mut String {
    while string.len() > 1 && string.chars().nth(0).unwrap() == '0' {
        string.remove(0);
        print!("This ran")
    }
    if string.len() >= 5 {
        string.truncate(5usize)
    };
    return string;
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut style: egui::Style = (*ctx.style()).clone();
        let font = FontId {
            size: 20.0,
            family: FontFamily::Monospace,
        };
        style.override_font_id = Some(font);
        ctx.set_style(style);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            label,
            value,
            hr_str,
            min_str,
            sec_str,
            ms_str,
        } = self;

        sanitize_time(hr_str);
        sanitize_time(min_str);
        sanitize_time(sec_str);
        sanitize_time(ms_str);

        let mut hr: i32 = 0;
        if hr_str.is_empty() != true {
            hr = hr_str.parse().unwrap();
        }
        let mut min: i32 = 0;
        if hr_str.is_empty() != true {
            min = min_str.parse().unwrap();
        }
        let mut sec: i32 = 0;
        if hr_str.is_empty() != true {
            sec = sec_str.parse().unwrap();
        }
        let mut ms: i32 = 0;
        if hr_str.is_empty() != true {
            ms = ms_str.parse().unwrap();
        }
        println!("{} hr {} min {} sec {} ms", &hr, min, sec, ms);

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(hr_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("hr");
                ui.add(
                    egui::TextEdit::singleline(min_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("min");
                ui.add(
                    egui::TextEdit::singleline(sec_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("sec");
                ui.add(
                    egui::TextEdit::singleline(ms_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("ms");
            });

            ui.heading(hr_str);

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

//! Beni App Picker – egui / eframe UI (feature-gated).

use eframe::egui;

use crate::app_picker::app::AppPickerState;

pub struct BeniAppPickerApp {
    state: AppPickerState,
}

impl BeniAppPickerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        BeniAppPickerApp {
            state: AppPickerState::new(),
        }
    }
}

impl eframe::App for BeniAppPickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Beni App Picker");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self.state.refresh();
                }
                if ui.button("Inspect").clicked() {
                    self.state.do_inspect();
                }
                if ui.button("Extract").clicked() {
                    self.state.do_extract();
                }
                if ui.button("Launch").clicked() {
                    self.state.do_launch();
                }
            });

            ui.separator();
            ui.label(format!("{} application(s)", self.state.apps.len()));

            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                for (i, app) in self.state.apps.iter().enumerate() {
                    let selected = self.state.selected == Some(i);
                    if ui.selectable_label(selected, format!("{} ({})", app.name, app.format_label())).clicked() {
                        self.state.selected = Some(i);
                    }
                }
            });

            ui.separator();
            if let Some(app) = self.state.selected_app() {
                ui.label(format!("Selected: {}", app.name));
                ui.label(format!("Path: {}", app.path.display()));
                ui.label(format!("Size: {}", app.size_label()));
                ui.label(format!("Status: {}", app.status_label()));
            } else {
                ui.label("Select an application.");
            }

            ui.separator();
            ui.label("Log");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.state.status_log)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
            });
        });
    }
}

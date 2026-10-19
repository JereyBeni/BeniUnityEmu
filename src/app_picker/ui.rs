//! Beni App Picker – egui / eframe UI.
//! Feature-gated: only compiled with `--features gui`.

use eframe::egui::{self, Color32, RichText};

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
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Beni App Picker");
                ui.separator();
                if ui.button("Refresh").clicked() {
                    self.state.refresh();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} apps", self.state.apps.len())).weak(),
                    );
                });
            });
        });

        egui::SidePanel::left("app_list")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Applications");
                ui.separator();
                if self.state.apps.is_empty() {
                    ui.colored_label(
                        Color32::from_rgb(180, 140, 60),
                        "No APK/IPA found.\nPut files in BeniApps/",
                    );
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, app) in self.state.apps.iter().enumerate() {
                            let selected = self.state.selected == Some(i);
                            let label = format!(
                                "{}  [{}]  {}",
                                app.name,
                                app.format_label(),
                                app.size_label()
                            );
                            if ui.selectable_label(selected, label).clicked() {
                                self.state.selected = Some(i);
                            }
                        }
                    });
                }
            });

        egui::TopBottomPanel::bottom("actions")
            .min_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let has_sel = self.state.selected.is_some();
                    ui.add_enabled_ui(has_sel, |ui| {
                        if ui.button("Launch").clicked() {
                            self.state.do_launch();
                        }
                        if ui.button("Inspect").clicked() {
                            self.state.do_inspect();
                        }
                        if ui.button("Extract").clicked() {
                            self.state.do_extract();
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Log");
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(RichText::new(&self.state.status_log).monospace());
                });
        });
    }
}

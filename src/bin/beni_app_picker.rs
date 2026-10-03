//! Beni App Picker – graphical launcher for BeniUnityEmu.
//! Binary name: BeniAppPicker.exe

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beni_unity_emu::app_picker::ui::BeniAppPickerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Beni App Picker"),
        ..Default::default()
    };

    eframe::run_native(
        "Beni App Picker",
        options,
        Box::new(|cc| Box::new(BeniAppPickerApp::new(cc))),
    )
}

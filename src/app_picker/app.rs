//! Beni App Picker application state.

use crate::app_picker::scanner;
use crate::{extract_app, inspect_app, run_app, AppInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerAction {
    None,
    Launch,
    Inspect,
    Extract,
    Refresh,
}

pub struct AppPickerState {
    pub apps: Vec<AppInfo>,
    pub selected: Option<usize>,
    pub status_log: String,
    pub last_action: PickerAction,
    pub busy: bool,
    pub show_settings: bool,
}

impl AppPickerState {
    pub fn new() -> Self {
        crate::ensure_directories();
        let mut state = AppPickerState {
            apps: Vec::new(),
            selected: None,
            status_log: String::from("Welcome to Beni App Picker\nScanning BeniApps/…\n"),
            last_action: PickerAction::None,
            busy: false,
            show_settings: false,
        };
        state.refresh();
        state
    }

    pub fn refresh(&mut self) {
        self.apps = scanner::scan_beni_apps();
        self.selected = None;
        self.status_log.push_str(&format!(
            "[Refresh] Found {} application(s)\n",
            self.apps.len()
        ));
        if self.apps.is_empty() {
            self.status_log.push_str(
                "Place .apk / .ipa files in the BeniApps/ folder next to this executable.\n",
            );
        }
    }

    pub fn selected_app(&self) -> Option<&AppInfo> {
        self.selected.and_then(|i| self.apps.get(i))
    }

    pub fn do_launch(&mut self) {
        let Some(app) = self.selected_app().cloned() else {
            self.status_log.push_str("[Launch] No application selected\n");
            return;
        };
        self.status_log
            .push_str(&format!("[Launch] Starting {}\u2026\n", app.name));
        match run_app(&app) {
            Ok(msg) => self.status_log.push_str(&format!("[Launch] {}\n", msg)),
            Err(e) => self.status_log.push_str(&format!("[Launch] Error: {}\n", e)),
        }
    }

    pub fn do_inspect(&mut self) {
        let Some(app) = self.selected_app().cloned() else {
            self.status_log.push_str("[Inspect] No application selected\n");
            return;
        };
        self.status_log
            .push_str(&format!("[Inspect] {}\n", app.name));
        let report = inspect_app(&app);
        self.status_log.push_str(&report);
        self.status_log.push('\n');
    }

    pub fn do_extract(&mut self) {
        let Some(app) = self.selected_app().cloned() else {
            self.status_log.push_str("[Extract] No application selected\n");
            return;
        };
        self.status_log
            .push_str(&format!("[Extract] {}\u2026\n", app.name));
        match extract_app(&app) {
            Ok(msg) => self.status_log.push_str(&format!("{}\n", msg)),
            Err(e) => self.status_log.push_str(&format!("[Extract] Error: {}\n", e)),
        }
    }
}

impl Default for AppPickerState {
    fn default() -> Self {
        Self::new()
    }
}

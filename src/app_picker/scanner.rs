//! Scan BeniApps/ for APK and IPA packages.

use crate::{enrich_app_info, scan_apps, AppInfo};

pub fn scan_beni_apps() -> Vec<AppInfo> {
    let mut apps = scan_apps();
    for app in &mut apps {
        enrich_app_info(app);
    }
    apps
}

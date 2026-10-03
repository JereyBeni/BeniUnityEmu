//! High-level extraction helpers for APK contents.

use std::path::Path;

use super::loader::ApkLoader;

pub fn extract_required(loader: &ApkLoader, dest_root: &Path) -> Result<usize, String> {
    let mut total = 0;

    let lib_dest = dest_root.join("lib");
    match loader.extract_prefix("lib/", &lib_dest) {
        Ok(n) => {
            println!("[CACHE] Extracted {} native library files", n);
            total += n;
        }
        Err(e) => println!("[CACHE] lib/ extraction note: {}", e),
    }

    let assets_dest = dest_root.join("assets");
    match loader.extract_prefix("assets/", &assets_dest) {
        Ok(n) => {
            println!("[CACHE] Extracted {} asset files", n);
            total += n;
        }
        Err(e) => println!("[CACHE] assets/ extraction note: {}", e),
    }

    let meta_dest = dest_root.join("meta");
    std::fs::create_dir_all(&meta_dest).ok();

    if loader.has_file("AndroidManifest.xml") {
        let dest = meta_dest.join("AndroidManifest.xml");
        if loader.extract_file("AndroidManifest.xml", &dest).is_ok() {
            total += 1;
            println!("[CACHE] Extracted AndroidManifest.xml");
        }
    }

    for name in &["classes.dex", "classes2.dex", "classes3.dex"] {
        if loader.has_file(name) {
            let dest = meta_dest.join(name);
            if loader.extract_file(name, &dest).is_ok() {
                total += 1;
            }
        }
    }

    Ok(total)
}

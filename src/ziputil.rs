//! Shared ZIP helpers for APK and IPA inspection/extraction.
//! Rejects path traversal and handles malformed / encrypted entries gracefully.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use zip::result::ZipError;
use zip::ZipArchive;

pub fn open_archive(path: &Path) -> Result<ZipArchive<File>, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open archive {:?}: {}", path, e))?;
    ZipArchive::new(file).map_err(|e| match e {
        ZipError::InvalidArchive(msg) => format!("Malformed archive {:?}: {}", path, msg),
        ZipError::UnsupportedArchive(msg) => format!("Unsupported archive {:?}: {}", path, msg),
        other => format!("Failed to read archive {:?}: {}", path, other),
    })
}

pub fn sanitize_entry_name(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return None;
    }
    let mut clean = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(s) => clean.push(s),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if clean.as_os_str().is_empty() {
        None
    } else {
        Some(clean)
    }
}

pub fn list_entries<R: Read + io::Seek>(archive: &mut ZipArchive<R>) -> Vec<String> {
    let mut names = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if sanitize_entry_name(&name).is_some() {
                names.push(name);
            }
        }
    }
    names
}

pub fn has_entry<R: Read + io::Seek>(archive: &mut ZipArchive<R>, name: &str) -> bool {
    archive.by_name(name).is_ok()
}

pub fn extract_file_to<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
    dest: &Path,
) -> Result<(), String> {
    let _safe = sanitize_entry_name(entry_name)
        .ok_or_else(|| format!("Unsafe or empty entry name: {}", entry_name))?;

    let mut entry = archive.by_name(entry_name).map_err(|e| match e {
        ZipError::FileNotFound => format!("Entry not found: {}", entry_name),
        ZipError::UnsupportedArchive(msg) => {
            format!("Unsupported/encrypted entry {}: {}", entry_name, msg)
        }
        other => format!("Cannot read entry {}: {}", entry_name, other),
    })?;

    if entry.is_dir() {
        fs::create_dir_all(dest).map_err(|e| format!("Cannot create dir {:?}: {}", dest, e))?;
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create parent for {:?}: {}", dest, e))?;
    }

    let mut out = File::create(dest).map_err(|e| format!("Cannot create {:?}: {}", dest, e))?;
    io::copy(&mut entry, &mut out).map_err(|e| format!("Write failed for {:?}: {}", dest, e))?;
    Ok(())
}

pub fn extract_prefix_to<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    prefix: &str,
    dest_root: &Path,
) -> Result<usize, String> {
    fs::create_dir_all(dest_root)
        .map_err(|e| format!("Cannot create dest root {:?}: {}", dest_root, e))?;

    let mut count = 0usize;
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();

    for name in names {
        if !name.starts_with(prefix) {
            continue;
        }
        let relative = &name[prefix.len()..];
        if relative.is_empty() {
            continue;
        }
        let safe = match sanitize_entry_name(relative) {
            Some(p) => p,
            None => {
                eprintln!("[ZIP] Skipping unsafe entry under prefix: {}", name);
                continue;
            }
        };
        let dest = dest_root.join(&safe);

        match archive.by_name(&name) {
            Ok(mut entry) => {
                if entry.is_dir() {
                    let _ = fs::create_dir_all(&dest);
                    continue;
                }
                if let Some(parent) = dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                match File::create(&dest) {
                    Ok(mut out) => {
                        if io::copy(&mut entry, &mut out).is_ok() {
                            count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("[ZIP] Cannot create {:?}: {}", dest, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[ZIP] Cannot extract {}: {}", name, e);
            }
        }
    }
    Ok(count)
}

pub fn read_entry_bytes<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Result<Vec<u8>, String> {
    let _safe = sanitize_entry_name(entry_name)
        .ok_or_else(|| format!("Unsafe entry name: {}", entry_name))?;
    let mut entry = archive.by_name(entry_name).map_err(|e| {
        format!("Cannot open entry {}: {}", entry_name, e)
    })?;
    let mut buf = Vec::new();
    entry
        .read_to_end(&mut buf)
        .map_err(|e| format!("Read failed for {}: {}", entry_name, e))?;
    Ok(buf)
}

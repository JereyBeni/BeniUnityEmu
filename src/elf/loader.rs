//! Minimal ELF inspection (no execution).

use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ElfInfo {
    pub is_elf: bool,
    pub is_64: bool,
    pub little_endian: bool,
    pub machine: u16,
    pub notes: Vec<String>,
}

pub fn inspect_elf(path: &Path) -> Result<ElfInfo, String> {
    let data = fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    if data.len() < 20 || &data[0..4] != b"\x7fELF" {
        return Ok(ElfInfo {
            is_elf: false,
            is_64: false,
            little_endian: true,
            machine: 0,
            notes: vec!["Not an ELF file".into()],
        });
    }
    let is_64 = data[4] == 2;
    let little_endian = data[5] == 1;
    let machine = if little_endian {
        u16::from_le_bytes([data[18], data[19]])
    } else {
        u16::from_be_bytes([data[18], data[19]])
    };
    Ok(ElfInfo {
        is_elf: true,
        is_64,
        little_endian,
        machine,
        notes: vec![format!("ELF machine=0x{:x}", machine)],
    })
}

pub struct ElfLoader;

impl ElfLoader {
    pub fn open(path: &Path) -> Result<ElfInfo, String> {
        inspect_elf(path)
    }
}

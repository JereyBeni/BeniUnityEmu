//! ELF helpers (inspection only — no native execution).

pub mod loader;

pub use loader::{inspect_elf, ElfInfo, ElfLoader};

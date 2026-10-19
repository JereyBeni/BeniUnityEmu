//! Minimal Mach-O header inspection (no execution).

use std::convert::TryInto;

const MH_MAGIC: u32 = 0xfeed_face;
const MH_CIGAM: u32 = 0xcefa_edfe;
const MH_MAGIC_64: u32 = 0xfeed_facf;
const MH_CIGAM_64: u32 = 0xcffa_edfe;
const FAT_MAGIC: u32 = 0xcafe_babe;
const FAT_CIGAM: u32 = 0xbeba_feca;
const FAT_MAGIC_64: u32 = 0xcafe_babf;
const FAT_CIGAM_64: u32 = 0xbfba_feca;

const CPU_TYPE_ARM: u32 = 12;
const CPU_TYPE_ARM64: u32 = 0x0100_000c;
const CPU_TYPE_X86: u32 = 7;
const CPU_TYPE_X86_64: u32 = 0x0100_0007;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachOArch {
    Arm,
    Arm64,
    X86,
    X86_64,
    Unknown(u32),
}

impl MachOArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            MachOArch::Arm => "armv7",
            MachOArch::Arm64 => "arm64",
            MachOArch::X86 => "x86",
            MachOArch::X86_64 => "x86_64",
            MachOArch::Unknown(_) => "unknown",
        }
    }
}

fn from_cputype(cpu: u32) -> MachOArch {
    match cpu {
        CPU_TYPE_ARM => MachOArch::Arm,
        CPU_TYPE_ARM64 => MachOArch::Arm64,
        CPU_TYPE_X86 => MachOArch::X86,
        CPU_TYPE_X86_64 => MachOArch::X86_64,
        other => MachOArch::Unknown(other),
    }
}

fn read_u32_native(data: &[u8], off: usize) -> Option<u32> {
    let bytes: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn read_u32_swapped(data: &[u8], off: usize) -> Option<u32> {
    let bytes: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes).swap_bytes())
}

pub fn inspect_macho_arches(data: &[u8]) -> Result<Vec<MachOArch>, String> {
    if data.len() < 8 {
        return Err("File too small for Mach-O header".into());
    }
    let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());

    match magic {
        MH_MAGIC | MH_MAGIC_64 => {
            let cpu = read_u32_native(data, 4).ok_or("Truncated Mach-O header")?;
            Ok(vec![from_cputype(cpu)])
        }
        MH_CIGAM | MH_CIGAM_64 => {
            let cpu = read_u32_swapped(data, 4).ok_or("Truncated Mach-O header")?;
            Ok(vec![from_cputype(cpu)])
        }
        FAT_MAGIC | FAT_MAGIC_64 => {
            let nfat = read_u32_native(data, 4).ok_or("Truncated fat header")? as usize;
            let arch_size = if magic == FAT_MAGIC_64 { 32 } else { 20 };
            let mut arches = Vec::new();
            for i in 0..nfat.min(16) {
                let base = 8 + i * arch_size;
                if let Some(cpu) = read_u32_native(data, base) {
                    arches.push(from_cputype(cpu));
                }
            }
            if arches.is_empty() {
                Err("No architecture slices found in fat binary".into())
            } else {
                Ok(arches)
            }
        }
        FAT_CIGAM | FAT_CIGAM_64 => {
            let nfat = read_u32_swapped(data, 4).ok_or("Truncated fat header")? as usize;
            let arch_size = if magic == FAT_CIGAM_64 { 32 } else { 20 };
            let mut arches = Vec::new();
            for i in 0..nfat.min(16) {
                let base = 8 + i * arch_size;
                if let Some(cpu) = read_u32_swapped(data, base) {
                    arches.push(from_cputype(cpu));
                }
            }
            if arches.is_empty() {
                Err("No architecture slices found in fat binary".into())
            } else {
                Ok(arches)
            }
        }
        _ => Err(format!("Not a Mach-O binary (magic {:#010x})", magic)),
    }
}

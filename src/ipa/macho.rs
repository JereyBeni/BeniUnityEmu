//! Minimal Mach-O header inspection (no execution).

use std::convert::TryInto;

const MH_MAGIC: u32 = 0xfeed_face;
const MH_CIGAM: u32 = 0xcefa_edfe;
const MH_MAGIC_64: u32 = 0xfeed_facf;
const MH_CIGAM_64: u32 = 0xcffa_edfe;
const FAT_MAGIC: u32 = 0xcafe_babe;
const FAT_CIGAM: u32 = 0xbeba_feca;

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

fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    let bytes: [u8; 4] = data.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

pub fn inspect_macho_arches(data: &[u8]) -> Result<Vec<MachOArch>, String> {
    if data.len() < 8 {
        return Err("File too small for Mach-O header".into());
    }
    let magic = read_u32_le(data, 0).ok_or("short read")?;
    match magic {
        MH_MAGIC | MH_MAGIC_64 => {
            let cpu = read_u32_le(data, 4).ok_or("short read")?;
            Ok(vec![from_cputype(cpu)])
        }
        MH_CIGAM | MH_CIGAM_64 => {
            let cpu = read_u32_le(data, 4).ok_or("short read")?.swap_bytes();
            Ok(vec![from_cputype(cpu)])
        }
        FAT_MAGIC | FAT_CIGAM => {
            let swapped = magic == FAT_CIGAM;
            let n = read_u32_le(data, 4).ok_or("short read")?;
            let narch = if swapped { n.swap_bytes() } else { n } as usize;
            let mut out = Vec::new();
            for i in 0..narch.min(16) {
                let off = 8 + i * 20;
                let cpu = read_u32_le(data, off).ok_or("fat short")?;
                let cpu = if swapped { cpu.swap_bytes() } else { cpu };
                out.push(from_cputype(cpu));
            }
            if out.is_empty() {
                Err("No fat architectures".into())
            } else {
                Ok(out)
            }
        }
        _ => Err(format!("Not a Mach-O magic: {:#x}", magic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_arm64_header() {
        let mut data = vec![0u8; 32];
        data[0..4].copy_from_slice(&MH_MAGIC_64.to_le_bytes());
        data[4..8].copy_from_slice(&CPU_TYPE_ARM64.to_le_bytes());
        let arches = inspect_macho_arches(&data).unwrap();
        assert_eq!(arches, vec![MachOArch::Arm64]);
    }

    #[test]
    fn reject_too_small() {
        assert!(inspect_macho_arches(&[1, 2, 3]).is_err());
    }
}

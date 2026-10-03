//! Virtual memory manager for the guest (ARM) address space.
//! Host process memory is NEVER used to execute guest code.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Code,
    Data,
    Stack,
    Heap,
    Mapped,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub kind: RegionKind,
    pub name: String,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

pub struct MemoryManager {
    pages: BTreeMap<u64, Vec<u8>>,
    regions: Vec<MemoryRegion>,
    page_size: u64,
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            pages: BTreeMap::new(),
            regions: Vec::new(),
            page_size: 4096,
        }
    }

    fn page_base(&self, addr: u64) -> u64 {
        addr & !(self.page_size - 1)
    }

    fn ensure_page(&mut self, addr: u64) -> &mut Vec<u8> {
        let base = self.page_base(addr);
        self.pages
            .entry(base)
            .or_insert_with(|| vec![0u8; self.page_size as usize])
    }

    pub fn map(
        &mut self,
        base: u64,
        size: u64,
        kind: RegionKind,
        name: &str,
        readable: bool,
        writable: bool,
        executable: bool,
    ) -> Result<(), String> {
        if size == 0 {
            return Err("[MEMORY] map size cannot be 0".to_string());
        }
        for r in &self.regions {
            let end = base.saturating_add(size);
            let r_end = r.base.saturating_add(r.size);
            if base < r_end && r.base < end {
                return Err(format!(
                    "[MEMORY] Overlap mapping '{}' with existing region '{}'",
                    name, r.name
                ));
            }
        }
        self.regions.push(MemoryRegion {
            base,
            size,
            kind,
            name: name.to_string(),
            readable,
            writable,
            executable,
        });
        let mut a = self.page_base(base);
        let end = base + size;
        while a < end {
            self.ensure_page(a);
            a += self.page_size;
        }
        println!(
            "[MEMORY] Mapped {:#010x}-{:#010x} ({}) [{}]",
            base,
            base + size,
            name,
            match kind {
                RegionKind::Code => "code",
                RegionKind::Data => "data",
                RegionKind::Stack => "stack",
                RegionKind::Heap => "heap",
                RegionKind::Mapped => "mapped",
            }
        );
        Ok(())
    }

    pub fn unmap(&mut self, base: u64) -> Result<(), String> {
        if let Some(pos) = self.regions.iter().position(|r| r.base == base) {
            let r = self.regions.remove(pos);
            println!("[MEMORY] Unmapped {} @ {:#010x}", r.name, base);
            Ok(())
        } else {
            Err(format!("[MEMORY] No region at {:#010x}", base))
        }
    }

    fn find_region(&self, addr: u64) -> Option<&MemoryRegion> {
        self.regions
            .iter()
            .find(|r| addr >= r.base && addr < r.base + r.size)
    }

    pub fn read8(&self, addr: u64) -> Result<u8, String> {
        let region = self
            .find_region(addr)
            .ok_or_else(|| format!("[MEMORY] Unmapped read8 @ {:#010x}", addr))?;
        if !region.readable {
            return Err(format!("[MEMORY] Not readable @ {:#010x}", addr));
        }
        let base = self.page_base(addr);
        let off = (addr - base) as usize;
        match self.pages.get(&base) {
            Some(page) => Ok(page[off]),
            None => Ok(0),
        }
    }

    pub fn read16(&self, addr: u64) -> Result<u16, String> {
        let b0 = self.read8(addr)? as u16;
        let b1 = self.read8(addr + 1)? as u16;
        Ok(b0 | (b1 << 8))
    }

    pub fn read32(&self, addr: u64) -> Result<u32, String> {
        let b0 = self.read8(addr)? as u32;
        let b1 = self.read8(addr + 1)? as u32;
        let b2 = self.read8(addr + 2)? as u32;
        let b3 = self.read8(addr + 3)? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    pub fn read64(&self, addr: u64) -> Result<u64, String> {
        let lo = self.read32(addr)? as u64;
        let hi = self.read32(addr + 4)? as u64;
        Ok(lo | (hi << 32))
    }

    pub fn write8(&mut self, addr: u64, val: u8) -> Result<(), String> {
        let region = self
            .find_region(addr)
            .ok_or_else(|| format!("[MEMORY] Unmapped write8 @ {:#010x}", addr))?;
        if !region.writable {
            return Err(format!("[MEMORY] Not writable @ {:#010x}", addr));
        }
        let base = self.page_base(addr);
        let off = (addr - base) as usize;
        let page = self.ensure_page(addr);
        page[off] = val;
        Ok(())
    }

    pub fn write16(&mut self, addr: u64, val: u16) -> Result<(), String> {
        self.write8(addr, (val & 0xFF) as u8)?;
        self.write8(addr + 1, ((val >> 8) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write32(&mut self, addr: u64, val: u32) -> Result<(), String> {
        self.write8(addr, (val & 0xFF) as u8)?;
        self.write8(addr + 1, ((val >> 8) & 0xFF) as u8)?;
        self.write8(addr + 2, ((val >> 16) & 0xFF) as u8)?;
        self.write8(addr + 3, ((val >> 24) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write64(&mut self, addr: u64, val: u64) -> Result<(), String> {
        self.write32(addr, (val & 0xFFFF_FFFF) as u32)?;
        self.write32(addr + 4, ((val >> 32) & 0xFFFF_FFFF) as u32)?;
        Ok(())
    }

    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), String> {
        for (i, &b) in data.iter().enumerate() {
            let a = addr + i as u64;
            let base = self.page_base(a);
            let off = (a - base) as usize;
            let page = self.ensure_page(a);
            page[off] = b;
        }
        Ok(())
    }

    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions
    }

    pub fn setup_default_arm32(&mut self) -> Result<(), String> {
        self.map(0x0001_0000, 0x0200_0000, RegionKind::Code, "code", true, false, true)?;
        self.map(0x0201_0000, 0x0100_0000, RegionKind::Data, "data", true, true, false)?;
        self.map(0x1000_0000, 0x0400_0000, RegionKind::Heap, "heap", true, true, false)?;
        self.map(0x7000_0000, 0x0010_0000, RegionKind::Stack, "stack", true, true, false)?;
        Ok(())
    }

    pub fn setup_default_arm64(&mut self) -> Result<(), String> {
        self.map(0x0000_0001_0000, 0x0400_0000, RegionKind::Code, "code", true, false, true)?;
        self.map(0x0000_0401_0000, 0x0200_0000, RegionKind::Data, "data", true, true, false)?;
        self.map(0x0000_1000_0000, 0x0800_0000, RegionKind::Heap, "heap", true, true, false)?;
        self.map(0x0000_7FFF_0000, 0x0010_0000, RegionKind::Stack, "stack", true, true, false)?;
        Ok(())
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

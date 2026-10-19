//! ARM32 (AArch32) interpreter stub – stage 2

use super::CpuBackend;
use crate::memory::MemoryManager;

#[derive(Debug)]
pub struct Arm32Cpu {
    pub regs: [u32; 16],
    pub cpsr: u32,
    pub halted: bool,
    pub steps: u64,
}

impl Arm32Cpu {
    pub fn new() -> Self {
        Arm32Cpu {
            regs: [0; 16],
            cpsr: 0x10,
            halted: false,
            steps: 0,
        }
    }
}

impl Default for Arm32Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuBackend for Arm32Cpu {
    fn reset(&mut self) {
        self.regs = [0; 16];
        self.cpsr = 0x10;
        self.halted = false;
        self.steps = 0;
        self.regs[13] = 0x7000_0000;
        println!("[ARM32] CPU reset");
    }

    fn step(&mut self, _mem: &mut MemoryManager) -> Result<(), String> {
        if self.halted {
            return Err("[ARM32] CPU is halted".to_string());
        }
        Err(format!(
            "[ARM32] Unsupported instruction at 0x{:08X} (interpreter not implemented yet)",
            self.pc()
        ))
    }

    fn run(&mut self, mem: &mut MemoryManager, max_steps: u64) -> Result<u64, String> {
        let mut executed = 0u64;
        while executed < max_steps && !self.halted {
            match self.step(mem) {
                Ok(()) => executed += 1,
                Err(e) => return Err(e),
            }
        }
        Ok(executed)
    }

    fn read_register(&self, reg: u8) -> u64 {
        if reg < 16 {
            self.regs[reg as usize] as u64
        } else {
            0
        }
    }

    fn write_register(&mut self, reg: u8, value: u64) {
        if reg < 16 {
            self.regs[reg as usize] = value as u32;
        }
    }

    fn pc(&self) -> u64 {
        self.regs[15] as u64
    }

    fn set_pc(&mut self, pc: u64) {
        self.regs[15] = pc as u32;
    }

    fn read_memory(
        &self,
        mem: &MemoryManager,
        addr: u64,
        size: usize,
    ) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(size);
        for i in 0..size {
            out.push(mem.read8(addr + i as u64)?);
        }
        Ok(out)
    }

    fn write_memory(
        &self,
        mem: &mut MemoryManager,
        addr: u64,
        data: &[u8],
    ) -> Result<(), String> {
        for (i, &b) in data.iter().enumerate() {
            mem.write8(addr + i as u64, b)?;
        }
        Ok(())
    }
}

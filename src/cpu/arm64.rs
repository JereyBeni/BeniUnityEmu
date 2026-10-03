//! ARM64 (AArch64) interpreter stub – stage 2

use super::CpuBackend;
use crate::memory::MemoryManager;

#[derive(Debug)]
pub struct Arm64Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub nzcv: u32,
    pub halted: bool,
    pub steps: u64,
}

impl Arm64Cpu {
    pub fn new() -> Self {
        Arm64Cpu {
            regs: [0; 32],
            pc: 0,
            nzcv: 0,
            halted: false,
            steps: 0,
        }
    }
}

impl Default for Arm64Cpu {
    fn default() -> Self { Self::new() }
}

impl CpuBackend for Arm64Cpu {
    fn reset(&mut self) {
        self.regs = [0; 32];
        self.pc = 0;
        self.nzcv = 0;
        self.halted = false;
        self.steps = 0;
        self.regs[31] = 0x0000_7FFF_F000;
        println!("[ARM64] CPU reset");
    }

    fn step(&mut self, _mem: &mut MemoryManager) -> Result<(), String> {
        if self.halted {
            return Err("[ARM64] CPU is halted".to_string());
        }
        Err(format!(
            "[ARM64] Unsupported instruction at 0x{:016X} (interpreter not implemented yet)",
            self.pc
        ))
    }

    fn run(&mut self, mem: &mut MemoryManager, max_steps: u64) -> Result<u64, String> {
        let start = self.steps;
        for _ in 0..max_steps {
            if self.halted { break; }
            self.step(mem)?;
        }
        Ok(self.steps.saturating_sub(start))
    }

    fn read_register(&self, reg: u8) -> u64 {
        if reg < 32 { self.regs[reg as usize] } else { 0 }
    }
    fn write_register(&mut self, reg: u8, value: u64) {
        if reg < 32 { self.regs[reg as usize] = value; }
    }
    fn pc(&self) -> u64 { self.pc }
    fn set_pc(&mut self, pc: u64) { self.pc = pc; }

    fn read_memory(&self, mem: &MemoryManager, addr: u64, size: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(size);
        for i in 0..size {
            out.push(mem.read8(addr + i as u64)?);
        }
        Ok(out)
    }
    fn write_memory(&self, mem: &mut MemoryManager, addr: u64, data: &[u8]) -> Result<(), String> {
        mem.write_bytes(addr, data)
    }
}

//! ARMv7 stub CPU backend (no guest execution).

use crate::cpu::CpuBackend;
use crate::memory::MemoryManager;

#[derive(Debug, Default)]
pub struct Arm32Cpu {
    regs: [u32; 16],
    cpsr: u32,
}

impl Arm32Cpu {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CpuBackend for Arm32Cpu {
    fn reset(&mut self) {
        self.regs = [0; 16];
        self.cpsr = 0x10; // user mode
    }

    fn step(&mut self, _mem: &mut MemoryManager) -> Result<(), String> {
        Err("Arm32Cpu::step not implemented — Stage A stub".into())
    }

    fn run(&mut self, _mem: &mut MemoryManager, _max_steps: u64) -> Result<u64, String> {
        Err("Arm32Cpu::run not implemented — Stage A stub".into())
    }

    fn read_register(&self, reg: u8) -> u64 {
        self.regs.get(reg as usize).copied().unwrap_or(0) as u64
    }

    fn write_register(&mut self, reg: u8, value: u64) {
        if let Some(r) = self.regs.get_mut(reg as usize) {
            *r = value as u32;
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
        mem.read(addr, size)
    }

    fn write_memory(
        &self,
        mem: &mut MemoryManager,
        addr: u64,
        data: &[u8],
    ) -> Result<(), String> {
        mem.write(addr, data)
    }
}

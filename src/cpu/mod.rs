pub mod arm32;
pub mod arm64;

pub use arm32::Arm32Cpu;
pub use arm64::Arm64Cpu;

use crate::memory::MemoryManager;

/// Common interface for CPU backends.
pub trait CpuBackend {
    fn reset(&mut self);
    fn step(&mut self, mem: &mut MemoryManager) -> Result<(), String>;
    fn run(&mut self, mem: &mut MemoryManager, max_steps: u64) -> Result<u64, String>;
    fn read_register(&self, reg: u8) -> u64;
    fn write_register(&mut self, reg: u8, value: u64);
    fn pc(&self) -> u64;
    fn set_pc(&mut self, pc: u64);
    fn read_memory(&self, mem: &MemoryManager, addr: u64, size: usize) -> Result<Vec<u8>, String>;
    fn write_memory(&self, mem: &mut MemoryManager, addr: u64, data: &[u8]) -> Result<(), String>;
}

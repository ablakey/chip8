use crate::opcode::OpCode;
use pretty_hex::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct Chip8 {
    memory: [u8; 4096],
    // cpu_registers: [u8; 16],
    // index_register: u16,
    program_counter: u16, // TODO: make this u16 and make memory indexable.
                          // graphics_buffer: [u8; 64 * 32], // 64 rows, 32 cols, row-major.
                          // delay_timer: u8,
                          // sound_timer: u8,
                          // stack: [u16; 16],
                          // stack_pointer: u16,
                          // keys: [u8; 16],
}

impl Chip8 {
    // Memory addresses (start, end).
    // const ADDR_INTERPRETER: (u16, u16) = (0x000, 0x1FF);
    // const ADDR_FONTSET: (u16, u16) = (0x050, 0x0A0);
    const ADDR_ROM: u16 = 0x200;

    pub fn init() -> Self {
        Self {
            memory: [0; 4096],
            // cpu_registers: [0; 16],
            // index_register: 0,
            program_counter: Chip8::ADDR_ROM,
            // graphics_buffer: [0; 64 * 32],
            // delay_timer: 0,
            // sound_timer: 0,
            // stack: [0; 16],
            // stack_pointer: 0,
            // keys: [0; 16],
        }
    }

    pub fn load_rom(&mut self, path: String) -> io::Result<()> {
        let start = Chip8::ADDR_ROM as usize;
        let mut f = File::open(path)?;
        f.read(&mut self.memory[start..])?;
        Ok(())
    }

    pub fn tick(&mut self) {
        // Fetch opcode.
        let opcode = self.get_word(self.program_counter);
        println!("{:x?}", opcode);

        // Decode opcode.
        let opcode = OpCode::from_word(opcode);
        println!("{:?}", opcode);
    }

    /// Return a byte from `address` in memory.
    fn get_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Return a 16-bit word from the two contiguous bytes beginning at `address` in memory.
    fn get_word(&self, address: u16) -> u16 {
        let high = self.memory[address as usize];
        let low = self.memory[address as usize + 1];
        ((high as u16) << 8) | low as u16
    }
}

#[cfg(debug_assertions)]
impl Chip8 {
    pub fn print_debug(&self) {
        println!("PC: {}", self.program_counter);
    }

    pub fn print_mem(&self) {
        let start = Chip8::ADDR_ROM as usize;
        println!("{:?}", self.memory[start..start + 200].to_vec().hex_dump());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROM_BYTES: &[u8] = &[
        96, 0, 97, 0, 162, 34, 194, 1, 50, 1, 162, 30, 208, 20, 112, 4, 48, 64, 18, 4, 96, 0, 113,
        4, 49, 32, 18, 4, 18, 28, 128, 64, 32, 16, 32, 64, 128, 16,
    ];

    /// Test that the machine initializes to a proper initial state.
    #[test]
    fn test_init() {
        let machine = Chip8::init();
        assert_eq!(machine.program_counter, Chip8::ADDR_ROM);
    }

    /// Test that the machine initializes to a proper zero state.
    #[test]
    fn test_load_rom() {
        let mut machine = Chip8::init();
        machine.load_rom(String::from("roms/maze.c8")).unwrap();
        let start = Chip8::ADDR_ROM as usize;
        let end = start + ROM_BYTES.len();
        assert_eq!(&machine.memory[start..end], ROM_BYTES);
    }
}

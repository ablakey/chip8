use std::fs::File;
use std::io;
use std::io::prelude::*;

pub struct Chip8 {
    pub memory: [u8; 4096],
    // cpu_registers: [u8; 16],
    // index_register: u16,
    // program_counter: u16,
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
    // const ADDR_ROM_RAM: (u16, u16) = (0x200, 0xFFF);

    pub fn init() -> Self {
        Self {
            memory: [0; 4096],
            // cpu_registers: [0; 16],
            // index_register: 0,
            // program_counter: 0,
            // graphics_buffer: [0; 64 * 32],
            // delay_timer: 0,
            // sound_timer: 0,
            // stack: [0; 16],
            // stack_pointer: 0,
            // keys: [0; 16],
        }
    }

    pub fn load_cartridge(&mut self, path: String) -> io::Result<()> {
        let mut f = File::open(path)?;
        let mut buffer = Vec::<u8>::new();
        f.read_to_end(&mut buffer)?;

        // Write buffer into memory.
        // TODO: This feels wrong. Surely I must be able to write the file into the array without
        // iterating over each element. And if the size is wrong, return an error.
        for (i, &n) in buffer.iter().enumerate() {
            self.memory[i] = n;
        }

        Ok(())
    }
}

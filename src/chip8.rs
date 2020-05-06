#![allow(non_snake_case)]
use pretty_hex::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;

/// Type aliases to make the code more legible. We aren't going to support nibbles and
/// triple-nibbles... tribbles? Hah! I tried to with a `ux` crate but the ergonomics were
/// unpleasant. They couldn't interact with the built-in primitives so easily, if I recall. I
/// am pretty sure that u4 and u12 not actually being those sizes will be fine, so long as we
/// perform bitwise masking on them carefully. The most significant nibbles will just be 0.
/// Rust won't type-check these though so I could pass a u4 where I meant to pass a u8.
#[allow(non_camel_case_types)]
pub type u4 = u8;
#[allow(non_camel_case_types)]
pub type u12 = u16;

/// A structure of unpacked symbols from an OpCode.
/// Not all symbols (and sometimes no symbols) are valid, depending on what the opcode is.
/// n: 4-bit constant
/// nn: 8-bit constant
/// nnn: 12-bit address
/// x: 4-bit register identifier
/// y: 4-bit register identifier
struct OpCodeSymbols {
    a: u4,
    x: u4,
    y: u4,
    n: u4,
    nn: u8,
    nnn: u12,
}

impl OpCodeSymbols {
    /// Return the symbols from an opcode's raw value.
    /// x and y need to be bit shifted to the least significant nibble before being casted to a
    /// u4 (actually a u8).
    fn from_value(opcode: u16) -> Self {
        return Self {
            a: ((opcode & 0xF000) >> 12) as u4,
            x: ((opcode & 0x0F00) >> 8) as u4,
            y: ((opcode & 0x00F0) >> 4) as u4,
            n: (opcode & 0x000F) as u4,
            nn: (opcode & 0x00FF) as u8,
            nnn: (opcode & 0x0FFF) as u12,
        };
    }
}

pub struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16], // 16 registers: V0 - VF
    // index_register: u16,
    program_counter: u16,
    pub graphics_buffer: [bool; 64 * 32], // 64 rows, 32 cols, row-major.
    // delay_timer: u8,
    // sound_timer: u8,
    // stack: [u16; 16],
    // stack_pointer: u16,
    // keys: [u8; 16],
    pub draw_graphics: bool,
}

impl Chip8 {
    // Memory addresses (start, end).
    // const ADDR_INTERPRETER: (u16, u16) = (0x000, 0x1FF);
    // const ADDR_FONTSET: (u16, u16) = (0x050, 0x0A0);
    const ADDRESS_ROM: u16 = 0x200;
    const OPCODE_SIZE: u16 = 2;

    pub fn init() -> Self {
        Self {
            memory: [0; 4096],
            registers: [0; 16],
            // index_register: 0,
            program_counter: Chip8::ADDRESS_ROM,
            graphics_buffer: [false; 64 * 32],
            // delay_timer: 0,
            // sound_timer: 0,
            // stack: [0; 16],
            // stack_pointer: 0,
            // keys: [0; 16],
            draw_graphics: false,
        }
    }

    pub fn load_rom(&mut self, path: String) -> io::Result<()> {
        let start = Chip8::ADDRESS_ROM as usize;
        let mut f = File::open(path)?;
        f.read(&mut self.memory[start..])?;
        Ok(())
    }

    /// Execute the next opcode.
    pub fn tick(&mut self) {
        // Reset flags.
        self.draw_graphics = false;
        // Execute opcode.
        let opcode = self.get_word(self.program_counter);
        println!("opcode: {:x?}", opcode);
        self.execute_opcode(opcode)
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

    ///  Advance the program counter.
    /// Because an opcode is 2 bits, it has to advance by that amount.
    fn increment_pc(&mut self) {
        self.program_counter += Chip8::OPCODE_SIZE;
    }

    fn execute_opcode(&mut self, opcode: u16) {
        #[rustfmt::skip]
    // These are possible opcode symbols, not all of which are valid. Depending on the matched
    // opcode, some of the symbols may be used.
    let OpCodeSymbols { a, x, y, n, nnn, nn } = OpCodeSymbols::from_value(opcode);

        // The order of these match branches are important.
        // Some opcodes are more specific than others.
        match (a, x, y, n) {
            (0, 0, 0xE, 0) => self.CLR(),
            (0, 0, 0xE, 0xE) => self.RTS(),
            (0, _, _, _) => self.SYS(nnn),
            (1, _, _, _) => self.JUMP(nnn),
            (2, _, _, _) => self.CALL(nnn),
            (3, _, _, _) => self.SKE(x, nn),
            (4, _, _, _) => self.SKNE(x, nn),
            (5, _, _, 0) => self.SKRE(x, y),
            (6, _, _, _) => self.LOAD(x, nn),
            (7, _, _, _) => self.ADD(x, nn),
            (8, _, _, 0) => self.MOVE(x, y),
            (8, _, _, 1) => self.OR(x, y),
            (8, _, _, 2) => self.AND(x, y),
            (8, _, _, 3) => self.XOR(x, y),
            (8, _, _, 4) => self.ADDR(x, y),
            (8, _, _, 5) => self.SUB(x, y),
            (8, _, _, 6) => self.SHR(x, y),
            (8, _, _, 7) => self.SUBN(x, y),
            (8, _, _, 0xE) => self.SHL(x, y),
            (9, _, _, 0) => self.SKRNE(x, y),
            (0xA, _, _, _) => self.LOADI(nnn),
            (0xB, _, _, _) => self.JUMPI(nnn),
            (0xC, _, _, _) => self.RAND(x, nn),
            (0xD, _, _, _) => self.DRAW(x, y, n),
            (0xE, _, 9, 0xE) => self.SKPR(x),
            (0xE, _, 0xA, 1) => self.SKUP(x),
            (0xF, _, 0, 7) => self.MOVED(x),
            (0xF, _, 0, 0xA) => self.KEYD(x),
            (0xF, _, 1, 5) => self.LOADD(x),
            (0xF, _, 1, 8) => self.LOADS(x),
            (0xF, _, 1, 0xE) => self.ADDI(x),
            (0xF, _, 2, 9) => self.LDSPR(x),
            (0xF, _, 3, 3) => self.BCD(x),
            (0xF, _, 5, 5) => self.STOR(x),
            (0xF, _, 6, 5) => self.READ(x),
            (_, _, _, _) => panic!("Tried to call opcode {:X?} that is not handled.", opcode),
        };
    }

    /// Clear the graphics buffer.
    fn CLR(&mut self) {
        self.graphics_buffer = [false; 64 * 32];
        self.draw_graphics = true;
        self.increment_pc();
    }

    fn RTS(&mut self) {}
    fn SYS(&mut self, nnn: u12) {}
    fn JUMP(&mut self, nnn: u12) {}
    fn CALL(&mut self, nnn: u12) {
        panic!(
            "opcode CALL not implemented. Attempted at address: {:#X}",
            nnn
        )
    }
    fn SKE(&mut self, x: u4, nn: u8) {
        panic!("Not implemented.")
    }
    fn SKNE(&mut self, x: u4, nn: u8) {
        panic!("Not implemented.")
    }
    fn SKRE(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }

    /// Set register X to NN;
    fn LOAD(&mut self, x: u4, nn: u8) {
        self.registers[x as usize] = nn;
        self.increment_pc();
    }

    fn ADD(&mut self, x: u4, nn: u8) {
        panic!("Not implemented.")
    }
    fn MOVE(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn OR(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn AND(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn XOR(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn ADDR(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn SUB(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn SHR(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn SUBN(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn SHL(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn SKRNE(&mut self, x: u4, y: u4) {
        panic!("Not implemented.")
    }
    fn LOADI(&mut self, nnn: u12) {
        panic!("Not implemented.")
    }
    fn JUMPI(&mut self, nnn: u12) {
        panic!("Not implemented.")
    }
    fn RAND(&mut self, x: u4, nn: u8) {
        panic!("Not implemented.")
    }
    fn DRAW(&mut self, x: u4, y: u4, n: u4) {
        panic!("Not implemented.")
    }
    fn SKPR(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn SKUP(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn MOVED(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn KEYD(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn LOADD(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn LOADS(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn ADDI(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn LDSPR(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn BCD(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn STOR(&mut self, x: u4) {
        panic!("Not implemented.")
    }
    fn READ(&mut self, x: u4) {
        panic!("Not implemented.")
    }
}

#[cfg(debug_assertions)]
impl Chip8 {
    pub fn print_debug(&self) {
        println!("PC: {}", self.program_counter);
    }

    pub fn print_mem(&self) {
        let start = Chip8::ADDRESS_ROM as usize;
        println!("{:?}", self.memory[start..start + 200].to_vec().hex_dump());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROM_BYTES: &[u8] = &[
        96, 0, 97, 0, 162, 34, 194, 1, 50, 1, 162, 30, 208, 20, 112, 4, 48, 64, 18, 4, 96, 0, 113,
        4, 49, 32, 18, 4, 18, 28, 128, 64, 32, 16, 32, 64, 128, 16,
    ];

    /// Test that the machine initializes to a proper initial state.
    #[test]
    fn test_init() {
        let machine = Chip8::init();
        assert_eq!(machine.program_counter, Chip8::ADDRESS_ROM);
    }

    /// Test that the machine initializes to a proper zero state.
    #[test]
    fn test_load_rom() {
        let mut machine = Chip8::init();
        machine.load_rom(String::from("roms/maze.c8")).unwrap();
        let start = Chip8::ADDRESS_ROM as usize;
        let end = start + TEST_ROM_BYTES.len();
        assert_eq!(&machine.memory[start..end], TEST_ROM_BYTES);
    }

    /// The CLR opcode must set all graphics to 0, set the graphics update flag, and increment
    // the program counter.
    #[test]
    fn test_opcode_clr() {
        let mut machine = Chip8::init();
        machine.execute_opcode(0x00E0);

        assert!(machine.graphics_buffer.iter().all(|&n| !n));
        assert!(machine.draw_graphics);
        assert_eq!(
            machine.program_counter,
            Chip8::ADDRESS_ROM + Chip8::OPCODE_SIZE
        );
    }
}

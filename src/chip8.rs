#![allow(non_snake_case)]
use pretty_hex::*;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::prelude::*;

/// A structure of unpacked symbols from an OpCode.
/// Not all symbols (and sometimes no symbols) are valid, depending on what the opcode is.
/// n: 4-bit constant
/// nn: 8-bit constant
/// nnn: 12-bit address
/// x: 4-bit register identifier
/// y: 4-bit register identifier
#[derive(Debug)]
struct OpCodeSymbols {
    a: usize,
    x: usize,
    y: usize,
    n: usize,
    nn: usize,
    nnn: usize,
}

impl OpCodeSymbols {
    /// Return the symbols from an opcode's raw value.
    /// x and y need to be bit shifted to the least significant nibble before being casted to a
    /// usize (actually a usize).
    fn from_value(opcode: usize) -> Self {
        return Self {
            a: ((opcode & 0xF000) >> 12),
            x: ((opcode & 0x0F00) >> 8),
            y: ((opcode & 0x00F0) >> 4),
            n: (opcode & 0x000F),
            nn: (opcode & 0x00FF),
            nnn: (opcode & 0x0FFF),
        };
    }
}

#[derive(Clone)]
pub struct Chip8 {
    memory: [usize; 4096],                // 4k of 8 bit memory.
    registers: [usize; 16],               // 16  8-bit registers: V0 - VF
    index_register: usize,                // 16-bit register (for memory addressing)
    program_counter: usize,               // 16-bit program counter.
    pub graphics_buffer: [bool; 64 * 32], // 64 rows, 32 cols, row-major.
    delay_timer: usize,
    sound_timer: usize,
    stack: [usize; 16],
    stack_pointer: usize,
    // keys: [usize; 16],
    pub has_graphics_update: bool,
}

/// Core feature implenentation.
impl Chip8 {
    pub const CPU_FREQUENCY: usize = 2000; // microseconds -> 500Hz
    pub const TIMER_FREQUENCY: usize = 16667; // microseconds -> 60Hz

    // Memory addresses (start, end).
    // const ADDR_INTERPRETER: (usize, usize) = (0x000, 0x1FF);
    // const ADDR_FONTSET: (usize, usize) = (0x050, 0x0A0);
    const ADDRESS_ROM: usize = 0x200;
    const OPCODE_SIZE: usize = 2;

    pub fn init() -> Self {
        Self {
            memory: [0; 4096],
            registers: [0; 16],
            index_register: 0,
            program_counter: Chip8::ADDRESS_ROM,
            graphics_buffer: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            // keys: [0; 16],
            has_graphics_update: false,
        }
    }

    pub fn load_rom(&mut self, path: String) -> io::Result<()> {
        let start = Chip8::ADDRESS_ROM;
        let mut buffer = Vec::new();
        let mut f = File::open(path)?;

        f.read_to_end(&mut buffer)?;

        for (idx, &value) in buffer.iter().enumerate() {
            self.memory[idx + start] = value as usize;
        }

        Ok(())
    }

    /// Execute the next opcode.
    pub fn tick(&mut self) {
        // Reset flags.
        self.has_graphics_update = false;

        // Get opcode by combining two bits from memory.
        let low = self.memory[self.program_counter + 1];
        let high = self.memory[self.program_counter];
        let opcode = ((high) << 8) | low;
        let opcode_symbols = OpCodeSymbols::from_value(opcode);
        println!("opcode: {:x?}", opcode);

        self.execute_opcode(&opcode_symbols);

        // Increment PC unless opcode is JUMP, JUMPI, or CALL.
        if ![0xB, 0x2, 0x1].contains(&opcode_symbols.a) {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Decrement both sound and delay timers.
    /// This should be getting called at 60hz by the emulator's controller.
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn execute_opcode(&mut self, opcode_symbols: &OpCodeSymbols) {
        #[rustfmt::skip]
    // These are possible opcode symbols, not all of which are valid. Depending on the matched
    // opcode, some of the symbols may be used.
    let OpCodeSymbols { a, x, y, n, nnn, nn } = *opcode_symbols;

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
            (_, _, _, _) => panic!("Tried to call {:?} but isn't handled.", opcode_symbols),
        };
    }
}
/// Opcode implementation.
impl Chip8 {
    /// Clear the graphics buffer.
    fn CLR(&mut self) {
        self.graphics_buffer = [false; 64 * 32];
        self.has_graphics_update = true;
    }

    fn RTS(&mut self) {}

    // Jump to machine code routine at nnn. Not implemented in modern CHIP8 emulators.
    fn SYS(&mut self, nnn: usize) {
        panic!(
            "opcode SYS not implemented. Attempted at address: {:#X}",
            nnn
        );
    }

    /// Jump PC to NNN.
    fn JUMP(&mut self, nnn: usize) {
        self.program_counter = nnn;
    }

    /// Call subroutine at NNN.
    fn CALL(&mut self, nnn: usize) {
        // Maintain current PC in the stack to be able to return from subroutine.
        self.stack[self.stack_pointer] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = nnn;
    }

    /// Skip next instruction if VX == NN.
    fn SKE(&mut self, x: usize, nn: usize) {
        if self.registers[x] == nn {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Skip next instruction if VX != NN.
    fn SKNE(&mut self, x: usize, nn: usize) {
        if self.registers[x] != nn {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Skip next instruction if VX == VY;
    fn SKRE(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Set register X to NN;
    fn LOAD(&mut self, x: usize, nn: usize) {
        self.registers[x] = nn;
    }

    /// Add NN to VX. Carry flag isn't changed.
    fn ADD(&mut self, x: usize, nn: usize) {
        self.registers[x] = (self.registers[x] + nn) / 0x100
    }

    /// Write VY to VX.
    fn MOVE(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    /// VX = VX | VY.
    fn OR(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] | self.registers[y];
    }

    /// VX = VX & VY.
    fn AND(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] & self.registers[y];
    }

    /// VX = VX ^ VY.
    fn XOR(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[x] ^ self.registers[y];
    }

    /// Add VX to VY. Set VF to 1 if overflow, else 0.
    fn ADDR(&mut self, x: usize, y: usize) {
        let vx = self.registers[x];
        let vy = self.registers[y];

        self.registers[0xF] = if vx + vy >= 0x100 { 1 } else { 0 };
        self.registers[x] = (vx + vy) / 0x100;
    }

    fn SUB(&mut self, x: usize, y: usize) {
        self.not_implemented();
    }
    fn SHR(&mut self, x: usize, y: usize) {
        self.not_implemented();
    }
    fn SUBN(&mut self, x: usize, y: usize) {
        self.not_implemented();
    }
    fn SHL(&mut self, x: usize, y: usize) {
        self.not_implemented();
    }

    /// Skip next instruction if VX != VY.
    fn SKRNE(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Set index register to NNN.
    fn LOADI(&mut self, nnn: usize) {
        self.index_register = nnn;
    }

    fn JUMPI(&mut self, nnn: usize) {
        self.program_counter = self.registers[0] + nnn;
    }

    /// Set VX to result of bitwise: NN & RANDOM
    fn RAND(&mut self, x: usize, nn: usize) {
        let rand = rand::thread_rng().gen_range(0, 255) & nn;
        self.registers[x] = rand;
    }

    /// Draws N sprite lines from memory[I] to coordinates (VX, VY). VF is set high if collision.
    fn DRAW(&mut self, x: usize, y: usize, n: usize) {
        // Read n bytes from memory starting at I.
        let start = self.index_register;
        let end = self.index_register + n;

        let vx = self.registers[x];
        let vy = self.registers[y];

        for (row, &pixels) in self.memory[start..end].iter().enumerate() {
            for col in 0..8 {
                // Get a pixel by masking 0x80 aka `0b10000000` and shifting the 1 right each time.
                // If it is 1, do collision detection and set the pixel.
                if pixels & 0x80 >> col > 0 {
                    // Get current pixel.
                    let idx = vx + col + ((vy + row) * 64);
                    let current_pixel = self.graphics_buffer[idx];

                    // If collision, set VF to 1.
                    if current_pixel {
                        self.registers[0xF] = 1;
                    }

                    // Update the pixel with XOR.
                    self.graphics_buffer[idx] = current_pixel ^ true;
                }
            }
        }

        self.has_graphics_update = true;
    }
    fn SKPR(&mut self, x: usize) {
        self.not_implemented();
    }
    fn SKUP(&mut self, x: usize) {
        self.not_implemented();
    }

    /// Load Delay Timer into VX.
    fn MOVED(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
    }

    fn KEYD(&mut self, x: usize) {
        self.not_implemented();
    }

    /// Set Delay Timer to VX.
    fn LOADD(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
    }

    /// Set Sound Timer to VX.
    fn LOADS(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
    }

    /// Add VX to I.  VF set to 1 if there is an overflow, else 0.
    fn ADDI(&mut self, x: usize) {
        let vx = self.registers[x];
        let i = self.index_register;

        self.registers[0xF] = if vx + i >= 0x1000 { 1 } else { 0 };
        self.index_register = (vx + i) / 0x1000
    }

    fn LDSPR(&mut self, x: usize) {
        self.not_implemented();
    }
    fn BCD(&mut self, x: usize) {
        self.not_implemented();
    }
    fn STOR(&mut self, x: usize) {
        self.not_implemented();
    }
    fn READ(&mut self, x: usize) {
        self.not_implemented();
    }
}

/// Debug functions.
// #[cfg(debug_assertions)]
impl Chip8 {
    pub fn print_debug(&self) {
        println!("PC: {}", self.program_counter);
        println!("I: {}", self.index_register);
        println!("Registers: {:x?}", self.registers)
    }

    pub fn print_mem(&self) {
        let start = Chip8::ADDRESS_ROM;
        println!(
            "{:?}",
            self.memory[start..start + 200]
                .to_vec()
                .iter()
                .map(|&f| f as u8)
                .collect::<Vec<u8>>()
                .hex_dump()
        );
    }

    fn not_implemented(&self) {
        self.print_debug();
        self.print_mem();
        panic!("Not implemented.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROM_BYTES: &[usize] = &[
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
        let start = Chip8::ADDRESS_ROM;
        let end = start + TEST_ROM_BYTES.len();
        assert_eq!(&machine.memory[start..end], TEST_ROM_BYTES);
    }

    /// The CLR opcode must set all graphics to 0, set the graphics update flag, and increment
    // the program counter.
    #[test]
    fn test_opcode_clr() {
        let mut machine = Chip8::init();
        let opcode_symbols = OpCodeSymbols::from_value(0x00E0);
        machine.execute_opcode(&opcode_symbols);

        assert!(machine.graphics_buffer.iter().all(|&n| !n));
        assert!(machine.has_graphics_update);
    }

    /// Timers should decrement by 1 each time `decrement_timers` is called, but never fall below 0.
    #[test]
    fn test_decrement_timers() {
        let mut machine = Chip8::init();
        machine.delay_timer = 5;
        machine.sound_timer = 0;

        machine.decrement_timers();

        assert_eq!(machine.sound_timer, 0);
        assert_eq!(machine.delay_timer, 4);
    }

    /// The Draw opcode should XOR black and white bits to the graphics buffer with overflow to next
    // line. It is byte-encoded sprite-based. See specifications online for more details.
    #[test]
    fn test_draw() {
        let mut machine = Chip8::init();
        machine.index_register = 0x204; // Where to look for the sprite data.
        machine.memory[0x204] = 0xCC; // 8 bits to draw:  11001100
        machine.memory[0x205] = 0xFF; // 8 bits to draw:  11111111
        machine.registers[0] = 0; // x coordinate
        machine.registers[1] = 0; // y-coordinate

        machine.DRAW(0, 1, 1); // Get x,y from 0,1 and draw a single byte of data.

        // The segment of the graphics buffer is as expected. We drew four pixels at x= 0, 1, 4, 5.
        assert_eq!(
            machine.graphics_buffer[0..8],
            [true, true, false, false, true, true, false, false]
        );

        // In this case, we draw two lines, not one.
        machine.DRAW(0, 1, 2);

        // XORing has turned off the pixels that were on.
        assert_eq!(
            machine.graphics_buffer[0..8],
            [false, false, false, false, false, false, false, false]
        );

        // But the second line is all on now.
        assert_eq!(
            machine.graphics_buffer[64..72],
            [true, true, true, true, true, true, true, true]
        )
    }
}

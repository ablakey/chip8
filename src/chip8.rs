#![allow(non_snake_case)]
use pretty_hex::*;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::prelude::*;
/// A structure of unpacked symbols from an OpCode.
/// Not all symbols (and sometimes no symbols) are valid, depending on what the opcode is.
/// Sometimes the opcode is identified by a combination of nibbles rather than just the first one.
#[derive(Debug)]
struct OpCodeSymbols {
    a: usize,   // 4-bit opcode identifier.
    x: usize,   // 4-bit register identifier
    y: usize,   // 4-bit register identifier
    n: usize,   // 4-bit constant
    nn: usize,  // 8-bit constant
    nnn: usize, // 12-bit address
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
    cycle: usize,                         // The current cycle count.
    delay_timer: usize,                   // TODO
    index_register: usize,                // 16-bit register (for memory addressing) aka I
    keyd_register: usize,                 // 8 bit register for the KEYD opcode.
    keys: [bool; 16],                     // TODO
    memory: [usize; 4096],                // 4k of 8 bit memory.
    program_counter: usize,               // 16-bit program counter.
    pub graphics_buffer: [bool; 64 * 32], // 64 rows, 32 cols, row-major.
    pub has_graphics_update: bool,        // TODO
    pub last_opcode: usize,               // Last run opcode.
    pub rom_size: usize,                  // Size of loaded ROM in bytes.
    pub wait_for_input: bool,             // Wait for input before next tick?
    registers: [usize; 16],               // 16  8-bit registers: V0 - VF
    sound_timer: usize,                   // TODO
    stack_pointer: usize,                 // TODO
    stack: [usize; 16],                   // TODO
}

/// Core feature implenentation.
impl Chip8 {
    // Memory addresses (start, end).
    // const ADDR_INTERPRETER: (usize, usize) = (0x000, 0x1FF);
    const ADDRESS_FONT: usize = 0x050; // Where the font is stored in memory.
    const ADDRESS_ROM: usize = 0x200;
    const OPCODE_SIZE: usize = 2;

    #[rustfmt::skip]
    /// 4x5 raster font. Each hex character represents a row of pixels.
    /// Only the least significant four pixels are used.
    const FONT: [usize; 80] = [
        	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        	0x20, 0x60, 0x20, 0x20, 0x70, // 1
        	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        	0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    pub fn init() -> Self {
        // Load font into memory.
        let mut memory = [0; 4096];
        Chip8::FONT
            .iter()
            .enumerate()
            .for_each(|(i, &n)| memory[i + Chip8::ADDRESS_FONT] = n);

        Self {
            cycle: 0,
            delay_timer: 0,
            graphics_buffer: [false; 64 * 32],
            has_graphics_update: false,
            index_register: 0,
            keyd_register: 0,
            keys: [false; 16],
            last_opcode: 0,
            memory,
            program_counter: Chip8::ADDRESS_ROM,
            registers: [0; 16],
            rom_size: 0,
            sound_timer: 0,
            stack_pointer: 0,
            stack: [0; 16],
            wait_for_input: false,
        }
    }

    pub fn load_rom(&mut self, path: &String) -> io::Result<()> {
        let start = Chip8::ADDRESS_ROM;
        let mut buffer = Vec::new();
        let mut f = File::open(path)?;

        f.read_to_end(&mut buffer)?;

        for (idx, &value) in buffer.iter().enumerate() {
            self.memory[idx + start] = value as usize;
        }

        self.rom_size = buffer.len();
        Ok(())
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

    /// Set the current state of all keys into the machine's memory.
    pub fn set_keys(&mut self, keys: [bool; 16]) {
        // If waiting for input and any key is pressed, continue self.KEYD opcode.
        if self.wait_for_input {
            for (key_name, &is_pressed) in keys.iter().enumerate() {
                if is_pressed {
                    self.KEYD_RESUME(key_name);
                    break;
                }
            }
        };

        self.keys = keys;
    }

    pub fn tick(&mut self) {
        self.cycle += 1;

        // Do nothing if awaiting input.
        if self.wait_for_input {
            return;
        }

        // Every tick, process 1 opcode.
        self.execute_opcode();

        // Every 8th tick, decrement timers.
        if self.cycle % 8 == 0 {
            self.decrement_timers();
        }
    }

    pub fn execute_opcode(&mut self) {
        // These are possible opcode symbols, not all of which are valid. Depending on the matched
        // opcode, some of the symbols may be used.

        // Reset flags.
        self.has_graphics_update = false;

        let opcode = self.get_opcode();
        let opcode_symbols = OpCodeSymbols::from_value(opcode);

        let OpCodeSymbols {
            a,
            x,
            y,
            n,
            nnn,
            nn,
        } = opcode_symbols;

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

        // Increment PC unless opcode is JUMP, JUMPI, or CALL.
        if ![0xB, 0x2, 0x1].contains(&opcode_symbols.a) {
            self.program_counter += Chip8::OPCODE_SIZE;
        }

        self.last_opcode = opcode;
    }

    fn get_opcode(&self) -> usize {
        // Get opcode by combining two bits from memory.
        let low = self.memory[self.program_counter + 1];
        let high = self.memory[self.program_counter];
        ((high) << 8) | low
    }
}
/// Opcode implementation.
impl Chip8 {
    /// Clear the graphics buffer.
    fn CLR(&mut self) {
        self.graphics_buffer = [false; 64 * 32];
        self.has_graphics_update = true;
    }

    /// Return from subroutine.
    fn RTS(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer];
    }

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
        self.registers[x] = (self.registers[x] + nn) % 0x100
    }

    /// Write VY to VX.
    fn MOVE(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    /// VX = VX | VY.
    fn OR(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    /// VX = VX & VY.
    fn AND(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    /// VX = VX ^ VY.
    fn XOR(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    /// Add VX to VY. Set VF to 1 if overflow, else 0.
    fn ADDR(&mut self, x: usize, y: usize) {
        let vx = self.registers[x];
        let vy = self.registers[y];

        self.registers[0xF] = if vx + vy > 0xFF { 1 } else { 0 };
        self.registers[x] = (vx + vy) % 0x100;
    }

    fn SUB(&mut self, x: usize, y: usize) {
        let vx = self.registers[x];
        let vy = self.registers[y];

        // Wrapping subtract as u8 to ensure it wraps around, as intended by the hardware.
        self.registers[x] = (vx as u8).wrapping_sub(vy as u8) as usize;

        self.registers[0xF] = if vx > vy { 1 } else { 0 };
    }

    // Store LSB of VX  to VF then bit shift right (divide by 2).
    /// Unused y. Opcode was undocumented, possibly unintended.
    /// TODO: understand y better. some docs claim it gets used.
    fn SHR(&mut self, x: usize, _y: usize) {
        let vx = self.registers[x];
        self.registers[0xF] = vx & 0x1;
        self.registers[x] = vx >> 1;
    }

    fn SUBN(&mut self, x: usize, y: usize) {
        self.not_implemented();
    }

    /// Store most-significant bit of VX in VF then shift VX left by 1 (multiply by 2).
    /// Unused y. Opcode was undocumented, possibly unintended.
    /// TODO: understand y better. some docs claim it gets used.
    fn SHL(&mut self, x: usize, _y: usize) {
        let vx = self.registers[x];
        // Mask by 0xFF to prevent values larger than 8 bits.
        self.registers[0xF] = (vx & 0x80) >> 7;
        self.registers[x] = (vx << 1) & 0xFF;
    }

    /// Skip next instruction if VX != VY.
    fn SKRNE(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
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

                    // If collision, set VF to 1, else 0.
                    self.registers[0xF] = if current_pixel { 1 } else { 0 };

                    // Update the pixel with XOR.
                    self.graphics_buffer[idx] = current_pixel ^ true;
                }
            }
        }
        self.has_graphics_update = true;
    }

    // Skip next operation if key stored at VX is pressed.
    fn SKPR(&mut self, x: usize) {
        let vx = self.registers[x];
        if self.keys[vx] {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    // Skip next operation if key stored at VX is not pressed.
    fn SKUP(&mut self, x: usize) {
        let vx = self.registers[x];
        if !self.keys[vx] {
            self.program_counter += Chip8::OPCODE_SIZE;
        }
    }

    /// Load Delay Timer into VX.
    fn MOVED(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
    }

    /// Wait for the next input before resuming the remainder of this operation.
    fn KEYD(&mut self, x: usize) {
        self.keyd_register = x;
        self.wait_for_input = true;
    }

    /// Set self.keyd_register to the name of the key (0-F).
    /// This is called when keyboard input has been received.
    fn KEYD_RESUME(&mut self, key: usize) {
        self.registers[self.keyd_register] = key;
        self.wait_for_input = false;
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

        self.registers[0xF] = if vx + i > 0xFFF { 1 } else { 0 };
        self.index_register = (vx + i) % 0x1000
    }

    // Set I to location of sprite for character VX.
    fn LDSPR(&mut self, x: usize) {
        let character = self.registers[x];
        self.index_register = Chip8::ADDRESS_FONT + character * 5; // Each character is 5 bytes.
    }

    // Store binary-coded decimal of VX at I, I+1, I+2.
    fn BCD(&mut self, x: usize) {
        let i = self.index_register;
        let vx = self.registers[x];

        // # TODO understand better
        self.memory[i] = vx / 100;
        self.memory[i + 1] = (vx % 100) / 10;
        self.memory[i + 2] = vx % 10;
    }

    // Store registers to memory starting at I.
    fn STOR(&mut self, x: usize) {
        for n in 0..x + 1 {
            self.memory[self.index_register + n] = self.registers[n];
        }
    }

    /// Populate registers V0 to VX with data starting at I.
    fn READ(&mut self, x: usize) {
        for n in 0..x + 1 {
            self.registers[n] = self.memory[self.index_register + n];
        }
    }
}

/// Debug functions.
// #[cfg(debug_assertions)]
impl Chip8 {
    pub fn format_debug(&self) -> String {
        let keys: Vec<usize> = self.keys.iter().map(|&k| if k { 1 } else { 0 }).collect();
        [
            format!("PC:      {:x}\n", self.program_counter - Chip8::ADDRESS_ROM),
            format!("SP:      {:x}\n", self.stack_pointer),
            format!("I:       {:x}\n", self.index_register),
            format!("Opcode:  {:#X}\n", self.last_opcode),
            format!("Cycle #: {}\n", self.cycle),
            format!("Registers: {:x?}\n", self.registers),
            format!("Stack:     {:x?}\n", self.stack),
            format!("Keys:      {:?}\n", keys),
        ]
        .concat()
    }

    pub fn format_memory(&self) -> String {
        let start = Chip8::ADDRESS_ROM;
        format!(
            "{:?}",
            self.memory[start..start + self.rom_size]
                .to_vec()
                .iter()
                .map(|&f| f as u8)
                .collect::<Vec<u8>>()
                .hex_dump()
        )
    }

    fn not_implemented(&self) {
        panic!("Not implemented. Called: {:X}.", self.get_opcode());
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
        machine.load_rom(&String::from("roms/maze.c8")).unwrap();
        let start = Chip8::ADDRESS_ROM;
        let end = start + TEST_ROM_BYTES.len();
        assert_eq!(&machine.memory[start..end], TEST_ROM_BYTES);
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

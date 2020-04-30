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

/// OpCode enumerates all possible opcodes. Each variant is a tuple of 0-3 elements depending on
/// The opcode's pattern. Details from: https://en.wikipedia.org/wiki/CHIP-8#Opcode_table and
/// https://github.com/craigthomas/Chip8Assembler#mnemonic-table
#[derive(Debug, PartialEq)]
pub enum OpCode {
    SYS { nnn: u12 },             // 0NNN Call RCA 1802 program
    CLR,                          // 00E0 Clear screen
    RTS,                          // 00EE Return from subroutine
    JUMP { nnn: u12 },            // 1NNN Jump to address
    CALL { nnn: u12 },            // 2NNN Call subroutine
    SKE { x: u4, nn: u8 },        // 3XNN Skip next instruction if x equals nn
    SKNE { x: u4, nn: u8 },       // 4XNN Do not skip next instruction if x equals nn
    SKRE { x: u4, y: u4 },        // 5XY0 Skip if x equals y
    LOAD { x: u4, nn: u8 },       // 6XNN Load x with value nn
    ADD { x: u4, nn: u8 },        // 7XNN Add value nn to x
    MOVE { x: u4, y: u4 },        // 8XY0 Move value from x to y
    OR { x: u4, y: u4 },          // 8XY1 Perform logical OR on x and y and store in y
    AND { x: u4, y: u4 },         // 8XY2 Perform logical AND on x and y and store in y
    XOR { x: u4, y: u4 },         // 8XY3 Perform logical XOR on x and y and store in y
    ADDR { x: u4, y: u4 },        // 8XY4 Add x to y and store in x - register F set on carry
    SUB { x: u4, y: u4 },         // 8XY5 Subtract x from y and store in x. F set on !borrow
    SHR { x: u4, y: u4 },         // 8XY6 Shift bits in x 1 bit right, store in y. Bit 0 shifts to F
    SUBN { x: u4, y: u4 },        // 8XY7 Sets VX to VY minus VX. VF to 0 when borrow, else 1
    SHL { x: u4, y: u4 },         // 8XYE Shift bits in x 1 bit left, store in y. Bit 7 shifts to  F
    SKRNE { x: u4, y: u4 },       // 9XY0 Skip next instruction if x not equal y
    LOADI { nnn: u12 },           // ANNN Load index with value nnn
    JUMPI { nnn: u12 },           // BNNN Jump to address nnn + index
    RAND { x: u4, nn: u8 },       // CXNN Generate random number between 0 and nn and store in y
    DRAW { x: u4, y: u4, n: u4 }, // DXYN Draw n byte sprite at x location x, y location y
    SKPR { x: u4 },               // EX9E Skip next instruction if the key in x is pressed
    SKUP { x: u4 },               // EXA1 Skip next instruction if the key in x is not pressed
    MOVED { x: u4 },              // FX07 Move delay timer value into y
    KEYD { x: u4 },               // FX0A Wait for keypress and store in y
    LOADD { x: u4 },              // FX15 Load delay timer with value in x
    LOADS { x: u4 },              // FX18 Load sound timer with value in x
    ADDI { x: u4 },               // FX1E Add value in x to index
    LDSPR { x: u4 },              // FX29 Load index with sprite from x
    BCD { x: u4 },                // FX33 Store the binary coded decimal value of x at index
    STOR { x: u4 },               // FX55 Store the values of x registers at index
    READ { x: u4 },               // FX65 Read back the stored values at index into registers
}

impl OpCode {
    pub fn from_value(opcode: u16) -> Self {
        #[rustfmt::skip]
        // These are possible opcode symbols, not all of which are valid. Depending on the matched
        // opcode, some of the symbols may be used.
        let OpCodeSymbols { a, x, y, n, nnn, nn } = OpCodeSymbols::from_value(opcode);

        // The order of these match branches are important.
        // Some opcodes are more specific than others.
        let opcode = match (a, x, y, n) {
            (0, 0, 0xE, 0) => OpCode::CLR,
            (0, 0, 0xE, 0xE) => OpCode::RTS,
            (0, _, _, _) => OpCode::SYS { nnn },
            (1, _, _, _) => OpCode::JUMP { nnn },
            (2, _, _, _) => OpCode::CALL { nnn },
            (3, _, _, _) => OpCode::SKE { x, nn },
            (4, _, _, _) => OpCode::SKNE { x, nn },
            (5, _, _, 0) => OpCode::SKRE { x, y },
            (6, _, _, _) => OpCode::LOAD { x, nn },
            (7, _, _, _) => OpCode::ADD { x, nn },
            (8, _, _, 0) => OpCode::MOVE { x, y },
            (8, _, _, 1) => OpCode::OR { x, y },
            (8, _, _, 2) => OpCode::AND { x, y },
            (8, _, _, 3) => OpCode::XOR { x, y },
            (8, _, _, 4) => OpCode::ADDR { x, y },
            (8, _, _, 5) => OpCode::SUB { x, y },
            (8, _, _, 6) => OpCode::SHR { x, y },
            (8, _, _, 7) => OpCode::SUBN { x, y },
            (8, _, _, 0xE) => OpCode::SHL { x, y },
            (9, _, _, 0) => OpCode::SKRNE { x, y },
            (0xA, _, _, _) => OpCode::LOADI { nnn },
            (0xB, _, _, _) => OpCode::JUMPI { nnn },
            (0xC, _, _, _) => OpCode::RAND { x, nn },
            (0xD, _, _, _) => OpCode::DRAW { x, y, n },
            (0xE, _, 9, 0xE) => OpCode::SKPR { x },
            (0xE, _, 0xA, 1) => OpCode::SKUP { x },
            (0xF, _, 0, 7) => OpCode::MOVED { x },
            (0xF, _, 0, 0xA) => OpCode::KEYD { x },
            (0xF, _, 1, 5) => OpCode::LOADD { x },
            (0xF, _, 1, 8) => OpCode::LOADS { x },
            (0xF, _, 1, 0xE) => OpCode::ADDI { x },
            (0xF, _, 2, 9) => OpCode::LDSPR { x },
            (0xF, _, 3, 3) => OpCode::BCD { x },
            (0xF, _, 5, 5) => OpCode::STOR { x },
            (0xF, _, 6, 5) => OpCode::READ { x },
            (_, _, _, _) => panic!("Tried to call opcode {:X?} that is not handled.", opcode),
        };

        return opcode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcodes() {
        let opcode_tests = [
            (0x00E0, OpCode::CLR),
            (0xD123, OpCode::DRAW { x: 1, y: 2, n: 3 }),
            (0x00EE, OpCode::RTS),
        ];

        for (input, opcode) in opcode_tests.iter() {
            assert!(OpCode::from_value(*input) == *opcode)
        }
    }

    #[test]
    fn test_opcode_symbols_from_value() {
        #[rustfmt::skip]
        let OpCodeSymbols { n, nn, nnn, x, y, .. } = OpCodeSymbols::from_value(0xABCD);

        assert_eq!(n, 0xD);
        assert_eq!(nn, 0xCD);
        assert_eq!(nnn, 0xBCD);
        assert_eq!(x, 0xB);
        assert_eq!(y, 0xC);
    }
}

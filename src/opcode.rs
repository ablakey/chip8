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
/// The opcode's pattern. The OpCode
#[derive(Debug, PartialEq)]
pub enum OpCode {
    SYS { nnn: u12 },             // 0NNN
    CLR,                          // 00E0
    RTS,                          // 00EE
    JUMP { nnn: u12 },            // 1NNN
    CALL { nnn: u12 },            // 2NNN
    SKE { x: u4, nn: u8 },        // 3XNN
    SKNE { x: u4, nn: u8 },       // 4XNN
    SKRE { x: u4, y: u4 },        // 5XY0
    LOAD { x: u4, nn: u8 },       // 6XNN
    ADD { x: u4, nn: u8 },        // 7XNN
    MOVE { x: u4, y: u4 },        // 8XY0
    OR { x: u4, y: u4 },          // 8XY1
    AND { x: u4, y: u4 },         // 8XY2
    XOR { x: u4, y: u4 },         // 8XY3
    ADDR { x: u4, y: u4 },        // 8XY4
    SUB { x: u4, y: u4 },         // 8XY5
    SHR { x: u4, y: u4 },         // 8XY6
    SUBN { x: u4, y: u4 },        // 8XY7
    SHL { x: u4, y: u4 },         // 8XYE
    SNE { x: u4, y: u4 },         // 9XY0
    LOADI { nnn: u12 },           // ANNN
    JUMPI { nnn: u12 },           // BNNN
    RAND(u16, u16),               // CXNN
    DRAW { x: u4, y: u4, n: u4 }, // DXYN
    SKPR(u16),                    // EX9E
    SKUP(u16),                    // EXA1
    MOVED(u16),                   // FX07
    KEYD(u16),                    // FX0A
    LOADD(u16),                   // FX15
    LOADS(u16),                   // FX18
    ADDI { x: u4 },               // FX1E
    LDSPR { x: u4 },              // FX29
    BCD(u16),                     // FX33
    STOR(u16),                    // FX55
    READ(u16),                    // FX65
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
            (0, 0, 0xE, 0xE) => OpCode::RTS,
            (0, 0, 0xE, 0) => OpCode::CLR,
            (0xD, _, _, _) => OpCode::DRAW { x, y, n },
            (2, _, _, _) => OpCode::CALL { nnn },
            (0, _, _, _) => OpCode::SYS { nnn },
            (1, _, _, _) => OpCode::JUMP { nnn },
            (3, _, _, _) => OpCode::SKE { x, nn },
            (4, _, _, _) => OpCode::SKNE { x, nn },
            (5, _, _, 0) => OpCode::SKRE { x, y },
            (9, _, _, 0) => OpCode::SNE { x, y },
            (6, _, _, _) => OpCode::LOAD { x, nn },
            (7, _, _, _) => OpCode::ADD { x, nn },
            (8, _, _, 0) => OpCode::MOVE { x, y },
            (0xA, _, _, _) => OpCode::LOADI { nnn },
            (0xB, _, _, _) => OpCode::JUMPI { nnn },
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
            (0x00E0, OpCode::Clear),
            (0xD123, OpCode::Draw { x: 1, y: 2, n: 3 }),
            (0x00EE, OpCode::Return),
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

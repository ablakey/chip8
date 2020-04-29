/// Type aliases to make the code more legible. We aren't going to support nibbles and
/// triple-nibbles... tribbles? Hah! I tried to with a `ux` crate but the ergonomics were
/// unpleasant. They couldn't interact with the built-in primitives so easily, if I recall. I
/// am pretty sure that u4 and u12 not actually being those sizes will be fine, so long as we
/// perform bitwise math on them carefully. The most significant nibbles will just be 0.
/// Rust won't type-check these though so I could pass a u4 where I meant to pass a u8.
#[allow(non_camel_case_types)]
pub type u4 = u8;
#[allow(non_camel_case_types)]
pub type u12 = u16;

/// OpCode enumerates all possible opcodes. Each variant is a tuple of 0-3 elements depending on
/// The opcode's pattern. The OpCode
#[derive(Debug, PartialEq)]
pub enum OpCode {
    // Flow
    SysAddr(u12),    // 0NNN
    Return,          // 00EE
    CallAddr(u12),   // 2NNN
    GotoAddrV0(u12), // BNNN

    // Display
    Clear,            // 00E0
    Draw(u4, u4, u4), // DXYN

    // Conditional
    SkipEq(u4, u8), // 3XNN
    // SkipNEqVx(u16, u16),   // 4XNN
    // SkipVxVy(u16, u16),    // 5XY0
    // SkipNEqVxVy(u16, u16), // 9XY0

    // // Get/Set
    SetVx(u4, u8), // 6XNN
                   // AddToVx(u16, u16),  // 7XNN
                   // CopyVxVy(u16, u16), // 8XY0
                   // SetI(u16),          // ANNN
                   // AddVxI(u16),        // FX1E
                   // SetBCD(u16),        // FX33
                   // DumpReg(u16),       // FX55
                   // LoadReg(u16),       // FX65

                   // // Math
                   // AddVxVy(u16, u16), // 8XY4
                   // SubVxVy(u16, u16), // 8XY5
                   // SubVyVx(u16, u16), // 8XY7
                   // RandVx(u16, u16),  // CXNN

                   // // Bitwise
                   // OrVxVy(u16, u16),   // 8XY1
                   // AndVxVy(u16, u16),  // 8XY2
                   // XorVxVy(u16, u16),  // 8XY3
                   // LSBShift(u16, u16), // 8XY6
                   // MSBShift(u16, u16), // 8XYE

                   // // Input
                   // KeyDown(u16),  // EX9E
                   // KeyUp(u16),    // EXA1
                   // AwaitKey(u16), // FX0A

                   // // Delay
                   // AddDelay(u16), // FX07
                   // SetDelay(u16), // FX15

                   // // Sound
                   // SetSound(u16), // FX18
}

impl OpCode {
    fn get_nn(opcode: u16) -> u8 {
        opcode.to_be_bytes()[1]
    }
    fn get_n(opcode: u16) -> u4 {
        return opcode.to_be_bytes()[1] & 0x0F; // Mask the high byte.
    }

    /// Return the 12-bit address represented by the last 3 nibbles in the opcode.
    fn get_nnn(opcode: u16) -> u12 {
        return opcode & 0x0FFF;
    }

    pub fn from_word(opcode: u16) -> Self {
        // Mask each nibble in the opcode and shift to isolate them.
        let a: u4 = ((opcode & 0xF000) >> 12).to_be_bytes()[1];
        let x: u4 = ((opcode & 0x0F00) >> 8).to_be_bytes()[1];
        let y: u4 = ((opcode & 0x00F0) >> 4).to_be_bytes()[1];
        let n: u4 = (opcode & 0x000F).to_be_bytes()[1];

        println!("{:#X} {:#X} {:#X} {:#X} ", a, x, y, n);

        // The order of these match branches are important as some opcodes are more specific than others.
        let opcode = match (a, x, y, n) {
            (0, 0, 0xE, 0xE) => OpCode::Return,
            (0, 0, 0xE, 0) => OpCode::Clear,
            (0xD, _, _, _) => OpCode::Draw(x, y, n),
            (2, _, _, _) => OpCode::CallAddr(OpCode::get_nnn(opcode)),
            (0, _, _, _) => OpCode::SysAddr(OpCode::get_nnn(opcode)),
            (0xB, _, _, _) => OpCode::GotoAddrV0(OpCode::get_nnn(opcode)),
            (3, _, _, _) => OpCode::SkipEq(x, OpCode::get_nn(opcode)),
            // (6, _,_,_)) => OpCode::SetVx()
            // TODO: implement the rest.
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
            (0xD123, OpCode::Draw(1, 2, 3)),
            (0x00EE, OpCode::Return),
        ];

        for (input, opcode) in opcode_tests.iter() {
            assert!(OpCode::from_word(*input) == *opcode)
        }
    }

    #[test]
    fn test_get_nnn() {
        assert_eq!(OpCode::get_nnn(0x0300), 0x300);
        assert_eq!(OpCode::get_nnn(0x1111), 0x111);
        assert_eq!(OpCode::get_nnn(0xABCD), 0xBCD);
    }

    #[test]
    fn test_get_nn() {
        assert_eq!(OpCode::get_nn(0x0300), 0x00);
        assert_eq!(OpCode::get_nn(0x1111), 0x11);
        assert_eq!(OpCode::get_nn(0xABCD), 0xCD);
    }
}

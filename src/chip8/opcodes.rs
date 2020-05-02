use super::Chip8;

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

fn RTS() {}
fn SYS(nnn: u12) {}
fn JUMP(nnn: u12) {}
fn CALL(nnn: u12) {}
fn SKE(x: u4, nn: u8) {}
fn SKNE(x: u4, nn: u8) {}
fn SKRE(x: u4, y: u4) {}
fn LOAD(x: u4, nn: u8) {}
fn ADD(x: u4, nn: u8) {}
fn MOVE(x: u4, y: u4) {}
fn OR(x: u4, y: u4) {}
fn AND(x: u4, y: u4) {}
fn XOR(x: u4, y: u4) {}
fn ADDR(x: u4, y: u4) {}
fn SUB(x: u4, y: u4) {}
fn SHR(x: u4, y: u4) {}
fn SUBN(x: u4, y: u4) {}
fn SHL(x: u4, y: u4) {}
fn SKRNE(x: u4, y: u4) {}
fn LOADI(nnn: u12) {}
fn JUMPI(nnn: u12) {}
fn RAND(x: u4, nn: u8) {}
fn DRAW(x: u4, y: u4, n: u4) {}
fn SKPR(x: u4) {}
fn SKUP(x: u4) {}
fn MOVED(x: u4) {}
fn KEYD(x: u4) {}
fn LOADD(x: u4) {}
fn LOADS(x: u4) {}
fn ADDI(x: u4) {}
fn LDSPR(x: u4) {}
fn BCD(x: u4) {}
fn STOR(x: u4) {}
fn READ(x: u4) {}

impl Chip8 {
    pub fn execute_opcode(&self, opcode: u16) {
        #[rustfmt::skip]
    // These are possible opcode symbols, not all of which are valid. Depending on the matched
    // opcode, some of the symbols may be used.
    let OpCodeSymbols { a, x, y, n, nnn, nn } = OpCodeSymbols::from_value(opcode);

        // The order of these match branches are important.
        // Some opcodes are more specific than others.
        match (a, x, y, n) {
            (0, 0, 0xE, 0) => self.CLR(),
            (0, 0, 0xE, 0xE) => RTS(),
            (0, _, _, _) => SYS(nnn),
            (1, _, _, _) => JUMP(nnn),
            (2, _, _, _) => CALL(nnn),
            (3, _, _, _) => SKE(x, nn),
            (4, _, _, _) => SKNE(x, nn),
            (5, _, _, 0) => SKRE(x, y),
            (6, _, _, _) => LOAD(x, nn),
            (7, _, _, _) => ADD(x, nn),
            (8, _, _, 0) => MOVE(x, y),
            (8, _, _, 1) => OR(x, y),
            (8, _, _, 2) => AND(x, y),
            (8, _, _, 3) => XOR(x, y),
            (8, _, _, 4) => ADDR(x, y),
            (8, _, _, 5) => SUB(x, y),
            (8, _, _, 6) => SHR(x, y),
            (8, _, _, 7) => SUBN(x, y),
            (8, _, _, 0xE) => SHL(x, y),
            (9, _, _, 0) => SKRNE(x, y),
            (0xA, _, _, _) => LOADI(nnn),
            (0xB, _, _, _) => JUMPI(nnn),
            (0xC, _, _, _) => RAND(x, nn),
            (0xD, _, _, _) => DRAW(x, y, n),
            (0xE, _, 9, 0xE) => SKPR(x),
            (0xE, _, 0xA, 1) => SKUP(x),
            (0xF, _, 0, 7) => MOVED(x),
            (0xF, _, 0, 0xA) => KEYD(x),
            (0xF, _, 1, 5) => LOADD(x),
            (0xF, _, 1, 8) => LOADS(x),
            (0xF, _, 1, 0xE) => ADDI(x),
            (0xF, _, 2, 9) => LDSPR(x),
            (0xF, _, 3, 3) => BCD(x),
            (0xF, _, 5, 5) => STOR(x),
            (0xF, _, 6, 5) => READ(x),
            (_, _, _, _) => panic!("Tried to call opcode {:X?} that is not handled.", opcode),
        };
    }

    fn CLR(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

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

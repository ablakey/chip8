use crate::opcode::OpCode;
use std::fmt::UpperHex;
use std::mem::align_of;

// Return a byte represented by the last 2 nibbles in the opcode.
fn get_byte(opcode: u16) -> u16 {
    return opcode & 0x00FF;
}

// Return the 12-bit address represented by the last 3 nibbles in the opcode.
fn get_address(opcode: u16) -> u16 {
    return opcode & 0x0FFF;
}

pub fn hex<T: UpperHex>(e: T) -> String {
    format!("{:#01$X}", e, align_of::<T>() * 2)
}

macro_rules! print_all {
    ($($args:expr),*) => {{
        let mut v = Vec::new();
        $(
            // let s = format!("{}, {}", s, hex($args).as_str());
            v.push(hex($args));
        )*
        println!("{}", v.join(", "));
    }}
}

pub fn match_opcode(opcode: u16) -> OpCode {
    // Mask each nibble in the opcode and shift to isolate them.
    let a = (opcode & 0xF000) >> 12;
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let n = opcode & 0x000F;

    print_all!(x, y, n);

    // The order of these match branches are important as some opcodes are more specific than others.
    let opcode = match (a, x, y, n) {
        (0, 0, 0xE, 0xE) => OpCode::Return,
        (0, 0, 0xE, 0) => OpCode::Clear,
        (0xD, _, _, _) => OpCode::Draw(x, y, n),
        (2, 0, 0, 0) => OpCode::CallAddr(get_address(opcode)),
        (0, _, _, _) => OpCode::SysAddr(get_address(opcode)),
        (0xB, _, _, _) => OpCode::GotoAddrV0(get_address(opcode)),
        (3, _, _, _) => OpCode::SkipEq(x, get_byte(opcode)),
        // TODO: implement the rest.
        (_, _, _, _) => panic!("Tried to call opcode {:X?} that is not handled.", opcode),
    };

    return opcode;
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
            assert!(match_opcode(*input) == *opcode)
        }
    }

    #[test]
    fn test_get_address() {
        assert_eq!(get_address(0x0300), 0x300);
        assert_eq!(get_address(0x1111), 0x111);
        assert_eq!(get_address(0xABCD), 0xBCD);
    }

    #[test]
    fn test_get_byte() {
        assert_eq!(get_byte(0x0300), 0x00);
        assert_eq!(get_byte(0x1111), 0x11);
        assert_eq!(get_byte(0xABCD), 0xCD);
    }
}

mod chip8;
mod opcode;
mod utils;

fn main() {
    // Load a program.

    let mut machine = chip8::Chip8::init();
    machine
        .load_cartridge(String::from("roms/maze.c8"))
        .unwrap();

    let x = opcode::OpCode::SysAddr(15);
    // println!("{:?}", x);
    // println!("{:?}", machine.memory[0]);

    let y = utils::match_opcode(0x00EE);
    // println!("{}", utils::hex(0xFF11u16));
}

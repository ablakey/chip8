mod chip8;
mod opcode;

fn main() {
    // Load a program.

    let mut machine = chip8::Chip8::init();
    machine.load_rom(String::from("roms/maze.c8")).unwrap();
    machine.print_debug();
    machine.print_mem();
    machine.tick();
}

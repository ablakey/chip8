mod chip8;
use chip8::Chip8;

fn main() {
    // Load a program.

    let mut machine = Chip8::init();
    machine.load_rom(String::from("roms/maze.c8")).unwrap();
    machine.print_debug();
    machine.print_mem();
    machine.tick();
}

mod chip8;
mod emulator;
mod input;
mod screen;
use emulator::Emulator;
use input::{Input, InputEvent};
use screen::Screen;

use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Debugger.
    let mut paused = true;

    // Init I/O components
    let sdl_context = sdl2::init()?;
    let mut input = Input::init(&sdl_context)?;
    let mut screen = Screen::create(&sdl_context, 30)?;

    // Init the emulated machine.
    let mut emulator = Emulator::init().unwrap();
    emulator.load_rom(String::from("roms/maze.c8"));

    'program: loop {
        // Handle emulator I/O (the inputs not destined for the Chip8).
        match input.get_event() {
            InputEvent::Exit => break 'program,
            InputEvent::ToggleRun => paused = !paused,
            InputEvent::Tick => {
                emulator.tick();
            }
            _ => (),
        }

        if !paused {
            // Write key states to emulator's memory.
            emulator.set_keys(input.get_chip8_keys());

            // Advance the emulator one tick.
            emulator.tick();
        }

        // Draw screen.
        if emulator.has_graphics_update() {
            screen.draw(emulator.get_screen_buffer());
        }

        // Sleep this hot loop at the same rate as the CPU (the most frequent thing).
        // In a more accurate emulator, there's better ways to handle this, given it's possible
        // that we sleep too long and never tick the CPU regularly enough.  We would also have to
        // handle "speeding up" to catch up with the expected frequency in that case.
        // 2000 Microseconds -> Milliseconds.
        sleep(Duration::new(0, (2000 * 1000) as u32))
    }

    Ok(())
}

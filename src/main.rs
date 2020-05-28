mod chip8;
mod debugger;
mod emulator;
mod input;
mod screen;
use debugger::Debugger;
use emulator::Emulator;
use input::{Input, InputEvent};
use screen::Screen;
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), String> {
    // Get arguments.
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // Init debugger.
    let debugger = Debugger::init();

    // Init I/O components.
    let sdl_context = sdl2::init()?;
    let mut input = Input::init(&sdl_context)?;
    let mut screen = Screen::create(&sdl_context, 30)?;

    // Init the emulated machine.
    let mut emu = Emulator::init().unwrap();

    emu.state.load_rom(filename).unwrap();

    // Print out initial debug state.
    debugger.write(emu.state.format_memory());
    debugger.write(emu.state.format_debug());

    'program: loop {
        // Handle emu I/O (the inputs not destined for the Chip8).
        match input.get_event() {
            InputEvent::Exit => break 'program,
            InputEvent::ToggleRun => emu.paused = !emu.paused,
            InputEvent::Tick => {
                emu.tick();
                debugger.overwrite(emu.state.format_debug());
            }
            _ => (),
        }

        if !emu.paused {
            // Write key states to emu's memory.
            emu.state.set_keys(input.get_chip8_keys());

            // Advance the emu one tick.
            emu.tick();
        }

        // Draw screen.
        if emu.state.has_graphics_update {
            screen.draw(&emu.state.graphics_buffer);
        }

        // Sleep at a rate that emulates about 500Hz. This won't be accurate.
        sleep(Duration::new(0, (2000 * 1000) as u32))
    }

    Ok(())
}

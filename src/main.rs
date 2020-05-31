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
use std::io::BufReader;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn play_sound(device: &rodio::Device) -> rodio::Sink {
    println!("Started beep1");

    let file = std::fs::File::open("./src/beep.wav").unwrap();
    let beep1 = rodio::play_once(device, BufReader::new(file)).unwrap();
    beep1.set_volume(0.2);
    println!("Started beep1");
    beep1
}

fn main() -> Result<(), String> {
    // let device = rodio::default_output_device().unwrap();
    // let _ = play_sound(&device);

    // let device = rodio::default_output_device().unwrap();
    // let file = std::fs::File::open("./src/beep.wav").unwrap();
    // let beep1 = rodio::play_once(&device, BufReader::new(file)).unwrap();
    // beep1.set_volume(0.2);
    // println!("Started beep1");

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
            InputEvent::SaveState => emu.save_state(),
            InputEvent::RestoreState => {
                emu.restore_state();
                screen.draw(&emu.state.graphics_buffer);
                debugger.overwrite(emu.state.format_debug());
            }
            InputEvent::Tick => {
                emu.state.tick();
                debugger.overwrite(emu.state.format_debug());
            }
            _ => (),
        }

        if !emu.paused {
            // Write key states to emu's memory.
            // TODO: maybe don't run this at 500hz.
            emu.state.set_keys(input.get_chip8_keys());

            // Advance the emu one tick.
            emu.state.tick();
            // debugger.overwrite(emu.state.format_debug());
        }

        // Draw screen.
        if emu.state.has_graphics_update {
            screen.draw(&emu.state.graphics_buffer);
        }

        // Sleep at a rate that emulates about 500Hz. This won't be accurate.
        sleep(Duration::new(0, 2_000_000 as u32))
    }

    Ok(())
}

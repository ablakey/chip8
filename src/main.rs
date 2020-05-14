mod chip8;
mod input;
mod screen;
use chip8::Chip8;
use input::{Input, InputEvent};
use screen::Screen;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

fn main() -> Result<(), String> {
    // Init I/O components
    let sdl_context = sdl2::init()?;
    let mut input = Input::init(&sdl_context)?;
    let mut screen = Screen::create(&sdl_context, 30)?;

    // Init "system clock".
    let clock = SystemTime::now();
    let mut last_cpu_tick: u128 = 0;
    let mut last_timer_tick: u128 = 0;

    // Load a program.
    let mut c8 = Chip8::init();
    c8.load_rom(String::from("roms/maze.c8")).unwrap();

    // Debug flags.
    let mut paused = false;

    c8.print_mem();

    // Loop controls the application, including debug tools.
    // It ticks at a very high frequency to more accurately count delta time between ticks.
    // The CPU runs at 500hz while the timers run at 60Hz.
    // The screen is drawn on any opcode that has changed the graphics buffer.
    'program: loop {
        // Handle clock rate.
        let now = clock.elapsed().unwrap().as_micros();

        // Handle emulator I/O (the inputs not destined for the Chip8).
        match input.get_event() {
            InputEvent::Exit => break 'program,
            InputEvent::ToggleRun => paused = !paused,
            _ => (),
        }

        // Do not run the machine if the emulator has paused it.
        if !paused {
            // Write keyboard state from I/O to emulator memory.
            c8.set_keys(input.get_chip8_keys());

            // CPU tick?
            if now - last_cpu_tick > Chip8::CPU_FREQUENCY as u128 && !c8.wait_for_input {
                c8.tick();
                last_cpu_tick = now;
            }

            // timer tick?
            if now - last_timer_tick > Chip8::TIMER_FREQUENCY as u128 {
                c8.decrement_timers();
                last_timer_tick = now;
            }

            // Draw to screen?
            if c8.has_graphics_update {
                screen.draw(&c8.graphics_buffer);
            }
        }

        // Sleep this hot loop at the same rate as the CPU (the most frequent thing).
        // In a more accurate emulator, there's better ways to handle this, given it's possible
        // that we sleep too long and never tick the CPU regularly enough.  We would also have to
        // handle "speeding up" to catch up with the expected frequency in that case.
        sleep(Duration::new(0, (Chip8::CPU_FREQUENCY * 1000) as u32))
    }

    Ok(())
}

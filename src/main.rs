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
    c8.load_rom(String::from("roms/particle_demo.c8")).unwrap();

    // Debug flags.
    let mut is_running = true;

    c8.print_mem();

    // Loop controls the application, including debug tools.
    // It ticks at a very high frequency to more accurately count delta time between ticks.
    // The CPU runs at 500hz while the timers run at 60Hz.
    // The screen is drawn on any opcode that has changed the graphics buffer.
    'program: loop {
        // Handle clock rate.
        let now = clock.elapsed().unwrap().as_micros();

        if now - last_cpu_tick > Chip8::CPU_FREQUENCY && is_running {
            c8.tick();
            last_cpu_tick = now;
        }

        if now - last_timer_tick > Chip8::TIMER_FREQUENCY && is_running {
            c8.decrement_timers();
            last_timer_tick = now;
        }

        // Handle I/O.
        match input.get_event() {
            InputEvent::Exit => break 'program,
            InputEvent::ToggleRun => is_running = !is_running,
            _ => (),
        }

        // Draw to screen?
        if c8.has_graphics_update {
            screen.draw(&c8.graphics_buffer);
        }

        // Cool the hot loop down, but always be faster than CPU_FREQUENCY to avoid runnin not
        // often enough. We loop twice as fast to attempt this, but ultimately it's up to the host
        // machine to yield back often enough. This is kind of sloppy, but the difference is between
        // 100% of a core and 2%, so it beats nothing.
        sleep(Duration::new(0, (Chip8::CPU_FREQUENCY * 1000 / 2) as u32))
    }

    Ok(())
}

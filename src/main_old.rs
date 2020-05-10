mod chip8;
mod graphics;
use chip8::Chip8;
use graphics::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{thread, time};

fn main() -> Result<(), String> {
    // Init SDL components (the CHIP8 I/O)
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut screen = Screen::create(&sdl_context, 30)?;

    // Load a program.
    let mut c8 = Chip8::init();
    c8.load_rom(String::from("roms/particle_demo.c8")).unwrap();

    c8.print_mem();

    'running: loop {
        // Advance the program at about 500hz (8.33 ticks per cycle.)
        // Note: we are rounding to 8 for simplicity, meaning the emulator runs a bit slow.
        for _ in 0..8 {
            c8.tick();
        }

        c8.decrement_timers();

        // Draw to screen?
        if c8.has_graphics_update {
            screen.draw(&c8.graphics_buffer);
        }

        // Handle keyboard/event input.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        // Loop at ~ 60hz.
        // Note: this speeds the emulator up (16ms vs. 16.66ms). It also means we sleep for  at
        // least 16ms but could be more. We should implement something more robust.
        thread::sleep(time::Duration::from_millis(16));
    }

    Ok(())
}

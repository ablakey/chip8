extern crate sdl2;
mod chip8;
mod graphics;
use chip8::Chip8;
use graphics::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() -> Result<(), String> {
    // Init SDL components (the CHIP8 I/O)
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut screen = Screen::create(&sdl_context, 30)?;

    // Load a program.
    let mut machine = Chip8::init();
    machine.load_rom(String::from("roms/maze.c8")).unwrap();
    machine.print_debug();
    machine.print_mem();

    'running: loop {
        // Advance the program a tick.
        machine.tick();

        // Draw to screen?
        if machine.draw_graphics {
            screen.blit(&machine.graphics_buffer)
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

        //Throttle the loop rate.
    }

    Ok(())
}

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

#[derive(PartialEq)]
pub enum InputEvent {
    None,
    Exit,
    ToggleRun,
    Tick,
    SaveState,
    RestoreState,
}

pub struct Input {
    event_pump: EventPump,
}

impl Input {
    // Binding each of the Chip8's keys to a real keyboard key. Chip8 has 16 keys: 0-F. Each is an
    // index in this array. See get_chip8_keys for details.
    const KEY_BINDINGS: [Scancode; 16] = [
        Scancode::X,
        Scancode::Num1,
        Scancode::Num2,
        Scancode::Num3,
        Scancode::Q,
        Scancode::W,
        Scancode::E,
        Scancode::A,
        Scancode::S,
        Scancode::D,
        Scancode::Z,
        Scancode::C,
        Scancode::Num4,
        Scancode::R,
        Scancode::F,
        Scancode::V,
    ];

    pub fn init(context: &sdl2::Sdl) -> Result<Self, String> {
        let event_pump = context.event_pump()?;

        Ok(Self { event_pump })
    }

    /// Return a single, highest priority event.
    /// This may be a call to quit the application, change a debug setting, or supply keyboard
    /// state to the emulator.
    pub fn get_event(&mut self) -> InputEvent {
        let mut x = InputEvent::None;

        for event in self.event_pump.poll_iter() {
            x = match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => InputEvent::Exit,
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => InputEvent::ToggleRun,
                Event::KeyUp {
                    keycode: Some(Keycode::F5),
                    ..
                } => InputEvent::SaveState,
                Event::KeyUp {
                    keycode: Some(Keycode::F9),
                    ..
                } => InputEvent::RestoreState,
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => InputEvent::Tick,
                Event::KeyDown { .. } => InputEvent::None,
                _ => InputEvent::None,
            };

            if x != InputEvent::None {
                break;
            }
        }

        return x;
    }

    /// Get The state of the 16 input keys Chip8 has.
    /// These keys are 0-F (but are typically laid out in a grid pattern). The returned array
    /// Represents the state of each key: 0-F
    ///The typical CHIP8 controller looks like the diagram below.
    /// The 2,4,6,8 are typically used as arrows.
    /// ```
    /// ╔═══╦═══╦═══╦═══╗
    /// ║ 1 ║ 2 ║ 3 ║ C ║
    /// ╠═══╬═══╬═══╬═══╣
    /// ║ 4 ║ 5 ║ 6 ║ D ║
    /// ╠═══╬═══╬═══╬═══╣
    /// ║ 7 ║ 8 ║ 9 ║ E ║
    /// ╠═══╬═══╬═══╬═══╣
    /// ║ A ║ 0 ║ B ║ F ║
    /// ╚═══╩═══╩═══╩═══╝
    /// ```
    pub fn get_chip8_keys(&mut self) -> [bool; 16] {
        let keys: Vec<Scancode> = self
            .event_pump
            .keyboard_state()
            .pressed_scancodes()
            .collect();

        // Hard coded binding of keyboard to keys.  We use the left 16 keys in the same grid pattern
        // which means none of the letters/numbers align, but the shape does.
        let key_states = Self::KEY_BINDINGS
            .iter()
            .map(|b| keys.contains(b))
            .collect::<Vec<bool>>();

        let mut foo = [false; 16];
        foo.copy_from_slice(&key_states[..]);
        return foo;
    }
}

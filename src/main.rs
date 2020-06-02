mod audio;
mod chip8;
mod input;
use audio::Audio;
mod screen;
use chip8::Chip8;
use console::Term;
use input::{Input, InputEvent};
use screen::Screen;
use std::env;
use std::thread::sleep;
use std::time::Duration;

struct Emulator {
    debugger: Debugger,
    input: Input,
    screen: Screen,
    audio: Audio,
    state: Chip8,
    saved_state: Option<Chip8>,
    is_paused: bool,
}

impl Emulator {
    const SCREEN_ZOOM: u32 = 20; // Multiple to zoom screen by.
    const TONE: u32 = 440; // Pitch for beep sound.

    fn init(path: &String) -> Result<Self, String> {
        // CLI debugging.
        let debugger = Debugger::init();

        // SDL-based I/O.
        let sdl_context = sdl2::init()?;
        let input = Input::init(&sdl_context)?;
        let screen = Screen::create(&sdl_context, Emulator::SCREEN_ZOOM)?;
        let audio = Audio::init(Emulator::TONE);

        // The emulated Chip8 state. This includes memory, registers, counters, timers, etc.
        let mut state = Chip8::init();
        state.load_rom(path).unwrap();

        debugger.write(state.dum_loaded_rom());

        Ok(Self {
            debugger,
            input,
            screen,
            state,
            audio,
            saved_state: None,
            is_paused: true,
        })
    }

    fn save_state(&mut self) {
        self.saved_state = Some(self.state.clone());
    }

    fn restore_state(&mut self) {
        match &self.saved_state {
            Some(s) => self.state = s.clone(),
            None => (),
        }
    }

    /// Loop forever at 500Hz.
    /// Handles input, ticks the Chip8 CPU, draws graphics and plays audio.
    pub fn run_forever(&mut self) {
        'program: loop {
            // Emulator and Chip8 I/O.
            match self.input.get_event() {
                InputEvent::Exit => break 'program,
                InputEvent::ToggleRun => self.is_paused = !self.is_paused,
                InputEvent::SaveState => self.save_state(),
                InputEvent::RestoreState => {
                    self.restore_state();
                    self.screen.draw(&self.state.graphics_buffer);
                    self.debugger.overwrite(self.state.dump_state());
                }
                InputEvent::Tick => {
                    self.state.tick();
                    self.debugger.overwrite(self.state.dump_state());
                }
                _ => (),
            }

            if !self.is_paused {
                self.state.set_keys(self.input.get_chip8_keys());
                self.state.tick();
                // debugger.overwrite(self.state.dump_state());
            }

            if self.state.has_graphics_update {
                self.screen.draw(&self.state.graphics_buffer);
            }

            if self.state.sound_timer > 0 && self.audio.is_paused() {
                self.audio.play();
            } else if self.state.sound_timer == 0 && !self.audio.is_paused() {
                self.audio.stop();
            }

            // Sleep at a rate that emulates about 500Hz. This won't be accurate.
            sleep(Duration::new(0, 2_000_000 as u32))
        }
    }
}

struct Debugger {
    terminal: Term,
}

impl Debugger {
    pub fn init() -> Self {
        let terminal = Term::stdout();
        Self { terminal }
    }

    pub fn write(&self, string: String) {
        self.terminal.write_line(string.as_str()).unwrap();
    }

    pub fn overwrite(&self, string: String) {
        let count = string.lines().count();
        self.terminal.clear_last_lines(count + 1).unwrap();
        self.write(string);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let filename = &args[1];

    let emulator = Emulator::init(filename);

    match emulator {
        Ok(mut e) => e.run_forever(),
        Err(e) => panic!("Could not launch emulator. Reason: {}", e),
    }
}

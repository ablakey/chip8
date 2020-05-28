use super::chip8::Chip8;

pub struct Emulator {
    // saved_state: Option<Chip8>,
    pub state: Chip8,
    cycle: usize,
    pub paused: bool,
}

impl Emulator {
    pub fn init() -> Result<Self, String> {
        Ok(Self {
            // saved_state: None,
            state: Chip8::init(),
            cycle: 0,
            paused: true,
        })
    }

    /// Emulate one tick of the application.  A Chip8 runs at about 500Hz (the timers are 60Hz)
    /// This isn't perfectly accurate but that is 1 Opcode per tick and decrementing timers once
    /// every 8 ticks.
    pub fn tick(&mut self) {
        self.cycle += 1;

        // Every tick, process 1 opcode.
        self.state.execute_opcode();

        // Every 8th tick, decrement timers.
        if self.cycle % 8 == 0 {
            self.state.decrement_timers();
        }

        // self.state.print_debug();
    }
}

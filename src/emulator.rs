use super::chip8::Chip8;

pub struct Emulator {
    states: Vec<Chip8>,
    tick_count: usize,
}

impl Emulator {
    const MAX_STATES: usize = 1000;

    pub fn init() -> Result<Self, String> {
        Ok(Self {
            states: Vec::with_capacity(Emulator::MAX_STATES * 2),
            tick_count: 0,
        })
    }

    pub fn load_rom(&mut self, path: String) {
        let mut c8 = Chip8::init();
        c8.load_rom(path).unwrap();
        self.save_state(c8);
    }

    /// Gets a clone of the latest state, updates the keys, and applies it to the stack of states.
    pub fn set_keys(&mut self, keys: [bool; 16]) {
        let mut s = self.get_state_clone();
        s.set_keys(keys);
        self.save_state(s);
    }

    /// Emulate one tick of the application.  A Chip8 runs at about 500Hz (the timers are 60Hz)
    /// This isn't perfectly accurate but that is 1 Opcode per tick and decrementing timers once
    /// every 8 ticks.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        let mut s = self.get_state_clone();

        // Every tick, process 1 opcode.
        s.execute_opcode();

        // Every 8th tick, decrement timers.
        if self.tick_count % 8 == 0 {
            s.decrement_timers();
        }

        self.save_state(s);
    }

    /// Return the current state's screen buffer.
    pub fn get_screen_buffer(&self) -> &[bool; 64 * 32] {
        return &self.states.last().unwrap().graphics_buffer;
    }

    pub fn has_graphics_update(&self) -> bool {
        return self.states.last().unwrap().has_graphics_update;
    }

    fn get_state_clone(&self) -> Chip8 {
        return self.states.last().unwrap().clone();
    }

    fn save_state(&mut self, c8: Chip8) {
        // Garbage collect older states.
        // TODO: A ring buffer makes a lot more sense. Let's use one rather than all this memory
        // allocation and cleanup.
        if self.states.len() > Emulator::MAX_STATES {
            self.states = Vec::from(&self.states[1000..]);
        }

        self.states.push(c8);
    }
}

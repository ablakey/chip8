use super::chip8::Chip8;

pub struct Emulator {
    saved_state: Option<Chip8>,
    pub state: Chip8,
    pub paused: bool,
}

impl Emulator {
    pub fn init() -> Result<Self, String> {
        Ok(Self {
            saved_state: None,
            state: Chip8::init(),
            paused: true,
        })
    }

    pub fn save_state(&mut self) {
        self.saved_state = Some(self.state.clone());
    }

    pub fn restore_state(&mut self) {
        match &self.saved_state {
            Some(s) => self.state = s.clone(),
            None => (),
        }
    }
}

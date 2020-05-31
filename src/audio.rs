use rodio::{source::SineWave, Device, Sink};

pub struct Audio {
    sink: Sink,
    _device: Device, // Needs to be held but not used.
}

impl Audio {
    pub fn init(freq: u32) -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);

        // Add a dummy source of the sake of the example.
        let source = SineWave::new(freq);
        sink.append(source);
        sink.pause(); // Start without playing.

        Self {
            sink: sink,
            _device: device,
        }
    }

    /// Begin playing the sinewave tone.
    pub fn play(&mut self) {
        self.sink.play();
    }

    /// Stop playing the sinewave tone.
    pub fn stop(&mut self) {
        self.sink.pause();
    }

    /// Is the tone currently paused?
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}

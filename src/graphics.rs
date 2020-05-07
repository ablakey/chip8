pub struct Screen {
    sdl_canvas: sdl2::render::Canvas<sdl2::video::Window>,
    scale_factor: u32,
}

impl Screen {
    const CHIP8_WIDTH: u32 = 64;
    const CHIP8_HEIGHT: u32 = 32;
    const BG_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);
    const PIXEL_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 255, 255);

    pub fn create(context: &sdl2::Sdl, scale_factor: u32) -> Result<Self, String> {
        let video_subsys = context.video()?;

        let window = video_subsys
            .window(
                "title: CHIP8",
                Self::CHIP8_WIDTH * scale_factor,
                Self::CHIP8_HEIGHT * scale_factor,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let c = window.into_canvas().build().map_err(|e| e.to_string())?;

        let mut f = Self {
            sdl_canvas: c,
            scale_factor,
        };
        f.sdl_canvas.set_draw_color(Self::BG_COLOR);

        return Ok(f);
    }

    /// Iterate through all pixels in buffer and draw only those that are set active.
    /// The screen is first blanked, then all pixels in buffer are evaluated for being active.
    /// The remaining pixels are drawn as filled rects, scaled by scale_factor.
    pub fn draw(&mut self, &buffer: &[bool; 64 * 32]) {
        let rects: Vec<sdl2::rect::Rect> = buffer
            .iter()
            .enumerate()
            .filter(|(_, &x)| x)
            .map(|(n, _)| {
                // Row-major, so we divide and modulo by width to get row and column number.
                let row = n / Self::CHIP8_WIDTH as usize;
                let col = n % Self::CHIP8_WIDTH as usize;

                return sdl2::rect::Rect::new(
                    (col * self.scale_factor as usize) as i32,
                    (row * self.scale_factor as usize) as i32,
                    self.scale_factor,
                    self.scale_factor,
                );
            })
            .collect();

        self.sdl_canvas.set_draw_color(Self::PIXEL_COLOR);
        self.sdl_canvas.fill_rects(&rects).unwrap();
        self.sdl_canvas.present();
    }
}

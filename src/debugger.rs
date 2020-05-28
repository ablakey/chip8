use console::Term;

pub struct Debugger {
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

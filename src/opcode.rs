#[derive(Debug, PartialEq)]
pub enum OpCode {
    // Flow
    SysAddr(u16),    // 0NNN
    Return,          // 00EE
    CallAddr(u16),   // 2NNN
    GotoAddrV0(u16), // BNNN

    // Display
    Clear,               // 00E0
    Draw(u16, u16, u16), // DXYN

    // Conditional
    SkipEq(u16, u16), // 3XNN
                      // SkipNEqVx(u16, u16),   // 4XNN
                      // SkipVxVy(u16, u16),    // 5XY0
                      // SkipNEqVxVy(u16, u16), // 9XY0

                      // // Get/Set
                      // SetVx(u16, u16),    // 6XNN
                      // AddToVx(u16, u16),  // 7XNN
                      // CopyVxVy(u16, u16), // 8XY0
                      // SetI(u16),          // ANNN
                      // AddVxI(u16),        // FX1E
                      // SetBCD(u16),        // FX33
                      // DumpReg(u16),       // FX55
                      // LoadReg(u16),       // FX65

                      // // Math
                      // AddVxVy(u16, u16), // 8XY4
                      // SubVxVy(u16, u16), // 8XY5
                      // SubVyVx(u16, u16), // 8XY7
                      // RandVx(u16, u16),  // CXNN

                      // // Bitwise
                      // OrVxVy(u16, u16),   // 8XY1
                      // AndVxVy(u16, u16),  // 8XY2
                      // XorVxVy(u16, u16),  // 8XY3
                      // LSBShift(u16, u16), // 8XY6
                      // MSBShift(u16, u16), // 8XYE

                      // // Input
                      // KeyDown(u16),  // EX9E
                      // KeyUp(u16),    // EXA1
                      // AwaitKey(u16), // FX0A

                      // // Delay
                      // AddDelay(u16), // FX07
                      // SetDelay(u16), // FX15

                      // // Sound
                      // SetSound(u16), // FX18
}

# Chip8

A basic Chip8 emulator written in Rust.

![example](./space.gif)

The purpose of this program is for the project involved in writing it. This is my first emulator and writing it gave me a chance to learn a ton of programming concepts I don't get exposed to during my day job.  I have also never used Rust before so it was a chance to get experience with the beautiful language.

Supports:
- Keyboard input
- Basic sound
- SDL graphics and I/O
- Save/load state



# How to Use

```
cargo run ./roms/TETRIS
```

# Controls
Chip8 Input keyboard mapping (it's clunky):
 ```
 ╔═══╦═══╦═══╦═══╗
 ║ 1 ║ 2 ║ 3 ║ C ║ 1 - 4
 ╠═══╬═══╬═══╬═══╣
 ║ 4 ║ 5 ║ 6 ║ D ║ Q - R
 ╠═══╬═══╬═══╬═══╣
 ║ 7 ║ 8 ║ 9 ║ E ║ A - F
 ╠═══╬═══╬═══╬═══╣
 ║ A ║ 0 ║ B ║ F ║ Z - V
 ╚═══╩═══╩═══╩═══╝
 ```

- Save state: F5
- Load state: F9
- Pause/unpause: spacebar
- Advance one tick while paused: Right Arrow

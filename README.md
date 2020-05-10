## TODO

- Optimize by having a screen update flag. If true, an opcode has made a change to the screen.

- Move clock into a clock.rs file and expose as an async interface (the loop blocks waiting for enough time to pass for ticks)

## Debugger TODO

- Have a higher level loop that handles events.
- Have a flag for advancing the program at regular speed.
- Change the sleep logic to be a hot loop awaiting a timer to hit a certain value
- That value can be adjusted (speed up, slow down)
- Space to toggle run flag
- Arrow keys advance or rewind, one opcode at a time.

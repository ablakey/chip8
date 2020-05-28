# Chip8 Rust Emulator

## TODO

- remove Debugger experiment.
- Implement WebSocket server (async).
- Write dead simple web UI for debugging:
  - client-side state
  - accepts messages for:
    - current state of registers, flags, counters, timers
    - opcodes (500hz to we want to bin these together and send after, say, 500ms)
  - probably want the entire thing to run on a system where we accumulate events, state, and send in bulk.
  - UI uses vanilla CSS and <pre> tags to show us the state.
  - React? Would make it easier to handle diffing when certain memory or flags change.

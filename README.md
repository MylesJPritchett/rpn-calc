# rpn-calc

[![CI](https://github.com/MylesJPritchett/rpn-calc/actions/workflows/ci.yml/badge.svg)](https://github.com/MylesJPritchett/rpn-calc/actions/workflows/ci.yml)

A Reverse Polish Notation calculator for the terminal, for people who'd rather type `3 7 +` than reach for a GUI calculator. Built in Rust with ratatui.

![rpn-calc in a terminal](docs/screenshot.png)

There's no live demo: it's a terminal app, so you run it locally (see [Getting started](#getting-started)).

## What it does

- Type a number and press Enter to push it, or type an operation and press Enter to apply it to the stack.
- Binary operations: `+ - * / % ^`. The second value is the left-hand side, so `2 3 ^` is 8, as on HP calculators and `dc`.
- Unary operations: `neg abs sqrt`, trig (`sin cos tan asin acos atan`), `deg`/`rad` conversion, `!`, `recip`, `log10 logn log2`.
- Stack commands: `swap`, `drop`, `clear`, and `dup` (or Enter on an empty line) to duplicate the top value.
- `undo` and `redo` for any change to the stack.
- Constants `pi` and `inf`.
- An Operations Guide panel next to the stack lists every command by category, so you don't need to memorise them.
- The stack is saved after every command and reloaded on start, so your work survives quitting.

Index `0` in the Stack panel is the top of the stack. If a command can't run, a status line says why (`unknown command: foo`, `swap needs 2 values, stack has 1`) and clears on your next keypress. Maths results follow IEEE 754, so `10 0 /` gives `inf` rather than an error.

## Design

The calculator core (`src/calculator.rs`) is pure and UI-agnostic; persistence and the ratatui TUI sit around it. [`GLOSSARY.md`](GLOSSARY.md) defines the vocabulary.

- [Undo/redo stores whole-stack snapshots](docs/adr/0001-snapshot-based-undo-redo.md), so every operation is reversible without writing an inverse for each one.
- [Maths results follow IEEE 754 instead of erroring](docs/adr/0002-ieee-754-results-not-errors.md), including `!`, which gives `inf` above 170! instead of looping.
- [One Operation table drives both dispatch and the Operations Guide](docs/adr/0003-single-operation-table.md), so adding an operation is one line and the guide can't drift.
- [The stack is plain text in the platform data directory](docs/adr/0004-plain-text-stack-in-data-directory.md), so it's the same stack wherever you launch from.

## Stack

- Rust (2021 edition)
- [ratatui](https://ratatui.rs) for the UI and [crossterm](https://github.com/crossterm-rs/crossterm) for terminal input
- [dirs](https://github.com/dirs-dev/dirs-rs) to find the platform data directory
- [color-eyre](https://github.com/eyre-rs/color-eyre) for error reports

## Getting started

Requires a stable Rust toolchain ([rustup](https://rustup.rs)).

```sh
cargo run --release
```

The app starts in editing mode. Press Enter to submit, Esc to leave editing mode, then `e` to edit again or `q` to quit. Typing `quit` also exits. The stack is kept in `rpn-calc/stack.txt` in your platform data directory (`$XDG_DATA_HOME` or `~/.local/share` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows), so it's the same wherever you launch the app from. If the platform has no data directory, `stack.txt` in the current directory is used instead.

## Testing

```sh
cargo test      # every operation, undo/redo, float edge cases, persistence and end-to-end sessions
cargo clippy --all-targets -- -D warnings
```

The tests drive the calculator through its public interface and use a temp directory for persistence, so they never touch your real stack. The screenshot is generated from [`docs/demo.tape`](docs/demo.tape) with [VHS](https://github.com/charmbracelet/vhs): `vhs docs/demo.tape`.

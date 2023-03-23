# Tomato Timer

A simple TUI-based pomodoro-style timer and tracker implemented using Rust's cursive crate (using ncurses as the backend).

Intended to be used e.g. in a small corner pane when using Tmux or another multiplexer.

Current Features:

- Can supply a duration in minutes using the `-d` / `--duration` arguments
- Can set the timer to immediately start with the `-s` flag
- Can start the timer with the `s` hotkey
- Can pause and resume the timer with the `p` hotkey
- Can quit with the `q` hotkey
- Can increment / decrement the session length or time remaining with the `↑` and `↓` hotkeys.
- Can reset the timer after a finished session with the `r` hotkey.
- Background color changes to dark red on finishing a session to signal completion.

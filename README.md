# waybar-pomodoro

A lightweight [Pomodoro timer](https://en.wikipedia.org/wiki/Pomodoro_Technique) widget for [Waybar](https://github.com/Alexays/Waybar), written in Rust.

Runs as a long-lived daemon that streams JSON to Waybar once per second. Control it with left/right/middle clicks directly on the bar, or from any terminal/keybinding.

## Timer cycle

```
Work (25m) → Short Break (5m) → Work → Short Break → Work → Short Break → Work → Long Break (15m) → repeat
```

After every 4 completed work sessions the next break is a long one (15 min). When a phase ends, a desktop notification fires via `notify-send` and the widget waits in idle state until you click to start the next phase.

## Requirements

- Rust / Cargo (`rustup.rs`)
- Waybar
- `notify-send` (provided by `libnotify` / `mako` / any notification daemon)

## Build & install

```bash
git clone https://github.com/youruser/waybar-pomodoro
cd waybar-pomodoro

cargo build --release
cp target/release/waybar-pomodoro ~/.local/bin/
```

Make sure `~/.local/bin` is in your `$PATH`. If it is not, add this to `~/.bashrc` / `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Usage

The binary is controlled entirely through subcommands. The daemon mode (no subcommand) is meant to be launched by Waybar, not by hand.

| Command | Description |
|---|---|
| `waybar-pomodoro` | Start daemon (run by Waybar automatically) |
| `waybar-pomodoro toggle` | Start if idle, pause if running, resume if paused |
| `waybar-pomodoro skip` | Skip to the next phase immediately |
| `waybar-pomodoro reset` | Reset everything back to the initial idle state |

## Adding to Waybar

### 1. Add the module definition to `~/.config/waybar/config.jsonc`

```jsonc
"custom/pomodoro": {
  "exec": "waybar-pomodoro",
  "return-type": "json",
  "restart-interval": 5,
  "on-click": "waybar-pomodoro toggle",
  "on-click-right": "waybar-pomodoro skip",
  "on-click-middle": "waybar-pomodoro reset"
},
```

### 2. Place it in a module list

```jsonc
"modules-center": ["clock", "custom/pomodoro"],
```

Substitute whichever position you prefer (`modules-left`, `modules-center`, or `modules-right`).

### 3. Add styles to `~/.config/waybar/style.css`

```css
#custom-pomodoro {
  min-width: 75px;
  margin: 0 7.5px;
}

/* Work session – red */
#custom-pomodoro.work.running {
  color: #f38ba8;
}

/* Break – green */
#custom-pomodoro.short-break.running,
#custom-pomodoro.long-break.running {
  color: #a6e3a1;
}

#custom-pomodoro.paused {
  opacity: 0.6;
}

#custom-pomodoro.idle {
  opacity: 0.4;
}
```

### 4. Restart Waybar

```bash
omarchy-restart-waybar   # Omarchy systems
# or
killall waybar && waybar &
```

## Waybar click bindings

| Click | Action |
|---|---|
| Left | Start / pause |
| Right | Skip to next phase |
| Middle | Reset to idle |

## CSS classes

The widget emits two classes simultaneously, allowing fine-grained CSS targeting.

**Phase class** — `work` · `short-break` · `long-break`

**State class** — `running` · `paused` · `idle`

Example selectors:

```css
#custom-pomodoro.work.running   { /* 25-minute focus block */ }
#custom-pomodoro.work.paused    { /* paused mid-session   */ }
#custom-pomodoro.long-break.running { /* well-earned rest */ }
```

## State file

Timer state is persisted to `~/.local/share/waybar-pomodoro/state.json` so it survives Waybar restarts. Delete or overwrite this file to reset manually.

## Customising durations

Edit the constants at the top of `src/main.rs` and rebuild:

```rust
const WORK_SECS: u64 = 25 * 60;
const SHORT_BREAK_SECS: u64 = 5 * 60;
const LONG_BREAK_SECS: u64 = 15 * 60;
const SESSIONS_BEFORE_LONG_BREAK: u32 = 4;
```

```bash
cargo build --release
cp target/release/waybar-pomodoro ~/.local/bin/
```

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
- Python 3 (for the install/uninstall scripts — available by default on most distros)
- `notify-send` (provided by `libnotify` / `mako` / any notification daemon)

## Install

```bash
git clone https://github.com/youruser/waybar-pomodoro
cd waybar-pomodoro
make install
```

`make install` will:
1. Compile the release binary
2. Copy it to `~/.local/bin/`
3. Inject the module definition into `~/.config/waybar/config.jsonc`
4. Inject the CSS into `~/.config/waybar/style.css`
5. Restart Waybar

Make sure `~/.local/bin` is in your `$PATH`. If it is not, add this to `~/.bashrc` / `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Uninstall

```bash
make uninstall
```

`make uninstall` will:
1. Remove the binary from `~/.local/bin/`
2. Delete the state file at `~/.local/share/waybar-pomodoro/`
3. Remove the module definition from `~/.config/waybar/config.jsonc`
4. Remove the CSS from `~/.config/waybar/style.css`
5. Restart Waybar

## Usage

The binary is controlled entirely through subcommands. The daemon mode (no subcommand) is meant to be launched by Waybar, not by hand.

| Command | Description |
|---|---|
| `waybar-pomodoro` | Start daemon (run by Waybar automatically) |
| `waybar-pomodoro toggle` | Start if idle, pause if running, resume if paused |
| `waybar-pomodoro skip` | Skip to the next phase immediately |
| `waybar-pomodoro reset` | Reset everything back to the initial idle state |

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
#custom-pomodoro.work.running       { /* 25-minute focus block */ }
#custom-pomodoro.work.paused        { /* paused mid-session   */ }
#custom-pomodoro.long-break.running { /* well-earned rest     */ }
```

The default colours and styles live in `config/waybar-style.css` and can be edited before running `make install`.

## State file

Timer state is persisted to `~/.local/share/waybar-pomodoro/state.json` so it survives Waybar restarts. Delete or overwrite this file to reset manually.

## Customising durations

Edit the constants at the top of `src/main.rs` and re-run `make install`:

```rust
const WORK_SECS: u64 = 25 * 60;
const SHORT_BREAK_SECS: u64 = 5 * 60;
const LONG_BREAK_SECS: u64 = 15 * 60;
const SESSIONS_BEFORE_LONG_BREAK: u32 = 4;
```

## Manual Waybar setup

If you prefer to patch your Waybar config by hand instead of using `make install`, add the following.

**`~/.config/waybar/config.jsonc`** — module definition:

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

Add `"custom/pomodoro"` to whichever module list you prefer:

```jsonc
"modules-center": ["clock", "custom/pomodoro"],
```

**`~/.config/waybar/style.css`** — styles (see `config/waybar-style.css` for the full block).

Then restart Waybar:

```bash
omarchy-restart-waybar   # Omarchy systems
# or
killall waybar && waybar &
```

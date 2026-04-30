use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const WORK_SECS: u64 = 25 * 60;
const SHORT_BREAK_SECS: u64 = 5 * 60;
const LONG_BREAK_SECS: u64 = 15 * 60;
const SESSIONS_BEFORE_LONG_BREAK: u32 = 4;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TimerState {
    Running,
    Paused,
    Idle,
}

#[derive(Serialize, Deserialize, Clone)]
struct PomodoroState {
    phase: Phase,
    timer_state: TimerState,
    started_at: u64,
    elapsed_secs: u64,
    session_count: u32,
}

impl PomodoroState {
    fn new() -> Self {
        Self {
            phase: Phase::Work,
            timer_state: TimerState::Idle,
            started_at: 0,
            elapsed_secs: 0,
            session_count: 0,
        }
    }

    fn phase_duration(&self) -> u64 {
        match self.phase {
            Phase::Work => WORK_SECS,
            Phase::ShortBreak => SHORT_BREAK_SECS,
            Phase::LongBreak => LONG_BREAK_SECS,
        }
    }

    fn total_elapsed(&self) -> u64 {
        match self.timer_state {
            TimerState::Running => {
                let since = now_secs().saturating_sub(self.started_at);
                self.elapsed_secs + since
            }
            TimerState::Paused | TimerState::Idle => self.elapsed_secs,
        }
    }

    fn remaining(&self) -> u64 {
        self.phase_duration().saturating_sub(self.total_elapsed())
    }

    fn is_finished(&self) -> bool {
        self.timer_state == TimerState::Running
            && self.total_elapsed() >= self.phase_duration()
    }
}

fn state_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/waybar-pomodoro")
}

fn state_path() -> PathBuf {
    state_dir().join("state.json")
}

fn load_state() -> PomodoroState {
    let path = state_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                return state;
            }
        }
    }
    PomodoroState::new()
}

fn save_state(state: &PomodoroState) {
    let dir = state_dir();
    fs::create_dir_all(&dir).ok();
    let path = state_path();
    let tmp = path.with_extension("tmp");
    if let Ok(content) = serde_json::to_string(state) {
        if fs::write(&tmp, content).is_ok() {
            fs::rename(tmp, path).ok();
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn fmt_time(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn notify(summary: &str, body: &str) {
    std::process::Command::new("notify-send")
        .args(["-a", "Pomodoro", "-i", "alarm-clock", summary, body])
        .spawn()
        .ok();
}

fn next_phase(state: &mut PomodoroState) {
    match state.phase {
        Phase::Work => {
            state.session_count += 1;
            state.phase = if state.session_count % SESSIONS_BEFORE_LONG_BREAK == 0 {
                Phase::LongBreak
            } else {
                Phase::ShortBreak
            };
        }
        Phase::ShortBreak | Phase::LongBreak => {
            state.phase = Phase::Work;
        }
    }
    state.elapsed_secs = 0;
    state.started_at = 0;
    state.timer_state = TimerState::Idle;
}

fn print_waybar(state: &PomodoroState) {
    let remaining = state.remaining();
    let elapsed = state.total_elapsed().min(state.phase_duration());
    let percentage = (elapsed * 100 / state.phase_duration()) as u32;

    let (icon, phase_label, phase_class) = match state.phase {
        Phase::Work => ("🍅", "Work", "work"),
        Phase::ShortBreak => ("☕", "Short Break", "short-break"),
        Phase::LongBreak => ("🌿", "Long Break", "long-break"),
    };

    let text = match state.timer_state {
        TimerState::Idle => format!("{} {}", icon, fmt_time(state.phase_duration())),
        _ => format!("{} {}", icon, fmt_time(remaining)),
    };

    let state_class = match state.timer_state {
        TimerState::Running => "running",
        TimerState::Paused => "paused",
        TimerState::Idle => "idle",
    };

    let current_session = match state.phase {
        Phase::Work => state.session_count + 1,
        _ => state.session_count,
    };
    let session_line = format!("Session {}/{}", current_session, SESSIONS_BEFORE_LONG_BREAK);

    let status_line = match state.timer_state {
        TimerState::Running => format!("{} remaining", fmt_time(remaining)),
        TimerState::Paused => format!("Paused — {} remaining", fmt_time(remaining)),
        TimerState::Idle => "Idle — click to start".to_string(),
    };

    let tooltip = format!(
        "{} · {}\n{}\n\nLeft-click  start / pause\nRight-click skip phase\nMiddle-click reset",
        phase_label, session_line, status_line
    );

    let json = serde_json::json!({
        "text": text,
        "tooltip": tooltip,
        "class": format!("{} {}", phase_class, state_class),
        "percentage": percentage,
    });

    println!("{}", json);
    io::stdout().flush().ok();
}

fn cmd_toggle() {
    let mut state = load_state();
    match state.timer_state {
        TimerState::Idle | TimerState::Paused => {
            state.timer_state = TimerState::Running;
            state.started_at = now_secs();
        }
        TimerState::Running => {
            let since = now_secs().saturating_sub(state.started_at);
            state.elapsed_secs += since;
            state.timer_state = TimerState::Paused;
        }
    }
    save_state(&state);
}

fn cmd_skip() {
    let mut state = load_state();
    next_phase(&mut state);
    save_state(&state);
}

fn cmd_reset() {
    save_state(&PomodoroState::new());
}

fn run_daemon() {
    loop {
        let mut state = load_state();

        if state.is_finished() {
            let (title, body) = match state.phase {
                Phase::Work => ("🍅 Pomodoro complete!", "Time for a break."),
                Phase::ShortBreak => ("☕ Break over!", "Back to work."),
                Phase::LongBreak => ("🌿 Long break over!", "Back to work."),
            };
            notify(title, body);
            next_phase(&mut state);
            save_state(&state);
        }

        print_waybar(&state);
        thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        None => run_daemon(),
        Some("toggle") => cmd_toggle(),
        Some("skip") => cmd_skip(),
        Some("reset") => cmd_reset(),
        Some(cmd) => {
            eprintln!("waybar-pomodoro: unknown command '{}'", cmd);
            eprintln!("usage: waybar-pomodoro [toggle|skip|reset]");
            std::process::exit(1);
        }
    }
}

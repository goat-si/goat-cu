use std::sync::{mpsc, Arc, Mutex, Once};
use std::time::{Duration, Instant};

pub const KEEP_DISPLAY_AWAKE_ENV: &str = "CUA_DRIVER_KEEP_DISPLAY_AWAKE";
pub const DEFAULT_IDLE_WINDOW: Duration = Duration::from_secs(600);

pub fn idle_window_from_env(value: Option<&str>) -> Option<Duration> {
    let Some(raw) = value.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Some(DEFAULT_IDLE_WINDOW);
    };
    match raw.to_ascii_lowercase().as_str() {
        "0" | "off" | "false" | "no" => None,
        "on" | "true" | "yes" => Some(DEFAULT_IDLE_WINDOW),
        other => match other.parse::<u64>() {
            Ok(seconds) => Some(Duration::from_secs(seconds.min(86_400))),
            Err(_) => {
                tracing::warn!("{KEEP_DISPLAY_AWAKE_ENV}={raw:?} is not a number of seconds or off; using the default");
                Some(DEFAULT_IDLE_WINDOW)
            }
        },
    }
}

#[derive(Default)]
struct IdleState {
    last_activity: Option<Instant>,
}

impl IdleState {
    fn activity(&mut self, now: Instant, idle: Option<Duration>) {
        if idle.is_some() {
            self.last_activity = Some(now);
        }
    }

    fn remaining(&self, now: Instant, idle: Option<Duration>) -> Option<Duration> {
        let elapsed = now.saturating_duration_since(self.last_activity?);
        idle?
            .checked_sub(elapsed)
            .filter(|remaining| !remaining.is_zero())
    }
}

pub fn install() {
    static INSTALL_ONCE: Once = Once::new();
    INSTALL_ONCE.call_once(start_worker);
}

fn start_worker() {
    let Some(idle) = idle_window_from_env(std::env::var(KEEP_DISPLAY_AWAKE_ENV).ok().as_deref())
    else {
        return;
    };
    let state = Arc::new(Mutex::new(IdleState::default()));
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker_state = Arc::clone(&state);
    if std::thread::Builder::new()
        .name("display-awake".into())
        .spawn(move || worker(worker_state, receiver, idle))
        .is_err()
    {
        tracing::warn!("could not start display-awake worker");
        return;
    }
    cua_driver_core::tool_activity::set_tool_activity_fn(move || {
        state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .activity(Instant::now(), Some(idle));
        let _ = sender.try_send(());
    });
}

const RETRY_FIRST: Duration = Duration::from_secs(5);
const RETRY_MAX: Duration = Duration::from_secs(120);

struct Retry {
    next_attempt: Option<Instant>,
    delay: Duration,
}

impl Retry {
    fn new() -> Self {
        Self {
            next_attempt: None,
            delay: RETRY_FIRST,
        }
    }

    fn wait(&self, now: Instant) -> Option<Duration> {
        self.next_attempt
            .and_then(|at| at.checked_duration_since(now))
            .filter(|wait| !wait.is_zero())
    }

    fn failed(&mut self, now: Instant) {
        self.next_attempt = Some(now + self.delay);
        self.delay = (self.delay * 2).min(RETRY_MAX);
    }
}

fn worker(state: Arc<Mutex<IdleState>>, receiver: mpsc::Receiver<()>, idle: Duration) {
    let mut guard = PlatformGuard::new();
    let mut active = false;
    let mut retry = Retry::new();
    loop {
        let now = Instant::now();
        let remaining = state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remaining(now, Some(idle));
        let mut wait = remaining;
        match remaining {
            Some(_) if !active => match retry.wait(now) {
                Some(backoff) => wait = wait.map(|wait| wait.min(backoff)),
                None if guard.acquire() => {
                    active = true;
                    retry = Retry::new();
                    continue;
                }
                None => {
                    retry.failed(Instant::now());
                    continue;
                }
            },
            None if active => {
                guard.release();
                active = false;
            }
            None => retry = Retry::new(),
            Some(_) => {}
        }
        let result = match wait {
            Some(wait) => receiver.recv_timeout(wait),
            None => receiver
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        if matches!(result, Err(mpsc::RecvTimeoutError::Disconnected)) {
            break;
        }
    }
    guard.release();
}

struct PlatformGuard;

impl PlatformGuard {
    fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    fn acquire(&mut self) -> bool {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED) != 0
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn acquire(&mut self) -> bool {
        true
    }

    #[cfg(target_os = "windows")]
    fn release(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn release(&mut self) {}
}

#[cfg(target_os = "windows")]
const ES_CONTINUOUS: u32 = 0x8000_0000;
#[cfg(target_os = "windows")]
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;
#[cfg(target_os = "windows")]
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn SetThreadExecutionState(flags: u32) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_contract() {
        for value in [
            None,
            Some(""),
            Some(" on "),
            Some("TRUE"),
            Some("yes"),
            Some("bad"),
        ] {
            assert_eq!(idle_window_from_env(value), Some(DEFAULT_IDLE_WINDOW));
        }
        for value in ["0", "off", "FALSE", "No"] {
            assert_eq!(idle_window_from_env(Some(value)), None);
        }
        assert_eq!(
            idle_window_from_env(Some("120")),
            Some(Duration::from_secs(120))
        );
        assert_eq!(
            idle_window_from_env(Some("999999")),
            Some(Duration::from_secs(86_400))
        );
    }

    #[test]
    fn activity_extends_idle_window_and_disabled_stays_inactive() {
        let start = Instant::now();
        let idle = Some(Duration::from_secs(10));
        let mut state = IdleState::default();
        state.activity(start, None);
        assert_eq!(state.remaining(start, None), None);
        state.activity(start, idle);
        assert_eq!(state.remaining(start, idle), idle);
        state.activity(start + Duration::from_secs(8), idle);
        assert_eq!(
            state.remaining(start + Duration::from_secs(10), idle),
            Some(Duration::from_secs(8))
        );
        assert_eq!(state.remaining(start + Duration::from_secs(18), idle), None);
        assert_eq!(state.remaining(start + Duration::from_secs(19), idle), None);
    }

    #[test]
    fn failed_acquisition_retries_with_bounded_backoff() {
        let start = Instant::now();
        let mut retry = Retry::new();
        assert_eq!(retry.wait(start), None);
        retry.failed(start);
        assert_eq!(retry.wait(start), Some(RETRY_FIRST));
        assert_eq!(retry.wait(start + RETRY_FIRST), None);
        let mut at = start;
        for _ in 0..10 {
            retry.failed(at);
            at += RETRY_MAX;
        }
        retry.failed(at);
        assert_eq!(retry.wait(at), Some(RETRY_MAX));
    }
}

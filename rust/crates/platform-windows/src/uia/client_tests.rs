use super::*;
use std::sync::{mpsc, Arc, Barrier};
use std::time::{Duration, Instant};
use windows::core::w;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, HMENU, WINDOW_EX_STYLE, WINDOW_STYLE,
};

struct Com;
impl Com {
    fn new() -> Self {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .expect("initialize MTA")
        };
        Self
    }
}
impl Drop for Com {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}

struct BlockedWindow {
    hwnd: isize,
    release: Option<mpsc::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl BlockedWindow {
    fn new() -> Self {
        let (hwnd_tx, hwnd_rx) = mpsc::channel();
        let (release, rx) = mpsc::channel();
        let thread = std::thread::spawn(move || unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("W4 fixture"),
                WINDOW_STYLE(0),
                0,
                0,
                100,
                100,
                HWND::default(),
                HMENU::default(),
                HINSTANCE::default(),
                None,
            )
            .expect("create hidden window");
            hwnd_tx.send(hwnd.0 as isize).expect("send HWND");
            let _ = rx.recv();
            DestroyWindow(hwnd).expect("destroy window");
        });
        let hwnd = hwnd_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("window ready");
        Self {
            hwnd,
            release: Some(release),
            thread: Some(thread),
        }
    }
}
impl Drop for BlockedWindow {
    fn drop(&mut self) {
        drop(self.release.take());
        if let Some(thread) = self.thread.take() {
            thread.join().expect("window thread");
        }
    }
}

#[test]
fn bounded_client_reports_configured_timeouts() {
    let _com = Com::new();
    let client = new_uia_client().expect("bounded client");
    let client2: IUIAutomation2 = client.cast().expect("IUIAutomation2");
    unsafe {
        assert_eq!(client2.ConnectionTimeout().unwrap(), CONNECTION_TIMEOUT_MS);
        assert_eq!(
            client2.TransactionTimeout().unwrap(),
            TRANSACTION_TIMEOUT_MS
        );
    }
}

// A shared test process cannot guarantee this starts with a cold UIA core.
#[test]
fn concurrent_clients_do_not_hit_reentrant_failure() {
    let barrier = Arc::new(Barrier::new(4));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let _com = Com::new();
                barrier.wait();
                let client = new_uia_client().expect("client construction");
                if let Err(error) = unsafe { client.GetRootElement() } {
                    eprintln!("GetRootElement HRESULT: {:?}", error.code());
                    assert_ne!(error.code().0, E_FAIL);
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn probe_distinguishes_blocked_and_destroyed_window() {
    let window = BlockedWindow::new();
    let hwnd = HWND(window.hwnd as *mut _);
    let start = Instant::now();
    assert_eq!(probe_root(hwnd), RootVerdict::NoAnswer);
    assert!(start.elapsed() < Duration::from_secs(2));
    drop(window);
    assert_eq!(probe_root(hwnd), RootVerdict::WindowGone);
}

/// The marker a test process passes to the lone child it spawns: its pid under a prefix, so a
/// bare pid or any other value left in the environment cannot match.
fn lone_child_marker(parent: u32) -> String {
    format!("spawned-by-{parent}")
}

/// Whether this process is the lone child its parent spawned.
fn is_lone_child(marker: Option<&str>, parent: Option<u32>) -> bool {
    parent.is_some_and(|parent| marker == Some(lone_child_marker(parent).as_str()))
}

fn parent_pid() -> Option<u32> {
    let me = std::process::id();
    crate::win32::apps::list_processes()
        .into_iter()
        .find(|process| process.pid == me)
        .map(|process| process.parent_pid)
}

#[test]
fn a_leaked_marker_never_makes_the_suite_the_lone_child() {
    assert!(is_lone_child(Some(&lone_child_marker(42)), Some(42)));
    assert!(!is_lone_child(Some("42"), Some(42)));
    assert!(!is_lone_child(Some(&lone_child_marker(41)), Some(42)));
    assert!(!is_lone_child(Some(&lone_child_marker(42)), None));
    assert!(!is_lone_child(None, Some(42)));
}

#[test]
fn bounded_element_from_handle_times_out_without_probe() {
    // The bound times one real UIA call, while other tests in this binary create UIA clients,
    // blocked windows and COM apartments on other threads of the same process; time the call in
    // a process that runs this test alone.
    const ALONE: &str = "CUA_UIA_BOUNDED_TIMEOUT_ALONE";
    if !is_lone_child(std::env::var(ALONE).ok().as_deref(), parent_pid()) {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "uia::client::tests::bounded_element_from_handle_times_out_without_probe",
            ])
            .env(ALONE, lone_child_marker(std::process::id()))
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains(" 1 passed;"),
            "{stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let window = BlockedWindow::new();
    let hwnd = window.hwnd;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _com = Com::new();
        let client = new_uia_client().expect("bounded client");
        let start = Instant::now();
        let result = unsafe { client.ElementFromHandle(HWND(hwnd as *mut _)) };
        let code = result.err().map(|error| error.code().0);
        tx.send((code, start.elapsed())).expect("send result");
    });
    let (code, elapsed) = rx
        .recv_timeout(Duration::from_secs(15))
        .expect("independent UIA watchdog");
    assert_eq!(code, Some(0x80131505u32 as i32));
    assert!(elapsed < Duration::from_secs(4), "elapsed: {elapsed:?}");
}

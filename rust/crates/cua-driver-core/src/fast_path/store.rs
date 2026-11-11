// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

//! In-memory read-time state for the guards: element fingerprints per
//! snapshot, and the latest window image plus window fingerprint per window.
//! Nothing goes to disk. Fingerprints of retired snapshots stay for the same
//! bounded window so an older token can be checked instead of refused.
//!
//! The store is daemon wide, so every entry carries the session that
//! published it and a lookup only sees its own session's entries: one MCP
//! session's read must not answer another session's target lookup.

use super::fingerprint::ElementFingerprint;
use super::pixel::GrayImage;
use super::window::WindowFingerprint;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

/// Snapshots whose fingerprints are kept (8 per pid in the snapshot LRU,
/// times a few busy pids).
pub const SNAPSHOT_CAPACITY: usize = 64;
/// Windows whose read-time image is kept.
pub const IMAGE_CAPACITY: usize = 16;

#[derive(Clone, Debug)]
pub struct SnapshotFingerprints {
    pub pid: i32,
    pub window_id: u64,
    /// The session that published the read, as the daemon named it. `None`
    /// is the unnamed (daemon-global) caller, and only matches `None`.
    pub session: Option<String>,
    pub elements: Vec<ElementFingerprint>,
    /// Cheap stats for target classification.
    pub element_count: usize,
    pub thin_frames: usize,
}

#[derive(Clone, Debug)]
pub struct ReadImage {
    pub snapshot_id: u32,
    pub image: GrayImage,
    pub window: WindowFingerprint,
}

#[derive(Default)]
struct State {
    snapshots: HashMap<u32, SnapshotFingerprints>,
    snapshot_order: VecDeque<u32>,
    images: HashMap<(Option<String>, i32, u64), ReadImage>,
    image_order: VecDeque<(Option<String>, i32, u64)>,
    /// Latest window fingerprint even when no image was kept.
    windows: HashMap<(Option<String>, i32, u64), WindowFingerprint>,
}

/// Key of a window-scoped entry: the publishing session, the pid, and the
/// window. Two sessions reading the same window keep separate entries.
type WindowKey = (Option<String>, i32, u64);

fn window_key(session: Option<&str>, pid: i32, window_id: u64) -> WindowKey {
    (session.map(str::to_owned), pid, window_id)
}

fn same_window(key: &WindowKey, pid: i32, window_id: u64) -> bool {
    key.1 == pid && key.2 == window_id
}

#[derive(Default)]
pub struct GuardStore {
    inner: Mutex<State>,
}

impl GuardStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put_snapshot(&self, snapshot_id: u32, fps: SnapshotFingerprints) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if s.snapshots.insert(snapshot_id, fps).is_none() {
            s.snapshot_order.push_back(snapshot_id);
        }
        while s.snapshot_order.len() > SNAPSHOT_CAPACITY {
            if let Some(old) = s.snapshot_order.pop_front() {
                s.snapshots.remove(&old);
            }
        }
    }

    pub fn element(
        &self,
        snapshot_id: u32,
        index: usize,
        session: Option<&str>,
    ) -> Option<(i32, u64, ElementFingerprint)> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let snap = s
            .snapshots
            .get(&snapshot_id)
            .filter(|f| f.session.as_deref() == session)?;
        snap.elements
            .get(index)
            .map(|fp| (snap.pid, snap.window_id, fp.clone()))
    }

    pub fn stats(&self, snapshot_id: u32, session: Option<&str>) -> Option<(usize, usize)> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.snapshots
            .get(&snapshot_id)
            .filter(|f| f.session.as_deref() == session)
            .map(|f| (f.element_count, f.thin_frames))
    }

    pub fn latest_stats(
        &self,
        pid: i32,
        window_id: u64,
        session: Option<&str>,
    ) -> Option<(usize, usize)> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.snapshot_order
            .iter()
            .rev()
            .filter_map(|id| s.snapshots.get(id))
            .find(|f| f.pid == pid && f.window_id == window_id && f.session.as_deref() == session)
            .map(|f| (f.element_count, f.thin_frames))
    }

    /// The smallest element of the window's latest snapshot whose read-time
    /// frame contains the screen point: what the agent saw there.
    pub fn element_at(
        &self,
        pid: i32,
        window_id: u64,
        x: f64,
        y: f64,
        session: Option<&str>,
    ) -> Option<ElementFingerprint> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let snap = s
            .snapshot_order
            .iter()
            .rev()
            .filter_map(|id| s.snapshots.get(id))
            .find(|f| {
                f.pid == pid && f.window_id == window_id && f.session.as_deref() == session
            })?;
        snap.elements
            .iter()
            .filter(|fp| fp.frame.is_some_and(|f| f.contains(x, y) && f.area() > 0.0))
            .min_by(|a, b| {
                let (a, b) = (a.frame.unwrap().area(), b.frame.unwrap().area());
                a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    pub fn put_window(&self, pid: i32, window: WindowFingerprint, session: Option<&str>) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let key = window_key(session, pid, window.window_id);
        s.windows.insert(key.clone(), window);
        if s.windows.len() > 4 * IMAGE_CAPACITY {
            let keep: Vec<_> = s.image_order.iter().cloned().collect();
            s.windows.retain(|k, _| keep.contains(k));
            s.windows.insert(key, window);
        }
    }

    pub fn window(
        &self,
        pid: i32,
        window_id: u64,
        session: Option<&str>,
    ) -> Option<WindowFingerprint> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.windows.get(&window_key(session, pid, window_id)).copied()
    }

    pub fn put_image(&self, pid: i32, image: ReadImage, session: Option<&str>) {
        let key = window_key(session, pid, image.window.window_id);
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.windows.insert(key.clone(), image.window);
        if s.images.insert(key.clone(), image).is_none() {
            s.image_order.push_back(key);
        } else {
            s.image_order.retain(|k| *k != key);
            s.image_order.push_back(key);
        }
        while s.image_order.len() > IMAGE_CAPACITY {
            if let Some(old) = s.image_order.pop_front() {
                s.images.remove(&old);
            }
        }
    }

    pub fn image(&self, pid: i32, window_id: u64, session: Option<&str>) -> Option<ReadImage> {
        let s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.images.get(&window_key(session, pid, window_id)).cloned()
    }

    /// Drop the window's recorded state for every session: the window is gone
    /// or was replaced, so no session's cached image may outlive it.
    pub fn forget_window(&self, pid: i32, window_id: u64) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.images.retain(|k, _| !same_window(k, pid, window_id));
        s.image_order.retain(|k| !same_window(k, pid, window_id));
        s.windows.retain(|k, _| !same_window(k, pid, window_id));
    }
}

/// The daemon-wide store.
pub fn global() -> &'static GuardStore {
    static STORE: OnceLock<GuardStore> = OnceLock::new();
    STORE.get_or_init(GuardStore::new)
}

#[cfg(test)]
mod tests {
    use super::super::Rect;
    use super::*;

    fn win(id: u64) -> WindowFingerprint {
        WindowFingerprint {
            window_id: id,
            pid: 1,
            frame: Rect::new(0.0, 0.0, 10.0, 10.0),
            scale: 1.0,
        }
    }

    #[test]
    fn retired_snapshot_fingerprints_stay_until_capacity() {
        let store = GuardStore::new();
        for id in 0..(SNAPSHOT_CAPACITY as u32 + 5) {
            store.put_snapshot(
                id,
                SnapshotFingerprints {
                    pid: 1,
                    window_id: 2,
                    session: None,
                    elements: vec![ElementFingerprint {
                        role: format!("r{id}"),
                        ..Default::default()
                    }],
                    element_count: 1,
                    thin_frames: 0,
                },
            );
        }
        assert!(store.element(0, 0, None).is_none());
        assert_eq!(store.element(10, 0, None).unwrap().2.role, "r10");
        assert!(store.element(10, 1, None).is_none());
        assert_eq!(store.latest_stats(1, 2, None), Some((1, 0)));
    }

    #[test]
    fn element_at_picks_the_smallest_containing_frame() {
        let store = GuardStore::new();
        let el = |role: &str, f: Rect| ElementFingerprint {
            role: role.into(),
            frame: Some(f),
            ..Default::default()
        };
        store.put_snapshot(
            1,
            SnapshotFingerprints {
                pid: 1,
                window_id: 2,
                session: None,
                elements: vec![
                    el("AXGroup", Rect::new(0.0, 0.0, 500.0, 500.0)),
                    el("AXButton", Rect::new(10.0, 10.0, 80.0, 24.0)),
                ],
                element_count: 2,
                thin_frames: 0,
            },
        );
        assert_eq!(
            store.element_at(1, 2, 20.0, 20.0, None).unwrap().role,
            "AXButton"
        );
        assert_eq!(
            store.element_at(1, 2, 300.0, 300.0, None).unwrap().role,
            "AXGroup"
        );
        assert!(store.element_at(1, 2, 900.0, 900.0, None).is_none());
        assert!(store.element_at(1, 3, 20.0, 20.0, None).is_none());
    }

    #[test]
    fn images_are_bounded_and_latest_wins() {
        let store = GuardStore::new();
        for id in 0..(IMAGE_CAPACITY as u64 + 3) {
            store.put_image(
                1,
                ReadImage {
                    snapshot_id: 1,
                    image: GrayImage::filled(2, 2, 0),
                    window: win(id),
                },
                None,
            );
        }
        assert!(store.image(1, 0, None).is_none());
        store.put_image(
            1,
            ReadImage {
                snapshot_id: 9,
                image: GrayImage::filled(2, 2, 0),
                window: win(5),
            },
            None,
        );
        assert_eq!(store.image(1, 5, None).unwrap().snapshot_id, 9);
        store.forget_window(1, 5);
        assert!(store.image(1, 5, None).is_none() && store.window(1, 5, None).is_none());
    }

    fn named_snapshot(session: Option<&str>) -> SnapshotFingerprints {
        SnapshotFingerprints {
            pid: 1,
            window_id: 2,
            session: session.map(str::to_owned),
            elements: vec![ElementFingerprint {
                role: "AXButton".into(),
                frame: Some(Rect::new(0.0, 0.0, 10.0, 10.0)),
                ..Default::default()
            }],
            element_count: 1,
            thin_frames: 0,
        }
    }

    #[test]
    fn another_sessions_snapshot_is_not_visible() {
        let store = GuardStore::new();
        store.put_snapshot(1, named_snapshot(Some("a")));
        store.put_image(
            1,
            ReadImage {
                snapshot_id: 1,
                image: GrayImage::filled(2, 2, 0),
                window: win(5),
            },
            Some("a"),
        );

        // Session "b" sees none of "a"'s entries.
        assert!(store.element(1, 0, Some("b")).is_none());
        assert!(store.stats(1, Some("b")).is_none());
        assert!(store.latest_stats(1, 2, Some("b")).is_none());
        assert!(store.element_at(1, 2, 5.0, 5.0, Some("b")).is_none());
        assert!(store.image(1, 5, Some("b")).is_none());
        assert!(store.window(1, 5, Some("b")).is_none());

        // "a" still sees its own.
        assert_eq!(store.element(1, 0, Some("a")).unwrap().2.role, "AXButton");
        assert_eq!(store.stats(1, Some("a")), Some((1, 0)));
        assert_eq!(store.latest_stats(1, 2, Some("a")), Some((1, 0)));
        assert_eq!(
            store.element_at(1, 2, 5.0, 5.0, Some("a")).unwrap().role,
            "AXButton"
        );
        assert_eq!(store.image(1, 5, Some("a")).unwrap().snapshot_id, 1);
        assert_eq!(store.window(1, 5, Some("a")).unwrap().window_id, 5);

        // An unnamed entry is invisible to a named session, and the other way
        // round.
        store.put_snapshot(2, named_snapshot(None));
        store.put_image(
            3,
            ReadImage {
                snapshot_id: 2,
                image: GrayImage::filled(2, 2, 0),
                window: win(6),
            },
            None,
        );
        assert!(store.element(2, 0, Some("a")).is_none());
        assert!(store.stats(2, Some("a")).is_none());
        assert!(store.element(1, 0, None).is_none());
        assert!(store.image(3, 6, Some("a")).is_none());
        assert!(store.window(3, 6, Some("a")).is_none());
        assert!(store.element(2, 0, None).is_some());
        assert!(store.image(3, 6, None).is_some());

        // forget_window drops every session's entries for the window.
        store.put_image(
            1,
            ReadImage {
                snapshot_id: 4,
                image: GrayImage::filled(2, 2, 0),
                window: win(5),
            },
            Some("b"),
        );
        store.forget_window(1, 5);
        assert!(store.image(1, 5, Some("a")).is_none());
        assert!(store.image(1, 5, Some("b")).is_none());
        assert!(store.window(1, 5, Some("a")).is_none());
        assert!(store.window(1, 5, Some("b")).is_none());
    }

    /// #288 review: the capacity and retention paths (image eviction, the
    /// window map's retain over the image order, snapshot eviction) keep every
    /// surviving entry answering only the session that published it.
    #[test]
    fn evictions_keep_each_entry_in_its_own_session() {
        let store = GuardStore::new();
        let own = |id: u64| if id % 2 == 0 { "a" } else { "b" };
        let other = |id: u64| if id % 2 == 0 { "b" } else { "a" };
        let images = IMAGE_CAPACITY as u64 + 4;
        for id in 0..images {
            store.put_image(
                1,
                ReadImage {
                    snapshot_id: id as u32,
                    image: GrayImage::filled(2, 2, 0),
                    window: win(id),
                },
                Some(own(id)),
            );
        }
        for id in 0..images {
            assert!(store.image(1, id, Some(other(id))).is_none(), "{id}");
            assert_eq!(
                store.image(1, id, Some(own(id))).map(|i| i.snapshot_id),
                (id >= 4).then_some(id as u32),
                "{id}"
            );
        }
        let windows = 100..(100 + 4 * IMAGE_CAPACITY as u64);
        for id in windows.clone() {
            store.put_window(1, win(id), Some(own(id)));
        }
        for id in 4..images {
            assert_eq!(
                store.window(1, id, Some(own(id))).map(|w| w.window_id),
                Some(id),
                "{id}"
            );
            assert!(store.window(1, id, Some(other(id))).is_none(), "{id}");
        }
        for id in windows {
            assert!(store.window(1, id, Some(other(id))).is_none(), "{id}");
        }
        let snapshots = SNAPSHOT_CAPACITY as u32 + 6;
        for id in 0..snapshots {
            store.put_snapshot(id, named_snapshot(Some(own(u64::from(id)))));
        }
        for id in 0..snapshots {
            let id64 = u64::from(id);
            assert!(store.element(id, 0, Some(other(id64))).is_none(), "{id}");
            assert_eq!(
                store.element(id, 0, Some(own(id64))).is_some(),
                id >= 6,
                "{id}"
            );
        }
    }
}

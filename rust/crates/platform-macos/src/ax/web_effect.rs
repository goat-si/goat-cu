// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

//! Whether a background press changed the web page it landed on.
//!
//! An AX press on web content returns success whatever the page does with it:
//! Chromium and WebKit turn it into a click event and report nothing back.
//! The page can ignore that click. A control that checks
//! `document.hasFocus()` sees an unfocused document while the browser is in
//! the background, so the press does nothing, and only
//! `delivery_mode:"foreground"` gets through (#4243).
//!
//! [`PageProbe`] reads the accessibility subtree of the browser window before
//! the press and again for a short settle window after it. The whole window is
//! read, not only the page, so a press that opens a tab or a download bubble
//! counts as a change. A window that reads the same for the whole settle
//! window is the evidence for a suspected no-op. Anything the probe cannot
//! read completely is `Unknown`, never a no-op.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use core_foundation::base::{CFRelease, CFTypeRef};

use super::batch_attrs::{copy_attrs_batch, BatchRead, BatchSlots};
use super::bindings::{
    bool_from_cf, point_from_cf, size_from_cf, string_from_cf, stringish_from_cf,
    AXUIElementGetTypeID,
};
use super::bindings::{
    copy_element_attr, copy_string_attr, AXUIElementRef, AXUIElementSetMessagingTimeout,
};
use super::snapshot::RetainedElement;
use core_foundation::{
    array::{CFArray, CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    base::{CFGetTypeID, TCFType},
};

/// How long after the press the page may take to show its change. A press the
/// page handled showed up in the tree within about 100 ms on Chrome.
pub(crate) const SETTLE: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(40);
/// A window bigger than this reads as `Unknown`: no no-op claim on a window
/// the probe did not read whole.
const MAX_NODES: usize = 4_000;
/// Time budget for one read of the window.
const READ_BUDGET: Duration = Duration::from_millis(250);
const MESSAGING_TIMEOUT_SECONDS: f32 = 0.25;
const MAX_ANCESTRY: usize = 40;

/// What the page did after the press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PageChange {
    Changed,
    Unchanged,
    /// The page could not be read completely, before or after.
    Unknown,
}

/// The window of a web element, read just before a press.
pub(crate) struct PageProbe {
    pid: i32,
    window: RetainedElement,
    before: u64,
}

impl PageProbe {
    /// Read the window that holds `element`. `None` when the element is not
    /// web content, is a text input (a press there only moves focus, which the
    /// tree does not show), or the window could not be read completely.
    ///
    /// # Safety
    ///
    /// `element` must be a valid, live `AXUIElementRef` of `pid`.
    pub(crate) unsafe fn before_press(pid: i32, element: AXUIElementRef) -> Option<Self> {
        if matches!(
            copy_string_attr(element, "AXRole").as_deref(),
            Some("AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField")
        ) {
            return None;
        }
        let window = web_window_containing(element)?;
        let before = read_window(pid, window.as_ptr() as AXUIElementRef)?;
        Some(Self {
            pid,
            window,
            before,
        })
    }

    /// Watch the window until it changes or `SETTLE` after `pressed_at` runs
    /// out.
    pub(crate) fn settle(&self, pressed_at: Instant) -> PageChange {
        let window = self.window.as_ptr() as AXUIElementRef;
        watch(
            self.before,
            || unsafe { read_window(self.pid, window) },
            || SETTLE.saturating_sub(pressed_at.elapsed()),
            std::thread::sleep,
        )
    }
}

/// Read until the page differs from `before` or no time is `remaining`.
/// A read that fails makes the outcome `Unknown`.
fn watch(
    before: u64,
    mut read: impl FnMut() -> Option<u64>,
    mut remaining: impl FnMut() -> Duration,
    mut sleep: impl FnMut(Duration),
) -> PageChange {
    loop {
        match read() {
            None => return PageChange::Unknown,
            Some(now) if now != before => return PageChange::Changed,
            Some(_) => {}
        }
        let left = remaining();
        if left.is_zero() {
            return PageChange::Unchanged;
        }
        sleep(POLL.min(left));
    }
}

/// The `AXWindow` above `element`, retained, when an `AXWebArea` lies between
/// them.
unsafe fn web_window_containing(element: AXUIElementRef) -> Option<RetainedElement> {
    let mut current = RetainedElement::retain(element as usize);
    let mut in_web_area = false;
    for _ in 0..MAX_ANCESTRY {
        let el = current.as_ptr() as AXUIElementRef;
        match copy_string_attr(el, "AXRole").as_deref() {
            Some("AXWebArea") => in_web_area = true,
            Some("AXWindow") => return in_web_area.then_some(current),
            Some("AXApplication") | None => return None,
            Some(_) => {}
        }
        let parent = copy_element_attr(el, "AXParent")?;
        current = RetainedElement::retain(parent as usize);
        CFRelease(parent as CFTypeRef);
    }
    None
}

/// The state of one node the digest covers. Focus is left out: the press
/// itself can move it without the page doing anything. Labels and geometry
/// are read only inside the page, plus the window title: the browser's own
/// chrome rewrites labels on its own (a tab's memory figure). Values are read
/// everywhere, so a navigation shows as soon as the address bar changes, and a
/// new tab or a download bubble shows as a structural change.
#[derive(Hash)]
struct NodeFacts {
    role: Option<String>,
    subrole: Option<String>,
    value: Option<String>,
    enabled: Option<bool>,
    selected: Option<bool>,
    children: usize,
    text: Option<NodeText>,
}

#[derive(Hash)]
struct NodeText {
    title: Option<String>,
    description: Option<String>,
    frame: Option<[u64; 4]>,
}

// Fixed positions in the GOAT batch decoder's requested array.
const ATTRS: [&str; 10] = [
    "AXRole",
    "AXSubrole",
    "AXValue",
    "AXEnabled",
    "AXSelected",
    "AXChildren",
    "AXTitle",
    "AXDescription",
    "AXPosition",
    "AXSize",
];

/// Distinguish an answered absent attribute from a missing or malformed slot.
unsafe fn decoded<T>(
    slots: &BatchSlots,
    index: usize,
    convert: unsafe fn(CFTypeRef) -> Option<T>,
) -> Option<Option<T>> {
    if !slots.answered(index) {
        return None;
    }
    match slots.raw(index) {
        Some(value) => Some(Some(convert(value)?)),
        None => Some(None),
    }
}

/// One digest of the subtree under `window`, or `None` when the read did not
/// cover every node within the budget.
unsafe fn read_window(pid: i32, window: AXUIElementRef) -> Option<u64> {
    // A hidden or covered Chromium window holds back its page changes until it
    // is asked something; ask first so the read sees the page as it is now.
    super::enablement::flush_pending_web_updates(pid, window);
    let started = Instant::now();
    let mut hasher = DefaultHasher::new();
    let mut stack = vec![(RetainedElement::retain(window as usize), false)];
    let mut visited = 0usize;
    while let Some((node, parent_in_page)) = stack.pop() {
        visited += 1;
        if visited > MAX_NODES || started.elapsed() > READ_BUDGET {
            return None;
        }
        let el = node.as_ptr() as AXUIElementRef;
        AXUIElementSetMessagingTimeout(el, MESSAGING_TIMEOUT_SECONDS);
        // Only a batched reply whose every digested attribute answered is
        // read: the one-by-one fallback and a failed attribute read cannot be
        // told from a missing attribute, so two failed reads could match and
        // pass for an unchanged page.
        let BatchRead::Slots(attrs) = copy_attrs_batch(el, &ATTRS) else {
            return None;
        };
        // An answered absent attribute is different from a missing slot.
        let role = Some(decoded(&attrs, 0, string_from_cf)??);
        let in_page = parent_in_page || role.as_deref() == Some("AXWebArea");
        let is_window = el == window;
        let subrole = decoded(&attrs, 1, string_from_cf)?;
        let value = decoded(&attrs, 2, stringish_from_cf)?.map(|value| value.state_value);
        let enabled = decoded(&attrs, 3, bool_from_cf)?;
        let selected = decoded(&attrs, 4, bool_from_cf)?;
        if !attrs.answered(5) {
            return None;
        }
        let mut children = Vec::new();
        if let Some(raw_children) = attrs.raw(5) {
            if CFGetTypeID(raw_children) != CFArray::<CFTypeRef>::type_id() {
                return None;
            }
            let array = raw_children as CFArrayRef;
            for index in 0..CFArrayGetCount(array) {
                let child = CFArrayGetValueAtIndex(array, index) as CFTypeRef;
                if child.is_null() || CFGetTypeID(child) != AXUIElementGetTypeID() {
                    return None;
                }
                children.push(RetainedElement::retain(child as usize));
            }
        }
        let text = if in_page {
            let title = decoded(&attrs, 6, string_from_cf)?;
            let description = decoded(&attrs, 7, string_from_cf)?;
            let position = decoded(&attrs, 8, point_from_cf)?;
            let size = decoded(&attrs, 9, size_from_cf)?;
            let frame = match (position, size) {
                (Some((x, y)), Some((w, h))) if w >= 1.0 && h >= 1.0 => {
                    Some([x, y, w, h].map(f64::to_bits))
                }
                _ => None,
            };
            Some(NodeText {
                title,
                description,
                frame,
            })
        } else {
            None
        };
        if is_window {
            decoded(&attrs, 6, string_from_cf)?.hash(&mut hasher);
        }
        NodeFacts {
            role,
            subrole,
            value,
            enabled,
            selected,
            children: children.len(),
            text,
        }
        .hash(&mut hasher);
        drop(attrs);
        for child in children.into_iter().rev() {
            stack.push((child, in_page));
        }
    }
    Some(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Scripted reads against a fake clock that advances only when the watch
    /// sleeps.
    fn run(before: u64, reads: &[Option<u64>]) -> (PageChange, usize) {
        let clock = Cell::new(Duration::ZERO);
        let mut script = reads.iter().copied();
        let count = Cell::new(0);
        let outcome = watch(
            before,
            || {
                count.set(count.get() + 1);
                script.next().unwrap_or(Some(before))
            },
            || SETTLE.saturating_sub(clock.get()),
            |d| clock.set(clock.get() + d),
        );
        (outcome, count.get())
    }

    #[test]
    fn a_page_that_never_changes_is_unchanged_after_the_settle_window() {
        let (outcome, reads) = run(7, &[]);
        assert_eq!(outcome, PageChange::Unchanged);
        let expected = SETTLE.as_millis().div_ceil(POLL.as_millis()) as usize + 1;
        assert_eq!(
            reads, expected,
            "reads until the window is spent, then stops"
        );
    }

    #[test]
    fn a_change_ends_the_watch_at_once() {
        let (outcome, reads) = run(7, &[Some(7), Some(9)]);
        assert_eq!(outcome, PageChange::Changed);
        assert_eq!(reads, 2);
    }

    #[test]
    fn a_failed_read_is_unknown_never_a_noop() {
        let (outcome, _) = run(7, &[Some(7), None]);
        assert_eq!(outcome, PageChange::Unknown);
    }

    #[test]
    fn the_digest_covers_the_state_a_handled_press_changes() {
        fn digest(facts: NodeFacts) -> u64 {
            let mut hasher = DefaultHasher::new();
            facts.hash(&mut hasher);
            hasher.finish()
        }
        let field = |subrole: &str, value: &str| NodeFacts {
            role: Some("AXTextField".into()),
            subrole: Some(subrole.into()),
            value: Some(value.into()),
            enabled: Some(true),
            selected: Some(false),
            children: 0,
            text: Some(NodeText {
                title: None,
                description: Some("Password".into()),
                frame: Some([1.0f64, 2.0, 3.0, 4.0].map(f64::to_bits)),
            }),
        };
        let masked = digest(field("AXSecureTextField", "••••"));
        assert_eq!(masked, digest(field("AXSecureTextField", "••••")));
        assert_ne!(masked, digest(field("AXSecureTextField", "•••••")));
        // A show/hide toggle swaps the secure field for a plain one.
        assert_ne!(masked, digest(field("", "••••")));

        // Browser chrome: a relabelled tab is the same node, a new tab is not.
        let tabs = |children: usize| NodeFacts {
            role: Some("AXTabGroup".into()),
            subrole: None,
            value: None,
            enabled: Some(true),
            selected: None,
            children,
            text: None,
        };
        assert_eq!(digest(tabs(2)), digest(tabs(2)));
        assert_ne!(digest(tabs(2)), digest(tabs(3)));
        // A navigation shows in the address bar before the new page loads.
        let omnibox = |url: &str| NodeFacts {
            role: Some("AXTextField".into()),
            subrole: None,
            value: Some(url.into()),
            enabled: Some(true),
            selected: None,
            children: 0,
            text: None,
        };
        assert_ne!(digest(omnibox("a.test/")), digest(omnibox("a.test/next")));
    }
}

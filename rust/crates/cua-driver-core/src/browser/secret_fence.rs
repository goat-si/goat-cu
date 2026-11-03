//! Read fence for tabs that hold values `browser_fill_secret` delivered.
//!
//! A fill marks its tab before the first write, so a lost reply still leaves
//! the mark. While any text field in the tab's top-level document holds one
//! of the filled values, page reads of the tab (`get_browser_state`
//! snapshots and screenshots) and desktop observation of its window are
//! refused with `browser_secret_held`. The check runs in the page on every
//! read; a check that cannot run counts as holding. The mark clears when no
//! field holds a filled value any more or the tab shows another document.
//!
//! The fence keeps no value: only a loader id and salted 64-bit FNV-1a
//! digests that the page recomputes for its own field values.

use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use serde_json::{json, Value};

use super::cdp_ws::CdpConnection;
use super::engine::BrowserEngine;
use super::refusal::{BrowserRefusal, BrowserRefusalCode};
use super::types::ProcessFingerprint;
use crate::protocol::ToolResult;

/// One tab of one browser process.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FenceKey {
    pub pid: i64,
    pub cdp_target_id: String,
}

/// A salted digest of one delivered value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FenceDigest {
    pub salt: String,
    pub digest: String,
}

impl FenceDigest {
    pub(crate) fn of(salt: &str, value: &str) -> Self {
        Self {
            salt: salt.to_owned(),
            digest: digest(salt, value),
        }
    }
}

/// What the fence remembers about one filled tab.
#[derive(Debug, Clone)]
pub(crate) struct SecretFence {
    /// Install counter value; a newer install replaces the fence.
    pub id: u64,
    pub window_id: u64,
    pub fingerprint: ProcessFingerprint,
    /// The runtime session and target that filled, used to reach the tab
    /// for a desktop observation check.
    pub session: String,
    pub target_id: String,
    /// Loader of the top-level document the values went into.
    pub loader_id: String,
    pub digests: Vec<FenceDigest>,
}

#[derive(Default)]
struct FenceState {
    epoch: u64,
    fences: HashMap<FenceKey, SecretFence>,
}

#[derive(Default)]
pub(crate) struct SecretFences {
    state: Mutex<FenceState>,
}

/// An install a fill may roll back when it sent nothing.
pub(crate) struct FenceInstall {
    key: FenceKey,
    id: u64,
    previous: Option<SecretFence>,
}

impl SecretFences {
    /// Mark a tab before the first write. Values filled earlier into the
    /// same document stay part of the fence.
    pub(crate) fn install(&self, key: FenceKey, mut fence: SecretFence) -> FenceInstall {
        let mut state = self.state.lock().unwrap();
        state.epoch += 1;
        fence.id = state.epoch;
        let previous = state.fences.get(&key).cloned();
        if let Some(previous) = previous.as_ref() {
            if previous.loader_id == fence.loader_id {
                let mut digests = previous.digests.clone();
                digests.extend(fence.digests);
                fence.digests = digests;
            }
        }
        let id = fence.id;
        state.fences.insert(key.clone(), fence);
        FenceInstall { key, id, previous }
    }

    /// Undo an install whose fill reached nothing, unless another install
    /// replaced it meanwhile.
    pub(crate) fn roll_back(&self, install: FenceInstall) {
        let mut state = self.state.lock().unwrap();
        if state.fences.get(&install.key).map(|fence| fence.id) != Some(install.id) {
            return;
        }
        match install.previous {
            Some(previous) => {
                state.fences.insert(install.key, previous);
            }
            None => {
                state.fences.remove(&install.key);
            }
        }
    }

    fn get(&self, key: &FenceKey) -> Option<SecretFence> {
        self.state.lock().unwrap().fences.get(key).cloned()
    }

    /// Remove the fence a check proved clear, unless a newer fill replaced it.
    fn clear(&self, key: &FenceKey, id: u64) {
        let mut state = self.state.lock().unwrap();
        if state.fences.get(key).map(|fence| fence.id) == Some(id) {
            state.fences.remove(key);
        }
    }

    fn epoch(&self) -> u64 {
        self.state.lock().unwrap().epoch
    }

    /// Fences of `pid` (all when `None`) on `window_id` (any when `None`).
    fn matching(&self, pid: Option<i64>, window_id: Option<u64>) -> Vec<(FenceKey, SecretFence)> {
        self.state
            .lock()
            .unwrap()
            .fences
            .iter()
            .filter(|(key, fence)| {
                pid.is_none_or(|pid| key.pid == pid)
                    && window_id.is_none_or(|window| fence.window_id == window)
            })
            .map(|(key, fence)| (key.clone(), fence.clone()))
            .collect()
    }

    /// Whether a fence matching the filter was installed after `epoch`.
    fn installed_since(&self, epoch: u64, pid: Option<i64>, window_id: Option<u64>) -> bool {
        self.matching(pid, window_id)
            .iter()
            .any(|(_, fence)| fence.id > epoch)
    }

    fn installed_on_tab_since(&self, epoch: u64, key: &FenceKey) -> bool {
        self.get(key).is_some_and(|fence| fence.id > epoch)
    }

    #[cfg(test)]
    pub(crate) fn is_held(&self, pid: i64, cdp_target_id: &str) -> bool {
        self.get(&FenceKey {
            pid,
            cdp_target_id: cdp_target_id.to_owned(),
        })
        .is_some()
    }
}

/// Salted 64-bit FNV-1a over the UTF-8 bytes of `salt` then `value`, as
/// lower-case hex. [`HOLDS`] computes the same digest in the page.
pub(crate) fn digest(salt: &str, value: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in salt.as_bytes().iter().chain(value.as_bytes()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Whether any text field in the document (open shadow roots included)
/// holds a value whose salted digest is listed. Returns `true`, `false`,
/// or `"unknown"` when the document is too large to walk.
pub(crate) const HOLDS: &str = r#"function (pairs) {
  // CUA_SECRET_FENCE_HOLDS
  const encoder = new TextEncoder();
  const digest = (salt, text) => {
    let hash = 0xcbf29ce484222325n;
    for (const byte of encoder.encode(salt + text)) {
      hash ^= BigInt(byte);
      hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
    }
    return hash.toString(16).padStart(16, "0");
  };
  let seen = 0;
  const roots = [document];
  while (roots.length) {
    const walker = document.createTreeWalker(roots.pop(), NodeFilter.SHOW_ELEMENT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (++seen > 200000) return "unknown";
      if (node.shadowRoot) roots.push(node.shadowRoot);
      if ((node instanceof HTMLInputElement || node instanceof HTMLTextAreaElement) && node.value &&
          pairs.some((pair) => digest(pair[0], node.value) === pair[1])) return true;
    }
  }
  return false;
}"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FenceCheck {
    Held,
    Clear,
}

/// Check one fence through an attached session on its tab.
async fn check_on(conn: &CdpConnection, cdp_session: &str, fence: &SecretFence) -> FenceCheck {
    let Ok(tree) = conn
        .call(Some(cdp_session), "Page.getFrameTree", json!({}))
        .await
    else {
        return FenceCheck::Held;
    };
    let Some(loader) = tree
        .pointer("/frameTree/frame/loaderId")
        .and_then(Value::as_str)
        .filter(|loader| !loader.is_empty())
    else {
        return FenceCheck::Held;
    };
    if loader != fence.loader_id {
        return FenceCheck::Clear;
    }
    let pairs: Vec<Value> = fence
        .digests
        .iter()
        .map(|entry| json!([entry.salt, entry.digest]))
        .collect();
    let expression = format!("({HOLDS})({})", Value::Array(pairs));
    match conn
        .call(
            Some(cdp_session),
            "Runtime.evaluate",
            json!({ "expression": expression, "returnByValue": true }),
        )
        .await
    {
        Ok(reply)
            if reply.get("exceptionDetails").is_none()
                && reply.pointer("/result/value") == Some(&Value::Bool(false)) =>
        {
            FenceCheck::Clear
        }
        _ => FenceCheck::Held,
    }
}

pub(crate) fn held_refusal(context: &str) -> BrowserRefusal {
    BrowserRefusal::new(
        BrowserRefusalCode::BrowserSecretHeld,
        format!(
            "{context}: a field in this browser tab holds a value the host filled. Page reads \
             of the tab and observation of its window stay off until the page submits or \
             navigates, or the field is cleared. Nothing was read."
        ),
    )
}

/// A refusal at the registry boundary is an error result, like every other
/// dispatcher refusal, so a batch stops at it.
fn dispatch_refusal(refusal: BrowserRefusal) -> ToolResult {
    let mut result = refusal.to_tool_result();
    result.is_error = Some(true);
    result
}

impl BrowserEngine {
    /// Before a page read of one tab: refuses while a fence on it holds.
    /// Returns the epoch to pass to [`Self::confirm_tab_read`].
    pub(crate) async fn admit_tab_read(
        &self,
        session: &str,
        target_id: &str,
        tab_id: &str,
    ) -> Result<u64, BrowserRefusal> {
        let epoch = self.secret_fences.epoch();
        // Unknown targets and tabs are refused by the read itself.
        let Ok(record) = self.store.get_target(session, target_id) else {
            return Ok(epoch);
        };
        let Some(tab) = record.tabs.get(tab_id) else {
            return Ok(epoch);
        };
        let key = FenceKey {
            pid: record.pid,
            cdp_target_id: tab.cdp_target_id.clone(),
        };
        let Some(fence) = self.secret_fences.get(&key) else {
            return Ok(epoch);
        };
        let check = match self.connection_for_record(session, &record).await {
            Ok(conn) => match self.attach(&conn, &key.cdp_target_id).await {
                Ok(cdp_session) => check_on(&conn, &cdp_session, &fence).await,
                Err(_) => FenceCheck::Held,
            },
            Err(_) => FenceCheck::Held,
        };
        match check {
            FenceCheck::Clear => {
                self.secret_fences.clear(&key, fence.id);
                Ok(epoch)
            }
            FenceCheck::Held => Err(held_refusal("get_browser_state was refused")),
        }
    }

    /// After a page read: a fill that marked the tab meanwhile discards it.
    pub(crate) fn confirm_tab_read(
        &self,
        session: &str,
        target_id: &str,
        tab_id: &str,
        epoch: u64,
    ) -> Result<(), BrowserRefusal> {
        let Ok(record) = self.store.get_target(session, target_id) else {
            return Ok(());
        };
        let Some(tab) = record.tabs.get(tab_id) else {
            return Ok(());
        };
        let key = FenceKey {
            pid: record.pid,
            cdp_target_id: tab.cdp_target_id.clone(),
        };
        if self.secret_fences.installed_on_tab_since(epoch, &key) {
            self.store
                .invalidate_tab_snapshots(session, target_id, tab_id);
            return Err(held_refusal(
                "the host filled a field in this tab while it was being read, so the read was \
                 discarded",
            ));
        }
        Ok(())
    }

    /// Before a desktop observation of `pid`'s `window_id` (a whole
    /// process when `window_id` is `None`, the whole desktop when `pid` is
    /// `None`): refuses while a fence on a matching window holds.
    async fn admit_window_observation(
        &self,
        tool: &str,
        pid: Option<i64>,
        window_id: Option<u64>,
    ) -> Result<u64, BrowserRefusal> {
        let epoch = self.secret_fences.epoch();
        for (key, fence) in self.secret_fences.matching(pid, window_id) {
            // A browser that quit (or a reused pid) shows no filled field.
            match self.platform.process_fingerprint(key.pid).await {
                Ok(now) if fence.fingerprint.matches(&now) => {}
                _ => {
                    self.secret_fences.clear(&key, fence.id);
                    continue;
                }
            }
            let check = match self.store.get_target(&fence.session, &fence.target_id) {
                Ok(record) => match self.connection_for_record(&fence.session, &record).await {
                    Ok(conn) => match self.attach(&conn, &key.cdp_target_id).await {
                        Ok(cdp_session) => check_on(&conn, &cdp_session, &fence).await,
                        Err(_) => FenceCheck::Held,
                    },
                    Err(_) => FenceCheck::Held,
                },
                Err(_) => FenceCheck::Held,
            };
            match check {
                FenceCheck::Clear => self.secret_fences.clear(&key, fence.id),
                FenceCheck::Held => {
                    return Err(held_refusal(&format!(
                        "{tool} was refused: it would show browser process {} window {}",
                        key.pid, fence.window_id
                    )))
                }
            }
        }
        Ok(epoch)
    }
}

/// Desktop tools that read a window's pixels or accessibility content, or
/// start a capture of it. Each is fenced by the window it names; one that
/// names no process observes the whole desktop.
const OBSERVATION_TOOLS: &[&str] = &[
    "get_window_state",
    "get_desktop_state",
    "get_accessibility_tree",
    "zoom",
    "screenshot",
    "parse_visual_regions",
    "verify_state",
    "start_recording",
    "page",
];

/// Admission of one dispatched call against the read fence, taken by the
/// registry before the tool runs.
pub(crate) struct ObservationTicket {
    engines: Vec<(Weak<BrowserEngine>, u64)>,
    pid: Option<i64>,
    window_id: Option<u64>,
}

impl ObservationTicket {
    /// After the tool ran: a fill that marked a matching window meanwhile
    /// discards what the tool observed.
    pub(crate) fn confirm(&self, tool: &str) -> Result<(), ToolResult> {
        for (engine, epoch) in &self.engines {
            let Some(engine) = engine.upgrade() else {
                continue;
            };
            if engine
                .secret_fences
                .installed_since(*epoch, self.pid, self.window_id)
            {
                return Err(dispatch_refusal(held_refusal(&format!(
                    "{tool}: the host filled a field in a browser window this call observed, \
                     so its result was discarded"
                ))));
            }
        }
        Ok(())
    }
}

fn call_target(args: &Value) -> (Option<i64>, Option<u64>) {
    let target = args
        .get("target")
        .filter(|target| target.get("kind").and_then(Value::as_str) == Some("window"));
    let pid = args.get("pid").and_then(Value::as_i64).or_else(|| {
        target
            .and_then(|target| target.get("pid"))
            .and_then(Value::as_i64)
    });
    let window_id = args.get("window_id").and_then(Value::as_u64).or_else(|| {
        target
            .and_then(|target| target.get("window_id"))
            .and_then(Value::as_u64)
    });
    (pid, window_id)
}

/// The registry's fence admission for `tool`: `browser_fill_secret` runs
/// only as a direct call while no recording is on, and desktop observation
/// of a fenced window is refused while its fence holds.
pub(crate) async fn admit_dispatch(
    tool: &str,
    args: &Value,
    engines: &[Weak<BrowserEngine>],
    recording: bool,
) -> Result<Option<ObservationTicket>, ToolResult> {
    if tool == super::secret_fill::TOOL_NAME {
        if crate::step_pacing::in_batch() {
            return Err(dispatch_refusal(super::secret_fill::batch_refusal()));
        }
        if recording {
            return Err(dispatch_refusal(super::secret_fill::fill_refusal(
                "recording_active",
                "a recording is running, and it would capture the filled fields; stop it first",
            )));
        }
        return Ok(None);
    }
    if !OBSERVATION_TOOLS.contains(&tool) || engines.is_empty() {
        return Ok(None);
    }
    let (pid, window_id) = call_target(args);
    let mut admitted = Vec::new();
    for weak in engines {
        let Some(engine) = weak.upgrade() else {
            continue;
        };
        let epoch = engine
            .admit_window_observation(tool, pid, window_id)
            .await
            .map_err(dispatch_refusal)?;
        admitted.push((weak.clone(), epoch));
    }
    Ok(Some(ObservationTicket {
        engines: admitted,
        pid,
        window_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_is_salted_fnv1a_over_utf8() {
        // FNV-1a 64 of the empty input is the offset basis.
        assert_eq!(digest("", ""), "cbf29ce484222325");
        // Published FNV-1a 64 test vector for "a".
        assert_eq!(digest("", "a"), "af63dc4c8601ec8c");
        assert_eq!(digest("s", "a"), digest("", "sa"));
        assert_ne!(digest("s1", "pw"), digest("s2", "pw"));
    }

    #[test]
    fn a_rolled_back_install_restores_the_earlier_fence_of_the_same_tab() {
        let fences = SecretFences::default();
        let key = FenceKey {
            pid: 1,
            cdp_target_id: "T1".into(),
        };
        let fence = |loader: &str, value: &str| SecretFence {
            id: 0,
            window_id: 7,
            fingerprint: ProcessFingerprint {
                pid: 1,
                start_time: None,
                executable: None,
            },
            session: "s".into(),
            target_id: "t".into(),
            loader_id: loader.into(),
            digests: vec![FenceDigest::of("salt", value)],
        };
        fences.install(key.clone(), fence("L1", "first"));
        let second = fences.install(key.clone(), fence("L1", "second"));
        assert_eq!(fences.get(&key).unwrap().digests.len(), 2, "same document");
        fences.roll_back(second);
        assert_eq!(fences.get(&key).unwrap().digests.len(), 1);
        let other_document = fences.install(key.clone(), fence("L2", "third"));
        assert_eq!(fences.get(&key).unwrap().digests.len(), 1);
        fences.roll_back(other_document);
        assert_eq!(fences.get(&key).unwrap().loader_id, "L1");
    }
}

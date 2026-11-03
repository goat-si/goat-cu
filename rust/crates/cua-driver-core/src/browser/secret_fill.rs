//! `browser_fill_secret`: the host fills a saved login, card or one-time
//! code into a tab of the person's own browser without the agent seeing it.
//!
//! Admission matches the other page tools (an exact existing-profile tab
//! binding, the capability manifest, `origin_scope` with exactly one
//! origin), and the call must also carry the host authority: the key the
//! host gave this driver in [`HOST_KEY_ENV`] at startup, passed as
//! [`HOST_AUTHORITY_ARG`]. The host strips that argument from every agent
//! call it relays, and the driver compares it in constant time, so an agent
//! cannot fill even through a relay that forgot to strip it.
//!
//! One fixed in-page function per kind finds the fields of the focused form
//! (or of the form holding the `ref` hint) and refuses, with nothing sent,
//! when they are missing or ambiguous. Each write is its own evaluation on
//! the document's own handle that first rechecks the origin against the
//! scope and the document and form identity, then sets the value. A
//! navigation destroys that handle, so a later write cannot reach another
//! document. Replies carry delivered flags, a verdict and a refusal reason:
//! never a value. The tab is fenced for reads before the first write (see
//! [`super::secret_fence`]).

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use serde_json::{json, Map, Value};
use zeroize::Zeroizing;

use super::engine::BrowserEngine;
use super::origin_scope::OriginScope;
use super::refusal::{BrowserRefusal, BrowserRefusalCode};
use super::secret_fence::{FenceDigest, FenceKey, SecretFence};
use super::store::FrameKind;
use super::types::EndpointAccessClass;
use crate::protocol::ToolResult;
use super::secret_fill_page::{
    find_message, find_reason, page_function, FIELDS, FIND, MUTATE, READ_BACK, RELEASE,
};
use crate::tool::{ProtectedResourceOwnership, Tool, ToolDef};

pub const TOOL_NAME: &str = "browser_fill_secret";
/// The call argument that carries the host authority.
pub const HOST_AUTHORITY_ARG: &str = "host_authority";
/// The environment variable the host starts the driver with: a random key
/// of at least [`MIN_HOST_KEY_CHARS`] characters.
pub const HOST_KEY_ENV: &str = "CUA_DRIVER_SECRET_FILL_HOST_KEY";
pub const MIN_HOST_KEY_CHARS: usize = 32;
/// The arguments that carry secrets: withheld from recordings.
pub const SECRET_ARGS: &[&str] = &["values", HOST_AUTHORITY_ARG];
const MAX_VALUE_CHARS: usize = 512;

static HOST_KEY: OnceLock<Option<Zeroizing<String>>> = OnceLock::new();

fn read_host_key() -> Option<Zeroizing<String>> {
    std::env::var(HOST_KEY_ENV)
        .ok()
        .map(Zeroizing::new)
        .filter(|key| key.chars().count() >= MIN_HOST_KEY_CHARS)
}

/// Read the host key once and remove it from this process's environment, so
/// processes the driver starts do not inherit it. Call at startup, before
/// other threads run.
pub fn capture_host_key_from_env() {
    HOST_KEY.get_or_init(read_host_key);
    std::env::remove_var(HOST_KEY_ENV);
}

fn process_host_key() -> Option<&'static str> {
    HOST_KEY
        .get_or_init(read_host_key)
        .as_ref()
        .map(|key| key.as_str())
}

/// Compare without stopping at the first differing byte.
fn same_key(presented: &str, expected: &str) -> bool {
    let (presented, expected) = (presented.as_bytes(), expected.as_bytes());
    if presented.len() != expected.len() {
        return false;
    }
    presented
        .iter()
        .zip(expected)
        .fold(0u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

pub(crate) fn batch_refusal() -> BrowserRefusal {
    BrowserRefusal::new(
        BrowserRefusalCode::BrowserBatchRefused,
        "browser_fill_secret is reachable only by a direct call from the host, not from \
         run_steps or run_script. Nothing was sent.",
    )
}

pub(crate) fn fill_refusal(reason: &str, message: &str) -> BrowserRefusal {
    BrowserRefusal::new(
        BrowserRefusalCode::BrowserSecretFillRefused,
        format!("browser_fill_secret: {message}."),
    )
    .with_detail(json!({ "reason": reason }))
}

fn host_only_refusal() -> BrowserRefusal {
    BrowserRefusal::new(
        BrowserRefusalCode::BrowserSecretFillHostOnly,
        "browser_fill_secret is for the host application only; ask the person to fill the \
         field. Nothing was sent.",
    )
}

/// Chromium's answer for a call on a handle whose document is gone: the
/// call did not run.
fn handle_gone(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("Could not find object with given id")
        || message.contains("Cannot find context with specified id")
}

fn invalid(detail: &str) -> ToolResult {
    ToolResult::error(format!("browser_fill_secret: {detail}")).with_structured(json!({
        "code": "invalid_arguments",
        "detail": detail,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FillKind {
    Password,
    Login,
    Card,
    OneTimeCode,
}

impl FillKind {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "password" => Some(Self::Password),
            "login" => Some(Self::Login),
            "card" => Some(Self::Card),
            "one_time_code" => Some(Self::OneTimeCode),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::Login => "login",
            Self::Card => "card",
            Self::OneTimeCode => "one_time_code",
        }
    }

    /// (accepted value keys, required value keys)
    fn value_keys(self) -> (&'static [&'static str], &'static [&'static str]) {
        match self {
            Self::Password => (&["password"], &["password"]),
            Self::Login => (&["username", "password"], &["username", "password"]),
            Self::Card => (
                &["number", "exp_month", "exp_year", "cvc", "name"],
                &["number", "exp_month", "exp_year", "cvc"],
            ),
            Self::OneTimeCode => (&["code"], &["code"]),
        }
    }
}

struct FillValues {
    values: Vec<(&'static str, Zeroizing<String>)>,
}

impl FillValues {
    /// Parse `values` for `kind`. Errors name a key, never a value.
    fn parse(kind: FillKind, raw: Option<&Value>) -> Result<Self, String> {
        let Some(Value::Object(raw)) = raw else {
            return Err("values must be an object".into());
        };
        let (accepted, required) = kind.value_keys();
        if let Some(extra) = raw.keys().find(|key| !accepted.contains(&key.as_str())) {
            return Err(format!(
                "values.{extra} is not accepted for kind {}",
                kind.as_str()
            ));
        }
        let mut values = Vec::new();
        for key in accepted {
            let Some(value) = raw.get(*key) else {
                if required.contains(key) {
                    return Err(format!("values.{key} is required for kind {}", kind.as_str()));
                }
                continue;
            };
            let Some(value) = value.as_str() else {
                return Err(format!("values.{key} must be a string"));
            };
            let count = value.chars().count();
            if count == 0 || count > MAX_VALUE_CHARS {
                return Err(format!(
                    "values.{key} must hold 1 to {MAX_VALUE_CHARS} characters"
                ));
            }
            values.push((*key, Zeroizing::new(value.to_owned())));
        }
        let parsed = Self { values };
        if kind == FillKind::Card {
            let month = parsed.get("exp_month").unwrap_or_default();
            if !(1..=2).contains(&month.len())
                || !month.bytes().all(|b| b.is_ascii_digit())
                || !(1..=12).contains(&month.parse::<u8>().unwrap_or(0))
            {
                return Err("values.exp_month must be a month number from 1 to 12".into());
            }
            let year = parsed.get("exp_year").unwrap_or_default();
            if !matches!(year.len(), 2 | 4) || !year.bytes().all(|b| b.is_ascii_digit()) {
                return Err("values.exp_year must be two or four digits".into());
            }
        }
        Ok(parsed)
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value.as_str())
    }

    fn month(&self) -> String {
        format!(
            "{:02}",
            self.get("exp_month")
                .and_then(|month| month.parse::<u8>().ok())
                .unwrap_or(0)
        )
    }

    fn full_year(&self) -> String {
        let year = self.get("exp_year").unwrap_or_default();
        if year.len() == 2 {
            format!("20{year}")
        } else {
            year.to_owned()
        }
    }

    /// Arguments every in-page write and read-back receives besides the
    /// field's own value.
    fn extra(&self) -> Value {
        json!({
            "username": self.get("username").unwrap_or_default(),
            "month": self.month(),
            "year": self.get("exp_year").unwrap_or_default(),
            "full_year": self.full_year(),
        })
    }
}

/// One planned write: the in-page field name, the value, and whether the
/// value is a secret the read fence must watch.
struct Write {
    field: &'static str,
    value: Zeroizing<String>,
    secret: bool,
}

/// The writes for the fields the page offered, in fill order.
fn plan(kind: FillKind, values: &FillValues, offered: &[String]) -> Result<Vec<Write>, ()> {
    let has = |field: &str| offered.iter().any(|offered| offered == field);
    let take = |key: &str| Zeroizing::new(values.get(key).unwrap_or_default().to_owned());
    let secret = |field: &'static str, key: &str| Write {
        field,
        value: take(key),
        secret: true,
    };
    let writes = match kind {
        FillKind::Password => vec![secret("password", "password")],
        FillKind::Login => vec![secret("username", "username"), secret("password", "password")],
        FillKind::OneTimeCode => vec![secret("code", "code")],
        FillKind::Card => {
            let mut writes = vec![secret("number", "number")];
            let month = values.month();
            let year = values.get("exp_year").unwrap_or_default();
            for (field, value) in [
                ("expiry", format!("{month}/{year}")),
                ("month", month.clone()),
                ("year", year.to_owned()),
            ] {
                if has(field) {
                    writes.push(Write {
                        field,
                        value: Zeroizing::new(value),
                        secret: false,
                    });
                }
            }
            writes.push(secret("cvc", "cvc"));
            if has("name") && values.get("name").is_some() {
                writes.push(secret("name", "name"));
            }
            writes
        }
    };
    if writes.iter().all(|write| has(write.field)) {
        Ok(writes)
    } else {
        Err(())
    }
}

// ── The tool ───────────────────────────────────────────────────────────────

enum HostKey {
    Process,
    #[cfg(test)]
    Fixed(Option<String>),
}

pub struct BrowserFillSecretTool {
    def: ToolDef,
    engine: Arc<BrowserEngine>,
    host_key: HostKey,
}

impl BrowserFillSecretTool {
    pub fn new(engine: Arc<BrowserEngine>) -> Self {
        Self {
            def: definition(),
            engine,
            host_key: HostKey::Process,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_host_key(engine: Arc<BrowserEngine>, key: Option<&str>) -> Self {
        Self {
            def: definition(),
            engine,
            host_key: HostKey::Fixed(key.map(str::to_owned)),
        }
    }

    fn host_authorized(&self, args: &Value) -> bool {
        let expected = match &self.host_key {
            HostKey::Process => process_host_key(),
            #[cfg(test)]
            HostKey::Fixed(key) => key.as_deref(),
        };
        match (
            expected,
            args.get(HOST_AUTHORITY_ARG).and_then(Value::as_str),
        ) {
            (Some(expected), Some(presented)) => same_key(presented, expected),
            _ => false,
        }
    }
}

fn definition() -> ToolDef {
    ToolDef {
        name: TOOL_NAME.into(),
        description: "Host only: fill a saved password, login (username and password), card \
            or one-time code into the focused form (or the form holding ref) of a tab in the \
            person's own browser, without returning any value. Requires the host_authority \
            the driver was started with; an agent's call is refused with \
            browser_secret_fill_host_only and nothing is sent. Afterwards page reads of the \
            tab and observation of its window are refused with browser_secret_held until \
            the page submits or navigates, or the fields are cleared."
            .into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "target_id": {
                    "type": "string",
                    "description": "Opaque browser target id minted by get_browser_state."
                },
                "tab_id": {
                    "type": "string",
                    "description": "Opaque tab id from get_browser_state."
                },
                "session": super::required_session_schema(),
                "origin_scope": super::origin_scope::schema(),
                "kind": {
                    "type": "string",
                    "enum": ["password", "login", "card", "one_time_code"],
                    "description": "password: the focused (or ref) password field. login: the \
                        form's one username and one password field. card: the form's cc-* \
                        fields. one_time_code: the form's one-time-code field."
                },
                "values": {
                    "type": "object",
                    "description": "The secret values, never echoed or recorded. password: \
                        {password}. login: {username, password}. card: {number, exp_month, \
                        exp_year, cvc, name?}. one_time_code: {code}.",
                    "properties": {
                        "password": { "type": "string" },
                        "username": { "type": "string" },
                        "number": { "type": "string" },
                        "exp_month": { "type": "string" },
                        "exp_year": { "type": "string" },
                        "cvc": { "type": "string" },
                        "name": { "type": "string" },
                        "code": { "type": "string" }
                    },
                    "additionalProperties": false
                },
                "ref": {
                    "type": "string",
                    "description": "Optional main-frame field ref from get_browser_state whose \
                        form to fill (for password, the field itself). Without it, the focused \
                        field's form is filled."
                },
                "host_authority": {
                    "type": "string",
                    "description": "Set by the host application only."
                }
            },
            "required": ["target_id", "tab_id", "session", "origin_scope", "kind", "values", "host_authority"],
            "additionalProperties": true
        }),
        read_only: false,
        destructive: false,
        idempotent: false,
        open_world: true,
    }
}

/// The structured outcome of one fill: delivered flags and a verdict, never
/// a value.
struct Outcome<'a> {
    kind: FillKind,
    writes: &'a [Write],
    delivered: usize,
    uncertain: bool,
    verdict: &'static str,
    held: bool,
}

impl Outcome<'_> {
    fn delivered_map(&self) -> Value {
        let mut map = Map::new();
        for (index, write) in self.writes.iter().enumerate() {
            map.insert(write.field.to_owned(), Value::Bool(index < self.delivered));
        }
        Value::Object(map)
    }

    fn finish(self, refusal: Option<BrowserRefusal>) -> ToolResult {
        let delivered = self.delivered_map();
        let uncertain_field = self
            .uncertain
            .then(|| self.writes.get(self.delivered).map(|write| write.field))
            .flatten();
        match refusal {
            None => ToolResult::text(format!(
                "browser_fill_secret: filled {} field(s) ({})",
                self.delivered, self.verdict
            ))
            .with_structured(json!({
                "status": "ok",
                "kind": self.kind.as_str(),
                "verdict": self.verdict,
                "delivered": delivered,
                "secret_held": self.held,
            })),
            Some(refusal) => {
                let mut result = refusal.to_tool_result();
                if let Some(Value::Object(structured)) = result.structured_content.as_mut() {
                    structured.insert("kind".into(), json!(self.kind.as_str()));
                    structured.insert("verdict".into(), json!(self.verdict));
                    structured.insert("delivered".into(), delivered);
                    structured.insert("uncertain_field".into(), json!(uncertain_field));
                    structured.insert("secret_held".into(), json!(self.held));
                }
                result
            }
        }
    }
}

#[async_trait]
impl Tool for BrowserFillSecretTool {
    fn def(&self) -> &ToolDef {
        &self.def
    }

    async fn protected_resource_ownership(
        &self,
        adapter_id: &str,
        args: &Value,
    ) -> ProtectedResourceOwnership {
        if adapter_id == "browser_bound_input" {
            super::tools::browser_resource_ownership(&self.engine, args)
        } else {
            ProtectedResourceOwnership::UserOwned
        }
    }

    async fn protected_resource_scope(
        &self,
        adapter_id: &str,
        args: &Value,
    ) -> Result<Option<Value>, String> {
        if adapter_id == "browser_bound_input" {
            super::tools::browser_protected_resource_scope(&self.engine, args, TOOL_NAME).await
        } else {
            Ok(None)
        }
    }

    async fn invoke(&self, args: Value) -> ToolResult {
        if crate::step_pacing::in_batch() {
            return batch_refusal().to_tool_result();
        }
        if !self.host_authorized(&args) {
            return host_only_refusal().to_tool_result();
        }
        let scope = match super::origin_scope::parse(&args) {
            Ok(Some(scope)) => scope,
            Ok(None) => {
                return BrowserRefusal::new(
                    BrowserRefusalCode::BrowserOriginScopeRequired,
                    "browser_fill_secret requires origin_scope with the one origin to fill. \
                     Nothing was sent.",
                )
                .to_tool_result()
            }
            Err(error) => return error,
        };
        let Some(origin) = scope.single_origin().map(str::to_owned) else {
            return invalid("origin_scope must name exactly one origin");
        };
        let Some(kind) = args
            .get("kind")
            .and_then(Value::as_str)
            .and_then(FillKind::parse)
        else {
            return invalid("kind must be password, login, card or one_time_code");
        };
        let values = match FillValues::parse(kind, args.get("values")) {
            Ok(values) => values,
            Err(detail) => return invalid(&detail),
        };
        let (Some(target_id), Some(tab_id)) = (
            args.get("target_id").and_then(Value::as_str),
            args.get("tab_id").and_then(Value::as_str),
        ) else {
            return invalid("target_id and tab_id are required");
        };
        let session = match super::tools::require_explicit_session(&args) {
            Ok(session) => session,
            Err(error) => return error,
        };
        let hint = args.get("ref").and_then(Value::as_str);
        let _mutation = match self.engine.lock_mutation(&session, target_id, tab_id).await {
            Ok(guard) => guard,
            Err(refusal) => return refusal.to_tool_result(),
        };
        self.fill(&session, target_id, tab_id, &scope, &origin, kind, &values, hint)
            .await
    }
}

impl BrowserFillSecretTool {
    #[allow(clippy::too_many_arguments)]
    async fn fill(
        &self,
        session: &str,
        target_id: &str,
        tab_id: &str,
        scope: &OriginScope,
        origin: &str,
        kind: FillKind,
        values: &FillValues,
        hint: Option<&str>,
    ) -> ToolResult {
        let validated = match self
            .engine
            .revalidate_for_mutation(session, target_id, Some(tab_id), Some(scope))
            .await
        {
            Ok(validated) => validated,
            Err(refusal) => return refusal.to_tool_result(),
        };
        if validated.record.endpoint_access_class != EndpointAccessClass::ExistingProfileApproved {
            return fill_refusal(
                "unsupported_target",
                "only a tab of the person's own browser attached with its existing profile \
                 can be filled; nothing was sent",
            )
            .to_tool_result();
        }
        let conn = &validated.conn;
        let cdp = validated.cdp_session.as_str();
        let Some(loader_id) = conn
            .call(Some(cdp), "Page.getFrameTree", json!({}))
            .await
            .ok()
            .and_then(|tree| {
                tree.pointer("/frameTree/frame/loaderId")
                    .and_then(Value::as_str)
                    .filter(|loader| !loader.is_empty())
                    .map(str::to_owned)
            })
        else {
            return BrowserRefusal::new(
                BrowserRefusalCode::BrowserRouteUnavailable,
                "browser_fill_secret could not prove the tab's document; nothing was sent",
            )
            .to_tool_result();
        };

        // Find the fields: on the ref's node, or on the focused field.
        let kind_arg = json!(kind.as_str());
        let found = match hint {
            Some(hint) => {
                let entry = match self
                    .engine
                    .store
                    .resolve_ref(session, target_id, tab_id, hint)
                {
                    Ok(entry) => entry,
                    Err(refusal) => return refusal.to_tool_result(),
                };
                let same_document = entry
                    .frame
                    .identity
                    .as_ref()
                    .is_none_or(|identity| identity.loader_id == loader_id);
                if entry.frame.kind != FrameKind::Main || !same_document {
                    return fill_refusal(
                        "field_in_frame",
                        "the ref is not a field of the tab's current top-level document; \
                         nothing was sent",
                    )
                    .to_tool_result();
                }
                let Some(object_id) = conn
                    .call(
                        Some(cdp),
                        "DOM.resolveNode",
                        json!({ "backendNodeId": entry.backend_node_id }),
                    )
                    .await
                    .ok()
                    .and_then(|resolved| {
                        resolved
                            .pointer("/object/objectId")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
                else {
                    return BrowserRefusal::new(
                        BrowserRefusalCode::BrowserRefStale,
                        "the ref's node no longer resolves in the live page; nothing was sent",
                    )
                    .to_tool_result();
                };
                conn.call(
                    Some(cdp),
                    "Runtime.callFunctionOn",
                    json!({
                        "objectId": object_id,
                        "functionDeclaration": page_function("kind, hinted", FIND),
                        "arguments": [{ "value": kind_arg }, { "value": true }],
                        "returnByValue": false,
                    }),
                )
                .await
            }
            None => {
                conn.call(
                    Some(cdp),
                    "Runtime.evaluate",
                    json!({
                        "expression": format!(
                            "({}).call(undefined, {kind_arg}, false)",
                            page_function("kind, hinted", FIND)
                        ),
                        "returnByValue": false,
                    }),
                )
                .await
            }
        };
        let found = match found {
            Ok(found) if found.get("exceptionDetails").is_none() => found,
            _ => {
                return fill_refusal(
                    "inspect_failed",
                    "the form could not be inspected; nothing was sent",
                )
                .to_tool_result()
            }
        };
        let Some(handle) = found
            .pointer("/result/objectId")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            let reason = find_reason(found.pointer("/result/value"));
            return fill_refusal(reason, &format!("{}; nothing was sent", find_message(reason)))
                .to_tool_result();
        };
        let result = self
            .write_fields(session, &validated, &handle, origin, kind, values, &loader_id)
            .await;
        let _ = conn
            .call(
                Some(cdp),
                "Runtime.callFunctionOn",
                json!({
                    "objectId": handle,
                    "functionDeclaration": RELEASE,
                    "returnByValue": true,
                }),
            )
            .await;
        result
    }

    #[allow(clippy::too_many_arguments)]
    async fn write_fields(
        &self,
        session: &str,
        validated: &super::engine::ValidatedTab,
        handle: &str,
        origin: &str,
        kind: FillKind,
        values: &FillValues,
        loader_id: &str,
    ) -> ToolResult {
        let conn = &validated.conn;
        let cdp = validated.cdp_session.as_str();
        let offered: Vec<String> = match conn
            .call(
                Some(cdp),
                "Runtime.callFunctionOn",
                json!({
                    "objectId": handle,
                    "functionDeclaration": FIELDS,
                    "returnByValue": true,
                }),
            )
            .await
        {
            Ok(reply) if reply.get("exceptionDetails").is_none() => reply
                .pointer("/result/value")
                .and_then(Value::as_array)
                .map(|fields| {
                    fields
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let Ok(writes) = plan(kind, values, &offered) else {
            return fill_refusal(
                "form_changed",
                "the form changed while it was inspected; nothing was sent",
            )
            .to_tool_result();
        };

        // Fence the tab before the first write: a lost reply may still have
        // delivered the value.
        let salt = uuid::Uuid::new_v4().simple().to_string();
        let digests = writes
            .iter()
            .filter(|write| write.secret)
            .map(|write| FenceDigest::of(&salt, &write.value))
            .collect();
        let install = self.engine.secret_fences.install(
            FenceKey {
                pid: validated.record.pid,
                cdp_target_id: validated.tab.cdp_target_id.clone(),
            },
            SecretFence {
                id: 0,
                window_id: validated.record.window_id,
                fingerprint: validated.record.fingerprint.clone(),
                session: session.to_owned(),
                target_id: validated.record.target_id.clone(),
                loader_id: loader_id.to_owned(),
                digests,
            },
        );

        let extra = values.extra();
        let mut delivered = 0usize;
        let mut stop: Option<(String, &'static str)> = None;
        let mut uncertain = false;
        for write in &writes {
            let reply = conn
                .call(
                    Some(cdp),
                    "Runtime.callFunctionOn",
                    json!({
                        "objectId": handle,
                        "functionDeclaration": page_function("which, value, origin, extra", MUTATE),
                        "arguments": [
                            { "value": write.field },
                            { "value": write.value.as_str() },
                            { "value": origin },
                            { "value": extra },
                        ],
                        "returnByValue": true,
                    }),
                )
                .await;
            let answer = match &reply {
                Ok(reply) if reply.get("exceptionDetails").is_none() => {
                    reply.pointer("/result/value").cloned()
                }
                // The handle no longer resolves: the document it belonged
                // to is gone, and the call ran nothing.
                Err(error) if handle_gone(error) => Some(json!("gone")),
                _ => None,
            };
            match answer {
                Some(Value::Bool(true)) => delivered += 1,
                Some(Value::String(answer))
                    if matches!(
                        answer.as_str(),
                        "changed" | "account" | "no_option" | "gone"
                    ) =>
                {
                    let (reason, message) = match answer.as_str() {
                        "account" => (
                            "account_mismatch".to_owned(),
                            "the username field shows another account",
                        ),
                        "no_option" => (
                            format!("no_option_{}", write.field),
                            "a list in the form has no single option for this value",
                        ),
                        "gone" => (
                            "document_changed".to_owned(),
                            "the tab left the document the form was found in",
                        ),
                        _ => (
                            "form_changed".to_owned(),
                            "the page, form or focus changed before this field was written",
                        ),
                    };
                    stop = Some((reason, message));
                    break;
                }
                _ => {
                    uncertain = true;
                    stop = Some((
                        "delivery_uncertain".to_owned(),
                        "the page did not confirm whether this field was written",
                    ));
                    break;
                }
            }
        }

        let verdict = if stop.is_none() {
            let expected: Map<String, Value> = writes
                .iter()
                .map(|write| (write.field.to_owned(), json!(write.value.as_str())))
                .collect();
            let checked = conn
                .call(
                    Some(cdp),
                    "Runtime.callFunctionOn",
                    json!({
                        "objectId": handle,
                        "functionDeclaration": page_function("origin, expected, extra", READ_BACK),
                        "arguments": [
                            { "value": origin },
                            { "value": Value::Object(expected) },
                            { "value": extra },
                        ],
                        "returnByValue": true,
                    }),
                )
                .await;
            match checked {
                Ok(reply)
                    if reply.get("exceptionDetails").is_none()
                        && reply.pointer("/result/value") == Some(&Value::Bool(true)) =>
                {
                    "confirmed"
                }
                _ => "unverifiable",
            }
        } else if delivered == 0 && !uncertain {
            "not_filled"
        } else {
            "unverifiable"
        };
        let held = delivered > 0 || uncertain;
        if !held {
            self.engine.secret_fences.roll_back(install);
        }
        let outcome = Outcome {
            kind,
            writes: &writes,
            delivered,
            uncertain,
            verdict,
            held,
        };
        let refusal = stop.map(|(reason, message)| {
            let sent = if delivered == 0 && !uncertain {
                "nothing was sent"
            } else {
                "see delivered for what reached the page"
            };
            fill_refusal(&reason, &format!("{message}; {sent}"))
        });
        outcome.finish(refusal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_errors_name_keys_and_never_values() {
        let sentinel = "SENTINEL-not-a-month";
        for (kind, raw) in [
            (FillKind::Card, json!({"number": "4", "exp_month": sentinel, "exp_year": "2030", "cvc": "1"})),
            (FillKind::Login, json!({"username": sentinel})),
            (FillKind::Password, json!({"password": sentinel, "code": sentinel})),
            (FillKind::OneTimeCode, json!({"code": 7})),
        ] {
            let error = FillValues::parse(kind, Some(&raw)).err().expect("refused");
            assert!(!error.contains("SENTINEL"), "{error}");
        }
        assert!(FillValues::parse(
            FillKind::Card,
            Some(&json!({"number": "4111", "exp_month": "7", "exp_year": "29", "cvc": "123"}))
        )
        .is_ok());
    }

    #[test]
    fn the_host_key_compares_whole_values() {
        assert!(same_key("k".repeat(40).as_str(), "k".repeat(40).as_str()));
        assert!(!same_key("k".repeat(40).as_str(), "k".repeat(39).as_str()));
        assert!(!same_key(&format!("{}x", "k".repeat(39)), "k".repeat(40).as_str()));
    }

    #[test]
    fn a_card_plan_fills_only_offered_expiry_fields_in_order() {
        let values = FillValues::parse(
            FillKind::Card,
            Some(&json!({"number": "4111", "exp_month": "7", "exp_year": "29", "cvc": "123", "name": "A B"})),
        )
        .unwrap();
        let offered = ["number", "month", "year", "cvc"].map(str::to_owned);
        let writes = plan(FillKind::Card, &values, &offered).unwrap();
        let order: Vec<_> = writes.iter().map(|write| (write.field, write.secret)).collect();
        assert_eq!(
            order,
            [("number", true), ("month", false), ("year", false), ("cvc", true)]
        );
        assert_eq!(writes[1].value.as_str(), "07");
        assert!(plan(FillKind::Login, &values, &offered).is_err());
    }
}


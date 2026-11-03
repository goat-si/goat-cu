//! `browser_fill_secret` and its read fence against the deterministic mock
//! CDP fixture: admission, per-write rechecks, partial and uncertain
//! delivery, and the reads the fence refuses.

use super::*;
use crate::browser::secret_fill::BrowserFillSecretTool;

const ORIGIN: &str = "https://fixture.test";
const HOST_KEY: &str = "host-key-0123456789abcdef0123456789abcdef";
const PASSWORD: &str = "SENTINEL-password-7f3a";
const USERNAME: &str = "SENTINEL-user-91c2";

fn refusal_code(result: &ToolResult) -> String {
    structured(result)["refusal"]["code"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn reason(result: &ToolResult) -> String {
    structured(result)["refusal"]["detail"]["reason"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

/// Everything a caller could see of a result.
fn rendered(result: &ToolResult) -> String {
    format!("{} {result:?}", serde_json::to_string(result).unwrap())
}

fn assert_no_secret(result: &ToolResult) {
    let text = rendered(result);
    for secret in [PASSWORD, USERNAME, HOST_KEY, "SENTINEL"] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
}

/// An existing-profile tab bound with the fixture's origin in scope.
async fn claimed(transport: &str, configure: impl FnOnce(&mut FixtureState)) -> (Fixture, String, String) {
    let (f, _) = protected_existing_profile_fixture().await;
    configure(&mut f.state.lock().unwrap());
    let prepare = BrowserPrepareTool::new(f.engine.clone())
        .invoke(json!({
            "pid": 1, "window_id": 7, "session": SESSION,
            "_transport_session_id": transport,
            "strategy": { "kind": "existing_profile" }
        }))
        .await;
    assert_eq!(structured(&prepare)["status"], "ok", "{}", structured(&prepare));
    let bound = GetBrowserStateTool::new(f.engine.clone())
        .invoke(json!({
            "pid": 1, "window_id": 7, "session": SESSION,
            "_transport_session_id": transport,
            "origin_scope": [ORIGIN],
        }))
        .await;
    let bound = structured(&bound).clone();
    assert_eq!(bound["status"], "ok", "{bound}");
    (
        f,
        bound["target_id"].as_str().unwrap().to_owned(),
        bound["tabs"][0]["tab_id"].as_str().unwrap().to_owned(),
    )
}

fn login_args(target: &str, tab: &str) -> Value {
    json!({
        "target_id": target, "tab_id": tab, "session": SESSION,
        "origin_scope": [ORIGIN],
        "kind": "login",
        "values": { "username": USERNAME, "password": PASSWORD },
        "host_authority": HOST_KEY,
    })
}

async fn fill(f: &Fixture, args: Value) -> ToolResult {
    BrowserFillSecretTool::with_host_key(f.engine.clone(), Some(HOST_KEY))
        .invoke(args)
        .await
}

fn login_page(st: &mut FixtureState) {
    st.fill.find = Some(Ok(vec!["username", "password"]));
    st.fill.read_back = true;
    st.fill.holds = Some(json!(true));
}

/// Calls of the fixed in-page functions (find and write).
fn page_function_calls(f: &Fixture) -> usize {
    f.state
        .lock()
        .unwrap()
        .calls
        .iter()
        .filter(|(_, method, params)| {
            let text = params.to_string();
            method.starts_with("Runtime.")
                && (text.contains("CUA_SECRET_FILL_FIND") || text.contains("CUA_SECRET_FILL_MUTATE"))
        })
        .count()
}

fn applied(f: &Fixture) -> Vec<(String, String)> {
    f.state.lock().unwrap().fill.applied.clone()
}

fn held(f: &Fixture) -> bool {
    f.engine.secret_fences.is_held(1, "T1")
}

async fn read(f: &Fixture, target: &str, tab: &str, extra: Value) -> ToolResult {
    let mut args = json!({
        "target_id": target, "tab_id": tab, "session": SESSION,
        "origin_scope": [ORIGIN],
    });
    args.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    GetBrowserStateTool::new(f.engine.clone()).invoke(args).await
}

#[tokio::test]
async fn a_login_fill_writes_both_fields_confirms_and_returns_no_value() {
    const TRANSPORT: &str = "transport-fill-login";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let result = fill(&f, login_args(&target, &tab)).await;
    let s = structured(&result).clone();
    assert_eq!(s["status"], "ok", "{s}");
    assert_eq!(s["verdict"], "confirmed");
    assert_eq!(s["delivered"], json!({"username": true, "password": true}));
    assert_eq!(s["secret_held"], true);
    assert_no_secret(&result);
    assert_eq!(
        applied(&f),
        [
            ("username".to_owned(), USERNAME.to_owned()),
            ("password".to_owned(), PASSWORD.to_owned())
        ]
    );
    // Every write carries the scope's origin for the in-page recheck.
    let writes: Vec<_> = recorded_calls(&f, "Runtime.callFunctionOn")
        .into_iter()
        .filter(|(_, params)| params.to_string().contains("CUA_SECRET_FILL_MUTATE"))
        .collect();
    assert_eq!(writes.len(), 2);
    for (_, params) in &writes {
        assert_eq!(params["arguments"][2]["value"], ORIGIN);
        assert!(params["objectId"].as_str().unwrap().starts_with("fill-handle-"));
    }
    assert!(held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn an_agent_call_is_refused_before_anything_reaches_the_page() {
    const TRANSPORT: &str = "transport-fill-host-only";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let mut unmarked = login_args(&target, &tab);
    unmarked.as_object_mut().unwrap().remove("host_authority");
    let mut forged = login_args(&target, &tab);
    forged["host_authority"] = json!("host-key-0123456789abcdef0123456789abcdeX");
    for args in [unmarked, forged] {
        let refused = fill(&f, args).await;
        assert_eq!(refusal_code(&refused), "browser_secret_fill_host_only");
        assert_no_secret(&refused);
    }
    // A driver started without a host key refuses every call.
    let keyless = BrowserFillSecretTool::with_host_key(f.engine.clone(), None)
        .invoke(login_args(&target, &tab))
        .await;
    assert_eq!(refusal_code(&keyless), "browser_secret_fill_host_only");
    assert_eq!(page_function_calls(&f), 0);
    assert!(!held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn a_fill_needs_one_origin_and_the_tab_inside_it() {
    const TRANSPORT: &str = "transport-fill-origin";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let mut unscoped = login_args(&target, &tab);
    unscoped.as_object_mut().unwrap().remove("origin_scope");
    assert_eq!(
        refusal_code(&fill(&f, unscoped).await),
        "browser_origin_scope_required"
    );
    let mut elsewhere = login_args(&target, &tab);
    elsewhere["origin_scope"] = json!(["https://other.example"]);
    let outside = fill(&f, elsewhere).await;
    assert_eq!(refusal_code(&outside), "browser_origin_outside_scope");
    let mut two = login_args(&target, &tab);
    two["origin_scope"] = json!([ORIGIN, "https://other.example"]);
    let wide = fill(&f, two).await;
    assert_eq!(structured(&wide)["code"], "invalid_arguments");
    for result in [&outside, &wide] {
        assert_no_secret(result);
    }
    assert_eq!(page_function_calls(&f), 0);
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn run_steps_and_recordings_never_carry_a_fill() {
    const TRANSPORT: &str = "transport-fill-batch";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let batched = crate::step_pacing::scope(
        crate::step_pacing::StepPacing::Batched,
        fill(&f, login_args(&target, &tab)),
    )
    .await;
    assert_eq!(refusal_code(&batched), "browser_batch_refused");

    // Through the registry: a running recording refuses the fill first.
    let mut registry = crate::tool::ToolRegistry::new();
    super::super::tools::register_browser_tools(&f.engine, &mut registry);
    let output = tempfile::tempdir().unwrap();
    registry
        .recording
        .start(output.path().to_str().unwrap(), false, None)
        .unwrap();
    let recording = registry
        .invoke_with_context("browser_fill_secret", login_args(&target, &tab), standard_context())
        .await;
    assert_eq!(refusal_code(&recording), "browser_secret_fill_refused");
    assert_eq!(reason(&recording), "recording_active");
    assert_no_secret(&recording);
    assert_eq!(page_function_calls(&f), 0);
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn a_navigation_before_a_write_stops_the_fill_and_reports_what_was_delivered() {
    const TRANSPORT: &str = "transport-fill-navigation";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        login_page(st);
        st.fill.navigate_before_write = Some(1);
    })
    .await;
    let result = fill(&f, login_args(&target, &tab)).await;
    let s = structured(&result).clone();
    assert_eq!(refusal_code(&result), "browser_secret_fill_refused", "{s}");
    assert_eq!(reason(&result), "document_changed");
    assert_eq!(s["delivered"], json!({"username": true, "password": false}));
    assert_eq!(s["verdict"], "unverifiable");
    assert_eq!(s["secret_held"], true);
    assert_eq!(applied(&f), [("username".to_owned(), USERNAME.to_owned())]);
    assert_no_secret(&result);
    // The tab now shows another document: the next read clears the fence.
    let after = read(&f, &target, &tab, json!({})).await;
    assert_eq!(structured(&after)["status"], "ok", "{}", structured(&after));
    assert!(!held(&f));
    assert_eq!(f.state.lock().unwrap().fill.holds_checks, 0);
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn a_navigation_before_the_first_write_sends_nothing_and_leaves_no_fence() {
    const TRANSPORT: &str = "transport-fill-navigation-first";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        login_page(st);
        st.fill.navigate_before_write = Some(0);
    })
    .await;
    let result = fill(&f, login_args(&target, &tab)).await;
    let s = structured(&result).clone();
    assert_eq!(reason(&result), "document_changed", "{s}");
    assert_eq!(s["delivered"], json!({"username": false, "password": false}));
    assert_eq!(s["verdict"], "not_filled");
    assert_eq!(s["secret_held"], false);
    assert!(applied(&f).is_empty());
    assert!(!held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn a_form_replaced_between_writes_is_not_written() {
    const TRANSPORT: &str = "transport-fill-form-replaced";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        login_page(st);
        st.fill.replace_form_before_write = Some(1);
    })
    .await;
    let result = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(reason(&result), "form_changed");
    assert_eq!(
        structured(&result)["delivered"],
        json!({"username": true, "password": false})
    );
    assert_eq!(applied(&f).len(), 1);
    assert!(held(&f));
    assert_no_secret(&result);
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn missing_or_ambiguous_fields_refuse_with_nothing_sent() {
    const TRANSPORT: &str = "transport-fill-fields";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        st.fill.find = Some(Err("ambiguous_password"));
    })
    .await;
    let ambiguous = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(reason(&ambiguous), "ambiguous_password");
    // A reason outside the closed set reads as no_form.
    f.state.lock().unwrap().fill.find = Some(Err("SENTINEL page text"));
    let odd = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(reason(&odd), "no_form");
    f.state.lock().unwrap().fill.find = None;
    let mut code = login_args(&target, &tab);
    code["kind"] = json!("one_time_code");
    code["values"] = json!({"code": "SENTINEL-123456"});
    assert_eq!(reason(&fill(&f, code).await), "no_form");
    for result in [&ambiguous, &odd] {
        assert_no_secret(result);
    }
    assert!(recorded_calls(&f, "Runtime.callFunctionOn")
        .iter()
        .all(|(_, params)| !params.to_string().contains("CUA_SECRET_FILL_MUTATE")));
    assert!(!held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn an_expiry_list_without_a_single_matching_option_stops_the_card_fill() {
    const TRANSPORT: &str = "transport-fill-card";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        st.fill.find = Some(Ok(vec!["number", "month", "year", "cvc"]));
        st.fill.answer_write = Some((1, "no_option"));
    })
    .await;
    let mut args = login_args(&target, &tab);
    args["kind"] = json!("card");
    args["values"] = json!({
        "number": "SENTINEL-4111", "exp_month": "7", "exp_year": "29", "cvc": "SENTINEL-321",
    });
    let result = fill(&f, args).await;
    let s = structured(&result).clone();
    assert_eq!(reason(&result), "no_option_month", "{s}");
    assert_eq!(
        s["delivered"],
        json!({"number": true, "month": false, "year": false, "cvc": false})
    );
    assert_eq!(s["secret_held"], true);
    assert_no_secret(&result);
    let month_write = recorded_calls(&f, "Runtime.callFunctionOn")
        .into_iter()
        .filter(|(_, params)| params.to_string().contains("CUA_SECRET_FILL_MUTATE"))
        .nth(1)
        .expect("the month write")
        .1;
    assert_eq!(month_write["arguments"][0]["value"], "month");
    assert_eq!(month_write["arguments"][1]["value"], "07");
    assert_eq!(
        month_write["arguments"][3]["value"],
        json!({"username": "", "month": "07", "year": "29", "full_year": "2029"})
    );
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn a_lost_reply_leaves_the_fence_held_until_the_field_lets_go() {
    const TRANSPORT: &str = "transport-fill-lost-reply";
    let (f, target, tab) = claimed(TRANSPORT, |st| {
        login_page(st);
        st.fill.lose_reply_of_write = Some(1);
        st.fill.holds = None;
    })
    .await;
    let result = fill(&f, login_args(&target, &tab)).await;
    let s = structured(&result).clone();
    assert_eq!(reason(&result), "delivery_uncertain", "{s}");
    assert_eq!(s["uncertain_field"], "password");
    assert_eq!(s["secret_held"], true);
    assert!(held(&f));
    // A check that cannot run counts as holding.
    let failed_check = read(&f, &target, &tab, json!({})).await;
    assert_eq!(refusal_code(&failed_check), "browser_secret_held");
    f.state.lock().unwrap().fill.holds = Some(json!("unknown"));
    let unknown = read(&f, &target, &tab, json!({})).await;
    assert_eq!(refusal_code(&unknown), "browser_secret_held");
    f.state.lock().unwrap().fill.holds = Some(json!(false));
    let cleared = read(&f, &target, &tab, json!({})).await;
    assert_eq!(structured(&cleared)["status"], "ok", "{}", structured(&cleared));
    assert!(!held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn reads_are_refused_while_a_field_holds_a_value_and_allowed_after() {
    const TRANSPORT: &str = "transport-fill-read-fence";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let filled = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(structured(&filled)["status"], "ok");
    let documents_before = recorded_calls(&f, "DOM.getDocument").len();
    for extra in [json!({}), json!({"include_screenshot": true}), json!({"snapshot_format": "semantic_v2"})] {
        let refused = read(&f, &target, &tab, extra).await;
        assert_eq!(refusal_code(&refused), "browser_secret_held");
    }
    assert_eq!(recorded_calls(&f, "DOM.getDocument").len(), documents_before);
    assert!(recorded_calls(&f, "Page.captureScreenshot").is_empty());
    // The check never carries a value: only salted digests.
    let checks: Vec<_> = recorded_calls(&f, "Runtime.evaluate")
        .into_iter()
        .filter(|(_, params)| params.to_string().contains("CUA_SECRET_FENCE_HOLDS"))
        .collect();
    assert_eq!(checks.len(), 3);
    for (_, params) in &checks {
        assert!(!params.to_string().contains("SENTINEL"));
    }

    // The field was cleared: reads work again.
    f.state.lock().unwrap().fill.holds = Some(json!(false));
    let read_again = read(&f, &target, &tab, json!({})).await;
    assert_eq!(structured(&read_again)["status"], "ok");

    // Filled again, then the tab navigated: cleared without a page check.
    f.state.lock().unwrap().fill.holds = Some(json!(true));
    let refilled = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(structured(&refilled)["status"], "ok");
    let checks_before = f.state.lock().unwrap().fill.holds_checks;
    f.state.lock().unwrap().main_loader = "L_MAIN_NEXT_DOCUMENT".into();
    let navigated = read(&f, &target, &tab, json!({})).await;
    assert_eq!(structured(&navigated)["status"], "ok", "{}", structured(&navigated));
    assert_eq!(f.state.lock().unwrap().fill.holds_checks, checks_before);
    crate::session::fire_session_end(TRANSPORT);
}

// ── Desktop observation through the registry ─────────────────────────────────

struct Observer(ToolDefHolder);
struct ToolDefHolder(crate::tool::ToolDef);

#[async_trait]
impl Tool for Observer {
    fn def(&self) -> &crate::tool::ToolDef {
        &self.0 .0
    }

    async fn invoke(&self, _args: Value) -> ToolResult {
        ToolResult::text("observed")
    }
}

fn observer(name: &str) -> Box<dyn Tool> {
    Box::new(Observer(ToolDefHolder(crate::tool::ToolDef {
        name: name.into(),
        description: "fixture observation".into(),
        input_schema: json!({"type": "object", "additionalProperties": true}),
        read_only: true,
        destructive: false,
        idempotent: true,
        open_world: false,
    })))
}

fn standard_context() -> Arc<crate::session_authorization::EffectiveAuthorizationContext> {
    crate::session_authorization::SessionAuthorizationRegistry::with_ceiling(
        crate::session_authorization::SessionModeCeiling::for_trusted_sessions(
            [crate::authorization::PermissionMode::Standard],
            false,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_secs(30),
        )
        .unwrap(),
    )
    .compatibility_context(crate::authorization::PermissionMode::Standard, None)
    .unwrap()
}

#[tokio::test]
async fn desktop_observation_of_the_filled_window_is_refused_while_held() {
    const TRANSPORT: &str = "transport-fill-desktop";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let mut registry = crate::tool::ToolRegistry::new();
    registry.register(observer("get_window_state"));
    registry.register(observer("get_desktop_state"));
    super::super::tools::register_browser_tools(&f.engine, &mut registry);
    let observe = |name: &'static str, args: Value| {
        let registry = &registry;
        async move {
            registry
                .invoke_with_context(name, args, standard_context())
                .await
        }
    };
    let window = json!({"pid": 1, "window_id": 7});
    let before = observe("get_window_state", window.clone()).await;
    assert_eq!(before.is_error, None, "{before:?}");

    let filled = fill(&f, login_args(&target, &tab)).await;
    assert_eq!(structured(&filled)["status"], "ok");
    for (name, args) in [
        ("get_window_state", window.clone()),
        ("get_window_state", json!({"target": {"kind": "window", "pid": 1, "window_id": 7}})),
        ("get_window_state", json!({"pid": 1})),
        ("get_desktop_state", json!({})),
    ] {
        let refused = observe(name, args.clone()).await;
        assert_eq!(refusal_code(&refused), "browser_secret_held", "{name} {args}");
        assert_eq!(refused.is_error, Some(true));
    }
    // Other windows and processes stay observable.
    for args in [json!({"pid": 1, "window_id": 8}), json!({"pid": 2, "window_id": 7})] {
        let other = observe("get_window_state", args.clone()).await;
        assert_eq!(other.is_error, None, "{args}: {other:?}");
    }
    f.state.lock().unwrap().fill.holds = Some(json!(false));
    let after = observe("get_desktop_state", json!({})).await;
    assert_eq!(after.is_error, None, "{after:?}");
    assert!(!held(&f));
    crate::session::fire_session_end(TRANSPORT);
}

#[tokio::test]
async fn telemetry_of_a_fill_holds_no_value() {
    const TRANSPORT: &str = "transport-fill-telemetry";
    let (f, target, tab) = claimed(TRANSPORT, login_page).await;
    let result = fill(&f, login_args(&target, &tab)).await;
    let response = crate::protocol::Response::ok(json!(1), serde_json::to_value(&result).unwrap());
    let observation = crate::server::ToolObservationTimer::start_with_operation(
        "browser_fill_secret".to_owned(),
        crate::server::tool_operation("browser_fill_secret", Some(&login_args(&target, &tab))),
        true,
        true,
        crate::server::StdioExecutionPath::DirectDaemon,
    )
    .finish(&response);
    let text = format!("{observation:?}");
    assert!(!text.contains("SENTINEL") && !text.contains(HOST_KEY), "{text}");
    assert!(observation.computer_action);
    crate::session::fire_session_end(TRANSPORT);
}


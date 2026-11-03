//! The fixed in-page functions of `browser_fill_secret`. No caller-supplied
//! script reaches the page: each kind runs only these.

use serde_json::Value;
// ── In-page functions ───────────────────────────────────────────────────────
//
// They run in the page's main world, the only world the reviewed
// existing-profile CDP surface reaches. Every write and read-back is called
// on the handle FIND returned, which belongs to the document FIND ran in:
// after a navigation it no longer resolves, so nothing reaches another
// document. Page script cannot redefine `location.origin`, so the origin
// check holds even where it redefines `self.origin`.

const PRELUDE: &str = r#"
  const visible = (field) => {
    if (!field.getClientRects().length) return false;
    for (let node = field; node; node = node.parentElement) {
      const style = getComputedStyle(node);
      if (style.display === "none" || style.visibility === "hidden" || style.visibility === "collapse" || Number(style.opacity) === 0) return false;
    }
    return true;
  };
  const focusedElement = () => {
    let focused = document.activeElement;
    while (focused && focused.shadowRoot && focused.shadowRoot.activeElement) focused = focused.shadowRoot.activeElement;
    return focused;
  };
  const tokens = (field) => (field.getAttribute("autocomplete") || "").toLowerCase().split(/\s+/).filter(Boolean);
  const matchOption = (field, which, month, year) => {
    const expected = which === "month" ? [String(Number(month)), String(Number(month)).padStart(2, "0")] : [year, year.slice(-2)];
    return [...field.options].filter((option) => !option.disabled && expected.some((value) => option.value.trim() === value || option.text.trim() === value));
  };
"#;

/// Finds the fields: returns the handle object, or a refusal reason string.
pub(super) const FIND: &str = r#"
  // CUA_SECRET_FILL_FIND
  const anchor = hinted ? this : focusedElement();
  if (anchor && (anchor.tagName === "IFRAME" || anchor.tagName === "FRAME")) return "field_in_frame";
  const make = (form, fields, extra) => Object.assign({ kind, doc: document, form, fields, hinted: !!hinted }, extra || {});
  if (kind === "password") {
    const field = anchor;
    if (!(field instanceof HTMLInputElement) || field.type !== "password" || field.ownerDocument !== document ||
        !field.isConnected || field.disabled || field.readOnly || !visible(field)) return "missing_password";
    return make(field.form, { password: field });
  }
  const form = anchor && anchor.form;
  if (!form) return "no_form";
  const usable = (field) => (field instanceof HTMLInputElement || (kind === "card" && field instanceof HTMLSelectElement)) &&
    field.form === form && field.ownerDocument === document && field.isConnected && !field.disabled && !field.readOnly && visible(field);
  const all = [...form.elements].filter(usable);
  if (kind === "login") {
    const passwords = all.filter((field) => field.type === "password");
    if (passwords.length !== 1) return passwords.length ? "ambiguous_password" : "missing_password";
    const password = passwords[0];
    const identifiers = all.filter((field) => ["text", "email", "tel"].includes(field.type));
    const named = identifiers.filter((field) => tokens(field).some((token) => token === "username" || token === "email"));
    const before = identifiers.filter((field) => field.compareDocumentPosition(password) & Node.DOCUMENT_POSITION_FOLLOWING);
    const candidates = named.length ? named : before;
    if (candidates.length !== 1) return candidates.length ? "ambiguous_username" : "missing_username";
    return make(form, { username: candidates[0], password }, { original: candidates[0].value.trim() });
  }
  if (kind === "card") {
    const match = (token) => all.filter((field) => tokens(field).includes(token));
    if (match("cc-given-name").length || match("cc-family-name").length) return "unsupported_split_name";
    const keys = [["number", "cc-number"], ["cvc", "cc-csc"], ["expiry", "cc-exp"], ["month", "cc-exp-month"], ["year", "cc-exp-year"], ["name", "cc-name"]];
    const fields = {};
    for (const [key, token] of keys) {
      const found = match(token);
      if (found.length > 1) return "ambiguous_" + key;
      if ((key === "number" || key === "cvc") && !found.length) return "missing_" + key;
      if (found[0] && ["number", "cvc", "name", "expiry"].includes(key) && !(found[0] instanceof HTMLInputElement)) return "unsupported_" + key;
      if (found[0]) fields[key] = found[0];
    }
    if (fields.expiry && (fields.month || fields.year)) return "ambiguous_expiry";
    return make(form, fields);
  }
  if (kind === "one_time_code") {
    const named = all.filter((field) => tokens(field).includes("one-time-code"));
    const singles = all.filter((field) => ["text", "tel", "number"].includes(field.type) && field.maxLength === 1);
    if (singles.some((field, i) => i > 0 && all.indexOf(field) === all.indexOf(singles[i - 1]) + 1)) return "unsupported_split_code";
    if (named.length > 1) return "ambiguous_code";
    const field = named[0] || (all.includes(anchor) && ["text", "tel", "number"].includes(anchor.type) && anchor.maxLength >= 4 && anchor.maxLength <= 10 ? anchor : null);
    if (!field || !["text", "tel", "number"].includes(field.type) || field.maxLength === 1) return "missing_code";
    return make(form, { code: field });
  }
  return "unsupported_kind";
"#;

/// The names of the fields the handle holds.
pub(super) const FIELDS: &str = "function () { return Object.keys(this.fields); }";

/// One write. Rechecks origin, document, form and (without a hint, for a
/// card or code) focus, then sets the value through the native setter and
/// fires input and change. Returns true, or "changed", "account" or
/// "no_option" with nothing written.
pub(super) const MUTATE: &str = r#"
  // CUA_SECRET_FILL_MUTATE
  const h = this;
  if (self.origin !== origin || location.origin !== origin || document !== h.doc) return "changed";
  const usable = (field) => (field instanceof HTMLInputElement || field instanceof HTMLSelectElement) && field.form === h.form &&
    field.ownerDocument === document && field.isConnected && !field.disabled && !field.readOnly && visible(field);
  if (!Object.values(h.fields).every(usable)) return "changed";
  if (!h.hinted && (h.kind === "card" || h.kind === "one_time_code")) {
    const focused = focusedElement();
    if (!focused || focused.form !== h.form) return "changed";
  }
  if (h.fields.password && h.fields.password.type !== "password") return "changed";
  if (h.kind === "login") {
    const shown = h.fields.username.value.trim();
    const wanted = extra.username.trim().toLowerCase();
    if (which === "username") {
      if (shown !== h.original || (shown !== "" && shown.toLowerCase() !== wanted)) return "account";
    } else if (shown.toLowerCase() !== wanted) return "account";
  }
  const field = h.fields[which];
  if (!field) return "changed";
  if (field instanceof HTMLSelectElement) {
    const options = matchOption(field, which, extra.month, extra.year);
    if (options.length !== 1) return "no_option";
    Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value").set.call(field, options[0].value);
  } else {
    const actual = which === "expiry" && field.type === "month" ? extra.full_year + "-" + extra.month : value;
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(field, actual);
  }
  field.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
  field.dispatchEvent(new Event("change", { bubbles: true }));
  return true;
"#;

/// Whether every written field still shows what was written.
pub(super) const READ_BACK: &str = r#"
  // CUA_SECRET_FILL_READ_BACK
  const h = this;
  if (self.origin !== origin || location.origin !== origin || document !== h.doc) return false;
  return Object.keys(expected).every((which) => {
    const field = h.fields[which];
    if (!field || !field.isConnected || field.ownerDocument !== document || field.form !== h.form) return false;
    if (field instanceof HTMLSelectElement) {
      const options = matchOption(field, which, extra.month, extra.year);
      return options.length === 1 && field.selectedOptions.length === 1 && field.selectedOptions[0] === options[0];
    }
    const actual = which === "expiry" && field.type === "month" ? extra.full_year + "-" + extra.month : expected[which];
    return field.value === actual;
  });
"#;

/// Drop the handle's references to the page.
pub(super) const RELEASE: &str =
    "function () { for (const key of Object.keys(this)) delete this[key]; return true; }";

pub(super) fn page_function(params: &str, body: &str) -> String {
    format!("function ({params}) {{{PRELUDE}{body}}}")
}

/// The reasons FIND may return; anything else reads as `no_form`.
const FIND_REASONS: &[&str] = &[
    "field_in_frame",
    "missing_password",
    "ambiguous_password",
    "no_form",
    "missing_username",
    "ambiguous_username",
    "unsupported_split_name",
    "ambiguous_number",
    "missing_number",
    "unsupported_number",
    "ambiguous_cvc",
    "missing_cvc",
    "unsupported_cvc",
    "ambiguous_expiry",
    "unsupported_expiry",
    "ambiguous_month",
    "ambiguous_year",
    "ambiguous_name",
    "unsupported_name",
    "unsupported_split_code",
    "ambiguous_code",
    "missing_code",
];

pub(super) fn find_reason(raw: Option<&Value>) -> &'static str {
    raw.and_then(Value::as_str)
        .and_then(|reason| FIND_REASONS.iter().copied().find(|known| *known == reason))
        .unwrap_or("no_form")
}

pub(super) fn find_message(reason: &str) -> &'static str {
    match reason {
        "field_in_frame" => "the field is inside a frame; fill it from the page itself",
        "no_form" => "no form holds the focused field; focus a field of the form or pass ref",
        reason if reason.starts_with("ambiguous_") => {
            "the form has more than one field this value could go into"
        }
        reason if reason.starts_with("unsupported_") => {
            "the form splits a value across fields this fill does not support"
        }
        _ => "the form lacks a field this fill needs",
    }
}


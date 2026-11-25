use cua_driver_core::browser::BrowserRefusal;

pub(crate) fn require_prompt_identity(title: &str, body: &str) -> Result<(), BrowserRefusal> {
    let body = body.to_ascii_lowercase();
    if title.trim().eq_ignore_ascii_case("allow remote debugging?")
        && body.contains("external app wants full control")
        && body.contains("saved data, cookies and site data")
        && body.contains("navigate to any url")
    {
        return Ok(());
    }
    Err(BrowserRefusal::new(
        cua_driver_core::browser::BrowserRefusalCode::BrowserWrongTargetRefused,
        "cannot identify the remote-debugging prompt; approve it in the browser yourself",
    ))
}

pub(crate) fn prompt_present(
    find_root: impl FnOnce() -> Result<Option<usize>, BrowserRefusal>,
) -> Result<bool, BrowserRefusal> {
    Ok(find_root()?.is_some())
}

pub(crate) fn cleanup(
    before: impl FnOnce() -> Result<bool, BrowserRefusal>,
    setup: impl FnOnce() -> Result<bool, BrowserRefusal>,
    after: impl FnOnce() -> Result<bool, BrowserRefusal>,
) -> Result<bool, BrowserRefusal> {
    let before = before()?;
    let setup = setup()?;
    let after = after()?;
    Ok(before || setup || after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cua_driver_core::browser::BrowserRefusalCode;
    use std::cell::Cell;

    #[test]
    fn prompt_identity_refuses_unrelated_and_unknown_locale_for_both_actions() {
        let expected = "An external app wants full control over this Chrome session. It can access your saved data, cookies and site data and navigate to any URL.";
        assert!(require_prompt_identity("Allow remote debugging?", expected).is_ok());
        for (title, body) in [
            ("Delete all data?", expected),
            ("Remote debugging", "unrelated"),
            ("リモート デバッグを許可しますか？", "日本語"),
        ] {
            for _action in ["allow", "cancel"] {
                assert!(require_prompt_identity(title, body).is_err(), "{title}");
            }
        }
    }

    #[test]
    fn ambiguous_roots_stop_dismiss_and_cleanup() {
        let cleanup_called = Cell::new(false);
        let result = cleanup(
            || {
                prompt_present(|| {
                    Err(BrowserRefusal::new(
                        BrowserRefusalCode::BrowserWrongTargetRefused,
                        "multiple native alerts",
                    ))
                })
            },
            || {
                cleanup_called.set(true);
                Ok(false)
            },
            || Ok(false),
        );
        assert!(result.is_err());
        assert!(!cleanup_called.get());
    }
}

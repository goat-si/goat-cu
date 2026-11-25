use cua_driver_core::browser::BrowserRefusal;

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

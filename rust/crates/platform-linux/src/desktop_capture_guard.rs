use anyhow::Result;

pub(crate) fn complete_capture<T>(
    probe_incomplete: impl FnOnce() -> Result<bool>,
    capture: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if probe_incomplete()? {
        anyhow::bail!(
            "rootless Xwayland cannot provide a complete desktop capture; use compositor capture"
        );
    }
    capture()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn incomplete_xwayland_never_publishes_an_image() {
        let called = Cell::new(false);
        let result = complete_capture(
            || Ok(true),
            || {
                called.set(true);
                Ok(vec![1u8])
            },
        );
        assert!(result.is_err());
        assert!(!called.get(), "incomplete captures must not be published");
    }

    #[test]
    fn readable_root_can_capture_and_probe_errors_propagate() {
        assert_eq!(complete_capture(|| Ok(false), || Ok(42)).unwrap(), 42);
        assert!(complete_capture(|| anyhow::bail!("probe failed"), || Ok(42)).is_err());
    }
}

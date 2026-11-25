use std::future::Future;

pub(crate) async fn after_cursor<T, E>(
    position: impl Future<Output = ()>,
    foreground: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    position.await;
    foreground()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[tokio::test]
    async fn dispatch_uses_the_foreground_after_cursor_positioning() {
        let foreground = Cell::new(1u32);
        let target = after_cursor(async { foreground.set(2) }, || {
            Ok::<_, ()>(foreground.get())
        })
        .await
        .unwrap();
        assert_eq!(
            target, 2,
            "the user switched foreground during the cursor wait"
        );
    }
}

pub(crate) fn require_current_desktop(result: Result<bool, String>) -> Result<(), String> {
    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(
            "background_unavailable: target is on another Task View desktop; no input was sent"
                .into(),
        ),
        Err(error) => Err(format!(
            "background_unavailable: cannot confirm the target Task View desktop ({error}); no input was sent"
        )),
    }
}

#[cfg(test)]
mod desktop_evidence_tests {
    use super::*;

    #[test]
    fn unproven_or_other_desktop_cannot_arm_a_style_guard() {
        assert!(require_current_desktop(Ok(true)).is_ok());
        assert!(require_current_desktop(Ok(false)).is_err());
        assert!(require_current_desktop(Err("shell unavailable".into())).is_err());
    }
}

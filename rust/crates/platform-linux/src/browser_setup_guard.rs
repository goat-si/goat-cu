pub(crate) fn validate(toolbar: bool, focused: bool, same_identity: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        toolbar && focused && same_identity,
        "the focused field is not the previously identified browser toolbar omnibox"
    );
    Ok(())
}

pub(crate) fn run_steps(
    mut validate: impl FnMut() -> anyhow::Result<()>,
    mut send: impl FnMut(&str) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for key in ["select", "paste", "enter"] {
        validate()?;
        send(key)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn unrelated_field_sends_no_keys() {
        let keys = RefCell::new(Vec::new());
        let result = run_steps(
            || validate(false, true, true),
            |key| {
                keys.borrow_mut().push(key.to_owned());
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(keys.borrow().is_empty());
    }

    #[test]
    fn focus_moving_after_select_stops_paste_and_enter() {
        let keys = RefCell::new(Vec::new());
        let mut count = 0;
        let result = run_steps(
            || {
                count += 1;
                validate(true, count == 1, true)
            },
            |key| {
                keys.borrow_mut().push(key.to_owned());
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(&*keys.borrow(), &["select"]);
    }
}

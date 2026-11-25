pub(crate) fn may_expose(is_password: Option<bool>, native_password: bool) -> bool {
    is_password == Some(false) && !native_password
}

pub(crate) fn selection_acknowledged(result: Option<isize>) -> bool {
    result.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_and_unknown_never_expose() {
        assert!(!may_expose(Some(true), false));
        assert!(!may_expose(Some(false), true));
        assert!(!may_expose(None, false));
        assert!(may_expose(Some(false), false));
    }

    #[test]
    fn replacement_requires_acknowledged_selection() {
        assert!(!selection_acknowledged(None));
        assert!(selection_acknowledged(Some(0)));
    }
}

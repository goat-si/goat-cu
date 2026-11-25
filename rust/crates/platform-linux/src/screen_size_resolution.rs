//! Choosing attested display metadata when a Wayland layout query fails.

pub(crate) fn resolve<T, E>(
    logical: Result<Option<T>, E>,
    fallback: impl FnOnce() -> Option<T>,
) -> Result<Option<T>, E> {
    match logical {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) => Ok(fallback()),
        Err(error) => match fallback() {
            Some(value) => Ok(Some(value)),
            None => Err(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn unavailable_logical_layout_can_use_attested_mutter_metadata() {
        assert_eq!(
            resolve::<_, &str>(Err("missing xdg output"), || Some(42)),
            Ok(Some(42))
        );
    }

    #[test]
    fn failed_queries_do_not_invent_a_frame() {
        assert_eq!(
            resolve::<u32, _>(Err("query failed"), || None),
            Err("query failed")
        );
    }

    #[test]
    fn valid_logical_layout_takes_precedence() {
        assert_eq!(
            resolve::<_, &str>(Ok(Some(7)), || panic!("fallback must not run")),
            Ok(Some(7))
        );
    }
}

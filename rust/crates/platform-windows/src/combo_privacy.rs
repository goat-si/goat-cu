pub(crate) fn may_expose(is_password: Option<bool>, native_password: bool) -> bool {
    is_password == Some(false) && !native_password
}

pub(crate) fn selection_acknowledged(result: Option<isize>) -> bool {
    result.is_some()
}

pub(crate) fn write_result(
    idx: usize,
    value: &str,
    secure: bool,
) -> cua_driver_core::protocol::ToolResult {
    if secure {
        cua_driver_core::protocol::ToolResult::text(format!("Set value on [{idx}]; verification withheld for a secure target."))
            .with_structured(serde_json::json!({"verified": false, "verification_withheld": true, "effect": "unverifiable"}))
    } else {
        cua_driver_core::protocol::ToolResult::text(format!("Set [{idx}] to '{value}'"))
    }
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

    #[test]
    fn secure_write_result_withholds_value_and_verification() {
        let result = write_result(4, "sentinel-secret", true);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("sentinel-secret"));
        assert!(json.contains("verification_withheld"));
    }
}

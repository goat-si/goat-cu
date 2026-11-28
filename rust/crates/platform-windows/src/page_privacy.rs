pub(crate) fn format_query_node(
    expose: bool,
    control_type: &str,
    name: &str,
    attributes: &[String],
) -> String {
    if !expose {
        return format!("- {control_type}");
    }
    let name = name.replace('"', "\\\"");
    if attributes.is_empty() {
        format!("- {control_type} \"{name}\"")
    } else {
        format!("- {control_type} \"{name}\" [{}]", attributes.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::format_query_node;
    use crate::combo_privacy::may_expose;

    #[test]
    fn password_and_unknown_fields_hide_all_supplied_content() {
        let attributes = vec![
            "value=secret-value".to_owned(),
            "outerHTML=<input secret-markup>".to_owned(),
        ];
        for password in [Some(true), None] {
            let line = format_query_node(
                may_expose(password, false),
                "Edit",
                "secret-name",
                &attributes,
            );
            assert_eq!(line, "- Edit");
        }
        let line = format_query_node(
            may_expose(Some(false), false),
            "Edit",
            "label",
            &["value=visible".to_owned()],
        );
        assert_eq!(line, "- Edit \"label\" [value=visible]");
    }
}

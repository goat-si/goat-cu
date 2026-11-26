use cua_driver_core::protocol::ToolResult;
use serde_json::json;
use std::time::Duration;

pub(crate) fn click_timeout(index: usize, budget: Duration) -> ToolResult {
    ToolResult::error(format!("click: the AT-SPI action for element [{index}] did not complete within {budget:?}; its effect is unknown and it may still complete. Read the state; do not resend while completion remains unknown."))
        .with_structured(json!({"code":"ax_timeout", "effect":"unknown"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cua_driver_core::protocol::Content;

    #[test]
    fn timed_out_click_never_recommends_resending_an_action_that_may_still_run() {
        let result = click_timeout(4, Duration::from_secs(1));
        let Content::Text { text, .. } = &result.content[0] else {
            panic!("text response")
        };
        assert!(text.contains("do not resend"), "{text}");
        assert!(!text.contains("retry"), "{text}");
        let structured = result.structured_content.unwrap();
        assert_eq!(structured["effect"], "unknown");
        assert!(structured.get("escalation").is_none());
    }
}

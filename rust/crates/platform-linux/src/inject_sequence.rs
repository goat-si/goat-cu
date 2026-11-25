// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

#[derive(Debug)]
pub(crate) struct DeliveryUncertain {
    pub(crate) acknowledged: usize,
    pub(crate) reason: String,
}

impl std::fmt::Display for DeliveryUncertain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cua-compositor input delivery is uncertain after {} acknowledged command(s): {}. Read state before retrying.", self.acknowledged, self.reason)
    }
}

impl std::error::Error for DeliveryUncertain {}

pub(crate) fn require_complete_line(line: &str) -> anyhow::Result<()> {
    if !line.ends_with('\n') {
        anyhow::bail!("cua-compositor reply ended before its newline");
    }
    Ok(())
}

/// Run each compositor command and stop as soon as its response is rejected.
pub(crate) fn send_all(
    lines: &[String],
    mut exchange: impl FnMut(&str) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for (acknowledged, line) in lines.iter().enumerate() {
        if let Err(error) = exchange(line) {
            return Err(DeliveryUncertain {
                acknowledged,
                reason: error.to_string(),
            }
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rejected_tick_prevents_later_commands() {
        let lines = vec!["one".into(), "two".into(), "three".into()];
        let mut sent = Vec::new();
        let result = send_all(&lines, |line| {
            sent.push(line.to_owned());
            if line == "two" {
                anyhow::bail!("refused")
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<DeliveryUncertain>()
                .unwrap()
                .acknowledged,
            1
        );
        assert_eq!(sent, ["one", "two"]);
    }

    #[test]
    fn truncated_ack_is_not_a_complete_reply() {
        assert!(require_complete_line("ok").is_err());
        assert!(require_complete_line("ok\n").is_ok());
        let mut sent = 0;
        let result = send_all(&["one".into(), "two".into()], |_| {
            sent += 1;
            require_complete_line("ok")
        });
        assert!(result.is_err());
        assert_eq!(sent, 1);
    }
}

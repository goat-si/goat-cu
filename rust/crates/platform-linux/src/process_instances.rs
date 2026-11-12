// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

//! `process_instance` labelling for Linux answers that name a pid (#299).
//!
//! The label is the kernel start time from `/proc/<pid>/stat` field 22, an
//! opaque decimal string compared for equality only. Each tool reads it once
//! before it builds its answer and once after, and keeps a label only where
//! the two reads agree; an unreadable or changed read is omitted, never
//! guessed. The readers keep the filesystem access in `proc_fs`; the decision
//! is pure so it is testable without a live process.

/// Outcome of the two reads around one answer. `Some` only when the same
/// start time was read both times.
pub(crate) fn label_instance(before: Option<u64>, after: Option<u64>) -> Option<String> {
    match (before, after) {
        (Some(before), Some(after)) if before == after => Some(after.to_string()),
        _ => None,
    }
}

/// One start-time read per pid.
pub(crate) fn read_instances(
    pids: impl IntoIterator<Item = u32>,
    mut read: impl FnMut(u32) -> Option<u64>,
) -> std::collections::HashMap<u32, Option<u64>> {
    pids.into_iter().map(|pid| (pid, read(pid))).collect()
}

/// Attach `process_instance` to every record naming a pid whose two reads
/// agree.
pub(crate) fn label_records(
    records: &mut [serde_json::Value],
    before: &std::collections::HashMap<u32, Option<u64>>,
    after: &std::collections::HashMap<u32, Option<u64>>,
) {
    for record in records.iter_mut() {
        let Some(pid) = record["pid"]
            .as_u64()
            .and_then(|pid| u32::try_from(pid).ok())
        else {
            continue;
        };
        if pid == 0 {
            continue;
        }
        if let Some(label) = label_instance(
            before.get(&pid).copied().flatten(),
            after.get(&pid).copied().flatten(),
        ) {
            record["process_instance"] = serde_json::json!(label);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #299: a label needs the same start time read twice. Anything else is
    /// omitted, never guessed: one unreadable pass, or a pid that restarted
    /// between the passes, names no instance.
    #[test]
    fn a_label_needs_two_agreeing_start_time_reads() {
        assert_eq!(label_instance(Some(77), Some(77)), Some("77".to_owned()));
        assert_eq!(label_instance(Some(77), Some(78)), None);
        assert_eq!(label_instance(None, Some(77)), None);
        assert_eq!(label_instance(Some(77), None), None);
        assert_eq!(label_instance(None, None), None);
    }

    /// #299: each record naming a live pid carries its instance label; an
    /// installed-only entry (pid 0) and a pid only one pass could read carry
    /// none.
    #[test]
    fn records_name_the_instance_of_the_pid_they_report() {
        let mut records = vec![
            serde_json::json!({"pid": 17, "name": "Alpha"}),
            serde_json::json!({"pid": 17, "name": "Alpha window"}),
            serde_json::json!({"pid": 0, "name": "Installed only", "running": false}),
            serde_json::json!({"pid": 18, "name": "Unreadable"}),
            serde_json::json!({"pid": 19, "name": "Restarted"}),
        ];
        let pids = [17, 18, 19];
        let mut reads = Vec::new();
        let before = read_instances(pids, |pid| {
            reads.push(pid);
            (pid != 18).then_some(u64::from(pid) * 10)
        });
        let after = read_instances(pids, |pid| {
            reads.push(pid);
            match pid {
                19 => Some(999),
                _ => (pid != 18).then_some(u64::from(pid) * 10),
            }
        });
        assert_eq!(reads, [17, 18, 19, 17, 18, 19], "one read per pid per pass");
        label_records(&mut records, &before, &after);
        assert_eq!(records[0]["process_instance"], "170");
        assert_eq!(records[1]["process_instance"], "170");
        assert!(
            records[2].get("process_instance").is_none(),
            "an installed-only entry has no process"
        );
        assert!(
            records[3].get("process_instance").is_none(),
            "an unreadable start time is omitted"
        );
        assert!(
            records[4].get("process_instance").is_none(),
            "a pid restarted during the read is omitted"
        );
    }
}

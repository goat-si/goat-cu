// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

//! `set_value` on a LibreOffice Calc cell or its Name Box.
//!
//! Calc cell and Name Box classification. The upstream key sequence is not
//! enabled: it lacks GOAT's per-keystroke process-instance checks.

use crate::ax::bindings::{copy_element_attr, copy_string_attr, AXUIElementRef};
use core_foundation::base::{CFRelease, CFTypeRef};

/// The Name Box's accessibility identifier.
const NAME_BOX_ID: &str = "pos_window";
/// What a Calc `set_value` addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CalcTarget {
    /// A sheet cell: go to `address`, then type the value into it.
    Cell { address: String },
    /// The Name Box: the value is the address to go to.
    NameBox,
}

/// A single-cell address such as `H9` or `ab120`, upper-cased: one to three
/// column letters, then a row number without a leading zero.
pub(super) fn cell_address(text: &str) -> Option<String> {
    let text = text.trim();
    let letters = text.chars().take_while(char::is_ascii_alphabetic).count();
    let row = &text[letters..];
    let row_ok = !row.is_empty()
        && row.len() <= 7
        && row.chars().all(|c| c.is_ascii_digit())
        && !row.starts_with('0');
    ((1..=3).contains(&letters) && row_ok).then(|| text.to_ascii_uppercase())
}

/// Text a cell takes as one entry: Tab and Return would commit it and move
/// on, so a value carrying them is not a single cell's value.
#[cfg(test)]
pub(super) fn single_entry(value: &str) -> bool {
    !value.chars().any(|c| matches!(c, '\n' | '\r' | '\t'))
}

/// Classify `element` from its accessibility attributes. Pure so it is
/// testable: `role`, `parent_role`, the identifier, and the title and
/// description (Calc names a cell by its address in one of them).
pub(super) fn classify(
    role: &str,
    parent_role: Option<&str>,
    identifier: Option<&str>,
    names: [Option<&str>; 2],
) -> Option<CalcTarget> {
    match role {
        "AXComboBox" if identifier == Some(NAME_BOX_ID) => Some(CalcTarget::NameBox),
        "AXTextField" if parent_role == Some("AXTable") => names
            .into_iter()
            .flatten()
            .find_map(cell_address)
            .map(|address| CalcTarget::Cell { address }),
        _ => None,
    }
}

/// [`classify`] on a live element. Only LibreOffice (VCL) targets qualify.
///
/// # Safety
///
/// `element` must be a live `AXUIElementRef`.
pub(super) unsafe fn target_of(pid: i32, element: AXUIElementRef) -> Option<CalcTarget> {
    let role = copy_string_attr(element, "AXRole")?;
    if role != "AXTextField" && role != "AXComboBox" {
        return None;
    }
    let parent_role = copy_element_attr(element, "AXParent").and_then(|parent| {
        let role = copy_string_attr(parent, "AXRole");
        CFRelease(parent as CFTypeRef);
        role
    });
    let identifier = copy_string_attr(element, "AXIdentifier");
    let title = copy_string_attr(element, "AXTitle");
    let description = copy_string_attr(element, "AXDescription");
    let target = classify(
        &role,
        parent_role.as_deref(),
        identifier.as_deref(),
        [title.as_deref(), description.as_deref()],
    )?;
    (crate::input::pointer_toolkit::detect(pid)
        == Some(crate::input::pointer_toolkit::PointerReadingToolkit::Vcl))
    .then_some(target)
}

/// A cell's text from its value and whether the element still answers.
#[cfg(test)]
pub(super) fn cell_text(value: Option<String>, answers: bool) -> Option<String> {
    value.or_else(|| answers.then(String::new))
}

/// What a cell write reports: confirmed when the cell reads the value, else
/// unverifiable with what it reads instead.
#[cfg(test)]
pub(super) fn cell_outcome(
    address: &str,
    element_index: usize,
    value: &str,
    shown: &str,
) -> (bool, String) {
    if shown == value {
        let action = if value.is_empty() { "Cleared" } else { "Set" };
        let to = if value.is_empty() {
            String::new()
        } else {
            format!(" to {value:?}")
        };
        return (
            true,
            format!("✅ {action} {address} [{element_index}]{to} through the Name Box and keys."),
        );
    }
    (
        false,
        format!(
            "📨 Typed {value:?} into {address} [{element_index}] through the Name Box and keys, \
             but the cell reads {shown:?}. Calc may have changed the entry (AutoCorrect, a number \
             or date format, or a formula's result). Read the cell before relying on it."
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{cell_address, cell_outcome, cell_text, classify, single_entry, CalcTarget};

    #[test]
    fn a_missing_value_is_empty_only_while_the_cell_answers() {
        assert_eq!(cell_text(None, true).as_deref(), Some(""));
        assert_eq!(cell_text(None, false), None);
        assert_eq!(cell_text(Some("x".into()), true).as_deref(), Some("x"));
    }

    #[test]
    fn a_cell_that_reads_the_value_is_confirmed() {
        let (verified, detail) = cell_outcome("H3", 32, "probe", "probe");
        assert!(verified);
        assert!(detail.contains("Set H3 [32] to \"probe\""), "{detail}");
        let (verified, detail) = cell_outcome("H3", 32, "", "");
        assert!(verified);
        assert!(detail.contains("Cleared H3 [32]"), "{detail}");
    }

    #[test]
    fn a_cell_that_reads_something_else_is_unverifiable_and_says_what_it_reads() {
        // Calc's AutoCorrect capitalised the first letter.
        let (verified, detail) = cell_outcome("I3", 33, "probe-i3", "Probe-i3");
        assert!(!verified);
        assert!(detail.contains("reads \"Probe-i3\""), "{detail}");
        assert!(detail.contains("AutoCorrect"), "{detail}");
        // A time format.
        let (verified, _) = cell_outcome("D4", 39, "12:15", "12:15:00 PM");
        assert!(!verified);
        // A clear that left text behind.
        let (verified, _) = cell_outcome("D4", 39, "", "12:15:00 PM");
        assert!(!verified);
    }

    #[test]
    fn cell_addresses_are_column_letters_then_a_row() {
        assert_eq!(cell_address("H9").as_deref(), Some("H9"));
        assert_eq!(cell_address(" ab120 ").as_deref(), Some("AB120"));
        for not_one in [
            "",
            "9",
            "H",
            "H09",
            "ABCD1",
            "A1:B2",
            "Sheet1.A1",
            "$A$1",
            "H 9",
        ] {
            assert_eq!(cell_address(not_one), None, "{not_one:?}");
        }
    }

    #[test]
    fn a_table_text_field_named_by_address_is_a_cell() {
        assert_eq!(
            classify("AXTextField", Some("AXTable"), None, [Some("C14"), None]),
            Some(CalcTarget::Cell {
                address: "C14".into()
            })
        );
        assert_eq!(
            classify("AXTextField", Some("AXTable"), None, [None, Some("j3")]),
            Some(CalcTarget::Cell {
                address: "J3".into()
            })
        );
        // Outside a table, or not named by an address: an ordinary field.
        assert_eq!(
            classify("AXTextField", Some("AXGroup"), None, [Some("C14"), None]),
            None
        );
        assert_eq!(
            classify("AXTextField", Some("AXTable"), None, [Some("Total"), None]),
            None
        );
    }

    #[test]
    fn only_the_combo_box_with_the_name_box_id_is_the_name_box() {
        assert_eq!(
            classify(
                "AXComboBox",
                Some("AXToolbar"),
                Some("pos_window"),
                [Some("A1"), None]
            ),
            Some(CalcTarget::NameBox)
        );
        assert_eq!(
            classify(
                "AXComboBox",
                Some("AXToolbar"),
                Some("font_name"),
                [None, None]
            ),
            None
        );
    }

    #[test]
    fn a_value_with_tab_or_return_is_not_one_entry() {
        assert!(single_entry("=SUM(A1:A4)"));
        assert!(single_entry("1,248.60"));
        for many in ["a\tb", "a\nb", "a\r"] {
            assert!(!single_entry(many), "{many:?}");
        }
    }
}

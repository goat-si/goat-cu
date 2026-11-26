pub(crate) fn select(
    dwm: Option<[i32; 4]>,
    win32: Option<[i32; 4]>,
) -> Option<(i32, i32, i32, i32)> {
    fn valid([left, top, right, bottom]: [i32; 4]) -> Option<(i32, i32, i32, i32)> {
        // Guard both arithmetic and implausible sentinel coordinates.
        const LIMIT: i32 = 1_000_000;
        if [left, top, right, bottom]
            .iter()
            .any(|v| !(-LIMIT..=LIMIT).contains(v))
        {
            return None;
        }
        let width = right.checked_sub(left)?;
        let height = bottom.checked_sub(top)?;
        (width > 0 && height > 0).then_some((left, top, width, height))
    }
    dwm.and_then(valid).or_else(|| win32.and_then(valid))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_dwm_falls_back_to_win32() {
        assert_eq!(
            select(Some([0, 0, 0, 0]), Some([200, 52, 1400, 852])),
            Some((200, 52, 1200, 800))
        );
    }
    #[test]
    fn invalid_win32_bounds_are_unknown() {
        assert_eq!(select(None, Some([0, 0, 0, 0])), None);
        assert_eq!(select(None, None), None);
        assert_eq!(select(None, Some([i32::MIN, 0, i32::MAX, 10])), None);
    }
}

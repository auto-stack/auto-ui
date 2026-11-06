// Backend integers are i64; the existing SDK find-from offset is i32.
// Shift the haystack by a character offset and search from zero so no offset
// narrowing or UTF-8 byte indexing changes the language's search semantics.
pub(crate) fn __merged_str_find_from<S: AsRef<str>, P: AsRef<str>, I: Into<i64>>(
    text: S,
    needle: P,
    start: I,
) -> i64 {
    let start = start.into().max(0);
    let text = text.as_ref();
    let byte_start = text.char_indices()
        .nth(usize::try_from(start).unwrap_or(usize::MAX))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    let relative = auto_lang_core::a2r_std::str_find_from(&text[byte_start..], needle, 0);
    if relative < 0 { -1 } else { start + relative }
}

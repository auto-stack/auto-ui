// Generated backends use wide integers. The current SDK accepts them without
// narrowing and owns the shared UTF-8 byte-offset contract (PLAN-956).
pub(crate) fn __merged_str_find_from<S: AsRef<str>, P: AsRef<str>, I: TryInto<usize>>(
    text: S,
    needle: P,
    start: I,
) -> i64 {
    auto_lang_core::a2r_std::str_find_from(text, needle, start)
}

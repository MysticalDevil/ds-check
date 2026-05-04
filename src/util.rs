use unicode_width::UnicodeWidthStr;

pub fn dw(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

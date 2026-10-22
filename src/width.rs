//! Display columns. Wide and Fullwidth count as two. Ambiguous stays one.
//! requirement-rust-tui

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}

pub fn clip_columns(text: &str, columns: usize) -> String {
    if columns == 0 {
        return String::new();
    }
    let mut kept = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let width = char_width(ch);
        if used + width > columns {
            break;
        }
        kept.push(ch);
        used += width;
    }
    kept
}

pub fn clip_columns_right(text: &str, columns: usize) -> String {
    if columns == 0 {
        return String::new();
    }
    let mut kept: Vec<char> = Vec::new();
    let mut used = 0usize;
    for ch in text.chars().rev() {
        let width = char_width(ch);
        if used + width > columns {
            break;
        }
        kept.push(ch);
        used += width;
    }
    kept.reverse();
    kept.into_iter().collect()
}

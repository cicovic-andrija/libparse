//! Parsers related to character-level processing

use crate::{Error, ErrorCode, PResult};

pub const END_OF_STRING: ErrorCode = ErrorCode::Char('\0');

/// Parser generator for parsers that recognize a single character
pub fn char(ch: char) -> impl Fn(&str) -> PResult<&str, char> {
    move |input: &str| match input.chars().next().map(|next| next == ch) {
        Some(true) => Ok((&input[ch.len_utf8()..], ch)),
        Some(false) => Err(Error::new(input, ErrorCode::Char(ch))),
        None => Err(Error::new(input, END_OF_STRING)),
    }
}

/// Parser that matches any character in a string
pub fn any_char(input: &str) -> PResult<&str, char> {
    match input.chars().next() {
        Some(ch) => Ok((&input[ch.len_utf8()..], ch)),
        _ => Err(Error::new(input, END_OF_STRING)),
    }
}

/// Parser that matches a line break (newline): LF or CRLF
pub fn line_break(input: &str) -> PResult<&str, &str> {
    if input.starts_with("\n") {
        Ok((&input["\n".len()..], "\n"))
    } else if input.starts_with("\r\n") {
        Ok((&input["\r\n".len()..], "\r\n"))
    } else {
        Err(Error::new(input, ErrorCode::LineBreak))
    }
}

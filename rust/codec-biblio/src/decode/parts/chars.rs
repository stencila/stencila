//! Functions for tests on individual characters

use winnow::{Parser, Result, token::take_while};

/// Check if a character is any kind of apostrophe
///
/// Handles both straight and curly apostrophes to accommodate different text
/// sources and encoding formats in bibliographic data.
pub fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '\u{2019}')
}

/// Parse exactly one apostrophe character
///
/// Used for parsing contractions and possessives in titles, author names,
/// and other bibliographic fields that may contain apostrophes.
pub fn one_apostrophe<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_apostrophe).parse_next(input)
}

/// Check if a character is any kind of hyphen
///
/// Recognizes various Unicode hyphen characters to handle different text encodings
/// and formatting conventions found in bibliographic references.
pub fn is_hyphen(c: char) -> bool {
    matches!(c, '-' | '–' | '‐' | '‒' | '—' | '―' | '−')
}

/// Parse exactly one hyphen character
///
/// Used to consume hyphen separators in page ranges, date ranges, and other
/// bibliographic elements while being tolerant of different hyphen variants.
pub fn one_hyphen<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_hyphen).parse_next(input)
}

/// Check if a character is an opening quotation mark
///
/// Recognizes various opening quote characters to handle different text encodings
/// and typographic conventions in bibliographic titles and quoted text.
pub fn is_open_quote(c: char) -> bool {
    matches!(c, '"' | '"' | '\u{201C}' | '\u{00AB}' | '\u{2018}')
}

/// Parse exactly one opening quote character
///
/// Used to identify the start of quoted titles, phrases, or other quoted content
/// in bibliographic references while handling various quote styles.
pub fn one_open_quote<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_open_quote).parse_next(input)
}

/// Check if a character is a closing quotation mark
///
/// Recognizes various closing quote characters to properly parse the end of
/// quoted content in bibliographic data from different sources.
pub fn is_close_quote(c: char) -> bool {
    matches!(c, '"' | '"' | '\u{201D}' | '\u{00BB}' | '\u{2019}')
}

/// Parse exactly one closing quote character
///
/// Used to identify the end of quoted titles, phrases, or other quoted content
/// in bibliographic references while handling various quote styles.
pub fn one_close_quote<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_close_quote).parse_next(input)
}

/// Check if a character is any kind of quotation mark
///
/// Combines opening and closing quote detection to handle any quotation mark
/// when the specific type doesn't matter for parsing logic.
pub fn is_quote(c: char) -> bool {
    is_open_quote(c) || is_close_quote(c)
}

/// Parse exactly one quote character (opening or closing)
///
/// Used when any quotation mark needs to be consumed regardless of whether
/// it's opening or closing, such as in general quote cleanup or detection.
pub fn one_quote<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_quote).parse_next(input)
}

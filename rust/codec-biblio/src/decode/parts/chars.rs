//! Functions for tests on individual characters

use winnow::{Parser, Result, token::take_while};

/// Check if a character is any kind of apostrophe
///
/// Handles both straight and curly apostrophes to accommodate different text
/// sources and encoding formats in bibliographic data.
pub fn is_apostrophe(c: char) -> bool {
    matches!(
        c,
        '\''  // U+0027 APOSTROPHE
    | '’' // U+2019 RIGHT SINGLE QUOTATION MARK (preferred apostrophe in typography)
    | 'ʼ' // U+02BC MODIFIER LETTER APOSTROPHE
    | 'ʽ' // U+02BD MODIFIER LETTER REVERSED COMMA
    | 'ʾ' // U+02BE MODIFIER LETTER RIGHT HALF RING
    | 'ʿ' // U+02BF MODIFIER LETTER LEFT HALF RING
    | 'ˈ' // U+02C8 MODIFIER LETTER VERTICAL LINE (used as stress mark, sometimes mistaken for apostrophe)
    | '՚' // U+055A ARMENIAN APOSTROPHE
    | 'Ꞌ' // U+A78B LATIN CAPITAL LETTER SALTILLO
    | 'ꞌ' // U+A78C LATIN SMALL LETTER SALTILLO
    | '＇' // U+FF07 FULLWIDTH APOSTROPHE
    )
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
    matches!(
        c,
        '-'   // U+002D HYPHEN-MINUS
    | '‐' // U+2010 HYPHEN
    | '‒' // U+2012 FIGURE DASH
    | '–' // U+2013 EN DASH
    | '—' // U+2014 EM DASH
    | '―' // U+2015 HORIZONTAL BAR
    | '⁻' // U+207B SUPERSCRIPT MINUS
    | '₋' // U+208B SUBSCRIPT MINUS
    | '−' // U+2212 MINUS SIGN
    | '﹘' // U+FE58 SMALL EM DASH
    | '﹣' // U+FE63 SMALL HYPHEN-MINUS
    | '－' // U+FF0D FULLWIDTH HYPHEN-MINUS
    )
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
    matches!(
        c,
        '"'     // U+0022 QUOTATION MARK
    | '\''  // U+0027 APOSTROPHE
    | '“'   // U+201C LEFT DOUBLE QUOTATION MARK
    | '„'   // U+201E DOUBLE LOW-9 QUOTATION MARK
    | '‟'   // U+201F DOUBLE HIGH-REVERSED-9 QUOTATION MARK
    | '〝'   // U+301D REVERSED DOUBLE PRIME QUOTATION MARK
    | '«'   // U+00AB LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    | '﹁'   // U+FE41 PRESENTATION FORM FOR VERTICAL LEFT CORNER BRACKET
    | '﹃'   // U+FE43 PRESENTATION FORM FOR VERTICAL LEFT WHITE CORNER BRACKET
    | '‘'   // U+2018 LEFT SINGLE QUOTATION MARK
    | '‚'   // U+201A SINGLE LOW-9 QUOTATION MARK
    | '‛'   // U+201B SINGLE HIGH-REVERSED-9 QUOTATION MARK
    | '‹' // U+2039 SINGLE LEFT-POINTING ANGLE QUOTATION MARK
    )
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
    matches!(
        c,
        '"'     // U+0022 QUOTATION MARK
    | '\''  // U+0027 APOSTROPHE
    | '”'   // U+201D RIGHT DOUBLE QUOTATION MARK
    | '‟'   // U+201F DOUBLE HIGH-REVERSED-9 QUOTATION MARK
    | '〞'   // U+301E DOUBLE PRIME QUOTATION MARK
    | '»'   // U+00BB RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    | '﹂'   // U+FE42 PRESENTATION FORM FOR VERTICAL RIGHT CORNER BRACKET
    | '﹄'   // U+FE44 PRESENTATION FORM FOR VERTICAL RIGHT WHITE CORNER BRACKET
    | '’'   // U+2019 RIGHT SINGLE QUOTATION MARK
    | '‛'   // U+201B SINGLE HIGH-REVERSED-9 QUOTATION MARK
    | '›' // U+203A SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
    )
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
#[allow(dead_code)]
pub fn is_quote(c: char) -> bool {
    is_open_quote(c) || is_close_quote(c)
}

/// Parse exactly one quote character (opening or closing)
///
/// Used when any quotation mark needs to be consumed regardless of whether
/// it's opening or closing, such as in general quote cleanup or detection.
#[allow(dead_code)]
pub fn one_quote<'s>(input: &mut &'s str) -> Result<&'s str> {
    take_while(1..=1, is_quote).parse_next(input)
}

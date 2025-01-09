use std::cell::{Ref, RefCell};

/// A UTF8-based line/column position in a string
#[derive(Debug, PartialEq)]
pub struct Position8 {
    /// The 0-based line index
    pub line: usize,

    /// The 0-based, UFT8-based code unit index
    pub column: usize,
}

impl Position8 {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// A UTF8-based range of positions
#[derive(Debug, PartialEq)]
pub struct Range8 {
    /// The start of the range
    pub start: Position8,

    /// The end of the range (exclusive)
    pub end: Position8,
}

impl Range8 {
    pub fn new(start: Position8, end: Position8) -> Self {
        Self { start, end }
    }
}

/// A UTF16-based line/column position in a string
///
/// This is the default representation used by the Language Server Protocol and it is
/// mandatory for servers to support it.
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#positionEncodingKind
#[derive(Debug, PartialEq)]
pub struct Position16 {
    /// The 0-based line index
    pub line: usize,

    /// The 0-based, UFT16-based byte index
    pub column: usize,
}

impl Position16 {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// A UTF16-based range of positions
#[derive(Debug, PartialEq)]
pub struct Range16 {
    /// The start of the range
    pub start: Position16,

    /// The end of the range (exclusive)
    pub end: Position16,
}

impl Range16 {
    pub fn new(start: Position16, end: Position16) -> Self {
        Self { start, end }
    }
}

/// A caching lookup structure for finding line/column positions from UTF8 character indices and vice verse.
///
/// Based on https://github.com/TheBerkin/line-col-rs/blob/master/src/lib.rs but with zero-based
/// line and character numbers and support for UTF16-based positions (and without support for
/// Unicode grapheme clusters).
pub struct Positions<'content> {
    /// The string content with the position
    content: &'content str,

    /// The UTF8 character index at the start of each line
    ///
    /// Just-in-time populated in the `lines()` method.
    lines: RefCell<Option<Vec<usize>>>,
}

impl<'content> Positions<'content> {
    pub fn new(content: &'content str) -> Self {
        Self {
            content,
            lines: RefCell::new(None),
        }
    }

    /// Get the lines (the UTF8 character indices at the start of each line) for the content
    fn lines(&self) -> Ref<'_, Option<Vec<usize>>> {
        if self.lines.borrow().is_none() {
            let lines: Vec<usize> = std::iter::once(0)
                .chain(
                    self.content
                        .chars()
                        .enumerate()
                        .filter_map(|(index, char)| (char == '\n').then_some(index + 1)),
                )
                .collect();
            self.lines.replace(Some(lines));
        }

        self.lines.borrow()
    }

    /// Find the line that a character is on using binary search
    ///
    /// Returns the index of the line and the index of the line start character.
    fn find_line(&self, char_index: usize) -> (usize, usize) {
        let lines = self.lines();
        let lines = lines.as_ref().expect("should always be some");

        let mut line_range = 0..lines.len();
        while line_range.end - line_range.start > 1 {
            let range_middle = line_range.start + (line_range.end - line_range.start) / 2;
            let (left, right) = (line_range.start..range_middle, range_middle..line_range.end);
            if (lines[left.start]..lines[left.end]).contains(&char_index) {
                line_range = left;
            } else {
                line_range = right;
            }
        }

        (line_range.start, lines[line_range.start])
    }

    /// Get a the character index at the start of a line
    fn get_line(&self, line_index: usize) -> Option<usize> {
        let lines = self.lines();
        let lines = lines.as_ref().expect("should always be some");

        lines.get(line_index).copied()
    }

    /// Get the UTF16-based column index for a UTF-8 column index
    fn utf8_to_utf16_column(&self, utf8_line_start: usize, utf8_column: usize) -> usize {
        if utf8_column == 0 {
            return 0;
        }

        let mut chars = self.content.chars().skip(utf8_line_start);

        let mut utf16_column = 0;
        for _ in 0..(utf8_column.saturating_sub(utf8_line_start)) {
            if let Some(char) = chars.next() {
                utf16_column += if char as u32 <= 0xFFF { 1 } else { 2 };
                if char == '\n' {
                    return utf16_column;
                }
            } else {
                return utf16_column;
            }
        }

        utf16_column
    }

    /// Get the UTF8-based column index for a UTF-16 column index
    fn utf16_to_utf8_column(&self, utf8_line_start: usize, utf16_column: usize) -> Option<usize> {
        if utf16_column == 0 {
            return Some(0);
        }

        let chars = self.content.chars().skip(utf8_line_start);

        let mut utf16_count = 0;
        let mut utf8_column = 0;
        for char in chars {
            let char_utf16_len = if char as u32 <= 0xFFFF { 1 } else { 2 };

            // Check if adding this character's UTF-16 length would exceed the specified position
            if utf16_count + char_utf16_len > utf16_column {
                return Some(utf8_column);
            }

            if char == '\n' {
                return None;
            }

            utf16_count += char_utf16_len;
            utf8_column += 1; // Each `char` in Rust is a valid Unicode code point, one UTF-8 character position
        }

        if utf8_column == utf16_count {
            return Some(utf8_column);
        }

        None
    }

    /// Get the UTF8-based position at the character index in the content
    pub fn position8_at_index(&self, char_index: usize) -> Position8 {
        let (line, line_start) = self.find_line(char_index);
        let column = char_index.saturating_sub(line_start);

        Position8 { line, column }
    }

    /// Get the UTF16-based position at the character index in the content
    pub fn position16_at_index(&self, char_index: usize) -> Position16 {
        let (line, line_start) = self.find_line(char_index);
        let column = self.utf8_to_utf16_column(line_start, char_index);

        Position16 { line, column }
    }

    /// Get the character index at the UTF8-based position in the content
    ///
    /// Returns `None` if the line index is out of bounds or the column index is beyond
    /// the end of the line.
    pub fn index_at_position8(&self, Position8 { line, column }: Position8) -> Option<usize> {
        let line_start = self.get_line(line)?;

        let index = line_start.saturating_add(column);

        if let Some(next_line_start) = self.get_line(line + 1) {
            (index < next_line_start).then_some(index)
        } else {
            (index < self.content.chars().count()).then_some(index)
        }
    }

    /// Get the character index at the UTF16-based position in the content
    ///
    /// Returns `None` if the line index is out of bounds or the column index is beyond
    /// the end of the line.
    pub fn index_at_position16(&self, Position16 { line, column }: Position16) -> Option<usize> {
        let line_start = self.get_line(line)?;

        let index = line_start + self.utf16_to_utf8_column(line_start, column)?;

        if let Some(next_line_start) = self.get_line(line + 1) {
            (index < next_line_start).then_some(index)
        } else {
            (index < self.content.chars().count()).then_some(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In the following tests, the emoji 😊 is used because it is a non-BMP
    /// character containing a surrogate pair.

    #[test]
    fn position8_at_index() {
        let content = "line1\n\n\nbefore😊after";
        let positions = Positions::new(content);

        assert_eq!(positions.position8_at_index(0), Position8::new(0, 0));
        assert_eq!(positions.position8_at_index(6), Position8::new(1, 0));
        assert_eq!(positions.position8_at_index(7), Position8::new(2, 0));
        assert_eq!(positions.position8_at_index(8), Position8::new(3, 0));
        assert_eq!(positions.position8_at_index(14), Position8::new(3, 6));
        assert_eq!(positions.position8_at_index(15), Position8::new(3, 7));
        assert_eq!(positions.position8_at_index(20), Position8::new(3, 12));
    }

    #[test]
    fn position16_at_index() {
        let content = "line1\n\n\nbefore😊after";
        let positions = Positions::new(content);

        assert_eq!(positions.position16_at_index(0), Position16::new(0, 0));
        assert_eq!(positions.position16_at_index(6), Position16::new(1, 0));
        assert_eq!(positions.position16_at_index(7), Position16::new(2, 0));
        assert_eq!(positions.position16_at_index(8), Position16::new(3, 0));
        assert_eq!(positions.position16_at_index(14), Position16::new(3, 6));
        assert_eq!(
            positions.position16_at_index(15),
            Position16::new(3, 8) // Note different to above
        );
        assert_eq!(positions.position16_at_index(20), Position16::new(3, 13));
    }

    #[test]
    fn index_at_position8() {
        let content = "line1\n\n\nbefore😊after";
        let positions = Positions::new(content);

        assert_eq!(positions.index_at_position8(Position8::new(0, 0)), Some(0));
        assert_eq!(positions.index_at_position8(Position8::new(0, 4)), Some(4));
        assert_eq!(positions.index_at_position8(Position8::new(0, 6)), None);
        assert_eq!(positions.index_at_position8(Position8::new(1, 0)), Some(6));
        assert_eq!(positions.index_at_position8(Position8::new(1, 1)), None);
        assert_eq!(positions.index_at_position8(Position8::new(2, 0)), Some(7));
        assert_eq!(positions.index_at_position8(Position8::new(2, 1)), None);
        assert_eq!(positions.index_at_position8(Position8::new(3, 0)), Some(8));
        assert_eq!(positions.index_at_position8(Position8::new(3, 6)), Some(14));
        assert_eq!(positions.index_at_position8(Position8::new(3, 7)), Some(15));
        assert_eq!(positions.index_at_position8(Position8::new(3, 12)), None);
        assert_eq!(positions.index_at_position8(Position8::new(4, 0)), None);
    }

    #[test]
    fn index_at_position16() {
        let content = "line1\n\n\nbefore😊after";
        let positions = Positions::new(content);

        assert_eq!(
            positions.index_at_position16(Position16::new(0, 0)),
            Some(0)
        );
        assert_eq!(
            positions.index_at_position16(Position16::new(0, 4)),
            Some(4)
        );
        assert_eq!(
            positions.index_at_position16(Position16::new(0, 5)),
            Some(5)
        );
        assert_eq!(
            positions.index_at_position16(Position16::new(1, 0)),
            Some(6)
        );
        assert_eq!(positions.index_at_position16(Position16::new(1, 1)), None);
        assert_eq!(
            positions.index_at_position16(Position16::new(2, 0)),
            Some(7)
        );
        assert_eq!(positions.index_at_position16(Position16::new(2, 1)), None);
        assert_eq!(
            positions.index_at_position16(Position16::new(3, 0)),
            Some(8)
        );
        assert_eq!(
            positions.index_at_position16(Position16::new(3, 6)),
            Some(14)
        );
        assert_eq!(
            positions.index_at_position16(Position16::new(3, 8)), // Note different to above
            Some(15)
        );
        assert_eq!(positions.index_at_position16(Position16::new(3, 13)), None);
    }

    #[test]
    fn line_after_emoji() {
        let content = "😊\nab";
        let positions = Positions::new(content);

        assert_eq!(positions.index_at_position8(Position8::new(0, 0)), Some(0));
        assert_eq!(positions.index_at_position8(Position8::new(0, 1)), Some(1));
        assert_eq!(positions.index_at_position8(Position8::new(1, 0)), Some(2));
        assert_eq!(positions.index_at_position8(Position8::new(1, 1)), Some(3));
    }
}

use std::cell::RefCell;

/// A UTF8-based line/column position in a string
#[derive(Debug, Default, PartialEq)]
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
#[derive(Debug, Default, PartialEq)]
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

/// Information about a line start position
#[derive(Clone, Copy)]
struct LineInfo {
    /// The UTF8 character index at the start of the line
    char_index: usize,
    /// The byte offset at the start of the line (for O(1) string slicing)
    byte_offset: usize,
}

/// A caching lookup structure for finding line/column positions from UTF8 character indices and vice verse.
///
/// Based on https://github.com/TheBerkin/line-col-rs/blob/master/src/lib.rs but with zero-based
/// line and character numbers and support for UTF16-based positions (and without support for
/// Unicode grapheme clusters).
pub struct Positions<'content> {
    /// The string content with the position
    content: &'content str,

    /// Line information (character indices and byte offsets at the start of each line)
    ///
    /// Just-in-time populated in the `ensure_initialized()` method.
    lines: RefCell<Option<Vec<LineInfo>>>,

    /// The total number of characters in the content
    ///
    /// Just-in-time populated in the `ensure_initialized()` method.
    char_count: RefCell<Option<usize>>,
}

impl<'content> Positions<'content> {
    pub fn new(content: &'content str) -> Self {
        Self {
            content,
            lines: RefCell::new(None),
            char_count: RefCell::new(None),
        }
    }

    /// Ensure line info and char count are computed
    fn ensure_initialized(&self) {
        if self.lines.borrow().is_some() {
            return;
        }

        let mut lines = Vec::new();
        lines.push(LineInfo {
            char_index: 0,
            byte_offset: 0,
        });

        let mut char_count = 0;
        for (byte_offset, char) in self.content.char_indices() {
            char_count += 1;
            if char == '\n' {
                lines.push(LineInfo {
                    char_index: char_count,
                    byte_offset: byte_offset + 1, // +1 to skip the newline
                });
            }
        }

        self.lines.replace(Some(lines));
        self.char_count.replace(Some(char_count));
    }

    /// Get the total character count
    fn char_count(&self) -> usize {
        self.ensure_initialized();
        self.char_count.borrow().expect("should always be some")
    }

    /// Find the line that a character is on using binary search
    ///
    /// Returns the line index and the LineInfo for that line.
    fn find_line(&self, char_index: usize) -> (usize, LineInfo) {
        self.ensure_initialized();
        let lines = self.lines.borrow();
        let lines = lines.as_ref().expect("should always be some");

        let mut line_range = 0..lines.len();
        while line_range.end - line_range.start > 1 {
            let range_middle = line_range.start + (line_range.end - line_range.start) / 2;
            let (left, right) = (line_range.start..range_middle, range_middle..line_range.end);
            if (lines[left.start].char_index..lines[left.end].char_index).contains(&char_index) {
                line_range = left;
            } else {
                line_range = right;
            }
        }

        (line_range.start, lines[line_range.start])
    }

    /// Get the LineInfo at the start of a line
    fn get_line(&self, line_index: usize) -> Option<LineInfo> {
        self.ensure_initialized();
        let lines = self.lines.borrow();
        let lines = lines.as_ref().expect("should always be some");
        lines.get(line_index).copied()
    }

    /// Get the UTF16-based column index for a UTF-8 column index
    ///
    /// Uses byte offset to slice directly into the string (O(1)) instead of
    /// using chars().skip() which is O(n).
    fn utf8_to_utf16_column(&self, line_info: LineInfo, utf8_char_index: usize) -> usize {
        let column_chars = utf8_char_index.saturating_sub(line_info.char_index);
        if column_chars == 0 {
            return 0;
        }

        // Slice from the line start using byte offset (O(1))
        let line_str = &self.content[line_info.byte_offset..];

        let mut utf16_column = 0;
        for (i, char) in line_str.chars().enumerate() {
            if i >= column_chars {
                break;
            }
            utf16_column += if char as u32 <= 0xFFFF { 1 } else { 2 };
            if char == '\n' {
                break;
            }
        }

        utf16_column
    }

    /// Get the UTF8-based column index for a UTF-16 column index
    ///
    /// Uses byte offset to slice directly into the string (O(1)) instead of
    /// using chars().skip() which is O(n).
    fn utf16_to_utf8_column(&self, line_info: LineInfo, utf16_column: usize) -> Option<usize> {
        if utf16_column == 0 {
            return Some(0);
        }

        // Slice from the line start using byte offset (O(1))
        let line_str = &self.content[line_info.byte_offset..];

        let mut utf16_count = 0;
        let mut utf8_column = 0;
        for char in line_str.chars() {
            let char_utf16_len = if char as u32 <= 0xFFFF { 1 } else { 2 };

            // Check if adding this character's UTF-16 length would exceed the specified position
            if utf16_count + char_utf16_len > utf16_column {
                return Some(utf8_column);
            }

            if char == '\n' {
                return None;
            }

            utf16_count += char_utf16_len;
            utf8_column += 1;
        }

        if utf8_column == utf16_count {
            return Some(utf8_column);
        }

        None
    }

    /// Get the UTF8-based position at the character index in the content
    pub fn position8_at_index(&self, char_index: usize) -> Position8 {
        let (line, line_info) = self.find_line(char_index);
        let column = char_index.saturating_sub(line_info.char_index);

        Position8 { line, column }
    }

    /// Get the UTF16-based position at the character index in the content
    pub fn position16_at_index(&self, char_index: usize) -> Position16 {
        let (line, line_info) = self.find_line(char_index);
        let column = self.utf8_to_utf16_column(line_info, char_index);

        Position16 { line, column }
    }

    /// Get the character index at the UTF8-based position in the content
    ///
    /// Returns `None` if the line index is out of bounds or the column index is beyond
    /// the end of the line.
    pub fn index_at_position8(&self, Position8 { line, column }: Position8) -> Option<usize> {
        let line_info = self.get_line(line)?;

        let index = line_info.char_index.saturating_add(column);

        if let Some(next_line_info) = self.get_line(line + 1) {
            (index < next_line_info.char_index).then_some(index)
        } else {
            (index < self.char_count()).then_some(index)
        }
    }

    /// Get the character index at the UTF16-based position in the content
    ///
    /// Returns `None` if the line index is out of bounds or the column index is beyond
    /// the end of the line.
    pub fn index_at_position16(&self, Position16 { line, column }: Position16) -> Option<usize> {
        let line_info = self.get_line(line)?;

        let index = line_info.char_index + self.utf16_to_utf8_column(line_info, column)?;

        if let Some(next_line_info) = self.get_line(line + 1) {
            (index < next_line_info.char_index).then_some(index)
        } else {
            (index < self.char_count()).then_some(index)
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

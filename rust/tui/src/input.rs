/// Text input buffer with a byte-offset cursor.
#[derive(Debug, Default)]
pub struct InputState {
    /// The text content of the input.
    buffer: String,
    /// Cursor position as a byte offset into `buffer`.
    cursor: usize,
}

#[allow(dead_code)]
impl InputState {
    /// The current text content.
    pub fn text(&self) -> &str {
        &self.buffer
    }

    /// The cursor position as a byte offset.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Insert a string at the cursor position.
    pub fn insert_str(&mut self, s: &str) {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    /// Insert a newline at the cursor position.
    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char_before(&mut self) {
        if self.cursor > 0 {
            // Find the start of the previous character
            let prev = self.prev_char_boundary();
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_char_at(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.next_char_boundary();
            self.buffer.drain(self.cursor..next);
        }
    }

    /// Move the cursor one character to the left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    /// Move the cursor one character to the right.
    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    /// Move the cursor to the beginning of the buffer.
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move the cursor to the end of the buffer.
    pub fn move_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    /// Take the buffer contents and reset the input state.
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.buffer)
    }

    /// Clear the buffer and reset the cursor.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    /// Replace a byte range in the buffer and position the cursor at the end of the replacement.
    pub fn replace_range(&mut self, range: std::ops::Range<usize>, replacement: &str) {
        self.buffer.replace_range(range.clone(), replacement);
        self.cursor = range.start + replacement.len();
    }

    /// Replace the buffer contents, placing the cursor at the end.
    pub fn set_text(&mut self, text: &str) {
        self.buffer = text.to_string();
        self.cursor = self.buffer.len();
    }

    /// Count the number of lines in the buffer (at least 1).
    pub fn line_count(&self) -> usize {
        self.buffer.lines().count().max(1) + usize::from(self.buffer.ends_with('\n'))
    }

    /// Whether the input is a single line (no embedded newlines).
    pub fn is_single_line(&self) -> bool {
        !self.buffer.contains('\n')
    }

    /// Whether the cursor is on the first line.
    pub fn is_on_first_line(&self) -> bool {
        self.line_start_before(self.cursor) == 0
    }

    /// Whether the cursor is on the last line.
    pub fn is_on_last_line(&self) -> bool {
        self.line_end_after(self.cursor) == self.buffer.len()
    }

    /// Move the cursor up one line, preserving the column position.
    ///
    /// If the cursor is on the first line, moves to the start of the buffer.
    pub fn move_up(&mut self) {
        let (line_start, col) = self.current_line_start_and_col();

        if line_start == 0 {
            // Already on the first line — move to start
            self.cursor = 0;
            return;
        }

        // Find the start of the previous line (byte before current line's \n)
        let prev_line_end = line_start - 1; // the \n
        let prev_line_start = self.line_start_before(prev_line_end);

        let prev_line_len = prev_line_end - prev_line_start;
        self.cursor = prev_line_start + col.min(prev_line_len);
    }

    /// Move the cursor down one line, preserving the column position.
    ///
    /// If the cursor is on the last line, moves to the end of the buffer.
    pub fn move_down(&mut self) {
        let (_, col) = self.current_line_start_and_col();

        // Find the end of the current line.
        let line_end = self.line_end_after(self.cursor);
        if line_end == self.buffer.len() {
            // Already on the last line — move to end
            self.cursor = self.buffer.len();
            return;
        }

        let next_line_start = line_end + 1;
        let next_line_end = self.line_end_after(next_line_start);

        let next_line_len = next_line_end - next_line_start;
        self.cursor = next_line_start + col.min(next_line_len);
    }

    /// Returns (`line_start_byte_offset`, `column_in_chars`) for the current cursor position.
    fn current_line_start_and_col(&self) -> (usize, usize) {
        let line_start = self.line_start_before(self.cursor);
        let col = self.buffer[line_start..self.cursor].chars().count();
        (line_start, col)
    }

    /// Move the cursor one word to the left.
    ///
    /// Skips whitespace, then moves to the start of the previous word.
    pub fn move_word_left(&mut self) {
        self.cursor = self.word_left_boundary(self.cursor);
    }

    /// Move the cursor one word to the right.
    ///
    /// Skips the current word, then skips whitespace after it.
    pub fn move_word_right(&mut self) {
        self.cursor = self.word_right_boundary(self.cursor);
    }

    /// Delete the word before the cursor (Ctrl+Backspace).
    pub fn delete_word_back(&mut self) {
        let end = self.cursor;
        self.move_word_left();
        self.buffer.drain(self.cursor..end);
    }

    /// Delete the word after the cursor (Ctrl+Delete).
    pub fn delete_word_forward(&mut self) {
        let start = self.cursor;
        self.move_word_right();
        self.buffer.drain(start..self.cursor);
        self.cursor = start;
    }

    /// Delete from cursor to the start of the line (Ctrl+U).
    pub fn delete_to_line_start(&mut self) {
        let line_start = self.line_start_before(self.cursor);
        self.buffer.drain(line_start..self.cursor);
        self.cursor = line_start;
    }

    /// Delete from cursor to the end of the line (Ctrl+K).
    pub fn delete_to_line_end(&mut self) {
        let line_end = self.line_end_after(self.cursor);
        self.buffer.drain(self.cursor..line_end);
    }

    /// Find the start byte of the line containing `pos`.
    fn line_start_before(&self, pos: usize) -> usize {
        self.buffer[..pos].rfind('\n').map_or(0, |index| index + 1)
    }

    /// Find the end byte of the line containing `pos` (newline or buffer end).
    fn line_end_after(&self, pos: usize) -> usize {
        self.buffer[pos..]
            .find('\n')
            .map_or(self.buffer.len(), |index| pos + index)
    }

    fn is_word_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric()
    }

    fn word_left_boundary(&self, mut pos: usize) -> usize {
        let bytes = self.buffer.as_bytes();

        // Skip whitespace/punctuation backwards.
        while pos > 0 && !Self::is_word_byte(bytes[pos - 1]) {
            pos -= 1;
            while pos > 0 && !self.buffer.is_char_boundary(pos) {
                pos -= 1;
            }
        }

        // Skip word characters backwards.
        while pos > 0 && Self::is_word_byte(bytes[pos - 1]) {
            pos -= 1;
        }

        pos
    }

    fn word_right_boundary(&self, mut pos: usize) -> usize {
        let len = self.buffer.len();
        let bytes = self.buffer.as_bytes();

        // Skip word characters forward.
        while pos < len && Self::is_word_byte(bytes[pos]) {
            pos += 1;
        }

        // Skip whitespace/punctuation forward.
        while pos < len && !Self::is_word_byte(bytes[pos]) {
            pos += 1;
        }

        pos
    }

    /// Find the byte offset of the previous character boundary.
    fn prev_char_boundary(&self) -> usize {
        let mut pos = self.cursor.saturating_sub(1);
        while pos > 0 && !self.buffer.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    /// Find the byte offset of the next character boundary.
    fn next_char_boundary(&self) -> usize {
        let mut pos = self.cursor + 1;
        while pos < self.buffer.len() && !self.buffer.is_char_boundary(pos) {
            pos += 1;
        }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_cursor() {
        let mut input = InputState::default();
        assert!(input.is_empty());
        assert_eq!(input.cursor(), 0);

        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.text(), "hi");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn backspace() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.delete_char_before();
        assert_eq!(input.text(), "ab");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn backspace_at_start() {
        let mut input = InputState::default();
        input.delete_char_before();
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_at_cursor() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.insert_char('b');
        input.move_home();
        input.delete_char_at();
        assert_eq!(input.text(), "b");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_at_end() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.delete_char_at();
        assert_eq!(input.text(), "a");
    }

    #[test]
    fn cursor_movement() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');

        input.move_left();
        assert_eq!(input.cursor(), 2);

        input.move_left();
        assert_eq!(input.cursor(), 1);

        input.move_right();
        assert_eq!(input.cursor(), 2);

        input.move_home();
        assert_eq!(input.cursor(), 0);

        input.move_end();
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn left_at_start_stays() {
        let mut input = InputState::default();
        input.move_left();
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn right_at_end_stays() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.move_right();
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn insert_in_middle() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.insert_char('c');
        input.move_left();
        input.insert_char('b');
        assert_eq!(input.text(), "abc");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn newline() {
        let mut input = InputState::default();
        input.insert_char('a');
        input.insert_newline();
        input.insert_char('b');
        assert_eq!(input.text(), "a\nb");
        assert!(!input.is_single_line());
        assert_eq!(input.line_count(), 2);
    }

    #[test]
    fn take_resets() {
        let mut input = InputState::default();
        input.insert_char('x');
        let text = input.take();
        assert_eq!(text, "x");
        assert!(input.is_empty());
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn set_text() {
        let mut input = InputState::default();
        input.set_text("hello");
        assert_eq!(input.text(), "hello");
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn multibyte_chars() {
        let mut input = InputState::default();
        input.insert_char('é');
        input.insert_char('ñ');
        assert_eq!(input.cursor(), 4); // é=2 bytes, ñ=2 bytes

        input.move_left();
        assert_eq!(input.cursor(), 2);

        input.delete_char_before();
        assert_eq!(input.text(), "ñ");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn move_up_basic() {
        let mut input = InputState::default();
        input.set_text("abc\ndef\nghi");
        // cursor at end (pos 11, line 2, col 3)
        input.move_up();
        // Should be on line 1, col 3 → "def" pos 7
        assert_eq!(input.cursor(), 7);
        input.move_up();
        // Should be on line 0, col 3 → "abc" pos 3
        assert_eq!(input.cursor(), 3);
        // Already on first line → move to start
        input.move_up();
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn move_down_basic() {
        let mut input = InputState::default();
        input.set_text("abc\ndef\nghi");
        input.move_home(); // pos 0
        input.move_down();
        // Should be on line 1, col 0 → pos 4
        assert_eq!(input.cursor(), 4);
        input.move_down();
        // Should be on line 2, col 0 → pos 8
        assert_eq!(input.cursor(), 8);
        // Already on last line → move to end
        input.move_down();
        assert_eq!(input.cursor(), 11);
    }

    #[test]
    fn move_up_clamps_column() {
        let mut input = InputState::default();
        input.set_text("abcdef\nhi");
        // cursor at end: line 1, col 2
        input.move_up();
        // line 0 has 6 chars, col 2 fits → pos 2
        assert_eq!(input.cursor(), 2);

        // Now test the other direction: short line above
        let mut input2 = InputState::default();
        input2.set_text("ab\ncdefgh");
        // cursor at end: line 1, col 6
        input2.move_up();
        // line 0 has 2 chars, col clamped to 2 → pos 2
        assert_eq!(input2.cursor(), 2);
    }

    #[test]
    fn move_down_clamps_column() {
        let mut input = InputState::default();
        // Type "abcdef", then newline, then "hi"
        // Position cursor at end of first line (col 6) by navigating
        input.set_text("abcdef\nhi");
        input.move_home(); // pos 0
        // Move right 6 times to reach end of "abcdef"
        for _ in 0..6 {
            input.move_right();
        }
        assert_eq!(input.cursor(), 6);
        input.move_down();
        // line 1 "hi" has 2 chars, col 6 clamped to 2 → pos 9
        assert_eq!(input.cursor(), 9);
    }

    #[test]
    fn move_word_left_basic() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.move_word_left();
        assert_eq!(input.cursor(), 6); // before "world"
        input.move_word_left();
        assert_eq!(input.cursor(), 0); // before "hello"
        // At start, stays
        input.move_word_left();
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn move_word_right_basic() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.move_home();
        input.move_word_right();
        assert_eq!(input.cursor(), 6); // after "hello "
        input.move_word_right();
        assert_eq!(input.cursor(), 11); // after "world"
        // At end, stays
        input.move_word_right();
        assert_eq!(input.cursor(), 11);
    }

    #[test]
    fn move_word_with_punctuation() {
        let mut input = InputState::default();
        input.set_text("foo.bar baz");
        input.move_home();
        input.move_word_right();
        // Skips "foo", then skips "."
        assert_eq!(input.cursor(), 4); // at "bar"
    }

    #[test]
    fn delete_word_back() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.delete_word_back();
        assert_eq!(input.text(), "hello ");
        assert_eq!(input.cursor(), 6);
        input.delete_word_back();
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_word_forward() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.move_home();
        input.delete_word_forward();
        assert_eq!(input.text(), "world");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_to_line_start() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.delete_to_line_start();
        assert_eq!(input.text(), "");

        // With multiline: only delete to start of current line
        input.set_text("line1\nline2");
        // cursor at end (11)
        input.delete_to_line_start();
        assert_eq!(input.text(), "line1\n");
        assert_eq!(input.cursor(), 6);
    }

    #[test]
    fn delete_to_line_end() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.move_home();
        input.delete_to_line_end();
        assert_eq!(input.text(), "");

        // With multiline
        input.set_text("line1\nline2");
        input.move_home();
        input.delete_to_line_end();
        assert_eq!(input.text(), "\nline2");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn line_count_empty() {
        let input = InputState::default();
        assert_eq!(input.line_count(), 1);
    }

    #[test]
    fn line_count_trailing_newline() {
        let mut input = InputState::default();
        input.set_text("a\n");
        assert_eq!(input.line_count(), 2);
    }

    #[test]
    fn replace_range_middle() {
        let mut input = InputState::default();
        input.set_text("hello world");
        input.replace_range(5..11, " rust");
        assert_eq!(input.text(), "hello rust");
        assert_eq!(input.cursor(), 10);
    }

    #[test]
    fn replace_range_start() {
        let mut input = InputState::default();
        input.set_text("@foo bar");
        // Replace "@foo" with "@path/to/file.rs " — note the space after "bar" is preserved
        input.replace_range(0..4, "@path/to/file.rs");
        assert_eq!(input.text(), "@path/to/file.rs bar");
        assert_eq!(input.cursor(), 16);
    }

    #[test]
    fn replace_range_end() {
        let mut input = InputState::default();
        input.set_text("prefix ./sr");
        input.replace_range(7..11, "./src/");
        assert_eq!(input.text(), "prefix ./src/");
        assert_eq!(input.cursor(), 13);
    }

    #[test]
    fn replace_range_empty_replacement() {
        let mut input = InputState::default();
        input.set_text("abc");
        input.replace_range(1..2, "");
        assert_eq!(input.text(), "ac");
        assert_eq!(input.cursor(), 1);
    }
}

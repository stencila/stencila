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
}

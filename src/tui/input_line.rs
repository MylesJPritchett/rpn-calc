/// A single line of text with a cursor measured in characters, not bytes.
#[derive(Debug, Default)]
pub struct InputLine {
    text: String,
    cursor: usize,
}

impl InputLine {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text.chars().count());
    }

    pub fn insert(&mut self, c: char) {
        let index = self.byte_index();
        self.text.insert(index, c);
        self.move_right();
    }

    pub fn delete_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let before = self.text.chars().take(self.cursor - 1);
        let after = self.text.chars().skip(self.cursor);
        self.text = before.chain(after).collect();
        self.move_left();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map_or(self.text.len(), |(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::InputLine;

    fn typed(text: &str) -> InputLine {
        let mut line = InputLine::default();
        for c in text.chars() {
            line.insert(c);
        }
        line
    }

    #[test]
    fn insert_appends_and_advances_cursor() {
        let line = typed("a");
        assert_eq!(line.text(), "a");
        assert_eq!(line.cursor(), 1);
    }

    #[test]
    fn cursor_moves_left_and_right() {
        let mut line = typed("hello");
        line.move_left();
        line.move_left();
        assert_eq!(line.cursor(), 3);
        line.move_right();
        assert_eq!(line.cursor(), 4);
    }

    #[test]
    fn cursor_stays_within_text() {
        let mut line = typed("hi");
        line.move_right();
        assert_eq!(line.cursor(), 2);
        for _ in 0..5 {
            line.move_left();
        }
        assert_eq!(line.cursor(), 0);
    }

    #[test]
    fn insert_at_cursor() {
        let mut line = typed("hllo");
        line.move_left();
        line.move_left();
        line.move_left();
        line.insert('e');
        assert_eq!(line.text(), "hello");
        assert_eq!(line.cursor(), 2);
    }

    #[test]
    fn delete_removes_character_before_cursor() {
        let mut line = typed("hello");
        line.move_left();
        line.move_left();
        line.delete_before_cursor();
        assert_eq!(line.text(), "helo");
        assert_eq!(line.cursor(), 2);
    }

    #[test]
    fn delete_at_start_does_nothing() {
        let mut line = typed("hi");
        line.move_left();
        line.move_left();
        line.delete_before_cursor();
        assert_eq!(line.text(), "hi");
        assert_eq!(line.cursor(), 0);
    }

    #[test]
    fn multibyte_characters_are_handled_by_character() {
        let mut line = typed("aπé");
        line.move_left();
        line.insert('√');
        assert_eq!(line.text(), "aπ√é");
        line.delete_before_cursor();
        line.delete_before_cursor();
        assert_eq!(line.text(), "aé");
        assert_eq!(line.cursor(), 1);
    }

    #[test]
    fn clear_empties_text_and_resets_cursor() {
        let mut line = typed("hello");
        line.clear();
        assert_eq!(line.text(), "");
        assert_eq!(line.cursor(), 0);
    }
}

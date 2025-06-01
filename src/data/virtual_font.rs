#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct VirtualFont {
    text: String,
    cursor_position: usize,
}

#[allow(dead_code)]
impl VirtualFont {
    /// Create a new virtual font
    pub fn new() -> Self {
        Self::default()
    }

    /// Add text at the current cursor position
    pub fn add_text(&mut self, text: &str) {
        self.text.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    /// Remove the character before the cursor
    pub fn backspace(&mut self) -> bool {
        if self.cursor_position > 0 {
            self.text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            true
        } else {
            false
        }
    }

    /// Get the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Set the cursor position
    pub fn set_cursor_position(&mut self, pos: usize) {
        self.cursor_position = pos.min(self.text.len());
    }
}

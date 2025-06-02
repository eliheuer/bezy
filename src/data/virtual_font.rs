//! Virtual font text editing functionality

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct VirtualFont {
    /// The current text content
    text: String,
    /// Current position of the cursor (0 = start of text)
    cursor_position: usize,
}

#[allow(dead_code)]
impl VirtualFont {
    /// Create a new empty text editor
    pub fn new() -> Self {
        Self::default()
    }

    /// Add text at the current cursor position
    pub fn add_text(&mut self, text: &str) {
        self.text.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    /// Remove the character before the cursor (like pressing backspace)
    pub fn backspace(&mut self) -> bool {
        if self.cursor_position == 0 {
            return false;
        }
        
        self.text.remove(self.cursor_position - 1);
        self.cursor_position -= 1;
        true
    }

    /// Get the current text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Move the cursor to a new position
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.min(self.text.len());
    }
}

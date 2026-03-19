pub struct TextInput {
    pub buffer: String,
    pub cursor: usize,
    pub prompt: String,
    history: Vec<String>,
    history_index: Option<usize>,
    saved_buffer: String,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            prompt: "> ".to_string(),
            history: Vec::new(),
            history_index: None,
            saved_buffer: String::new(),
        }
    }

    pub fn with_prompt(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            ..Self::new()
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.buffer.len();
    }

    pub fn submit(&mut self) -> String {
        let content = std::mem::take(&mut self.buffer);
        if !content.is_empty() {
            self.history.push(content.clone());
        }
        self.cursor = 0;
        self.history_index = None;
        self.saved_buffer.clear();
        content
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn cancel(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = None;
        self.saved_buffer.clear();
    }

    pub fn content(&self) -> &str {
        &self.buffer
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.saved_buffer = self.buffer.clone();
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return,
            Some(i) => {
                self.history_index = Some(i - 1);
            }
        }

        self.buffer = self.history[self.history_index.unwrap()].clone();
        self.cursor = self.buffer.len();
    }

    pub fn history_down(&mut self) {
        let i = match self.history_index {
            None => return,
            Some(i) => i,
        };

        if i + 1 >= self.history.len() {
            self.history_index = None;
            self.buffer = std::mem::take(&mut self.saved_buffer);
        } else {
            self.history_index = Some(i + 1);
            self.buffer = self.history[i + 1].clone();
        }

        self.cursor = self.buffer.len();
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

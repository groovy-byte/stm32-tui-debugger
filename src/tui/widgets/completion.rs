/// A completion/autocomplete dropdown state manager.
/// Pure data/logic — rendering handled by pane code.
#[derive(Clone, Debug)]
pub struct CompletionItem {
    pub text: String,
    pub detail: String,
}

pub struct CompletionList {
    items: Vec<CompletionItem>,
    selected: usize,
    max_visible: usize,
    scroll_offset: usize,
}

impl CompletionList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            max_visible: 8,
            scroll_offset: 0,
        }
    }

    pub fn update(&mut self, items: Vec<CompletionItem>) {
        self.items = items;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_active(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn items(&self) -> &[CompletionItem] {
        &self.items
    }

    pub fn visible_items(&self) -> &[CompletionItem] {
        let end = (self.scroll_offset + self.max_visible).min(self.items.len());
        &self.items[self.scroll_offset..end]
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn visible_selected(&self) -> usize {
        self.selected - self.scroll_offset
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected + 1 >= self.items.len() {
            self.selected = 0;
            self.scroll_offset = 0;
        } else {
            self.selected += 1;
            if self.selected >= self.scroll_offset + self.max_visible {
                self.scroll_offset = self.selected - self.max_visible + 1;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
            self.scroll_offset = self.items.len().saturating_sub(self.max_visible);
        } else {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn accept(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn set_max_visible(&mut self, n: usize) {
        self.max_visible = n.max(1);
    }
}

impl Default for CompletionList {
    fn default() -> Self {
        Self::new()
    }
}

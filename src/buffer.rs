use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    Focused,
    Full,
}

#[derive(Debug, Clone)]
struct HistoryState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}

#[derive(Debug, Clone)]
pub struct TypewriterBuffer {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub mode: ViewMode,
    pub decay_rate: f32,
    pub min_opacity: f32,
    pub file_path: Option<PathBuf>,
    pub selection: Option<((usize, usize), (usize, usize))>,
    history: Vec<HistoryState>,
    redo_stack: Vec<HistoryState>,
}

impl TypewriterBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            mode: ViewMode::Focused,
            decay_rate: 0.35, // High decay for quick disappearance
            min_opacity: 0.0,  // Allow complete disappearance
            file_path: None,
            selection: None,
            history: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Creates a buffer initialised with values from the user config
    pub fn new_with_config(decay_rate: f32, min_opacity: f32) -> Self {
        let mut b = Self::new();
        b.decay_rate = decay_rate;
        b.min_opacity = min_opacity;
        b
    }

    pub fn push_history(&mut self) {
        self.history.push(HistoryState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        });
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(state) = self.history.pop() {
            self.redo_stack.push(HistoryState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = state.lines;
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
            self.selection = None;
        }
    }

    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.history.push(HistoryState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = state.lines;
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
            self.selection = None;
        }
    }

    pub fn toggle_view(&mut self) {
        self.mode = match self.mode {
            ViewMode::Focused => ViewMode::Full,
            ViewMode::Full => ViewMode::Focused,
        };
    }

    pub fn update_config(&mut self, decay_rate: f32, min_opacity: f32) {
        self.decay_rate = decay_rate;
        self.min_opacity = min_opacity;
    }

    /// Opacity of a line at distance `dist` from the cursor line.
    pub fn line_opacity(&self, dist: usize) -> f32 {
        if dist == 0 {
            return 1.0;
        }
        (1.0 - (dist as f32 * self.decay_rate)).max(self.min_opacity)
    }

    pub fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        self.selection.map(|(start, end)| {
            if start <= end { (start, end) } else { (end, start) }
        })
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let ((s_line, s_col), (e_line, e_col)) = self.get_selection_range()?;
        let mut result = String::new();

        if s_line == e_line {
            result.push_str(&self.lines[s_line][s_col..e_col]);
        } else {
            result.push_str(&self.lines[s_line][s_col..]);
            result.push('\n');
            for i in (s_line + 1)..e_line {
                result.push_str(&self.lines[i]);
                result.push('\n');
            }
            result.push_str(&self.lines[e_line][..e_col]);
        }
        Some(result)
    }

    pub fn delete_selection(&mut self) -> bool {
        let ((s_line, s_col), (e_line, e_col)) = match self.get_selection_range() {
            Some(range) => range,
            None => return false,
        };

        if s_line == e_line {
            self.lines[s_line].replace_range(s_col..e_col, "");
        } else {
            let suffix = self.lines[e_line][e_col..].to_string();
            self.lines[s_line].truncate(s_col);
            self.lines[s_line].push_str(&suffix);
            self.lines.drain((s_line + 1)..=e_line);
        }

        self.cursor_line = s_line;
        self.cursor_col = s_col;
        self.selection = None;
        true
    }

    pub fn select_all(&mut self) {
        let last_line = self.lines.len() - 1;
        let last_col = self.lines[last_line].len();
        self.selection = Some(((0, 0), (last_line, last_col)));
        self.cursor_line = last_line;
        self.cursor_col = last_col;
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();

        if c == '\n' {
            self.insert_newline();
            return;
        }
        
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            if self.cursor_col <= line.len() {
                line.insert(self.cursor_col, c);
                self.cursor_col += c.len_utf8();
            }
        }
    }

    pub fn insert_newline(&mut self) {
        self.delete_selection();

        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            let remainder = if self.cursor_col <= line.len() {
                line.split_off(self.cursor_col)
            } else {
                String::new()
            };
            
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.lines.insert(self.cursor_line, remainder);
        }
    }

    pub fn delete_backwards(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor_line < self.lines.len() {
            if self.cursor_col > 0 {
                let line = &mut self.lines[self.cursor_line];
                // Support unicode characters deletion
                if let Some((idx, c)) = line[..self.cursor_col].char_indices().last() {
                    line.remove(idx);
                    self.cursor_col -= c.len_utf8();
                }
            } else if self.cursor_line > 0 {
                let current_line = self.lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                
                let prev_line = &mut self.lines[self.cursor_line];
                self.cursor_col = prev_line.len();
                prev_line.push_str(&current_line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_newline() {
        let mut buf = TypewriterBuffer::new();
        buf.insert_char('H');
        buf.insert_char('i');
        buf.insert_newline();
        buf.insert_char('!');
        
        assert_eq!(buf.lines.len(), 2);
        assert_eq!(buf.lines[0], "Hi");
        assert_eq!(buf.lines[1], "!");
        assert_eq!(buf.cursor_line, 1);
        assert_eq!(buf.cursor_col, 1);
    }

    #[test]
    fn test_delete_backwards() {
        let mut buf = TypewriterBuffer::new();
        buf.insert_char('a');
        buf.insert_newline();
        buf.insert_char('b');
        buf.delete_backwards(); // deletes 'b'
        assert_eq!(buf.lines[1], "");
        buf.delete_backwards(); // deletes newline
        assert_eq!(buf.lines.len(), 1);
        assert_eq!(buf.cursor_line, 0);
        assert_eq!(buf.cursor_col, 1);
    }
}

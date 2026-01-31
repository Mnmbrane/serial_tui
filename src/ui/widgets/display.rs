//! Main display area for serial output.
//!
//! Uses a circular buffer (VecDeque) with 10,000 line limit.
//! Supports cursor-based scrolling with margin-based auto-scroll.
//!
//! Lines are stored as pre-rendered `Line<'static>` for efficiency.
//! Cursor highlighting is applied at render time.

use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use super::focused_block;

/// Actions the display widget can request.
pub enum DisplayAction {
    /// Move focus to the input bar
    FocusInput,
    /// Notify user of yank result
    Notify(String),
}

/// Main area for displaying serial port output.
///
/// Uses a VecDeque as a circular buffer for efficient push/pop.
/// Cursor-based scrolling with 25% margin triggers auto-scroll.
pub struct Display {
    /// Circular buffer of pre-rendered display lines (max 10,000)
    lines: VecDeque<Line<'static>>,
    /// Current cursor position (absolute index in buffer)
    cursor: usize,
    /// First visible line index
    view_start: usize,
    /// Tracks if 'g' was pressed (for gg sequence)
    pending_g: bool,
    /// Visual selection start (None = not in visual mode)
    selection_start: Option<usize>,
    /// Whether we're in search input mode
    search_mode: bool,
    /// Current search query
    search_query: String,
    /// Indices of lines matching the search
    search_matches: Vec<usize>,
    /// Current match index (for n/N navigation)
    search_match_idx: usize,
    /// Clipboard instance kept alive for Linux compatibility
    clipboard: Option<arboard::Clipboard>,
}

impl Display {
    /// Maximum lines to keep in buffer
    const MAX_LINES: usize = 10_000;
    /// Scroll margin as fraction of visible height (25%)
    const SCROLL_MARGIN: f32 = 0.25;

    /// Creates a new empty display.
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            cursor: 0,
            view_start: 0,
            pending_g: false,
            selection_start: None,
            search_mode: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_idx: 0,
            clipboard: None,
        }
    }

    /// Adds a pre-styled line to the buffer, removing oldest if at capacity.
    /// Auto-scrolls to bottom by moving cursor to the new line.
    pub fn push_line(&mut self, line: Line<'static>) {
        if self.lines.len() >= Self::MAX_LINES {
            self.lines.pop_front();
            // Adjust view if it was pointing at removed line
            self.view_start = self.view_start.saturating_sub(1);
        }
        self.lines.push_back(line);
        // Auto-scroll: move cursor to the last line
        self.cursor = self.lines.len().saturating_sub(1);
    }

    /// Moves cursor up one line.
    pub fn move_up(&mut self, height: usize) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.adjust_scroll(height);
        }
    }

    /// Moves cursor down one line.
    pub fn move_down(&mut self, height: usize) {
        if self.cursor < self.lines.len().saturating_sub(1) {
            self.cursor += 1;
            self.adjust_scroll(height);
        }
    }

    /// Moves cursor up half a page (Ctrl+u).
    pub fn half_page_up(&mut self, height: usize) {
        let half = height / 2;
        self.cursor = self.cursor.saturating_sub(half);
        self.adjust_scroll(height);
    }

    /// Moves cursor down half a page (Ctrl+d).
    pub fn half_page_down(&mut self, height: usize) {
        let half = height / 2;
        let max_cursor = self.lines.len().saturating_sub(1);
        self.cursor = (self.cursor + half).min(max_cursor);
        self.adjust_scroll(height);
    }

    /// Moves cursor to the first line (gg).
    pub fn go_to_top(&mut self, height: usize) {
        self.cursor = 0;
        self.adjust_scroll(height);
    }

    /// Moves cursor to the last line (G).
    pub fn go_to_bottom(&mut self, height: usize) {
        self.cursor = self.lines.len().saturating_sub(1);
        self.adjust_scroll(height);
    }

    /// Toggles visual selection mode.
    /// If not in visual mode, starts selection at cursor.
    /// If in visual mode, exits visual mode.
    pub fn toggle_visual(&mut self) {
        if self.selection_start.is_some() {
            self.selection_start = None;
        } else {
            self.selection_start = Some(self.cursor);
        }
    }

    /// Returns true if in visual selection mode.
    pub fn in_visual_mode(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Returns the selection range as (start, end) inclusive.
    /// Returns None if not in visual mode.
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let (a, b) = (start, self.cursor);
            (a.min(b), a.max(b))
        })
    }

    /// Returns true if the given line index is within the selection.
    fn is_selected(&self, idx: usize) -> bool {
        self.selection_range()
            .map(|(start, end)| idx >= start && idx <= end)
            .unwrap_or(false)
    }

    /// Gets the text content of selected lines (for yank).
    /// Returns the current line if not in visual mode.
    pub fn get_selected_text(&self) -> String {
        let (start, end) = self.selection_range().unwrap_or((self.cursor, self.cursor));

        self.lines
            .iter()
            .skip(start)
            .take(end - start + 1)
            .map(|line| {
                // Extract raw text from Line's spans
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Yanks (copies) selected text to clipboard.
    /// Returns Ok(num_lines) on success, Err on clipboard failure.
    /// Keeps clipboard alive for Linux compatibility.
    pub fn yank(&mut self) -> Result<usize, arboard::Error> {
        let text = self.get_selected_text();
        let num_lines = self.selection_range().map(|(s, e)| e - s + 1).unwrap_or(1);

        // Initialize clipboard if not already done, then reuse it
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new()?);
        }

        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(text)?;
        }

        // Exit visual mode after yank
        self.selection_start = None;
        Ok(num_lines)
    }

    /// Returns true if in search input mode.
    pub fn in_search_mode(&self) -> bool {
        self.search_mode
    }

    /// Enters search input mode.
    pub fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_matches.clear();
        self.search_match_idx = 0;
    }

    /// Exits search input mode and executes the search.
    pub fn finish_search(&mut self, height: usize) {
        self.search_mode = false;
        self.execute_search(height);
    }

    /// Cancels search mode without executing.
    pub fn cancel_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_matches.clear();
    }

    /// Adds a character to the search query.
    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
    }

    /// Removes the last character from the search query.
    pub fn search_pop(&mut self) {
        self.search_query.pop();
    }

    /// Returns the current search query.
    #[allow(dead_code)]
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Executes the search and populates matches.
    fn execute_search(&mut self, height: usize) {
        self.search_matches.clear();
        self.search_match_idx = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        for (idx, line) in self.lines.iter().enumerate() {
            // Extract text from line spans
            let text: String = line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect();

            if text.to_lowercase().contains(&query_lower) {
                self.search_matches.push(idx);
            }
        }

        // Jump to first match if any
        if !self.search_matches.is_empty() {
            self.cursor = self.search_matches[0];
            self.adjust_scroll(height);
        }
    }

    /// Jumps to the next search match.
    pub fn next_match(&mut self, height: usize) {
        if self.search_matches.is_empty() {
            return;
        }

        self.search_match_idx = (self.search_match_idx + 1) % self.search_matches.len();
        self.cursor = self.search_matches[self.search_match_idx];
        self.adjust_scroll(height);
    }

    /// Jumps to the previous search match.
    pub fn prev_match(&mut self, height: usize) {
        if self.search_matches.is_empty() {
            return;
        }

        self.search_match_idx = if self.search_match_idx == 0 {
            self.search_matches.len() - 1
        } else {
            self.search_match_idx - 1
        };
        self.cursor = self.search_matches[self.search_match_idx];
        self.adjust_scroll(height);
    }

    /// Returns true if the given line index is a search match.
    fn is_match(&self, idx: usize) -> bool {
        self.search_matches.contains(&idx)
    }

    /// Adjusts view_start to keep cursor within scroll margins.
    fn adjust_scroll(&mut self, height: usize) {
        if height == 0 {
            return;
        }

        let margin = (height as f32 * Self::SCROLL_MARGIN) as usize;
        let cursor_in_view = self.cursor.saturating_sub(self.view_start);

        // Cursor in top 25% → scroll up
        if cursor_in_view < margin {
            self.view_start = self.cursor.saturating_sub(margin);
        }

        // Cursor in bottom 25% → scroll down
        if cursor_in_view >= height.saturating_sub(margin) {
            self.view_start = self
                .cursor
                .saturating_sub(height.saturating_sub(margin).saturating_sub(1));
        }

        // Clamp to valid range
        let max_start = self.lines.len().saturating_sub(height);
        self.view_start = self.view_start.min(max_start);
    }

    /// Returns iterator over visible lines for the current view.
    /// Uses VecDeque::range() for efficient slice-like access.
    pub fn visible_lines(&self, height: usize) -> impl Iterator<Item = (usize, &Line<'static>)> {
        let end = (self.view_start + height).min(self.lines.len());
        self.lines
            .range(self.view_start..end)
            .enumerate()
            .map(move |(i, line)| (self.view_start + i, line))
    }

    /// Renders the display with highlighted cursor, selection, and search matches.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        // Update block title to show mode indicators
        let title = if self.in_search_mode() {
            " Display [SEARCH] ".to_string()
        } else if self.in_visual_mode() {
            " Display [VISUAL] ".to_string()
        } else if !self.search_matches.is_empty() {
            // Show match count when search is active
            format!(
                " Display [{}/{}] ",
                self.search_match_idx + 1,
                self.search_matches.len()
            )
        } else {
            " Display ".to_string()
        };
        let block = focused_block(&title, focused);
        let inner = block.inner(area);

        // Reserve one line for search input when in search mode
        let content_height = if self.in_search_mode() {
            inner.height.saturating_sub(1) as usize
        } else {
            inner.height as usize
        };

        // Adjust scroll for current height
        self.adjust_scroll(content_height);

        let cursor_style = Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);

        let selection_style = Style::default().bg(Color::Blue);

        let match_style = Style::default().bg(Color::Yellow).fg(Color::Black);

        // Pre-allocate Vec to avoid resizing during iteration
        let mut lines = Vec::with_capacity(content_height);

        // Build visible lines, applying cursor/selection/match highlight
        for (idx, line) in self.visible_lines(content_height) {
            let styled_line = if idx == self.cursor {
                // Cursor line gets cursor style
                line.clone().style(cursor_style)
            } else if self.is_selected(idx) {
                // Selected lines get selection style
                line.clone().style(selection_style)
            } else if self.is_match(idx) {
                // Search match lines get match style
                line.clone().style(match_style)
            } else {
                // Normal lines
                line.clone()
            };
            lines.push(styled_line);
        }

        // Add search input line when in search mode
        if self.in_search_mode() {
            lines.push(Line::styled(
                format!("/{}", self.search_query),
                Style::default().fg(Color::Cyan),
            ));
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Handles key input when this widget is focused.
    ///
    /// - `j` / `Down` -> Move cursor down
    /// - `k` / `Up` -> Move cursor up
    /// - `Ctrl+u` -> Half page up
    /// - `Ctrl+d` -> Half page down
    /// - `gg` -> Go to top
    /// - `G` -> Go to bottom
    /// - `v` / `V` -> Toggle visual selection mode
    /// - `y` -> Yank (copy) selected lines to clipboard
    /// - `/` -> Start search mode
    /// - `n` -> Next search match
    /// - `N` -> Previous search match
    /// - `Esc` -> Exit visual/search mode (if active)
    /// - `Enter` -> Move focus to input bar (or execute search in search mode)
    pub fn handle_key(&mut self, key: KeyEvent, height: usize) -> Option<DisplayAction> {
        // Handle search mode input
        if self.in_search_mode() {
            match key.code {
                KeyCode::Esc => {
                    self.cancel_search();
                }
                KeyCode::Enter => {
                    self.finish_search(height);
                }
                KeyCode::Backspace => {
                    self.search_pop();
                }
                KeyCode::Char(c) => {
                    self.search_push(c);
                }
                _ => {}
            }
            return None;
        }

        // Handle 'gg' sequence
        if self.pending_g {
            self.pending_g = false;
            if key.code == KeyCode::Char('g') {
                self.go_to_top(height);
                return None;
            }
            // If not 'g', fall through to normal handling
        }

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.half_page_up(height);
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                self.half_page_down(height);
                None
            }
            (_, KeyCode::Char('g')) => {
                // First 'g' - wait for second
                self.pending_g = true;
                None
            }
            (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                self.go_to_bottom(height);
                None
            }
            // Start search mode
            (_, KeyCode::Char('/')) => {
                self.start_search();
                None
            }
            // Next search match
            (_, KeyCode::Char('n')) => {
                self.next_match(height);
                None
            }
            // Previous search match
            (KeyModifiers::SHIFT, KeyCode::Char('N')) => {
                self.prev_match(height);
                None
            }
            // Visual mode toggle (v and V do the same thing - line selection)
            (_, KeyCode::Char('v')) | (KeyModifiers::SHIFT, KeyCode::Char('V')) => {
                self.toggle_visual();
                None
            }
            // Yank selected text to clipboard
            (_, KeyCode::Char('y')) => match self.yank() {
                Ok(n) => Some(DisplayAction::Notify(format!("Yanked {n} line(s)"))),
                Err(e) => Some(DisplayAction::Notify(format!("Yank failed: {e}"))),
            },
            // Escape exits visual mode (doesn't exit app when in visual)
            (_, KeyCode::Esc) if self.in_visual_mode() => {
                self.selection_start = None;
                None
            }
            (_, KeyCode::Char('k') | KeyCode::Up) => {
                self.move_up(height);
                None
            }
            (_, KeyCode::Char('j') | KeyCode::Down) => {
                self.move_down(height);
                None
            }
            (_, KeyCode::Enter) => Some(DisplayAction::FocusInput),
            _ => None,
        }
    }
}

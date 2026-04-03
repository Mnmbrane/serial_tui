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

/// Mutually exclusive interaction modes for the display widget.
enum DisplayMode {
    /// Normal navigation mode.
    Normal,
    /// Visual line-selection mode.
    Visual {
        /// Anchor line index where the visual selection started.
        selection_start: usize,
    },
    /// Search input mode (`/` prompt is active).
    Search {
        /// In-progress query typed after `/`.
        search_query: String,
    },
}

/// Search results from the last executed query.
struct SearchState {
    /// Cached line indices that match the last completed search.
    matches: Vec<usize>,
    /// Current index within `matches` for n/N navigation.
    match_idx: usize,
}

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
    /// Mode that display is currently in.
    mode: DisplayMode,
    /// Search result cache used by normal/visual mode.
    search: SearchState,
    /// Circular buffer of pre-rendered display lines (max 10,000)
    lines: VecDeque<Line<'static>>,
    /// Current cursor position (absolute index in buffer)
    cursor: usize,
    /// First visible line index
    view_start: usize,
    /// Tracks if 'g' was pressed (for gg sequence)
    pending_g: bool,
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
            mode: DisplayMode::Normal,
            search: SearchState {
                matches: Vec::new(),
                match_idx: 0,
            },
            lines: VecDeque::new(),
            cursor: 0,
            view_start: 0,
            pending_g: false,
            clipboard: None,
        }
    }

    /// Clears all lines and resets display state.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.cursor = 0;
        self.view_start = 0;
        self.pending_g = false;
        self.mode = DisplayMode::Normal;
        self.search.matches.clear();
        self.search.match_idx = 0;
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

    /// Toggles visual selection mode.
    /// If not in visual mode, starts selection at cursor.
    /// If in visual mode, exits visual mode.
    pub fn toggle_visual(&mut self) {
        match self.mode {
            DisplayMode::Normal => {
                self.mode = DisplayMode::Visual {
                    selection_start: self.cursor,
                };
            }
            DisplayMode::Visual { .. } => {
                self.mode = DisplayMode::Normal;
            }
            DisplayMode::Search { .. } => {}
        }
    }

    /// Returns the selection range as (start, end) inclusive.
    /// Returns None if not in visual mode.
    fn selection_range(&self) -> Option<(usize, usize)> {
        match self.mode {
            DisplayMode::Visual { selection_start } => {
                let (a, b) = (selection_start, self.cursor);
                Some((a.min(b), a.max(b)))
            }
            _ => None,
        }
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
        if matches!(self.mode, DisplayMode::Visual { .. }) {
            self.mode = DisplayMode::Normal;
        }

        Ok(num_lines)
    }

    /// Exits search input mode and executes the search.
    pub fn finish_search(&mut self, height: usize) {
        let DisplayMode::Search { search_query } =
            std::mem::replace(&mut self.mode, DisplayMode::Normal)
        else {
            return;
        };

        self.search.matches.clear();
        self.search.match_idx = 0;

        if !search_query.is_empty() {
            let query_lower = search_query.to_lowercase();

            for (idx, line) in self.lines.iter().enumerate() {
                let text: String = line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect();

                if text.to_lowercase().contains(&query_lower) {
                    self.search.matches.push(idx);
                }
            }
        }

        if let Some(&first) = self.search.matches.first() {
            self.cursor = first;
            self.adjust_scroll(height);
        }
    }

    /// Cancels search mode without executing.
    pub fn cancel_search(&mut self) {
        if matches!(self.mode, DisplayMode::Search { .. }) {
            self.mode = DisplayMode::Normal;
        }
    }

    /// Adds a character to the search query.
    pub fn search_push(&mut self, c: char) {
        if let DisplayMode::Search { search_query } = &mut self.mode {
            search_query.push(c);
        }
    }

    /// Removes the last character from the search query.
    pub fn search_pop(&mut self) {
        if let DisplayMode::Search { search_query } = &mut self.mode {
            search_query.pop();
        }
    }

    /// Jumps to the next search match.
    pub fn next_match(&mut self, height: usize) {
        if self.search.matches.is_empty() {
            return;
        }

        self.search.match_idx = (self.search.match_idx + 1) % self.search.matches.len();
        self.cursor = self.search.matches[self.search.match_idx];
        self.adjust_scroll(height);
    }

    /// Jumps to the previous search match.
    pub fn prev_match(&mut self, height: usize) {
        if self.search.matches.is_empty() {
            return;
        }

        self.search.match_idx = if self.search.match_idx == 0 {
            self.search.matches.len() - 1
        } else {
            self.search.match_idx - 1
        };
        self.cursor = self.search.matches[self.search.match_idx];
        self.adjust_scroll(height);
    }

    /// Returns true if the given line index is a search match.
    fn is_match(&self, idx: usize) -> bool {
        self.search.matches.binary_search(&idx).is_ok()
    }

    /// Adjusts view_start to keep cursor within scroll margins.
    fn adjust_scroll(&mut self, height: usize) {
        if height == 0 {
            return;
        }

        let margin = (height as f32 * Self::SCROLL_MARGIN) as usize;
        let cursor_in_view = self.cursor.saturating_sub(self.view_start);

        // Cursor in top 25% -> scroll up
        if cursor_in_view < margin {
            self.view_start = self.cursor.saturating_sub(margin);
        }

        // Cursor in bottom 25% -> scroll down
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
        let title = match self.mode {
            DisplayMode::Search { .. } => " Display [SEARCH] ".to_string(),
            DisplayMode::Visual { .. } => " Display [VISUAL] ".to_string(),
            DisplayMode::Normal if !self.search.matches.is_empty() => {
                format!(
                    " Display [{}/{}] ",
                    self.search.match_idx + 1,
                    self.search.matches.len()
                )
            }
            DisplayMode::Normal => " Display ".to_string(),
        };

        let block = focused_block(&title, focused);
        let inner = block.inner(area);

        // Reserve one line for search input when in search mode
        let content_height = if matches!(self.mode, DisplayMode::Search { .. }) {
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
        if let DisplayMode::Search { search_query } = &self.mode {
            lines.push(Line::styled(
                format!("/{search_query}"),
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
        if matches!(self.mode, DisplayMode::Search { .. }) {
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

        match (key.modifiers, key.code) {
            // Go half way up the page
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                let half = height / 2;
                self.cursor = self.cursor.saturating_sub(half);
                self.adjust_scroll(height);
                None
            }
            // Go half way down the page
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                let half = height / 2;
                let max_cursor = self.lines.len().saturating_sub(1);
                self.cursor = (self.cursor + half).min(max_cursor);
                self.adjust_scroll(height);
                None
            }
            (_, KeyCode::Char('g')) => {
                if self.pending_g == true {
                    self.pending_g = false;
                    self.cursor = 0;
                    self.adjust_scroll(height);
                } else {
                    // First 'g' - wait for second
                    self.pending_g = true;
                }
                None
            }
            // Go to bottom
            (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                self.cursor = self.lines.len().saturating_sub(1);
                self.adjust_scroll(height);
                None
            }
            // Start search mode
            (_, KeyCode::Char('/')) => {
                self.mode = DisplayMode::Search {
                    search_query: String::new(),
                };
                self.pending_g = false;
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
            (_, KeyCode::Esc) => {
                if matches!(self.mode, DisplayMode::Visual { .. }) {
                    self.mode = DisplayMode::Normal;
                }
                None
            }
            (_, KeyCode::Char('k') | KeyCode::Up) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.adjust_scroll(height);
                }
                None
            }
            (_, KeyCode::Char('j') | KeyCode::Down) => {
                if self.cursor < self.lines.len().saturating_sub(1) {
                    self.cursor += 1;
                    self.adjust_scroll(height);
                }
                None
            }
            (_, KeyCode::Enter) => Some(DisplayAction::FocusInput),
            _ => None,
        }
    }
}

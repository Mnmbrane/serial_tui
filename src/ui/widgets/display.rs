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
        }
    }

    /// Adds a line to the buffer, removing oldest if at capacity.
    /// Auto-scrolls to bottom by moving cursor to the new line.
    /// Converts the String to a pre-rendered Line<'static>.
    pub fn push_line(&mut self, text: String) {
        if self.lines.len() >= Self::MAX_LINES {
            self.lines.pop_front();
            // Adjust view if it was pointing at removed line
            self.view_start = self.view_start.saturating_sub(1);
        }
        // Convert String to owned Line<'static>
        self.lines.push_back(Line::raw(text));
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

    /// Renders the display with highlighted cursor line.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let block = focused_block(" Display ", focused);
        let inner = block.inner(area);
        let height = inner.height as usize;

        // Adjust scroll for current height
        self.adjust_scroll(height);

        let cursor_style = Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);

        // Build visible lines, applying cursor highlight only to the selected line
        let lines: Vec<Line> = self
            .visible_lines(height)
            .map(|(idx, line)| {
                if idx == self.cursor {
                    // Clone and apply cursor style
                    line.clone().style(cursor_style)
                } else {
                    // Use pre-rendered line as-is (clone is cheap for Line)
                    line.clone()
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Handles key input when this widget is focused.
    ///
    /// - `j` / `Down` -> Move cursor down
    /// - `k` / `Up` -> Move cursor up
    /// - `Ctrl+u` -> Half page up
    /// - `Ctrl+d` -> Half page down
    /// - `Enter` -> Move focus to input bar
    pub fn handle_key(&mut self, key: KeyEvent, height: usize) -> Option<DisplayAction> {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                self.half_page_up(height);
                None
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                self.half_page_down(height);
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

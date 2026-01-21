//! Toast-style notification overlay.
//!
//! Shows brief messages in the top-right corner with automatic
//! dismissal based on message length.

use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Auto-dismissing notification toast.
///
/// Appears in the top-right corner and fades after a calculated
/// duration (longer messages stay longer).
pub struct Notification {
    /// Current message being shown (None = hidden)
    message: Option<String>,
    /// When the message was shown (for timing dismissal)
    shown_at: Option<Instant>,
    /// How long to show the current message
    duration: Duration,
}

impl Notification {
    /// Base display time in milliseconds
    const BASE_MS: u64 = 1500;
    /// Additional milliseconds per character in the message
    const MS_PER_CHAR: u64 = 50;

    /// Creates a new hidden notification.
    pub fn new() -> Self {
        Self {
            message: None,
            shown_at: None,
            duration: Duration::ZERO,
        }
    }

    /// Shows a notification message.
    ///
    /// Duration is calculated as `BASE_MS + (char_count * MS_PER_CHAR)`.
    /// Replaces any existing notification.
    pub fn show(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        let duration_ms = Self::BASE_MS + (msg.len() as u64 * Self::MS_PER_CHAR);
        self.duration = Duration::from_millis(duration_ms);
        self.message = Some(msg);
        self.shown_at = Some(Instant::now());
    }

    /// Immediately hides the notification.
    fn dismiss(&mut self) {
        self.message = None;
        self.shown_at = None;
    }

    /// Returns true if a notification is currently shown.
    pub fn is_visible(&self) -> bool {
        self.message.is_some()
    }

    /// Checks if the notification should auto-dismiss.
    ///
    /// Call this each frame (or during render) to handle timing.
    pub fn tick(&mut self) {
        if let Some(shown_at) = self.shown_at {
            if shown_at.elapsed() >= self.duration {
                self.dismiss();
            }
        }
    }

    /// Renders the notification in the top-right corner.
    ///
    /// Width adjusts to fit the message. Clears the area behind it.
    pub fn render(&mut self, frame: &mut Frame) {
        // Check for auto-dismiss
        self.tick();

        let Some(msg) = &self.message else { return };

        // Calculate size: message + padding + borders
        let width = (msg.len() as u16 + 4).min(frame.area().width);
        let area = Rect {
            x: frame.area().width.saturating_sub(width),
            y: 0,
            width,
            height: 3,
        };

        // Clear background and render
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let text = Paragraph::new(msg.as_str()).block(block);
        frame.render_widget(text, area);
    }
}

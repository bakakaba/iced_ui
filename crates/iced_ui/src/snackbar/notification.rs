//! Notification data types for the [`Snackbar`](super::Snackbar) widget.

use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// A unique identifier for a notification.
///
/// Can be constructed from a `&str`, `String`, or `u64`. Use
/// [`NotificationId::unique()`] to generate a random ID without
/// external state.
///
/// # Example
///
/// ```
/// use iced_ui::snackbar::NotificationId;
///
/// let auto = NotificationId::unique();
/// let from_str = NotificationId::from("order-123");
/// let from_num = NotificationId::from(42u64);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationId(IdInner);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IdInner {
    Num(u64),
    Str(String),
}

impl NotificationId {
    /// Generates a unique random ID.
    ///
    /// Uses an internal counter mixed with a random seed to produce
    /// non-sequential, non-repeating identifiers without requiring
    /// the consumer to track any external state.
    pub fn unique() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(IdInner::Num(RandomState::new().hash_one(count)))
    }
}

impl From<u64> for NotificationId {
    fn from(value: u64) -> Self {
        Self(IdInner::Num(value))
    }
}

impl From<&str> for NotificationId {
    fn from(value: &str) -> Self {
        Self(IdInner::Str(value.to_owned()))
    }
}

impl From<String> for NotificationId {
    fn from(value: String) -> Self {
        Self(IdInner::Str(value))
    }
}

impl fmt::Display for NotificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            IdInner::Num(n) => write!(f, "{n:016x}"),
            IdInner::Str(s) => write!(f, "{s}"),
        }
    }
}

/// The severity level of a notification.
///
/// Controls the accent color (left border) and the icon displayed at
/// the start of the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Severity {
    /// No severity indicator (default). Dark background, no accent.
    #[default]
    Neutral,
    /// Informational — uses the information color token (cyan/blue).
    Information,
    /// Success — uses the success color token (green).
    Success,
    /// Warning — uses the warning color token (amber/yellow).
    Warning,
    /// Error — uses the danger color token (red).
    Error,
}

impl Severity {
    /// Returns the icon glyph and font for this severity level.
    ///
    /// When the `lucide` feature is enabled, returns the Lucide icon
    /// character and font. Otherwise returns a Unicode fallback with
    /// the default font.
    #[cfg(feature = "lucide")]
    pub(super) fn icon_content(self) -> Option<(String, iced::Font)> {
        use crate::icons::FONT;
        use lucide_icons::Icon;

        let icon = match self {
            Self::Neutral => return None,
            Self::Information => Icon::Info,
            Self::Success => Icon::CheckCircle2,
            Self::Warning => Icon::AlertTriangle,
            Self::Error => Icon::CircleX,
        };
        Some((char::from(icon).to_string(), FONT))
    }

    /// Returns the icon glyph and font for this severity level.
    #[cfg(not(feature = "lucide"))]
    pub(super) fn icon_content(self) -> Option<(String, iced::Font)> {
        let glyph = match self {
            Self::Neutral => return None,
            Self::Information => "i",
            Self::Success => "\u{2713}", // ✓
            Self::Warning => "!",
            Self::Error => "\u{00D7}", // ×
        };
        Some((glyph.to_string(), iced::Font::default()))
    }
}

/// A notification entry managed by [`Notifications`](super::Notifications).
///
/// `Notification` is plain data with no lifetimes or generics — it can
/// be stored directly in application state and persisted across frames.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use iced_ui::snackbar::{Notification, Severity};
///
/// let notif = Notification::new("File deleted")
///     .severity(Severity::Warning)
///     .action("Undo")
///     .action("Details")
///     .auto_dismiss(Duration::from_secs(5));
/// ```
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique identifier for this notification.
    pub id: NotificationId,
    /// The message text displayed in the notification bar.
    pub message: String,
    /// Severity level controlling accent color and icon.
    pub severity: Severity,
    /// If set, the notification auto-dismisses after this duration
    /// once it becomes visible.
    pub auto_dismiss: Option<Duration>,
    /// Action button labels. Each action triggers `on_action(id, index)`.
    pub actions: Vec<String>,
    /// When this notification was created.
    pub created_at: Instant,
    /// When this notification was dismissed. `None` means active.
    pub dismissed_at: Option<Instant>,
}

impl Notification {
    /// Creates a new notification with the given message.
    ///
    /// A unique ID is auto-generated. Use [`.id()`](Self::id) to
    /// override with a consumer-provided identifier.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            id: NotificationId::unique(),
            message: message.into(),
            severity: Severity::Neutral,
            auto_dismiss: None,
            actions: Vec::new(),
            created_at: Instant::now(),
            dismissed_at: None,
        }
    }

    /// Sets a consumer-provided ID, replacing the auto-generated one.
    pub fn id(mut self, id: impl Into<NotificationId>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the severity level.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Enables auto-dismiss after the given duration.
    ///
    /// The countdown only starts when the notification becomes visible
    /// (i.e. within the [`Snackbar`](super::Snackbar)'s `max` display
    /// window).
    pub fn auto_dismiss(mut self, duration: Duration) -> Self {
        self.auto_dismiss = Some(duration);
        self
    }

    /// Adds an action button with the given label.
    ///
    /// Multiple actions can be added by chaining `.action()` calls.
    /// When clicked, the snackbar publishes `on_action(notification_id, index)`
    /// where `index` is the zero-based position of this action in the
    /// order they were added.
    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.actions.push(label.into());
        self
    }

    /// Returns `true` if the notification has been dismissed.
    pub fn is_dismissed(&self) -> bool {
        self.dismissed_at.is_some()
    }

    /// Returns `true` if the notification is still active (not dismissed).
    pub fn is_active(&self) -> bool {
        self.dismissed_at.is_none()
    }

    /// Marks this notification as dismissed at the current instant.
    ///
    /// Has no effect if already dismissed.
    pub fn dismiss(&mut self) {
        if self.dismissed_at.is_none() {
            self.dismissed_at = Some(Instant::now());
        }
    }
}

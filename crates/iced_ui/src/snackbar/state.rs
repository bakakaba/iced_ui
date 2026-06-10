//! Managed notification state for the [`Snackbar`](super::Snackbar) widget.
//!
//! [`Notifications`] owns the full notification log and provides
//! methods to publish, dismiss, and query notifications. Store it in
//! your application state and pass a reference to the
//! [`Snackbar`](super::Snackbar) widget in `view()`.
//!
//! # Example
//!
//! ```
//! use iced_ui::snackbar::{Notification, Notifications, Severity};
//! use std::time::Duration;
//!
//! let mut notifications = Notifications::new();
//!
//! // Publish a notification.
//! notifications.push(
//!     Notification::new("File saved").severity(Severity::Success)
//! );
//!
//! // Query active notifications.
//! assert_eq!(notifications.active_count(), 1);
//!
//! // Dismiss by ID.
//! let id = notifications.active()[0].id.clone();
//! notifications.dismiss(&id);
//! assert_eq!(notifications.active_count(), 0);
//! ```

use super::notification::{Notification, NotificationId};

/// Default maximum number of entries (active + dismissed) retained.
const DEFAULT_CAPACITY: usize = 100;

/// Managed notification state.
///
/// Maintains a log of all notifications (active and dismissed). Active
/// notifications are those without a `dismissed_at` timestamp.
///
/// When the total entry count exceeds [`capacity`](Self::capacity),
/// the oldest dismissed entries are pruned automatically on
/// [`dismiss()`](Self::dismiss).
#[derive(Debug, Clone)]
pub struct Notifications {
    entries: Vec<Notification>,
    capacity: usize,
}

impl Default for Notifications {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    /// Creates an empty notification store with the default capacity (100).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }

    /// Creates an empty notification store as a const (for use in statics).
    pub(super) const fn empty() -> Self {
        Self {
            entries: Vec::new(),
            capacity: DEFAULT_CAPACITY,
        }
    }

    /// Creates an empty notification store with a custom capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    /// Sets the maximum number of entries retained (active + dismissed).
    ///
    /// When the count exceeds this value after a dismiss, the oldest
    /// dismissed entries are pruned. Active entries are never pruned.
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
        self.prune();
    }

    /// Returns the configured capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Publishes a new notification.
    ///
    /// The notification is appended to the end of the internal log.
    /// Newer entries appear last.
    pub fn push(&mut self, notification: Notification) {
        self.entries.push(notification);
    }

    /// Dismisses the notification with the given ID.
    ///
    /// Sets `dismissed_at` to the current instant. If the total entry
    /// count now exceeds capacity, the oldest dismissed entries are
    /// pruned.
    ///
    /// Returns `true` if a notification was found and dismissed.
    pub fn dismiss(&mut self, id: &NotificationId) -> bool {
        let found = self
            .entries
            .iter_mut()
            .find(|n| n.id == *id && n.is_active());

        if let Some(notif) = found {
            notif.dismiss();
            self.prune();
            true
        } else {
            false
        }
    }

    /// Returns references to all active (non-dismissed) notifications.
    ///
    /// The order matches insertion order: oldest first, newest last.
    pub fn active(&self) -> Vec<&Notification> {
        self.entries.iter().filter(|n| n.is_active()).collect()
    }

    /// Returns the number of active notifications.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|n| n.is_active()).count()
    }

    /// Returns a reference to all entries (active and dismissed).
    pub fn all(&self) -> &[Notification] {
        &self.entries
    }

    /// Returns references to dismissed notifications only.
    pub fn dismissed(&self) -> Vec<&Notification> {
        self.entries.iter().filter(|n| n.is_dismissed()).collect()
    }

    /// Returns the total number of entries (active + dismissed).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if there are no entries at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all entries (active and dismissed).
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Removes all dismissed entries, keeping only active ones.
    pub fn clear_dismissed(&mut self) {
        self.entries.retain(|n| n.is_active());
    }

    /// Prunes oldest dismissed entries until the total count is within
    /// capacity. Active entries are never removed.
    fn prune(&mut self) {
        while self.entries.len() > self.capacity {
            // Find the first (oldest) dismissed entry and remove it.
            if let Some(pos) = self.entries.iter().position(|n| n.is_dismissed()) {
                self.entries.remove(pos);
            } else {
                // All remaining entries are active — cannot prune further.
                break;
            }
        }
    }
}

/// Shared application state that multiple pages may read.
///
/// This is distinct from page-local state which lives inside each page's
/// own struct. Only truly cross-cutting state belongs here.
#[derive(Debug, Default)]
pub(crate) struct ActionLog {
    pub(crate) counter: u32,
    pub(crate) last_action: Option<String>,
}

impl ActionLog {
    pub(crate) fn record(&mut self, action: impl Into<String>) {
        self.counter = self.counter.saturating_add(1);
        self.last_action = Some(action.into());
    }

    pub(crate) fn set_last(&mut self, action: impl Into<String>) {
        self.last_action = Some(action.into());
    }
}

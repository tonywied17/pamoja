//! Telemetry events: a severity level, a stable code, and an optional value.

/// The severity of a telemetry event, ordered from most verbose to most urgent.
///
/// [`Trace`](Level::Trace) is the least urgent and [`Error`](Level::Error) the most,
/// so a [`Reporter`](crate::Reporter) ships an event when its level is at or above the
/// current threshold and drops it otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Fine-grained detail, useful only when chasing a specific problem.
    Trace,
    /// Diagnostic detail for development.
    Debug,
    /// A normal, noteworthy event.
    Info,
    /// Something unexpected that the node recovered from.
    Warn,
    /// A failure that needs attention.
    Error,
}

/// A structured telemetry event.
///
/// An event pairs a [`Level`] with a stable, short `code` - a label such as
/// `"battery.low"` or `"link.lost"` rather than a free-form message - so events stay
/// tiny, group cleanly into counts, and need no allocation. An optional `value`
/// carries an associated measurement, such as the battery level that triggered it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    /// The event's severity.
    pub level: Level,
    /// A stable, short identifier for what happened.
    pub code: &'static str,
    /// An optional measurement associated with the event.
    pub value: Option<f32>,
}

impl Event {
    /// Creates an event at `level` with the given code and no value.
    ///
    /// # Arguments
    ///
    /// * `level` - the event's severity.
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn new(level: Level, code: &'static str) -> Self {
        Self {
            level,
            code,
            value: None,
        }
    }

    /// Creates a [`Level::Trace`] event.
    ///
    /// # Arguments
    ///
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn trace(code: &'static str) -> Self {
        Self::new(Level::Trace, code)
    }

    /// Creates a [`Level::Debug`] event.
    ///
    /// # Arguments
    ///
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn debug(code: &'static str) -> Self {
        Self::new(Level::Debug, code)
    }

    /// Creates a [`Level::Info`] event.
    ///
    /// # Arguments
    ///
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn info(code: &'static str) -> Self {
        Self::new(Level::Info, code)
    }

    /// Creates a [`Level::Warn`] event.
    ///
    /// # Arguments
    ///
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn warn(code: &'static str) -> Self {
        Self::new(Level::Warn, code)
    }

    /// Creates a [`Level::Error`] event.
    ///
    /// # Arguments
    ///
    /// * `code` - a stable, short identifier for the event.
    ///
    /// # Returns
    ///
    /// The event.
    pub fn error(code: &'static str) -> Self {
        Self::new(Level::Error, code)
    }

    /// Attaches a measurement to the event.
    ///
    /// # Arguments
    ///
    /// * `value` - the measurement to associate with the event.
    ///
    /// # Returns
    ///
    /// The event, for chaining.
    pub fn with_value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }
}

use std::{collections::BTreeMap, ops::AddAssign};

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    serde::Serialize,
    strum::Display,
    tracing,
};

/// The response to take when there are losses in decoding or encoding
#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum LossesResponse {
    /// Ignore the losses; do nothing
    Ignore,
    /// Log losses as separate log entries with the `TRACE` severity level
    Trace,
    /// Log losses as separate log entries with the `DEBUG` severity level
    Debug,
    /// Log losses as separate log entries with the `INFO` severity level
    Info,
    /// Log losses as separate log entries with the `WARN` severity level
    Warn,
    /// Log losses as separate log entries with the `ERROR` severity level
    Error,
    /// Abort the current function call by returning a `Err` result with the losses enumerated
    Abort,
}

/// Decoding and encoding losses
#[derive(Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct Losses {
    #[serde(flatten)]
    inner: BTreeMap<String, usize>,
}

impl Losses {
    /// Create a set of losses
    pub fn new<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut losses = Self::default();
        for label in labels {
            losses.add(label)
        }
        losses
    }

    /// Create an empty set of losses
    ///
    /// Equivalent to [`Losses::new`] but provided to make it more explicit
    /// when a codec is lossless (i.e. it returns `Losses::none()`)
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a set of losses with a single [`Loss`]
    pub fn one<S>(label: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::new([label])
    }

    /// Indicate that enumerating the losses is not yet implemented
    pub fn todo() -> Self {
        Self::default()
    }

    /// Add a loss to the current set
    ///
    /// If the type of loss is already registered then increments the count by one.
    pub fn add<S>(&mut self, label: S)
    where
        S: AsRef<str>,
    {
        let label = label.as_ref().to_string();
        self.inner
            .entry(label)
            .and_modify(|count| count.add_assign(1))
            .or_insert(1);
    }

    /// Merge another set of losses into this one
    pub fn merge(&mut self, losses: Losses) {
        for (label, count) in losses.inner {
            self.inner
                .entry(label)
                .and_modify(|current| current.add_assign(count))
                .or_insert(count);
        }
    }

    /// Is this set of losses empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Respond to losses according to the `LossesResponse` variant
    pub fn respond(&self, response: LossesResponse) -> Result<()> {
        use LossesResponse::*;

        if self.inner.is_empty() || matches!(response, Ignore) {
            return Ok(());
        }

        if matches!(response, Abort) {
            let summary = self
                .inner
                .iter()
                .map(|(label, count)| format!("{label}({count})"))
                .join(", ");
            let error = eyre!(summary).wrap_err("Conversion losses occurred");
            return Err(error);
        }

        for (label, count) in self.inner.iter() {
            match response {
                Trace => {
                    tracing::event!(tracing::Level::TRACE, "{label}({count})");
                }
                Debug => {
                    tracing::event!(tracing::Level::DEBUG, "{label}({count})");
                }
                Info => {
                    tracing::event!(tracing::Level::INFO, "{label}({count})");
                }
                Warn => {
                    tracing::event!(tracing::Level::WARN, "{label}({count})");
                }
                Error => {
                    tracing::event!(tracing::Level::ERROR, "{label}({count})");
                }
                _ => bail!("Should be unreachable"),
            };
        }

        Ok(())
    }
}

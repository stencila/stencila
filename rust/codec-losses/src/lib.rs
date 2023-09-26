use common::{
    clap::{self, ValueEnum},
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    smart_default::SmartDefault,
    strum::Display,
    tracing,
};

/// A record of a loss during encoding or decoding
#[derive(Debug, SmartDefault)]
pub struct Loss {
    /// A label for the loss
    ///
    /// The convention used for the label will depend upon the format
    /// and the direction (encoding or decoding).
    label: String,

    /// A count of the number of times the loss occurred
    #[default = 1]
    count: usize,
}

impl Loss {
    /// Create a new loss
    pub fn new<T>(label: T) -> Self
    where
        T: AsRef<str>,
    {
        Loss {
            label: label.as_ref().to_string(),
            ..Default::default()
        }
    }
}

impl From<&str> for Loss {
    fn from(label: &str) -> Self {
        Self::new(label)
    }
}

impl From<String> for Loss {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

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
#[derive(Debug, Default)]
pub struct Losses {
    inner: Vec<Loss>,
}

impl Losses {
    /// Create a set of losses
    pub fn new<T>(inner: T) -> Self
    where
        T: IntoIterator<Item = Loss>,
    {
        Self {
            inner: inner.into_iter().collect_vec(),
        }
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
        Self::new([Loss::new(label)])
    }

    /// Indicate that enumerating the losses is not yet implemented
    pub fn todo() -> Self {
        Self::default()
    }

    /// Push a loss onto this list of losses
    ///
    /// If the type of loss is already registered then increments the count by one.
    pub fn add<L>(&mut self, loss: L)
    where
        L: Into<Loss>,
    {
        let loss = loss.into();
        for existing in self.inner.iter_mut() {
            if existing.label == loss.label {
                existing.count += 1;
                return;
            }
        }

        self.inner.push(loss)
    }

    /// Append another list of losses onto this one
    pub fn add_all(&mut self, losses: &mut Losses) {
        for _ in 0..losses.inner.len() {
            let loss = losses.inner.swap_remove(0);
            self.add(loss)
        }
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
                .map(|Loss { label, count }| format!("{label}({count})"))
                .join(", ");
            let error = eyre!(summary).wrap_err("Conversion losses occurred");
            return Err(error);
        }

        for Loss { label, count } in self.inner.iter() {
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

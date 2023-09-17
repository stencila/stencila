use std::fmt;

use common::{
    clap::{self, ValueEnum},
    derive_more::{Deref, DerefMut},
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    smart_default::SmartDefault,
    strum::Display,
    tracing,
};

/// The direction of loss
#[derive(Debug, PartialEq)]
pub enum LossDirection {
    Decode,
    Encode,
}

impl fmt::Display for LossDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LossDirection::*;

        match self {
            Decode => write!(f, "Decoding loss"),
            Encode => write!(f, "Encoding loss"),
        }
    }
}

/// The kind of a loss
#[derive(Debug, PartialEq)]
pub enum LossKind {
    Type,
    Structure,
    Properties(Vec<String>),
    Todo,
}

impl fmt::Display for LossKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LossKind::*;

        match self {
            Type => write!(f, "type lost"),
            Structure => write!(f, "structure lost"),
            Properties(props) => match props.len() {
                0 => write!(f, "properties lost"),
                1 => write!(f, "property {} lost", props[0]),
                _ => write!(f, "properties {} lost", props.join(", ")),
            },
            Todo => write!(f, "not yet implemented"),
        }
    }
}

/// A record of a loss during encoding or decoding
#[derive(Debug, SmartDefault)]
pub struct Loss {
    /// The direction of loss e.g. [`LossDirection::Decode`]
    #[default(_code = "LossDirection::Decode")]
    direction: LossDirection,

    /// The type for which the loss occurred e.g. `Paragraph`
    r#type: String,

    /// The kind of loss e.g. [`LossKind::Structure`]
    #[default(_code = "LossKind::Type")]
    kind: LossKind,

    /// A count of the number of times the loss occurred
    #[default = 1]
    count: usize,
}

impl Loss {
    /// Create a loss with [`LossKind::Type`]
    pub fn of_type<T>(direction: LossDirection, r#type: T) -> Self
    where
        T: AsRef<str>,
    {
        Loss {
            direction,
            r#type: r#type.as_ref().to_string(),
            kind: LossKind::Type,
            ..Default::default()
        }
    }

    /// Create a loss with [`LossKind::Structure`]
    pub fn of_structure<T>(direction: LossDirection, r#type: T) -> Self
    where
        T: AsRef<str>,
    {
        Loss {
            direction,
            r#type: r#type.as_ref().to_string(),
            kind: LossKind::Structure,
            ..Default::default()
        }
    }

    /// Create a loss with [`LossKind::Properties`] for a single property
    pub fn of_property<T, P>(direction: LossDirection, r#type: T, property: P) -> Self
    where
        T: AsRef<str>,
        P: AsRef<str>,
    {
        Loss {
            direction,
            r#type: r#type.as_ref().to_string(),
            kind: LossKind::Properties(vec![property.as_ref().to_string()]),
            ..Default::default()
        }
    }

    /// Create a loss with [`LossKind::Properties`]
    pub fn of_properties<T, I>(direction: LossDirection, r#type: T, properties: I) -> Self
    where
        T: AsRef<str>,
        I: IntoIterator<Item = String>,
    {
        let properties = properties.into_iter().collect_vec();
        Loss {
            direction,
            r#type: r#type.as_ref().to_string(),
            kind: LossKind::Properties(properties),
            ..Default::default()
        }
    }

    /// Create a loss with [`LossKind::Todo`]
    pub fn todo<T>(direction: LossDirection, r#type: T) -> Self
    where
        T: AsRef<str>,
    {
        Loss {
            direction,
            r#type: r#type.as_ref().to_string(),
            kind: LossKind::Todo,
            ..Default::default()
        }
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
#[derive(Debug, Default, Deref, DerefMut)]
pub struct Losses {
    inner: Vec<Loss>,
}

impl Losses {
    /// Create a new set of losses
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator<Item = Loss>,
    {
        Self {
            inner: items.into_iter().collect_vec(),
        }
    }

    /// Create an empty set of losses
    ///
    /// Equivalent to [`Losses::new`] but provided to make it more explicit
    /// when a codec is lossless (i.e. it returns `Losses::none()`)
    pub fn none() -> Self {
        Self::default()
    }

    /// Indicate that enumerating the losses for a codec is yet to be implemented
    pub fn todo() -> Self {
        Self::default()
    }

    /// Create a set of losses for entire node
    pub fn of_all<S>(r#type: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::new([Loss::of_properties(
            LossDirection::Encode,
            r#type,
            ["*".to_string()],
        )])
    }

    /// Create a set of losses with one entry for the loss of the id property
    ///
    /// This is a convenience function provided because often, `id` is the
    /// only property that is potentially lost.
    pub fn of_id<S>(r#type: S) -> Self
    where
        S: AsRef<str>,
    {
        Self::new([Loss::of_properties(
            LossDirection::Encode,
            r#type,
            ["id".to_string()],
        )])
    }

    /// Push a loss onto this list of losses
    ///
    /// If the type of loss is already registered then increments the count by one.
    pub fn add(&mut self, loss: Loss) {
        for existing in self.iter_mut() {
            if existing.r#type == loss.r#type && existing.kind == loss.kind {
                existing.count += 1;
                return;
            }
        }

        self.push(loss)
    }

    /// Append another list of losses onto this one
    pub fn add_all(&mut self, losses: &mut Losses) {
        for _ in 0..losses.len() {
            let loss = losses.swap_remove(0);
            self.add(loss)
        }
    }

    /// Respond to losses according to the `LossesResponse` variant
    pub fn respond(&self, response: LossesResponse) -> Result<()> {
        use LossesResponse::*;

        if self.is_empty() || matches!(response, Ignore) {
            return Ok(());
        }

        if matches!(response, Abort) {
            let summary = self
                .iter()
                .map(
                    |Loss {
                         direction,
                         r#type,
                         kind,
                         count,
                     }| format!("{direction} for {type}: {kind} ({count})"),
                )
                .join("; ");
            let error = eyre!(summary).wrap_err("Conversion losses occurred");
            return Err(error);
        }

        for Loss {
            direction,
            r#type,
            kind,
            count,
        } in self.iter()
        {
            match response {
                Trace => {
                    tracing::event!(
                        tracing::Level::TRACE,
                        "{direction} for {type}: {kind} ({count})"
                    );
                }
                Debug => {
                    tracing::event!(
                        tracing::Level::DEBUG,
                        "{direction} for {type}: {kind} ({count})"
                    );
                }
                Info => {
                    tracing::event!(
                        tracing::Level::INFO,
                        "{direction} for {type}: {kind} ({count})"
                    );
                }
                Warn => {
                    tracing::event!(
                        tracing::Level::WARN,
                        "{direction} for {type}: {kind} ({count})"
                    );
                }
                Error => {
                    tracing::event!(
                        tracing::Level::ERROR,
                        "{direction} for {type}: {kind} ({count})"
                    );
                }
                _ => bail!("Should be unreachable"),
            };
        }

        Ok(())
    }
}

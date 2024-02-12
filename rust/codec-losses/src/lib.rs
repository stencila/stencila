use std::{collections::BTreeMap, fmt::Display, fs, ops::AddAssign, path::PathBuf};

use common::{
    eyre::{bail, eyre, Result},
    inflector::Inflector,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_json, serde_yaml,
    strum::Display,
    tracing,
};
use format::Format;

/// The response to take when there are losses in decoding or encoding
#[derive(Debug, Clone, Display, Serialize, Deserialize)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
#[serde(rename_all = "lowercase", crate = "common::serde")]
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
    /// Write losses to a file
    Write(String),
}

impl From<String> for LossesResponse {
    fn from(value: String) -> Self {
        use LossesResponse::*;
        match value.to_lowercase().as_str() {
            "ignore" => Ignore,
            "trace" => Trace,
            "debug" => Debug,
            "info" => Info,
            "warn" => Warn,
            "error" => Error,
            "abort" => Abort,
            _ => Write(value),
        }
    }
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

    /// Create a set of losses for the properties of an object
    pub fn props<T, I, S>(_object: &T, prop_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let type_name = std::any::type_name::<T>();
        let type_name = type_name.rsplit("::").next().unwrap_or(type_name);

        let labels = prop_names
            .into_iter()
            .map(|name| format!("{type_name}.{}", name.into().to_camel_case()));

        Self::new(labels)
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

    /// Add a loss of a property to the current set
    pub fn add_prop<T>(&mut self, _object: &T, prop_name: &str) {
        let type_name = std::any::type_name::<T>();
        let type_name = type_name.rsplit("::").next().unwrap_or(type_name);
        let prop_name = prop_name.to_camel_case();

        self.add(format!("{type_name}.{prop_name}"));
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
    pub fn respond<D>(&self, what: D, response: LossesResponse) -> Result<()>
    where
        D: Display,
    {
        use LossesResponse::*;

        if self.inner.is_empty() || matches!(response, Ignore) {
            return Ok(());
        }

        if let LossesResponse::Write(path) = &response {
            let path = PathBuf::from(path);
            let format = Format::from_path(&path)?;
            let content = match format {
                Format::Json => serde_json::to_string_pretty(self)?,
                Format::Yaml => serde_yaml::to_string(self)?,
                _ => bail!(
                    "Unsupported format for conversion losses: {format} (expected JSON or YAML)"
                ),
            };
            fs::write(path, content)?;
            return Ok(());
        }

        let summary = self
            .inner
            .iter()
            .map(|(label, count)| format!("{label}:{count}"))
            .join(", ");

        match response {
            Trace => tracing::trace!("{what}: {summary}"),
            Debug => tracing::debug!("{what}: {summary}"),
            Info => tracing::info!("{what}: {summary}"),
            Warn => tracing::warn!("{what}: {summary}"),
            Error => tracing::error!("{what}: {summary}"),
            Abort => return Err(eyre!("{summary}").wrap_err(what.to_string())),
            _ => bail!("Should be unreachable"),
        };

        Ok(())
    }
}

/// Create a set of losses for properties of a type
///
///
#[macro_export]
macro_rules! lost_props {
    ($object:expr, $($field:literal),*) => {{
        Losses::props(&$object, [$($field,)*])
    }};
}

/// Create a set of losses for optional properties
///
/// A loss will be recorded for the property if it `is_some()`
/// but not if it `is_none()`.
#[macro_export]
macro_rules! lost_options {
    ($object:expr, $($field:ident),*) => {{
        let mut losses = Losses::none();
        $(
            if $object.$field.is_some() {
                losses.add_prop(&$object, stringify!($field));
            }
        )*
        losses
    }};
}

/// Create a set of losses for optional fields on `Executable` nodes
#[macro_export]
macro_rules! lost_exec_options {
    ($object:expr) => {
        codec_losses::lost_options!(
            $object.options,
            compilation_digest,
            compilation_errors,
            execution_digest,
            execution_dependencies,
            execution_dependants,
            execution_tags,
            execution_count,
            execution_required,
            execution_status,
            execution_actor,
            execution_ended,
            execution_duration,
            execution_messages
        )
    };
}

/// Create a set of losses for optional fields on `CreativeWork` nodes
#[macro_export]
macro_rules! lost_work_options {
    ($object:expr) => {
        codec_losses::lost_options!($object.options, authors)
    };
}

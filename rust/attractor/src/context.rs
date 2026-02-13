use std::fmt;
use std::sync::RwLock;

use indexmap::IndexMap;
use serde_json::Value;

/// A thread-safe key-value store for pipeline execution context.
///
/// Values are stored as [`serde_json::Value`] to support heterogeneous data.
/// All access is protected by a [`RwLock`]; poisoned locks are recovered
/// automatically so that a panic in one handler does not block other reads.
pub struct Context {
    inner: RwLock<ContextInner>,
}

struct ContextInner {
    values: IndexMap<String, Value>,
    logs: Vec<String>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_struct("Context")
            .field("values", &inner.values)
            .field("logs", &inner.logs)
            .finish()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create an empty context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ContextInner {
                values: IndexMap::new(),
                logs: Vec::new(),
            }),
        }
    }

    /// Set a value in the context, replacing any previous value for this key.
    pub fn set(&self, key: impl Into<String>, value: Value) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.insert(key.into(), value);
    }

    /// Get a clone of the value for a key, or `None` if not present.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Value> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.get(key).cloned()
    }

    /// Get a string representation of a value, per spec §5.1.
    ///
    /// Coercion rules:
    /// - Missing key → `""`
    /// - String → the string contents (without quotes)
    /// - Number, Bool → `to_string()`
    /// - Null, Array, Object → JSON serialization
    #[must_use]
    pub fn get_string(&self, key: &str) -> String {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match inner.values.get(key) {
            None => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(other) => other.to_string(),
        }
    }

    /// Get an integer value, if the key exists and holds a number that fits in `i64`.
    #[must_use]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.get(key).and_then(Value::as_i64)
    }

    /// Return a snapshot of all current values.
    ///
    /// The returned map is independent — subsequent mutations to the context
    /// do not affect the snapshot.
    #[must_use]
    pub fn snapshot(&self) -> IndexMap<String, Value> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.clone()
    }

    /// Create a fully independent clone of this context, including logs.
    #[must_use]
    pub fn deep_clone(&self) -> Self {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self {
            inner: RwLock::new(ContextInner {
                values: inner.values.clone(),
                logs: inner.logs.clone(),
            }),
        }
    }

    /// Apply a batch of key-value updates, overwriting existing keys.
    pub fn apply_updates(&self, updates: &IndexMap<String, Value>) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (key, value) in updates {
            inner.values.insert(key.clone(), value.clone());
        }
    }

    /// Append a log entry.
    pub fn append_log(&self, entry: impl Into<String>) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.logs.push(entry.into());
    }

    /// Return a clone of all log entries in order.
    #[must_use]
    pub fn logs(&self) -> Vec<String> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.logs.clone()
    }
}

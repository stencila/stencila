use std::fmt;
use std::sync::RwLock;

use indexmap::IndexMap;
use serde_json::Value;

// ---------------------------------------------------------------------------
// ContextBackend trait
// ---------------------------------------------------------------------------

/// Pluggable storage backend for pipeline context.
///
/// Implementations handle their own concurrency. The in-memory backend uses
/// [`RwLock`]; a `SQLite` backend would use its own locking.
///
/// Convenience methods that don't need backend-specific logic (`get_string`,
/// `get_i64`) live on [`Context`] as thin wrappers over [`get()`](Self::get).
pub trait ContextBackend: Send + Sync + fmt::Debug {
    /// Set a value, replacing any previous value for this key.
    fn set(&self, key: &str, value: Value);

    /// Get a clone of the value for a key, or `None` if not present.
    fn get(&self, key: &str) -> Option<Value>;

    /// Return a snapshot of all current key-value pairs.
    fn snapshot(&self) -> IndexMap<String, Value>;

    /// Create a fully independent clone of this backend, including logs.
    fn clone_backend(&self) -> Box<dyn ContextBackend>;

    /// Apply a batch of key-value updates, overwriting existing keys.
    fn apply_updates(&self, updates: &IndexMap<String, Value>);

    /// Append a log entry.
    fn append_log(&self, entry: &str);

    /// Return a clone of all log entries in order.
    fn logs(&self) -> Vec<String>;

    /// Downcast support for backend-specific operations.
    fn as_any(&self) -> &dyn std::any::Any;
}

// ---------------------------------------------------------------------------
// InMemoryBackend
// ---------------------------------------------------------------------------

/// In-memory context backend using [`RwLock`].
///
/// This is the default backend — it implements the spec (§5.1) with no
/// external dependencies. Poisoned locks are recovered automatically so
/// that a panic in one handler does not block other reads.
pub(crate) struct InMemoryBackend {
    inner: RwLock<InMemoryInner>,
}

struct InMemoryInner {
    values: IndexMap<String, Value>,
    logs: Vec<String>,
}

impl fmt::Debug for InMemoryBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_struct("InMemoryBackend")
            .field("values", &inner.values)
            .field("logs", &inner.logs)
            .finish()
    }
}

impl InMemoryBackend {
    pub(crate) fn new() -> Self {
        Self {
            inner: RwLock::new(InMemoryInner {
                values: IndexMap::new(),
                logs: Vec::new(),
            }),
        }
    }

    pub(crate) fn with_data(values: IndexMap<String, Value>, logs: Vec<String>) -> Self {
        Self {
            inner: RwLock::new(InMemoryInner { values, logs }),
        }
    }
}

impl ContextBackend for InMemoryBackend {
    fn set(&self, key: &str, value: Value) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<Value> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.get(key).cloned()
    }

    fn snapshot(&self) -> IndexMap<String, Value> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.values.clone()
    }

    fn clone_backend(&self) -> Box<dyn ContextBackend> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Box::new(InMemoryBackend::with_data(
            inner.values.clone(),
            inner.logs.clone(),
        ))
    }

    fn apply_updates(&self, updates: &IndexMap<String, Value>) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (key, value) in updates {
            inner.values.insert(key.clone(), value.clone());
        }
    }

    fn append_log(&self, entry: &str) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.logs.push(entry.to_string());
    }

    fn logs(&self) -> Vec<String> {
        let inner = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.logs.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// A thread-safe key-value store for pipeline execution context.
///
/// Values are stored as [`serde_json::Value`] to support heterogeneous data.
/// Delegates to a pluggable [`ContextBackend`] — by default, an in-memory
/// backend protected by a [`RwLock`].
pub struct Context {
    backend: Box<dyn ContextBackend>,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("backend", &self.backend)
            .finish()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create an empty context with the default in-memory backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            backend: Box::new(InMemoryBackend::new()),
        }
    }

    /// Create a context with a custom backend.
    #[must_use]
    pub fn with_backend(backend: Box<dyn ContextBackend>) -> Self {
        Self { backend }
    }

    /// Set a value in the context, replacing any previous value for this key.
    pub fn set(&self, key: impl Into<String>, value: Value) {
        self.backend.set(&key.into(), value);
    }

    /// Get a clone of the value for a key, or `None` if not present.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Value> {
        self.backend.get(key)
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
        match self.backend.get(key) {
            None => String::new(),
            Some(Value::String(s)) => s,
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(other) => other.to_string(),
        }
    }

    /// Get an integer value, if the key exists and holds a number that fits in `i64`.
    #[must_use]
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.backend.get(key).and_then(|v| v.as_i64())
    }

    /// Return a snapshot of all current values.
    ///
    /// The returned map is independent — subsequent mutations to the context
    /// do not affect the snapshot.
    #[must_use]
    pub fn snapshot(&self) -> IndexMap<String, Value> {
        self.backend.snapshot()
    }

    /// Create a fully independent clone of this context, including logs.
    #[must_use]
    pub fn deep_clone(&self) -> Self {
        Self {
            backend: self.backend.clone_backend(),
        }
    }

    /// Apply a batch of key-value updates, overwriting existing keys.
    pub fn apply_updates(&self, updates: &IndexMap<String, Value>) {
        self.backend.apply_updates(updates);
    }

    /// Append a log entry.
    pub fn append_log(&self, entry: impl Into<String>) {
        self.backend.append_log(&entry.into());
    }

    /// Return a clone of all log entries in order.
    #[must_use]
    pub fn logs(&self) -> Vec<String> {
        self.backend.logs()
    }

    /// Get a reference to the underlying backend as `&dyn Any`.
    ///
    /// This enables downcasting to a specific backend type (e.g.
    /// `SqliteBackend`) when needed for backend-specific operations.
    #[must_use]
    pub fn backend_as_any(&self) -> &dyn std::any::Any {
        self.backend.as_any()
    }

    /// Create a context backed by `SQLite`.
    ///
    /// Opens (or creates) the database at `db_path`, runs migrations,
    /// and scopes all operations to `run_id`.
    ///
    /// # Errors
    ///
    /// Returns a `rusqlite::Error` if the database cannot be opened or
    /// migrations fail.
    #[cfg(feature = "sqlite")]
    pub fn with_sqlite(db_path: &std::path::Path, run_id: &str) -> Result<Self, rusqlite::Error> {
        let backend = crate::sqlite_backend::SqliteBackend::open(db_path, run_id)?;
        Ok(Self {
            backend: Box::new(backend),
        })
    }

    /// Get the `SQLite` connection handle, if this context uses a `SQLite` backend.
    ///
    /// Returns `None` for in-memory backends. Tool executors use this to get
    /// shared access to the database connection.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlite_connection(
        &self,
    ) -> Option<&std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>> {
        self.backend
            .as_any()
            .downcast_ref::<crate::sqlite_backend::SqliteBackend>()
            .map(super::sqlite_backend::SqliteBackend::connection)
    }

    /// Get the run ID, if this context uses a `SQLite` backend.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlite_run_id(&self) -> Option<&str> {
        self.backend
            .as_any()
            .downcast_ref::<crate::sqlite_backend::SqliteBackend>()
            .map(super::sqlite_backend::SqliteBackend::run_id)
    }

    /// Get a reference to the `SQLite` backend, if this context uses one.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlite_backend(&self) -> Option<&crate::sqlite_backend::SqliteBackend> {
        self.backend
            .as_any()
            .downcast_ref::<crate::sqlite_backend::SqliteBackend>()
    }
}

//! Thread-safe session pool.
//!
//! Provides [`SessionPool`], an `Arc<Mutex<…>>`-backed map of session
//! entries keyed by thread ID.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[derive(Debug, Clone, Default)]
pub struct SessionEntry {
    pub agent_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct SessionPool {
    inner: Arc<Mutex<HashMap<String, SessionEntry>>>,
}

impl SessionPool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn lock_entries(&self) -> MutexGuard<'_, HashMap<String, SessionEntry>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn take(&self, thread_id: &str) -> Option<SessionEntry> {
        self.lock_entries().remove(thread_id)
    }

    pub fn put_back(&self, thread_id: String, entry: SessionEntry) {
        self.lock_entries().insert(thread_id, entry);
    }

    pub fn drain(&self) -> HashMap<String, SessionEntry> {
        std::mem::take(&mut *self.lock_entries())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offline_session_entry(agent_name: &str) -> SessionEntry {
        SessionEntry {
            agent_name: agent_name.to_string(),
        }
    }

    #[test]
    fn take_on_empty_pool_returns_none() -> eyre::Result<()> {
        let pool = SessionPool::new();
        let result = pool.take("nonexistent-thread");
        assert!(
            result.is_none(),
            "take() on a freshly created pool should return None"
        );
        Ok(())
    }

    #[test]
    fn pool_is_clone() -> eyre::Result<()> {
        let pool = SessionPool::new();
        let pool2 = pool.clone();
        // Both clones should work independently
        assert!(pool.take("a").is_none());
        assert!(pool2.take("b").is_none());
        Ok(())
    }

    /// Compile-time assertion that SessionPool is Send + Sync.
    /// If SessionPool is not Send + Sync, this function will fail to compile.
    #[test]
    fn pool_is_send_and_sync() -> eyre::Result<()> {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SessionPool>();
        Ok(())
    }

    #[test]
    fn put_back_then_take_round_trips_agent_name() -> eyre::Result<()> {
        let pool = SessionPool::new();
        let thread_id = "thread-123";

        pool.put_back(
            thread_id.to_string(),
            offline_session_entry("offline-agent"),
        );

        let entry = pool.take(thread_id);
        assert_eq!(
            entry.map(|entry| entry.agent_name),
            Some("offline-agent".to_string()),
            "put_back() should allow take() to retrieve the same SessionEntry with its agent_name preserved"
        );

        Ok(())
    }

    #[test]
    fn drain_empties_pool_before_next_take() -> eyre::Result<()> {
        let pool = SessionPool::new();

        pool.put_back("thread-a".to_string(), offline_session_entry("agent-a"));
        pool.put_back("thread-b".to_string(), offline_session_entry("agent-b"));

        let drained = pool.drain();

        assert_eq!(
            drained.len(),
            2,
            "drain() should remove every pooled session entry"
        );
        assert!(
            pool.take("thread-a").is_none(),
            "take() should return None after drain() empties the pool"
        );
        assert!(
            pool.take("thread-b").is_none(),
            "drain() should remove all thread IDs, not just one"
        );

        Ok(())
    }
}

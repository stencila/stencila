//! Thread-safe session pool.
//!
//! Provides [`SessionPool`], an `Arc<Mutex<…>>`-backed map of session
//! entries keyed by thread ID.
//!
//! # Best practices
//!
//! - Use an explicit `thread_id` attribute on nodes inside loops so that
//!   the same session is reused across iterations rather than relying on
//!   the fallback (previous-node) thread ID which may be unstable.
//! - Nodes that share a `thread_id` should reference the same agent so
//!   the conversation history remains coherent.
//! - Set `max_session_turns` on long-running loops to bound the number
//!   of turns accumulated on a single session and avoid unbounded
//!   context growth.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[derive(Debug, Clone, Default)]
/// A single pooled session entry, holding the agent name and the
/// cumulative turn count for the session.
pub struct SessionEntry {
    pub agent_name: String,
    pub turn_count: u64,
}

#[derive(Debug, Clone, Default)]
/// Thread-safe pool of agent sessions keyed by thread ID.
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

    /// Remove and return the session entry for `thread_id`, if present.
    pub fn take(&self, thread_id: &str) -> Option<SessionEntry> {
        self.lock_entries().remove(thread_id)
    }

    /// Insert (or replace) a session entry for `thread_id`.
    pub fn put_back(&self, thread_id: String, entry: SessionEntry) {
        self.lock_entries().insert(thread_id, entry);
    }

    /// Return the current turn count for `thread_id`, if the entry exists.
    pub fn turn_count(&self, thread_id: &str) -> Option<u64> {
        self.lock_entries()
            .get(thread_id)
            .map(|entry| entry.turn_count)
    }

    /// Remove and return all session entries, leaving the pool empty.
    pub fn drain(&self) -> HashMap<String, SessionEntry> {
        std::mem::take(&mut *self.lock_entries())
    }
}

/// RAII guard that returns a [`SessionEntry`] to its [`SessionPool`] on drop.
#[derive(Debug)]
pub struct SessionGuard {
    pool: SessionPool,
    thread_id: String,
    entry: Option<SessionEntry>,
}

impl SessionGuard {
    /// Create a guard that will return `entry` to `pool` under `thread_id` when dropped.
    pub fn from_pool(pool: SessionPool, thread_id: String, entry: SessionEntry) -> Self {
        Self {
            pool,
            thread_id,
            entry: Some(entry),
        }
    }

    /// Prevent this guard from returning the entry to the pool on drop.
    pub fn discard(&mut self) {
        self.entry.take();
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Some(mut entry) = self.entry.take() {
            entry.turn_count += 1;
            self.pool.put_back(self.thread_id.clone(), entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offline_session_entry(agent_name: &str) -> SessionEntry {
        SessionEntry {
            agent_name: agent_name.to_string(),
            ..Default::default()
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

    #[test]
    fn dropping_session_guard_puts_entry_back_in_pool() -> eyre::Result<()> {
        let pool = SessionPool::new();
        let thread_id = "thread-guard";

        {
            let _guard = SessionGuard::from_pool(
                pool.clone(),
                thread_id.to_string(),
                offline_session_entry("offline-agent"),
            );
        }

        let entry = pool.take(thread_id);
        assert_eq!(
            entry.map(|entry| entry.agent_name),
            Some("offline-agent".to_string()),
            "dropping a non-discarded SessionGuard should return its SessionEntry to the pool"
        );

        Ok(())
    }

    #[test]
    fn discarding_session_guard_skips_return_to_pool() -> eyre::Result<()> {
        let pool = SessionPool::new();
        let thread_id = "thread-discard";

        {
            let mut guard = SessionGuard::from_pool(
                pool.clone(),
                thread_id.to_string(),
                offline_session_entry("offline-agent"),
            );
            guard.discard();
        }

        assert!(
            pool.take(thread_id).is_none(),
            "discard() should prevent SessionGuard::drop from returning the SessionEntry to the pool"
        );
        assert_eq!(
            pool.turn_count(thread_id),
            None,
            "discarded guards should not leave behind turn_count state in the pool"
        );

        Ok(())
    }

    #[test]
    fn dropping_session_guard_increments_turn_count_each_time_it_returns_entry() -> eyre::Result<()>
    {
        let pool = SessionPool::new();
        let thread_id = "thread-turn-count";

        {
            let _guard = SessionGuard::from_pool(
                pool.clone(),
                thread_id.to_string(),
                offline_session_entry("offline-agent"),
            );
        }

        assert_eq!(
            pool.turn_count(thread_id),
            Some(1),
            "the first non-discarded drop should record turn_count = 1"
        );

        let entry = pool
            .take(thread_id)
            .ok_or_else(|| eyre::eyre!("expected first drop to return SessionEntry to the pool"))?;

        {
            let _guard = SessionGuard::from_pool(pool.clone(), thread_id.to_string(), entry);
        }

        assert_eq!(
            pool.turn_count(thread_id),
            Some(2),
            "each subsequent non-discarded drop should increment turn_count before returning the SessionEntry to the pool"
        );

        Ok(())
    }
}

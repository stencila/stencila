use crate::{
    pubsub::publish,
    utils::uuids::{self, Family},
};
use defaults::Defaults;
use eyre::{bail, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};
use tokio::{sync::RwLock, task::JoinHandle};

/// A session event
#[derive(Debug, Clone, Serialize)]
pub enum SessionEvent {
    /// One or more of the session's properties was updated
    Updated { session: Session },

    /// A heartbeat event
    Heartbeat { session: Session },
}

/// The status if a session
#[derive(Debug, Clone, Serialize)]
pub enum SessionStatus {
    Pending,
    Starting,
    Started,
    Stopping,
    Stopped,
}

/// A session
#[derive(Debug, Clone, Defaults, Serialize)]
pub struct Session {
    /// The id of the session
    pub id: String,

    /// The id of the project that this session is for
    project: String,

    /// The ids of clients that are subscribed to this session
    ///
    /// Session events will only be published if there is at least
    /// on subscriber.
    subscribers: HashSet<String>,

    /// The status of the session
    #[def = "SessionStatus::Pending"]
    status: SessionStatus,
}

impl Session {
    pub fn new(project: &str) -> Session {
        Session {
            id: uuids::generate(uuids::Family::Session),
            project: project.to_string(),
            ..Default::default()
        }
    }

    pub fn start(&mut self) {
        self.status = SessionStatus::Started;
        self.updated();
    }

    pub fn stop(&mut self) {
        self.status = SessionStatus::Stopped;
        self.updated();
    }

    pub fn subscribe(&mut self, client: &str) -> bool {
        let inserted = self.subscribers.insert(client.to_string());
        self.updated();
        inserted
    }

    pub fn unsubscribe(&mut self, client: &str) -> bool {
        let removed = self.subscribers.remove(client);
        self.updated();
        removed
    }

    pub fn publish(&self, topic: &str, event: SessionEvent) {
        if !self.subscribers.is_empty() {
            publish(&["sessions:", &self.id, ":", topic].concat(), &event)
        }
    }

    pub fn updated(&self) {
        self.publish(
            "updated",
            SessionEvent::Updated {
                session: self.clone(),
            },
        )
    }

    pub fn heartbeat(&self) {
        self.publish(
            "heartbeat",
            SessionEvent::Heartbeat {
                session: self.clone(),
            },
        )
    }
}

/// A session store
#[derive(Debug)]
pub struct Sessions {
    sessions: Arc<RwLock<HashMap<String, Session>>>,

    /// The join handle for the monitoring thread.
    ///
    /// Held so that when this is dropped, the
    /// monitoring thread is aborted.
    monitoring: JoinHandle<()>,
}

impl Drop for Sessions {
    fn drop(&mut self) {
        self.monitoring.abort()
    }
}

impl Sessions {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Sessions {
        let sessions: Arc<RwLock<HashMap<String, Session>>> = Arc::new(RwLock::new(HashMap::new()));

        let sessions_clone = sessions.clone();
        let monitor = tokio::spawn(Sessions::monitor(sessions_clone));

        Sessions {
            sessions,
            monitoring: monitor,
        }
    }

    pub async fn start(&self, project: &str) -> Result<Session> {
        let mut sessions = self.sessions.write().await;
        let mut session = Session::new(project);
        session.start();
        sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    pub async fn stop(&self, session: &str) -> Result<Session> {
        let session = uuids::assert(Family::Session, session)?;
        let mut sessions = self.sessions.write().await;
        match sessions.entry(session.clone()) {
            Entry::Occupied(mut entry) => {
                let session = entry.get_mut();
                session.stop();
                Ok(entry.remove())
            }
            Entry::Vacant(..) => bail!("No session with id '{}'", session),
        }
    }

    pub async fn subscribe(&self, session: &str, client: &str) -> Result<Session> {
        let session = uuids::assert(Family::Session, session)?;
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session) {
            session.subscribe(client);
            Ok(session.clone())
        } else {
            bail!("No session with id '{}'", session)
        }
    }

    pub async fn unsubscribe(&self, session: &str, client: &str) -> Result<Session> {
        let session = uuids::assert(Family::Session, session)?;
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session) {
            session.unsubscribe(client);
            Ok(session.clone())
        } else {
            bail!("No session with id '{}'", session)
        }
    }

    async fn monitor(sessions: Arc<RwLock<HashMap<String, Session>>>) {
        use tokio::time::{sleep, Duration};

        loop {
            // Get a copy of all the current sessions
            // Doing this allows us to not hold a lock on the sessions while publishing
            // heartbeats AND sleeping. If this is not done the lock is held for 5 seconds
            // on each loop.
            let guard = sessions.read().await;
            let sessions = guard.values().into_iter().cloned().collect_vec();
            drop(guard);

            tracing::debug!("Monitoring {} sessions", sessions.len());
            for session in sessions {
                session.heartbeat()
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}

/// The global session store
pub static SESSIONS: Lazy<Sessions> = Lazy::new(Sessions::new);

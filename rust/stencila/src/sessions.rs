use crate::utils::schemas;
use defaults::Defaults;
use events::publish;
use eyre::{bail, Result};
use maplit::hashset;
use once_cell::sync::Lazy;
use schemars::{schema::Schema, JsonSchema};
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};
use tokio::{sync::RwLock, task::JoinHandle};

/// A session event
#[derive(Debug, JsonSchema, Serialize)]
#[serde(tag = "type")]
#[schemars(deny_unknown_fields)]
pub enum SessionEvent {
    /// One or more of the session's properties was updated
    Updated {
        #[schemars(schema_with = "SessionEvent::session_schema")]
        session: Session,
    },

    /// A heartbeat event
    Heartbeat {
        #[schemars(schema_with = "SessionEvent::session_schema")]
        session: Session,
    },
}

impl SessionEvent {
    fn session_schema<Generator>(_: Generator) -> Schema {
        schemas::typescript("Session", true)
    }
}

/// The status of a session
#[derive(Debug, Clone, JsonSchema, Serialize)]
pub enum SessionStatus {
    Pending,
    Starting,
    Started,
    Stopping,
    Stopped,
}

/// A session
#[derive(Debug, Clone, Defaults, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Session {
    /// The id of the session
    pub id: String,

    /// The id of the project that this session is for
    project: String,

    /// The clients that are subscribed to each topic for this session
    ///
    /// This is an optimization to avoid collecting session metrics
    /// and / or publishing events if there are no clients subscribed.
    subscriptions: HashMap<String, HashSet<String>>,

    /// The status of the session
    #[def = "SessionStatus::Pending"]
    status: SessionStatus,
}

impl Session {
    // Create a new session for a project
    pub fn new(project: &str) -> Session {
        Session {
            id: uuids::generate("se").to_string(),
            project: project.to_string(),
            ..Default::default()
        }
    }

    // Start the session
    pub fn start(&mut self) {
        self.status = SessionStatus::Started;
        self.updated();
    }

    // Stop the session
    pub fn stop(&mut self) {
        self.status = SessionStatus::Stopped;
        self.updated();
    }

    /// Generate a topic string for the session
    pub fn topic(&self, subtopic: &str) -> String {
        ["sessions:", &self.id, ":", subtopic].concat()
    }

    /// Subscribe a client to one of the session's topics
    pub fn subscribe(&mut self, topic: &str, client: &str) -> String {
        match self.subscriptions.entry(topic.into()) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().insert(client.into());
            }
            Entry::Vacant(vacant) => {
                vacant.insert(hashset! {client.into()});
            }
        };
        self.updated();
        self.topic(topic)
    }

    /// Unsubscribe a client from one of the session's topics
    pub fn unsubscribe(&mut self, topic: &str, client: &str) -> String {
        if let Entry::Occupied(mut occupied) = self.subscriptions.entry(topic.to_string()) {
            let subscribers = occupied.get_mut();
            subscribers.remove(client);
            if subscribers.is_empty() {
                occupied.remove();
            }
            self.updated();
        }
        self.topic(topic)
    }

    /// Get the number of subscribers to one of the session's topics
    fn subscribers(&self, topic: &str) -> usize {
        if let Some(subscriptions) = self.subscriptions.get(topic) {
            subscriptions.len()
        } else {
            0
        }
    }

    /// Publish an event for this session
    ///
    /// Checks that there is at least one subscriber for the topic before publishing
    fn publish(&self, topic: &str, event: SessionEvent) {
        if self.subscribers(topic) > 0 {
            publish(&self.topic(topic), &event)
        }
    }

    /// Publish an `Updated` event for this session
    fn updated(&self) {
        self.publish(
            "updated",
            SessionEvent::Updated {
                session: self.clone(),
            },
        )
    }

    /// Publish a `Heartbeat` event for this session
    fn heartbeat(&self) {
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
    sessions: Arc<RwLock<HashMap<String, RwLock<Session>>>>,

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
    /// Create a new sessions store
    ///
    /// Starts a session monitoring thread.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Sessions {
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let monitoring = tokio::spawn(Sessions::monitor(sessions.clone()));
        Sessions {
            sessions,
            monitoring,
        }
    }

    /// Start a session for a project
    pub async fn start(&self, project: &str) -> Result<Session> {
        let mut session = Session::new(project);
        session.start();
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), RwLock::new(session.clone()));
        Ok(session)
    }

    /// Stop a session
    pub async fn stop(&self, id: &str) -> Result<Session> {
        let id = uuids::assert("se", id.into())?.to_string();
        let mut sessions = self.sessions.write().await;
        match sessions.entry(id.clone()) {
            Entry::Occupied(entry) => {
                let mut session_guard = entry.get().write().await;
                session_guard.stop();
                drop(session_guard);

                let session = entry.remove().read().await.clone();
                Ok(session)
            }
            Entry::Vacant(..) => bail!("No session with id '{}'", id),
        }
    }

    /// Subscribe a client to a topic for a session
    pub async fn subscribe(
        &self,
        id: &str,
        topic: &str,
        client: &str,
    ) -> Result<(Session, String)> {
        let id = uuids::assert("se", id.into())?.to_string();
        let sessions = self.sessions.read().await;
        if let Some(session_lock) = sessions.get(&id) {
            let mut session_guard = session_lock.write().await;
            let topic = session_guard.subscribe(topic, client);
            drop(session_guard);

            let session = session_lock.read().await.clone();
            Ok((session, topic))
        } else {
            bail!("No session with id '{}'", id)
        }
    }

    /// Unsubscribe a client from a topic for a session
    pub async fn unsubscribe(
        &self,
        id: &str,
        topic: &str,
        client: &str,
    ) -> Result<(Session, String)> {
        let id = uuids::assert("se", id.into())?.to_string();
        let sessions = self.sessions.read().await;
        if let Some(session_lock) = sessions.get(&id) {
            let mut session_guard = session_lock.write().await;
            let topic = session_guard.unsubscribe(topic, client);
            drop(session_guard);

            let session = session_lock.read().await.clone();
            Ok((session, topic))
        } else {
            bail!("No session with id '{}'", id)
        }
    }

    /// Monitor sessions
    ///
    /// Generates heartbeat events for each session for which there are heartbeat subscriptions
    async fn monitor(sessions: Arc<RwLock<HashMap<String, RwLock<Session>>>>) {
        use tokio::time::{sleep, Duration};

        loop {
            // Get a copy of all the current sessions with heartbeat subscriptions.
            // Doing this allows us to not hold a lock on the sessions while publishing
            // heartbeats AND sleeping. If this is not done the lock is held for 5 seconds
            // on each loop.
            let sessions_guard = sessions.read().await;
            let mut sessions_heartbeats: Vec<Session> = Vec::with_capacity(sessions_guard.len());
            for session_lock in sessions_guard.values() {
                let session_guard = session_lock.read().await;
                if session_guard.subscribers("heartbeat") > 0 {
                    sessions_heartbeats.push(session_guard.clone())
                }
            }
            drop(sessions_guard);

            if !sessions_heartbeats.is_empty() {
                for session in sessions_heartbeats {
                    session.heartbeat()
                }
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}

/// The global session store
pub static SESSIONS: Lazy<Sessions> = Lazy::new(Sessions::new);

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Session>()?,
        schemas::generate::<SessionEvent>()?,
    ]);
    Ok(schemas)
}

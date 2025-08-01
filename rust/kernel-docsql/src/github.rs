use std::sync::{Arc, Mutex as SyncMutex};

use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result,
            tracing,
        },
        schema::{ExecutionMessage, Node},
    },
    minijinja::{
        Environment, Error, State, Value,
        value::Object,
    },
};

/// Add GitHub functions to the Jinja environment
pub(crate) fn add_github_functions(
    env: &mut Environment,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) {
    let github = Arc::new(GitHubQuery::new(messages.clone()));
    env.add_global("github", Value::from_object((*github).clone()));
}

/// GitHub query builder for generating API calls
#[derive(Debug, Clone)]
pub(crate) struct GitHubQuery {
    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// The GitHub entity type (e.g. repositories)
    entity_type: String,
}

impl GitHubQuery {
    /// Create a new GitHub query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            messages,
            entity_type: String::new(),
        }
    }

    /// Whether this is the base query for which no method has been called yet
    pub fn is_base(&self) -> bool {
        self.entity_type.is_empty()
    }

    /// Execute the query and return the resulting [`Node`]s
    #[tracing::instrument(skip(self))]
    pub fn nodes(&self) -> Vec<Node> {
        todo!()
    }
}

impl Object for GitHubQuery {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        todo!()
    }
}

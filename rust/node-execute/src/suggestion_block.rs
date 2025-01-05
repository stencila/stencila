use schema::{SuggestionBlock, SuggestionStatus};

use crate::prelude::*;

impl Executable for SuggestionBlock {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        match self.suggestion_status {
            Some(SuggestionStatus::Original | SuggestionStatus::Accepted) => {
                // Suggestion is original or has been accepted so compile using the main executor
                tracing::trace!("Compiling accepted suggestion block `{node_id}`");
                if let Err(error) = self.content.walk_async(executor).await {
                    tracing::error!("While compiling suggestion block: {error}");
                }
            }
            Some(SuggestionStatus::Rejected) | None => {
                // Suggestion is proposed or rejected so compile within a fork
                tracing::trace!("Compiling proposed or rejected suggestion block `{node_id}` with forked executor");
                let mut fork = executor.fork_for_compile();
                if let Err(error) = self.content.walk_async(&mut fork).await {
                    tracing::error!("While compiling suggestion block: {error}");
                }
            }
        }

        // Break walk because content already compiled above
        return WalkControl::Break;
    }

    #[tracing::instrument(skip_all)]
    async fn prepare(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        match self.suggestion_status {
            Some(SuggestionStatus::Original | SuggestionStatus::Accepted) | None => {
                // Suggestion is original, has been accepted, or is proposed so prepare using the main executor
                // which will set the status of descendent nodes to `ExecutionStatus::Pending`
                tracing::trace!("Preparing accepted suggestion block `{node_id}`");
                if let Err(error) = self.content.walk_async(executor).await {
                    tracing::error!("While preparing suggestion block: {error}");
                }
            }
            Some(SuggestionStatus::Rejected) => {
                // Suggestion has been rejected so prepare within an executor fork which will
                // set the status of descendent nodes to `ExecutionStatus::Rejected` (so that rejected
                // nodes are not executed unless explicitly)
                tracing::trace!("Preparing proposed or rejected suggestion block `{node_id}` with forked executor");
                let mut fork = executor.fork_for_prepare(ExecutionStatus::Rejected);
                if let Err(error) = self.content.walk_async(&mut fork).await {
                    tracing::error!("While preparing suggestion block: {error}");
                }
            }
        }

        // Break walk because content already prepared above
        return WalkControl::Break;
    }

    #[tracing::instrument(skip_all)]
    async fn execute(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();

        match self.suggestion_status {
            Some(SuggestionStatus::Original | SuggestionStatus::Accepted) => {
                // Suggestion is the original or has been accepted so execute in the main kernel set
                tracing::trace!("Executing accepted suggestion block `{node_id}`");
                if let Err(error) = self.content.walk_async(executor).await {
                    tracing::error!("While executing suggestion block: {error}");
                }
            }
            Some(SuggestionStatus::Rejected) | None => {
                // Suggestion is rejected or proposed so execute within a fork, but only if that is possible
                let forkable = executor.kernels().await.supports_forks().await;
                if forkable {
                    tracing::trace!(
                        "Executing proposed suggestion block `{node_id}` with forked executor"
                    );
                    match executor.fork_for_execute().await {
                        Ok(mut fork) => {
                            if let Err(error) =
                                fork.compile_prepare_execute(&mut self.content).await
                            {
                                tracing::error!("While executing suggestion block: {error}");
                            }
                        }
                        Err(error) => {
                            tracing::error!("While forking suggestion block executor: {error}")
                        }
                    };
                } else {
                    tracing::debug!("Skipping execution of proposed suggestion block `{node_id}` because kernels are not forkable");
                }
            }
        }

        // Break walk because content already executed (or not) above
        return WalkControl::Break;
    }
}

//! App config
///
/// Likely to be user configurable in the future.
#[allow(unused)]
#[derive(Default)]
pub(crate) struct AppConfig {
    /// Verbosity of agent thinking segments
    ///
    /// Use None to not show a thinking segment at all. Otherwise represents the
    /// number of lines (raw lines) of thinking to show.
    pub thinking_verbosity: Option<usize>,

    /// Verbosity of workflow runs
    ///
    /// How much detail to show for workflow pipeline runs.
    pub workflow_verbosity: WorkflowVerbosity,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkflowVerbosity {
    /// Single updating message for the whole pipeline run.
    Minimal,
    /// Messages at the start and end of the pipeline and one updating message
    /// per stage.
    Simple,
    /// Full exchange per stage (prompt + streaming response).
    #[default]
    Detailed,
}

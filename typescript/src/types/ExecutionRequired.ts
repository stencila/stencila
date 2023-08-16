// Generated file. Do not edit; see `rust/schema-gen` crate.\n\n
            
// Under which circumstances the document node should be automatically executed.
export type ExecutionRequired =
  'No' |
  'NeverExecuted' |
  'SemanticsChanged' |
  'DependenciesChanged' |
  'DependenciesFailed' |
  'Failed' |
  'KernelRestarted';

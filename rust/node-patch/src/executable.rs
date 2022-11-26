//! Patching for executable nodes

use stencila_schema::*;

use super::prelude::*;

patchable_struct!(
    CodeChunk,
    id,
    programming_language,
    guess_language,
    code,
    outputs,
    errors,
    label,
    caption,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execution_auto,
    execution_pure,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);
patchable_variants!(
    CodeChunkCaption,
    CodeChunkCaption::VecBlockContent,
    CodeChunkCaption::String
);

patchable_struct!(
    CodeExpression,
    id,
    programming_language,
    guess_language,
    code,
    output,
    errors,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);

patchable_struct!(
    ExecutionDigest,
    state_digest,
    semantic_digest,
    dependencies_digest,
    dependencies_failed,
    dependencies_stale
);

// Make `CodeError`s replaceable to avoid large patches associated with
// changes in stack trace and to simplify Web Component development (no
// need to observe for change in error level or presence/absence of stacktrace)
replaceable_struct!(CodeError, error_message, error_type, stack_trace);

patchable_struct!(ExecutionDependency, dependency_relation, dependency_node);
patchable_enum!(ExecutionDependencyRelation);
patchable_variants!(
    ExecutionDependencyNode,
    ExecutionDependencyNode::Button,
    ExecutionDependencyNode::CodeChunk,
    ExecutionDependencyNode::File,
    ExecutionDependencyNode::Parameter,
    ExecutionDependencyNode::SoftwareSourceCode,
    ExecutionDependencyNode::Variable
);

patchable_struct!(ExecutionDependent, dependent_relation, dependent_node);
patchable_enum!(ExecutionDependentRelation);
patchable_variants!(
    ExecutionDependentNode,
    ExecutionDependentNode::Button,
    ExecutionDependentNode::Call,
    ExecutionDependentNode::CodeChunk,
    ExecutionDependentNode::CodeExpression,
    ExecutionDependentNode::Division,
    ExecutionDependentNode::File,
    ExecutionDependentNode::Parameter,
    ExecutionDependentNode::Span,
    ExecutionDependentNode::Variable
);

patchable_enum!(ExecutionAuto);
patchable_enum!(ExecutionRequired);
patchable_enum!(ExecutionStatus);

patchable_struct!(
    Division,
    id,
    programming_language,
    guess_language,
    code,
    css,
    classes,
    errors,
    content,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execution_auto,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);

patchable_struct!(
    Span,
    id,
    programming_language,
    guess_language,
    code,
    css,
    classes,
    errors,
    content,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_auto,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);

patchable_struct!(
    Include,
    id,
    source,
    media_type,
    select,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
    content,
);

patchable_struct!(
    Call,
    id,
    source,
    media_type,
    select,
    arguments,
    execution_dependencies,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
    content,
);
patchable_struct!(
    CallArgument,
    id,
    name,
    validator,
    value,
    code,
    programming_language,
    guess_language
);

patchable_struct!(
    For,
    id,
    symbol,
    code,
    programming_language,
    guess_language,
    content,
    otherwise,
    iterations,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
);

patchable_struct!(
    Form,
    id,
    derive_from,
    derive_action,
    derive_item,
    content,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
);

patchable_enum!(FormDeriveAction);
patchable_variants!(
    FormDeriveItem,
    FormDeriveItem::Integer,
    FormDeriveItem::String
);

patchable_struct!(
    If,
    id,
    clauses,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
);
patchable_struct!(
    IfClause,
    code,
    programming_language,
    guess_language,
    is_active,
    content,
    errors,
);

patchable_struct!(
    Button,
    id,
    name,
    label,
    code,
    programming_language,
    guess_language,
    is_disabled,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_count,
    execution_ended
);

// Previously we implemented a custom `Patchable` for `Parameter` to ensure that values of
// `default` and `value` fields (which can be any `Node`) meet the requirements of the `validator`.
// However, doing that here causes a lot of inconsistencies, especially in the UI. For that
// reason we revert to using standard struct macro here.
patchable_struct!(
    Parameter,
    id,
    name,
    label,
    derived_from,
    validator,
    default,
    value,
    errors,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_count,
    execution_ended
);

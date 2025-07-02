use common::tracing;

/// Generate warnings if a node is being encoded with the `--reproducible` option but
/// does not have necessary metadata
pub fn reproducible_warnings(source: &Option<String>, commit: &Option<String>) {
    let Some(source) = source else {
        tracing::warn!(
            "Reproducible document created from a file that does not appear to be within a Git repository. This may make it harder to resolve conflicts when merging changes."
        );
        return;
    };

    if commit.is_none() {
        tracing::warn!(
            "Reproducible document created from file `{source}` whose Git commit is unknown. You may need to specify the commit when merging changes."
        );
    } else if matches!(commit.as_deref(), Some("untracked")) {
        tracing::warn!(
            "Reproducible document created from file `{source}` which is in a Git repository but not tracked. Consider committing this file to be able to merge changes correctly."
        );
    } else if matches!(commit.as_deref(), Some("dirty")) {
        tracing::warn!(
            "Reproducible document created from file `{source}` which is has uncommitted changes. Consider committing these changes to be able to merge changes correctly."
        );
    }
}

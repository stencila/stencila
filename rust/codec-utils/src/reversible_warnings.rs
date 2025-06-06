use common::tracing;

/// Generate warnings if a node is being encoded with the `--reversible` option but
/// does not have necessary metadata
pub fn reversible_warnings(source: &Option<String>, commit: &Option<String>) {
    let Some(source) = source else {
        tracing::warn!(
            "`--reversible` flag used with a document that does not appear to be within a Git repository. You will need to specify the source file when reversing out changes."
        );
        return;
    };

    if commit.is_none() {
        tracing::warn!(
            "`--reversible` flag used with file `{source}` whose Git commit is unknown. You will likely need to specify the commit when reversing out changes."
        );
    } else if matches!(commit.as_deref(), Some("untracked")) {
        tracing::warn!(
            "`--reversible` flag used with file `{source}` which is in a Git repository but not tracked. Consider committing this file to be able to reverse changes correctly."
        );
    } else if matches!(commit.as_deref(), Some("dirty")) {
        tracing::warn!(
            "`--reversible` flag used with file `{source}` which is has uncommitted changes. Consider committing these changes to be able to reverse changes correctly."
        );
    }
}

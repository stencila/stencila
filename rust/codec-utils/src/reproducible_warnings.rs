use common::tracing;

/// Generate warnings if a node is being encoded with the `--reproducible` option but
/// does not have necessary metadata
pub fn reproducible_warnings(source: &Option<String>, commit: &Option<String>) {
    let Some(source) = source else {
        tracing::warn!(
            "`--reproducible` flag used with a document that does not appear to be within a Git repository. You will need to specify the source file when reversing out changes."
        );
        return;
    };

    if commit.is_none() {
        tracing::warn!(
            "`--reproducible` flag used with file `{source}` whose Git commit is unknown. You will likely need to specify the commit when reversing out changes."
        );
    } else if matches!(commit.as_deref(), Some("untracked")) {
        tracing::warn!(
            "`--reproducible` flag used with file `{source}` which is in a Git repository but not tracked. Consider committing this file to be able to reverse changes correctly."
        );
    } else if matches!(commit.as_deref(), Some("dirty")) {
        tracing::warn!(
            "`--reproducible` flag used with file `{source}` which is has uncommitted changes. Consider committing these changes to be able to reverse changes correctly."
        );
    }
}

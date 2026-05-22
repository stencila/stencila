//! Reference and path helpers shared by graph collectors.
//!
//! Document links, source-code literals, and workspace entries all need the same
//! answer to a few small questions: is this string a local relative path, which
//! workspace-relative path does it name, and is a URI clearly non-local? Keeping
//! those rules in one module avoids edge-case drift between collectors.

use std::path::{Component, Path, PathBuf};

use percent_encoding::percent_decode_str;

use crate::ids::WorkspaceRelPath;

/// Check whether a reference has a URI scheme that should not be localized.
pub(crate) fn has_non_local_uri_scheme(reference: &str) -> bool {
    let Some(colon) = reference.find(':') else {
        return false;
    };

    let scheme = &reference[..colon];
    if scheme.contains(['/', '?', '#']) {
        return false;
    }

    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_alphabetic()
        || !chars.all(|char| char.is_ascii_alphanumeric() || matches!(char, '+' | '-' | '.'))
    {
        return false;
    }

    let rest = &reference[(colon + 1)..];
    if rest.starts_with("//") || rest.starts_with('\\') {
        return true;
    }

    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "data"
            | "doi"
            | "file"
            | "ftp"
            | "ftps"
            | "gs"
            | "http"
            | "https"
            | "mailto"
            | "s3"
            | "urn"
    )
}

/// Check whether a reference should be interpreted as a local relative path.
pub(crate) fn is_local_relative_reference(reference: &str) -> bool {
    let reference = reference.trim();
    if reference.is_empty()
        || reference.starts_with('#')
        || reference.starts_with('/')
        || reference.starts_with("//")
        || has_non_local_uri_scheme(reference)
    {
        return false;
    }

    !Path::new(reference).is_absolute()
}

/// Build possible path spellings from a URL-like local reference.
pub(crate) fn reference_path_candidates(reference: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    push_reference_candidate(reference, &mut candidates);

    if let Some(stripped) = strip_query_or_fragment(reference)
        && !stripped.is_empty()
    {
        push_reference_candidate(stripped, &mut candidates);
    }

    candidates
}

/// Resolve a document-relative path reference into a workspace-relative path.
pub(crate) fn document_relative_workspace_path(
    document_rel: &WorkspaceRelPath,
    reference: &str,
) -> Option<WorkspaceRelPath> {
    let reference = Path::new(reference);
    if reference.is_absolute() {
        return None;
    }

    let mut path = PathBuf::new();
    if let Some(parent) = document_rel.parent()
        && parent.as_str() != "."
    {
        path.push(parent.as_str());
    }
    path.push(reference);

    let path = normalize_path_lexically(&path);
    WorkspaceRelPath::from_relative_path(&path).ok()
}

/// Normalize `.` and `..` path components without dereferencing symlinks.
pub(crate) fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            component => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

/// Push raw and percent-decoded forms of one reference path candidate.
fn push_reference_candidate(candidate: &str, candidates: &mut Vec<String>) {
    push_unique_candidate(candidate.to_string(), candidates);

    if let Ok(decoded) = percent_decode_str(candidate).decode_utf8()
        && decoded != candidate
    {
        push_unique_candidate(decoded.into_owned(), candidates);
    }
}

/// Push a candidate once, preserving priority order.
fn push_unique_candidate(candidate: String, candidates: &mut Vec<String>) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

/// Strip URL query or fragment suffixes from a local path reference.
fn strip_query_or_fragment(reference: &str) -> Option<&str> {
    reference.find(['?', '#']).map(|index| &reference[..index])
}

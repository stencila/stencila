//! PDF helpers for embedding C2PA manifests.
//!
//! The c2pa SDK can read embedded PDF manifests, but its public stream
//! writer registry does not expose PDF writing. These helpers provide the
//! minimal PDF object manipulation needed by the content-credentials producer
//! while keeping PDF internals out of the signing workflow.

use std::{ops::Range, path::Path};

use lopdf::{
    Document, Object,
    Object::{Array, Integer, Name, Reference},
    ObjectId, Stream, dictionary,
};

use crate::error::{Error, Result};

const AF_RELATIONSHIP_KEY: &[u8] = b"AFRelationship";
const ASSOCIATED_FILE_KEY: &[u8] = b"AF";
const C2PA_RELATIONSHIP: &[u8] = b"C2PA_Manifest";
const CONTENT_CREDS: &str = "Content Credentials";
const C2PA_MIME_TYPE: &str = "application/x-c2pa-manifest-store";
const EMBEDDED_FILES_KEY: &[u8] = b"EmbeddedFiles";
const NAMES_KEY: &[u8] = b"Names";
const SUBTYPE_KEY: &[u8] = b"Subtype";
const TYPE_KEY: &[u8] = b"Type";

/// Return PDF bytes containing the supplied manifest plus its byte range.
///
/// PDF signing uses the c2pa placeholder workflow. The producer needs to
/// insert placeholder bytes first, hash the resulting PDF while excluding that
/// exact byte range, and later patch the signed manifest into the same range so
/// the PDF structure and hard binding stay consistent.
pub(super) fn with_manifest_bytes(
    input_path: &Path,
    manifest_bytes: &[u8],
) -> Result<(Vec<u8>, Range<usize>)> {
    let mut document = Document::load(input_path)?;
    remove_manifest(&mut document)?;
    add_manifest(&mut document, manifest_bytes.to_vec())?;

    let mut bytes = Vec::new();
    document.save_to(&mut bytes)?;
    let range = find_unique_subslice(&bytes, manifest_bytes).ok_or_else(|| {
        Error::other("embedded PDF manifest placeholder could not be located in output PDF")
    })?;

    Ok((bytes, range))
}

/// Remove existing C2PA manifest objects from a PDF document.
///
/// Re-signing should replace Stencila's previous embedded manifest rather
/// than append another C2PA file specification. The SDK currently reads only
/// one embedded PDF manifest, so leaving stale entries can make verification
/// pick up old credentials instead of the manifest just signed.
fn remove_manifest(document: &mut Document) -> Result<()> {
    let file_spec_refs = c2pa_file_spec_refs(document);
    if file_spec_refs.is_empty() {
        return Ok(());
    }

    remove_associated_file_refs(document, &file_spec_refs)?;
    remove_embedded_file_names(document, &file_spec_refs)?;

    for file_spec_ref in file_spec_refs {
        if let Some(file_stream_ref) = file_stream_ref(document, file_spec_ref) {
            document.delete_object(file_stream_ref);
        }
        document.delete_object(file_spec_ref);
    }

    Ok(())
}

/// Collect associated-file object IDs that are marked as C2PA manifests.
///
/// PDF associated files can contain arbitrary attachments. Filtering by
/// `/AFRelationship /C2PA_Manifest` lets removal target only C2PA credentials
/// and leave unrelated user attachments intact.
fn c2pa_file_spec_refs(document: &Document) -> Vec<ObjectId> {
    let Ok(associated_files) = associated_files(document) else {
        return Vec::new();
    };

    associated_files
        .iter()
        .filter_map(|object| object.as_reference().ok())
        .filter(|object_id| {
            document
                .get_object(*object_id)
                .and_then(Object::as_dict)
                .and_then(|dict| dict.get(AF_RELATIONSHIP_KEY))
                .and_then(Object::as_name)
                .is_ok_and(|name| name == C2PA_RELATIONSHIP)
        })
        .collect()
}

/// Read the document catalog's associated-file array.
///
/// The `/AF` entry may be inline or indirect. Normalizing both cases to an
/// owned vector keeps callers simple and avoids holding immutable borrows while
/// later mutating the document.
fn associated_files(document: &Document) -> Result<Vec<Object>> {
    let associated_files = document.catalog()?.get(ASSOCIATED_FILE_KEY)?;
    let associated_files = if let Ok(object_id) = associated_files.as_reference() {
        document.get_object(object_id)?
    } else {
        associated_files
    };

    Ok(associated_files.as_array()?.clone())
}

/// Remove C2PA file-spec references from the catalog associated files.
///
/// Deleting the file-spec objects alone would leave dangling references in
/// `/AF`. Cleaning the catalog relationship keeps the PDF object graph coherent
/// for readers that inspect associated files before resolving embedded names.
fn remove_associated_file_refs(document: &mut Document, file_spec_refs: &[ObjectId]) -> Result<()> {
    let associated_files_ref = document
        .catalog()?
        .get(ASSOCIATED_FILE_KEY)
        .ok()
        .and_then(|object| object.as_reference().ok());

    let associated_files = if let Some(object_id) = associated_files_ref {
        document.get_object_mut(object_id)?.as_array_mut()?
    } else {
        document
            .catalog_mut()?
            .get_mut(ASSOCIATED_FILE_KEY)?
            .as_array_mut()?
    };

    associated_files.retain(|object| {
        object
            .as_reference()
            .map_or(true, |object_id| !file_spec_refs.contains(&object_id))
    });

    Ok(())
}

/// Remove C2PA entries from the catalog embedded-file name tree.
///
/// PDF viewers and validators may discover attachments through `/Names`
/// rather than `/AF`. Removing the matching name/value pairs prevents stale
/// Content Credentials attachments from remaining visible after re-signing.
fn remove_embedded_file_names(document: &mut Document, file_spec_refs: &[ObjectId]) -> Result<()> {
    let names_ref = document
        .catalog()?
        .get(NAMES_KEY)
        .ok()
        .and_then(|object| object.as_reference().ok());
    let Some(names_ref) = names_ref else {
        return Ok(());
    };

    let embedded_files_ref = document
        .get_object(names_ref)?
        .as_dict()?
        .get(EMBEDDED_FILES_KEY)
        .ok()
        .and_then(|object| object.as_reference().ok());
    let Some(embedded_files_ref) = embedded_files_ref else {
        return Ok(());
    };

    let embedded_files = document.get_object_mut(embedded_files_ref)?.as_dict_mut()?;
    let Ok(names) = embedded_files
        .get_mut(NAMES_KEY)
        .and_then(Object::as_array_mut)
    else {
        return Ok(());
    };

    let mut retained = Vec::with_capacity(names.len());
    for pair in names.chunks(2) {
        match pair {
            [_name, Reference(file_spec_ref)] if file_spec_refs.contains(file_spec_ref) => {}
            [name, value] => {
                retained.push(name.clone());
                retained.push(value.clone());
            }
            [single] => retained.push(single.clone()),
            _ => {}
        }
    }
    *names = retained;

    Ok(())
}

/// Resolve a C2PA file specification to its embedded file stream object.
///
/// After a manifest file spec is detached from `/AF` and `/Names`, its
/// stream object is otherwise unreachable but still serialized. Returning the
/// stream reference lets removal delete both descriptor and bytes together.
fn file_stream_ref(document: &Document, file_spec_ref: ObjectId) -> Option<ObjectId> {
    document
        .get_object(file_spec_ref)
        .ok()?
        .as_dict()
        .ok()?
        .get(b"EF")
        .ok()?
        .as_dict()
        .ok()?
        .get(b"F")
        .ok()?
        .as_reference()
        .ok()
}

/// Add C2PA manifest bytes as an embedded PDF file specification.
///
/// The C2PA PDF convention stores the manifest as an associated embedded
/// file with `/AFRelationship /C2PA_Manifest`. Creating both the stream and
/// file specification gives c2pa readers a standard place to find the manifest.
fn add_manifest(document: &mut Document, manifest_bytes: Vec<u8>) -> Result<()> {
    let manifest_len = i64::try_from(manifest_bytes.len())
        .map_err(|_| Error::other("PDF C2PA manifest is too large to embed"))?;
    let file_stream_ref = document.add_object(Stream::new(
        dictionary! {
            "F" => dictionary! {
                SUBTYPE_KEY => C2PA_MIME_TYPE,
                "Length" => Integer(manifest_len),
            },
        },
        manifest_bytes,
    ));
    let file_spec_ref = document.add_object(dictionary! {
        AF_RELATIONSHIP_KEY => Name(C2PA_RELATIONSHIP.into()),
        "Desc" => Object::string_literal(CONTENT_CREDS),
        "F" => Object::string_literal(CONTENT_CREDS),
        "EF" => dictionary! {
            "F" => Reference(file_stream_ref),
        },
        TYPE_KEY => Name("FileSpec".into()),
        "UF" => Object::string_literal(CONTENT_CREDS),
    });

    push_associated_file(document, file_spec_ref)?;
    add_embedded_file_name(document, file_spec_ref)?;
    Ok(())
}

/// Append the C2PA file specification to the catalog associated files.
///
/// `/AF` is the semantic binding between the PDF document and its C2PA
/// manifest. The entry may already exist inline or by reference, so this helper
/// handles both shapes while preserving any existing associated files.
fn push_associated_file(document: &mut Document, file_spec_ref: ObjectId) -> Result<()> {
    {
        let catalog = document.catalog_mut()?;
        if catalog.get_mut(ASSOCIATED_FILE_KEY).is_err() {
            catalog.set(ASSOCIATED_FILE_KEY, Array(Vec::new()));
        }
    }

    let associated_files_ref = document
        .catalog()?
        .get(ASSOCIATED_FILE_KEY)
        .ok()
        .and_then(|object| object.as_reference().ok());

    if let Some(object_id) = associated_files_ref {
        document
            .get_object_mut(object_id)?
            .as_array_mut()?
            .push(Reference(file_spec_ref));
    } else {
        document
            .catalog_mut()?
            .get_mut(ASSOCIATED_FILE_KEY)?
            .as_array_mut()?
            .push(Reference(file_spec_ref));
    }

    Ok(())
}

/// Add the C2PA file specification to the embedded-file name tree.
///
/// Many PDF tools surface attachments through `/Names /EmbeddedFiles`.
/// Updating or creating that structure improves interoperability while the
/// `/AF` relationship carries the C2PA-specific semantics.
fn add_embedded_file_name(document: &mut Document, file_spec_ref: ObjectId) -> Result<()> {
    let name_pair = vec![
        Object::string_literal(CONTENT_CREDS),
        Reference(file_spec_ref),
    ];

    if document.catalog()?.get(NAMES_KEY).is_err() {
        let embedded_files_ref = document.add_object(dictionary! {
            NAMES_KEY => name_pair,
        });
        let names_ref = document.add_object(dictionary! {
            EMBEDDED_FILES_KEY => Reference(embedded_files_ref),
        });
        document.catalog_mut()?.set(NAMES_KEY, Reference(names_ref));
        return Ok(());
    }

    let names_ref = document.catalog()?.get(NAMES_KEY)?.as_reference().ok();
    let names_dictionary = if let Some(object_id) = names_ref {
        document.get_object_mut(object_id)?.as_dict_mut()?
    } else {
        document.catalog_mut()?.get_mut(NAMES_KEY)?.as_dict_mut()?
    };

    if names_dictionary.get(EMBEDDED_FILES_KEY).is_err() {
        names_dictionary.set(
            EMBEDDED_FILES_KEY,
            dictionary! {
                NAMES_KEY => name_pair,
            },
        );
        return Ok(());
    }

    let embedded_files_ref = names_dictionary
        .get(EMBEDDED_FILES_KEY)?
        .as_reference()
        .ok();

    let embedded_files_dictionary = if let Some(object_id) = embedded_files_ref {
        document.get_object_mut(object_id)?.as_dict_mut()?
    } else {
        names_dictionary
            .get_mut(EMBEDDED_FILES_KEY)?
            .as_dict_mut()?
    };

    if embedded_files_dictionary.get(NAMES_KEY).is_err() {
        embedded_files_dictionary.set(NAMES_KEY, Array(name_pair));
        return Ok(());
    }

    let names_array_ref = embedded_files_dictionary
        .get(NAMES_KEY)?
        .as_reference()
        .ok();
    if let Some(object_id) = names_array_ref {
        document
            .get_object_mut(object_id)?
            .as_array_mut()?
            .extend(name_pair);
    } else {
        embedded_files_dictionary
            .get_mut(NAMES_KEY)?
            .as_array_mut()?
            .extend(name_pair);
    }

    Ok(())
}

/// Find one exact byte range for a manifest payload.
///
/// After lopdf serializes the document, the producer must know where the
/// placeholder bytes landed so it can exclude that range from the hard binding
/// and patch the final manifest in-place. Duplicate matches are rejected because
/// patching the wrong occurrence would create an invalid signature.
fn find_unique_subslice(haystack: &[u8], needle: &[u8]) -> Option<Range<usize>> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    let mut matches = haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(index, window)| (window == needle).then_some(index));
    let start = matches.next()?;
    matches
        .next()
        .is_none()
        .then_some(start..start + needle.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_unique_subslice_rejects_empty_missing_and_duplicate() {
        assert_eq!(find_unique_subslice(b"abcdef", b"cd"), Some(2..4));
        assert_eq!(find_unique_subslice(b"abcdef", b""), None);
        assert_eq!(find_unique_subslice(b"abcdef", b"xy"), None);
        assert_eq!(find_unique_subslice(b"abcabc", b"ab"), None);
    }
}

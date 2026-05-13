//! Readable, collision-safe filenames for extracted and collected media.
//!
//! Media used to be named only by content hash, which is deterministic but
//! opaque in exported folders. This module keeps the hash as a fallback while
//! preferring nearby author-facing ids such as `fig-1`. It lives in
//! `node-media` so extraction and collection share the same rules without
//! changing the generated schema walker APIs.

use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use eyre::Result;
use seahash::SeaHasher;
use tempfile::NamedTempFile;

use stencila_schema::{CodeChunk, Figure, LabelType};

/// Tracks readable media filename state while walking a document tree.
///
/// The extract and collect visitors push naming contexts when they intercept
/// Figures and CodeChunks. That lets media below those nodes get filenames such
/// as `fig-1.png` or `fig-1a.png`, while this type still
/// handles duplicate names and falls back to hashes when no useful id exists.
#[derive(Default)]
pub struct MediaNamer {
    /// Stack of active Figure and CodeChunk naming contexts.
    ///
    /// A stack is needed because Figures may contain subfigures or executable
    /// figure chunks. The innermost context names the next media item, while
    /// parent figure contexts allocate alpha suffixes for child figure-like
    /// contexts.
    contexts: Vec<NamingContext>,

    /// Paths already produced by this namer, keyed by their content hash.
    ///
    /// This avoids reusing the same requested stem for different bytes during a
    /// single walk, even before the filesystem collision check gets involved.
    used: HashMap<PathBuf, u64>,

    /// First path produced for a content hash and extension during this walk.
    ///
    /// This preserves hash-based deduplication even when later media objects ask
    /// for a different readable stem.
    by_hash: HashMap<ContentKey, PathBuf>,

    /// Whether readable stems should include the content hash on the first
    /// candidate path.
    ///
    /// Collection writes into shared output directories, including parallel site
    /// renders, so hash suffixes avoid cross-document name races while keeping
    /// filenames readable.
    hash_readable_names: bool,
}

/// A single active source of filename stems.
///
/// Contexts carry just enough state to name anonymous media from a surrounding
/// Figure or CodeChunk, or to allocate an alpha fallback for anonymous
/// subfigures.
struct NamingContext {
    /// Slugified base stem from an author-facing id, if one is available.
    stem: Option<String>,

    /// Whether this context came from a Figure.
    ///
    /// Only Figure contexts allocate alpha suffixes for child subfigures; a
    /// CodeChunk nested inside another CodeChunk should not be treated as a
    /// figure parent just because it has a stem.
    is_figure: bool,

    /// Next zero-based alpha suffix for child subfigures.
    next_subfigure: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ContentKey {
    hash: u64,
    extension: String,
}

impl ContentKey {
    fn new(hash: u64, extension: &str) -> Self {
        Self {
            hash,
            extension: extension.to_string(),
        }
    }
}

impl MediaNamer {
    /// Create an empty media namer.
    ///
    /// Visitors construct one namer per extraction or collection walk so
    /// collision tracking is scoped to a single output operation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a media namer that includes hashes in readable filenames.
    pub fn with_hashed_readable_names() -> Self {
        Self {
            hash_readable_names: true,
            ..Default::default()
        }
    }

    /// Push a Figure naming context.
    ///
    /// Figures use their own `id` as the top-level stem. Anonymous nested
    /// figures receive the parent's next alpha suffix as a fallback.
    pub fn push_figure(&mut self, figure: &Figure) {
        let stem = self.figure_stem(figure);
        self.contexts.push(NamingContext {
            stem,
            is_figure: true,
            next_subfigure: 0,
        });
    }

    /// Push a CodeChunk naming context.
    ///
    /// CodeChunks use their own `id`. An anonymous FigureLabel chunk inside a
    /// Figure is treated as a subfigure and receives the parent's alpha suffix.
    pub fn push_code_chunk(&mut self, chunk: &CodeChunk) {
        let stem = self.code_chunk_stem(chunk);
        self.contexts.push(NamingContext {
            stem,
            is_figure: false,
            next_subfigure: 0,
        });
    }

    /// Pop the current naming context after its node has been walked.
    ///
    /// The extract and collect visitors manually walk intercepted Figure and
    /// CodeChunk nodes, so they also own balancing these pushes and pops.
    pub fn pop(&mut self) {
        self.contexts.pop();
    }

    /// Return the stem for the next media item.
    ///
    /// The media object's own id wins over any context because generated
    /// subfigure ids such as `fig-1a` are already stable and unique. Anonymous
    /// media inherit the nearest Figure or CodeChunk context.
    pub fn next_media_stem(&mut self, media_id: Option<&str>) -> Option<String> {
        if let Some(stem) = media_id.and_then(slugify) {
            return Some(stem);
        }

        self.contexts
            .last()
            .and_then(|context| context.stem.clone())
    }

    /// Write decoded data URI bytes using the requested readable stem.
    ///
    /// The caller supplies a content hash so this method can reuse an existing
    /// identical file, avoid overwriting different bytes, and fall back to
    /// hash-based names only when the readable stem collides.
    pub fn write_bytes(
        &mut self,
        media_dir: &Path,
        desired_stem: Option<&str>,
        extension: &str,
        hash: u64,
        bytes: &[u8],
    ) -> Result<PathBuf> {
        create_dir_all(media_dir)?;

        let key = ContentKey::new(hash, extension);
        if let Some(path) = self.by_hash.get(&key)
            && file_matches_bytes(path, bytes)?
        {
            return Ok(path.clone());
        }

        for path in Self::candidate_paths(
            media_dir,
            desired_stem,
            extension,
            hash,
            self.hash_readable_names,
        ) {
            if self.path_used_with_different_hash(&path, hash) {
                continue;
            }

            if path.exists() {
                if file_matches_bytes(&path, bytes)? {
                    self.remember_path(path.clone(), hash, extension);
                    return Ok(path);
                }

                let temp_file = bytes_to_temp_file(media_dir, bytes)?;
                persist_replacing(temp_file, &path)?;
                self.remember_path(path.clone(), hash, extension);
                return Ok(path);
            }

            let temp_file = bytes_to_temp_file(media_dir, bytes)?;
            if persist_without_clobbering(temp_file, &path)? || file_matches_bytes(&path, bytes)? {
                self.remember_path(path.clone(), hash, extension);
                return Ok(path);
            }
        }

        unreachable!("candidate_paths yields an unbounded sequence")
    }

    /// Copy a local media file using the requested readable stem.
    ///
    /// This mirrors [`write_bytes`](Self::write_bytes) for collection: readable
    /// names are preferred, identical files are reused, and a hash suffix keeps
    /// unrelated files from clobbering each other.
    pub fn copy_file(
        &mut self,
        source_path: &Path,
        media_dir: &Path,
        desired_stem: Option<&str>,
        extension: &str,
        hash: u64,
    ) -> Result<PathBuf> {
        create_dir_all(media_dir)?;

        let key = ContentKey::new(hash, extension);
        if let Some(path) = self.by_hash.get(&key)
            && files_match(source_path, path)?
        {
            return Ok(path.clone());
        }

        for path in Self::candidate_paths(
            media_dir,
            desired_stem,
            extension,
            hash,
            self.hash_readable_names,
        ) {
            if self.path_used_with_different_hash(&path, hash) {
                continue;
            }

            if path.exists() {
                if files_match(source_path, &path)? {
                    self.remember_path(path.clone(), hash, extension);
                    return Ok(path);
                }

                let temp_file = copy_to_temp_file(source_path, media_dir)?;
                persist_replacing(temp_file, &path)?;
                self.remember_path(path.clone(), hash, extension);
                return Ok(path);
            }

            let temp_file = copy_to_temp_file(source_path, media_dir)?;
            if persist_without_clobbering(temp_file, &path)? || files_match(source_path, &path)? {
                self.remember_path(path.clone(), hash, extension);
                return Ok(path);
            }
        }

        unreachable!("candidate_paths yields an unbounded sequence")
    }

    /// Determine the stem for a Figure context.
    ///
    /// A Figure's own persistent id is always preferred. Anonymous nested
    /// Figures get an alpha suffix from the closest figure parent.
    fn figure_stem(&mut self, figure: &Figure) -> Option<String> {
        if let Some(stem) = figure.id.as_deref().and_then(slugify) {
            return Some(stem);
        }

        if let Some(parent) = self.closest_figure_context_mut()
            && let Some(parent_stem) = &parent.stem
        {
            let stem = format!("{parent_stem}-{}", alpha_suffix(parent.next_subfigure));
            parent.next_subfigure += 1;
            return Some(stem);
        }

        None
    }

    /// Determine the stem for a CodeChunk context.
    ///
    /// A CodeChunk's own persistent id is always preferred. Anonymous
    /// FigureLabel chunks inside Figures receive alpha suffixes.
    fn code_chunk_stem(&mut self, chunk: &CodeChunk) -> Option<String> {
        if let Some(stem) = chunk.id.as_deref().and_then(slugify) {
            return Some(stem);
        }

        if matches!(chunk.label_type, Some(LabelType::FigureLabel))
            && let Some(parent) = self.closest_figure_context_mut()
            && let Some(parent_stem) = &parent.stem
        {
            let stem = format!("{parent_stem}-{}", alpha_suffix(parent.next_subfigure));
            parent.next_subfigure += 1;
            return Some(stem);
        }

        None
    }

    /// Find the nearest Figure context that can allocate subfigure suffixes.
    ///
    /// CodeChunk contexts are skipped intentionally; only Figure parents should
    /// be responsible for `a`, `b`, `c` style child numbering.
    fn closest_figure_context_mut(&mut self) -> Option<&mut NamingContext> {
        self.contexts
            .iter_mut()
            .rev()
            .find(|context| context.is_figure && context.stem.is_some())
    }

    /// Check whether this walk already used a path for different bytes.
    ///
    /// The filesystem may not have enough information to disambiguate files
    /// created earlier in the same walk, so the namer keeps a small in-memory
    /// content-hash map as a first line of collision detection.
    fn path_used_with_different_hash(&self, path: &Path, hash: u64) -> bool {
        self.used.get(path).is_some_and(|used| *used != hash)
    }

    fn remember_path(&mut self, path: PathBuf, hash: u64, extension: &str) {
        self.used.insert(path.clone(), hash);
        self.by_hash
            .entry(ContentKey::new(hash, extension))
            .or_insert(path);
    }

    /// Generate candidate output paths in preference order.
    ///
    /// Readable names are tried first, then readable names with a hash suffix,
    /// then pure hash names. The final numeric fallback keeps the iterator
    /// unbounded for rare collisions between existing non-identical files.
    fn candidate_paths<'a>(
        media_dir: &'a Path,
        desired_stem: Option<&'a str>,
        extension: &'a str,
        hash: u64,
        hash_readable_names: bool,
    ) -> impl Iterator<Item = PathBuf> + 'a {
        let hash = format!("{hash:x}");
        let primary = match desired_stem {
            Some(stem) if hash_readable_names => format!("{stem}-{hash}"),
            Some(stem) => stem.to_string(),
            None => hash.clone(),
        };
        let readable_hash =
            desired_stem.and_then(|stem| (!hash_readable_names).then(|| format!("{stem}-{hash}")));
        let pure_hash = desired_stem.map(|_| hash.clone());

        std::iter::once(primary)
            .chain(readable_hash)
            .chain(pure_hash)
            .chain((1usize..).map(move |index| format!("{hash}-{index}")))
            .map(move |stem| media_dir.join(format!("{stem}.{extension}")))
    }
}

/// Hash bytes using the same lightweight hash used for media filenames.
///
/// The hash is not used for security; it is only a deterministic fallback stem
/// and collision suffix when readable names are unavailable or already taken.
pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = SeaHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Convert an author-facing id into a conservative filename stem.
///
/// Stencila ids are generally URL/HTML friendly already, but exports should be
/// portable across filesystems, so this keeps ASCII alphanumerics and collapses
/// everything else to single hyphens.
fn slugify(value: &str) -> Option<String> {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }

    if previous_dash {
        slug.pop();
    }

    (!slug.is_empty()).then_some(slug)
}

/// Convert a zero-based subfigure index to letters.
///
/// This gives familiar suffixes such as `a`, `b`, and `aa` for nested Figure
/// and FigureLabel CodeChunk contexts.
fn alpha_suffix(mut index: usize) -> String {
    let mut suffix = String::new();
    loop {
        let character = (b'a' + (index % 26) as u8) as char;
        suffix.insert(0, character);
        index /= 26;
        if index == 0 {
            break;
        }
        index -= 1;
    }
    suffix
}

/// Write bytes to a temporary file in the media directory.
///
/// The temporary file is made complete before it is published to the final
/// media path so concurrent readers never compare against partial content.
fn bytes_to_temp_file(media_dir: &Path, bytes: &[u8]) -> Result<NamedTempFile> {
    let mut temp_file = NamedTempFile::new_in(media_dir)?;
    {
        let mut writer = BufWriter::new(temp_file.as_file_mut());
        writer.write_all(bytes)?;
        writer.flush()?;
    }
    Ok(temp_file)
}

/// Copy a source file to a temporary file in the media directory.
fn copy_to_temp_file(source_path: &Path, media_dir: &Path) -> Result<NamedTempFile> {
    let mut temp_file = NamedTempFile::new_in(media_dir)?;
    {
        let mut reader = BufReader::new(File::open(source_path)?);
        let mut writer = BufWriter::new(temp_file.as_file_mut());
        std::io::copy(&mut reader, &mut writer)?;
        writer.flush()?;
    }
    Ok(temp_file)
}

/// Atomically publish a completed temporary file without replacing an existing
/// destination.
fn persist_without_clobbering(temp_file: NamedTempFile, path: &Path) -> Result<bool> {
    match temp_file.persist_noclobber(path) {
        Ok(_) => Ok(true),
        Err(error) if error.error.kind() == ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error.error.into()),
    }
}

/// Atomically replace an existing generated media file.
fn persist_replacing(temp_file: NamedTempFile, path: &Path) -> Result<()> {
    let permissions = path.metadata().ok().map(|metadata| metadata.permissions());
    temp_file.persist(path).map_err(|error| error.error)?;
    if let Some(permissions) = permissions {
        std::fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

/// Check whether a file already contains the provided bytes.
///
/// Existing identical media should be reused instead of copied under another
/// name. The size check avoids streaming obvious mismatches.
fn file_matches_bytes(path: &Path, bytes: &[u8]) -> Result<bool> {
    if path.metadata()?.len() != bytes.len() as u64 {
        return Ok(false);
    }

    let mut reader = BufReader::new(File::open(path)?);
    let mut offset = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(true);
        }
        if buffer[..bytes_read] != bytes[offset..offset + bytes_read] {
            return Ok(false);
        }
        offset += bytes_read;
    }
}

/// Check whether two files have identical bytes.
///
/// Collection may encounter the same source image through several nodes. Reuse
/// keeps output folders smaller while still preserving readable names when
/// contents differ.
fn files_match(left: &Path, right: &Path) -> Result<bool> {
    if left.metadata()?.len() != right.metadata()?.len() {
        return Ok(false);
    }

    let mut left = BufReader::new(File::open(left)?);
    let mut right = BufReader::new(File::open(right)?);
    let mut left_buffer = [0u8; 8192];
    let mut right_buffer = [0u8; 8192];

    loop {
        let left_read = left.read(&mut left_buffer)?;
        let right_read = right.read(&mut right_buffer)?;
        if left_read != right_read {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
        if left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
    }
}

//! Methods associated with calling documents and accessing their parameters

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use common::{
    eyre::{bail, Result},
    indexmap::IndexMap,
    itertools::Itertools,
    serde_json, tracing,
};
use node_address::Address;
use node_pointer::{resolve, resolve_mut};
use node_validate::Validator;
use path_utils::lexiclean::Lexiclean;
use stencila_schema::{EnumValidator, InlineContent, Node, Parameter, ValidatorTypes};

use crate::{document::Document, When};

impl Document {
    /// Get the parameters of the document
    ///
    /// Returns all parameters within the root node of the document as a map, indexed by the
    /// parameter name, containing a tuple of the parameter `id`, [`Address`] as well as the [`Parameter`] itself.
    ///
    /// Used in `Document::call` and when compiling a `Call` node so that the `Call` inherits the parameters of
    /// the document as it's own `arguments`.
    pub async fn params(&mut self) -> Result<IndexMap<String, (String, Address, Parameter)>> {
        // Assemble the document to ensure its `addresses` are up to date
        self.assemble(When::Never, When::Never, When::Never).await?;

        // Collect parameters from addresses
        let addresses = self.addresses.read().await;
        let root = &*self.root.read().await;
        let params = addresses
            .iter()
            .filter_map(|(id, address)| {
                if let Ok(pointer) = resolve(root, Some(address.clone()), Some(id.clone())) {
                    if let Some(InlineContent::Parameter(param)) = pointer.as_inline() {
                        return Some((
                            param.name.clone(),
                            (id.clone(), address.clone(), param.clone()),
                        ));
                    }
                }
                None
            })
            .collect();

        Ok(params)
    }

    /// Call the document with arguments
    ///
    /// This function is similar to `Document::execute`, and indeed calls that function, but first
    /// sets the value of parameters to the `Node` with the same name in `args`.
    ///
    /// # Arguments
    ///
    /// - `args`: A map of the arguments to call the document with
    pub async fn call(&mut self, args: HashMap<String, Node>) -> Result<()> {
        let mut params = self.params().await?;

        {
            let root = &mut *self.root.write().await;
            for (name, value) in args {
                if let Some((id, address, param)) = params.remove(&name) {
                    if let Some(validator) = param.validator.as_deref() {
                        match validator.validate(&value) {
                            Ok(..) => {
                                if let Ok(mut pointer) = resolve_mut(root, Some(address), Some(id))
                                {
                                    if let Some(InlineContent::Parameter(param)) =
                                        pointer.as_inline_mut()
                                    {
                                        param.value = Some(Box::new(value));
                                    }
                                }
                            }
                            Err(error) => bail!(
                                "While attempting to parse document parameter `{}`: {}",
                                name,
                                error
                            ),
                        }
                    }
                } else {
                    bail!("Document does not have a parameter named `{}`", name)
                }
            }
        }

        self.execute(When::Never, None, None, None).await?;

        Ok(())
    }

    /// Call the document with arguments having string values
    ///
    /// This calls the `Document::call` function but first parses each string value into
    /// a `Node` using the corresponding validator.
    ///
    /// Used to call a document with strings taken from the command line.
    ///
    /// # Arguments
    ///
    /// - `args`: A map of the arguments to call the document with
    pub async fn call_with_strings(&mut self, args: HashMap<String, String>) -> Result<()> {
        let mut params = self.params().await?;
        let mut args_parsed = HashMap::new();
        for (name, value) in args {
            if let Some((_id, _address, param)) = params.remove(&name) {
                if let Some(validator) = param.validator.as_deref() {
                    match validator.parse(&value) {
                        Ok(value) => {
                            args_parsed.insert(name, value);
                        }
                        Err(error) => bail!(
                            "While attempting to parse document parameter `{}`: {}",
                            name,
                            error
                        ),
                    }
                }
            } else {
                bail!("Document does not have a parameter named `{}`", name)
            }
        }

        self.call(args_parsed).await?;

        Ok(())
    }

    /// Call the document with arguments parsed from a path
    ///
    /// This calls the `Document::call_with_strings` function but first extracts arguments
    /// from URL path by matching `/<arg>/` segments with `/$<param>/` (or `.$<param>.`)
    /// segments within the document's filesystem path.
    ///
    /// Used by the server and by the site generator to render a document with a parameterized
    /// path. Also exposed on the command line e.g. `stencila run 2020/west` would run either
    /// `./$year/$region.md` or `./$year.$region.docx` files.
    ///
    /// Will return an error if the `path` does not match the path of the document.
    ///
    /// # Arguments
    ///
    /// - `path`: The path to be called e.g. `2020/west` for a document with filesystem path
    ///           of `./$year/$region.md` or `./$year.$region.md`.
    ///
    /// - `dir`: The directory that the `path` is relative to (defaults to current working directory)
    pub async fn call_with_path(&mut self, path: &Path, dir: Option<&Path>) -> Result<()> {
        let dir = dir.map_or_else(
            || std::env::current_dir().expect("Directory not supplied and unable to get CWD"),
            PathBuf::from,
        );
        let path = dir.join(path).lexiclean();

        if !Self::path_matches(&path, &self.path) {
            bail!(
                "Unable to call document at `{}` with path `{}` because paths do not match",
                self.path.to_string_lossy(),
                path.to_string_lossy()
            );
        }

        let mut args = HashMap::new();
        let arg_values = path.components().collect_vec();
        for (index, component) in Self::path_segments(&self.path).iter().enumerate() {
            if let Some(name) = component.strip_prefix('$') {
                args.insert(
                    name.to_string(),
                    arg_values[index].as_os_str().to_string_lossy().to_string(),
                );
            }
        }

        self.call_with_strings(args).await?;

        Ok(())
    }

    /// Get a list of possible URL paths for a document
    ///
    /// When a document has a parameterized path that involves only parameters with `EnumValidator`s,
    /// tis function will return all possible combinations of paths for those parameters. If a document does not
    /// have any parameters, or one of the parameters in the path is not an `EnumValidator`, then
    /// only one path (`self.path`) will be returned.
    ///
    /// # Arguments
    ///
    /// - `dir`: The directory that the URLs should be relative to (defaults to current working directory)
    ///          The document must be withing this directory.
    ///
    /// - `expand`: Whether to expand parametrized paths
    pub async fn enumerate_urls(
        &mut self,
        dir: Option<&Path>,
        expand: bool,
    ) -> Result<Vec<String>> {
        let home = dir
            .map_or_else(
                || std::env::current_dir().expect("Directory not supplied and unable to get CWD"),
                PathBuf::from,
            )
            .canonicalize()?;

        let path = self.path.strip_prefix(&home)?;
        let path_segments = Self::path_segments(path);
        let path_params = Self::path_params(path);

        // Collect the values of all enum parameters
        let params = self.params().await?;
        let param_values: HashMap<String, &Vec<Node>> = path_params.into_iter().filter_map(|name| match params.get(&name) {
            Some((.., param)) => {
                match param.validator.as_deref() {
                    Some(ValidatorTypes::EnumValidator(EnumValidator{values,..})) => {
                        match !values.is_empty() {
                            true => Some((name, values)),
                            false => None
                        }
                    },
                    _ => None
                }
            },
            None => {
                tracing::warn!("Parameter `${}` is used in path for document `{}` but is not represented by a `Parameter` in the content of the document", name, self.path.to_string_lossy());
                None
            }
        }).collect();

        // If there is not a set of values for all of the docs parameters then return the
        // expanded (forward slashes only) URL path
        if !expand || param_values.len() != params.len() {
            return Ok(vec![path_segments.join("/")]);
        }

        // Expand the segments ito URL paths using the enum values
        let mut urls: Vec<String> = vec![String::new()];
        for segment in path_segments {
            // A parameter segment so expand paths for that segment
            if let Some(param) = segment.strip_prefix('$') {
                if let Some(values) = param_values.get(param) {
                    urls = urls
                        .iter()
                        .flat_map(|path| {
                            values.iter().map(|node| {
                                [
                                    path,
                                    "/",
                                    &match node {
                                        Node::Boolean(value) => value.to_string(),
                                        Node::Integer(value) => value.to_string(),
                                        Node::Number(value) => value.0.to_string(),
                                        Node::String(value) => value.to_string(),
                                        _ => serde_json::to_string(node)
                                            .unwrap_or_else(|_| segment.clone()),
                                    },
                                ]
                                .concat()
                            })
                        })
                        .collect();
                    continue;
                }
            }

            // Not a parameter segment (or one that for some reason has no values) so just add to
            // each of the existing paths
            for url in urls.iter_mut() {
                if !url.is_empty() {
                    url.push('/');
                }
                url.push_str(&segment)
            }
        }

        Ok(urls)
    }

    /// Does a filesystem path contain parameters?
    pub fn path_has_parameters(path: &Path) -> bool {
        for component in path.components() {
            let component = component.as_os_str().to_string_lossy();
            if component.starts_with('$') || component.contains(".$") {
                return false;
            }
        }
        false
    }

    /// Split a filesystem path into segments, including segments separated by dots
    /// in directory of file names
    ///
    /// This assumes that the `path` includes a terminating file extension and excludes that
    /// as a segment.
    fn path_segments(path: &Path) -> Vec<String> {
        let mut parts = Vec::new();
        for component in path.components() {
            let string = component.as_os_str().to_string_lossy();
            for part in string.split('.') {
                parts.push(part.to_string());
            }
        }
        parts.pop();
        parts
    }

    /// Get the name of the parameters is a document's filesystem path
    fn path_params(path: &Path) -> Vec<String> {
        Self::path_segments(path)
            .iter()
            .filter_map(|part| part.strip_prefix('$').map(String::from))
            .collect_vec()
    }

    /// Match a URL path to a, possibly parametrized, filesystem path
    ///
    /// Used in `Document::find_path` to find a file system path matching a path.
    pub fn path_matches<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, file: P2) -> bool {
        let path = path.as_ref();
        let file = file.as_ref();

        // Equal as is? (note that Rust removes trailing slashes in paths)
        if path == file {
            return true;
        }

        // Equal to file stem (i.e. if drop extension)
        let file_stem = match file.file_stem() {
            Some(stem) => file
                .components()
                .take(file.components().count() - 1)
                .collect::<PathBuf>()
                .join(stem),
            None => file.to_path_buf(),
        };
        if path == file_stem {
            return true;
        }

        // Split path into segments
        let path_segments = path.components();

        // Split the file into segments (includes dots)
        let file_segments = Self::path_segments(file);

        // Equal number of segments?
        if path_segments.count() != file_segments.len() {
            return false;
        }

        // Do the `path_segments` match the `file_segments` allowing for `$<name>` "wildcards"
        for (index, component) in path.components().enumerate() {
            if file_segments[index].starts_with('$') {
                continue;
            }

            let path_part = component.as_os_str().to_string_lossy();
            if path_part != file_segments[index] {
                return false;
            }
        }

        true
    }

    /// Find a filesystem path that matches a, possibly parametrized, path
    ///
    /// Searches the `dir` for a document with a path (parameterized, or not) that matches
    /// `path`. Returns the first file, in lexical order of path, that matches the path.
    ///
    /// Used when opening documents from the command line or by the server.
    ///
    /// # Arguments
    ///
    /// - `path`: The path possibly containing arguments and lacking an extension e.g. `2020/west`
    ///
    /// - `dir`: The directory that the `path` is relative to (defaults to current working directory)
    pub fn find_matching_path(path: &Path, dir: Option<&Path>) -> Result<PathBuf> {
        if path.exists() {
            return Ok(path.to_path_buf());
        } else if path.is_absolute() {
            bail!("Path does not exist: {}", path.to_string_lossy())
        }

        let dir = dir.map_or_else(
            || std::env::current_dir().expect("Directory not supplied and unable to get CWD"),
            PathBuf::from,
        );

        let walker = ignore::WalkBuilder::new(&dir)
            .sort_by_file_path(Path::cmp)
            .build();
        for entry in walker.filter_map(Result::ok) {
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(&dir).unwrap_or(entry_path);
            if entry_path.is_file() && Self::path_matches(path, rel_path) {
                return Ok(entry_path.to_path_buf());
            }
        }

        bail!(
            "Unable to find a file matching path: {}",
            path.to_string_lossy()
        );
    }
}

#[cfg(test)]
mod tests {
    use common::tokio;

    use crate::document::Document;

    #[tokio::test]
    async fn matches_path() {
        // Match on equal
        assert!(Document::path_matches("file.ext", "file.ext"));
        assert!(Document::path_matches("dir/file.ext", "dir/file.ext"));
        assert!(Document::path_matches("dir/file.ext/", "dir/file.ext"));

        // Match on stem
        assert!(Document::path_matches("file", "file.ext"));
        assert!(Document::path_matches("dir/file", "dir/file.ext"));
        assert!(Document::path_matches("dir/file/", "dir/file.ext"));

        // Match on parameters
        assert!(Document::path_matches("2002", "$year.ext"));
        assert!(Document::path_matches("2002/west", "$year.$region.ext"));
        assert!(Document::path_matches("2002/west", "$year/$region.ext"));
        assert!(Document::path_matches("2002/west/", "$year/$region.ext"));
        assert!(Document::path_matches(
            "2002/west/habitat/pelagic",
            "$year.$region.habitat.$habitat.ext"
        ));
        assert!(Document::path_matches(
            "2002/west/habitat/pelagic",
            "$year/$region/habitat/$habitat.ext"
        ));
        assert!(Document::path_matches(
            "2002/region/west/pelagic",
            "$year/region/$region.$habitat.ext"
        ));
        assert!(Document::path_matches(
            "2002/region/west/pelagic",
            "$year.region.$region/$habitat.ext"
        ));
        assert!(Document::path_matches(
            "2002/region/west/habitat/pelagic/",
            "$year.region.$region.habitat.$habitat.ext"
        ));

        // Non matches
        assert!(!Document::path_matches("a", "b"));
        assert!(!Document::path_matches("a", "b.ext"));
        assert!(!Document::path_matches("a/c", "b/$c"));
        assert!(!Document::path_matches("a/b/c/d", "a/$b/c"));
    }
}

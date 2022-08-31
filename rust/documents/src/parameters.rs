//! Methods associated with document parameters and calling documents

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use common::{
    eyre::{bail, Result},
    indexmap::IndexMap,
    itertools::Itertools,
};
use node_address::Address;
use node_pointer::{resolve, resolve_mut};
use node_validate::Validator;
use path_utils::lexiclean::Lexiclean;
use stencila_schema::{InlineContent, Node, Parameter};

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

        if !Self::is_matching_path(&path, &self.path) {
            bail!(
                "Unable to call document at `{}` with path `{}` because paths do not match",
                self.path.to_string_lossy(),
                path.to_string_lossy()
            );
        }

        let mut args = HashMap::new();
        let arg_values = path.components().collect_vec();
        for (index, component) in Self::path_parts(&self.path).iter().enumerate() {
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

    /// Split a filesystem path into parts, including parts separated by dots
    /// in directory of file names
    fn path_parts(path: &Path) -> Vec<String> {
        let mut parts = Vec::new();
        for component in path.components() {
            let string = component.as_os_str().to_string_lossy();
            for part in string.split('.') {
                parts.push(part.to_string());
            }
        }
        parts
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
            if entry_path.is_file() && Self::is_matching_path(path, rel_path) {
                return Ok(entry_path.to_path_buf());
            }
        }

        bail!(
            "Unable to find a file matching path: {}",
            path.to_string_lossy()
        );
    }

    /// Match a path to a, possibly parametrized, filesystem path
    ///
    /// Used in `Document::find_path` to find a file system path matching a path.
    pub fn is_matching_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, file: P2) -> bool {
        let path = path.as_ref();
        let file = file.as_ref();

        // Equal as is? (note that Rust removes trailing slashes in paths)
        if path == file {
            return true;
        }

        // Equal to file stem (i.e. if drop extension)
        let file = match file.file_stem() {
            Some(stem) => file
                .components()
                .take(file.components().count() - 1)
                .collect::<PathBuf>()
                .join(stem),
            None => file.to_path_buf(),
        };
        if path == file {
            return true;
        }

        // Split path into components
        let path_parts = path.components();

        // Split the file into parts (includes dots)
        let file_parts = Self::path_parts(&file);

        // Equal number of parts?
        if path_parts.count() != file_parts.len() {
            return false;
        }

        // Do the `path_parts` match the `file_parts` allowing for `$<name>` "wildcards"
        for (index, component) in path.components().enumerate() {
            if file_parts[index].starts_with('$') {
                continue;
            }

            let path_part = component.as_os_str().to_string_lossy();
            if path_part != file_parts[index] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use common::tokio;

    use crate::document::Document;

    #[tokio::test]
    async fn matches_path() {
        // Match on equal
        assert!(Document::is_matching_path("file.ext", "file.ext"));
        assert!(Document::is_matching_path("dir/file.ext", "dir/file.ext"));
        assert!(Document::is_matching_path("dir/file.ext/", "dir/file.ext"));

        // Match on stem
        assert!(Document::is_matching_path("file", "file.ext"));
        assert!(Document::is_matching_path("dir/file", "dir/file.ext"));
        assert!(Document::is_matching_path("dir/file/", "dir/file.ext"));

        // Match on parameters
        assert!(Document::is_matching_path("2002", "$year.ext"));
        assert!(Document::is_matching_path("2002/west", "$year.$region.ext"));
        assert!(Document::is_matching_path("2002/west", "$year/$region.ext"));
        assert!(Document::is_matching_path(
            "2002/west/",
            "$year/$region.ext"
        ));
        assert!(Document::is_matching_path(
            "2002/west/habitat/pelagic",
            "$year.$region.habitat.$habitat.ext"
        ));
        assert!(Document::is_matching_path(
            "2002/west/habitat/pelagic",
            "$year/$region/habitat/$habitat.ext"
        ));
        assert!(Document::is_matching_path(
            "2002/region/west/pelagic",
            "$year/region/$region.$habitat.ext"
        ));
        assert!(Document::is_matching_path(
            "2002/region/west/pelagic",
            "$year.region.$region/$habitat.ext"
        ));
        assert!(Document::is_matching_path(
            "2002/region/west/habitat/pelagic/",
            "$year.region.$region.habitat.$habitat.ext"
        ));

        // Non matches
        assert!(!Document::is_matching_path("a", "b"));
        assert!(!Document::is_matching_path("a", "b.ext"));
        assert!(!Document::is_matching_path("a/c", "b/$c"));
        assert!(!Document::is_matching_path("a/b/c/d", "a/$b/c"));
    }
}

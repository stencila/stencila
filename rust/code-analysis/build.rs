use std::{error::Error, fmt::Write, fs, path::Path};

use sha2::{Digest, Sha256};

fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let mut files = vec![root.join("Cargo.toml"), root.join("build.rs")];
    for directory in ["src", "rules", "include"] {
        collect_files(&root.join(directory), &mut files)?;
        println!("cargo:rerun-if-changed={directory}");
    }
    files.sort();
    let mut digest = Sha256::new();
    for path in files {
        let relative = path
            .strip_prefix(&root)?
            .to_string_lossy()
            .replace('\\', "/");
        let contents = fs::read(&path)?;
        // Length prefixes make the names/content framing unambiguous. Only
        // relative names enter the hash, never checkout or registry paths.
        digest.update((relative.len() as u64).to_le_bytes());
        digest.update(relative.as_bytes());
        digest.update((contents.len() as u64).to_le_bytes());
        digest.update(contents);
    }
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");
    let mut hexadecimal = String::with_capacity(64);
    for byte in digest.finalize() {
        write!(hexadecimal, "{byte:02x}")?;
    }
    println!("cargo:rustc-env=STENCILA_CODE_ANALYSIS_DIGEST={hexadecimal}");

    let manifest: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    let dependencies = manifest["dependencies"]
        .as_table()
        .ok_or("missing dependencies")?;
    let mut generated = String::from("const GRAMMARS: &[GrammarVersion] = &[\n");
    let mut grammars = dependencies
        .iter()
        .filter(|(name, _)| name.starts_with("tree-sitter-"))
        .collect::<Vec<_>>();
    grammars.sort_by_key(|(name, _)| *name);
    for (name, specification) in grammars {
        if name == "tree-sitter-language" {
            continue;
        }
        let (version, revision) = if let Some(version) = specification.as_str() {
            (
                version
                    .strip_prefix('=')
                    .ok_or("grammar version must be exact")?,
                None,
            )
        } else {
            // Snakemake's immutable Git revision is pinned in Cargo.toml.
            let revision = specification["rev"]
                .as_str()
                .ok_or("grammar revision must be pinned")?;
            let version = specification["version"]
                .as_str()
                .ok_or("missing Git grammar version")?;
            (
                version
                    .strip_prefix('=')
                    .ok_or("Git grammar version must be exact")?,
                Some(revision),
            )
        };
        let commit = revision.map_or_else(|| "None".to_string(), |rev| format!("Some({rev:?})"));
        generated.push_str(&format!(
            "GrammarVersion {{ package: {name:?}, version: {version:?}, git_commit: {commit} }},\n"
        ));
    }
    generated.push_str("];\n");
    let output = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    fs::write(output.join("identity.rs"), generated)?;
    Ok(())
}

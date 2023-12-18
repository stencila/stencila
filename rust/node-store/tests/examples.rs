//! Tests on examples of Stencila documents

use std::path::PathBuf;

use common::{eyre::Result, glob::glob, itertools::Itertools, tokio};
use common_dev::pretty_assertions::assert_eq;
use node_store::{ReadNode, WriteNode, WriteStore};
use schema::Node;

/// Test writing/reading examples to/from store
///
/// For each `examples/**/*.json` file, read it from JSON as a `Node`, dump
/// it to an Automerge store, read it back from the store, and finally assert it
/// is equal to the original.
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .canonicalize()?
        .join("**/*.json");
    let pattern = pattern.to_str().unwrap_or_default();

    let examples = glob(pattern)?.flatten().collect_vec();

    for path in examples {
        let name = path.file_name().unwrap().to_string_lossy();
        eprintln!("> {name}");

        let node = codecs::from_path(&path, None).await?;

        let mut store = WriteStore::default();
        node.dump(&mut store)?;
        let roundtrip = Node::load(&store)?;

        assert_eq!(roundtrip, node)
    }

    Ok(())
}

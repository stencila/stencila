use eyre::Result;
use graph_triples::Relations;
use node_address::Address;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use stencila_schema::*;

// Re-exports
pub use kernels::{KernelSelector, KernelSpace, TaskResult};

mod executable;
pub use executable::*;

/// A map of node ids to their address
///
/// Used to enable faster access to a node based on it's id.
type Addresses = HashMap<String, Address>;

/// Compile a node
///
/// Compiling a document involves walking over the node tree and compiling each
/// individual node so that it is ready to be built & executed. This includes
/// (but is not limited to):
///
/// - for those node types needing to be accesses directly (e.g. executable nodes) ensuring
///   they have an `id` and recording their address
/// - for executable nodes (e.g. `CodeChunk`) performing semantic analysis of the code
/// - determining dependencies within and between documents and other resources
#[tracing::instrument(skip(node))]
pub fn compile(node: &mut Node, path: &Path, project: &Path) -> Result<(Addresses, Relations)> {
    let mut address = Address::default();
    let mut context = CompileContext {
        path: PathBuf::from(path),
        project: PathBuf::from(project),
        ..Default::default()
    };
    node.compile(&mut address, &mut context)?;

    let addresses = context.addresses;
    let relations = context.relations.into_iter().collect();
    Ok((addresses, relations))
}

/// Execute a node
#[tracing::instrument(skip(node, kernels))]
pub async fn execute<Type>(node: &mut Type, kernels: &mut KernelSpace) -> Result<()>
where
    Type: Executable + Send,
{
    node.execute(kernels).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::CodecTrait;
    use codec_md::MdCodec;
    use std::path::PathBuf;
    use test_snaps::{
        insta::assert_json_snapshot, snapshot_add_suffix, snapshot_fixtures_path_content, fixtures,
    };

    /// Higher level tests of the top level functions in this crate
    #[test]
    fn md_articles() {
        let fixtures = fixtures();
        snapshot_fixtures_path_content("articles/code*.md", |path, content| {
            // Strip the fixtures prefix from the path (so it's the same regardless of machine)
            let stripped_path = path.strip_prefix(&fixtures).unwrap();

            // Load the article
            let mut article = MdCodec::from_str(content, None).unwrap();

            // Compile the article and snapshot the result
            let (addresses, relations) =
                compile(&mut article, stripped_path, &PathBuf::new()).unwrap();
            snapshot_add_suffix("-compile", || {
                assert_json_snapshot!((&addresses, &relations));
            });
        })
    }
}

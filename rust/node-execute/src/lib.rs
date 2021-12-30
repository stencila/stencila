use eyre::Result;
use graph_triples::Relations;
use node_address::{Address, Addresses};
use std::path::Path;
use stencila_schema::*;

// Re-exports
pub use kernels::{KernelSelector, KernelSpace, TaskResult};

mod executable;
pub use executable::*;

mod plan;
pub use plan::Plan;

/// Compile a document
///
/// Compiling a document involves walking over its node tree and compiling each
/// individual child node so that it is ready to be built & executed. This includes
/// (but is not limited to):
///
/// - for those node types needing to be accesses directly (e.g. executable nodes) ensuring
///   they have an `id` and recording their address
/// - for executable nodes (e.g. `CodeChunk`) performing semantic analysis of the code
/// - determining dependencies within and between documents and other resources
#[tracing::instrument(skip(document))]
pub fn compile(document: &mut Node, path: &Path, project: &Path) -> Result<(Addresses, Relations)> {
    let mut address = Address::default();
    let mut context = CompileContext::new(path, project);
    document.compile(&mut address, &mut context)?;
    Ok((context.addresses, context.relations))
}

/// Plan the execution of a document
#[tracing::instrument(skip(document, addresses, relations))]
#[allow(clippy::ptr_arg)]
pub fn plan(
    document: &Node,
    path: &Path,
    addresses: &Addresses,
    relations: &Relations,
) -> Result<Plan> {
    Plan::make(document, path, addresses, relations)
}

/// Execute a document
#[tracing::instrument(skip(document, kernels))]
pub async fn execute<Type>(document: &mut Type, kernels: &mut KernelSpace) -> Result<()>
where
    Type: Executable + Send,
{
    document.execute(kernels).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::CodecTrait;
    use codec_md::MdCodec;
    use std::path::PathBuf;
    use test_snaps::{
        fixtures, insta::assert_json_snapshot, snapshot_add_suffix, snapshot_fixtures_path_content,
    };

    /// Higher level tests of the top level functions in this crate
    #[test]
    fn md_articles() {
        let fixtures = fixtures();
        snapshot_fixtures_path_content("articles/code*.md", |path, content| {
            // Strip the fixtures prefix from the path (so it's the same regardless of machine)
            let path = path.strip_prefix(&fixtures).unwrap();

            // Load the article
            let mut article = MdCodec::from_str(content, None).unwrap();

            // Compile the article and snapshot the result
            let (addresses, relations) = compile(&mut article, path, &PathBuf::new()).unwrap();
            snapshot_add_suffix("-compile", || {
                assert_json_snapshot!((&addresses, &relations));
            });

            // Create an execution plan for the article
            let plan = plan(&article, path, &addresses, &relations).unwrap();
            snapshot_add_suffix("-plan", || {
                assert_json_snapshot!(&plan);
            });
        })
    }
}

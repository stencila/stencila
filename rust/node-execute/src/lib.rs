// Re-exports
pub use kernels::{KernelSelector, KernelSpace, TaskResult};

mod executable;
pub use executable::*;

mod compile;
pub use compile::*;

mod execute;
pub use execute::*;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use codec::CodecTrait;
    use codec_md::MdCodec;
    use eyre::Result;
    use graph::{Graph, PlanOptions, PlanOrdering};
    use kernels::{Kernel, KernelType};
    use node_patch::diff;
    use test_snaps::{
        fixtures,
        insta::{self, assert_json_snapshot},
        snapshot_set_suffix,
    };

    /// Higher level tests of the top level functions in this crate
    #[tokio::test]
    async fn md_articles() -> Result<()> {
        let fixtures = fixtures();

        // So that test results are not dependant upon the the machine the test is run on or how
        // the test is compiled only use built-in kernels
        let kernels: Vec<Kernel> = kernels::available()
            .await?
            .into_iter()
            .filter(|kernel| matches!(kernel.r#type, KernelType::Builtin))
            .collect();

        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.bind_to_thread();

        for name in ["code.md", "code-relations.md"] {
            let path = fixtures.join("articles").join(name);

            // Load the article
            let mut article = MdCodec::from_path(&path, None).await?;

            // Strip the fixtures path so it does not differ between machines
            let path = path.strip_prefix(&fixtures)?;
            let project = path.parent().unwrap();

            // Compile the article and snapshot the result
            let (addresses, resources) = compile(&mut article, path, project)?;
            snapshot_set_suffix(&[name, "-compile"].concat(), || {
                assert_json_snapshot!((&addresses, &resources))
            });

            // Generate and snapshot the article dependency graph
            let graph = Graph::from_resource_infos(path, resources)?;
            snapshot_set_suffix(&[name, "-graph"].concat(), || assert_json_snapshot!(&graph));

            // Generate various execution plans for the article using alternative options
            // and snapshot them all. Always specify `max_concurrency` to avoid differences
            // due to machine (number of CPUs)
            for (suffix, options) in [
                (
                    "appearance",
                    PlanOptions {
                        ordering: PlanOrdering::Appearance,
                        max_concurrency: 10,
                    },
                ),
                (
                    "appearance-concurrency-0",
                    PlanOptions {
                        ordering: PlanOrdering::Appearance,
                        max_concurrency: 0,
                    },
                ),
                (
                    "topological",
                    PlanOptions {
                        ordering: PlanOrdering::Topological,
                        max_concurrency: 10,
                    },
                ),
                (
                    "topological-concurrency-0",
                    PlanOptions {
                        ordering: PlanOrdering::Topological,
                        max_concurrency: 0,
                    },
                ),
            ] {
                let plan = graph
                    .plan(None, Some(kernels.clone()), Some(options))
                    .await?;
                snapshot_set_suffix(&[name, "-", suffix].concat(), || {
                    assert_json_snapshot!(&plan)
                });
            }

            // Execute the article (with default execution plan) and snapshot changes in it
            let pre = article.clone();
            execute(
                &mut article,
                path,
                project,
                Arc::new(KernelSpace::new()),
                None,
            )
            .await?;

            let patch = diff(&pre, &article);
            snapshot_set_suffix(&[name, "-change"].concat(), || {
                assert_json_snapshot!(&patch)
            });
        }

        Ok(())
    }
}

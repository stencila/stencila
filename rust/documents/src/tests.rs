use std::{path::PathBuf, sync::Arc};

use codec::CodecTrait;
use codec_md::MdCodec;
use common::{
    eyre::Result,
    serde_json,
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
};
use graph::{Plan, PlanOptions, PlanOrdering};
use graph_triples::TagMap;
use kernels::{Kernel, KernelSpace, KernelType};
use node_address::Slot;
use node_patch::{Operation, Patch};
use stencila_schema::Node;
use test_snaps::{
    fixtures,
    insta::{self, assert_json_snapshot},
    snapshot_set_suffix,
};

use crate::{
    assemble::assemble,
    compile::compile,
    document::CallDocuments,
    execute::execute,
    messages::{CancelRequest, PatchRequest},
};

/// Higher level tests of the top level functions in this crate
#[tokio::test]
async fn md_articles() -> Result<()> {
    let fixtures = fixtures();

    // So that test results are not dependant upon the the machine the test is run on or how
    // the test is compiled only use built-in kernels
    let kernels: Vec<Kernel> = kernels::available()
        .await
        .into_iter()
        .filter(|kernel| matches!(kernel.r#type, KernelType::Builtin))
        .collect();

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);

    #[allow(deprecated)] // Using bind_to_scope leads to renaming of snapshot files
    settings.bind_to_thread();

    for name in ["code.md", "code-relations.md"] {
        let path = fixtures.join("articles").join(name);

        // Load the article
        let root = Arc::new(RwLock::new(MdCodec::from_path(&path, None).await?));

        // Strip the fixtures path so it does not differ between machines
        let path = path.strip_prefix(&fixtures)?;
        let project = path.parent().unwrap();

        let (patch_request_sender, mut patch_request_receiver) =
            mpsc::unbounded_channel::<PatchRequest>();
        tokio::spawn(async move {
            while let Some(_request) = patch_request_receiver.recv().await {
                // Ignore for this test
            }
        });

        let call_docs = Arc::new(RwLock::new(CallDocuments::default()));
        let tag_map = Arc::new(RwLock::new(TagMap::default()));

        // Assemble the article and snapshot the result
        let address_map = assemble(path, &root, &call_docs, &patch_request_sender).await?;
        snapshot_set_suffix(&[name, "-assemble"].concat(), || {
            assert_json_snapshot!(&address_map)
        });
        let address_map = Arc::new(RwLock::new(address_map));

        // Compile the article and snapshot the result
        let graph = compile(
            path,
            project,
            &root,
            &address_map,
            &tag_map,
            &patch_request_sender,
        )
        .await?;
        snapshot_set_suffix(&[name, "-compile"].concat(), || {
            assert_json_snapshot!(&graph)
        });

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
                .plan(None, Some(kernels.clone()), None, Some(options))
                .await?;
            snapshot_set_suffix(&[name, "-", suffix].concat(), || {
                assert_json_snapshot!(&plan)
            });
        }

        // Execute the article (with topological execution plan) and snapshot the resultant patches
        let plan = graph
            .plan(
                None,
                Some(kernels.clone()),
                None,
                Some(PlanOptions {
                    ordering: PlanOrdering::Topological,
                    max_concurrency: 10,
                }),
            )
            .await?;

        let (patch_request_sender, mut patch_request_receiver) =
            mpsc::unbounded_channel::<PatchRequest>();
        let patches = tokio::spawn(async move {
            let mut patches = Vec::new();
            while let Some(PatchRequest { mut patch, .. }) = patch_request_receiver.recv().await {
                // Redact execute time and duration from patch because they will
                // change across test runs
                for op in patch.ops.iter_mut() {
                    if let Operation::Add { address, value, .. } = op {
                        if let Some(Slot::Name(name)) = address.back() {
                            if name == "execute_ended" || name == "execute_duration" {
                                *value = Box::new("<redacted>".to_string());
                            }
                        }
                    }
                }
                patches.push(patch);
            }
            patches
        });

        let (cancel_request_sender, mut cancel_request_receiver) =
            mpsc::channel::<CancelRequest>(1);

        execute(
            &plan,
            &root,
            &address_map,
            &tag_map,
            &Arc::new(RwLock::new(KernelSpace::new(None, None))),
            &call_docs,
            &patch_request_sender,
            &mut cancel_request_receiver,
        )
        .await?;

        drop(patch_request_sender);
        drop(cancel_request_sender);

        let _patches = patches.await?;
        /*
        Snapshotting of patches turned off for now because order is not-deterministic

        snapshot_set_suffix(&[name, "-patches"].concat(), || {
            assert_json_snapshot!(&patches);
        });
        */
    }

    Ok(())
}

/// Regression test for executing a `CreativeWork`
///
/// Normally we execute a type of creative work such as an `Article`.
/// But sometimes, when you have a flat array of `Node`s, it is necessary
/// to use a `CreativeWork`. The first time we tried to do that it failed.
/// This just tests that there is a patch with an "execute_ended" op
#[tokio::test]
async fn regression_creative_work() -> Result<()> {
    let node = serde_json::from_value(serde_json::json!({
        "type": "CreativeWork",
        "content": [
            {
                "type": "CodeChunk",
                "text": "2 * 2",
                "programmingLanguage": "calc"
            }
        ]
    }))?;

    let (_plan, patches) = assemble_compile_plan_execute(node).await?;
    let was_executed = patches
        .iter()
        .flat_map(|patch| &patch.ops)
        .any(|op| match op {
            Operation::Add { address, .. } => match &address[0] {
                Slot::Name(name) => name == "execute_ended",
                _ => false,
            },
            _ => false,
        });
    assert!(was_executed);

    Ok(())
}

/// Convenience function to compile, plan and execute a node
///
/// Returns the plan and generated patches.
async fn assemble_compile_plan_execute(node: Node) -> Result<(Plan, Vec<Patch>)> {
    let root = Arc::new(RwLock::new(node));
    let call_docs = Arc::new(RwLock::new(CallDocuments::default()));
    let tags = Arc::new(RwLock::new(TagMap::default()));

    let (patch_request_sender, mut patch_request_receiver) =
        mpsc::unbounded_channel::<PatchRequest>();
    let patches = tokio::spawn(async move {
        let mut patches = Vec::new();
        while let Some(request) = patch_request_receiver.recv().await {
            patches.push(request.patch)
        }
        patches
    });

    let (_cancel_request_sender, mut cancel_request_receiver) = mpsc::channel::<CancelRequest>(1);

    let address_map = assemble(&PathBuf::new(), &root, &call_docs, &patch_request_sender).await?;
    let address_map = &Arc::new(RwLock::new(address_map));

    let graph = compile(
        &PathBuf::new(),
        &PathBuf::new(),
        &root,
        address_map,
        &tags,
        &patch_request_sender,
    )
    .await?;

    let plan = graph.plan(None, None, None, None).await?;

    execute(
        &plan,
        &root,
        address_map,
        &tags,
        &Arc::new(RwLock::new(KernelSpace::new(None, None))),
        &call_docs,
        &patch_request_sender,
        &mut cancel_request_receiver,
    )
    .await?;

    drop(patch_request_sender);
    let patches = patches.await?;

    Ok((plan, patches))
}

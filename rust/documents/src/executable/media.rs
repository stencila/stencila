use std::path::Path;

use common::{async_trait::async_trait, eyre::Result, tracing};
use graph_triples::{
    resources::{self},
    Relation, ResourceInfo,
};

use node_address::Address;

use stencila_schema::{
    AudioObject, AudioObjectSimple, ImageObject, ImageObjectSimple, MediaObject, VideoObject,
    VideoObjectSimple,
};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

/// Compile the `content_url` property of `MediaObject` node types
///
/// If the `content_url` property is  a `file://` URL (implicitly
/// or explicitly) then resolves the file path, records it as
/// a file dependency, and returns an absolute `file://` URL.
fn compile_content_url(content_url: &str, context: &mut CompileContext) -> String {
    if content_url.starts_with("http://")
        || content_url.starts_with("https://")
        || content_url.starts_with("data:")
    {
        return content_url.into();
    }

    // Extract the path
    let path = if let Some(path) = content_url.strip_prefix("file://") {
        path
    } else {
        content_url
    };

    // If necessary make the path absolute
    let path = Path::new(path);
    let path = if path.is_relative() {
        match context.path.parent() {
            Some(dir) => dir.join(path),
            None => path.to_path_buf(),
        }
    } else {
        path.to_path_buf()
    };

    // Assert that the path is within the project
    if path.strip_prefix(&context.project).is_err() {
        tracing::warn!(
            "Document contains a link to a file outside of its project: '{}'. Resolved path '{}' is outside of project path '{}'",
            content_url,
            path.display(),
            &context.project.display()
        )
    }

    format!("file://{}", path.display())
}

/// Compile a `MediaObject` node type
///
/// Note that this patches the `content_url` property so that any absolute file path
/// that is resolved in `compile_content_url()` is available, for example
/// for encoding to other formats.
macro_rules! executable_media_object {
    ($type:ty, $prefix:expr) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                register_id!($prefix, self, address, context);
                Ok(())
            }

            async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
                let id = assert_id!(self)?;
                let resource = resources::node(&context.path, &id, stringify!($type));

                let url = compile_content_url(&self.content_url, context);
                let object = if url.starts_with("http") || url.starts_with("data:") {
                    resources::url(&url)
                } else {
                    let url = url.strip_prefix("file://").unwrap_or(&url);
                    resources::file(&Path::new(&url))
                };
                self.content_url = url;

                let relations = vec![(Relation::Embeds, object)];
                let resource_info = ResourceInfo::new(
                    resource,
                    Some(relations),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                context.resource_infos.push(resource_info);

                Ok(())
            }
        }
    };
}

executable_media_object!(MediaObject, "me");
executable_media_object!(AudioObject, "au");
executable_media_object!(AudioObjectSimple, "au");
executable_media_object!(ImageObject, "im");
executable_media_object!(ImageObjectSimple, "im");
executable_media_object!(VideoObject, "vi");
executable_media_object!(VideoObjectSimple, "vi");

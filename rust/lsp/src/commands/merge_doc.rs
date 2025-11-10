use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams},
};
use eyre::Result;
use serde_json::Value;

use stencila_codecs::{DecodeOptions, EncodeOptions};

use super::{internal_error, path_buf_arg};

/// Handle the merge-doc command
///
/// This merges changes from an edited document back into the original.
/// This is a separate function from [`doc_command`] because it does not perform a
/// command on the in-memory document, but rather operates directly on file paths.
pub(crate) async fn merge_doc(
    params: ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();

    let edited_path = path_buf_arg(args.next())?;
    let original_path = path_buf_arg(args.next())?;

    match stencila_codecs::merge(
        &edited_path,
        Some(&original_path),
        None,
        None,
        true,
        DecodeOptions::default(),
        EncodeOptions::default(),
        None,
    )
    .await
    {
        Ok(modified) => {
            let modified = serde_json::to_value(&modified).map_err(internal_error)?;
            Ok(Some(modified))
        }
        Err(error) => {
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to merge document: {error}"),
                })
                .ok();
            Ok(None)
        }
    }
}

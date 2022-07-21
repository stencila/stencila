//! # Pull and push from a node to a Google Doc
//!
//! ## Pulling
//! 
//! Pulling from a Google Doc to a local file is implemented. It simply uses `GoogleDriveProvider::import`
//! to do a complete replacement of the local mirror file.
//! 
//! ## Pushing
//! 
//! It is not possible to do a wholesale update of an entire Google Doc. That is also probably
//! undesirable anyway, particularly if it is currently being edited.
//!
//! Instead, [`Batch updates`](https://developers.google.com/docs/api/reference/rest/v1/documents/batchUpdate)
//! can be sent. However that requires generating a patch (straightforward) and then translating those into
//! Google Doc batchupdates (harder).  I made some initial attempts to implement this using the
//! `node_ranges` captured during decoding but decided it was likely to be very time consuming.

use std::path::Path;

use codec::{
    common::{
        eyre::{bail, Result},
        tokio::fs::read_to_string,
    },
    stencila_schema::{Article, Node},
};
use provider_gdrive::{FileKind, GoogleDriveProvider, ProviderTrait};

use crate::{decode, gdoc, NodeRanges};

/// Pull from a Google Doc to a local file path
///
/// This does not attempt to "merge" changes from the remote Google Doc with any
/// local changes. It assumes that the local `.gdoc` file is just a mirror of the remote
/// and should be overwritten.
pub(crate) async fn pull(path: &Path) -> Result<(Node, NodeRanges)> {
    pull_gdoc(path).await?;
    let json = read_to_string(path).await?;
    decode::decode_async(&json).await
}

async fn read_gdoc(path: &Path) -> Result<gdoc::Document> {
    let json = read_to_string(path).await?;
    let doc: gdoc::Document = serde_json::from_str(&json)?;
    Ok(doc)
}

async fn get_id(path: &Path) -> Result<String> {
    let doc = read_gdoc(path).await?;
    match doc.document_id {
        Some(id) => Ok(id),
        None => bail!("Google Doc file does not have a document id"),
    }
}

async fn pull_gdoc(path: &Path) -> Result<()> {
    let id = get_id(path).await?;

    let url = GoogleDriveProvider::create_url(&FileKind::Doc, &id);

    let node = Node::Article(Article {
        url: Some(Box::new(url)),
        ..Default::default()
    });
    GoogleDriveProvider::import(&node, path, None).await?;

    Ok(())
}

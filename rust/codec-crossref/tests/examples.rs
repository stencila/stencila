use std::{fs::read_to_string, path::PathBuf};

use insta::assert_json_snapshot;

use stencila_codec::{
    eyre::{Context, OptionExt, Result, eyre},
    stencila_schema::Node,
};
use stencila_codec_crossref::{WorkListResponse, WorkResponse};

/// Decode Crossref API responses into nodes
#[test]
fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.json";

    for path in glob::glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".json").map(String::from))
            .ok_or_eyre("should have .json suffix")?;

        let json = read_to_string(path)?;

        let node: Node = if id.starts_with("work-list") {
            let mut response: WorkListResponse =
                serde_json::from_str(&json).wrap_err_with(|| eyre!("In file {id}.json"))?;
            response.message.items.swap_remove(0)
        } else {
            let response: WorkResponse =
                serde_json::from_str(&json).wrap_err_with(|| eyre!("In file {id}.json"))?;
            response.message
        }
        .into();

        assert_json_snapshot!(id, node);
    }

    Ok(())
}

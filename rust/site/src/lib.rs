use std::path::Path;

use eyre::{Result, eyre};
use url::Url;

use stencila_codec_utils::{get_current_branch, slugify_branch_name};

mod headings;
pub use headings::extract_headings_from_node;

mod layout;
pub use layout::{ResolvedLayout, resolve_layout};

mod list;
pub use list::{RouteEntry, RouteType, list};

mod nav;
pub use nav::{NavTreeItem, build_nav_tree};

mod render;
pub use render::{RenderProgress, RenderResult, render};

mod upload;
pub use upload::{UploadProgress, UploadResult, upload};

mod push;
pub use push::{PushProgress, PushResult, push};

mod watch;
pub use watch::{SiteChangeEvent, watch};

/// Convert a canonical Stencila Site URL to a browseable URL
///
/// For main/master branches, returns the canonical URL unchanged.
/// For other branches, prefixes the host with the branch slug (e.g., `feature--siteid.stencila.site`).
///
/// # Arguments
///
/// * `canonical` - The canonical site URL (e.g., `https://siteid.stencila.site/route/`)
/// * `path` - Optional file path to determine the git branch context
///
/// # Examples
///
/// ```
/// // On main branch:
/// // https://siteid.stencila.site/test/ -> https://siteid.stencila.site/test/
///
/// // On feature-foo branch:
/// // https://siteid.stencila.site/test/ -> https://feature-foo--siteid.stencila.site/test/
/// ```
pub fn browseable_url(canonical: &Url, path: Option<&Path>) -> Result<Url> {
    // Extract workspace_id from canonical URL host
    let host = canonical
        .host_str()
        .ok_or_else(|| eyre!("Invalid URL: no host"))?;

    // Get current branch
    let branch_name = get_current_branch(path).unwrap_or_else(|| "main".to_string());
    let branch_slug = slugify_branch_name(&branch_name);

    // For main/master, return canonical URL as-is
    if branch_slug == "main" || branch_slug == "master" {
        return Ok(canonical.clone());
    }

    // For other branches, add branch prefix to host
    let new_host = format!("{branch_slug}--{host}");
    let mut browseable = canonical.clone();
    browseable
        .set_host(Some(&new_host))
        .map_err(|_| eyre!("Failed to set host to {new_host}"))?;

    Ok(browseable)
}

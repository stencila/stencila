use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use archive_utils::extract;
use http_utils::{download, download_temp, url};
use provider::{
    async_trait::async_trait,
    eyre::Result,
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{Node, Thing},
    ImportOptions, ParseItem, Provider, ProviderTrait,
};

pub struct HttpProvider;

#[async_trait]
impl ProviderTrait for HttpProvider {
    fn spec() -> Provider {
        Provider::new("http")
    }

    fn parse(string: &str) -> Vec<ParseItem> {
        static URL_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"https?://[^\s]+").expect("Unable to create regex"));

        URL_REGEX
            .captures_iter(string)
            .into_iter()
            .map(|captures| {
                let capture = captures.get(0).expect("Should be a match group");
                ParseItem {
                    begin: capture.start(),
                    end: capture.end(),
                    node: Node::Thing(Thing {
                        url: Some(Box::new(captures[0].to_string())),
                        ..Default::default()
                    }),
                }
            })
            .collect()
    }

    async fn import(node: &Node, dest: &Path, _options: Option<ImportOptions>) -> Result<bool> {
        let thing = match node {
            Node::Thing(thing) => thing,
            _ => return Ok(false),
        };
        let url = match &thing.url {
            Some(url) => url.as_ref(),
            None => return Ok(false),
        };

        let dest_ext = dest
            .extension()
            .map_or_else(OsString::new, |ext| ext.to_owned());
        let dest_ext = dest_ext.as_os_str();

        let src_url = url::Url::parse(url)?;
        let src_path = PathBuf::from(src_url.path());
        let src_filename = src_path.file_name();
        let src_ext = if src_path.ends_with("tar.gz") {
            "tar.gz".to_string()
        } else if src_path.ends_with("tar.xz") {
            "tar.xz".to_string()
        } else {
            src_path
                .extension()
                .map_or_else(String::new, |ext| ext.to_string_lossy().to_string())
        };

        if !dest_ext.is_empty() {
            // If the destination appears to be a file: just download it
            download(url, dest).await?
        } else if src_ext == "tar"
            || src_ext == "tgz"
            || src_ext == "tar.gz"
            || src_ext == "tar.xz"
            || src_ext == "zip"
        {
            // If the destination appears to be a folder and the source is an archive:
            // extract it into the folder
            let archive = download_temp(url, Some(&src_ext)).await?;
            extract(archive.path(), dest, 0, None)?;
        } else if let Some(filename) = src_filename {
            // If the destination appears to be a folder and the source has a filename:
            // download it into the folder using the filename
            download(url, &dest.join(filename)).await?;
        } else {
            // Otherwise, just download to the destination (event though it appears to be a folder)
            download(url, dest).await?;
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_is;

    #[test]
    fn parse() {
        assert_json_is!(
            HttpProvider::parse("http://example.com/file.csv")[0].node,
            {
                "type": "Thing",
                "url": "http://example.com/file.csv",
            }
        );

        assert_json_is!(
            HttpProvider::parse("https://example.com/anything")[0].node,
            {
                "type": "Thing",
                "url": "https://example.com/anything",
            }
        );

        let parse_items = HttpProvider::parse(
            "
            http://example.com/sub/file.csv som word to be ignored
            and then another url https://example.com/anything/archive.tar.gz
        ",
        );
        assert_eq!(parse_items.len(), 2);
        assert_json_is!(parse_items[0].node, {
            "type": "Thing",
            "url": "http://example.com/sub/file.csv",
        });
        assert_json_is!(parse_items[1].node, {
            "type": "Thing",
            "url": "https://example.com/anything/archive.tar.gz",
        });
    }
}

use std::{env, path::Path};

use google_drive::Client;
use provider::{
    async_trait::async_trait,
    eyre::{self, eyre, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    stencila_schema::{CreativeWork, Node},
    tracing, ImportOptions, ParseItem, Provider, ProviderTrait, SyncOptions,
};
use server_utils::{
    axum::{
        body::Bytes,
        http::{header::HeaderMap, StatusCode},
        response::Headers,
        routing, Router,
    },
    serde_json, serve_gracefully,
};

pub struct GoogleDriveProvider;

const FILE: &str = "file";
const FOLDER: &str = "folder";

/// The base URL for a Google Drive file URL.
///
/// Note that Google Docs, Sheets etc have `docs` URLs like `https://docs.google.com/spreadsheets`
/// when you open them but this is the "canonical" base URL we use
const FILE_URL: &str = "https://drive.google.com/file/d/";

const FOLDER_URL: &str = "https://drive.google.com/drive/folders/";

/// Default port for the webhook server (it's useful to have a fixed port for testing)
const WATCH_SERVER_PORT: u16 = 1665; // python3 -c "print(1024 + sum([ord(c) for c in 'gdrive']))"

impl GoogleDriveProvider {
    /// Create an API client
    fn client(token: Option<String>) -> Client {
        Client::new(
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
        )
    }

    /// Extract the kind and id of a Google Drive resource from the URL of a [`CreativeWork`]
    fn kind_id(node: &CreativeWork) -> Option<(&str, &str)> {
        node.url
            .as_ref()
            .map(|url| match url.strip_prefix(FILE_URL) {
                Some(id) => (FILE, id),
                None => (FOLDER, url.strip_prefix(FOLDER_URL).unwrap_or_default()),
            })
    }
}

#[async_trait]
impl ProviderTrait for GoogleDriveProvider {
    fn spec() -> Provider {
        Provider::new("gdrive")
    }

    fn parse(string: &str) -> Vec<ParseItem> {
        // Regex targeting short identifiers e.g. gdrive:org/name
        static SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"gdrive:(file|folder)/([a-zA-Z0-9-_]+)").expect("Unable to create regex")
        });

        // Regex targeting URL copied from the browser address bar
        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?:https?://)?(?:drive|docs)\.google\.com/(?:drive|file|document|spreadsheets)(?:.*?)/(folders|d)/([a-zA-Z0-9-_]+)(?:/[^\s]*)?",
            )
            .expect("Unable to create regex")
        });

        SIMPLE_REGEX
            .captures_iter(string)
            .into_iter()
            .map(|captures| {
                let capture = captures.get(0).unwrap();
                (
                    capture.start(),
                    capture.end(),
                    captures[1].to_string(),
                    captures[2].to_string(),
                )
            })
            .chain(URL_REGEX.captures_iter(string).into_iter().map(|captures| {
                let capture = captures.get(0).unwrap();
                (
                    capture.start(),
                    capture.end(),
                    captures[1].to_string(),
                    captures[2].to_string(),
                )
            }))
            .map(|(begin, end, kind, id)| ParseItem {
                begin,
                end,
                node: Node::CreativeWork(CreativeWork {
                    url: Some(Box::new(
                        [
                            if kind.starts_with(FOLDER) {
                                FOLDER_URL
                            } else {
                                FILE_URL
                            },
                            &id,
                        ]
                        .concat(),
                    )),
                    ..Default::default()
                }),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_is;

    #[test]
    fn parse() {
        for string in [
            "gdrive:file/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
            "drive.google.com/file/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
            "https://drive.google.com/file/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
            "https://drive.google.com/file/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/view?usp=sharing",

            "docs.google.com/document/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
            "https://docs.google.com/document/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/",
            "https://docs.google.com/document/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/edit",
            "https://docs.google.com/document/u/1/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/edit",

            "docs.google.com/spreadsheets/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
            "https://docs.google.com/spreadsheets/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/",
            "https://docs.google.com/spreadsheets/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/edit",
            "https://docs.google.com/spreadsheets/u/0/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA/edit",
        ] {
            assert_json_is!(
                GoogleDriveProvider::parse(string)[0].node,
                {
                    "type": "CreativeWork",
                    "url": "https://drive.google.com/file/d/1BW6MubIyDirCGW9Wq-tSqCma8pioxBI6VpeLyXn5mZA",
                }
            );
        }

        for string in [
            "gdrive:folder/1OcB7VTWb3lc0u8FJX2LXc5GraKpn-r_m",
            "drive.google.com/drive/folders/1OcB7VTWb3lc0u8FJX2LXc5GraKpn-r_m",
            "https://drive.google.com/drive/folders/1OcB7VTWb3lc0u8FJX2LXc5GraKpn-r_m",
            "https://drive.google.com/drive/u/1/folders/1OcB7VTWb3lc0u8FJX2LXc5GraKpn-r_m",
        ] {
            assert_json_is!(
                GoogleDriveProvider::parse(string)[0].node,
                {
                    "type": "CreativeWork",
                    "url": "https://drive.google.com/drive/folders/1OcB7VTWb3lc0u8FJX2LXc5GraKpn-r_m",
                }
            );
        }

        // Multiple items in a string
        let parse_items = GoogleDriveProvider::parse(
            "
            gdrive:file/17Fw92iZgjD9dEcE8N2m08m-CRa3g-6_Ar24TLumjVV0 som word to be ignored
            and then another url https://docs.google.com/spreadsheets/d/1STkgekwd0Vqo9wj8huU2ps9RaPRvfAWDF7GoR5Vb3GY
        ",
        );
        assert_eq!(parse_items.len(), 2);
        assert_json_is!(parse_items[0].node, {
            "type": "CreativeWork",
            "url": "https://drive.google.com/file/d/17Fw92iZgjD9dEcE8N2m08m-CRa3g-6_Ar24TLumjVV0",
        });
        assert_json_is!(parse_items[1].node, {
            "type": "CreativeWork",
            "url": "https://drive.google.com/file/d/1STkgekwd0Vqo9wj8huU2ps9RaPRvfAWDF7GoR5Vb3GY",
        });
    }
}

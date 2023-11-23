use codec::{common::futures::executor::block_on, EncodeOptions};

mod fixtures;

/// Encode a node to a string using a codec with options
///
/// To minimize the proportion of time spent on spawning async task, constructing the node,
/// getting codec etc, this performs multiple iterations of encoding.
fn to_string(codec: &str, options: Option<EncodeOptions>) {
    let node = fixtures::one_of_each();
    let codec = codecs::get(Some(&String::from(codec)), None, None).expect("Should find codec");

    block_on(async move {
        for _iter in 0..100 {
            codec
                .to_string(&node, options.clone())
                .await
                .expect("Should encode successfully");
        }
    })
}

pub fn main() {
    divan::main();
}

#[divan::bench]
fn html() {
    to_string(
        "html",
        Some(EncodeOptions {
            compact: Some(false),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn html_compact() {
    to_string(
        "html",
        Some(EncodeOptions {
            compact: Some(true),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn jats() {
    to_string(
        "jats",
        Some(EncodeOptions {
            compact: Some(false),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn jats_compact() {
    to_string(
        "jats",
        Some(EncodeOptions {
            compact: Some(true),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn json() {
    to_string(
        "json",
        Some(EncodeOptions {
            compact: Some(false),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn json_compact() {
    to_string(
        "json",
        Some(EncodeOptions {
            compact: Some(true),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn json5() {
    to_string(
        "json5",
        Some(EncodeOptions {
            compact: Some(false),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn json5_compact() {
    to_string(
        "json5",
        Some(EncodeOptions {
            compact: Some(true),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn jsonld() {
    to_string(
        "jsonld",
        Some(EncodeOptions {
            compact: Some(false),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn jsonld_compact() {
    to_string(
        "jsonld",
        Some(EncodeOptions {
            compact: Some(true),
            ..Default::default()
        }),
    )
}

#[divan::bench]
fn markdown() {
    to_string("markdown", None)
}

#[divan::bench]
fn text() {
    to_string("text", None)
}

#[divan::bench]
fn yaml() {
    to_string("yaml", None)
}

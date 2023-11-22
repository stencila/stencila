use codec::{common::futures::executor::block_on, DecodeOptions};

mod fixtures;

/// Decode a node from a string using a codec with options
///
/// To minimize the proportion of time spent on spawning async task, getting codec etc,
/// this performs multiple iterations of encoding.
fn from_str(str: &str, codec: &str, options: Option<DecodeOptions>) {
    let codec = codecs::get(Some(&String::from(codec)), None, None).expect("Should find codec");

    block_on(async move {
        for _iter in 0..100 {
            codec
                .from_str(str, options.clone())
                .await
                .expect("Should decode successfully");
        }
    })
}

pub fn main() {
    divan::main();
}

#[divan::bench]
fn jats() {
    from_str(include_ark_str!("jats.xml"), "jats", None)
}

#[divan::bench]
fn jats_compact() {
    from_str(include_ark_str!("compact.jats.xml"), "jats", None)
}

#[divan::bench]
fn json() {
    from_str(include_ark_str!("json"), "json", None)
}

#[divan::bench]
fn json5() {
    from_str(include_ark_str!("json5"), "json5", None)
}

#[divan::bench]
fn json5_compact() {
    from_str(include_ark_str!("compact.json5"), "json5", None)
}

#[divan::bench]
fn markdown() {
    from_str(include_ark_str!("md"), "markdown", None)
}

#[divan::bench]
fn yaml() {
    from_str(include_ark_str!("yaml"), "yaml", None)
}

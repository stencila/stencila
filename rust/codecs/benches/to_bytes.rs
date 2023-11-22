use codec::{common::futures::executor::block_on, EncodeOptions};
use codecs::Format;

mod fixtures;

/// Encode a node to bytes using a codec with options
///
/// To minimize the proportion of time spent on spawning async task, constructing the node,
/// getting codec etc, this performs multiple iterations of encoding.
fn to_bytes(format: Format, options: Option<EncodeOptions>) {
    let node = fixtures::one_of_each();
    let codec = codecs::get(None, Some(format), None).expect("Should find codec");
    let options = Some(EncodeOptions {
        format: Some(format),
        ..options.unwrap_or_default()
    });

    block_on(async move {
        for _iter in 0..100 {
            codec
                .to_bytes(&node, options.clone())
                .await
                .expect("Should encode successfully");
        }
    })
}

pub fn main() {
    divan::main();
}

#[divan::bench]
fn cbor() {
    to_bytes(Format::Cbor, None)
}

#[divan::bench]
fn cbor_zst() {
    to_bytes(Format::CborZst, None)
}

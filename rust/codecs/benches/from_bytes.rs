use codec::{common::futures::executor::block_on, DecodeOptions};
use codecs::Format;

mod fixtures;

/// Decode a node from bytes using a codec with options
///
/// To minimize the proportion of time spent on spawning async task, getting codec etc,
/// this performs multiple iterations of encoding.
fn from_bytes(bytes: &[u8], format: Format, options: Option<DecodeOptions>) {
    let codec = codecs::get(None, Some(format), None).expect("Should find codec");
    let options = Some(DecodeOptions {
        format: Some(format),
        ..options.unwrap_or_default()
    });

    block_on(async move {
        for _iter in 0..100 {
            codec
                .from_bytes(bytes, options.clone())
                .await
                .expect("Should decode successfully");
        }
    })
}

pub fn main() {
    divan::main();
}

#[divan::bench]
fn cbor() {
    from_bytes(include_ark_bytes!("cbor"), Format::Cbor, None)
}

#[divan::bench]
fn cbor_zst() {
    from_bytes(include_ark_bytes!("cbor.zst"), Format::CborZst, None)
}

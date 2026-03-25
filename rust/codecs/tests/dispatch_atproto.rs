//! Tests for AT Protocol codec registration and dispatch through the codecs crate.
//!
//! These tests verify that the AT Protocol codec is registered in the codecs list
//! and can be discovered via format-based dispatch (Phase 2 / Slice 2).

use stencila_codec::{CodecDirection, eyre::Result, stencila_format::Format};

// ===========================================================================
// AC8: stencila_codecs::get() returns the AtProtoCodec for AtProtoJson encode
// ===========================================================================

/// A codec supporting Format::AtProtoJson for encoding can be found via stencila_codecs::get()
#[test]
fn codec_dispatch_by_format_and_encode_direction() -> Result<()> {
    let format = Format::AtProtoJson;
    let codec = stencila_codecs::get(None, Some(&format), Some(CodecDirection::Encode))?;

    assert_eq!(
        codec.name(),
        "atproto",
        "Codec dispatched for Format::AtProtoJson (Encode) should be named 'atproto'"
    );

    Ok(())
}

/// A codec can be looked up by name "atproto" via stencila_codecs::get()
#[test]
fn codec_dispatch_by_name() -> Result<()> {
    let name = "atproto".to_string();
    let codec = stencila_codecs::get(Some(&name), None, None)?;

    assert_eq!(
        codec.name(),
        "atproto",
        "Codec looked up by name 'atproto' should be named 'atproto'"
    );

    Ok(())
}

/// The AT Protocol codec appears in the codecs list
#[test]
fn codec_in_list() {
    let codecs = stencila_codecs::list();
    let has_atproto = codecs.iter().any(|c| c.name() == "atproto");
    assert!(
        has_atproto,
        "The codecs list should include a codec named 'atproto'"
    );
}

/// The codec_maybe function recognizes "atproto" as a valid codec name
#[test]
fn codec_maybe_recognizes_atproto() {
    let result = stencila_codecs::codec_maybe("atproto");
    assert_eq!(
        result,
        Some("atproto".to_string()),
        "codec_maybe(\"atproto\") should return Some(\"atproto\")"
    );
}

// Note: A test for decode-direction dispatch failure is intentionally deferred.
// In Red phase it would pass trivially (no codec is registered yet), so it does
// not help drive the implementation. It can be added after the codec is registered.

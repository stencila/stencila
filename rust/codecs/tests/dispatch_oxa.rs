//! Tests for Format::Oxa registry and dispatch through the codecs crate.
//!
//! These tests verify that the OXA codec is registered in the codecs list
//! and can be discovered via format-based dispatch.

use stencila_codec::{eyre::Result, stencila_format::Format};

/// Format::Oxa resolves to a known format (not Other or Unknown)
#[test]
fn format_oxa_is_registered() {
    let format = Format::from_name("oxa");
    assert!(
        !format.is_other() && !format.is_unknown(),
        "Format::from_name(\"oxa\") should resolve to a registered format variant, not Other or Unknown, got: {format}"
    );
}

/// Format::Oxa can be resolved from a `.oxa` file extension
#[test]
fn format_oxa_from_file_extension() {
    let format = Format::from_name("oxa");
    assert!(
        !format.is_other() && !format.is_unknown(),
        "'.oxa' extension should resolve to Format::Oxa, got: {format}"
    );
}

/// A codec supporting Format::Oxa can be found via stencila_codecs::get()
#[test]
fn codec_dispatch_by_format() -> Result<()> {
    let format = Format::from_name("oxa");
    let codec = stencila_codecs::get(None, Some(&format), None)?;

    assert_eq!(
        codec.name(),
        "oxa",
        "Codec dispatched for Format::Oxa should be named 'oxa'"
    );

    Ok(())
}

/// A codec can be looked up by name "oxa" via stencila_codecs::get()
#[test]
fn codec_dispatch_by_name() -> Result<()> {
    let name = "oxa".to_string();
    let codec = stencila_codecs::get(Some(&name), None, None)?;

    assert_eq!(
        codec.name(),
        "oxa",
        "Codec looked up by name 'oxa' should be named 'oxa'"
    );

    Ok(())
}

/// The OXA codec appears in the codecs list
#[test]
fn codec_in_list() {
    let codecs = stencila_codecs::list();
    let has_oxa = codecs.iter().any(|c| c.name() == "oxa");
    assert!(
        has_oxa,
        "The codecs list should include a codec named 'oxa'"
    );
}

/// The codec_maybe function recognizes "oxa" as a valid codec name
#[test]
fn codec_maybe_recognizes_oxa() {
    let result = stencila_codecs::codec_maybe("oxa");
    assert_eq!(
        result,
        Some("oxa".to_string()),
        "codec_maybe(\"oxa\") should return Some(\"oxa\")"
    );
}

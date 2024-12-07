use pandoc_types::definition::{self as pandoc};

use codec::schema::*;

use crate::{
    inlines::{inlines_from_pandoc, inlines_to_pandoc},
    shared::{PandocDecodeContext, PandocEncodeContext},
};

pub(super) fn string_to_meta_value(string: &str) -> pandoc::MetaValue {
    pandoc::MetaValue::MetaString(string.into())
}

pub(super) fn string_from_meta_value(meta: pandoc::MetaValue) -> String {
    match meta {
        pandoc::MetaValue::MetaString(string) => string,
        _ => String::new(),
    }
}

pub(super) fn inlines_to_meta_inlines(
    inlines: &[Inline],
    context: &mut PandocEncodeContext,
) -> pandoc::MetaValue {
    pandoc::MetaValue::MetaInlines(inlines_to_pandoc(inlines, context))
}

pub(super) fn inlines_from_meta_inlines(
    meta: pandoc::MetaValue,
    context: &mut PandocDecodeContext,
) -> Vec<Inline> {
    match meta {
        pandoc::MetaValue::MetaInlines(inlines) => inlines_from_pandoc(inlines, context),
        _ => Vec::new(),
    }
}

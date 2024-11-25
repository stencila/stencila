use codec::Losses;
use pandoc_types::definition as pandoc;

/// The context for encoding to Pandoc AST
#[derive(Default)]
pub(super) struct PandocEncodeContext {
    pub losses: Losses,
}

/// The context for decoding from Pandoc AST
#[derive(Default)]
pub(super) struct PandocDecodeContext {
    pub losses: Losses,
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_empty() -> pandoc::Attr {
    pandoc::Attr {
        identifier: String::new(),
        classes: Vec::new(),
        attributes: Vec::new(),
    }
}

/// Get an attribute from a Pandoc `Attr` tuple struct
pub(super) fn get_attr(attrs: &pandoc::Attr, name: &str) -> Option<String> {
    match name {
        "id" => match attrs.identifier.is_empty() {
            true => None,
            false => Some(attrs.identifier.clone()),
        },
        "classes" => match attrs.classes.is_empty() {
            true => None,
            false => Some(attrs.classes.join(" ")),
        },
        _ => attrs.attributes.iter().find_map(|(key, value)| {
            if key == name {
                Some(value.clone())
            } else {
                None
            }
        }),
    }
}

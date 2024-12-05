use codec::Losses;
use pandoc_types::definition as pandoc;

/// The context for encoding to Pandoc AST
#[derive(Default)]
pub(super) struct PandocEncodeContext {
    pub losses: Losses,

    /// Encode paragraphs as Pandoc `Plain` blocks in places
    /// like figure and table captions.
    pub paragraph_to_plain: bool,
}

/// The context for decoding from Pandoc AST
#[derive(Default)]
pub(super) struct PandocDecodeContext {
    pub losses: Losses,
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_empty() -> pandoc::Attr {
    pandoc::Attr::default()
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_classes(classes: Vec<String>) -> pandoc::Attr {
    pandoc::Attr {
        classes,
        ..Default::default()
    }
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_attributes(attributes: Vec<(String, String)>) -> pandoc::Attr {
    pandoc::Attr {
        attributes,
        ..Default::default()
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

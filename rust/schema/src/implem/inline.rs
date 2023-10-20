use node_store::{automerge::ObjId, get_type, ReadNode, ReadStore};

use crate::{prelude::*, *};

impl ReadNode for Inline {
    fn load_null() -> Result<Self> {
        Ok(Inline::Null(Null {}))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(Inline::Boolean(*value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Ok(Inline::Integer(*value))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Ok(Inline::UnsignedInteger(*value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(Inline::Number(*value))
    }

    fn load_map<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        let Some(node_type) = get_type(store, obj_id)? else {
            bail!("Automerge object has no `type` property needed for loading `Inline`");
        };

        let inline = match node_type.as_str() {
            "AudioObject" => Inline::AudioObject(AudioObject::load_map(store, obj_id)?),
            "Button" => Inline::Button(Button::load_map(store, obj_id)?),
            "Cite" => Inline::Cite(Cite::load_map(store, obj_id)?),
            "CiteGroup" => Inline::CiteGroup(CiteGroup::load_map(store, obj_id)?),
            "CodeExpression" => Inline::CodeExpression(CodeExpression::load_map(store, obj_id)?),
            "CodeFragment" => Inline::CodeFragment(CodeFragment::load_map(store, obj_id)?),
            "Date" => Inline::Date(Date::load_map(store, obj_id)?),
            "DateTime" => Inline::DateTime(DateTime::load_map(store, obj_id)?),
            "Delete" => Inline::Delete(Delete::load_map(store, obj_id)?),
            "Duration" => Inline::Duration(Duration::load_map(store, obj_id)?),
            "Emphasis" => Inline::Emphasis(Emphasis::load_map(store, obj_id)?),
            "ImageObject" => Inline::ImageObject(ImageObject::load_map(store, obj_id)?),
            "Insert" => Inline::Insert(Insert::load_map(store, obj_id)?),
            "Link" => Inline::Link(Link::load_map(store, obj_id)?),
            "MathFragment" => Inline::MathFragment(MathFragment::load_map(store, obj_id)?),
            "MediaObject" => Inline::MediaObject(MediaObject::load_map(store, obj_id)?),
            "Note" => Inline::Note(Note::load_map(store, obj_id)?),
            "Parameter" => Inline::Parameter(Parameter::load_map(store, obj_id)?),
            "Quote" => Inline::Quote(Quote::load_map(store, obj_id)?),
            "Span" => Inline::Span(Span::load_map(store, obj_id)?),
            "Strikeout" => Inline::Strikeout(Strikeout::load_map(store, obj_id)?),
            "Strong" => Inline::Strong(Strong::load_map(store, obj_id)?),
            "Subscript" => Inline::Subscript(Subscript::load_map(store, obj_id)?),
            "Superscript" => Inline::Superscript(Superscript::load_map(store, obj_id)?),
            "Text" => Inline::Text(Text::load_map(store, obj_id)?),
            "Time" => Inline::Time(Time::load_map(store, obj_id)?),
            "Timestamp" => Inline::Timestamp(Timestamp::load_map(store, obj_id)?),
            "Underline" => Inline::Underline(Underline::load_map(store, obj_id)?),
            "VideoObject" => Inline::VideoObject(VideoObject::load_map(store, obj_id)?),

            _ => bail!("Unexpected type `{node_type}` in Automerge store for `Inline`"),
        };

        Ok(inline)
    }
}

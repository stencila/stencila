use node_store::{automerge::ObjId, get_type, ReadNode, ReadStore};

use crate::{prelude::*, transforms::blocks_to_inlines, *};

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
            "CodeInline" => Inline::CodeInline(CodeInline::load_map(store, obj_id)?),
            "Date" => Inline::Date(Date::load_map(store, obj_id)?),
            "DateTime" => Inline::DateTime(DateTime::load_map(store, obj_id)?),
            "DeleteInline" => Inline::DeleteInline(DeleteInline::load_map(store, obj_id)?),
            "Duration" => Inline::Duration(Duration::load_map(store, obj_id)?),
            "Emphasis" => Inline::Emphasis(Emphasis::load_map(store, obj_id)?),
            "ImageObject" => Inline::ImageObject(ImageObject::load_map(store, obj_id)?),
            "InsertInline" => Inline::InsertInline(InsertInline::load_map(store, obj_id)?),
            "Link" => Inline::Link(Link::load_map(store, obj_id)?),
            "MathInline" => Inline::MathInline(MathInline::load_map(store, obj_id)?),
            "MediaObject" => Inline::MediaObject(MediaObject::load_map(store, obj_id)?),
            "Note" => Inline::Note(Note::load_map(store, obj_id)?),
            "Parameter" => Inline::Parameter(Parameter::load_map(store, obj_id)?),
            "QuoteInline" => Inline::QuoteInline(QuoteInline::load_map(store, obj_id)?),
            "StyledInline" => Inline::StyledInline(StyledInline::load_map(store, obj_id)?),
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

impl From<Vec<Inline>> for Inline {
    fn from(mut inlines: Vec<Inline>) -> Self {
        if inlines.len() == 1 {
            // Take first inline
            inlines.swap_remove(0)
        } else {
            // Collapse inlines into a single inline text node
            Inline::Text(Text::from(inlines.to_text().0))
        }
    }
}

impl From<Block> for Inline {
    fn from(block: Block) -> Self {
        match block {
            // Blocks with inline analogues
            Block::CodeBlock(code_block) => Inline::CodeInline(CodeInline {
                code: code_block.code,
                programming_language: code_block.programming_language,
                ..Default::default()
            }),
            Block::MathBlock(math_block) => Inline::MathInline(MathInline {
                code: math_block.code,
                math_language: math_block.math_language,
                ..Default::default()
            }),
            Block::QuoteBlock(quote_block) => Inline::QuoteInline(QuoteInline {
                content: blocks_to_inlines(quote_block.content),
                cite: quote_block.cite,
                ..Default::default()
            }),

            // Blocks with inline content
            Block::Heading(heading) => heading.content.into(),
            Block::Paragraph(paragraph) => paragraph.content.into(),

            // Blocks with block content
            Block::Claim(claim) => claim.content.into(),
            Block::IncludeBlock(IncludeBlock {
                source, content, ..
            })
            | Block::CallBlock(CallBlock {
                source, content, ..
            }) => match content {
                Some(content) => content.into(),
                None => Inline::Text(Text::from(source)),
            },

            // Fallback to inline text
            _ => Inline::Text(Text::from(block.to_text().0)),
        }
    }
}

impl From<Vec<Block>> for Inline {
    fn from(mut blocks: Vec<Block>) -> Self {
        if blocks.len() == 1 {
            // Transform first block to inlines
            blocks.swap_remove(0).into()
        } else {
            // Transform blocks to inlines and wrap in an inline span
            Inline::StyledInline(StyledInline {
                content: blocks_to_inlines(blocks),
                ..Default::default()
            })
        }
    }
}

impl From<Block> for Vec<Inline> {
    fn from(block: Block) -> Self {
        match &block {
            // Variants with inline content
            Block::Heading(heading) => heading.content.to_owned(),
            Block::Paragraph(paragraph) => paragraph.content.to_owned(),

            // Variants with block content
            Block::Claim(claim) => blocks_to_inlines(claim.content.to_owned()),
            Block::IncludeBlock(IncludeBlock { content, .. })
            | Block::CallBlock(CallBlock { content, .. }) => match &content {
                Some(content) => blocks_to_inlines(content.to_owned()),
                None => vec![block.into()],
            },

            // Fallback to a single item vector of `block` transformed to an inline
            _ => vec![block.into()],
        }
    }
}

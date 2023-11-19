//! Shortcut functions for conveniently creating nodes of various types
//!
//! Function names are one to three characters long and often, but not always,
//! align to the equivalent HTML tag names.

use crate::types::*;

// Inline nodes (in alphabetic order of node type)

/// Create an [`Inline::AudioObject`] node
pub fn aud<S: Into<String>>(url: S) -> Inline {
    Inline::AudioObject(AudioObject::new(url.into()))
}

/// Create an [`Inline::Button`] node
pub fn btn<C: Into<Cord>, S: Into<String>>(name: S, code: C) -> Inline {
    Inline::Button(Button::new(code.into(), name.into()))
}

/// Create an [`Inline::Cite`] node
pub fn ct<S: Into<String>>(target: S) -> Inline {
    Inline::Cite(Cite::new(target.into(), CitationMode::Parenthetical))
}

/// Create an [`Inline::CiteGroup`] node
pub fn ctg<C, S>(items: C) -> Inline
where
    C: IntoIterator<Item = S>,
    S: Into<String>,
{
    Inline::CiteGroup(CiteGroup::new(
        items
            .into_iter()
            .map(|target| Cite::new(target.into(), CitationMode::Parenthetical))
            .collect(),
    ))
}

/// Create a [`Inline::CodeExpression`] node
pub fn ce<C: Into<Cord>, S: Into<String>>(code: C, lang: Option<S>) -> Inline {
    Inline::CodeExpression(CodeExpression {
        code: code.into(),
        programming_language: lang.map(|lang| lang.into()),
        ..Default::default()
    })
}

/// Create an [`Inline::CodeInline`] node
pub fn ci<C: Into<Cord>>(code: C) -> Inline {
    Inline::CodeInline(CodeInline::new(code.into()))
}

/// Create an [`Inline::Delete`] node
pub fn del<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Delete(Delete::new(content.into()))
}

/// Create an [`Inline::Emphasis`] node
pub fn em<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Emphasis(Emphasis::new(content.into()))
}

/// Create an [`Inline::ImageObject`] node
pub fn img<S: Into<String>>(url: S) -> Inline {
    Inline::ImageObject(ImageObject::new(url.into()))
}

/// Create an [`Inline::Insert`] node
pub fn ins<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Insert(Insert::new(content.into()))
}

/// Create an [`Inline::Link`] node
pub fn lnk<I: Into<Vec<Inline>>, S: Into<String>>(content: I, target: S) -> Inline {
    Inline::Link(Link::new(content.into(), target.into()))
}

/// Create an [`Inline::MathInline`] node
pub fn mi<C: Into<Cord>, S: Into<String>>(code: C, lang: Option<S>) -> Inline {
    Inline::MathInline(MathInline {
        code: code.into(),
        math_language: lang.map(|lang| lang.into()),
        ..Default::default()
    })
}

/// Create an [`Inline::Note`] node
pub fn nte<B: Into<Vec<Block>>>(note_type: NoteType, content: B) -> Inline {
    Inline::Note(Note::new(note_type, content.into()))
}

/// Create an [`Inline::Parameter`] node
pub fn par<S: Into<String>>(name: S) -> Inline {
    Inline::Parameter(Parameter::new(name.into()))
}

/// Create an [`Inline::QuoteInline`] node
pub fn qi<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::QuoteInline(QuoteInline::new(content.into()))
}

/// Create an [`Inline::StyledInline`] node
pub fn sti<C: Into<Cord>, I: Into<Vec<Inline>>>(code: C, content: I) -> Inline {
    Inline::StyledInline(StyledInline::new(code.into(), content.into()))
}

/// Create an [`Inline::Strikeout`] node
pub fn stk<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Strikeout(Strikeout::new(content.into()))
}

/// Create an [`Inline::Strong`] node
pub fn stg<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Strong(Strong::new(content.into()))
}

/// Create an [`Inline::Subscript`] node
pub fn sub<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Subscript(Subscript::new(content.into()))
}

/// Create an [`Inline::Superscript`] node
pub fn sup<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Superscript(Superscript::new(content.into()))
}

/// Create an [`Inline::Text`] node
pub fn t<C: Into<Cord>>(value: C) -> Inline {
    Inline::Text(Text::new(value.into()))
}

/// Create an [`Inline::Underline`] node
pub fn u<I: Into<Vec<Inline>>>(content: I) -> Inline {
    Inline::Underline(Underline::new(content.into()))
}

/// Create an [`Inline::VideoObject`] node
pub fn vid<S: Into<String>>(url: S) -> Inline {
    Inline::VideoObject(VideoObject::new(url.into()))
}

// Block nodes (in alphabetic order of node type)

/// Create a [`Block::Call`] node
pub fn adm<C: Into<Cord>, B: Into<Vec<Block>>>(
    admonition_type: AdmonitionType,
    title: Option<C>,
    content: B,
) -> Block {
    Block::Admonition(Admonition {
        admonition_type,
        title: title.map(|title| vec![t(title)]),
        content: content.into(),
        ..Default::default()
    })
}

/// Create a [`Block::Call`] node
pub fn cal<S: Into<String>, A: Into<Vec<CallArgument>>>(source: S, args: A) -> Block {
    Block::Call(Call::new(source.into(), args.into()))
}

/// Create an [`CallArgument`] node
pub fn arg<S: Into<String>, C: Into<Cord>>(name: S, code: C) -> CallArgument {
    CallArgument {
        name: name.into(),
        code: code.into(),
        ..Default::default()
    }
}

/// Create a [`Block::Claim`] node
pub fn clm<B: Into<Vec<Block>>>(claim_type: ClaimType, content: B) -> Block {
    Block::Claim(Claim::new(claim_type, content.into()))
}

/// Create an [`Block::CodeBlock`] node
pub fn cb<C: Into<Cord>, S: Into<String>>(code: C, lang: Option<S>) -> Block {
    Block::CodeBlock(CodeBlock {
        code: code.into(),
        programming_language: lang.map(|lang| lang.into()),
        ..Default::default()
    })
}

/// Create a [`Block::CodeChunk`] node
pub fn cc<C: Into<Cord>, S: Into<String>>(code: C, lang: Option<S>) -> Block {
    Block::CodeChunk(CodeChunk {
        code: code.into(),
        programming_language: lang.map(|lang| lang.into()),
        ..Default::default()
    })
}

/// Create a [`Block::Division`] node
pub fn div<C: Into<Cord>, B: Into<Vec<Block>>>(code: C, content: B) -> Block {
    Block::Division(Division::new(code.into(), content.into()))
}

/// Create a [`Block::Figure`] node
pub fn fig<B: Into<Vec<Block>>>(content: B) -> Block {
    Block::Figure(Figure::new(content.into()))
}

/// Create a [`Block::For`] node
pub fn r#for<S: Into<String>, C: Into<Cord>, B: Into<Vec<Block>>>(
    symbol: S,
    code: C,
    content: B,
) -> Block {
    Block::For(For::new(code.into(), symbol.into(), content.into()))
}

/// Create a [`Block::Heading`] node with a specified `level`
pub fn h<I: Into<Vec<Inline>>>(level: i64, content: I) -> Block {
    Block::Heading(Heading::new(level, content.into()))
}

/// Create a [`Block::Heading`] node with `level: 1`
pub fn h1<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(1, content.into())
}

/// Create a [`Block::Heading`] node with `level: 2`
pub fn h2<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(2, content.into())
}

/// Create a [`Block::Heading`] node with `level: 3`
pub fn h3<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(3, content.into())
}

/// Create a [`Block::Heading`] node with `level: 4`
pub fn h4<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(4, content.into())
}

/// Create a [`Block::Heading`] node with `level: 5`
pub fn h5<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(5, content.into())
}

/// Create a [`Block::Heading`] node with `level: 6`
pub fn h6<I: Into<Vec<Inline>>>(content: I) -> Block {
    h(6, content.into())
}

/// Create a [`Block::If`] node
pub fn r#if<C: Into<Vec<IfClause>>>(clauses: C) -> Block {
    Block::If(If::new(clauses.into()))
}

/// Create an [`IfClause`] node
pub fn ifc<C: Into<Cord>, S: Into<String>, B: Into<Vec<Block>>>(
    code: C,
    lang: Option<S>,
    content: B,
) -> IfClause {
    IfClause {
        code: code.into(),
        programming_language: lang.map(|lang| lang.into()),
        content: content.into(),
        ..Default::default()
    }
}

/// Create a [`Block::Include`] node
pub fn inc<S: Into<String>>(source: S) -> Block {
    Block::Include(Include::new(source.into()))
}

/// Create a [`Block::List`] node with ascending order
pub fn ol<I: Into<Vec<ListItem>>>(items: I) -> Block {
    Block::List(List::new(items.into(), ListOrder::Ascending))
}

/// Create a [`Block::List`] node with no ordering
pub fn ul<I: Into<Vec<ListItem>>>(items: I) -> Block {
    Block::List(List::new(items.into(), ListOrder::Unordered))
}

/// Create a [`ListItem`] node
pub fn li<I: Into<Vec<Inline>>>(content: I) -> ListItem {
    ListItem {
        content: vec![p(content)],
        ..Default::default()
    }
}

/// Create an [`Block::MathBlock`] node
pub fn mb<C: Into<Cord>, S: Into<String>>(code: C, lang: Option<S>) -> Block {
    Block::MathBlock(MathBlock {
        code: code.into(),
        math_language: lang.map(|lang| lang.into()),
        ..Default::default()
    })
}

/// Create a [`Block::Paragraph`] node
pub fn p<I: Into<Vec<Inline>>>(content: I) -> Block {
    Block::Paragraph(Paragraph::new(content.into()))
}

/// Create a [`Block::QuoteBlock`] node
pub fn qb<B: Into<Vec<Block>>>(content: B) -> Block {
    Block::QuoteBlock(QuoteBlock::new(content.into()))
}

/// Create a [`Block::Section`] node
pub fn sec<B: Into<Vec<Block>>>(content: B) -> Block {
    Block::Section(Section::new(content.into()))
}

/// Create a [`Block::Table`] node
pub fn tab<I: Into<Vec<TableRow>>>(rows: I) -> Block {
    Block::Table(Table {
        rows: rows.into(),
        ..Default::default()
    })
}

/// Create a [`TableRow`] node
pub fn tr<I: Into<Vec<TableCell>>>(cells: I) -> TableRow {
    TableRow {
        cells: cells.into(),
        ..Default::default()
    }
}

/// Create a [`TableCell`] node with [`TableCellType::Header`]
pub fn th<I: Into<Vec<Inline>>>(content: I) -> TableCell {
    TableCell {
        cell_type: Some(TableCellType::HeaderCell),
        content: vec![p(content)],
        ..Default::default()
    }
}

/// Create a [`TableCell`] node with [`TableCellType::Data`]
pub fn td<I: Into<Vec<Inline>>>(content: I) -> TableCell {
    TableCell {
        content: vec![p(content)],
        ..Default::default()
    }
}

/// Create a [`Block::ThematicBreak`] node
pub fn tb() -> Block {
    Block::ThematicBreak(ThematicBreak::new())
}

// Creative works

/// Create a [`Node::Article`] node
pub fn art<I: Into<Vec<Block>>>(content: I) -> Node {
    Node::Article(Article::new(content.into()))
}

//! Inspect sizes of enums and their variants
//!
//! The `sizes` function is not actually a test but rather a way to check that
//! the size of an enum is not being excessively bloated by
//! the largest variants.
//!
//! Where appropriate properties of large variants should be made
//! non-core so that they are heap allocated, rather than stack
//! allocated (which means that they do no inflate the size of the enum).
//!
//! Run like this:
//!
//!   cargo test -p schema --test sizes -- --nocapture

use schema::*;

macro_rules! sizes {
    ($($t:ty),*) => {
        {
            let mut sizes: Vec<(&str, usize)> = vec![
                $((stringify!($t), std::mem::size_of::<$t>()),)*
            ];
            sizes.sort_by(|a, b| b.1.cmp(&a.1));

            for (name, size) in sizes {
                eprintln!("{:<20} {:>6}", name, size);
            }
        }
    };
}

#[test]
#[allow(clippy::print_stderr)]
fn sizes() {
    sizes!(
        CreativeWorkType,
        Article,
        AudioObject,
        Chat,
        Claim,
        Collection,
        Comment,
        Datatable,
        Figure,
        ImageObject,
        MediaObject,
        Periodical,
        Prompt,
        PublicationIssue,
        PublicationVolume,
        Review,
        SoftwareApplication,
        SoftwareSourceCode,
        Table,
        VideoObject
    );

    eprintln!("\n");

    sizes!(
        Block,
        Admonition,
        CallBlock,
        Chat,
        ChatMessage,
        ChatMessageGroup,
        Claim,
        CodeBlock,
        CodeChunk,
        Excerpt,
        Figure,
        ForBlock,
        Form,
        Heading,
        IfBlock,
        IncludeBlock,
        InstructionBlock,
        List,
        MathBlock,
        Paragraph,
        PromptBlock,
        QuoteBlock,
        RawBlock,
        Section,
        StyledBlock,
        SuggestionBlock,
        Table,
        ThematicBreak,
        Walkthrough
    );

    eprintln!("\n");

    sizes!(
        Inline,
        AudioObject,
        Button,
        Cite,
        CiteGroup,
        CodeExpression,
        CodeInline,
        Date,
        DateTime,
        Duration,
        Emphasis,
        ImageObject,
        InstructionInline,
        Link,
        MathInline,
        MediaObject,
        Note,
        Parameter,
        QuoteInline,
        StyledInline,
        Strikeout,
        Strong,
        Subscript,
        Superscript,
        Text,
        Time,
        Timestamp,
        Underline,
        VideoObject,
        Null,
        Boolean,
        Integer,
        UnsignedInteger,
        Number
    );
}

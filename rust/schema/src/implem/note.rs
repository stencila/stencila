use stencila_codec_info::lost_options;

use crate::{Inline, Note, NoteType, prelude::*};

impl LatexCodec for Note {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        let command = match self.note_type {
            NoteType::Footnote => "footnote",
            NoteType::Endnote => "endnote",
            NoteType::Sidenote => "sidenote",
        };

        context
            .enter_node(self.node_type(), self.node_id())
            .command_begin(command)
            .property_fn(NodeProperty::Content, |context| {
                // Convert the block content of the note to inlines. If this is not done, all sorts
                // of broken LaTeX is generated, in particular paragraphs in the notes will cause
                // the context's paragraph content (used for line wrapping) to be reset.
                // Usually there will be just one para, but this puts spaces between blocks
                for (index, block) in self.content.iter().enumerate() {
                    if index > 0 {
                        context.char(' ');
                    }
                    Vec::<Inline>::from(block.clone()).to_latex(context)
                }
            })
            .command_end()
            .exit_node();
    }
}

impl MarkdownCodec for Note {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.merge_losses(lost_options!(self, id));

        if self.note_type != NoteType::Footnote {
            context.add_loss("Note.noteType");
        }

        let index = context.footnotes.len() + 1;

        let mut footnote_context = MarkdownEncodeContext::default();
        footnote_context.enter_node(NodeType::Note, self.node_id());
        footnote_context.push_str("[^");
        footnote_context.push_str(&index.to_string());
        footnote_context.push_str("]: ");
        footnote_context.push_line_prefix("    ");
        self.content.to_markdown(&mut footnote_context);
        footnote_context.exit_node();
        context.footnotes.push(footnote_context);

        context.push_str("[^");
        context.push_str(&index.to_string());
        context.push_str("]");
    }
}

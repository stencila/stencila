use codec_losses::lost_options;

use crate::{prelude::*, Note, NoteType};

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
        footnote_context.push_line_prefix("  ");
        self.content.to_markdown(&mut footnote_context);
        footnote_context.exit_node();
        context.footnotes.push(footnote_context);

        context.push_str("[^");
        context.push_str(&index.to_string());
        context.push_str("]");
    }
}

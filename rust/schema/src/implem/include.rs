use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, Include};

impl Include {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let md = ["/", &self.source].concat();

        let mut losses = lost_options!(self, id, media_type, select, content);
        losses.merge(lost_exec_options!(self));

        (md, losses)
    }
}

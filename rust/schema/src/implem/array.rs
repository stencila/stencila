use codec_html_trait::encode::elem;
use node_store::{
    automerge::{ObjId, Prop},
    ReadNode, ReadStore, WriteNode, WriteStore,
};

use crate::{prelude::*, Array, Primitive};

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Array")
    }
}

impl StripNode for Array {}

impl CondenseNode for Array {}

impl ReadNode for Array {
    fn load_list<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_list(store, obj_id)?))
    }

    fn load_none() -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_none()?))
    }
}

impl WriteNode for Array {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.0.insert_prop(store, obj_id, prop)
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        self.0.put_prop(store, obj_id, prop)
    }

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        self.0.similarity(store, obj_id, prop)
    }
}

impl DomCodec for Array {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_elem("stencila-array");

        for (index, value) in self.iter().enumerate() {
            context
                .enter_elem("stencila-array-item")
                .push_attr("index", &index.to_string());
            value.to_dom(context);
            context.exit_elem();
        }

        context.exit_elem();
    }
}

impl HtmlCodec for Array {
    fn to_html_parts(&self, context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        // Uses spans, rather than say <ol>/<li> because needs to be
        // include e.g for output of a `CodeExpression`
        (
            "stencila-array",
            vec![],
            self.iter()
                .map(|value| elem("stencila-array-item", &[], &[value.to_html(context)]))
                .collect_vec(),
        )
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl MarkdownCodec for Array {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let (text, losses) = self.to_text();
        context.push_str(&text);
        context.merge_losses(losses);
    }
}

impl TextCodec for Array {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::one("Array#");

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.merge(item_losses);
        }

        if !text.is_empty() {
            text.push(' ');
        }

        (text, losses)
    }
}

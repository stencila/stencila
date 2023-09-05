use codec_html_trait::encode::{attr, elem};
use node_store::{
    automerge::{ObjId, Prop},
    Read, ReadStore, Write, WriteStore,
};

use crate::{prelude::*, Array, Primitive};

impl StripNode for Array {}

impl Read for Array {
    fn load_list<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_list(store, obj_id)?))
    }

    fn load_none() -> Result<Self> {
        Ok(Self(Vec::<Primitive>::load_none()?))
    }
}

impl Write for Array {
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

impl HtmlCodec for Array {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        // Uses spans, rather than say <ol>/<li> because needs to be
        // include e.g for output of a `CodeExpression`
        (
            "span",
            vec![attr("is", "stencila-array")],
            self.iter()
                .map(|value| {
                    elem(
                        "span",
                        &[attr("is", "stencila-array-item")],
                        &[value.to_html()],
                    )
                })
                .collect_vec(),
        )
    }

    fn to_html_attr(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl MarkdownCodec for Array {
    fn to_markdown(&self) -> (String, Losses) {
        let mut markdown = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                markdown.push(' ');
            }

            let (item_markdown, mut item_losses) = item.to_markdown();
            markdown.push_str(&item_markdown);
            losses.add_all(&mut item_losses);
        }

        (markdown, losses)
    }
}

impl TextCodec for Array {
    fn to_text(&self) -> (String, Losses) {
        let mut text = String::new();
        let mut losses = Losses::none();

        for (index, item) in self.iter().enumerate() {
            if index != 0 {
                text.push(' ');
            }

            let (item_text, mut item_losses) = item.to_text();
            text.push_str(&item_text);
            losses.add_all(&mut item_losses);
        }

        (text, losses)
    }
}

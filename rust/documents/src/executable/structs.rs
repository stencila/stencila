use super::prelude::*;

/// Implementation of `Executable` for various fields of a struct
macro_rules! executable_fields {
    ($type:ty $(, $field:ident)* ) => {
        #[async_trait]
        impl Executable for $type {
            async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
                $(
                    address.push_back(Slot::Name(stringify!($field).to_string()));
                    self.$field.compile(address, context).await?;
                    address.pop_back();
                )*
                Ok(())
            }
        }
    };
}

executable_fields!(CiteGroup, items);

executable_fields!(Collection, parts);
executable_fields!(Directory, parts);

executable_fields!(List, items);
executable_fields!(ListItem, item, content);

executable_fields!(Table, rows, caption);
executable_fields!(TableSimple, rows, caption);
executable_fields!(TableRow, cells);
executable_fields!(TableCell, content);

/// Implementation of `Executable` for only the `content` field of a struct
macro_rules! executable_content {
    ($type:ty) => {
        executable_fields!($type, content);
    };
    ( $( $type:ty ),* ) => {
        $(
            executable_content!($type);
        )*
    };
}

executable_content!(
    Article,
    Cite,
    Claim,
    ClaimSimple,
    Comment,
    CreativeWork,
    Delete,
    Emphasis,
    Figure,
    FigureSimple,
    Heading,
    NontextualAnnotation,
    Note,
    Paragraph,
    Quote,
    QuoteBlock,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Underline
);

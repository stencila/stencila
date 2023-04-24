use smol_str::SmolStr;

use common::eyre::Result;
use node_store::{automerge::ObjId, Read, ReadStore};

use crate::{Array, Node, Null, Object};

impl Read for Node {
    fn load_null() -> Result<Self> {
        Ok(Node::Null(Null {}))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(Node::Boolean(*value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Ok(Node::Integer(*value))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Ok(Node::UnsignedInteger(*value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(Node::Number(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(Node::String(value.to_string()))
    }

    fn load_list<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        Ok(Node::Array(Array::load_list(store, obj_id)?))
    }

    fn load_map<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        let r#type = node_store::get_type::<Self, _>(store, obj_id).unwrap_or_default();

        macro_rules! load_map_variants {
            ($( $variant:ident ),*) => {
                match r#type.as_str() {
                    $(
                        stringify!($variant) => Ok(Node::$variant(crate::$variant::load_map(store, obj_id)?)),
                    )*

                    // There is no type, or it does not match any known type, so load as an `Object`
                    _ => Ok(Node::Object(Object::load_map(store, obj_id)?)),
                }
            };
        }

        load_map_variants!(
            ArrayValidator,
            Article,
            AudioObject,
            BooleanValidator,
            Brand,
            Button,
            Call,
            CallArgument,
            Cite,
            CiteGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeError,
            CodeExpression,
            CodeFragment,
            Collection,
            Comment,
            ConstantValidator,
            ContactPoint,
            CreativeWork,
            Datatable,
            DatatableColumn,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            Directory,
            Division,
            Duration,
            DurationValidator,
            Emphasis,
            EnumValidator,
            Enumeration,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionDigest,
            ExecutionTag,
            Figure,
            File,
            For,
            Form,
            Function,
            Grant,
            Heading,
            If,
            IfClause,
            ImageObject,
            Include,
            IntegerValidator,
            Link,
            List,
            ListItem,
            MathBlock,
            MathFragment,
            MediaObject,
            MonetaryGrant,
            Note,
            NumberValidator,
            Organization,
            Paragraph,
            Parameter,
            Periodical,
            Person,
            PostalAddress,
            Product,
            PropertyValue,
            PublicationIssue,
            PublicationVolume,
            Quote,
            QuoteBlock,
            Review,
            SoftwareApplication,
            SoftwareSourceCode,
            Span,
            Strikeout,
            StringValidator,
            Strong,
            Subscript,
            Superscript,
            Table,
            TableCell,
            TableRow,
            Text,
            ThematicBreak,
            Thing,
            Time,
            TimeValidator,
            Timestamp,
            TimestampValidator,
            TupleValidator,
            Underline,
            Variable,
            VideoObject
        )
    }
}

use node_store::{automerge::ObjId, get_node_type, ReadNode, ReadStore};

use crate::{prelude::*, Array, Block, Inline, Node, Null, Object, Primitive};

impl ReadNode for Node {
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
        let node_type = get_node_type(store, obj_id)?;

        let Some(node_type) = node_type else {
            // There is no type, or it does not match any known type, so load as an `Object`
            return Ok(Node::Object(Object::load_map(store, obj_id)?));
        };

        macro_rules! load_map_variants {
            ($( $variant:ident ),*) => {
                match node_type {
                    $(
                        NodeType::$variant => Ok(Node::$variant(crate::$variant::load_map(store, obj_id)?)),
                    )*

                    // It is not expected to have a map with type: "Object", but if there is,
                    // then treat it as such
                    NodeType::Object => Ok(Node::Object(Object::load_map(store, obj_id)?)),

                    NodeType::Null |
                    NodeType::Boolean |
                    NodeType::Integer |
                    NodeType::UnsignedInteger |
                    NodeType::Number |
                    NodeType::String |
                    NodeType::Array => bail!("Node::load_map unexpectedly called for {node_type}")
                }
            };
        }

        load_map_variants!(
            Admonition,
            ArrayValidator,
            Article,
            AudioObject,
            AuthorRole,
            BooleanValidator,
            Brand,
            Button,
            CallBlock,
            CallArgument,
            Cite,
            CiteGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeExpression,
            CodeInline,
            CodeLocation,
            Collection,
            Comment,
            CompilationDigest,
            CompilationError,
            ConstantValidator,
            ContactPoint,
            Cord,
            CreativeWork,
            Datatable,
            DatatableColumn,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            DeleteBlock,
            DeleteInline,
            Directory,
            Duration,
            DurationValidator,
            Emphasis,
            EnumValidator,
            Enumeration,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionError,
            ExecutionTag,
            Figure,
            File,
            ForBlock,
            Form,
            Function,
            Grant,
            Heading,
            IfBlock,
            IfBlockClause,
            ImageObject,
            IncludeBlock,
            InsertBlock,
            InsertInline,
            InstructionBlock,
            InstructionInline,
            IntegerValidator,
            Link,
            List,
            ListItem,
            MathBlock,
            MathInline,
            MediaObject,
            Message,
            ModifyBlock,
            ModifyInline,
            ModifyOperation,
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
            QuoteBlock,
            QuoteInline,
            ReplaceBlock,
            ReplaceInline,
            Review,
            Section,
            SoftwareApplication,
            SoftwareSourceCode,
            Strikeout,
            StringOperation,
            StringPatch,
            StringValidator,
            Strong,
            StyledBlock,
            StyledInline,
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

impl From<Primitive> for Node {
    fn from(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Null(node) => Node::Null(node),
            Primitive::Boolean(node) => Node::Boolean(node),
            Primitive::Integer(node) => Node::Integer(node),
            Primitive::UnsignedInteger(node) => Node::UnsignedInteger(node),
            Primitive::Number(node) => Node::Number(node),
            Primitive::String(node) => Node::String(node),
            Primitive::Array(node) => Node::Array(node),
            Primitive::Object(node) => Node::Object(node),
        }
    }
}

impl From<Inline> for Node {
    fn from(inline: Inline) -> Self {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match inline {
                    $(
                        Inline::$variant(node) => Node::$variant(node),
                    )*
                }
            };
        }

        variants!(
            AudioObject,
            Boolean,
            Button,
            Cite,
            CiteGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            DeleteInline,
            Duration,
            Emphasis,
            ImageObject,
            InsertInline,
            InstructionInline,
            Integer,
            Link,
            MathInline,
            MediaObject,
            ModifyInline,
            Note,
            Null,
            Number,
            Parameter,
            QuoteInline,
            ReplaceInline,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            UnsignedInteger,
            VideoObject
        )
    }
}

impl From<Block> for Node {
    fn from(block: Block) -> Self {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match block {
                    $(
                        Block::$variant(node) => Node::$variant(node),
                    )*
                }
            };
        }

        variants!(
            Admonition,
            CallBlock,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            QuoteBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            Table,
            ThematicBreak
        )
    }
}

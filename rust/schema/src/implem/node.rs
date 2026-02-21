use stencila_node_store::{ReadNode, ReadStore, automerge::ObjId, get_node_type};

use crate::{Array, Block, CreativeWorkVariant, Inline, Node, Null, Object, Primitive, prelude::*};

impl Node {
    pub fn node_type(&self) -> NodeType {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Node::$variant(..) => NodeType::$variant,)*
                }
            };
        }

        variants!(
            Admonition,
            Agent,
            Annotation,
            AppendixBreak,
            Array,
            ArrayHint,
            ArrayValidator,
            Article,
            AudioObject,
            AuthorRole,
            Bibliography,
            Boolean,
            BooleanValidator,
            Brand,
            Button,
            CallArgument,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Citation,
            CitationGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeExpression,
            CodeInline,
            CodeLocation,
            Collection,
            Comment,
            CompilationDigest,
            CompilationMessage,
            ConstantValidator,
            ContactPoint,
            Cord,
            CreativeWork,
            Datatable,
            DatatableColumn,
            DatatableColumnHint,
            DatatableHint,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            Directory,
            Duration,
            DurationValidator,
            Emphasis,
            Enumeration,
            EnumValidator,
            Excerpt,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionMessage,
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
            InlinesBlock,
            InstructionBlock,
            InstructionInline,
            InstructionMessage,
            Integer,
            IntegerValidator,
            Island,
            Link,
            List,
            ListItem,
            MathBlock,
            MathInline,
            MediaObject,
            ModelParameters,
            MonetaryGrant,
            Note,
            Null,
            Number,
            NumberValidator,
            Object,
            ObjectHint,
            Organization,
            Page,
            Paragraph,
            Parameter,
            Periodical,
            Person,
            PostalAddress,
            Product,
            Prompt,
            PromptBlock,
            PropertyValue,
            ProvenanceCount,
            PublicationIssue,
            PublicationVolume,
            QuoteBlock,
            QuoteInline,
            RawBlock,
            Reference,
            Review,
            Section,
            Sentence,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Strikeout,
            String,
            StringHint,
            StringValidator,
            Strong,
            StyledBlock,
            StyledInline,
            Subscript,
            SuggestionBlock,
            SuggestionInline,
            Superscript,
            Supplement,
            Table,
            TableCell,
            TableRow,
            Text,
            ThematicBreak,
            Thing,
            Time,
            Timestamp,
            TimestampValidator,
            TimeValidator,
            TupleValidator,
            Underline,
            Unknown,
            UnsignedInteger,
            Variable,
            VideoObject,
            Walkthrough,
            WalkthroughStep,
            Workflow
        )
    }

    pub fn node_id(&self) -> Option<NodeId> {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Node::$variant(node) => Some(node.node_id()),)*

                    Node::Null(..) |
                    Node::Boolean(..) |
                    Node::Integer(..) |
                    Node::UnsignedInteger(..) |
                    Node::Number(..) |
                    Node::String(..) |
                    Node::Cord(..) |
                    Node::Array(..) |
                    Node::Object(..) => None,
                }
            };
        }

        variants!(
            Admonition,
            Agent,
            Annotation,
            AppendixBreak,
            ArrayHint,
            ArrayValidator,
            Article,
            AudioObject,
            AuthorRole,
            Bibliography,
            BooleanValidator,
            Brand,
            Button,
            CallArgument,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Citation,
            CitationGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeExpression,
            CodeInline,
            CodeLocation,
            Collection,
            Comment,
            CompilationDigest,
            CompilationMessage,
            ConstantValidator,
            ContactPoint,
            CreativeWork,
            Datatable,
            DatatableColumn,
            DatatableColumnHint,
            DatatableHint,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            Directory,
            Duration,
            DurationValidator,
            Emphasis,
            Enumeration,
            EnumValidator,
            Excerpt,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionMessage,
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
            InlinesBlock,
            InstructionBlock,
            InstructionInline,
            InstructionMessage,
            IntegerValidator,
            Island,
            Link,
            List,
            ListItem,
            MathBlock,
            MathInline,
            MediaObject,
            ModelParameters,
            MonetaryGrant,
            Note,
            NumberValidator,
            ObjectHint,
            Organization,
            Page,
            Paragraph,
            Parameter,
            Periodical,
            Person,
            PostalAddress,
            Product,
            Prompt,
            PromptBlock,
            PropertyValue,
            ProvenanceCount,
            PublicationIssue,
            PublicationVolume,
            QuoteBlock,
            QuoteInline,
            RawBlock,
            Reference,
            Review,
            Section,
            Sentence,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Strikeout,
            StringHint,
            StringValidator,
            Strong,
            StyledBlock,
            StyledInline,
            Subscript,
            SuggestionBlock,
            SuggestionInline,
            Superscript,
            Supplement,
            Table,
            TableCell,
            TableRow,
            Text,
            ThematicBreak,
            Thing,
            Time,
            Timestamp,
            TimestampValidator,
            TimeValidator,
            TupleValidator,
            Underline,
            Unknown,
            Variable,
            VideoObject,
            Walkthrough,
            WalkthroughStep,
            Workflow
        )
    }
}

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

                    NodeType::Config => bail!("Config options not yet supported"),

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
            Agent,
            Annotation,
            AppendixBreak,
            ArrayHint,
            ArrayValidator,
            Article,
            AudioObject,
            AuthorRole,
            Bibliography,
            BooleanValidator,
            Brand,
            Button,
            CallArgument,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Citation,
            CitationGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeExpression,
            CodeInline,
            CodeLocation,
            Collection,
            Comment,
            CompilationDigest,
            CompilationMessage,
            ConstantValidator,
            ContactPoint,
            Cord,
            CreativeWork,
            Datatable,
            DatatableColumn,
            DatatableColumnHint,
            DatatableHint,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            Directory,
            Duration,
            DurationValidator,
            Emphasis,
            Enumeration,
            EnumValidator,
            Excerpt,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionMessage,
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
            InlinesBlock,
            InstructionBlock,
            InstructionInline,
            InstructionMessage,
            IntegerValidator,
            Island,
            Link,
            List,
            ListItem,
            MathBlock,
            MathInline,
            MediaObject,
            ModelParameters,
            MonetaryGrant,
            Note,
            NumberValidator,
            ObjectHint,
            Organization,
            Page,
            Paragraph,
            Parameter,
            Periodical,
            Person,
            PostalAddress,
            Product,
            Prompt,
            PromptBlock,
            PropertyValue,
            ProvenanceCount,
            PublicationIssue,
            PublicationVolume,
            QuoteBlock,
            QuoteInline,
            RawBlock,
            Reference,
            Review,
            Section,
            Sentence,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Strikeout,
            StringHint,
            StringValidator,
            Strong,
            StyledBlock,
            StyledInline,
            Subscript,
            SuggestionBlock,
            SuggestionInline,
            Superscript,
            Supplement,
            Table,
            TableCell,
            TableRow,
            Text,
            ThematicBreak,
            Thing,
            Time,
            Timestamp,
            TimestampValidator,
            TimeValidator,
            TupleValidator,
            Underline,
            Unknown,
            Variable,
            VideoObject,
            Walkthrough,
            WalkthroughStep,
            Workflow
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
            Annotation,
            AudioObject,
            Boolean,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Integer,
            Link,
            MathInline,
            MediaObject,
            Note,
            Null,
            Number,
            Parameter,
            QuoteInline,
            Sentence,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            SuggestionInline,
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
            AppendixBreak,
            AudioObject,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Datatable,
            Excerpt,
            Figure,
            File,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            ImageObject,
            IncludeBlock,
            InlinesBlock,
            InstructionBlock,
            Island,
            List,
            MathBlock,
            Page,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Supplement,
            Table,
            ThematicBreak,
            VideoObject,
            Walkthrough
        )
    }
}

impl From<CreativeWorkVariant> for Node {
    fn from(work: CreativeWorkVariant) -> Self {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match work {
                    $(
                        CreativeWorkVariant::$variant(node) => Node::$variant(node),
                    )*
                }
            };
        }

        variants!(
            Agent,
            Article,
            AudioObject,
            Chat,
            Claim,
            Collection,
            Comment,
            Datatable,
            Figure,
            File,
            ImageObject,
            MediaObject,
            Periodical,
            Prompt,
            PublicationIssue,
            PublicationVolume,
            Review,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Table,
            VideoObject,
            Workflow
        )
    }
}

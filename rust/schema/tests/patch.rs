use std::{collections::BTreeMap, fs::read_to_string};

use codec::{format::Format, Codec, DecodeOptions};
use codec_markdown::MarkdownCodec;
use common::{eyre::Result, glob::glob, serde::Serialize, tokio};
use common_dev::{insta::assert_yaml_snapshot, pretty_assertions::assert_eq};
use node_authorship::author_roles;
use schema::{
    diff, patch,
    shortcuts::{art, p, sec, t},
    Article, Author, AuthorRole, Block, Figure, Inline, Node, NodeProperty, Paragraph, PatchNode,
    PatchOp, PatchPath, PatchSlot, PatchValue, Primitive, Strong, Text, TimeUnit, Visitor,
    WalkControl,
};

/// An individual fixture
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
struct Fixture {
    /// The old node read from the fixture file
    old: Node,

    /// The new node read from the fixture file
    new: Node,

    /// The operations required to go from old to new
    ops: Vec<(PatchPath, PatchOp)>,

    /// The number of author roles before applying ops
    authors_before: usize,

    /// The number of author roles after applying ops
    authors_after: usize,
}

/// A summary of all the fixtures in this test
/// This is snap shotted so we can catch changes in key metrics
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
struct Summary {
    /// The number of operations for each fixture to go from old to new
    ops_count: BTreeMap<String, usize>,

    /// The total number of operations across all fixtures
    ops_total: usize,

    /// The number of operations for each fixture to go from old to new
    author_diffs: BTreeMap<String, isize>,

    /// The total number of operations across all fixtures
    author_diffs_sum: isize,
}

/// A visitor to count the author roles in a node
struct Authors(usize);

impl Authors {
    fn count(&mut self, authors: &Option<Vec<Author>>) {
        self.0 += authors.as_ref().map(|vec| vec.len()).unwrap_or_default()
    }
}

impl Visitor for Authors {
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        macro_rules! apply {
            ($($variant:ident),*) => {
                use Inline::*;
                match inline {
                    $($variant(node) => self.count(&node.options.authors),)*
                    _ => {}
                }
            };
        }

        // Should be applied to all inlines with the `authors` property
        apply!(
            AudioObject,
            CodeExpression,
            CodeInline,
            ImageObject,
            InstructionInline,
            MathInline,
            MediaObject,
            StyledInline,
            VideoObject
        );

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        macro_rules! apply {
            ($($variant:ident),*) => {
                use Block::*;
                match block {
                    $($variant(node) => self.count(&node.options.authors),)*
                    _ => {}
                }
            };
        }

        // Should be applied to all blocks with the `authors` property
        apply!(
            Admonition,
            Claim,
            CodeBlock,
            CodeChunk,
            Figure,
            ForBlock,
            Heading,
            IfBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            QuoteBlock,
            StyledBlock,
            Table
        );

        WalkControl::Continue
    }
}

// Count the authors in a node
fn authors(node: &Node) -> usize {
    let mut authors = Authors(0);
    authors.visit(node);
    authors.0
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn fixtures() -> Result<()> {
    let mut ops_count = BTreeMap::new();
    let mut ops_total = 0;
    let mut author_diffs = BTreeMap::new();
    let mut author_diffs_sum = 0;
    for path in glob("tests/fixtures/*.md")?.flatten() {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap()
            .to_string();

        // Read in the fixture and split into old and new Markdown
        let content = read_to_string(path)?;
        let mut parts = content.splitn(2, "===\n").map(String::from);
        let (old, new) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );

        // Decode the old and new Markdown into nodes
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });
        let codec = MarkdownCodec {};
        let (old, ..) = codec.from_str(&old, options.clone()).await?;
        let (new, ..) = codec.from_str(&new, options).await?;

        // To test the retention of properties not represented in source Markdown
        // apply the default author role recursively to the old node
        let mut enriched = old.clone();
        author_roles(&mut enriched, vec![AuthorRole::default()]);
        let authors_before = authors(&enriched);

        // Calculate the ops
        let ops = diff(&old, &new)?;
        ops_count.insert(name.clone(), ops.len());
        ops_total += ops.len();

        // Apply ops and assert that get new node
        let mut merged = old.clone();
        patch(&mut merged, ops.clone())?;
        assert_eq!(merged, new, "{name}\n{ops:#?}");

        // Apply the ops to enriched node and count authors after
        patch(&mut enriched, ops.clone())?;
        let authors_after = authors(&enriched);
        let author_diff = authors_after as isize - authors_before as isize;
        author_diffs.insert(name.clone(), author_diff);
        author_diffs_sum += author_diff;

        // Snapshot the fixture
        assert_yaml_snapshot!(
            name,
            Fixture {
                old,
                new,
                ops,
                authors_before,
                authors_after
            }
        );
    }

    // Snapshot summary
    assert_yaml_snapshot!(
        "summary",
        Summary {
            ops_count,
            ops_total,
            author_diffs,
            author_diffs_sum
        }
    );

    Ok(())
}

#[test]
fn atoms() -> Result<()> {
    // Boolean

    assert_eq!(diff(&true, &true)?, vec![]);

    let mut old = true;
    let new = false;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Integer

    assert_eq!(diff(&1_i64, &1_i64)?, vec![]);

    let mut old = 1_i64;
    let new = 2_i64;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Number

    assert_eq!(diff(&1_f64, &1_f64)?, vec![]);

    let mut old = 1_f64;
    let new = 2_f64;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // String

    assert_eq!(diff(&String::from("abc"), &String::from("abc"))?, vec![]);

    let mut old = String::from("abc");
    let new = String::from("bcd");
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vecs() -> Result<()> {
    // No ops: Both empty
    let mut old: Vec<i32> = vec![];
    let new = vec![];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Push: Old empty, new only has one
    let mut old = vec![];
    let new = vec![1];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(1.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Append: Old empty, new has more than one
    let mut old = vec![];
    let new = vec![1, 2, 3];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![1.to_value()?, 2.to_value()?, 3.to_value()?])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Clear: New empty
    let mut old = vec![1, 2, 3];
    let new = vec![];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Clear)]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: all different
    let mut old = vec![1, 2, 3, 4, 5, 6, 7];
    let new = vec![1, 3, 7];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 5]))]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: some same
    let mut old = vec![1, 1, 2, 2, 2, 3, 3, 4, 5, 5];
    let new = vec![1, 2, 3, 3, 4, 5];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 9]))]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Push: adding one
    let mut old = vec![1];
    let new = vec![1, 2];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(2.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Append: adding more than one
    let mut old = vec![1];
    let new = vec![1, 1, 2, 3];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![1.to_value()?, 2.to_value()?, 3.to_value()?])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Insert
    let mut old = vec![1, 3];
    let new = vec![0, 1, 2, 3, 4, 5];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Insert(vec![
                (0, 0.to_value()?),
                (2, 2.to_value()?),
                (4, 4.to_value()?),
                (5, 5.to_value()?)
            ])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Change: same size, all different
    let mut old = vec![1, 2];
    let new = vec![3, 4];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![
            (
                PatchPath::from([PatchSlot::Index(0)]),
                PatchOp::Set(3.to_value()?)
            ),
            (
                PatchPath::from([PatchSlot::Index(1)]),
                PatchOp::Set(4.to_value()?)
            )
        ]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Move
    let mut old = vec![1, 2, 3];
    let new = vec![3, 2, 1];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Move(vec![(0, 2), (0, 1)]))]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_sec() -> Result<()> {
    // This is a regression test for a bug found during testing
    let mut old = art([sec([p([t("para1")])])]);
    let new = art([sec([p([t("para1")]), p([t("para2")])])]);
    let ops = diff(&old, &new)?;
    assert!(matches!(ops[0].1, PatchOp::Push(..)));
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn enums() -> Result<()> {
    // Equal variants and values: no ops

    let node = Node::Article(Article::default());
    assert_eq!(diff(&node, &node)?, vec![]);

    let block = Block::Paragraph(Paragraph::default());
    assert_eq!(diff(&block, &block)?, vec![]);

    let inline = Inline::Text(Text::default());
    assert_eq!(diff(&inline, &inline)?, vec![]);

    let primitive = Primitive::Integer(1);
    assert_eq!(diff(&primitive, &primitive)?, vec![]);

    let time_unit = TimeUnit::Millisecond;
    assert_eq!(diff(&time_unit, &time_unit)?, vec![]);

    // Different variants: single replace op at root

    let node1 = Node::Article(Article::default());
    let node2 = Node::Integer(1);
    assert_eq!(
        diff(&node1, &node2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Node(node2)))]
    );

    let block1 = Block::Paragraph(Paragraph::default());
    let block2 = Block::Figure(Figure::default());
    assert_eq!(
        diff(&block1, &block2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Block(block2)))]
    );

    let inline1 = Inline::Text(Text::default());
    let inline2 = Inline::Strong(Strong::default());
    assert_eq!(
        diff(&inline1, &inline2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Inline(inline2)))]
    );

    let primitive1 = Primitive::Integer(1);
    let primitive2 = Primitive::String(String::new());
    assert_eq!(
        diff(&primitive1, &primitive2)?,
        vec![(PatchPath::new(), PatchOp::Set(primitive2.to_value()?))]
    );

    let time_unit1 = TimeUnit::Day;
    let time_unit2 = TimeUnit::Month;
    assert_eq!(
        diff(&time_unit1, &time_unit2)?,
        vec![(PatchPath::new(), PatchOp::Set(time_unit2.to_value()?))]
    );

    // Same variants, different values: ops depend differences
    use PatchSlot::*;

    let node1 = art([]);
    let node2 = art([p([t("para1")])]);
    assert_eq!(
        diff(&node1, &node2)?,
        vec![(
            PatchPath::from([Property(NodeProperty::Content)]),
            PatchOp::Push(PatchValue::Block(p([t("para1")])))
        )]
    );

    Ok(())
}

use std::{
    collections::{BTreeMap, HashMap},
    fs::read_to_string,
};

use codec::{format::Format, Codec, DecodeOptions};
use codec_markdown::MarkdownCodec;
use common::{
    eyre::{bail, Ok, Result},
    glob::glob,
    serde::Serialize,
    tokio,
};
use common_dev::{insta::assert_yaml_snapshot, pretty_assertions::assert_eq};
use node_strip::{strip, StripScope, StripTargets};
use schema::{
    authorship, diff, merge, patch,
    shortcuts::{art, p, sec, t},
    Article, Author, AuthorRole, AuthorRoleName, Block, CodeChunk, Cord, CordAuthorship, CordOp,
    Figure, Inline, InstructionBlock, InstructionType, Node, NodeProperty, Paragraph, Patch,
    PatchNode, PatchOp, PatchPath, PatchSlot, PatchValue, Person, Primitive, ProvenanceCategory,
    ProvenanceCount, SoftwareApplication, Strong, Text, TimeUnit,
};

/// An individual fixture
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
struct Fixture {
    /// The old node read from the fixture file
    old: Node,

    /// The new node read from the fixture file
    new: Node,

    /// The result of merging `new` into `old`
    merged: Node,

    /// The operations required to go from `old` to `merged`
    ops: Vec<(PatchPath, PatchOp)>,
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
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn fixtures() -> Result<()> {
    let mut ops_count = BTreeMap::new();
    let mut ops_total = 0;
    for path in glob("tests/fixtures/*.md")?.flatten() {
        eprintln!("{}", path.display());

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
        let (mut old, ..) = codec.from_str(&old, options.clone()).await?;
        let (new, ..) = codec.from_str(&new, options).await?;

        // Apply authorship to old
        let alice = AuthorRole::person(
            Person {
                given_names: Some(vec!["Alice".to_string()]),
                ..Default::default()
            },
            AuthorRoleName::Importer,
        );
        schema::authorship(&mut old, vec![alice])?;

        // Calculate the ops
        let bob = AuthorRole::person(
            Person {
                given_names: Some(vec!["Bob".to_string()]),
                ..Default::default()
            },
            AuthorRoleName::Writer,
        );
        let patch = diff(&old, &new, Some(Format::Markdown), Some(vec![bob]))?;
        ops_count.insert(name.clone(), patch.ops.len());
        ops_total += patch.ops.len();

        // Apply ops
        let mut merged = old.clone();
        schema::patch(&mut merged, patch.clone())?;

        // Assert that, when authors and provenance are stripped from both,
        // `merged` is the same as `new`
        let strip_targets = StripTargets::scopes(vec![StripScope::Authors, StripScope::Provenance]);
        let mut merged_strip = merged.clone();
        strip(&mut merged_strip, strip_targets.clone());
        let mut new_strip = new.clone();
        strip(&mut new_strip, strip_targets);
        assert_eq!(merged_strip, new_strip, "{name}\n{patch:#?}");

        // Snapshot the fixture. Strip timestamp for deterministic snapshots
        strip(&mut merged, StripTargets::scope(StripScope::Timestamps));
        assert_yaml_snapshot!(
            name,
            Fixture {
                old,
                new,
                ops: patch.ops,
                merged
            }
        );
    }

    // Snapshot summary
    assert_yaml_snapshot!(
        "summary",
        Summary {
            ops_count,
            ops_total
        }
    );

    Ok(())
}

/// Do a diff and get the ops
pub fn diff_ops<T: PatchNode>(old: &T, new: &T) -> Result<Vec<(PatchPath, PatchOp)>> {
    Ok(diff(old, new, Some(Format::Markdown), None)?.ops)
}

/// Patch a node with an anonymous author role
fn patch_anon<T: PatchNode + std::fmt::Debug>(
    old: &mut T,
    ops: Vec<(PatchPath, PatchOp)>,
) -> Result<()> {
    patch(
        old,
        Patch {
            ops,
            authors: Some(vec![AuthorRole::anon(AuthorRoleName::Writer)]),
            ..Default::default()
        },
    )
}

#[test]
fn atoms() -> Result<()> {
    // Boolean

    assert_eq!(diff_ops(&true, &true)?, vec![]);

    let mut old = true;
    let new = false;
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Integer

    assert_eq!(diff_ops(&1_i64, &1_i64)?, vec![]);

    let mut old = 1_i64;
    let new = 2_i64;
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Number

    assert_eq!(diff_ops(&1_f64, &1_f64)?, vec![]);

    let mut old = 1_f64;
    let new = 2_f64;
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // String

    assert_eq!(
        diff_ops(&String::from("abc"), &String::from("abc"))?,
        vec![]
    );

    let mut old = String::from("abc");
    let new = String::from("bcd");
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn cord() -> Result<()> {
    assert_eq!(diff_ops(&Cord::from("abc"), &Cord::from("abc"))?, vec![]);

    let mut old = Cord::from("abc");
    let new = Cord::from("bcad");
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Apply(vec![
                CordOp::Delete(0..1),
                CordOp::Insert(2, "ad".to_string())
            ])
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    let mut old = Cord::from("height in feet");
    let new = Cord::from("height in metres");
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Apply(vec![
                CordOp::Replace(10..11, "m".to_string()),
                CordOp::Insert(12, "tr".to_string()),
                CordOp::Replace(15..16, "s".to_string())
            ])
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vecs() -> Result<()> {
    // No ops: Both empty
    let mut old: Vec<i32> = vec![];
    let new = vec![];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Change: same size, all different
    let mut old = vec![1, 2];
    let new = vec![3, 4];
    let ops = diff_ops(&old, &new)?;
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
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_push() -> Result<()> {
    // Push: Old empty, new only has one
    let mut old = vec![];
    let new = vec![1];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(1.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Push: adding one
    let mut old = vec![1];
    let new = vec![1, 2];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(2.to_value()?))]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_append() -> Result<()> {
    // Append: Old empty, new has more than one
    let mut old = vec![];
    let new = vec![1, 2, 3];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![1.to_value()?, 2.to_value()?, 3.to_value()?])
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Append: adding more than one
    let mut old = vec![1];
    let new = vec![1, 2, 3];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![2.to_value()?, 3.to_value()?])
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_insert() -> Result<()> {
    // Insert
    let mut old = vec![1, 3];
    let new = vec![0, 1, 2, 3, 4, 5];
    let ops = diff_ops(&old, &new)?;
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
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vecs_remove() -> Result<()> {
    // Clear: New empty
    let mut old = vec![1, 2, 3];
    let new = vec![];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Clear)]);
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: all different
    let mut old = vec![1, 2, 3, 4, 5, 6, 7];
    let new = vec![1, 3, 7];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 5]))]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: some same
    let mut old = vec![1, 1, 2, 2, 2, 3, 3, 4, 5, 5];
    let new = vec![1, 2, 3, 3, 4, 5];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 9]))]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_copy() -> Result<()> {
    // Copy forward
    let mut old = vec![1, 2, 3];
    let new = vec![1, 1, 2, 1, 3];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Copy(HashMap::from([(0, vec![1, 3])]))
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Copy back
    let mut old = vec![1, 2, 3];
    let new = vec![1, 3, 2, 3, 3];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Copy(HashMap::from([(2, vec![1, 4])]))
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    // Copy forward and back
    let mut old = vec![1, 2, 3, 4];
    let new = vec![1, 4, 2, 3, 4, 1];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Copy(HashMap::from([(0, vec![5]), (3, vec![1])]))
        )]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_move() -> Result<()> {
    let mut old = vec![1, 2, 3];
    let new = vec![3, 2, 1];
    let ops = diff_ops(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Move(vec![(0, 2), (0, 1)]))]
    );
    patch_anon(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn vec_section() -> Result<()> {
    // This is a regression test for a bug found during testing
    let mut old = art([sec([p([t("para1")])])]);
    let new = art([sec([p([t("para1")]), p([t("para2")])])]);
    let ops = diff_ops(&old, &new)?;
    assert!(matches!(ops[0].1, PatchOp::Push(..)));
    patch_anon(&mut old, ops)?;
    strip(
        &mut old,
        StripTargets::scopes(vec![StripScope::Authors, StripScope::Provenance]),
    );
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn enums() -> Result<()> {
    // Equal variants and values: no ops

    let node = Node::Article(Article::default());
    assert_eq!(diff_ops(&node, &node)?, vec![]);

    let block = Block::Paragraph(Paragraph::default());
    assert_eq!(diff_ops(&block, &block)?, vec![]);

    let inline = Inline::Text(Text::default());
    assert_eq!(diff_ops(&inline, &inline)?, vec![]);

    let primitive = Primitive::Integer(1);
    assert_eq!(diff_ops(&primitive, &primitive)?, vec![]);

    let time_unit = TimeUnit::Millisecond;
    assert_eq!(diff_ops(&time_unit, &time_unit)?, vec![]);

    // Different variants: single replace op at root

    let node1 = Node::Article(Article::default());
    let node2 = Node::Integer(1);
    assert_eq!(
        diff_ops(&node1, &node2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Node(node2)))]
    );

    let block1 = Block::Paragraph(Paragraph::default());
    let block2 = Block::Figure(Figure::default());
    assert_eq!(
        diff_ops(&block1, &block2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Block(block2)))]
    );

    let inline1 = Inline::Text(Text::default());
    let inline2 = Inline::Strong(Strong::default());
    assert_eq!(
        diff_ops(&inline1, &inline2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Inline(inline2)))]
    );

    let primitive1 = Primitive::Integer(1);
    let primitive2 = Primitive::String(String::new());
    assert_eq!(
        diff_ops(&primitive1, &primitive2)?,
        vec![(PatchPath::new(), PatchOp::Set(primitive2.to_value()?))]
    );

    let time_unit1 = TimeUnit::Day;
    let time_unit2 = TimeUnit::Month;
    assert_eq!(
        diff_ops(&time_unit1, &time_unit2)?,
        vec![(PatchPath::new(), PatchOp::Set(time_unit2.to_value()?))]
    );

    // Same variants, different values: ops depend differences
    use PatchSlot::*;

    let node1 = art([]);
    let node2 = art([p([t("para1")])]);
    assert_eq!(
        diff_ops(&node1, &node2)?,
        vec![(
            PatchPath::from([Property(NodeProperty::Content)]),
            PatchOp::Push(PatchValue::Block(p([t("para1")])))
        )]
    );

    Ok(())
}

#[test]
fn authors() -> Result<()> {
    let alice = AuthorRole::person(
        Person {
            given_names: Some(vec!["Alice".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let bob = AuthorRole::person(
        Person {
            given_names: Some(vec!["Bob".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let carol = AuthorRole::person(
        Person {
            given_names: Some(vec!["Carol".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let _hal = AuthorRole::software(
        SoftwareApplication {
            name: "Hal".to_string(),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );

    use ProvenanceCategory::*;
    fn prov(cat: ProvenanceCategory, count: u64, percent: u64) -> ProvenanceCount {
        ProvenanceCount {
            provenance_category: cat,
            character_count: count,
            character_percent: Some(percent),
            ..Default::default()
        }
    }

    // Strip timestamps below, for determinism
    let strip_targets = StripTargets::scope(StripScope::Timestamps);

    // For code chunk, and any thing else with authors, authorship is recorded
    // at that level.
    let mut chunk = CodeChunk::new("a".into());
    authorship(&mut chunk, vec![alice.clone()])?;
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 1, 100)]));

    // Extend authorship run for alice
    merge(
        &mut chunk,
        &CodeChunk::new("abc".into()),
        Some(Format::Markdown),
        Some(vec![alice.clone()]),
    )?;
    strip(&mut chunk, strip_targets.clone());
    assert_eq!(chunk.code, "abc".into());
    assert_eq!(chunk.code.authorship, vec![CordAuthorship::new(1, 0, 0, 3)]);
    assert_eq!(chunk.authors, Some(vec![Author::AuthorRole(alice.clone())]));
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 3, 100)]));

    // Append by bob
    merge(
        &mut chunk,
        &CodeChunk::new("abcd".into()),
        Some(Format::Markdown),
        Some(vec![bob.clone()]),
    )?;
    strip(&mut chunk, strip_targets.clone());
    assert_eq!(chunk.code, "abcd".into());
    assert_eq!(
        chunk.code.authorship,
        vec![
            CordAuthorship::new(1, 0, 0, 3),
            CordAuthorship::new(1, 1, 0, 1)
        ]
    );
    assert_eq!(
        chunk.authors,
        Some(vec![
            Author::AuthorRole(alice.clone()),
            Author::AuthorRole(bob.clone())
        ])
    );
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 4, 100)]));

    // Insert by bob
    merge(
        &mut chunk,
        &CodeChunk::new("abxcd".into()),
        Some(Format::Markdown),
        Some(vec![bob.clone()]),
    )?;
    strip(&mut chunk, strip_targets.clone());
    assert_eq!(chunk.code, "abxcd".into());
    assert_eq!(
        chunk.code.authorship,
        vec![
            CordAuthorship::new(1, 0, 0, 2),
            CordAuthorship::new(1, 1, 0, 1),
            CordAuthorship::new(1, 0, 0, 1),
            CordAuthorship::new(1, 1, 0, 1)
        ]
    );
    assert_eq!(
        chunk.authors,
        Some(vec![
            Author::AuthorRole(alice.clone()),
            Author::AuthorRole(bob.clone())
        ])
    );
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 5, 100)]));

    // Delete by carol
    merge(
        &mut chunk,
        &CodeChunk::new("ad".into()),
        Some(Format::Markdown),
        Some(vec![carol.clone()]),
    )?;
    strip(&mut chunk, strip_targets.clone());
    assert_eq!(chunk.code, "ad".into());
    assert_eq!(
        chunk.code.authorship,
        vec![
            CordAuthorship::new(1, 0, 0, 1),
            CordAuthorship::new(1, 1, 0, 1)
        ]
    );
    assert_eq!(
        chunk.authors,
        Some(vec![
            Author::AuthorRole(alice.clone()),
            Author::AuthorRole(bob.clone()),
            Author::AuthorRole(carol.clone())
        ])
    );
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 2, 100)]));

    // Insert by carol
    merge(
        &mut chunk,
        &CodeChunk::new("and".into()),
        Some(Format::Markdown),
        Some(vec![carol.clone()]),
    )?;
    strip(&mut chunk, strip_targets.clone());
    assert_eq!(chunk.code, "and".into());
    assert_eq!(
        chunk.code.authorship,
        vec![
            CordAuthorship::new(1, 0, 0, 1),
            CordAuthorship::new(1, 2, 0, 1),
            CordAuthorship::new(1, 1, 0, 1)
        ]
    );
    assert_eq!(
        chunk.authors,
        Some(vec![
            Author::AuthorRole(alice.clone()),
            Author::AuthorRole(bob.clone()),
            Author::AuthorRole(carol.clone())
        ])
    );
    assert_eq!(chunk.provenance, Some(vec![prov(Hw, 3, 100)]));

    /*
    TODO: Fix and re-enable

    // Edit by bob
    merge(
        &mut chunk,
        &CodeChunk::new("ant".into()),
        Some(vec![bob.clone()]),
    )?;
    assert_eq!(chunk.code, "ant".into());
    assert_eq!(
        chunk.code.runs,
        vec![
            CordRun::new(1, 0, 0, 1),
            CordRun::new(1, 2, 0, 1),
            CordRun::new(2, 513, 2, 1)
        ]
    );
    assert_eq!(
        chunk.authors,
        Some(vec![
            Author::AuthorRole(alice.clone()),
            Author::AuthorRole(bob.clone()),
            Author::AuthorRole(carol.clone())
        ])
    );
    assert_eq!(
        chunk.provenance,
        Some(vec![prov(Hw, 3, 100), prov(HwHe, 1, 100)])
    );
    */

    Ok(())
}

/// Test that authorship is set and provenance is passed back up
#[test]
fn authorship_on_nodes() -> Result<()> {
    let alice = AuthorRole::person(
        Person {
            given_names: Some(vec!["Alice".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let authors = vec![alice.clone()];

    let mut para = Paragraph::new(vec![t("abc")]);
    authorship(&mut para, authors.clone())?;

    // Authors set OK?
    assert_eq!(para.authors, Some(vec![Author::AuthorRole(alice.clone())]));

    // Provenance on Cord?
    match &para.content[0] {
        Inline::Text(Text { value: cord, .. }) => assert!(cord.provenance().is_some()),
        _ => bail!("unexpected type"),
    }

    // Provenance on Text
    match &para.content[0] {
        Inline::Text(text) => assert!(text.provenance().is_some()),
        _ => bail!("unexpected type"),
    }

    // Provenance on Inline?
    assert!(para.content[0].provenance().is_some());

    // Provenance on Vec<Inline>?
    assert!(para.content.provenance().is_some());

    Ok(())
}

/// Test an archive patch
#[test]
fn archive_patch() -> Result<()> {
    // Archive an instruction with no content
    let inb = InstructionBlock::new(InstructionType::New);
    let mut article = Article::new(vec![Block::InstructionBlock(inb.clone())]);

    patch(
        &mut article,
        Patch {
            node_id: Some(inb.node_id()),
            ops: vec![(PatchPath::default(), PatchOp::Archive)],
            ..Default::default()
        },
    )?;

    assert_eq!(
        article.archive.iter().flatten().next().unwrap(),
        &Node::InstructionBlock(inb)
    );
    assert!(article.content.is_empty());

    // Archive an instruction with one block in content
    let block = Block::Paragraph(Paragraph {
        content: vec![t("one")],
        authors: Some(vec![Author::SoftwareApplication(
            SoftwareApplication::default(),
        )]),
        ..Default::default()
    });
    let inb = InstructionBlock {
        instruction_type: InstructionType::New,
        content: Some(vec![block.clone()]),
        ..Default::default()
    };
    let mut article = Article::new(vec![Block::InstructionBlock(inb.clone())]);

    patch(
        &mut article,
        Patch {
            node_id: Some(inb.node_id()),
            ops: vec![(PatchPath::default(), PatchOp::Archive)],
            // The author of the patch should not be applied to
            // the content in archived node or it's replacement content
            authors: Some(vec![AuthorRole::default()]),
            ..Default::default()
        },
    )?;

    assert_eq!(
        article.archive.iter().flatten().next().unwrap(),
        &Node::InstructionBlock(inb)
    );
    if let Some(Block::Paragraph(Paragraph {
        content, authors, ..
    })) = article.content.first()
    {
        assert_eq!(content, &vec![t("one")]);
        assert_eq!(
            authors,
            &Some(vec![Author::SoftwareApplication(
                SoftwareApplication::default(),
            )])
        );
    } else {
        bail!("unexpected content");
    };

    // Archive an instruction with two blocks in content
    let blocks = vec![p([t("one")]), p([t("two")])];
    let inb = InstructionBlock {
        instruction_type: InstructionType::New,
        content: Some(blocks.clone()),
        ..Default::default()
    };
    let mut article = Article::new(vec![Block::InstructionBlock(inb.clone())]);

    patch(
        &mut article,
        Patch {
            node_id: Some(inb.node_id()),
            ops: vec![(PatchPath::default(), PatchOp::Archive)],
            ..Default::default()
        },
    )?;

    assert_eq!(
        article.archive.iter().flatten().next().unwrap(),
        &Node::InstructionBlock(inb)
    );
    assert_eq!(&article.content, &blocks);

    Ok(())
}

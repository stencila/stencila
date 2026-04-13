use insta::assert_yaml_snapshot;
use stencila_codec::{
    eyre::{Result, bail},
    stencila_schema::{Article, Block, Inline, Node, Paragraph, SuggestionBlock},
};

fn decode_blocks(md: &str) -> Result<Vec<Block>> {
    let (Node::Article(Article { content, .. }), ..) = stencila_codec_markdown::decode(md, None)?
    else {
        bail!("Expected an Article")
    };

    Ok(content)
}

fn decode_inlines(md: &str) -> Result<Vec<Inline>> {
    let blocks = decode_blocks(md)?;

    let Some(Block::Paragraph(Paragraph { content, .. })) = blocks.first() else {
        bail!("Expected a Paragraph")
    };

    Ok(content.clone())
}

#[test]
fn replacement_inline() -> Result<()> {
    let inlines = decode_inlines("Before {~~old text~>new text~~} after")?;

    assert_yaml_snapshot!(inlines, @r#"
    - type: Text
      value:
        string: "Before "
    - type: SuggestionInline
      suggestionType: Replace
      content:
        - type: Text
          value:
            string: new text
      original:
        - type: Text
          value:
            string: old text
    - type: Text
      value:
        string: " after"
    "#);

    Ok(())
}

#[test]
fn replacement_block() -> Result<()> {
    let blocks = decode_blocks(
        r#":~~

Old paragraph.

:~>

New paragraph.

:~~"#,
    )?;

    let Some(Block::SuggestionBlock(SuggestionBlock {
        suggestion_type,
        original,
        content,
        ..
    })) = blocks.first()
    else {
        bail!("Expected a SuggestionBlock")
    };

    assert_eq!(
        suggestion_type.as_ref(),
        Some(&stencila_codec::stencila_schema::SuggestionType::Replace)
    );
    assert!(matches!(original, Some(original) if original.len() == 1));
    assert_eq!(content.len(), 1);

    assert_yaml_snapshot!(blocks, @r#"
    - type: SuggestionBlock
      suggestionType: Replace
      content:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: New paragraph.
      original:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: Old paragraph.
    "#);

    Ok(())
}

#[test]
fn replacement_block_can_have_empty_original() -> Result<()> {
    let blocks = decode_blocks(
        r#":~~

:~>

Inserted paragraph.

:~~"#,
    )?;

    assert_yaml_snapshot!(blocks, @r#"
    - type: SuggestionBlock
      suggestionType: Replace
      content: []
      original:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: Inserted paragraph.
    "#);

    Ok(())
}

#[test]
fn replacement_block_can_have_empty_content() -> Result<()> {
    let blocks = decode_blocks(
        r#":~~

Removed paragraph.

:~>

:~~"#,
    )?;

    assert_yaml_snapshot!(blocks, @r#"
    - type: SuggestionBlock
      suggestionType: Replace
      content: []
      original:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: Removed paragraph.
    "#);

    Ok(())
}

#[test]
fn replacement_block_folds_single_paragraph_sides() -> Result<()> {
    let blocks = decode_blocks(":~~\n\nOld\n\n:~>\n\nNew\n\n:~~")?;

    assert_yaml_snapshot!(blocks, @r#"
    - type: SuggestionBlock
      suggestionType: Replace
      content:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: New
      original:
        - type: Paragraph
          content:
            - type: Text
              value:
                string: Old
    "#);

    Ok(())
}

#[test]
fn replacement_block_inside_instruction_with_explicit_suggestion_slot() -> Result<()> {
    let blocks = decode_blocks(
        r#"::: edit something >>>

:~~

Old

:~>

New

:~~

:::"#,
    )?;

    let Some(Block::InstructionBlock(instruction)) = blocks.first() else {
        bail!("Expected an InstructionBlock")
    };

    let Some(suggestions) = &instruction.suggestions else {
        bail!("Expected instruction suggestions")
    };
    let Some(SuggestionBlock {
        suggestion_type,
        original,
        content,
        ..
    }) = suggestions.first()
    else {
        bail!("Expected a suggestion")
    };

    assert_eq!(
        suggestion_type.as_ref(),
        Some(&stencila_codec::stencila_schema::SuggestionType::Replace)
    );
    assert!(matches!(original, Some(original) if original.len() == 1));
    assert!(content.is_empty());

    assert_yaml_snapshot!(blocks, @r#"
    - type: InstructionBlock
      instructionType: Edit
      prompt:
        type: PromptBlock
        instructionType: Edit
        query: something
      message:
        type: InstructionMessage
        content:
          - type: Text
            value:
              string: something
      modelParameters:
        type: ModelParameters
      content: []
      suggestions:
        - type: SuggestionBlock
          suggestionType: Replace
          content: []
          original:
            - type: Paragraph
              content:
                - type: Text
                  value:
                    string: Old
    "#);

    Ok(())
}

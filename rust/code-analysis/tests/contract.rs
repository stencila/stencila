mod support;

use eyre::Result;
use stencila_code_analysis::{
    ANALYZER_IDENTITY, Analysis, AnalysisDiagnostic, CodeLanguage, analyze_file,
    integration::source_position,
};

#[test]
fn fixture_json_contract() -> Result<()> {
    let fixtures = support::fixtures()?;
    assert_eq!(
        fixtures.len(),
        31,
        "update fixture coverage when the corpus changes"
    );
    for fixture in fixtures {
        let analysis = analyze_file(fixture.language, &fixture.path, &fixture.source);
        let name = fixture.path.replace(['/', '.', '#', '-'], "_");
        insta::assert_json_snapshot!(name, analysis);
        let value = serde_json::to_value(&analysis)?;
        let decoded: Analysis = serde_json::from_value(value.clone())?;
        // Deserialized facts intentionally do not reconstruct private metadata.
        assert_eq!(serde_json::to_value(decoded)?, value);
        for private in [
            "importedSymbols",
            "variableSources",
            "definitionOffsets",
            "useOffsets",
            "readBeforeWriteSymbols",
        ] {
            assert!(value["facts"].get(private).is_none());
        }
    }
    Ok(())
}

#[test]
fn detection_uses_logical_names() {
    for (path, expected) in [
        ("never-created/analysis.py", CodeLanguage::Python),
        ("remote/Snakefile", CodeLanguage::Snakemake),
        ("remote/nextflow.config", CodeLanguage::Nextflow),
        (r"C:\logical\Snakefile", CodeLanguage::Snakemake),
        ("source.MTS", CodeLanguage::TypeScript),
        ("source.cjs", CodeLanguage::JavaScript),
        ("logical/python", CodeLanguage::Python),
    ] {
        assert_eq!(CodeLanguage::from_path(path), Some(expected));
    }
    assert_eq!(CodeLanguage::from_extension(".R"), Some(CodeLanguage::R));
    assert_eq!(CodeLanguage::from_path("unsupported.sh"), None);
    assert_eq!(CodeLanguage::from_programming_language("bash"), None);
    assert_eq!(CodeLanguage::from_path("no-extension"), None);
    assert_eq!(CodeLanguage::from_path(".py"), None);
}

#[test]
fn positions_use_unicode_columns_and_validate_boundaries() -> Result<()> {
    let position = source_position("α\nβc", 5).ok_or_else(|| eyre::eyre!("valid UTF-8 offset"))?;
    assert_eq!((position.line, position.column, position.offset), (1, 1, 5));
    assert!(source_position("α", 1).is_none());
    assert!(source_position("α", 3).is_none());
    let invalid = analyze_file(CodeLanguage::Python, "missing/bad.py", "if (");
    assert_eq!(invalid.diagnostics, vec![AnalysisDiagnostic::SyntaxError]);
    Ok(())
}

#[test]
fn identity_has_exact_grammar_pins() {
    assert_eq!(ANALYZER_IDENTITY.source_digest.len(), 64);
    assert!(
        ANALYZER_IDENTITY
            .source_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    );
    assert_eq!(ANALYZER_IDENTITY.grammars.len(), 8);
    assert!(
        ANALYZER_IDENTITY
            .grammars
            .iter()
            .any(|grammar| grammar.package == "tree-sitter-snakemake"
                && grammar.git_commit == Some("7731408e5e8095fe242fdd423c3d3ae886fbf9fd"))
    );
}

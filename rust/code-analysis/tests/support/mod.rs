use std::{fs, path::Path};

use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stencila_code_analysis::CodeLanguage;

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub path: String,
    pub language: CodeLanguage,
    pub source: String,
}

fn files(path: &Path, entries: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            files(&path, entries)?;
        } else {
            entries.push(path);
        }
    }
    Ok(())
}

fn document_code(value: &Value, path: &str, cases: &mut Vec<Source>) {
    match value {
        Value::Object(node) => {
            if matches!(
                node.get("type").and_then(Value::as_str),
                Some("CodeChunk" | "CodeExpression")
            ) && let Some(language) = node
                .get("programmingLanguage")
                .and_then(Value::as_str)
                .and_then(CodeLanguage::from_programming_language)
                && let Some(source) = node.get("code").and_then(Value::as_str)
            {
                let id = node
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| cases.len().to_string());
                cases.push(Source {
                    path: format!("{path}#{id}"),
                    language,
                    source: source.to_string(),
                });
            }
            for child in node.values() {
                document_code(child, path, cases);
            }
        }
        Value::Array(children) => {
            for child in children {
                document_code(child, path, cases);
            }
        }
        _ => {}
    }
}

/// The existing graph fixtures remain the source of truth for both targets.
/// Reading documents here is test-only; the production API takes extracted text.
pub fn fixtures() -> Result<Vec<Source>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../graph/tests/fixtures");
    let mut paths = Vec::new();
    files(&root, &mut paths)?;
    paths.sort();
    let mut cases = Vec::new();
    for path in paths {
        let logical = path
            .strip_prefix(&root)?
            .to_string_lossy()
            .replace('\\', "/");
        if let Some(language) = CodeLanguage::from_path(&path) {
            cases.push(Source {
                path: logical,
                language,
                source: fs::read_to_string(&path)?,
            });
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            // One negative fixture deliberately contains invalid package JSON.
            if let Ok(value) = serde_json::from_str::<Value>(&fs::read_to_string(path)?) {
                document_code(&value, &logical, &mut cases);
            }
        } else if path.extension().is_some_and(|extension| extension == "smd") {
            let text = fs::read_to_string(path)?;
            let mut lines = text.split_inclusive('\n');
            let mut index = 0;
            while let Some(line) = lines.next() {
                let trimmed = line.trim_start();
                let Some(info) = trimmed.strip_prefix("```") else {
                    continue;
                };
                let indent = &line[..line.len() - trimmed.len()];
                let label = info.split_whitespace().next().unwrap_or_default();
                if label.is_empty() {
                    continue;
                }
                let language = CodeLanguage::from_programming_language(label);
                let mut source = String::new();
                for line in lines.by_ref() {
                    if line.trim() == "```" {
                        break;
                    }
                    source.push_str(line.strip_prefix(indent).unwrap_or(line));
                }
                if let Some(language) = language {
                    cases.push(Source {
                        path: format!("{logical}#fence-{index}"),
                        language,
                        source,
                    });
                }
                index += 1;
            }
        }
    }
    Ok(cases)
}

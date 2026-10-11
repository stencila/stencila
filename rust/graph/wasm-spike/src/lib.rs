#![allow(dead_code)]
mod code;
mod package;
mod reference;
#[cfg(target_arch = "wasm32")]
mod wasm_allocator;

use wasm_bindgen::prelude::*;

pub fn analyze(language: &str, source: &str) -> Result<String, String> {
    let language = code::CodeLanguage::from_programming_language(language)
        .ok_or_else(|| format!("Unsupported language: {language}"))?;
    serde_json::to_string(&code::analyze_source(language, source))
        .map_err(|error| error.to_string())
}

#[wasm_bindgen]
pub fn analyze_source(language: &str, source: &str) -> Result<String, JsValue> {
    analyze(language, source).map_err(|error| JsValue::from_str(&error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bash() {
        assert!(analyze("bash", "cat data.csv").is_err());
    }

    #[test]
    fn serializes_fixture_facts_and_private_state() -> Result<(), Box<dyn std::error::Error>> {
        let source = include_str!("../../tests/fixtures/code-polyglot-tools-io/analysis.py");
        let facts: serde_json::Value = serde_json::from_str(&analyze("python", source)?)?;
        assert_eq!(facts["syntax_error"], false);
        assert!(facts["io"].as_array().is_some_and(|io| !io.is_empty()));
        assert!(facts.get("definition_offsets").is_some());
        assert_eq!(analyze("python", source)?, analyze("python", source)?);
        Ok(())
    }
}

/// Allocated WebAssembly linear memory; it grows monotonically, so the value
/// after a call records its capacity high-water mark, including C allocations.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn linear_memory_bytes() -> usize {
    core::arch::wasm32::memory_size(0) * 65536
}

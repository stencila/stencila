//! Tokenization for search indexing
//!
//! This module implements text tokenization that must produce identical
//! results to the TypeScript tokenizer in `web/src/site/search/tokenize.ts`.
//!
//! Tokenization rules:
//! 1. NFD normalize (preserving case for camelCase detection)
//! 2. Fold diacritics (é → e, ü → u)
//! 3. Split on non-alphanumeric boundaries
//! 4. Split code identifiers (camelCase, snake_case, kebab-case) and lowercase
//! 5. Filter tokens < 2 Unicode code points
//! 6. No stemming (keeps cross-language parity simple)

use unicode_normalization::UnicodeNormalization;

/// Tokenize text into searchable tokens
///
/// This function must produce identical output to the TypeScript
/// `tokenize()` function for the same input.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    // NFD normalize (but don't lowercase yet - we need case for camelCase splitting)
    let normalized: String = text.nfd().collect();

    // Process character by character, folding diacritics and splitting on boundaries
    let mut current_word = String::new();

    for c in normalized.chars() {
        // Skip combining diacritical marks (U+0300-U+036F) - this folds diacritics
        // They appear after NFD normalization (e.g., "é" -> "e" + combining accent)
        if ('\u{0300}'..='\u{036F}').contains(&c) {
            continue;
        }

        if c.is_alphanumeric() {
            current_word.push(c);
        } else {
            // Non-alphanumeric boundary (whitespace, punctuation, underscore, hyphen)
            if !current_word.is_empty() {
                // Split camelCase (this also lowercases)
                tokens.extend(split_camel_case(&current_word));
                current_word.clear();
            }
        }
    }

    // Don't forget the last word
    if !current_word.is_empty() {
        tokens.extend(split_camel_case(&current_word));
    }

    // Filter short tokens (< 2 Unicode code points)
    // Note: Use chars().count() for parity with TypeScript's [...t].length
    tokens.retain(|t| t.chars().count() >= 2);

    tokens
}

/// Split camelCase and PascalCase identifiers into separate tokens
///
/// Examples:
/// - "camelCase" -> ["camel", "case"]
/// - "PascalCase" -> ["pascal", "case"]
/// - "HTMLParser" -> ["html", "parser"]
/// - "getID" -> ["get", "id"]
fn split_camel_case(word: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = word.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        let is_upper = c.is_ascii_uppercase();
        let was_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
        let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase();

        // Start new token if:
        // 1. Uppercase after lowercase (camelCase boundary): "camelCase" splits at 'C'
        // 2. Uppercase followed by lowercase in uppercase run (HTMLParser): "HTML" + "Parser"
        let should_split = is_upper && !current.is_empty() && (was_lower || next_is_lower);

        if should_split {
            tokens.push(current.to_lowercase());
            current.clear();
        }

        current.push(c);
    }

    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens
}

/// Get the 2-character prefix for shard lookup
///
/// This determines which shard a token belongs to.
pub fn token_prefix(token: &str) -> String {
    token.chars().take(2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // These test vectors must match the TypeScript tests exactly

    #[test]
    fn test_basic_tokenization() {
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
        assert_eq!(tokenize("Hello World"), vec!["hello", "world"]);
        assert_eq!(tokenize("HELLO WORLD"), vec!["hello", "world"]);
    }

    #[test]
    fn test_diacritic_folding() {
        assert_eq!(tokenize("café"), vec!["cafe"]);
        assert_eq!(tokenize("naïve"), vec!["naive"]);
        assert_eq!(tokenize("résumé"), vec!["resume"]);
        assert_eq!(tokenize("Zürich"), vec!["zurich"]);
        assert_eq!(tokenize("São Paulo"), vec!["sao", "paulo"]);
    }

    #[test]
    fn test_camel_case_splitting() {
        assert_eq!(tokenize("camelCase"), vec!["camel", "case"]);
        assert_eq!(tokenize("PascalCase"), vec!["pascal", "case"]);
        assert_eq!(tokenize("HTMLParser"), vec!["html", "parser"]);
        assert_eq!(tokenize("getID"), vec!["get", "id"]);
        assert_eq!(
            tokenize("parseXMLDocument"),
            vec!["parse", "xml", "document"]
        );
    }

    #[test]
    fn test_snake_case_splitting() {
        assert_eq!(tokenize("snake_case"), vec!["snake", "case"]);
        assert_eq!(tokenize("SCREAMING_SNAKE"), vec!["screaming", "snake"]);
        assert_eq!(tokenize("mixed_camelCase"), vec!["mixed", "camel", "case"]);
    }

    #[test]
    fn test_kebab_case_splitting() {
        assert_eq!(tokenize("kebab-case"), vec!["kebab", "case"]);
        assert_eq!(
            tokenize("my-component-name"),
            vec!["my", "component", "name"]
        );
    }

    #[test]
    fn test_file_paths() {
        assert_eq!(
            tokenize("src/components/Button.tsx"),
            vec!["src", "components", "button", "tsx"]
        );
        assert_eq!(
            tokenize("my-project/README.md"),
            vec!["my", "project", "readme", "md"]
        );
    }

    #[test]
    fn test_short_token_filtering() {
        assert_eq!(tokenize("a b c"), Vec::<String>::new());
        assert_eq!(tokenize("I am a test"), vec!["am", "test"]);
        assert_eq!(tokenize("x = 42"), vec!["42"]);
        // Single non-ASCII characters should be filtered (1 Unicode char < 2)
        assert_eq!(tokenize("你"), Vec::<String>::new());
        assert_eq!(tokenize("你 好"), Vec::<String>::new()); // Both are single chars
        // But two-char CJK words should pass
        assert_eq!(tokenize("你好"), vec!["你好"]);
    }

    #[test]
    fn test_punctuation_handling() {
        assert_eq!(tokenize("hello, world!"), vec!["hello", "world"]);
        assert_eq!(tokenize("what's up?"), vec!["what", "up"]);
        assert_eq!(tokenize("test@example.com"), vec!["test", "example", "com"]);
    }

    #[test]
    fn test_numbers() {
        assert_eq!(tokenize("test123"), vec!["test123"]);
        assert_eq!(tokenize("123test"), vec!["123test"]);
        assert_eq!(tokenize("test 123 more"), vec!["test", "123", "more"]);
    }

    #[test]
    fn test_empty_and_whitespace() {
        assert_eq!(tokenize(""), Vec::<String>::new());
        assert_eq!(tokenize("   "), Vec::<String>::new());
        assert_eq!(tokenize("\n\t"), Vec::<String>::new());
    }

    #[test]
    fn test_token_prefix() {
        assert_eq!(token_prefix("hello"), "he");
        assert_eq!(token_prefix("a"), "a");
        assert_eq!(token_prefix("ab"), "ab");
        assert_eq!(token_prefix("abc"), "ab");
    }

    #[test]
    fn test_astral_unicode() {
        // Astral characters (outside BMP, U+10000+) should be counted as single code points
        // 𝒜 = U+1D49C (Mathematical Script Capital A) - 1 code point, 2 UTF-16 code units
        assert_eq!(tokenize("𝒜"), Vec::<String>::new()); // 1 char, filtered
        assert_eq!(tokenize("𝒜𝒷"), vec!["𝒜𝒷"]); // 2 chars, kept

        // Token prefix should use code points, not UTF-16 units
        assert_eq!(token_prefix("𝒜𝒷𝒸"), "𝒜𝒷"); // First 2 code points
        assert_eq!(token_prefix("𝒜bc"), "𝒜b"); // Mixed astral and ASCII
    }
}

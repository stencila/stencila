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

/// Generate character trigrams (3-grams) for fuzzy matching
///
/// Returns overlapping 3-character sequences from the token.
/// Tokens with fewer than 3 characters return an empty vector.
///
/// This function must produce identical output to the TypeScript
/// `generateTrigrams()` function for the same input.
///
/// Examples:
/// - "search" → ["sea", "ear", "arc", "rch"]
/// - "ab" → [] (too short)
/// - "abc" → ["abc"] (exactly 3 chars = 1 trigram)
pub fn generate_trigrams(token: &str) -> Vec<String> {
    let chars: Vec<char> = token.chars().collect();
    let len = chars.len();

    if len < 3 {
        return Vec::new();
    }

    let mut trigrams = Vec::with_capacity(len - 2);
    for i in 0..=(len - 3) {
        trigrams.push(chars[i..i + 3].iter().collect());
    }

    trigrams
}

/// Token with position information in original text
///
/// Used during trigram extraction to track where tokens appear.
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    /// The normalized token (lowercased, diacritics folded)
    pub token: String,
    /// Start position in original text (UTF-16 code units)
    pub start: u32,
    /// End position in original text (UTF-16 code units)
    pub end: u32,
}

/// Extract tokens with their positions from text
///
/// Splits text on non-alphanumeric boundaries, normalizes each word,
/// and returns tokens with their UTF-16 positions in the original text.
///
/// Note: CamelCase tokens will have the same start/end as the original word
/// since precise sub-positioning is complex and the highlighting will
/// still be visually correct (highlighting the whole word).
pub fn tokenize_with_positions(text: &str) -> Vec<TokenWithPosition> {
    let mut result = Vec::new();

    // Track position in UTF-16 code units
    let mut utf16_pos: u32 = 0;
    let mut word_start_utf16: u32 = 0;
    let mut current_word = String::new();
    let mut in_word = false;

    for c in text.chars() {
        let char_utf16_len = c.len_utf16() as u32;

        if c.is_alphanumeric() {
            if !in_word {
                word_start_utf16 = utf16_pos;
                in_word = true;
            }
            current_word.push(c);
        } else {
            // Non-alphanumeric boundary
            if in_word && !current_word.is_empty() {
                // Tokenize the word (normalizes and splits camelCase)
                let tokens = tokenize(&current_word);
                let word_end_utf16 = utf16_pos;

                // All tokens from this word share the same position span
                for token in tokens {
                    if token.chars().count() >= 2 {
                        result.push(TokenWithPosition {
                            token,
                            start: word_start_utf16,
                            end: word_end_utf16,
                        });
                    }
                }

                current_word.clear();
                in_word = false;
            }
        }

        utf16_pos += char_utf16_len;
    }

    // Don't forget the last word
    if in_word && !current_word.is_empty() {
        let tokens = tokenize(&current_word);
        let word_end_utf16 = utf16_pos;

        for token in tokens {
            if token.chars().count() >= 2 {
                result.push(TokenWithPosition {
                    token,
                    start: word_start_utf16,
                    end: word_end_utf16,
                });
            }
        }
    }

    result
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

    // Trigram tests - these must match TypeScript tests exactly

    #[test]
    fn test_trigrams_basic() {
        assert_eq!(
            generate_trigrams("search"),
            vec!["sea", "ear", "arc", "rch"]
        );
        assert_eq!(
            generate_trigrams("database"),
            vec!["dat", "ata", "tab", "aba", "bas", "ase"]
        );
        assert_eq!(generate_trigrams("hello"), vec!["hel", "ell", "llo"]);
    }

    #[test]
    fn test_trigrams_short_tokens() {
        // Tokens < 3 chars return empty vec
        assert_eq!(generate_trigrams(""), Vec::<String>::new());
        assert_eq!(generate_trigrams("a"), Vec::<String>::new());
        assert_eq!(generate_trigrams("ab"), Vec::<String>::new());
        // Exactly 3 chars = 1 trigram
        assert_eq!(generate_trigrams("abc"), vec!["abc"]);
    }

    #[test]
    fn test_trigrams_unicode() {
        // Diacritics should already be folded before trigram generation
        assert_eq!(generate_trigrams("cafe"), vec!["caf", "afe"]);
        // CJK characters (each is 1 code point)
        assert_eq!(generate_trigrams("你好世界"), vec!["你好世", "好世界"]);
    }

    #[test]
    fn test_trigrams_astral() {
        // Astral characters should be counted as single code points
        // 𝒜𝒷𝒸𝒹 = 4 code points → 2 trigrams
        assert_eq!(generate_trigrams("𝒜𝒷𝒸𝒹"), vec!["𝒜𝒷𝒸", "𝒷𝒸𝒹"]);
    }

    // tokenize_with_positions tests

    #[test]
    fn test_tokenize_with_positions_basic() {
        let result = tokenize_with_positions("hello world");
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].token, "hello");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 5);

        assert_eq!(result[1].token, "world");
        assert_eq!(result[1].start, 6);
        assert_eq!(result[1].end, 11);
    }

    #[test]
    fn test_tokenize_with_positions_camelcase() {
        // "camelCase" should produce two tokens with same position span
        let result = tokenize_with_positions("camelCase");
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].token, "camel");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 9);

        assert_eq!(result[1].token, "case");
        assert_eq!(result[1].start, 0);
        assert_eq!(result[1].end, 9);
    }

    #[test]
    fn test_tokenize_with_positions_diacritics() {
        // "café" is 4 UTF-16 code units (c, a, f, é)
        // After normalization: "cafe"
        let result = tokenize_with_positions("café test");
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].token, "cafe");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 4); // UTF-16: c(1) a(1) f(1) é(1) = 4

        assert_eq!(result[1].token, "test");
        assert_eq!(result[1].start, 5);
        assert_eq!(result[1].end, 9);
    }

    #[test]
    fn test_tokenize_with_positions_emoji() {
        // 👋 is 2 UTF-16 code units (surrogate pair)
        // "👋hello" should have "hello" starting at UTF-16 offset 2
        let result = tokenize_with_positions("👋hello");
        assert_eq!(result.len(), 1);

        assert_eq!(result[0].token, "hello");
        assert_eq!(result[0].start, 2); // After emoji (2 UTF-16 units)
        assert_eq!(result[0].end, 7); // 2 + 5
    }

    #[test]
    fn test_tokenize_with_positions_short_tokens_filtered() {
        // Single-char tokens should be filtered
        let result = tokenize_with_positions("a b hello c");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token, "hello");
    }

    #[test]
    fn test_tokenize_with_positions_punctuation() {
        let result = tokenize_with_positions("hello, world!");
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].token, "hello");
        assert_eq!(result[0].start, 0);
        assert_eq!(result[0].end, 5);

        assert_eq!(result[1].token, "world");
        assert_eq!(result[1].start, 7);
        assert_eq!(result[1].end, 12);
    }
}

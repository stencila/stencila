//! Provider-specific model ID parsers.
//!
//! Each parser decomposes a model ID string into structured parts used by
//! both the sort-key function and the alias generator.

/// Parsed components of an Anthropic model ID.
///
/// Handles both formats:
/// - `claude-{family}-{major}-{minor}[-date]`  (e.g. `claude-opus-4-6`)
/// - `claude-{major}-{minor}-{family}[-date]`  (e.g. `claude-3-7-sonnet-20250219`)
#[derive(Debug, Clone)]
pub(crate) struct ParsedAnthropicId {
    pub family: String,
    pub major: u32,
    pub minor: u32,
    #[cfg_attr(not(test), allow(dead_code))]
    pub date: Option<String>,
}

/// Parsed components of an OpenAI GPT-series model ID.
///
/// Covers `gpt-{major}[.{minor}][-variant][-date]` patterns.
#[derive(Debug, Clone)]
pub(crate) struct ParsedGptId {
    pub major: u32,
    pub minor: u32,
    pub variant: Option<String>,
    pub date: Option<String>,
    pub is_chat_latest: bool,
}

/// Parsed components of an OpenAI o-series model ID.
///
/// Covers `o{generation}[-variant][-date]` patterns.
#[derive(Debug, Clone)]
pub(crate) struct ParsedOId {
    pub generation: u32,
    pub variant: Option<String>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub date: Option<String>,
}

/// Parsed components of a Gemini model ID.
#[derive(Debug, Clone)]
pub(crate) struct ParsedGeminiId {
    pub version_major: u32,
    pub version_minor: u32,
    pub tier: String,
    pub suffix: String,
}

/// Parsed components of a Mistral model ID.
#[derive(Debug, Clone)]
pub(crate) struct ParsedMistralId {
    pub family: String,
    pub is_latest: bool,
}

/// Union of all parsed ID types.
#[derive(Debug, Clone)]
pub(crate) enum ParsedId {
    Anthropic(ParsedAnthropicId),
    Gpt(ParsedGptId),
    OSeries(ParsedOId),
    Gemini(ParsedGeminiId),
    Mistral(ParsedMistralId),
    Unknown,
}

// ---------------------------------------------------------------------------
// Anthropic
// ---------------------------------------------------------------------------

pub(crate) fn parse_anthropic(id: &str) -> Option<ParsedAnthropicId> {
    let id = id.strip_prefix("claude-")?;

    // Try new format: {family}-{major}-{minor}[-date]
    // e.g. "opus-4-6", "sonnet-4-5-20250929", "haiku-4-5-20251001"
    for family in &["opus", "sonnet", "haiku"] {
        if let Some(rest) = id.strip_prefix(family).and_then(|r| r.strip_prefix('-')) {
            let parts: Vec<&str> = rest.splitn(3, '-').collect();
            if parts.len() >= 2
                && let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                // Minor version is a small number; large numbers are dates (e.g. 20250514)
                && minor < 100
            {
                let date = if parts.len() == 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                };
                return Some(ParsedAnthropicId {
                    family: (*family).to_string(),
                    major,
                    minor,
                    date,
                });
            }
            // Bare family-major without minor (e.g. "opus-4-20250514")
            if let Ok(major) = parts[0].parse::<u32>() {
                let date = if parts.len() >= 2 {
                    Some(parts[1..].join("-"))
                } else {
                    None
                };
                return Some(ParsedAnthropicId {
                    family: (*family).to_string(),
                    major,
                    minor: 0,
                    date,
                });
            }
        }
    }

    // Try old format: {major}-{minor}-{family}[-date]
    // e.g. "3-7-sonnet-20250219", "3-5-haiku-20241022"
    let parts: Vec<&str> = id.splitn(4, '-').collect();
    if parts.len() >= 3
        && let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
    {
        let family_part = parts[2];
        for family in &["opus", "sonnet", "haiku"] {
            if family_part == *family {
                let date = if parts.len() == 4 {
                    Some(parts[3].to_string())
                } else {
                    None
                };
                return Some(ParsedAnthropicId {
                    family: (*family).to_string(),
                    major,
                    minor,
                    date,
                });
            }
        }
    }

    None
}

/// Family tier for sorting (lower = better).
pub(crate) fn anthropic_family_tier(family: &str) -> u8 {
    match family {
        "opus" => 0,
        "sonnet" => 1,
        "haiku" => 2,
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// OpenAI GPT series
// ---------------------------------------------------------------------------

pub(crate) fn parse_gpt(id: &str) -> Option<ParsedGptId> {
    let rest = id.strip_prefix("gpt-")?;

    let is_chat_latest = rest.ends_with("-chat-latest");
    let rest = rest.strip_suffix("-chat-latest").unwrap_or(rest);

    // Parse version: "5.2", "5.1", "5", "4.1", "4o", "4", "3.5"
    let (major, minor, remainder) = parse_gpt_version(rest)?;

    // remainder might be: "", "-mini", "-pro", "-codex", "-mini-2025-08-07", etc.
    let remainder = remainder.strip_prefix('-').unwrap_or(remainder);

    if remainder.is_empty() {
        return Some(ParsedGptId {
            major,
            minor,
            variant: None,
            date: None,
            is_chat_latest,
        });
    }

    // Try to separate variant from date
    // Date patterns: YYYY-MM-DD or MMDD
    let (variant, date) = split_variant_date(remainder);

    Some(ParsedGptId {
        major,
        minor,
        variant,
        date,
        is_chat_latest,
    })
}

fn parse_gpt_version(s: &str) -> Option<(u32, u32, &str)> {
    // Try "X.Y" first (e.g. "5.2-pro", "4.1-mini")
    if let Some(dot_pos) = s.find('.') {
        let major_str = &s[..dot_pos];
        let after_dot = &s[dot_pos + 1..];
        if let Ok(major) = major_str.parse::<u32>() {
            // Find where the minor version ends
            let minor_end = after_dot
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_dot.len());
            let minor_str = &after_dot[..minor_end];
            if let Ok(minor) = minor_str.parse::<u32>() {
                return Some((major, minor, &after_dot[minor_end..]));
            }
        }
    }

    // Try special versions: "4o" -> (4, special encoding)
    if let Some(rest) = s.strip_prefix("4o") {
        return Some((4, 100, rest)); // 4o gets minor=100 to sort above 4
    }

    // Try bare major: "5", "4", "3"
    let major_end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if major_end > 0
        && let Ok(major) = s[..major_end].parse::<u32>()
    {
        return Some((major, 0, &s[major_end..]));
    }

    None
}

/// Split "variant-YYYY-MM-DD" or "variant-MMDD" or just "variant" or just "YYYY-MM-DD"
fn split_variant_date(s: &str) -> (Option<String>, Option<String>) {
    if s.is_empty() {
        return (None, None);
    }

    // Check if the whole thing is a date (YYYY-MM-DD or MMDD)
    if is_date_string(s) {
        return (None, Some(s.to_string()));
    }

    // Try to find a date suffix at the end: "-YYYY-MM-DD" (11 chars with leading hyphen)
    if s.len() >= 11 {
        let candidate = &s[s.len() - 11..];
        if candidate.starts_with('-') && is_date_string(&candidate[1..]) {
            let variant = &s[..s.len() - 11];
            if variant.is_empty() {
                return (None, Some(candidate[1..].to_string()));
            }
            return (Some(variant.to_string()), Some(candidate[1..].to_string()));
        }
    }

    // Try bare YYYY-MM-DD (10 chars, the whole string)
    if s.len() == 10 && is_date_string(s) {
        return (None, Some(s.to_string()));
    }

    // Check for 4-digit date suffix: "-MMDD"
    if s.len() >= 5 {
        let tail = &s[s.len() - 4..];
        if tail.chars().all(|c| c.is_ascii_digit())
            && s.as_bytes()[s.len() - 5] == b'-'
            && !s[..s.len() - 5]
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_digit())
        {
            let variant = &s[..s.len() - 5];
            if variant.is_empty() {
                return (None, Some(tail.to_string()));
            }
            return (Some(variant.to_string()), Some(tail.to_string()));
        }
    }

    (Some(s.to_string()), None)
}

fn is_date_string(s: &str) -> bool {
    // YYYY-MM-DD
    if s.len() == 10
        && s.as_bytes()[4] == b'-'
        && s.as_bytes()[7] == b'-'
        && s[..4].chars().all(|c| c.is_ascii_digit())
        && s[5..7].chars().all(|c| c.is_ascii_digit())
        && s[8..].chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }
    // MMDD (4 digits)
    s.len() == 4 && s.chars().all(|c| c.is_ascii_digit())
}

/// GPT variant tier for sorting (lower = better).
pub(crate) fn gpt_variant_tier(variant: Option<&str>) -> u8 {
    match variant {
        Some("pro") => 0,
        None | Some("turbo") => 1,
        Some(v) if v.starts_with("turbo") => 1,
        Some(v) if v.contains("codex") => 2,
        Some("mini") => 3,
        Some("nano") => 4,
        Some(_) => 5,
    }
}

// ---------------------------------------------------------------------------
// OpenAI o-series
// ---------------------------------------------------------------------------

pub(crate) fn parse_o_series(id: &str) -> Option<ParsedOId> {
    let rest = id.strip_prefix('o')?;

    // Parse generation number
    let gen_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if gen_end == 0 {
        return None;
    }
    let generation = rest[..gen_end].parse::<u32>().ok()?;
    let remainder = &rest[gen_end..];

    if remainder.is_empty() {
        return Some(ParsedOId {
            generation,
            variant: None,
            date: None,
        });
    }

    let remainder = remainder.strip_prefix('-')?;
    let (variant, date) = split_variant_date(remainder);

    Some(ParsedOId {
        generation,
        variant,
        date,
    })
}

/// O-series variant tier for sorting (lower = better).
pub(crate) fn o_variant_tier(variant: Option<&str>) -> u8 {
    match variant {
        Some("pro") => 0,
        None => 1,
        Some("mini") => 2,
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// Gemini
// ---------------------------------------------------------------------------

pub(crate) fn parse_gemini(id: &str) -> Option<ParsedGeminiId> {
    // Handle "gemini-X.Y-tier[-suffix]" and "gemini-X-tier[-suffix]"
    let rest = id.strip_prefix("gemini-")?;

    // Parse version
    let (major, minor, after_version) = parse_gemini_version(rest)?;

    // After version, expect "-tier[-suffix]"
    let after_version = after_version.strip_prefix('-')?;

    // Extract tier
    let tier_end = after_version.find('-').unwrap_or(after_version.len());
    let tier = &after_version[..tier_end];
    let suffix = if tier_end < after_version.len() {
        &after_version[tier_end + 1..]
    } else {
        ""
    };

    Some(ParsedGeminiId {
        version_major: major,
        version_minor: minor,
        tier: tier.to_string(),
        suffix: suffix.to_string(),
    })
}

fn parse_gemini_version(s: &str) -> Option<(u32, u32, &str)> {
    // Try "X.Y" first
    if let Some(dot_pos) = s.find('.') {
        let major_str = &s[..dot_pos];
        if let Ok(major) = major_str.parse::<u32>() {
            let after_dot = &s[dot_pos + 1..];
            let minor_end = after_dot
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(after_dot.len());
            if let Ok(minor) = after_dot[..minor_end].parse::<u32>() {
                return Some((major, minor, &after_dot[minor_end..]));
            }
        }
    }

    // Try bare major: "3", "2"
    let major_end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if major_end > 0
        && let Ok(major) = s[..major_end].parse::<u32>()
    {
        return Some((major, 0, &s[major_end..]));
    }

    None
}

/// Gemini tier for sorting (lower = better).
pub(crate) fn gemini_tier(tier: &str) -> u8 {
    match tier {
        "pro" => 0,
        "flash" => 1,
        _ if tier.starts_with("flash") => 2, // flash-lite, flash-image etc
        _ => 3,
    }
}

// ---------------------------------------------------------------------------
// Gemma (served via Gemini provider)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Mistral
// ---------------------------------------------------------------------------

pub(crate) fn parse_mistral(id: &str) -> ParsedMistralId {
    let is_latest = id.ends_with("-latest");
    let base = id.strip_suffix("-latest").unwrap_or(id);

    // Strip date suffix (e.g. "-2512", "-2501", "-2506")
    let base = strip_mistral_date(base);

    ParsedMistralId {
        family: base.to_string(),
        is_latest,
    }
}

fn strip_mistral_date(s: &str) -> &str {
    // Strip trailing "-YYMM" (4 digits after last hyphen)
    if let Some(last_hyphen) = s.rfind('-') {
        let tail = &s[last_hyphen + 1..];
        if tail.len() == 4 && tail.chars().all(|c| c.is_ascii_digit()) {
            return &s[..last_hyphen];
        }
    }
    s
}

/// Mistral family tier for sorting (lower = better).
pub(crate) fn mistral_family_tier(family: &str) -> u8 {
    match family {
        "mistral-large" => 0,
        "mistral-medium" => 1,
        "pixtral-large" | "mistral-large-pixtral" => 2,
        "codestral" => 3,
        "devstral-medium" | "devstral" => 4,
        "magistral-medium" => 5,
        "mistral-small" => 6,
        "devstral-small" => 7,
        "magistral-small" => 8,
        "ministral-14b" => 9,
        "ministral-8b" => 10,
        "ministral-3b" => 11,
        "mistral-tiny" => 12,
        "mistral-vibe-cli" | "mistral-vibe-cli-with-tools" => 13,
        _ if family.starts_with("open-mistral") => 13,
        _ if family.starts_with("voxtral") => 14,
        _ if family.starts_with("labs-") => 15,
        _ => 16,
    }
}

/// Extract the base family name from a Mistral family string,
/// stripping size/variant suffixes like -large, -medium, -small, -tiny, -Nb.
///
/// Compound family names like `open-mistral-nemo` and `mistral-vibe-cli` are
/// returned as-is since they have no meaningful shorter form.
pub(crate) fn mistral_base_family(family: &str) -> &str {
    // Known compound families that should not be shortened
    if family.starts_with("open-mistral") || family.starts_with("mistral-vibe") {
        return family;
    }
    // Strip known size suffixes
    for suffix in &["-large", "-medium", "-small", "-tiny"] {
        if let Some(base) = family.strip_suffix(suffix) {
            return base;
        }
    }
    // Strip -Nb (e.g. -14b, -8b, -3b)
    if let Some(pos) = family.rfind('-') {
        let tail = &family[pos + 1..];
        if let Some(digits) = tail.strip_suffix('b')
            && !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit())
        {
            return &family[..pos];
        }
    }
    family
}

// ---------------------------------------------------------------------------
// Unified parse entry point
// ---------------------------------------------------------------------------

/// Parse any model ID based on its provider.
pub(crate) fn parse_model_id(provider: &str, id: &str) -> ParsedId {
    match provider {
        "anthropic" => parse_anthropic(id).map_or(ParsedId::Unknown, ParsedId::Anthropic),
        "openai" => {
            if id.starts_with("gpt-") {
                parse_gpt(id).map_or(ParsedId::Unknown, ParsedId::Gpt)
            } else if id.starts_with('o') && id.chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
                parse_o_series(id).map_or(ParsedId::Unknown, ParsedId::OSeries)
            } else {
                ParsedId::Unknown
            }
        }
        "gemini" => {
            if id.starts_with("gemma-") {
                ParsedId::Unknown // Gemma models sort after Gemini
            } else if id.starts_with("gemini-") {
                parse_gemini(id).map_or(ParsedId::Unknown, ParsedId::Gemini)
            } else {
                ParsedId::Unknown
            }
        }
        "mistral" => ParsedId::Mistral(parse_mistral(id)),
        _ => ParsedId::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Sort key
// ---------------------------------------------------------------------------

/// Composite sort key: lower = better/first.
///
/// Returns `(family_tier, version_desc, variant_tier, capability_desc,
///           context_desc, not_latest, cost_desc)`.
///
/// Descending fields are negated (for u32) or inverted (for f64) so that
/// ascending sort on the tuple gives the desired ordering.
pub(crate) type SortKey = (u8, u64, u8, u8, u64, u8, u64);

#[allow(clippy::too_many_lines)]
pub(crate) fn sort_key(provider: &str, id: &str, model: &super::ModelInfo) -> SortKey {
    let parsed = parse_model_id(provider, id);

    let capability_score = u8::from(model.supports_tools)
        + u8::from(model.supports_vision)
        + u8::from(model.supports_reasoning);
    let capability_desc = 3 - capability_score;
    let context_desc = u64::MAX - model.context_window;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cost_desc = model
        .output_cost_per_million
        .map_or(u64::MAX, |c| u64::MAX - (c * 1_000_000.0) as u64);

    match parsed {
        ParsedId::Anthropic(p) => {
            let family_tier = anthropic_family_tier(&p.family);
            // Higher version = better, so negate for ascending sort
            let version_desc = u64::MAX - (u64::from(p.major) * 1000 + u64::from(p.minor));
            let not_latest: u8 = u8::from(p.date.is_some());
            (
                family_tier,
                version_desc,
                0,
                capability_desc,
                context_desc,
                not_latest,
                cost_desc,
            )
        }
        ParsedId::Gpt(p) => {
            let version_desc = u64::MAX - (u64::from(p.major) * 1000 + u64::from(p.minor));
            let variant_tier = gpt_variant_tier(p.variant.as_deref());
            let not_latest: u8 = u8::from(p.date.is_some());
            let chat_latest_penalty: u8 = u8::from(p.is_chat_latest);
            (
                0, // GPT family tier handled via version
                version_desc,
                variant_tier,
                capability_desc,
                context_desc,
                not_latest.max(chat_latest_penalty),
                cost_desc,
            )
        }
        ParsedId::OSeries(p) => {
            // o-series sorts after gpt-5.x but before gpt-4.x
            // Use family_tier = 1 to put after gpt (family_tier 0)
            let gen_desc = u64::MAX - u64::from(p.generation);
            let variant_tier = o_variant_tier(p.variant.as_deref());
            let not_latest: u8 = u8::from(p.date.is_some());
            (
                1,
                gen_desc,
                variant_tier,
                capability_desc,
                context_desc,
                not_latest,
                cost_desc,
            )
        }
        ParsedId::Gemini(p) => {
            let tier = gemini_tier(&p.tier);
            let version_desc =
                u64::MAX - (u64::from(p.version_major) * 1000 + u64::from(p.version_minor));
            // Penalize preview/experimental suffixes
            let suffix_penalty: u8 =
                u8::from(p.suffix.contains("preview") || p.suffix.contains("exp"));
            (
                tier,
                version_desc,
                suffix_penalty,
                capability_desc,
                context_desc,
                0, // Gemini has no dated variants; suffix_penalty handles preview/exp
                cost_desc,
            )
        }
        ParsedId::Mistral(p) => {
            let family_tier = mistral_family_tier(&p.family);
            // Prefer -latest over dated
            let latest_pref: u8 = u8::from(!p.is_latest);
            (
                family_tier,
                0,
                0,
                capability_desc,
                context_desc,
                latest_pref,
                cost_desc,
            )
        }
        ParsedId::Unknown => {
            // Unknown models sort last; extend heuristic to catch MMDD dates
            let has_date = id.contains("-202")
                || id.rfind('-').is_some_and(|pos| {
                    let tail = &id[pos + 1..];
                    tail.len() == 4 && tail.chars().all(|c| c.is_ascii_digit())
                });
            let not_latest: u8 = u8::from(has_date && !id.ends_with("-latest"));
            (
                u8::MAX,
                0,
                0,
                capability_desc,
                context_desc,
                not_latest,
                cost_desc,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_anthropic_new_format() {
        let p = parse_anthropic("claude-opus-4-6").expect("should parse");
        assert_eq!(p.family, "opus");
        assert_eq!(p.major, 4);
        assert_eq!(p.minor, 6);
        assert!(p.date.is_none());
    }

    #[test]
    fn parse_anthropic_new_format_with_date() {
        let p = parse_anthropic("claude-sonnet-4-5-20250929").expect("should parse");
        assert_eq!(p.family, "sonnet");
        assert_eq!(p.major, 4);
        assert_eq!(p.minor, 5);
        assert_eq!(p.date.as_deref(), Some("20250929"));
    }

    #[test]
    fn parse_anthropic_old_format() {
        let p = parse_anthropic("claude-3-7-sonnet-20250219").expect("should parse");
        assert_eq!(p.family, "sonnet");
        assert_eq!(p.major, 3);
        assert_eq!(p.minor, 7);
        assert_eq!(p.date.as_deref(), Some("20250219"));
    }

    #[test]
    fn parse_anthropic_bare_major() {
        let p = parse_anthropic("claude-opus-4-20250514").expect("should parse");
        assert_eq!(p.family, "opus");
        assert_eq!(p.major, 4);
        assert_eq!(p.minor, 0);
        assert_eq!(p.date.as_deref(), Some("20250514"));
    }

    #[test]
    fn parse_gpt_5_2() {
        let p = parse_gpt("gpt-5.2").expect("should parse");
        assert_eq!(p.major, 5);
        assert_eq!(p.minor, 2);
        assert!(p.variant.is_none());
        assert!(p.date.is_none());
    }

    #[test]
    fn parse_gpt_5_2_pro_dated() {
        let p = parse_gpt("gpt-5.2-pro-2025-12-11").expect("should parse");
        assert_eq!(p.major, 5);
        assert_eq!(p.minor, 2);
        assert_eq!(p.variant.as_deref(), Some("pro"));
        assert_eq!(p.date.as_deref(), Some("2025-12-11"));
    }

    #[test]
    fn parse_gpt_4o() {
        let p = parse_gpt("gpt-4o").expect("should parse");
        assert_eq!(p.major, 4);
        assert_eq!(p.minor, 100); // special encoding for "o"
        assert!(p.variant.is_none());
    }

    #[test]
    fn parse_gpt_4o_mini() {
        let p = parse_gpt("gpt-4o-mini").expect("should parse");
        assert_eq!(p.major, 4);
        assert_eq!(p.minor, 100);
        assert_eq!(p.variant.as_deref(), Some("mini"));
    }

    #[test]
    fn parse_gpt_chat_latest() {
        let p = parse_gpt("gpt-5-chat-latest").expect("should parse");
        assert_eq!(p.major, 5);
        assert!(p.is_chat_latest);
    }

    #[test]
    fn parse_gpt_3_5_turbo() {
        let p = parse_gpt("gpt-3.5-turbo").expect("should parse");
        assert_eq!(p.major, 3);
        assert_eq!(p.minor, 5);
        assert_eq!(p.variant.as_deref(), Some("turbo"));
    }

    #[test]
    fn parse_o_series_o3() {
        let p = parse_o_series("o3").expect("should parse");
        assert_eq!(p.generation, 3);
        assert!(p.variant.is_none());
    }

    #[test]
    fn parse_o_series_o3_pro() {
        let p = parse_o_series("o3-pro").expect("should parse");
        assert_eq!(p.generation, 3);
        assert_eq!(p.variant.as_deref(), Some("pro"));
    }

    #[test]
    fn parse_o_series_o4_mini_dated() {
        let p = parse_o_series("o4-mini-2025-04-16").expect("should parse");
        assert_eq!(p.generation, 4);
        assert_eq!(p.variant.as_deref(), Some("mini"));
        assert_eq!(p.date.as_deref(), Some("2025-04-16"));
    }

    #[test]
    fn parse_gemini_2_5_pro() {
        let p = parse_gemini("gemini-2.5-pro").expect("should parse");
        assert_eq!(p.version_major, 2);
        assert_eq!(p.version_minor, 5);
        assert_eq!(p.tier, "pro");
    }

    #[test]
    fn parse_gemini_3_pro_preview() {
        let p = parse_gemini("gemini-3-pro-preview").expect("should parse");
        assert_eq!(p.version_major, 3);
        assert_eq!(p.version_minor, 0);
        assert_eq!(p.tier, "pro");
        assert_eq!(p.suffix, "preview");
    }

    #[test]
    fn parse_gemini_flash_lite() {
        let p = parse_gemini("gemini-2.5-flash-lite").expect("should parse");
        assert_eq!(p.version_major, 2);
        assert_eq!(p.version_minor, 5);
        assert_eq!(p.tier, "flash");
        assert_eq!(p.suffix, "lite");
    }

    #[test]
    fn parse_mistral_large_latest() {
        let p = parse_mistral("mistral-large-latest");
        assert_eq!(p.family, "mistral-large");
        assert!(p.is_latest);
    }

    #[test]
    fn parse_mistral_dated() {
        let p = parse_mistral("mistral-large-2512");
        assert_eq!(p.family, "mistral-large");
        assert!(!p.is_latest);
    }

    #[test]
    fn parse_devstral_medium() {
        let p = parse_mistral("devstral-medium-latest");
        assert_eq!(p.family, "devstral-medium");
        assert!(p.is_latest);
    }

    #[test]
    fn mistral_base_family_extraction() {
        assert_eq!(mistral_base_family("mistral-large"), "mistral");
        assert_eq!(mistral_base_family("mistral-small"), "mistral");
        assert_eq!(mistral_base_family("mistral-tiny"), "mistral");
        assert_eq!(mistral_base_family("devstral-medium"), "devstral");
        assert_eq!(mistral_base_family("magistral-small"), "magistral");
        assert_eq!(mistral_base_family("ministral-8b"), "ministral");
        assert_eq!(mistral_base_family("ministral-14b"), "ministral");
        assert_eq!(mistral_base_family("codestral"), "codestral");
        assert_eq!(mistral_base_family("pixtral-large"), "pixtral");
        assert_eq!(
            mistral_base_family("open-mistral-nemo"),
            "open-mistral-nemo"
        );
        assert_eq!(mistral_base_family("mistral-vibe-cli"), "mistral-vibe-cli");
    }
}

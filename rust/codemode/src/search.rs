use crate::modules::{SnapshotTool, ToolSnapshot, tool_to_definition};
use crate::types::{SearchResultEntry, SearchResults, SearchToolsOptions};

/// Default maximum number of search results.
const DEFAULT_LIMIT: usize = 50;

/// Minimum fuzzy similarity score to include a result (0.0–1.0).
const FUZZY_THRESHOLD: f64 = 0.7;

/// A candidate tool with its score, server ID, and tool index.
type ScoredCandidate<'a> = (f64, &'a str, usize);

/// Search tools across all servers using substring matching with fuzzy fallback.
///
/// First attempts case-insensitive substring matching on tool name and description.
/// Only if no substring matches are found, falls back to Jaro-Winkler fuzzy matching
/// on the entire candidate set.
/// Results are sorted by relevance (name matches first, then description matches).
pub(crate) fn search_tools(
    snapshot: &ToolSnapshot,
    query: &str,
    options: &SearchToolsOptions,
) -> SearchResults {
    let query_lower = query.to_lowercase();
    let detail = options.detail.unwrap_or_default();
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT);

    // Pass 1: substring matching
    let mut scored: Vec<ScoredCandidate<'_>> = Vec::new();
    let candidates = filtered_candidates(snapshot, options);

    for &(server_id, tool_idx, ref name_lower, ref desc_lower) in &candidates {
        if name_lower.contains(&query_lower) {
            scored.push((1.0, server_id, tool_idx));
        } else if desc_lower.contains(&query_lower) {
            scored.push((0.9, server_id, tool_idx));
        }
    }

    // Pass 2: fuzzy fallback only when no substring matches found
    if scored.is_empty() {
        for &(server_id, tool_idx, ref name_lower, ref desc_lower) in &candidates {
            let name_sim = strsim::jaro_winkler(&query_lower, name_lower);
            let desc_sim = if desc_lower.is_empty() {
                0.0
            } else {
                strsim::jaro_winkler(&query_lower, desc_lower)
            };
            let score = name_sim.max(desc_sim);
            if score >= FUZZY_THRESHOLD {
                scored.push((score, server_id, tool_idx));
            }
        }
    }

    // Sort by score descending (stable sort preserves order for equal scores)
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Limit results
    scored.truncate(limit);

    // Convert to SearchResultEntry
    let results = scored
        .into_iter()
        .filter_map(|(_, server_id, tool_idx)| {
            let server = snapshot
                .servers
                .iter()
                .find(|s| s.normalized_id == server_id)?;
            let tool = server.tools.get(tool_idx)?;
            Some(to_search_entry(server_id, tool, detail))
        })
        .collect();

    SearchResults {
        query: query.to_string(),
        results,
    }
}

/// Build the list of candidate tools with pre-computed lowercase strings.
fn filtered_candidates<'a>(
    snapshot: &'a ToolSnapshot,
    options: &SearchToolsOptions,
) -> Vec<(&'a str, usize, String, String)> {
    let mut candidates = Vec::new();
    for server in &snapshot.servers {
        if let Some(ref sid) = options.server_id
            && server.normalized_id != *sid
        {
            continue;
        }
        for (tool_idx, tool) in server.tools.iter().enumerate() {
            let name_lower = tool.name.to_lowercase();
            let desc_lower = tool.description.as_deref().unwrap_or("").to_lowercase();
            candidates.push((&*server.normalized_id, tool_idx, name_lower, desc_lower));
        }
    }
    candidates
}

/// Create a `SearchResultEntry` from a snapshot tool, delegating detail-level
/// projection to the shared `tool_to_definition`.
fn to_search_entry(
    server_id: &str,
    tool: &SnapshotTool,
    detail: crate::types::DetailLevel,
) -> SearchResultEntry {
    let def = tool_to_definition(tool, detail);
    SearchResultEntry {
        server_id: server_id.to_string(),
        tool_name: def.tool_name,
        export_name: def.export_name,
        description: def.description,
        annotations: def.annotations,
        input_schema: def.input_schema,
        output_schema: def.output_schema,
    }
}

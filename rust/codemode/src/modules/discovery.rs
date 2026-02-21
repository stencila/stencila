/// JavaScript source for the `@codemode/discovery` module.
///
/// Exports per spec §4.1:
/// - `specVersion` — semver string (§12.1)
/// - `listServers()` — returns `Promise<ServerInfo[]>`
/// - `describeServer(serverId)` — returns `Promise<ServerDescription>`
/// - `listTools(serverId, options?)` — returns `Promise<ToolDefinition[]>`
/// - `getTool(serverId, toolName)` — returns `Promise<ToolDefinition>`
/// - `searchTools(query, options?)` — returns `Promise<SearchResults>`
///
/// All functions call host bridge methods on `globalThis.__codemode_internal__`
/// which return JSON strings. The module parses the JSON and wraps results
/// in resolved promises (via `async function`).
pub(crate) const JS_SOURCE: &str = r#"
import {
    ServerNotFoundError,
    ToolNotFoundError,
} from "@codemode/errors";

const __internal__ = globalThis.__codemode_internal__;

export const specVersion = "0.1.0";

export async function listServers() {
    return JSON.parse(__internal__.listServers());
}

export async function describeServer(serverId) {
    const json = __internal__.describeServer(serverId);
    const result = JSON.parse(json);
    if (result === null) {
        throw new ServerNotFoundError(
            "Server not found: " + serverId,
            { serverId, hint: "Use listServers() to see available servers." }
        );
    }
    return result;
}

export async function listTools(serverId, options) {
    const detail = (options && options.detail) || "description";
    const json = __internal__.listTools(serverId, detail);
    const result = JSON.parse(json);
    if (result === null) {
        throw new ServerNotFoundError(
            "Server not found: " + serverId,
            { serverId, hint: "Use listServers() to see available servers." }
        );
    }
    return result;
}

export async function getTool(serverId, toolName) {
    const json = __internal__.getTool(serverId, toolName);
    const result = JSON.parse(json);
    if (result && result.error === "server_not_found") {
        throw new ServerNotFoundError(
            "Server not found: " + serverId,
            { serverId, hint: "Use listServers() to see available servers." }
        );
    }
    if (result && result.error === "tool_not_found") {
        throw new ToolNotFoundError(
            "Tool '" + toolName + "' not found on server '" + serverId + "'",
            { serverId, toolName, hint: "Use listTools('" + serverId + "') to see available tools." }
        );
    }
    return result;
}

export async function searchTools(query, options) {
    const optsJson = options ? JSON.stringify(options) : "{}";
    return JSON.parse(__internal__.searchTools(query, optsJson));
}
"#;

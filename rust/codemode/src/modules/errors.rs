/// JavaScript source for the `@codemode/errors` module.
///
/// Defines the error class hierarchy per spec §7.1:
/// - `CodemodeError` — base class (extends Error)
/// - `SchemaValidationError` — input failed schema validation (§7.2)
/// - `ToolNotFoundError` — tool does not exist on server
/// - `ServerNotFoundError` — server not connected
/// - `ToolCallError` — wraps MCP `isError` results
/// - `AuthenticationError` — credential rejection
/// - `SandboxLimitError` — sandbox limit exceeded
///
/// Each error includes a `hint` property per §7.3.
pub(crate) const JS_SOURCE: &str = r#"
export class CodemodeError extends Error {
    constructor(message, hint) {
        super(message);
        this.name = 'CodemodeError';
        this.hint = hint || null;
    }
}

export class SchemaValidationError extends CodemodeError {
    constructor(message, { toolName, exportName, path, expected, received, hint } = {}) {
        super(message, hint);
        this.name = 'SchemaValidationError';
        this.toolName = toolName || null;
        this.exportName = exportName || null;
        this.path = path || null;
        this.expected = expected || null;
        this.received = received || null;
    }
}

export class ToolNotFoundError extends CodemodeError {
    constructor(message, { serverId, toolName, hint } = {}) {
        super(message, hint);
        this.name = 'ToolNotFoundError';
        this.serverId = serverId || null;
        this.toolName = toolName || null;
    }
}

export class ServerNotFoundError extends CodemodeError {
    constructor(message, { serverId, hint } = {}) {
        super(message, hint);
        this.name = 'ServerNotFoundError';
        this.serverId = serverId || null;
    }
}

export class ToolCallError extends CodemodeError {
    constructor(message, { serverId, toolName, hint } = {}) {
        super(message, hint);
        this.name = 'ToolCallError';
        this.serverId = serverId || null;
        this.toolName = toolName || null;
    }
}

export class AuthenticationError extends CodemodeError {
    constructor(message, { serverId, hint } = {}) {
        super(message, hint);
        this.name = 'AuthenticationError';
        this.serverId = serverId || null;
    }
}

export class SandboxLimitError extends CodemodeError {
    constructor(message, { kind, hint } = {}) {
        super(message, hint);
        this.name = 'SandboxLimitError';
        this.kind = kind || null;
    }
}
"#;

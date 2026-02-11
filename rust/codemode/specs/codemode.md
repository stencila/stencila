# Code Mode for MCP

## 0. Background (Non-normative)

"Code Mode" is an interaction pattern where the model is primarily asked to **write and run JavaScript** that orchestrates external capabilities, instead of emitting one JSON tool call per step.

The host exposes **one execution tool** ("run code") and makes MCP servers available as **discoverable, typed code APIs**.

This approach is described publicly by:

* Cloudflare ([Code Mode](https://blog.cloudflare.com/code-mode/): convert MCP tools into a TypeScript API and ask the LLM to write code against it)
* Anthropic ([Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp): progressive disclosure, filesystem-like exploration, and search)

This document defines a **public, agent-facing API** for Code Mode.

---

## 1. Purpose, scope, and non-goals

### 1.1 Purpose

This specification defines:

* The **single-tool** execution interface exposed to an agent.
* The **in-sandbox JavaScript API** used by agent-authored code.
* The **discovery** and **type-hint** mechanisms used to present MCP tools as **high-quality, agent-usable function definitions**.
* The **error model** and diagnostics required for fast self-correction.

### 1.2 Scope

This specification is concerned with **agent-visible behavior** and interoperability.

### 1.3 Non-goals (Out of scope for this version)

The following are explicitly out of scope for this version of the specification:

* Runtime embedding details (e.g., QuickJS configuration).
* Transport details for connecting to MCP servers.
* Authentication, authorization, and credential storage.
* Persistence and storage backends.
* **Concurrency model** (parallel `codemode.run` calls).
* MCP "resources" and "prompts" support (beyond tools).
* UI/UX rendering of logs and traces.

Hosts MAY support any of the above, but they are not required for conformance.

### 1.4 Sandbox lifetime

Each `codemode.run` invocation MUST start with a fresh sandbox. No state from a previous invocation is carried over.

---

## 2. Overview

1. The agent is exposed to **exactly one** outer tool: `codemode.run`.
2. `codemode.run` executes JavaScript in an isolated sandbox.
3. MCP servers are exposed inside the sandbox as:

   * Discoverable server/tool metadata, and
   * Generated modules that map tools to async functions.
4. The sandbox provides **discovery APIs** (list, get, search) for server and tool metadata.

---

## 3. Outer tool: `codemode.run`

### 3.1 Purpose

`codemode.run` is the only tool exposed to the agent in the outer loop. The agent uses it to execute code that calls MCP tools through the in-sandbox API.

### 3.2 Request

A `codemode.run` request MUST be a JSON object with the following fields:

* `code` (**REQUIRED**, string): JavaScript source text.
* `limits` (**OPTIONAL**, object): Host-defined execution limits.
* `requestedCapabilities` (**OPTIONAL**, array of strings): Coarse capability hints.

#### 3.2.1 Language level

The sandbox MUST execute `code` with **ES module semantics**, including:

* `import` / `export`
* **top-level `await`**

A host MUST document any additional restrictions (e.g., disabled syntax features) in its system prompt.

#### 3.2.2 Result value

The sandbox MUST define a global helper:

```js
globalThis.__codemode_result__
```

At the end of execution, the host MUST set the `result` field in the response to:

* The final value assigned to `globalThis.__codemode_result__`, if any, otherwise
* `null`.

Agent-authored code SHOULD set this explicitly when a structured return value is desired:

```js
globalThis.__codemode_result__ = { ok: true, count };
```

This mechanism is required to avoid ambiguity around module completion values.

#### 3.2.3 `requestedCapabilities`

If provided, `requestedCapabilities` MUST be treated as **lazy-connection hints**. A host:

* SHOULD attempt to ensure the requested servers are available.
* SHOULD emit an eager diagnostic if a requested capability is unavailable.
* MUST NOT silently claim capabilities it cannot provide.
* MUST return a structured error if a requested capability is unavailable and the script attempts to use it.

A host MUST document supported capability identifiers in its system prompt.

#### 3.2.4 `limits`

A host:

* SHOULD support at least: `timeoutMs` (integer, milliseconds), `maxMemoryBytes` (integer, bytes), `maxLogBytes` (integer, bytes), and `maxToolCalls` (integer, count).
* MUST ignore unrecognized limit keys without error.
* MUST document supported limit keys in its system prompt.

### 3.3 Response

A `codemode.run` response MUST be a JSON object with the following fields:

* `logs` (**REQUIRED**, array): Captured console events.
* `result` (**REQUIRED**): A JSON-serializable value. Defaults to `null` if the script did not assign `globalThis.__codemode_result__`.
* `diagnostics` (**REQUIRED**, array): Structured diagnostics (see §3.3.3).
* `toolTrace` (**OPTIONAL**, array): A redacted tool-call trace.

#### 3.3.1 `logs`

Each log entry MUST be an object:

* `level`: one of `"debug" | "log" | "warn" | "error"`
* `message`: string
* `timeMs`: integer milliseconds since sandbox start

##### Console serialization

The host MUST capture all console calls with the following rule:

* All arguments passed to `console.*` MUST be serialized and concatenated into a single `message` string.
* For primitives, the host MUST use JavaScript `String(value)` semantics.
* For objects and arrays, the host MUST use a stable JSON serialization.

  * If serialization fails (e.g., circular structure), the host MUST fall back to a best-effort string such as `"[Unserializable Object]"`.

If logs are truncated, the host MUST append a final log entry:

* `level: "warn"`
* `message` indicating truncation and the applied limit

#### 3.3.2 `toolTrace`

If present, each trace entry MUST be an object:

* `serverId`: string
* `toolName`: string (canonical MCP tool name)
* `durationMs`: integer
* `ok`: boolean
* `error` (optional): string summary

The `toolTrace` MUST NOT include tool inputs or outputs.

#### 3.3.3 `diagnostics`

Each diagnostic entry MUST be an object with the following fields:

* `severity` (string, REQUIRED): one of `"error" | "warning" | "info"`
* `code` (string, REQUIRED): a machine-readable error code (e.g., `"SYNTAX_ERROR"`, `"UNCAUGHT_EXCEPTION"`, `"IMPORT_FAILURE"`, `"SANDBOX_LIMIT"`)
* `message` (string, REQUIRED): human-readable description
* `hint` (string, OPTIONAL): a single recommended corrective action
* `path` (string, OPTIONAL): source location or JSON Pointer relevant to the error
* `errorClass` (string, OPTIONAL): the `@codemode/errors` class name, if applicable (e.g., `"SchemaValidationError"`)

#### 3.3.4 Fatal script errors

`codemode.run` MUST always return a structured response, even when the script fails to execute. Syntax errors, uncaught exceptions, and import failures MUST NOT cause the outer tool call itself to error. Instead:

* The host MUST populate `diagnostics` with at least one entry describing the failure.
* `result` MUST be `null`.
* `logs` MUST include any console output captured before the failure.
* `toolTrace` SHOULD include any tool calls completed before the failure.

### 3.4 Isolation requirements

A conforming host MUST ensure:

* No ambient network access from within the sandbox.
* External side effects occur only through MCP tool calls.

### 3.5 Sandbox globals

The sandbox MUST provide the following globals:

* **Standard built-ins**: `JSON`, `Math`, `Date`, `URL`, `URLSearchParams`, `Promise`, `Map`, `Set`, `WeakMap`, `WeakSet`, `Symbol`, `Proxy`, `Reflect`, `RegExp`, `Error`, `Array`, `Object`, `String`, `Number`, `Boolean`, `BigInt`, `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `Infinity`, `NaN`, `undefined`
* **Text encoding**: `TextEncoder`, `TextDecoder`
* **Typed arrays**: `ArrayBuffer`, `DataView`, `Uint8Array`, `Int8Array`, `Uint16Array`, `Int16Array`, `Uint32Array`, `Int32Array`, `Float32Array`, `Float64Array`
* **Timers**: `setTimeout`, `clearTimeout`
* **Console**: `console.log`, `console.debug`, `console.warn`, `console.error`
* **Result**: `globalThis.__codemode_result__`

The sandbox MUST NOT provide:

* `fetch`, `XMLHttpRequest`, `WebSocket`, or any network API
* `setInterval` (use `setTimeout` for single delays; polling loops are discouraged)
* `eval`, `Function` constructor with string arguments
* `process`, `require`, or Node.js-specific globals

A host MUST document any deviations from this list in its system prompt.

---

## 4. In-sandbox standard library

The sandbox MUST provide a stable set of built-in modules under `@codemode/*`.

### 4.1 `@codemode/discovery`

To optimize for agent ergonomics, discovery APIs MUST be available through a single module.

The module MUST export:

* `specVersion: string`
* `listServers(): Promise<ServerInfo[]>`
* `describeServer(serverId: string): Promise<ServerDescription>`
* `listTools(serverId: string, options?: ListToolsOptions): Promise<ToolDefinition[]>`
* `getTool(serverId: string, toolName: string): Promise<ToolDefinition>`
* `searchTools(query: string, options?: SearchToolsOptions): Promise<SearchResults>`

Fields returned on each `ToolDefinition` depend on the `detail` level (see §4.3). At lower detail levels, optional fields (`description`, `annotations`, `inputSchema`, `outputSchema`) are omitted from the response.

### 4.2 Core data types

The following types are part of the public API and MUST be supported.

#### 4.2.1 `ServerInfo`

* `serverId` (string, REQUIRED)
* `serverName` (string, REQUIRED)
* `capabilities` (string[], OPTIONAL)

#### 4.2.2 `ServerDescription`

* All fields of `ServerInfo`
* `description` (string, OPTIONAL)
* `version` (string, OPTIONAL)

#### 4.2.3 `ToolSummary`

* `toolName` (string, REQUIRED)
* `exportName` (string, REQUIRED)
* `description` (string, OPTIONAL)
* `annotations` (object, OPTIONAL)

#### 4.2.4 `ToolDefinition`

* All fields of `ToolSummary`
* `inputSchema` (object, OPTIONAL)
* `outputSchema` (object, OPTIONAL)

#### 4.2.5 `SearchResults`

* `query` (string, REQUIRED)
* `results` (array of `ToolDefinition` extended with `serverId`, REQUIRED). Each result object includes `serverId` (string, REQUIRED) in addition to the `ToolDefinition` fields. Fields returned depend on the `detail` level (see §4.3).

#### 4.2.6 `ListToolsOptions`

* `detail` (string, OPTIONAL): one of `"name" | "description" | "full"`. Defaults to `"description"`.

#### 4.2.7 `SearchToolsOptions`

* `detail` (string, OPTIONAL): one of `"name" | "description" | "full"`. Defaults to `"description"`.
* `serverId` (string, OPTIONAL): restrict search to a single server.
* `limit` (integer, OPTIONAL): maximum number of results. Defaults to host-defined maximum.

### 4.3 Discovery detail levels

`searchTools` and `listTools` MUST support `detail`:

* `"name"` — returns only `toolName` and `exportName`.
* `"description"` — includes `description` and `annotations`.
* `"full"` — includes `inputSchema` and `outputSchema`.

### 4.4 Type information

A host MUST inject TypeScript type declarations for connected servers into the agent context (e.g., the system prompt or a pre-populated file).

The injected declarations MUST:

* Cover all exported tool bindings for each connected server.
* Follow the generation rules in §6.2.

A host MAY defer injection until a server is first referenced in a `requestedCapabilities` hint or imported by agent code.

### 4.5 Errors

The sandbox MUST provide `@codemode/errors` defining standardized error classes (see §7).

---

## 5. Generated per-server modules

For each connected MCP server, the sandbox MUST expose an importable module under `@codemode/servers/`:

```js
import * as gdrive from "@codemode/servers/google-drive";
```

#### 5.0.1 Module path mapping

The `serverId` MUST be mapped to a module path segment using the following rules:

* Characters outside `[a-z0-9-]` MUST be replaced with `-`.
* Uppercase letters MUST be lowercased before the replacement step.
* Consecutive `-` characters MUST be collapsed to a single `-`.
* Leading and trailing `-` MUST be stripped.
* If two servers produce the same module path after normalization, the host MUST disambiguate by appending `--N` (where `N` starts at `2`; the first occurrence keeps the clean path).

The normalized path MUST be reflected in `__meta__.serverId` and discovery results.

### 5.1 Exports

A per-server module MUST export:

* One async function per MCP tool.
* A `__meta__` export.

### 5.2 `__meta__` schema

`__meta__` MUST be an object:

* `serverId`: string
* `serverName`: string
* `serverVersion` (optional): string
* `tools`: array of `{ toolName, exportName, description? }`

### 5.3 Call shape and unwrapping rules

Each tool binding MUST be callable as:

```js
await server.toolName(input)
```

#### 5.3.1 Input

* If the MCP tool `inputSchema` is an object schema, the binding MUST accept one argument `input`.
* If the MCP tool `inputSchema` is empty or absent, the binding MUST accept either:

  * no arguments, OR
  * `{}`.
* If the MCP tool `inputSchema` has a non-object root type (e.g., array, string), the binding MUST accept one argument `input` of that type. Hosts SHOULD document this edge case, as MCP tools overwhelmingly use object schemas.

#### 5.3.2 Output

Bindings MUST return the MCP tool result **unwrapped** into a JSON value according to the following rules, applied in order:

1. If the MCP result contains `structuredContent`, return the `structuredContent` value. The `content` array, if also present, MUST be ignored.
2. If the MCP result is a single `content` block of type `"text"`, return that text string.
3. If the MCP result contains `"image"` or `"audio"` content blocks, return the full MCP result object. Binary payloads within content blocks MUST be represented as base64-encoded strings.
4. Otherwise return the full MCP result object.

A host MUST document these unwrapping rules in its system prompt.

---

## 6. Schema-to-function-definition generation

### 6.1 Identifier mapping

Each MCP tool name MUST map to a stable JavaScript export identifier.

* Illegal identifier characters MUST be replaced with `_`.
* If the resulting identifier starts with a digit, the host MUST prepend `_` (e.g., `123tool` becomes `_123tool`).
* If the resulting identifier is a JavaScript reserved word (`break`, `case`, `class`, `const`, `continue`, `debugger`, `default`, `delete`, `do`, `else`, `export`, `extends`, `false`, `finally`, `for`, `function`, `if`, `import`, `in`, `instanceof`, `new`, `null`, `return`, `super`, `switch`, `this`, `throw`, `true`, `try`, `typeof`, `var`, `void`, `while`, `with`, `yield`, `let`, `static`, `await`), the host MUST append `_` to the identifier.
* If a collision occurs after the above transformations, the host MUST disambiguate by appending `__N`.

  * `N` MUST start at `2`.
  * The first occurrence MUST keep the clean name.
* The collision ordering MUST be deterministic.

  * The host MUST order tools alphabetically by canonical MCP tool name before assigning identifiers.

The mapping MUST be reflected in `__meta__`.

### 6.2 TypeScript declaration generation

A host MUST generate `.d.ts` definitions for tools using MCP schemas.

The mapping MUST correctly handle at least:

* `enum` and `const`
* `oneOf` / `anyOf`
* `nullable`
* `$ref` (within the same schema document)
* Recursive object schemas
* `additionalProperties`
* `patternProperties`
* Tuple schemas (`items` as an array)

If a schema feature cannot be represented, the host MUST fall back to `unknown` and include a doc comment warning.

### 6.3 Tool annotations

If MCP tool annotations are present (e.g., `readOnlyHint`, `destructiveHint`, `idempotentHint`):

* The host MUST surface them in generated doc comments.
* The host MUST include them in `ToolSummary.annotations` and `ToolDefinition.annotations`.

---

## 7. Error model

Errors MUST be structured and optimized for agent self-correction.

### 7.1 Error classes

`@codemode/errors` MUST define:

* `SchemaValidationError` — input failed schema validation.
* `ToolNotFoundError` — the requested tool does not exist on the server.
* `ServerNotFoundError` — the requested server is not connected.
* `ToolCallError` — wraps MCP `isError` results.
* `AuthenticationError` — MCP server rejected the call due to invalid or missing credentials.
* `SandboxLimitError` — sandbox exceeded a configured limit (timeout, memory, tool call cap).

All error classes MUST extend a common `CodemodeError` base class.

### 7.2 Schema validation errors

If input fails validation, the host MUST throw a `SchemaValidationError` including:

* Canonical MCP tool name
* JS export name
* JSON Pointer path to the failing property
* Expected vs received type/value

The host SHOULD include a minimal example payload.

* If schema `examples` are present, they SHOULD be used.
* Otherwise the host MAY synthesize examples.

### 7.3 Error hints

Each error MUST include a `hint` string recommending a single corrective action.

### 7.4 Timeouts, partial results, and cancellation

* If the sandbox exceeds limits, the host MUST throw `SandboxLimitError`.
* The response MUST still include any logs captured prior to termination.
* Tool calls completed prior to termination MAY be visible only through logs.

If the host supports cancellation:

* It SHOULD attempt to cancel in-flight MCP tool calls.
* It MUST terminate the sandbox execution.
* If the host uses MCP cancellation, it SHOULD send the appropriate MCP cancellation notification.

---

## 8. Tool list changes

If an MCP server declares `tools.listChanged`, the host MUST refresh its tool index.

### 8.1 Refresh timing

A host MUST refresh tool definitions before the next `codemode.run` invocation that imports that server module.

### 8.2 In-flight executions

Tool list changes MUST NOT affect the tool set visible within an in-flight `codemode.run` execution.

The updated tool set MUST become visible on the next invocation.

---

## 9. Logging conventions

The sandbox MUST support:

* `console.log`
* `console.debug`
* `console.warn`
* `console.error`

Hosts SHOULD encourage agents to:

* Aggregate and filter large datasets before logging.
* Prefer summaries (counts, top-N) over full dumps.

Log truncation MUST be signaled (see §3.3.1).

---

## 10. Multi-server orchestration

A conforming host MUST allow a single `codemode.run` execution to:

* Import multiple `@codemode/servers/*` modules.
* Compose tool calls across servers.

Failures in one server call MUST surface as a `ToolCallError` (or a subclass) thrown as a normal JS exception.

The host MUST NOT abort the entire script unless a sandbox limit is exceeded.

### 10.1 Intra-script concurrency

A host SHOULD support concurrent tool calls within a single script, e.g.:

```js
await Promise.all([serverA.foo(), serverB.bar()]);
```

If concurrency is unsupported, the host MUST document this restriction.

---

## 11. Security considerations

A conforming host is responsible for:

* Preventing resource exhaustion (timeouts, memory limits, tool call caps).
* Preventing prototype pollution and unsafe host-object exposure.
* Treating MCP server responses as untrusted input.
* Preventing agent-authored code from subverting the binding layer (e.g., tampering with `@codemode/*` module exports).
* Preventing code generation from strings (`eval`, `new Function(string)` MUST NOT be available; see §3.5).

---

## 12. Versioning and conformance

### 12.1 Spec version

A host MUST expose a version identifier inside the sandbox:

```js
import { specVersion } from "@codemode/discovery";
```

`specVersion` MUST be a [semver](https://semver.org/) string (e.g., `"1.0.0"`).

### 12.2 Conformance

A host conforms to this specification if it satisfies all MUST requirements.

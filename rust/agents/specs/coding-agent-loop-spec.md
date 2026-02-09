# Coding Agent Loop Specification

This document is a language-agnostic specification for building a coding agent -- an autonomous system that pairs a large language model with developer tools through an agentic loop. It is designed to be implementable from scratch by any developer or coding agent in any programming language.

This spec layers on top of the [Unified LLM Client Specification](./unified-llm-spec.md), which handles all LLM communication. The agent loop uses the SDK's low-level `Client.complete()` and `Client.stream()` methods directly, implementing its own turn loop to interleave tool execution with truncation, steering, events, and loop detection.

---

## Table of Contents

1. [Overview and Goals](#1-overview-and-goals)
2. [Agentic Loop](#2-agentic-loop)
3. [Provider-Aligned Toolsets](#3-provider-aligned-toolsets)
4. [Tool Execution Environment](#4-tool-execution-environment)
5. [Tool Output and Context Management](#5-tool-output-and-context-management)
6. [System Prompts and Environment Context](#6-system-prompts-and-environment-context)
7. [Subagents](#7-subagents)
8. [Out of Scope (Nice-to-Haves)](#8-out-of-scope-nice-to-haves)
9. [Definition of Done](#9-definition-of-done)

---

## 1. Overview and Goals

### 1.1 Problem Statement

A coding agent is a system that takes a natural language instruction ("fix the login bug", "add dark mode", "write tests for this module"), plans a solution, and executes it by reading files, editing code, running commands, and iterating until the task is done. The core challenge is orchestrating an agentic loop that coordinates LLM calls, tool execution, context management, and provider-specific behavior into a reliable autonomous workflow.

Each LLM provider's models are trained and optimized for specific tool interfaces and system prompts. GPT-5.2 and the GPT-5.2-codex series work best with the same tools and prompts as codex-rs. Gemini models work best with the same tools and prompts as gemini-cli. Anthropic models work best with the same tools and prompts as Claude Code. A good coding agent respects this reality rather than forcing a universal toolset on all models.

### 1.2 Why a Library, Not a CLI

Tools like Claude Code, Codex CLI, and Gemini CLI exist as end-user CLIs. You can run them in non-interactive mode and pipe output, but that gives you a black box: text goes in, text comes out. You cannot programmatically inspect the conversation mid-run, inject steering messages between tool calls, swap the execution environment, change reasoning effort on the fly, observe individual tool calls as they happen, or compose agents into larger systems.

This spec defines a **library** -- a programmable agentic loop that a host application controls at every step. The host can:

- **Submit input and observe every event** as the agent thinks, calls tools, and produces output -- not after the fact, but in real time as it happens.
- **Steer the agent mid-task** by injecting messages between tool rounds, redirecting it without restarting.
- **Change configuration on the fly** -- reasoning effort, model, timeouts -- between any two turns.
- **Swap where tools run** by providing a different execution environment (local, Docker, Kubernetes, WASM, SSH) without changing any tool logic.
- **Compose agents** by spawning subagents for parallel work, each with their own history but sharing the same filesystem.
- **Build on top of it** -- CLIs, IDEs, web UIs, batch systems, CI pipelines, evaluation harnesses, and agent-to-agent coordination systems all consume the same library.

The fidelity of control is the point. Every coding agent CLI is built on an agentic loop internally; this spec makes that loop a first-class, programmable interface.

### 1.3 Design Principles

**Programmable-first.** The agent is a library, not a CLI. Every aspect of the loop -- tool execution, event delivery, steering, configuration -- is accessible programmatically. A CLI is one possible host application, not the primary interface.

**Provider-aligned.** Each model family works best with its native agent's tools and system prompts. The spec defines provider-specific tool profiles, not a single universal set. Start from the provider's native toolset and extend it.

**Extensible execution.** Tool execution is abstracted behind an `ExecutionEnvironment` interface. The default runs locally. Implementations can target Docker, Kubernetes, WASM, or any remote host. Changing where tools run should not require changing the tools themselves.

**Event-driven.** Every agent action emits a typed event for UI rendering, logging, and integration. The event stream is the primary interface for host applications.

**Hackable.** Reasonable defaults with override points everywhere -- timeouts, output sizes, tool sets, execution environments, system prompts, reasoning effort. The spec prescribes defaults, not ceilings.

**Language-agnostic.** All code is pseudocode. Data structures use neutral notation. No specific programming language is assumed.

### 1.3 Architecture

```
+--------------------------------------------------+
|  Host Application (CLI, IDE, Web UI)              |
+--------------------------------------------------+
        |                            ^
        | submit(input)              | events
        v                            |
+--------------------------------------------------+
|  Coding Agent Loop                                |
|  +--------------------+  +---------------------+ |
|  | Session            |  | Provider Profiles   | |
|  |  - history         |  |  - OpenAI (codex)   | |
|  |  - steering queue  |  |  - Anthropic (cc)   | |
|  |  - event emitter   |  |  - Gemini (cli)     | |
|  +--------------------+  +---------------------+ |
|  +--------------------+  +---------------------+ |
|  | Tool Registry      |  | Execution Env       | |
|  |  - tool dispatch   |  |  - local (default)  | |
|  |  - truncation      |  |  - docker           | |
|  |  - validation      |  |  - k8s / wasm / ssh | |
|  +--------------------+  +---------------------+ |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|  Unified LLM SDK (Client.complete / stream)       |
+--------------------------------------------------+
        |
        v
+--------------------------------------------------+
|  LLM Provider APIs                                |
|  (OpenAI Responses / Anthropic Messages / Gemini) |
+--------------------------------------------------+
```

The agent loop does NOT use the Unified LLM SDK's `generate()` high-level function (which has its own tool loop). It uses the low-level `Client.complete()` and implements its own loop because it needs to interleave tool execution with output truncation, steering message injection, event emission, timeout enforcement, and loop detection -- concerns that the SDK's generic tool loop does not handle.

### 1.4 Reference Projects

The following open-source projects solve related problems and are worth studying for anyone implementing this spec.

- **codex-rs** (https://github.com/openai/codex/tree/main/codex-rs) -- Rust. OpenAI's coding agent. Demonstrates async turn-based loop, 15+ tools including `apply_patch` (v4a diff format), output truncation with head/tail split (1 MiB cap), 10-second default command timeout, platform-specific sandboxing, sub-agent spawning, and environment variable filtering.

- **pi-agent-core** (https://github.com/badlogic/pi-mono/tree/main/packages/agent) -- TypeScript. Minimal agent core by @mariozechner. Demonstrates 4-tool minimalism (read, write, edit, bash), explicit `steer()` and `followUp()` queues for mid-turn message injection, 15+ event types, configurable thinking levels, context transform hooks, and abort signal support.

- **gemini-cli** (https://github.com/google-gemini/gemini-cli) -- TypeScript. Google's CLI agent. Demonstrates ReAct loop, 18+ built-in tools including web search and web fetch, GEMINI.md for project-specific instructions, headless/non-interactive mode for automation, and multiple authentication methods.

### 1.5 Relationship to the Unified LLM SDK

This spec assumes the companion Unified LLM Client Specification is implemented. The agent loop imports and uses these types directly:

- `Client`, `Request`, `Response` -- for LLM communication
- `Message`, `ContentPart`, `Role` -- for conversation history
- `Tool`, `ToolCall`, `ToolResult` -- for tool definitions and results
- `StreamEvent` -- for streaming responses
- `Usage` -- for token tracking
- `FinishReason` -- for stop condition detection

The agent builds `Request` objects, calls `Client.complete()` or `Client.stream()`, processes the `Response`, executes any `ToolCall` objects through the execution environment, constructs `ToolResult` objects, appends them to the conversation, and loops.

---

## 2. Agentic Loop

### 2.1 Session

The Session is the central orchestrator. It holds the conversation state, dispatches tool calls, manages the event stream, and enforces limits.

```
RECORD Session:
    id                : String                  -- UUID, assigned at creation
    provider_profile  : ProviderProfile         -- tools + system prompt for the active model
    execution_env     : ExecutionEnvironment    -- where tools run
    history           : List<Turn>              -- ordered conversation turns
    event_emitter     : EventEmitter            -- delivers events to host application
    config            : SessionConfig           -- limits, timeouts, settings
    state             : SessionState            -- current lifecycle state
    llm_client        : Client                  -- from the Unified LLM SDK
    steering_queue    : Queue<String>           -- messages to inject between tool rounds
    followup_queue    : Queue<String>           -- messages to process after current input completes
    subagents         : Map<String, SubAgent>   -- active child agents
```

### 2.2 Session Configuration

```
RECORD SessionConfig:
    max_turns                   : Integer = 0       -- 0 = unlimited
    max_tool_rounds_per_input   : Integer = 200     -- per user input, not per session
    default_command_timeout_ms  : Integer = 10000   -- 10 seconds
    max_command_timeout_ms      : Integer = 600000  -- 10 minutes
    reasoning_effort            : String | None     -- "low", "medium", "high", or null
    tool_output_limits          : Map<String, Integer>  -- per-tool char limits (see Section 5)
    enable_loop_detection       : Boolean = true
    loop_detection_window       : Integer = 10      -- consecutive identical calls before warning
    max_subagent_depth          : Integer = 1       -- max nesting level for subagents
```

### 2.3 Session Lifecycle

```
ENUM SessionState:
    IDLE              -- waiting for user input
    PROCESSING        -- running the agentic loop
    AWAITING_INPUT    -- model asked the user a question
    CLOSED            -- session terminated (normal or error)
```

State transitions:

```
IDLE -> PROCESSING          -- on submit()
PROCESSING -> PROCESSING    -- tool loop continues
PROCESSING -> AWAITING_INPUT -- model asks user a question (no tool calls, open-ended)
PROCESSING -> IDLE          -- natural completion or turn limit
PROCESSING -> CLOSED        -- unrecoverable error
IDLE -> CLOSED              -- explicit close()
any -> CLOSED               -- abort signal
AWAITING_INPUT -> PROCESSING -- user provides answer
```

### 2.4 Turn Types

A Turn is a single entry in the conversation history.

```
RECORD UserTurn:
    content     : String
    timestamp   : Timestamp

RECORD AssistantTurn:
    content     : String            -- text output
    tool_calls  : List<ToolCall>    -- tool invocations requested by the model
    reasoning   : String | None     -- thinking/reasoning text (if available)
    usage       : Usage             -- token counts for this turn
    response_id : String | None     -- provider response ID
    timestamp   : Timestamp

RECORD ToolResultsTurn:
    results     : List<ToolResult>  -- one per tool call
    timestamp   : Timestamp

RECORD SystemTurn:
    content     : String
    timestamp   : Timestamp

RECORD SteeringTurn:
    content     : String            -- injected steering message
    timestamp   : Timestamp
```

### 2.5 The Core Agentic Loop

This is the centerpiece of the spec. The loop runs until the model produces a text-only response (no tool calls), a limit is hit, or an abort signal fires.

```
FUNCTION process_input(session, user_input):
    session.state = PROCESSING
    session.history.APPEND(UserTurn(content = user_input))
    session.emit(USER_INPUT, content = user_input)

    -- Drain any pending steering messages before the first LLM call
    drain_steering(session)

    round_count = 0

    LOOP:
        -- 1. Check limits
        IF round_count >= session.config.max_tool_rounds_per_input:
            session.emit(TURN_LIMIT, round = round_count)
            BREAK

        IF session.config.max_turns > 0 AND count_turns(session) >= session.config.max_turns:
            session.emit(TURN_LIMIT, total_turns = count_turns(session))
            BREAK

        IF session.abort_signaled:
            BREAK

        -- 2. Build LLM request using provider profile
        system_prompt = session.provider_profile.build_system_prompt(
            environment = session.execution_env,
            project_docs = discover_project_docs(session.execution_env.working_directory())
        )
        messages = convert_history_to_messages(session.history)
        tool_defs = session.provider_profile.tools()

        request = Request(
            model           = session.provider_profile.model,
            messages        = [Message.system(system_prompt)] + messages,
            tools           = tool_defs,
            tool_choice     = "auto",
            reasoning_effort = session.config.reasoning_effort,
            provider        = session.provider_profile.id,
            provider_options = session.provider_profile.provider_options()
        )

        -- 3. Call LLM via Unified LLM SDK (single-shot, no SDK-level tool loop)
        response = session.llm_client.complete(request)

        -- 4. Record assistant turn
        assistant_turn = AssistantTurn(
            content     = response.text,
            tool_calls  = response.tool_calls,
            reasoning   = response.reasoning,
            usage       = response.usage,
            response_id = response.id
        )
        session.history.APPEND(assistant_turn)
        session.emit(ASSISTANT_TEXT_END, text = response.text, reasoning = response.reasoning)

        -- 5. If no tool calls, natural completion
        IF response.tool_calls IS EMPTY:
            BREAK

        -- 6. Execute tool calls through the execution environment
        round_count += 1
        results = execute_tool_calls(session, response.tool_calls)
        session.history.APPEND(ToolResultsTurn(results = results))

        -- 7. Drain steering messages injected during tool execution
        drain_steering(session)

        -- 8. Loop detection
        IF session.config.enable_loop_detection:
            IF detect_loop(session.history, session.config.loop_detection_window):
                warning = "Loop detected: the last " + session.config.loop_detection_window
                        + " tool calls follow a repeating pattern. Try a different approach."
                session.history.APPEND(SteeringTurn(content = warning))
                session.emit(LOOP_DETECTION, message = warning)

    END LOOP

    -- Process follow-up messages if any are queued
    IF session.followup_queue IS NOT EMPTY:
        next_input = session.followup_queue.DEQUEUE()
        process_input(session, next_input)
        RETURN

    session.state = IDLE
    session.emit(SESSION_END)


FUNCTION drain_steering(session):
    WHILE session.steering_queue IS NOT EMPTY:
        msg = session.steering_queue.DEQUEUE()
        session.history.APPEND(SteeringTurn(content = msg))
        session.emit(STEERING_INJECTED, content = msg)


FUNCTION execute_tool_calls(session, tool_calls):
    results = []

    -- Execute tool calls (concurrently if profile supports parallel execution)
    IF session.provider_profile.supports_parallel_tool_calls AND LENGTH(tool_calls) > 1:
        results = AWAIT_ALL([
            execute_single_tool(session, tc) FOR tc IN tool_calls
        ])
    ELSE:
        FOR EACH tc IN tool_calls:
            result = execute_single_tool(session, tc)
            results.APPEND(result)
    RETURN results


FUNCTION execute_single_tool(session, tool_call):
    session.emit(TOOL_CALL_START, tool_name = tool_call.name, call_id = tool_call.id)

    -- Look up tool in registry
    registered = session.provider_profile.tool_registry.get(tool_call.name)
    IF registered IS None:
        error_msg = "Unknown tool: " + tool_call.name
        session.emit(TOOL_CALL_END, call_id = tool_call.id, error = error_msg)
        RETURN ToolResult(tool_call_id = tool_call.id, content = error_msg, is_error = true)

    -- Execute via execution environment
    TRY:
        raw_output = registered.execute(tool_call.arguments, session.execution_env)

        -- Truncate output before sending to LLM (character-based first, then line-based)
        truncated_output = truncate_tool_output(raw_output, tool_call.name, session.config)

        -- Emit full output via event stream (not truncated)
        session.emit(TOOL_CALL_END, call_id = tool_call.id, output = raw_output)

        RETURN ToolResult(
            tool_call_id = tool_call.id,
            content = truncated_output,
            is_error = false
        )

    CATCH error:
        error_msg = "Tool error (" + tool_call.name + "): " + str(error)
        session.emit(TOOL_CALL_END, call_id = tool_call.id, error = error_msg)
        RETURN ToolResult(tool_call_id = tool_call.id, content = error_msg, is_error = true)
```

### 2.6 Steering

Steering allows the host application to inject messages into the conversation between tool rounds. This is how a user can redirect the agent mid-task without waiting for it to finish.

```
session.steer(message: String)
    -- Queue a message to be injected after the current tool round completes.
    -- The message becomes a SteeringTurn in the history, converted to a
    -- user message for the LLM on the next call.
    -- If the agent is idle, the message is delivered on the next submit().

session.follow_up(message: String)
    -- Queue a message to be processed after the current input is fully handled
    -- (model has produced a text-only response). Triggers a new processing cycle.
```

SteeringTurns are converted to user-role messages when building the LLM request. This means the model sees them as additional user instructions.

### 2.7 Reasoning Effort

The `reasoning_effort` config controls how much reasoning/thinking the model does. It maps directly to the Unified LLM SDK's `reasoning_effort` field on the Request.

| Value    | Effect                                                           |
|----------|------------------------------------------------------------------|
| "low"    | Minimal reasoning. Faster, cheaper. Good for simple tasks.      |
| "medium" | Balanced reasoning. Default for most tasks.                     |
| "high"   | Deep reasoning. Slower, more expensive. Good for complex tasks. |
| null     | Provider default (no override).                                 |

Changing `reasoning_effort` mid-session takes effect on the next LLM call. For OpenAI reasoning models (GPT-5.2 series), this controls the reasoning token budget. For Anthropic models with extended thinking, this maps to the thinking budget. For Gemini models with thinking, this maps to thinkingConfig.

### 2.8 Stop Conditions

The loop exits when any of these conditions is met:

1. **Natural completion.** The model responds with text only (no tool calls). The model is done.
2. **Round limit.** `max_tool_rounds_per_input` is reached. The agent stops and returns what it has.
3. **Turn limit.** `max_turns` across the entire session is reached.
4. **Abort signal.** The host application signals cancellation. The current LLM stream is closed, running processes are killed, and the session transitions to CLOSED.
5. **Unrecoverable error.** An authentication error, context overflow, or other non-retryable error. The session transitions to CLOSED.

### 2.9 Event System

Every agent action emits a typed event. Events are delivered via an async iterator (or language-appropriate equivalent) to the host application.

```
RECORD SessionEvent:
    kind        : EventKind
    timestamp   : Timestamp
    session_id  : String
    data        : Map<String, Any>

ENUM EventKind:
    SESSION_START           -- session created
    SESSION_END             -- session closed (includes final state)
    USER_INPUT              -- user submitted input
    ASSISTANT_TEXT_START     -- model began generating text
    ASSISTANT_TEXT_DELTA     -- incremental text token
    ASSISTANT_TEXT_END       -- model finished text (includes full text)
    TOOL_CALL_START         -- tool execution began (includes tool name, call ID)
    TOOL_CALL_OUTPUT_DELTA  -- incremental tool output (for streaming tools)
    TOOL_CALL_END           -- tool execution finished (includes FULL untruncated output)
    STEERING_INJECTED       -- a steering message was added to history
    TURN_LIMIT              -- a turn limit was hit
    LOOP_DETECTION          -- a loop pattern was detected
    ERROR                   -- an error occurred
```

**Key design decision:** The `TOOL_CALL_END` event carries the FULL untruncated tool output. The LLM receives the truncated version. This means the host application (UI, logs) always has access to complete output even though the model sees an abbreviated version.

### 2.10 Loop Detection

Track the signature of each tool call (name + arguments hash). If the last N calls (default: 10) contain a repeating pattern (e.g., the same 2-3 calls cycling), inject a warning as a SteeringTurn telling the model to try a different approach.

```
FUNCTION detect_loop(history, window_size) -> Boolean:
    recent_calls = extract_tool_call_signatures(history, last = window_size)
    IF LENGTH(recent_calls) < window_size: RETURN false

    -- Check for repeating patterns of length 1, 2, or 3
    FOR pattern_len IN [1, 2, 3]:
        IF window_size % pattern_len != 0: CONTINUE
        pattern = recent_calls[0..pattern_len]
        all_match = true
        FOR i FROM pattern_len TO window_size STEP pattern_len:
            IF recent_calls[i..i+pattern_len] != pattern:
                all_match = false
                BREAK
        IF all_match: RETURN true

    RETURN false
```

---

## 3. Provider-Aligned Toolsets

### 3.1 The Provider Alignment Principle

Models are trained and optimized for specific tool interfaces. OpenAI's models are trained on codex-rs's apply_patch format and tool schemas. Anthropic's models are trained on Claude Code's old_string/new_string editing and tool schemas. Gemini models are trained on gemini-cli's tool set.

Using a provider's native tool format produces better results than forcing a universal format. **The initial base for each provider should be a 1:1 copy of the provider's reference agent -- the exact same system prompt, the exact same tool definitions, byte for byte.** Not a similar prompt. Not similar tools. The actual prompt and harness that the model was evaluated and optimized against. Then extend it with additional capabilities (like subagents). Do not make all providers conform to a single tool interface.

### 3.2 ProviderProfile Interface

```
INTERFACE ProviderProfile:
    id              : String            -- "openai", "anthropic", "gemini"
    model           : String            -- model identifier (e.g., "gpt-5.2-codex")
    tool_registry   : ToolRegistry      -- all tools available to this profile

    FUNCTION build_system_prompt(environment, project_docs) -> String
    FUNCTION tools() -> List<ToolDefinition>
    FUNCTION provider_options() -> Map | None

    -- Capability flags
    supports_reasoning           : Boolean
    supports_streaming           : Boolean
    supports_parallel_tool_calls : Boolean
    context_window_size          : Integer
```

### 3.3 Shared Core Tools

All profiles include these base tools. The parameter schemas and output formats may vary between profiles (to match the provider's native conventions), but the functionality is the same.

#### read_file

Reads a file's contents with line numbers.

```
TOOL read_file:
    description: "Read a file from the filesystem. Returns line-numbered content."
    parameters:
        file_path   : String (required)     -- absolute path to the file
        offset      : Integer (optional)    -- 1-based line number to start reading from
        limit       : Integer (optional)    -- max lines to read (default: 2000)
    returns: Line-numbered text content in "NNN | content" format
    errors: File not found, permission denied, binary file
```

Behavior: Read the file, prepend line numbers, respect offset/limit. For image files, return the image data for multimodal models. For very large files without offset/limit, the tool output will be truncated by the truncation layer (Section 5).

#### write_file

Writes content to a file, creating it if it does not exist.

```
TOOL write_file:
    description: "Write content to a file. Creates the file and parent directories if needed."
    parameters:
        file_path   : String (required)     -- absolute path
        content     : String (required)     -- the full file content
    returns: Confirmation message with bytes written
    errors: Permission denied, disk full
```

#### edit_file

Searches for an exact string in a file and replaces it. This is the native editing format for Anthropic models.

```
TOOL edit_file:
    description: "Replace an exact string occurrence in a file."
    parameters:
        file_path   : String (required)
        old_string  : String (required)     -- exact text to find
        new_string  : String (required)     -- replacement text
        replace_all : Boolean (optional)    -- replace all occurrences (default: false)
    returns: Confirmation with number of replacements made
    errors: File not found, old_string not found, old_string not unique (when replace_all=false)
```

Behavior: Exact string match. If `old_string` is not found exactly, the implementation may attempt fuzzy matching (whitespace normalization, Unicode equivalence) and report the match. If `old_string` matches multiple locations and `replace_all` is false, return an error asking the model to provide more context.

#### shell

Executes a command in the system shell.

```
TOOL shell:
    description: "Execute a shell command. Returns stdout, stderr, and exit code."
    parameters:
        command     : String (required)     -- the command to run
        timeout_ms  : Integer (optional)    -- override default timeout
        description : String (optional)     -- human-readable description of what this does
    returns: Command output (stdout + stderr), exit code, duration
    errors: Timeout, permission denied, command not found
```

Behavior: Run in a new process group. Enforce timeout (default from SessionConfig, overridable per-call). On timeout: SIGTERM, wait 2 seconds, SIGKILL. Return collected output plus timeout message. Environment variable filtering applied (see Section 4).

#### grep

Searches file contents by pattern.

```
TOOL grep:
    description: "Search file contents using regex patterns."
    parameters:
        pattern         : String (required)     -- regex pattern
        path            : String (optional)     -- directory or file to search (default: working dir)
        glob_filter     : String (optional)     -- file pattern filter (e.g., "*.py")
        case_insensitive: Boolean (optional)    -- default: false
        max_results     : Integer (optional)    -- default: 100
    returns: Matching lines with file paths and line numbers
    errors: Invalid regex, path not found
```

#### glob

Finds files by name pattern.

```
TOOL glob:
    description: "Find files matching a glob pattern."
    parameters:
        pattern     : String (required)     -- glob pattern (e.g., "**/*.ts")
        path        : String (optional)     -- base directory (default: working dir)
    returns: List of matching file paths, sorted by modification time (newest first)
    errors: Invalid pattern, path not found
```

### 3.4 OpenAI Profile (codex-rs-aligned)

For GPT-5.2, GPT-5.2-codex, and other OpenAI models. Mirrors the codex-rs toolset.

**Key difference: `apply_patch` replaces `edit_file` and `write_file` for file modifications.** OpenAI models are specifically trained on this format and produce significantly better edits when using it.

Additional/modified tools beyond the shared core:

#### apply_patch (OpenAI-specific)

```
TOOL apply_patch:
    description: "Apply code changes using the patch format. Supports creating, deleting,
                  and modifying files in a single operation."
    parameters:
        patch       : String (required)     -- the patch content in v4a format
    returns: List of affected file paths and operations performed
    errors: Parse error, file not found (for updates), verification failure
```

The patch format is defined in full in [Appendix A](#appendix-a-apply_patch-v4a-format-reference).

**Profile tool list for OpenAI:**
- `read_file` (same as shared core, maps to codex-rs `read_file`)
- `apply_patch` (replaces `edit_file` and `write_file` for modifications)
- `write_file` (kept for creating new files without patch overhead)
- `shell` (maps to codex-rs `exec_command`, 10s default timeout)
- `grep` (maps to codex-rs `grep_files`)
- `glob` (maps to codex-rs `list_dir`)
- `spawn_agent`, `send_input`, `wait`, `close_agent` (subagent tools, Section 7)

**System prompt:** Should mirror the codex-rs system prompt structure. Cover identity, tool usage guidelines, the apply_patch format expectations, and coding best practices.

**Provider options:** The OpenAI profile should set `reasoning.effort` on the Responses API request when `reasoning_effort` is configured.

### 3.5 Anthropic Profile (Claude Code-aligned)

For Claude Opus 4.6, Opus 4.5, Sonnet 4.5, Haiku 4.5, and older Claude models. Mirrors the Claude Code toolset.

**Key difference: `edit_file` with `old_string`/`new_string` is the native editing format.** Anthropic models are specifically trained on this exact-match search-and-replace pattern. Do NOT use apply_patch with Anthropic models.

**Profile tool list for Anthropic:**
- `read_file` (line-numbered output, offset/limit support)
- `write_file` (full file writes)
- `edit_file` (old_string/new_string -- this is the native format)
- `shell` (bash execution, 120s default timeout per Claude Code convention)
- `grep` (ripgrep-backed with output modes: content, files_with_matches, count)
- `glob` (file pattern matching sorted by mtime)
- Subagent tools (maps to Claude Code's Task tool pattern, Section 7)

**System prompt:** Should mirror the Claude Code system prompt structure. Cover identity, tool selection guidance, the edit_file format (explain that `old_string` must be unique), file operation preferences (edit existing files over creating new ones), and coding best practices.

**Provider options:** The Anthropic profile should pass beta headers (e.g., for extended thinking, 1M context) via `provider_options.anthropic.beta_headers`.

### 3.6 Gemini Profile (gemini-cli-aligned)

For Gemini 3 Flash, Gemini 2.5 Pro/Flash, and other Gemini models. Mirrors the gemini-cli toolset.

**Profile tool list for Gemini:**
- `read_file` / `read_many_files` (batch reading support)
- `write_file`
- `edit_file` (search-and-replace style, matching gemini-cli conventions)
- `shell` (command execution, 10s default timeout)
- `grep` (ripgrep semantics)
- `glob` (file pattern matching)
- `list_dir` (directory listing with depth options)
- `web_search` (optional -- Gemini models have native grounding capabilities)
- `web_fetch` (optional -- fetch and extract content from URLs)
- Subagent tools (Section 7)

**System prompt:** Should mirror the gemini-cli system prompt structure. Cover identity, tool usage, GEMINI.md conventions, and coding best practices.

**Provider options:** Gemini profile should configure safety settings and grounding via `provider_options.gemini`.

### 3.7 Extending Profiles with Custom Tools

After a provider profile is loaded, additional tools can be registered:

```
profile = create_openai_profile(model = "gpt-5.2-codex")

-- Add a custom tool on top of the profile
profile.tool_registry.register(RegisteredTool(
    definition = ToolDefinition(
        name = "run_tests",
        description = "Run the project's test suite",
        parameters = { "type": "object", "properties": { "filter": { "type": "string" } } }
    ),
    executor = run_tests_function
))
```

Name collisions are resolved by latest-wins: a custom tool with the same name as a profile tool overrides it.

### 3.8 Tool Registry

```
RECORD ToolDefinition:
    name        : String            -- unique identifier
    description : String            -- for the LLM
    parameters  : Dict              -- JSON Schema (root must be "object")

RECORD RegisteredTool:
    definition  : ToolDefinition
    executor    : Function          -- (arguments, execution_env) -> String

RECORD ToolRegistry:
    _tools      : Map<String, RegisteredTool>

    register(tool)                  -- add or replace a tool
    unregister(name)                -- remove a tool
    get(name) -> RegisteredTool | None
    definitions() -> List<ToolDefinition>
    names() -> List<String>
```

**Tool execution pipeline:**

```
1. LOOKUP      -- find the RegisteredTool by name
2. VALIDATE    -- parse and validate arguments against JSON Schema
3. EXECUTE     -- call executor with (arguments, execution_env)
4. TRUNCATE    -- apply output size limits (Section 5)
5. EMIT        -- emit TOOL_CALL_END event with full output
6. RETURN      -- return truncated output as ToolResult
```

---

## 4. Tool Execution Environment

### 4.1 The Execution Environment Abstraction

All tool operations pass through an `ExecutionEnvironment` interface. This decouples tool logic from where it runs. The default runs locally. Swap in a different implementation to run the same tools in Docker, on a Kubernetes pod, over SSH, or in WASM.

```
INTERFACE ExecutionEnvironment:
    -- File operations
    read_file(path: String, offset: Integer | None, limit: Integer | None) -> String
    write_file(path: String, content: String) -> void
    file_exists(path: String) -> Boolean
    list_directory(path: String, depth: Integer) -> List<DirEntry>

    -- Command execution
    exec_command(
        command     : String,
        timeout_ms  : Integer,
        working_dir : String | None,
        env_vars    : Map<String, String> | None
    ) -> ExecResult

    -- Search operations
    grep(pattern: String, path: String, options: GrepOptions) -> String
    glob(pattern: String, path: String) -> List<String>

    -- Lifecycle
    initialize() -> void
    cleanup() -> void

    -- Metadata
    working_directory() -> String
    platform() -> String           -- "darwin", "linux", "windows", "wasm"
    os_version() -> String

RECORD ExecResult:
    stdout      : String
    stderr      : String
    exit_code   : Integer
    timed_out   : Boolean
    duration_ms : Integer

RECORD DirEntry:
    name        : String
    is_dir      : Boolean
    size        : Integer | None
```

### 4.2 LocalExecutionEnvironment (Required Implementation)

The default. Runs everything on the local machine.

**File operations:** Direct filesystem access. Paths are resolved relative to `working_directory()`.

**Command execution:**
- Spawn in a new process group for clean killability
- Use the platform's default shell (`/bin/bash -c` on Linux/macOS, `cmd.exe /c` on Windows)
- Enforce timeout: on timeout, send SIGTERM to the process group, wait 2 seconds, then SIGKILL
- Capture stdout and stderr separately, then combine for the result
- Record wall-clock duration

**Environment variable filtering:**
- By default, exclude variables matching: `*_API_KEY`, `*_SECRET`, `*_TOKEN`, `*_PASSWORD`, `*_CREDENTIAL` (case-insensitive)
- Always include: `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `TERM`, `TMPDIR`, language-specific paths (`GOPATH`, `CARGO_HOME`, `NVM_DIR`, etc.)
- Customizable via an env var policy: inherit all, inherit none (start clean), or inherit core only

**Search operations:** Use `ripgrep` for grep if available, fall back to language-native regex search. Use filesystem globbing for glob.

### 4.3 Alternative Environments (Extension Points)

These are not required implementations. They demonstrate the extensibility of the interface.

**DockerExecutionEnvironment:**
```
-- Commands execute inside a Docker container
exec_command(cmd, ...) -> docker exec <container_id> sh -c <cmd>
-- File operations use volume mounts or docker cp
read_file(path) -> docker cp <container_id>:<path> - | read
write_file(path, content) -> pipe content | docker cp - <container_id>:<path>
```

**KubernetesExecutionEnvironment:**
```
-- Commands execute in a Kubernetes pod
exec_command(cmd, ...) -> kubectl exec <pod> -- sh -c <cmd>
-- File operations use kubectl cp
read_file(path) -> kubectl cp <pod>:<path> /dev/stdout
```

**WASMExecutionEnvironment:**
```
-- For in-browser or embedded use
-- File operations use an in-memory filesystem (e.g., memfs)
-- Command execution is limited or emulated via WASI
```

**RemoteSSHExecutionEnvironment:**
```
-- Commands execute over SSH
exec_command(cmd, ...) -> ssh <host> <cmd>
-- File operations use SCP/SFTP
read_file(path) -> sftp get <host>:<path>
```

### 4.4 Composing Environments

Execution environments can be wrapped for cross-cutting concerns:

```
-- Logging wrapper
LoggingExecutionEnvironment(inner: ExecutionEnvironment):
    exec_command(cmd, ...):
        LOG("exec: " + cmd)
        result = inner.exec_command(cmd, ...)
        LOG("exit: " + result.exit_code + " in " + result.duration_ms + "ms")
        RETURN result

-- Read-only wrapper (rejects all writes)
ReadOnlyExecutionEnvironment(inner: ExecutionEnvironment):
    write_file(path, content):
        RAISE "Write operations are disabled in read-only mode"
    exec_command(cmd, ...):
        -- Could analyze command for write intent, or allow all
        RETURN inner.exec_command(cmd, ...)
```

---

## 5. Tool Output and Context Management

### 5.1 Tool Output Truncation

When tool output exceeds the configured limit, it MUST be truncated before being sent to the LLM. The full output is always available via the event stream (`TOOL_CALL_END` event).

**Truncation algorithm (head/tail split):**

```
FUNCTION truncate_output(output: String, max_chars: Integer, mode: String) -> String:
    IF LENGTH(output) <= max_chars:
        RETURN output

    IF mode == "head_tail":
        half = max_chars / 2
        removed = LENGTH(output) - max_chars
        RETURN output[0..half]
             + "\n\n[WARNING: Tool output was truncated. "
             + removed + " characters were removed from the middle. "
             + "The full output is available in the event stream. "
             + "If you need to see specific parts, re-run the tool with more targeted parameters.]\n\n"
             + output[-half..]

    IF mode == "tail":
        removed = LENGTH(output) - max_chars
        RETURN "[WARNING: Tool output was truncated. First "
             + removed + " characters were removed. "
             + "The full output is available in the event stream.]\n\n"
             + output[-max_chars..]
```

The truncation message explicitly tells the model that output was truncated, how much was removed, and where the full output lives. This prevents the model from making decisions based on incomplete information without knowing it is incomplete.

### 5.2 Default Output Size Limits

| Tool         | Default Max (chars) | Truncation Mode | Rationale                                            |
|--------------|---------------------|-----------------|------------------------------------------------------|
| read_file    | 50,000              | head_tail       | Keep beginning (imports/types) and end (recent code) |
| shell        | 30,000              | head_tail       | Beginning has startup info, end has results          |
| grep         | 20,000              | tail            | Keep the most recent/relevant matches                |
| glob         | 20,000              | tail            | Most recently modified files first                   |
| edit_file    | 10,000              | tail            | Confirmation output, usually short                   |
| apply_patch  | 10,000              | tail            | Patch results, usually short                         |
| write_file   | 1,000               | tail            | Confirmation, always short                           |
| spawn_agent  | 20,000              | head_tail       | Subagent results                                     |

These defaults are overridable via `SessionConfig.tool_output_limits`.

### 5.3 Truncation Order (Important)

Character-based truncation (Section 5.1) is the primary safeguard and MUST always run first. It handles every case including pathological ones like a 2-line CSV where each line is 10MB. Line-based truncation is a secondary readability pass that runs after character truncation.

The full pipeline for every tool output:

```
FUNCTION truncate_tool_output(output, tool_name, config) -> String:
    max_chars = config.tool_output_limits.get(tool_name, DEFAULT_TOOL_LIMITS[tool_name])

    -- Step 1: Character-based truncation (always runs, handles all size concerns)
    result = truncate_output(output, max_chars, DEFAULT_TRUNCATION_MODES[tool_name])

    -- Step 2: Line-based truncation (secondary, for readability)
    max_lines = config.tool_line_limits.get(tool_name, DEFAULT_LINE_LIMITS[tool_name])
    IF max_lines IS NOT None:
        result = truncate_lines(result, max_lines)

    RETURN result
```

**Default line limits** (applied after character truncation):

| Tool         | Default Max Lines | Rationale                                |
|--------------|-------------------|------------------------------------------|
| shell        | 256               | Command output with many short lines     |
| grep         | 200               | Search results, one per line             |
| glob         | 500               | File listings, one path per line         |
| read_file    | None              | Character limit is sufficient            |
| edit_file    | None              | Character limit is sufficient            |

Line-based truncation uses the same head/tail split:

```
FUNCTION truncate_lines(output: String, max_lines: Integer) -> String:
    lines = SPLIT(output, "\n")
    IF LENGTH(lines) <= max_lines:
        RETURN output

    head_count = max_lines / 2
    tail_count = max_lines - head_count
    omitted = LENGTH(lines) - head_count - tail_count

    RETURN JOIN(lines[0..head_count], "\n")
         + "\n[... " + omitted + " lines omitted ...]\n"
         + JOIN(lines[-tail_count..], "\n")
```

**Why character truncation must come first:** A file could have 2 lines that are each 10MB. Line-based truncation would see "only 2 lines" and pass it through untouched, blowing up the context window. Character truncation catches this because it operates on raw size, not line count. Always truncate by size first, then by line count.

### 5.4 Default Command Timeouts

Every command execution has a default timeout. The model can override the timeout per-call via the shell tool's `timeout_ms` parameter.

| Setting                      | Default   | Purpose                                |
|------------------------------|-----------|----------------------------------------|
| default_command_timeout_ms   | 10,000    | Applied when timeout_ms is not set     |
| max_command_timeout_ms       | 600,000   | Upper bound (10 minutes)               |

When a timeout fires:
1. Send SIGTERM to the process group
2. Wait 2 seconds for graceful shutdown
3. Send SIGKILL if the process is still running
4. Return collected output so far plus a timeout message

The timeout message sent to the LLM:
```
[ERROR: Command timed out after {X}ms. Partial output is shown above.
You can retry with a longer timeout by setting the timeout_ms parameter.]
```

### 5.5 Context Window Awareness

The agent should track approximate token usage using the heuristic: 1 token ~ 4 characters. Emit a warning event when usage exceeds 80% of the provider profile's `context_window_size`.

This is informational only. The agent does NOT perform automatic compaction or summarization (that is out of scope for this spec). The host application can use this signal to implement its own context management strategy.

```
FUNCTION check_context_usage(session):
    approx_tokens = total_chars_in_history(session.history) / 4
    threshold = session.provider_profile.context_window_size * 0.8
    IF approx_tokens > threshold:
        session.emit(WARNING, message = "Context usage at ~"
            + ROUND(approx_tokens / session.provider_profile.context_window_size * 100)
            + "% of context window")
```

---

## 6. System Prompts and Environment Context

### 6.1 Layered System Prompt Construction

The system prompt is assembled from multiple layers, with later layers taking precedence:

```
final_system_prompt =
    1. Provider-specific base instructions     (from ProviderProfile)
  + 2. Environment context                     (platform, git, working dir, date, model info)
  + 3. Tool descriptions                       (from the active profile's tool set)
  + 4. Project-specific instructions           (AGENTS.md, CLAUDE.md, GEMINI.md, etc.)
  + 5. User instructions override              (appended last, highest priority)
```

### 6.2 Provider-Specific Base Instructions

Each profile supplies its own base prompt tuned for the model family. The base instructions should closely mirror the system prompts of the provider's native agent:

- **OpenAI profile:** Mirror codex-rs system prompt. Cover identity, tool usage (especially apply_patch conventions), coding best practices, error handling guidance.
- **Anthropic profile:** Mirror Claude Code system prompt. Cover identity, tool selection guidance (read before edit, edit over write), the edit_file format (old_string must be unique), file operation preferences.
- **Gemini profile:** Mirror gemini-cli system prompt. Cover identity, tool usage, GEMINI.md conventions, coding best practices.

The spec does NOT prescribe full system prompt text -- those are implementation details that change frequently. It specifies what topics the prompt must cover.

### 6.3 Environment Context Block

Include a structured block with runtime information:

```
<environment>
Working directory: {working_directory}
Is git repository: {true/false}
Git branch: {current_branch}
Platform: {darwin/linux/windows}
OS version: {os_version_string}
Today's date: {YYYY-MM-DD}
Model: {model_display_name}
Knowledge cutoff: {knowledge_cutoff_date}
</environment>
```

This block is generated at session start and included in every system prompt.

### 6.4 Git Context

Snapshot at session start. Include:
- Current branch
- Short status (modified/untracked file count, not full diff)
- Recent commit messages (last 5-10)

The model can always run `git status`, `git diff`, etc. via the shell tool for current state. The snapshot provides initial orientation.

### 6.5 Project Document Discovery

Walk from the git root (or working directory if not in a git repo) to the current working directory. Recognized instruction files:

| File Name                  | Convention       |
|----------------------------|------------------|
| `AGENTS.md`                | Universal        |
| `CLAUDE.md`                | Anthropic-aligned|
| `GEMINI.md`                | Gemini-aligned   |
| `.codex/instructions.md`   | OpenAI-aligned   |

**Loading rules:**
- Root-level files are loaded first
- Subdirectory files are appended (deeper = higher precedence)
- Total byte budget: 32KB. If exceeded, truncate with a marker: "[Project instructions truncated at 32KB]"
- Only load files matching the active provider profile (e.g., Anthropic profile loads AGENTS.md and CLAUDE.md, not GEMINI.md)
- AGENTS.md is always loaded regardless of provider

---

## 7. Subagents

### 7.1 Concept

A subagent is a child session spawned by the parent to handle a scoped task. The subagent runs its own agentic loop with its own conversation history but shares the parent's execution environment (same filesystem, same working directory or a subdirectory). This enables parallel work and task decomposition.

### 7.2 Spawn Interface

```
TOOL spawn_agent:
    description: "Spawn a subagent to handle a scoped task autonomously."
    parameters:
        task            : String (required)     -- natural language task description
        working_dir     : String (optional)     -- subdirectory to scope the agent to
        model           : String (optional)     -- model override (default: parent's model)
        max_turns       : Integer (optional)    -- turn limit (default: 50)
    returns: Agent ID and initial status

TOOL send_input:
    description: "Send a message to a running subagent."
    parameters:
        agent_id        : String (required)
        message         : String (required)
    returns: Acknowledgement

TOOL wait:
    description: "Wait for a subagent to complete and return its result."
    parameters:
        agent_id        : String (required)
    returns: SubAgentResult (output text, success boolean, turns used)

TOOL close_agent:
    description: "Terminate a subagent."
    parameters:
        agent_id        : String (required)
    returns: Final status
```

### 7.3 SubAgent Lifecycle

```
RECORD SubAgentHandle:
    id          : String
    session     : Session           -- independent session with own history
    status      : "running" | "completed" | "failed"

RECORD SubAgentResult:
    output      : String            -- final text output from the subagent
    success     : Boolean
    turns_used  : Integer
```

The subagent:
- Gets its own Session with independent conversation history
- Shares the parent's `ExecutionEnvironment` (same filesystem)
- Uses the parent's `ProviderProfile` (or an overridden model)
- Has its own turn limits (configurable, default: 50)
- Cannot spawn sub-sub-agents (depth limiting, default max depth: 1, configurable via `max_subagent_depth`)

### 7.4 Use Cases

- **Parallel exploration:** Spawn multiple agents to investigate different parts of the codebase simultaneously
- **Focused refactoring:** Scope an agent to a single module with a specific task
- **Test execution:** Spawn an agent to run and fix tests while the parent continues other work
- **Alternative approaches:** Spawn agents to try different solutions and pick the best one

---

## 8. Out of Scope (Nice-to-Haves)

The following features are intentionally excluded from this core spec. They are valuable extensions that can be added on top of the architecture defined here. The spec's design has natural extension points for each.

**MCP (Model Context Protocol).** An MCP client can extend the agent with tools from external servers (GitHub, databases, Slack, etc.). The tool registry supports registering MCP-discovered tools with namespaced names (e.g., `github__create_pr`). This is a natural extension but not a core requirement for a functional coding agent.

**Skills / Custom Commands.** Reusable prompt templates stored as markdown files with YAML frontmatter. Skills standardize common workflows (e.g., `/commit`, `/review-pr`) and can be loaded from project directories or user home. The system prompt layer has a natural insertion point for skill descriptions.

**Sandbox / Security Policies.** OS-level sandboxing (macOS Seatbelt, Linux Landlock/Seccomp, Windows restricted tokens) constrains file and network access. The `ExecutionEnvironment` abstraction provides a natural hook -- a `SandboxedLocalExecutionEnvironment` could wrap the default environment. For stronger isolation, use `DockerExecutionEnvironment`.

**Compaction / Context Summarization.** Automatic conversation history summarization when approaching context limits. This is a complex feature with significant tradeoffs (information loss, summarization cost, pinned turns). The context window awareness signal (Section 5.5) gives host applications the information they need to implement their own strategy.

**Approval / Permission System.** User approval gates for sensitive operations (file writes, shell commands, destructive actions). The tool execution pipeline (Section 3.8) has a natural extension point between VALIDATE and EXECUTE where an approval step can be inserted.

**Read-Before-Write Guardrail.** Tracking which files have been read and blocking writes to unread files. A heuristic safety net that can be implemented as a tool execution middleware wrapping the execution environment.

---

## 9. Definition of Done

This section defines how to validate that an implementation of this spec is complete and correct. An implementation is done when every item is checked off.

### 9.1 Core Loop

- [ ] Session can be created with a ProviderProfile and ExecutionEnvironment
- [ ] `process_input()` runs the agentic loop: LLM call -> tool execution -> loop until natural completion
- [ ] Natural completion: model responds with text only (no tool calls) and the loop exits
- [ ] Round limits: `max_tool_rounds_per_input` stops the loop when reached
- [ ] Session turn limits: `max_turns` stops the loop across all inputs
- [ ] Abort signal: cancellation stops the loop, kills running processes, transitions to CLOSED
- [ ] Loop detection: consecutive identical tool call patterns trigger a warning SteeringTurn
- [ ] Multiple sequential inputs work: submit, wait for completion, submit again

### 9.2 Provider Profiles

- [ ] OpenAI profile provides codex-rs-aligned tools including `apply_patch` (v4a format)
- [ ] Anthropic profile provides Claude Code-aligned tools including `edit_file` (old_string/new_string)
- [ ] Gemini profile provides gemini-cli-aligned tools
- [ ] Each profile produces a provider-specific system prompt covering identity, tool usage, and coding guidance
- [ ] Custom tools can be registered on top of any profile
- [ ] Tool name collisions resolved: custom registration overrides profile defaults

### 9.3 Tool Execution

- [ ] Tool calls are dispatched through the ToolRegistry
- [ ] Unknown tool calls return an error result to the LLM (not an exception)
- [ ] Tool argument JSON is parsed and validated against the tool's parameter schema
- [ ] Tool execution errors are caught and returned as error results (`is_error = true`)
- [ ] Parallel tool execution works when the profile's `supports_parallel_tool_calls` is true

### 9.4 Execution Environment

- [ ] `LocalExecutionEnvironment` implements all file and command operations
- [ ] Command timeout default is 10 seconds
- [ ] Command timeout is overridable per-call via the shell tool's `timeout_ms` parameter
- [ ] Timed-out commands: process group receives SIGTERM, then SIGKILL after 2 seconds
- [ ] Environment variable filtering excludes sensitive variables (`*_API_KEY`, `*_SECRET`, etc.) by default
- [ ] The `ExecutionEnvironment` interface is implementable by consumers for custom environments (Docker, K8s, WASM, SSH)

### 9.5 Tool Output Truncation

- [ ] Character-based truncation runs FIRST on all tool outputs (handles pathological cases like 10MB single-line CSVs)
- [ ] Line-based truncation runs SECOND where configured (shell: 256, grep: 200, glob: 500)
- [ ] Truncation inserts a visible marker: `[WARNING: Tool output was truncated. N characters removed...]`
- [ ] The full untruncated output is available via the `TOOL_CALL_END` event
- [ ] Default character limits match the table in Section 5.2 (read_file: 50k, shell: 30k, grep: 20k, etc.)
- [ ] Both character and line limits are overridable via `SessionConfig`

### 9.6 Steering

- [ ] `steer()` queues a message that is injected after the current tool round
- [ ] `follow_up()` queues a message that is processed after the current input completes
- [ ] Steering messages appear as SteeringTurn in the history
- [ ] SteeringTurns are converted to user-role messages for the LLM

### 9.7 Reasoning Effort

- [ ] `reasoning_effort` is passed through to the LLM SDK Request
- [ ] Changing `reasoning_effort` mid-session takes effect on the next LLM call
- [ ] Valid values: "low", "medium", "high", null (provider default) (certain providers might have other options like `xhigh`)

### 9.8 System Prompts

- [ ] System prompt includes provider-specific base instructions
- [ ] System prompt includes environment context (platform, git, working dir, date, model info)
- [ ] System prompt includes tool descriptions from the active profile
- [ ] Project documentation files (AGENTS.md + provider-specific files) are discovered and included
- [ ] User instruction overrides are appended last (highest priority)
- [ ] Only relevant project files are loaded (e.g., Anthropic profile loads CLAUDE.md, not GEMINI.md)

### 9.9 Subagents

- [ ] Subagents can be spawned with a scoped task via the `spawn_agent` tool
- [ ] Subagents share the parent's execution environment (same filesystem)
- [ ] Subagents maintain independent conversation history
- [ ] Depth limiting prevents recursive spawning (default max depth: 1)
- [ ] Subagent results are returned to the parent as tool results
- [ ] `send_input`, `wait`, and `close_agent` tools work correctly

### 9.10 Event System

- [ ] All event kinds listed in Section 2.9 are emitted at the correct times
- [ ] Events are delivered via async iterator or language-appropriate equivalent
- [ ] `TOOL_CALL_END` events carry full untruncated tool output
- [ ] Session lifecycle events (SESSION_START, SESSION_END) bracket the session

### 9.11 Error Handling

- [ ] Tool execution errors -> error result sent to LLM (model can recover)
- [ ] LLM API transient errors (429, 500-503) -> retry with backoff (handled by Unified LLM SDK layer)
- [ ] Authentication errors -> surface immediately, no retry, session transitions to CLOSED
- [ ] Context window overflow -> emit warning event (no automatic compaction)
- [ ] Graceful shutdown: abort signal -> cancel LLM stream -> kill running processes -> flush events -> emit SESSION_END

### 9.12 Cross-Provider Parity Matrix

Run this validation matrix -- each cell must pass:

| Test Case                                    | OpenAI | Anthropic | Gemini |
|----------------------------------------------|--------|-----------|--------|
| Simple file creation task                    | [ ]    | [ ]       | [ ]    |
| Read file, then edit it                      | [ ]    | [ ]       | [ ]    |
| Multi-file edit in one session               | [ ]    | [ ]       | [ ]    |
| Shell command execution                      | [ ]    | [ ]       | [ ]    |
| Shell command timeout handling               | [ ]    | [ ]       | [ ]    |
| Grep + glob to find files                    | [ ]    | [ ]       | [ ]    |
| Multi-step task (read -> analyze -> edit)    | [ ]    | [ ]       | [ ]    |
| Tool output truncation (large file)          | [ ]    | [ ]       | [ ]    |
| Parallel tool calls (if supported)           | [ ]    | [ ]       | [ ]    |
| Steering mid-task                            | [ ]    | [ ]       | [ ]    |
| Reasoning effort change                      | [ ]    | [ ]       | [ ]    |
| Subagent spawn and wait                      | [ ]    | [ ]       | [ ]    |
| Loop detection triggers warning              | [ ]    | [ ]       | [ ]    |
| Error recovery (tool fails, model retries)   | [ ]    | [ ]       | [ ]    |
| Provider-specific editing format works       | [ ]    | [ ]       | [ ]    |

### 9.13 Integration Smoke Test

End-to-end test with real API keys:

```
FOR EACH profile IN [openai_profile, anthropic_profile, gemini_profile]:
    env = LocalExecutionEnvironment(working_dir = temp_directory())
    session = Session(profile, env)

    -- 1. Simple file creation
    session.submit("Create a file called hello.py that prints 'Hello World'")
    ASSERT env.file_exists("hello.py")
    ASSERT env.read_file("hello.py") CONTAINS "Hello"

    -- 2. Read and edit
    session.submit("Read hello.py and add a second print statement that says 'Goodbye'")
    content = env.read_file("hello.py")
    ASSERT content CONTAINS "Hello"
    ASSERT content CONTAINS "Goodbye"

    -- 3. Shell execution
    session.submit("Run hello.py and show the output")
    -- Verify the agent executed the command (check event stream for shell tool call)

    -- 4. Truncation verification
    env.write_file("big.txt", REPEAT("x", 100000))
    session.submit("Read big.txt")
    -- Verify TOOL_CALL_END event has full 100k chars
    -- Verify the ToolResult sent to LLM has truncation marker

    -- 5. Steering
    session.submit("Create a Flask web application with multiple routes")
    session.steer("Actually, just create a single /health endpoint for now")
    -- Verify the agent adjusts its approach

    -- 6. Subagent
    session.submit("Spawn a subagent to write tests for hello.py, then review its output")
    -- Verify subagent tool calls appear in the event stream

    -- 7. Timeout handling
    session.submit("Run 'sleep 30' with the default timeout")
    -- Verify the command times out after 10s and the agent handles it gracefully
```

---

## Appendix A: apply_patch v4a Format Reference

The `apply_patch` tool (used by the OpenAI profile) accepts patches in the v4a format. This format supports creating, deleting, updating, and renaming files in a single patch.

### Grammar

```
patch       = "*** Begin Patch\n" operations "*** End Patch\n"
operations  = (add_file | delete_file | update_file)*

add_file    = "*** Add File: " path "\n" added_lines
delete_file = "*** Delete File: " path "\n"
update_file = "*** Update File: " path "\n" [move_line] hunks

move_line   = "*** Move to: " new_path "\n"
added_lines = ("+" line "\n")*
hunks       = hunk+
hunk        = "@@ " [context_hint] "\n" hunk_lines
hunk_lines  = (context_line | delete_line | add_line)+
context_line = " " line "\n"           -- space prefix = unchanged line
delete_line  = "-" line "\n"           -- minus prefix = remove this line
add_line     = "+" line "\n"           -- plus prefix = add this line
eof_marker   = "*** End of File\n"     -- optional, marks end of last hunk
```

### Operations

**Add File:** Creates a new file. All lines are prefixed with `+`.
```
*** Begin Patch
*** Add File: src/utils/helpers.py
+def greet(name):
+    return f"Hello, {name}!"
*** End Patch
```

**Delete File:** Removes a file entirely.
```
*** Begin Patch
*** Delete File: src/old_module.py
*** End Patch
```

**Update File:** Modifies an existing file using context-based hunks.
```
*** Begin Patch
*** Update File: src/main.py
@@ def main():
     print("Hello")
-    return 0
+    print("World")
+    return 1
*** End Patch
```

**Update + Rename:** Modify and rename in one operation.
```
*** Begin Patch
*** Update File: old_name.py
*** Move to: new_name.py
@@ import os
 import sys
-import old_dep
+import new_dep
*** End Patch
```

### Hunk Matching

The `@@` line provides a context hint (typically a function signature or recognizable line near the change). The implementation uses this hint plus the context lines (space-prefixed) to locate the correct position in the file. Convention: show 3 lines of context above and below each change.

When exact matching fails, the implementation should attempt fuzzy matching (whitespace normalization, Unicode punctuation equivalence) before reporting an error.

### Multi-Hunk Updates

A single Update File block can contain multiple `@@` hunks:

```
*** Begin Patch
*** Update File: src/config.py
@@ DEFAULT_TIMEOUT = 30
-DEFAULT_TIMEOUT = 30
+DEFAULT_TIMEOUT = 60
@@ def load_config():
     config = {}
-    config["debug"] = False
+    config["debug"] = True
*** End Patch
```

---

## Appendix B: Error Handling

### Tool-Level Errors

Tool execution errors are caught by the agent and sent to the LLM as error results (`is_error = true`). This gives the model the opportunity to recover, retry, or try a different approach.

| Error Type          | Example                                      | Recovery                         |
|---------------------|----------------------------------------------|----------------------------------|
| FileNotFound        | read_file on nonexistent path                | Model can search for correct path|
| EditConflict        | old_string not found or not unique           | Model can read file and retry    |
| ShellExitError      | Command returned nonzero exit code           | Model can inspect output and fix |
| ShellTimeout        | Command exceeded timeout_ms                  | Model can retry with longer timeout |
| PermissionDenied    | Write to protected path                      | Model can choose different path  |
| ValidationError     | Invalid JSON arguments for tool              | Model can fix the arguments      |
| UnknownTool         | Model called a tool not in the registry      | Error result tells model the tool name is wrong |

### Session-Level Errors

These errors affect the session itself, not individual tool calls.

| Error Type              | Retryable | Behavior                                        |
|-------------------------|-----------|--------------------------------------------------|
| ProviderError (429)     | Yes       | Retry with backoff (handled by Unified LLM SDK) |
| ProviderError (500-503) | Yes       | Retry with backoff (handled by Unified LLM SDK) |
| AuthenticationError     | No        | Surface immediately, session -> CLOSED           |
| ContextLengthError      | No        | Emit warning, session -> CLOSED                  |
| NetworkError            | Yes       | Retry with backoff (handled by Unified LLM SDK) |
| TurnLimitExceeded       | No        | Emit TURN_LIMIT event, session -> IDLE           |

### Graceful Shutdown Sequence

When an abort signal fires or an unrecoverable error occurs:

```
1. Cancel any in-flight LLM stream
2. Send SIGTERM to all running command process groups
3. Wait 2 seconds
4. Send SIGKILL to any remaining processes
5. Flush pending events
6. Emit SESSION_END event with final state
7. Clean up subagents (close_agent on all active subagents)
8. Transition session to CLOSED
```

---

## Appendix C: Design Decision Rationale

**Why provider-aligned toolsets instead of a universal tool set?** Models may be trained on specific tool formats. GPT-5.2-codex is trained on apply_patch; forcing it to use old_string/new_string editing produces worse results. Claude is trained on old_string/new_string; forcing it to use apply_patch produces worse results. The initial base for each provider profile should be the exact system prompt and tool harness from that provider's reference agent -- not a similar prompt, not similar tools, but a 1:1 byte-for-byte copy of the original prompt and tool definitions as the starting point. Then extend from there. Starting from the native toolset gives the best baseline experience because the model has been evaluated and optimized against exactly that harness.

**Why an extensible execution environment instead of a fixed local implementation?** A coding agent that can only run on the local machine is limited. By abstracting tool execution behind an interface, the same agent logic works in Docker (for sandboxing), in Kubernetes (for cloud execution), over SSH (for remote development), or in WASM (for browser-based agents). The abstraction costs almost nothing in complexity but opens up major deployment flexibility.

**Why head/tail truncation instead of just truncating from the end?** The beginning of a file (imports, type definitions, module docstring) or the beginning of command output (startup messages, headers) is often as important as the end (final results). Head/tail split keeps both. The explicit truncation marker tells the model exactly what happened so it can request specific parts if needed.

**Why not automatic compaction?** Compaction (summarizing conversation history to free context space) is complex, lossy, and implementation-specific. Different host applications have different requirements -- a CLI might want aggressive compaction, an IDE might want to restart the session, a batch system might want to fail. The spec provides context window awareness signals and leaves the strategy to the host application.

**Why does the loop use Client.complete() instead of the SDK's generate()?** The SDK's `generate()` function has its own tool execution loop, but it does not handle: output truncation with explicit markers, steering message injection between tool rounds, event emission for UI rendering, per-tool timeout enforcement, loop detection, or execution environment abstraction. The agent loop needs all of these, so it manages its own loop using the lower-level `Client.complete()`.

**Why 10-second default command timeout?** This matches codex-rs. Most developer commands (compile, lint, test a single file, git operations) complete in under 10 seconds. Long-running commands (full test suites, builds) should be explicitly requested with a longer timeout. The default protects against runaway processes without being so short that normal operations fail.

**Why exclude sensitive environment variables by default?** API keys, secrets, and tokens in the environment should not be visible to the LLM (which might include them in responses or log them). The default excludes `*_API_KEY`, `*_SECRET`, `*_TOKEN`, `*_PASSWORD` patterns. This is a safety default, not a security boundary -- the agent can still run commands that access these variables through the shell's own environment if needed.

# Kimi as a programming subagent — 2026-10-01

Follow-up: the later user-authorized [runtime validation](kimi_runtime_validation_2026-10-01.md)
now records real local delegation, coding edits, resumption and exit checks.
The research-only observations below remain historical; use root AGENTS.md
for the subsequently verified operational recipe.

The subsequent [ACP interaction capture](kimi_acp_interaction_2026-10-01.md)
also verifies real questions/answers, streamed multi-turn work, cancellation,
reconnection and bounded shutdown through a local ACP client. MCP wrapping
remains a separate design, not a configured server.

## Decision and evidence boundary

Use a bounded Kimi Code CLI subprocess as an external worker of the current
Codex task. The coordinator supplies the task and necessary context, collects
the proposed patch/findings, independently reviews them, applies approved
changes with `apply_patch`, and owns HPC verification and Git delivery.

This is documentation research, not an executed programming delegation. Local
`--version` and `--help` inspection succeeded; no prompt was submitted, no model
session was started by this task, and no Kimi-generated patch was produced.
Account access, inference, custom-profile enforcement, session resumption and
end-to-end coding remain unverified. No Atlas build/test, HPC submission,
mathematical acceptance or performance claim is part of this report.

The current session's native `spawn_agent` model choices do not include Kimi.
Calling an external CLI is a different integration: it has its own session,
credentials, permissions, logs and lifetime. It does not automatically inherit
the Codex conversation or appear as a native Codex subagent.

## Local installation and the earlier failures

Observed on 2026-10-01:

- Executable: `/home/hoxide/.kimi-code/bin/kimi`, version **2.1.1**.
- Binary SHA-256:
  `66f47536e40b02bb1d577cdd324768f73d39b8b9472e0ca140e73d8e225cd7de`.
- Configuration default model alias: `kimi-code/k3-256k`. Alias existence is
  not evidence of current account entitlement or successful authentication.
- Data root exists at `/home/hoxide/.kimi-code/`; the old
  `/home/hoxide/.kimi/` path does not exist. Only nonsensitive configuration
  fields and directory metadata were inspected; credentials were not printed.
- Local capability output and hashes are recorded in
  [kimi_cli_capabilities_2026-10-01.json](kimi_cli_capabilities_2026-10-01.json).

The earlier incident report records **0.42.0**, a rejected `--plan -p`
combination, and a subsequent `storage write failed: unrecognized I/O error`.
Those observations remain historical evidence; do not rewrite them as 2.1.1
results. The old diagnosis was a possible unwritable session store, not a
proven log-initialization failure. This research does not prove it repaired.
The earlier denied repository-payload retry was not repeated.

Two product generations are easily confused. The old MoonshotAI/kimi-cli
documentation describes Python-module tool names, YAML agent definitions,
`--print`, `--work-dir`, and Wire mode. The installed binary points to
MoonshotAI/kimi-code. Its help describes Markdown agent definitions and `-p`
as the noninteractive entry point. Do not transfer old flags or tool names to
2.1.1 without checking the installed help.

## Four distinct integration choices

| Choice | Who coordinates | Appropriate use |
| --- | --- | --- |
| CLI subprocess using `-p` | This Codex task | Recommended first integration: bounded task, finite process, captured result |
| Kimi's internal `Agent` delegation | A Kimi main agent | Kimi coordinates its own `coder`, `explore`, `plan` or custom workers |
| `kimi acp` | An ACP client | Longer-lived structured sessions and permission interaction; requires a JSON-RPC client |
| Kimi as a Codex model provider | A separately configured Codex client | Changes the model backend; does not add a Kimi choice to this running task's tool schema |

Kimi documents Responses API provider support for Codex. That is a separate
option, not necessary for a CLI worker and not configured here. ACP is an agent
protocol; it is not an MCP server. Kimi consuming MCP tools does not itself
expose a callable `kimi_task` MCP tool. Such a wrapper would be a separate
implementation with its own verification.

## Command contract for installed 2.1.1

The following is a launch pattern, not a claim that the example was run. Create
and review the profile and prompt first; `worker.md` below is an example path.
Run from the intended existing working directory (or set subprocess `cwd`):

```sh
/home/hoxide/.kimi-code/bin/kimi \
  --model kimi-code/k3-256k \
  --agent-file /absolute/path/to/worker.md \
  --output-format stream-json \
  --prompt 'A fully specified, bounded task with the necessary input text'
```

Use a subprocess argument array and `shell=False` in an orchestrator; put the
frozen prompt text in one argument. Never interpolate model output into shell
code. The subprocess working directory replaces the old `--work-dir` recipe.

Important constraints from the version-matched command reference:

- `-p` already runs without a TUI and uses automatic permission handling.
  Static deny rules still apply. It is not a read-only mode.
- Do not combine `-p` with `--plan`, `--yolo`, or `--auto`.
- Use `--output-format stream-json` for integration. Stdout contains JSONL
  assistant/tool messages, not one JSON object or just the final answer.
  Progress/resumption messages use stderr; retain both streams and exit status.
- `--agent-file` selects one Markdown profile for a **new** session. It cannot
  be combined with `--agent`, `--session`, or `--continue`.
- Continue the exact existing session with `--session <actual-id> -p <text>`;
  omit the profile flags because the session retains its profile. Avoid
  `--continue` in concurrent orchestration: it selects the latest session in
  that working directory, which may belong to another task.
- `--print`, `--quiet`, `--wire`, `--work-dir` and the older loop-limit CLI
  flags are absent from the installed help; they are not this workflow's API.

The upstream configuration-overrides page contains contradictory examples,
including `--yolo -p` despite listing it as invalid. This report follows the
dedicated command reference and the earlier observed conflict; it does not
treat every upstream example as validated.

## Start with a worker that returns a patch

A programming worker does not need permission to execute commands or edit the
shared checkout in order to propose a useful patch. Supply the necessary source
excerpts and independently justified expected behavior in the task packet.
This example profile intentionally has no tools:

```markdown
---
name: atlas-patch-proposer
description: Propose a small mechanical programming patch from supplied inputs
tools: []
subagents: []
---

You are a bounded programming assistant for an Atlas-Rust coordinator.
Use only the task packet supplied in the prompt. If necessary context is
missing, report the missing input instead of inventing repository facts.
Do not run commands, edit files, create worktrees, contact HPC, commit, push,
delegate, or decide mathematical correctness or acceptance.
Preserve existing user changes. Limit the proposed diff to the explicit
owned-file list. Tests may be proposed but must be marked NOT RUN.
Your final message is the complete handoff: findings with source locations,
one proposed unified diff if requested, assumptions, and unverified checks.
```

This is documentation-only scaffolding; it has not been installed or executed.
The profile replaces the default prompt and has no `${base_prompt}` or
`${agents_md}` expansion. The coordinator must explicitly include the relevant
repository requirements and context. This reduces accidental broad context
inclusion, but is not an audited guarantee about the complete outbound request
or client startup side effects; inspect enabled hooks/plugins/configuration
before an actual delegation.

For repository exploration, an alternative profile can allow exactly
`Read`, `Grep`, and `Glob`, while continuing to exclude command, write, network,
and delegation tools. Tool names are case-sensitive. A tool allowlist controls
tool classes, **not file paths**: a prompt saying "read only file X" is not a
filesystem sandbox. Use supplied excerpts with `tools: []` when an exact data
boundary matters. Actual direct-edit work needs reviewed path restrictions,
explicit ownership and before/after state capture.

Omitting `tools` permits all tools. `disallowedTools: ['*']` is not a universal
deny; the documented universal no-tools form is `tools: []`. A Claude-style
`model:` field in an agent's frontmatter is ignored; select the external
worker's model with `--model`.

## Kimi-native subagents

Within a Kimi conversation, `Agent` can delegate to `coder` (implementation),
`explore` (repository investigation), or `plan` (planning). Custom Markdown
profiles can be discovered under project `.kimi-code/agents/` or
`.agents/agents/`, or user `~/.kimi-code/agents/`. This research created none.

Give each worker its own task packet: normal delegation uses an independent
context and does not give it the parent's full conversation. Context isolation
does not give file isolation, so concurrent writers still need non-overlapping
ownership. Prefer one bounded worker until one actual handoff is reviewed.

For internal model selection the documented mechanism is `[secondary_model]`
in Kimi configuration (including a default, candidate pool and optional
`force`), not the profile's ignored `model` field. Internal child timeouts,
background work and extra model tokens also require separate accounting.
No such configuration or nested delegation was enabled here.

## Task packet and collection protocol

Before any model-bearing invocation, freeze a durable record containing:

1. Task ID, timestamp, CLI version/binary hash, model alias, working directory,
   source commit and relevant dirty-file hashes.
2. Exact prompt bytes and hash, every supplied input, profile bytes/hash,
   permitted read scope, owned-file list and explicit exclusions.
3. Deliverable and acceptance criteria: e.g. a proposed mechanical diff and
   fixture scaffolding; mathematical expected results come from independent
   evidence, never from the worker's invention.
4. Outer wall-clock limit, step/attempt limits, no recursive delegation and
   the owner responsible for interruption, review and verification.

Version-matched environment controls include
`KIMI_CODE_NO_AUTO_UPDATE=1`, `KIMI_LOOP_MAX_STEPS_PER_TURN`,
`KIMI_LOOP_MAX_ATTEMPTS_PER_STEP`, and `KIMI_CODE_INFINITE_RETRY=0`.
Use positive finite loop limits and an outer process deadline. Do not mistake
a tool's short polling interval for a deadline. A production wrapper must
terminate/reap its process group on interruption or timeout and reconcile any
background work before retrying. Prompt mode can otherwise remain alive while
background tasks finish. No wrapper was implemented in this research.

Afterward, retain the actual model/session identifiers when available, raw
output in an appropriate private evidence location, output hashes, exit state,
the returned patch, and a reviewed/redacted report. Do not publish OAuth data,
config dumps or unredacted session exports. Classify timeouts and partial
changes separately from successful completion; never blindly rerun an
interrupted editor. An exit-zero response is not code acceptance.

The coordinator reviews every changed byte or claim, applies accepted work,
runs the relevant HPC gate, and records accepted/rejected suggestions and a
lesson derived from that actual invocation. Kimi must not submit jobs itself,
use a new worktree, test locally, change the mathematical acceptance ledger,
or commit/push. Repository limits and the current smallest gate remain in force.

## Storage failure diagnosis

The documented root is `~/.kimi-code`, including configuration, credentials,
session state and logs. `KIMI_CODE_HOME` relocates **all** of these, not just
logs. Pointing it at an empty directory can therefore lose access to the
configured provider and authentication. It is not a drop-in fix for the old
storage error. Check actual runtime identity, allowed write access, filesystem
state and redacted stderr/session diagnostics before attributing the cause.
Do not weaken a sandbox, copy credentials into the repository, or retry a
previously denied payload through another mechanism.

## Sources and reproducibility

Primary documentation was read on 2026-10-01. The release tag
`@moonshot-ai/kimi-code@2.1.1` resolves through annotated tag
`a00639d0654d2c91c6f0c1388267b62ad51f8b9c` to commit
`f67e6398fb3210ad8ace970e2dfd5bcc984ed61f`.
The command, agents, and configuration pages were also hash-compared
with main's tree `21406fb4c805cc8c715e6d1f16ad3fb5f25f4fe3` and are identical;
the environment page differs, so the release version is the reference below.

| Release documentation path | SHA-256 |
| --- | --- |
| `docs/en/reference/kimi-command.md` | `8722db5aa446e4f65e4146c5684b15ff5c9a143c6e60bcb65dbef5efa6e481d0` |
| `docs/en/customization/agents.md` | `e0027474fe809d42a398e860ffa66c825553684b6db34bca29ca9a407f7b4bb6` |
| `docs/en/configuration/config-files.md` | `26a18ec04cfd571e49a8d5f7b0ca8959d04d7b22234f6374da20825cd9ef519a` |
| `docs/en/configuration/env-vars.md` | `4cde4977adc9a10a3022ce91205cde94dbecc7f9e5cacc2a73c5bbe9d80bd94b` |

- [Version-pinned CLI reference](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/docs/en/reference/kimi-command.md)
- [Version-pinned agent format](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/docs/en/customization/agents.md)
- [Version-pinned configuration](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/docs/en/configuration/config-files.md)
- [Version-pinned environment controls](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/docs/en/configuration/env-vars.md)
- [Kimi data locations](https://www.kimi.com/code/docs/kimi-code-cli/configuration/data-locations.html)
- [Kimi ACP](https://www.kimi.com/code/docs/kimi-code-cli/reference/kimi-acp)
- [Kimi's Codex provider guide](https://www.kimi.com/code/docs/third-party-tools/codex.html)
- [OpenAI subagent documentation](https://learn.chatgpt.com/docs/agent-configuration/subagents)

No local temporary path, source clone, worktree, transport archive, new skill,
agent configuration or Kimi task session was created. Documentation-only
research does not change a mathematical/algorithmic KB page. Existing dirty
repository work belongs to other work and is excluded from this task's commit.
The read-only worktree preflight matched all25 registered entries before
editing and again before staging. The staged Git whitespace check passed.
This documentation task introduces no executable source change requiring an
HPC build/test; the unexecuted delegation example remains explicitly unverified.

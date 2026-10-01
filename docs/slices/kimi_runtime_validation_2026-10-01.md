# Kimi runtime validation — 2026-10-01

Status: **local Kimi integration verified**, including real programming file
edits and session continuation. This is not mathematical acceptance or a test
pass for the generated Python program, which was never executed.

The user explicitly requested actual Codex-to-Kimi programming delegation and
asked that verified operation, precautions and exit mechanisms be recorded in
AGENTS.md only after successful execution. The user explicitly authorized
local Kimi integration checks; Atlas builds and program tests remain HPC-only.
Use the installed local CLI and existing local account. No credentials are
copied to HPC, the repository or the temporary working directory.

## Frozen scope before first invocation

Coordinator-owned files: `tools/kimi_subagent.py`, `.agents/kimi/probe.md`,
`.agents/kimi/editor.md`, this report and its evidence directory, later the
verified AGENTS.md addition and handoff/index updates. Existing user edits
remain outside this task's commit.

Kimi owns no repository file. The first call has no owned file or tool and
receives exactly this prompt (also frozen verbatim by the runner before launch):

```text
You are being called as an external subagent of Codex. This is a connection and normal-exit check. Do not use tools. Reply with exactly this one line and nothing else: CODEX_KIMI_READY_20261001
```

Each later invocation must freeze its exact prompt, profile, executable hash,
model, working directory, owned-file hashes and deadline in its repository
evidence `request.json` before launching. Preserve failure records as well as
success. A single context-managed temporary directory holds synthetic inputs
only and must be removed before handoff. No new Git worktree is permitted.

Planned checks: real response and normal exit; bounded Read/Edit programming
task with independent file inspection; continuation of the exact session;
SIGINT and SIGTERM cancellation; outer timeout and process-group cleanup.
The synthetic program is not executed locally. File/content inspection and
subprocess lifecycle checks are within the user's Kimi-integration exception.

The preparation statement above was written before any model-bearing call.
The following records describe the subsequently observed results.

## Observed environment and evidence

Local executable `/home/hoxide/.kimi-code/bin/kimi`, version2.1.1, SHA-256
`66f47536e40b02bb1d577cdd324768f73d39b8b9472e0ca140e73d8e225cd7de`.
The selected alias `kimi-code/k3-256k` maps in local configuration to provider
`managed:kimi-code`, model `k3-256k`. Real responses demonstrate account access
for this observed run, not future availability or a general entitlement claim.

All raw requests, prompts, agent profiles, stdout/stderr, process identities,
results, synthetic before/after files and controller observations are under
[`../evidence/kimi-runtime-20261001/`](../evidence/kimi-runtime-20261001/).
`review.json` contains the independent coordinator inspection; `manifest.json`
hashes the retained artifacts. No credential/configuration file was copied.

| Invocation | Expected observation | Wrapper exit | Observed duration |
| --- | --- | ---: | ---: |
| `01-connect` | Exact `CODEX_KIMI_READY_20261001` response, normal exit | 0 | 5.564s |
| `02-edit` | Read/Edit repairs only `labels.py` | 0 | 14.474s |
| `03-resume` | Same session recalls receipt and adds requested docstring | 0 | 18.030s |
| `04-sigint` | Startup cancellation by SIGINT | 130 | 1.506s |
| `05-sigterm` | Startup cancellation by SIGTERM | 143 | 1.334s |
| `06-timeout` | 2-second deadline stops the CLI | 124 | 2.238s |
| `07-kill-fallback` | Controlled SIGSTOP forces TERM-to-KILL escalation | 124 | 4.266s |
| `08-inflight-sigint` | SIGINT after a real Read tool result | 130 | 5.579s |
| `09-invalid-profile` | Malformed YAML is retained as a startup failure | 1 | 1.576s |
| `10-final-normal` | Final runner still returns the exact connection receipt | 0 | 5.761s |
| `11-final-sigterm` | Signal sent using the final recorded wrapper PID | 143 | 2.355s |

Durations measure worker launch through cleanup, excluding the preceding
version inspection and binary hashing. They are not mathematical benchmarks.
All eleven invocations had zero remaining live process-group members after
cleanup. The SIGKILL check deliberately paused its own Kimi group; it is not
evidence of a naturally hung Kimi service. `04` and `05` ended before a session
was observed; `08` demonstrates interruption after model-driven tool activity.
The invalid profile is an intentional rejection, not a successful inference.

Invocations01–09 used runner SHA
`679dbdd4f18fe02410bdbfaec4c2aa7351a4c9c1e720d32c1bbeb0f88e12e631`;
its exact bytes are retained as `runner-before-pid-metadata.py.txt`. Final
runner SHA is
`beda5202021bff7f78f39af64f658c0f7e7cd501155d4474647b05b78e04cc0e`.
The only change adds the runner hash and explicit wrapper PID to evidence;
process control is unchanged. Invocations10–11 exercised the final file.

## Independent review of the programming handoff

The initial two-line synthetic function returned raw comma-separated pieces.
Kimi used one `Read` and one `Edit` to produce:

```python
def split_labels(text: str) -> list[str]:
    return [label for label in (part.strip() for part in text.split(",")) if label]
```

Coordinator inspection confirms that the expression strips each piece before
filtering and retains iteration order and duplicates. The signature is
unchanged and no dependency was introduced. The tool trace and complete scratch
directory inventory show only `labels.py` was edited; `untouched.txt` remained
byte-identical. This is reviewed source text, not an executed function test.

The follow-up prompt did not repeat receipt `LABELS_WORKER_71E4`. Resuming
`session_f9c7bd36-01b9-4d68-82a7-1f6701e68493` returned that receipt,
`RESUME_EDIT_OK`, and added exactly the requested docstring while preserving
the implementation. The resumed session again used only Read/Edit. Both
handoffs correctly said program tests had not run. All proposed edits in this
synthetic exercise were accepted by inspection; none were adopted into Atlas
source and no Kimi suggestions were rejected or repurposed as math evidence.

Session IDs were taken from `meta/session.resume_hint` messages or the CLI's
workspace-filtered session list, not guessed. `session_inventory.json` records
the exact six persisted local sessions and their runtime directories; the other
four working directories have no observed session. The coding session has two
invocations. These synthetic diagnostic sessions remain in Kimi's normal local
runtime store; no other user's session or credential was deleted or copied.

## Verified use and exit contract

The authoritative concise instructions are now in root `AGENTS.md`, with
`.agents/kimi/probe.md`, `.agents/kimi/editor.md` and
`tools/kimi_subagent.py` as the reusable artifacts. Run from an existing
workspace with an exact task packet and owned-file list. The runner never
starts a native Codex child task or a new worktree. Its Python subprocess uses
an argument list, not shell interpolation.

Output is JSONL, with a version metadata record before responses and a session
metadata record after the final assistant message. Preserve all messages;
do not take the last JSON line as the answer. The request record binds the
profile snapshot actually passed to Kimi. The normal credential root remains
untouched, and the runner changes only a small nonsecret environment-control
allowlist for finite execution and repeatable versions.

The `wrapper_pid` in `process.json` is the cancellation target. SIGINT and
SIGTERM to that process are handled by the runner, which signals only its
child process group and escalates after the grace period. Its deadline is
independent of a tool call's polling/yield interval. On return, inspect
`result.json`, output, actual file changes and remaining process members.
An interrupted edit is not rolled back, and a model/service request may have
already consumed tokens before local cancellation. Do not blindly retry.

The wrapper's own SIGKILL, uninterruptible kernel processes, detached daemons,
host crashes, and a changed tool profile are outside this proof. Tool allowlists
are not path sandboxes. The runner does not implement a security boundary around
the operating system or turn exit0 into an acceptance decision. Keep Shell,
network and nested agents disabled for this verified route.

Reusable lessons from the actual invocations:

1. The installed local account works without copying credentials or changing
   `KIMI_CODE_HOME`; the historical storage failure was not reproduced. Its
   old root cause remains unproven, so do not claim to have diagnosed it.
2. Read/Edit restriction is sufficient for bounded programming edits. A
   complete task packet and explicit nonexecution requirement were honored.
3. Explicit session continuation preserves useful task memory and the agent
   profile. A trailing metadata record must not be mistaken for the answer.
4. Signal the wrapper and allow it to reap its own process group; retain
   separate interrupted/timeout/error states and review partial edits.

## Temporary-path retirement

Every task-created temporary directory used a context manager, and all were
removed before handoff. Individual controller records retain this observation;
the coordinator checked their absence again during evidence review:

Independent scenarios used sequential scratch directories, with at most one
active at a time; the coding task and its continuation shared the same directory.

- `/tmp/codex-kimi-connect-7j8qhgvc` — REMOVED
- `/tmp/codex-kimi-edit-gi3ingyo` — REMOVED
- `/tmp/codex-kimi-exit-lu4gr37h` — REMOVED
- `/tmp/codex-kimi-exit-42mq4jxf` — REMOVED
- `/tmp/codex-kimi-exit-klprhrwx` — REMOVED
- `/tmp/codex-kimi-exit-ly157o0d` — REMOVED
- `/tmp/codex-kimi-inflight-w8q51gia` — REMOVED
- `/tmp/codex-kimi-invalid-47uuwus2` — REMOVED
- `/tmp/codex-kimi-final-ornpmvgw` — REMOVED
- `/tmp/codex-kimi-final-syw69cg2` — REMOVED

No transport or worktree was created. No existing temporary directory or
unrelated user process was removed. No Atlas code, math acceptance ledger or
mathematical KB content changed; no HPC job was submitted.

Git whitespace inspection passes for authored changes. The intentionally
invalid-profile stderr retains one trailing space in Kimi's raw YAML diagnostic
(`09-invalid-profile/stderr.txt`, line6); it is excluded from that editorial
whitespace check and retained byte-for-byte under its evidence hash. The
worktree registry still matches all25 entries before handoff.

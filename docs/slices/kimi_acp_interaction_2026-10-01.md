# Codex–Kimi bidirectional interaction — 2026-10-01

Preparation record, frozen before this task's new model invocations: the user
requests a working interaction method, building on the already verified CLI
worker. Local Kimi integration checks are explicitly authorized. Atlas code,
program tests, mathematical acceptance and HPC state are outside this task.

Coordinator-owned additions: `.agents/kimi/interactive.md`,
`tools/kimi_acp_session.py`, a bounded integration capture driver, this report,
`docs/evidence/kimi-acp-20261001/`, and later AGENTS/handoff/index additions.
Kimi owns no repository or scratch file: the new profile exposes only
AskUserQuestion and produces proposed source as text. Synthetic task text,
profile, model, CLI hash and request scope are frozen before each launch/send.
Use one context-managed scratch directory, then remove it before handoff.

The first CLI prompt will create a restricted session using the existing
runner. An ACP client will attach to that exact session, retain streamed
events, answer a synthetic programming question, send a follow-up, cancel an
in-flight question and verify the session remains usable. Close the live
session and reap the exact owned process group. Do not delete saved sessions,
log out, alter global configuration, expose credentials or create a worktree.

Research uses the installed Kimi Code2.1.1 binary and official source at
`MoonshotAI/kimi-code@f67e6398fb3210ad8ace970e2dfd5bcc984ed61f`.
The ACP CLI entry point does not forward global `--agent-file`/`--model`
options. Therefore bootstrap a restricted CLI session and load its explicit
ID instead of assuming `kimi --agent-file ... acp` enforces a profile.

Results below will distinguish observed behavior from a proposed MCP wrapper.

## Result: a working two-way local subagent channel

**Verified:** Codex can drive the local Kimi account through a persistent ACP
subprocess, receive progress and questions, answer those questions, send later
turns, cancel a waiting turn, reconnect and close the worker. The reusable
client is `tools/kimi_acp_session.py`; it needs only Python's standard library
and the already installed local Kimi. No service port or additional API key
is required. Neither global Kimi nor global Codex configuration was edited.

The existing `tools/kimi_subagent.py` remains the verified route for a bounded
Read/Edit task. The ACP route verified here uses
`.agents/kimi/interactive.md`: **AskUserQuestion only**, with proposed code
returned as text. Codex reviews and applies any accepted proposal, then runs
the project's required verification on HPC. These two profiles have different
tool boundaries; do not load an unrestricted session into this ACP client.

```text
Codex coordinator
  | explicit task / answer / follow-up / cancel / close
  v
tools/kimi_acp_session.py  <-- JSONL controller commands and events
  | JSON-RPC over stdin/stdout (ACP v1 negotiated in this capture)
  v
local kimi acp  <--> existing local login <--> Kimi model
  | streamed response / question / tool status / turn result
  +----------------------------------------------------> Codex
```

Codex's terminal tools can own the client subprocess and send lines to its
stdin. In this capture, `tools/kimi_acp_probe.py` served as the bounded
coordinator driver and exercised the actual local Kimi process through pipes.
This is an external worker; it does not add Kimi to the current native
`spawn_agent` model enumeration or give Kimi the parent conversation.

## Independent inspection of the actual runs

Evidence: [`../evidence/kimi-acp-20261001/`](../evidence/kimi-acp-20261001/).
`review.json` records independent inspection of all twelve process captures;
`manifest.json` binds117 retained files and has SHA-256
`53de204a7afa569c8282588f3205acbd356acaf8705e2bf5a96c2f33ad7d943f`.
There were **twelve Kimi processes and fourteen model-bearing prompts**, across
three synthetic sessions. Nine processes ran ACP; three bootstrapped CLI sessions.
Deadline and signal captures replayed history but sent no new model prompt.

| Capture | Scope and observed result | Client/runner exit |
| --- | --- | ---: |
| `01-first/01-bootstrap` | Restricted session, exact `ACP_BOOT_READY` | 0 |
| `01-first/02-dialogue` | Failed interaction: model skipped the required question; driver timed out waiting, then stopped the client | 143 |
| `02-manual-context/01-bootstrap` | Fresh restricted session, exact bootstrap receipt | 0 |
| `02-manual-context/02-dialogue` | Real question/answer, coding proposal, remembered follow-up, busy-turn rejection, cancellation, recovery, explicit close | 0 |
| `02-manual-context/03-reload` | New ACP process, same persisted session recalls marker and chosen policy; controller EOF closes it | 0 |
| `02-manual-context/04-deadline` | Three-second client deadline, graceful ACP close | 124 |
| `02-manual-context/05-signal` | SIGTERM to the client after readiness, graceful ACP close | 143 |
| `03-settled-cancel/01-bootstrap` | Final-client capture: restricted session and exact bootstrap receipt | 0 |
| `03-settled-cancel/02-dialogue` | Final-client full dialogue/recovery; cancellation also settles the pending question RPC | 0 |
| `03-settled-cancel/03-reload` | Final-client new-process history recall and EOF shutdown | 0 |
| `03-settled-cancel/04-deadline` | Final-client three-second deadline and clean shutdown | 124 |
| `03-settled-cancel/05-signal` | Final-client SIGTERM and clean shutdown | 143 |

All twelve child process groups had zero live members after cleanup. Kimi
itself exited0 in the ACP captures, including timeout/signal: the **client's**
exit/status distinguishes why work stopped. These ACP captures required no
TERM/KILL escalation. The older CLI runner's separate forced-kill capture is
documented in the runtime report; do not present it as an ACP hang test.

The failed session was `session_0094f3da-7f31-48ca-bff2-33ac11b0c41c`.
The first successful dialogue session was
`session_f91c2372-6fc2-409d-9762-697d0fd1a490`; the final-client session was
`session_165e898d-763b-41b1-8b3a-5eb9a8c07d71`.
All used CLI2.1.1 and `kimi-code/k3-256k`. ACP `session/load` independently
returned that model alias. The executable SHA remains
`66f47536e40b02bb1d577cdd324768f73d39b8b9472e0ca140e73d8e225cd7de`.

The first model response wrongly followed the auto-mode reminder retained
from CLI `-p`, despite a successful ACP switch to `default`. It proposed both
duplicate policies instead of asking. **Rejected:** that unilateral policy
choice and its claimed current auto-mode state. Its proposed source was not
applied. This failure remains in raw evidence; it is not counted as a working
question round trip. The capture driver's original timeout is recorded as
`Empty: `; the trace shows the actual cause was completion without a question.

The corrected client keeps the successful `session/set_mode` call and prefixes
each new task with a truthful current-state notice: default/manual is now
active, while the earlier auto reminder describes the bootstrap turn. It does
not grant extra tools or change account configuration. The old client/driver
bytes are retained under `01-first/*.py.txt`.

Final evidence inspection found cleanup warnings in the second capture: the
cancelled turn's unanswered elicitation RPC remained pending until connection
close, and redundant cancellation after close targeted an already disposed
session. The final client settles pending forms with `action: cancel` and only
sends a session cancellation while a turn is active. The intermediate bytes
are preserved under `02-manual-context/*.py.txt`. The full `03-settled-cancel`
capture repeats the workflow with the final client; every captured stderr is
empty, every pending question has an accept/cancel response, and all process
groups are clean. Final client SHA:
`2afe62107a1082d49d1d96ee8076fd6a16746c42a904d5045a42e4a032fb5298`.

Inspection of the corrected wire trace confirms:

1. Kimi called `AskUserQuestion`, generating `elicitation/create`. The
   coordinator answered its actual request ID with `Preserve duplicates`.
   Only after the answer did Kimi produce the requested replacement function.
2. The second capture's comprehension and final capture's explicit loop strip
   each part and then filter empty labels; order and duplicates are retained.
   This is source inspection,
   **not execution of the generated Python program**. No file was edited.
3. A second turn recalled `ACP_CODER_82B7`, preserved the chosen policy and
   added a suitable docstring. That prompt did not repeat the marker.
4. During a second pending question, an additional prompt was rejected by the
   client, preventing two competing turns. `session/cancel` then settled the
   original prompt with `stopReason: cancelled`; no fabricated answer was sent.
5. The same live session returned exact `ACP_RECOVERED_82B7` after cancellation.
   A new process later loaded its saved history and correctly recalled both
   the original marker and the earlier duplicate-policy answer.

The final proposal also offered a walrus-expression variant described as
requiring Python3.8+. **Rejected:** that version claim for the complete shown
function: its `list[str]` annotation, without postponed evaluation, needs
Python3.9+ ([PEP585](https://peps.python.org/pep-0585/)). The walrus operator
alone is a different compatibility constraint. The final explicit-loop
proposal retains the input signature and was accepted as text for its scoped
logic, with this version qualification; none was adopted into Atlas.
No build, generated-program test, fixture, HPC job,
mathematical acceptance or performance claim occurred. The scratch directory
remained empty in both successful captures. All context-managed temporary paths
are removed: `/tmp/codex-kimi-acp-v5iolxif`,
`/tmp/codex-kimi-acp-v0r5jtwl` and `/tmp/codex-kimi-acp-3mu4vpvi`.
Three synthetic sessions remain in Kimi's normal
local runtime store. Unrelated sessions and credentials were untouched.

## Reusable operation

First use the existing runner with `.agents/kimi/interactive.md` to create a
new restricted session, with a short frozen bootstrap prompt and an existing
work directory. Read its actual `session.resume_hint.session_id` from the
JSONL metadata. Keep its evidence/profile snapshot and the work directory.
Do not reuse a concurrently running session or another task's session.

Then start the client with pipes or an interactive terminal:

```sh
python3 -B tools/kimi_acp_session.py \
  --session session_ACTUAL_ID \
  --work-dir /absolute/path/to/the/same/workspace \
  --model kimi-code/k3-256k \
  --output-dir /absolute/path/to/new/acp-evidence \
  --timeout 300
```

Wait for the `ready` event before sending one JSON object per stdin line.
Example commands (IDs and field names in `answer` must come from the actual
`question.requestedSchema`, not these illustrative values):

```json
{"command":"prompt","text":"Review this supplied function; ask if a requirement is missing, then propose the code as text: ..."}
{"command":"answer","request_id":0,"content":{"q0":"Preserve duplicates"}}
{"command":"prompt","text":"Keep that policy and add a docstring."}
{"command":"cancel"}
{"command":"close"}
```

These lines illustrate different states, not a batch to submit all at once:

| State/event | Coordinator action |
| --- | --- |
| `ready` | Send the first task packet with explicit context and acceptance criteria |
| `update` | Read response/tool progress; retain the full evidence |
| `question` | Answer from known requirements, or ask the user if information/authorization is genuinely missing |
| `answer` sent | Let the same pending prompt continue; do not start a second prompt |
| `turn_end` with `end_turn` | Inspect the proposed code/result; send another prompt or close |
| `turn_end` with `cancelled` | Inspect partial output; a later prompt may reuse this session |
| `turn_end` with `error` | Record a failed turn; closing the process does not turn it into success |
| `closed` | Check result status, process exit and remaining group members |

The client supports required single-choice enum fields in question forms.
It deliberately rejects malformed/stale answers, unknown commands and an
overlapping prompt. Tool-approval requests are denied; filesystem and terminal
reverse RPCs are not enabled. These policies and the profile are **not an OS
sandbox**. Do not infer that ACP mediates every filesystem operation: the
upstream ACP filesystem backend delegates several non-text operations locally.

`session/load` replays earlier messages before readiness. Those replayed chunks
are historical output, not a new answer. `session/cancel` is a notification;
wait for the outstanding prompt's result rather than waiting for a separate
cancel response. Pending question RPCs must also receive `action: cancel`;
otherwise the model turn may stop while its reverse request remains unsettled.
Closing disposes a live session; it does **not** delete saved
history or log out. Normal close/EOF, deadline and SIGTERM were exercised.

For emergency shutdown, signal only the recorded client `wrapper_pid` with
SIGINT/SIGTERM. The client attempts cancel/close, closes Kimi stdin, waits,
then uses the earlier runner's bounded TERM/KILL cleanup for its own process
group. Never use `pkill kimi` or SIGKILL the wrapper. Host crashes, detached
daemons and uninterruptible processes remain outside the demonstrated cleanup.
Exit0 means closed,124 deadline,130 SIGINT,143 SIGTERM,1 other client failure;
inspect every `turn_end` separately. Interruption may already have consumed
model tokens and never authorizes blind retries or rolls back previous edits.

The replayable integration driver is:

```sh
python3 -B tools/kimi_acp_probe.py --output-dir /absolute/path/to/new/probe-evidence
```

It uses the real account/model and therefore sends new synthetic prompts.
Its local use is covered by this task's explicit integration exception; it
does not authorize local Atlas or generated-program execution.

## Why ACP now, and where MCP fits

Official [Kimi ACP documentation](https://www.kimi.com/code/docs/en/kimi-code-cli/reference/kimi-acp)
describes the session/prompt/cancel and reverse-question surfaces exercised
above. Release-pinned implementation references are the
[CLI entry point](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/apps/kimi-code/src/cli/sub/acp.ts),
[question mapper](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/packages/acp-server/src/question.ts),
and [filesystem backend](https://github.com/MoonshotAI/kimi-code/blob/f67e6398fb3210ad8ace970e2dfd5bcc984ed61f/packages/acp-server/src/acp-fs/acpFsService.ts).

Codex supports local stdio MCP servers according to its
[official MCP guide](https://learn.chatgpt.com/docs/extend/mcp). Therefore a
future adapter could expose `kimi_start`, `kimi_poll`, `kimi_answer`,
`kimi_followup`, `kimi_cancel` and `kimi_close`, with a task ID, explicit session
ownership and a cursor for newly observed events. Long turns should return a
handle and be polled, allowing questions and cancellation while work runs.
The adapter must retain the same restricted profile, question states, finite
deadline and child cleanup; it must clean up on client disconnect too.

**This MCP adapter is a design, not an installed or tested server.** ACP and
MCP are distinct protocols. Do not configure an MCP server command as plain
`kimi acp`, or as `kimi_acp_session.py`: neither implements MCP initialize /
tools/list / tools/call. The actual working method in this task needs no MCP
installation: Codex controls the supplied ACP client through its terminal
tools. Recheck version, protocol negotiation, profile persistence and question
behavior when upgrading either side; do not apply old `--wire` SDK recipes to
the installed2.1.1 CLI.

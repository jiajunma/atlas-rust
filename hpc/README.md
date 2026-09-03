# HPC verification

Use local execution for small, bounded checks when practical. Use the XMU
login node for checkout, source synchronization, and dependency/build
preparation, and run large Rust suites, the reference Atlas executable, CWEB
expansion, differential comparisons, and benchmarks through SLURM on a compute
node.

## Repository location and toolchain

The shared project directory is `/public/home/majj/atlas-rust` on XMU. The
login node has internet access for initial dependency acquisition; compute
nodes do not. The repository follows the installed stable toolchain, with
Rust 1.90 as the enforced minimum because Malachite 0.10 requires it. Install
or update a suitable stable toolchain on the login node before building:

```bash
rustup toolchain install stable --profile minimal --component clippy,rustfmt
```

Build binaries and cache dependencies on the login node before submitting
jobs. Every job records the commit, dirty-tree state, Rust toolchain, reference
Atlas revision, CWEB version, SLURM job/node, fixture manifest, exit status,
and report checksums.

## Memory requests and partitions

The current XMU policy reports `DefMemPerCPU=MaxMemPerCPU=4012` on `cpu`.
An 8 GB request with one CPU was rejected by Slurm with `requested 8.0GB
memory/node exceeds the allowed 4.0GB for partition 'cpu'`; the same 8 GB
request is accepted with two CPUs, and 16 GB with four CPUs. The effective
job ceiling is therefore approximately `4012 MB * allocated CPUs`, subject to
the account/QOS and node limits. This is not evidence that a CPU node has only
4 GB of physical RAM. Query `scontrol show partition`, `sinfo`, and the
account/QOS association when the cluster policy changes.

Large workloads such as E7/E8 deformation, unitarity, massif, or profiling
must explicitly select `fat` and request the measured memory, for example:

```bash
sbatch --partition=fat --mem=32G --time=06:00:00 hpc/probe_diff.sbatch
```

An explicit `--mem` supplied to `sbatch` overrides the script default; it does
not change the partition's accounting or QOS limits. In particular, do not
interpret `--mem=4G` as the physical size of a CPU node or as a limit shared by
all users on that node. It is the allocation for this job; cgroup enforcement
and the node's actual `RealMemory` must be checked separately. The checked-in
jobs intentionally request conservative totals (usually 4G with 2-4 CPUs),
so increasing `--cpus-per-task` does not automatically increase `--mem`.

The corpus driver adds another, independent guard: each Rust/C++ child gets a
`RLIMIT_AS` virtual-address-space cap (`MEM_CAP_GB`, default 3). This prevents
one runaway script from killing the batch job while leaving headroom for the
Python driver and runtime mappings; it is not a measurement of RSS. Keep it
below the Slurm allocation for CPU jobs; raise it explicitly only together
with a fat-partition request, for example
`--export=ALL,MEM_CAP_GB=24` with `--mem=32G`.

Run `hpc/memory_snapshot.sbatch` to capture the live controller, node,
allocation, and cgroup values after a cluster-policy change. The checked-in
snapshot uses one CPU and 1G, so it is safe to submit as a diagnostic job.

Never put tokens, credentials, or large generated outputs in Git.

Typical workflow:

```bash
atlas_commit="$(git rev-parse HEAD)"
atlas_dirty=false
if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
  atlas_dirty=true
fi
rsync -az --exclude=.git --exclude=target --exclude=results ./ \
  majj@10.26.14.64:/public/home/majj/atlas-rust/
ssh majj@10.26.14.64 \
  'cd atlas-rust && export PATH=$HOME/.cargo/bin:$PATH && cargo build --workspace --release --locked'
```

The differential jobs now verify the submit checkout rather than trusting a
declared commit string. After syncing a committed tree, make the remote
checkout point at the same commit (and verify it is clean) before submission;
an rsync that leaves a stale `.git/HEAD` is intentionally rejected:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && git checkout --detach $atlas_commit && git rev-parse HEAD && git status --porcelain --untracked-files=all"
```

Then submit a versioned job script:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && ATLAS_COMMIT=$atlas_commit ATLAS_DIRTY_TREE=$atlas_dirty sbatch hpc/differential.sbatch"
```

For the structural Rust layer, use the smaller preflight job first:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && ATLAS_COMMIT=$atlas_commit ATLAS_DIRTY_TREE=$atlas_dirty sbatch hpc/real_group_preflight.sbatch"
```

For the typed scalar operator stage, first capture the upstream oracle only.
The job writes raw output for each scalar fixture, per-stream checksums, exit
statuses, the Atlas revision, and a compact validation report. It deliberately
does not invoke Rust, so it is valid evidence for freezing the reference before
the implementation stage:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && ATLAS_COMMIT=$atlas_commit ATLAS_DIRTY_TREE=$atlas_dirty sbatch hpc/scalar_reference.sbatch"
```

The report is `results/<commit>/<job-id>/scalar_reference_report.json`; its
SHA-256 sidecar covers the manifest, while the manifest covers every captured
stdout/stderr artifact.

The scalar event expectations assert values and diagnostic text only. The
capture records the oracle exit status but does not infer an exit-code policy
from a diagnostic fixture unless that policy is explicitly added to the event
schema.

For the typed pipeline swap, compare the Rust CLI against the already frozen
Atlas event files with:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && ATLAS_COMMIT=$atlas_commit ATLAS_DIRTY_TREE=$atlas_dirty sbatch hpc/pipeline_swap_diff.sbatch"
```

The report is
`results/<commit>/<job-id>/pipeline_swap/pipeline_swap_diff_report.json`.
The constructors and linear-value fixtures run in full, including
`root_datum(LieType,mat,bool)`, `Cartan_matrix(RootDatum)`, and
`involution(KGBElt)`. The domain-equality fixture currently runs only its
RootDatum prefix: InnerClass/RealForm/KGB setup, full domain renderings, and
relation outputs are explicit pending cases until those renderers and stable
numbering are ported. The report also lists the three selected-stage upstream
overloads still outside the Rust type surface as `uncovered_overload` pending
cases: the two primitive `involution` constructors and synthetic
`real_form(InnerClass,mat,ratvec)`. This is a selected typed-pipeline scope,
not a claim that no later Atlas overloads exist. The suite remains `PARTIAL`
until these pending surfaces are ported. A mismatch in any runnable fixture
still fails the job.
The job records both the declared `ATLAS_COMMIT`/`ATLAS_DIRTY_TREE` and the
values detected from the submit checkout, and refuses a mismatch before and
immediately after creating the frozen source snapshot. A clean versioned
checkout is frozen with `git archive <detected-commit>`; a dirty or unversioned
checkout retains the live-tree snapshot but labels that state explicitly in
the report. Before it uses the dirty-tree helper, the job loads the clean-tree
helper from the detected Git object or freezes and hashes a dirty helper copy.
It also requires the Slurm spool script to match the copy in that snapshot
and, for a clean versioned checkout, the script blob in the detected commit. After
the safe numeric/token preflight and report-directory creation, an EXIT handler writes a
machine-readable FAIL fallback (and checksum) if build, harness, or snapshot
verification aborts before a normal report exists. Malformed source tokens or
an invalid Slurm job id are rejected before a safe report path can be
constructed.

To capture an upstream fixture before editing its checked-in expectation, use
the raw reference job. With no fixture argument it captures
`commands/subscription_context.atlas`:

```bash
ssh majj@10.26.14.64 \
  "cd atlas-rust && ATLAS_COMMIT=$atlas_commit ATLAS_DIRTY_TREE=$atlas_dirty sbatch hpc/reference_capture.sbatch"
```

Pass one or more paths below `tests/fixtures/` after the job script to capture
other fixtures. The job pins and verifies the Atlas Git revision, records the
binary checksum against the frozen pipeline-oracle binary by default, and
stores raw stdout/stderr plus a JSON manifest under
`results/<commit>/<job-id>/reference_capture/`. It neither consumes nor
modifies event expectations, and its `PASS` status means only that the oracle
capture itself is valid; it is not a Rust compatibility claim. Set
`EXPECTED_ATLAS_BINARY_SHA256` explicitly only when intentionally using a
separately audited rebuild of the pinned reference revision.

Fixture arguments must be repository-relative paths including the
`tests/fixtures/` prefix and `.atlas` extension, for example:

```bash
sbatch hpc/reference_capture.sbatch \
  tests/fixtures/domain/print_block_words.atlas \
  tests/fixtures/domain/print_block_words_rejected.atlas
```

The manifest also compares declared and detected submit-repository commit and
dirty state; either mismatch makes the capture FAIL. The batch job freezes a
clean versioned checkout from `git archive <detected-commit>` and labels a
dirty or unversioned live-tree snapshot explicitly; it binds the source-state
helper before use, rechecks state after freezing, and requires the Slurm spool
script to match both the frozen copy and, for a clean versioned checkout, the
committed script blob. Raw files retain their fixture-relative directory paths
to avoid artifact-name collisions. The capture rechecks the upstream revision, dirty
state, executable checksum, and `atlas-scripts` tree hash after the final
fixture; a runtime replacement during capture therefore cannot produce a PASS
report.

Heavy differential jobs must use `sbatch`; do not run them on the login node.
Job scripts must fail on a mismatch and write a machine-readable report under
`results/<commit>/<job-id>/`. Pull only summaries and checksums back:

```bash
rsync -az majj@10.26.14.64:/public/home/majj/atlas-rust/results/ ./results/
```

SLURM opens `#SBATCH --output` before the script body runs. The checked-in jobs
therefore use root-level output filenames and exclude only the current job's
exact untracked stdout path from the submit-tree dirty check. Other Slurm logs,
untracked files, and tracked changes still make the checkout dirty. Exercise
that rule locally with `bash hpc/test_source_state.sh`.

Compute-node jobs should also set `PATH="$HOME/.cargo/bin:$PATH"` explicitly;
the login-node shell environment is not guaranteed to be inherited.

Both checked-in jobs follow the `rustcox` cluster convention: the submit
directory is made explicit, the Rustup toolchain is set explicitly, and every
run writes a JSON report plus a SHA-256 sidecar under
`results/<commit>/<job-id>/`. `differential.sbatch` is still a lexer-stage
preflight, not differential evidence. `real_group_preflight.sbatch` runs the
Rust structural format check, Clippy with warnings denied, and the unit suite,
also without an Atlas-compatibility claim. A real-group differential job
requires an exposed Atlas constructor fixture, a reference event adapter, and a
Rust domain event adapter; until then
`tests/reference/domain/real_group.meta.json` records only an HPC preflight,
not domain compatibility.

`real_group_preflight.sbatch` freezes the complete submit directory before
Cargo runs, excluding only `.git`, build targets, prior `results`, and the
Slurm stdout file. It hashes that frozen tree, executes from it with targets
and reports outside it, then rejects the job if the hash changes. Each domain
fixture entry in its report is explicitly marked as a declared future
differential fixture rather than an executed one.

This rule was verified by HPC job `3462432`: reporting a hash from the mutable
submit directory in an exit trap could otherwise describe different inputs than
the Cargo commands consumed. Always derive `ATLAS_DIRTY_TREE` from
`git status --porcelain --untracked-files=all`, because an untracked source
module is part of a synchronized submit tree too.

## Verified repair notes

### Owned-involution builder compilation

- Root cause: `CartanFiber::build_owned` took ownership of its involution, but
  a read-only numerator helper was called without borrowing it.
- Diagnostic: frozen HPC job `3463647` reached Clippy compilation and reported
  the exact `expected &LatticeInvolution, found LatticeInvolution` error.
- Prevention: after changing a builder from borrowed to owned input, audit
  every pre-move helper call and rerun the package preflight; job `3463683`
  verified the repair with format, Clippy, and 77 tests.

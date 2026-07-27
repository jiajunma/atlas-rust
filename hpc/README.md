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

Heavy differential jobs must use `sbatch`; do not run them on the login node.
Job scripts must fail on a mismatch and write a machine-readable report under
`results/<commit>/<job-id>/`. Pull only summaries and checksums back:

```bash
rsync -az majj@10.26.14.64:/public/home/majj/atlas-rust/results/ ./results/
```

SLURM opens `#SBATCH --output` before the script body runs, so the output path
must not require a directory that the script creates itself. Create the report
directory inside the job and use a root-level output filename, or pre-create
the directory before submission.

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

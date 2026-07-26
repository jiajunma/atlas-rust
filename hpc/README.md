# HPC-only verification

Local execution is intentionally prohibited. Use the XMU login node for
checkout, source synchronization, and dependency/build preparation only. Run
Rust tests, the reference Atlas executable, CWEB expansion, differential
comparisons, and benchmarks through SLURM on a compute node.

## Repository location and toolchain

The shared project directory is `/public/home/majj/atlas-rust` on XMU. The
login node has internet access for initial dependency acquisition; compute
nodes do not. Build binaries and cache dependencies on the login node before
submitting jobs. Every job records the commit, dirty-tree state, Rust
toolchain, reference Atlas revision, CWEB version, SLURM job/node, fixture
manifest, exit status, and report checksums.

Never put tokens, credentials, or large generated outputs in Git.

Typical workflow:

```bash
rsync -az --exclude=.git --exclude=target --exclude=results ./ \
  majj@10.26.14.64:/public/home/majj/atlas-rust/
ssh majj@10.26.14.64 \
  'cd atlas-rust && export PATH=$HOME/.cargo/bin:$PATH && cargo build --workspace --release'
```

Then submit a versioned job script:

```bash
ssh majj@10.26.14.64 'cd atlas-rust && sbatch hpc/differential.sbatch'
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

The repository does not yet claim that `differential.sbatch` exists; adding it
is the first HPC implementation task after the reference corpus is frozen.

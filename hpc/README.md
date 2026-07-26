# HPC-only verification

Local execution is intentionally prohibited. Use the XMU login node for
checkout and build preparation, then run tests and oracle comparisons through
SLURM on a compute node.

Typical workflow:

```bash
rsync -az --exclude=.git --exclude=target ./ majj@10.26.14.64:/public/home/majj/atlas-rust/
ssh majj@10.26.14.64 'cd atlas-rust && export PATH=$HOME/.cargo/bin:$PATH && cargo test --workspace'
```

Heavy differential jobs must use `sbatch`. Job scripts must record the commit,
toolchain, host, input corpus, exit status, and output artifact checksums.

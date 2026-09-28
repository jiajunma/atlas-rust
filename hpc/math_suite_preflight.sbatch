#!/usr/bin/env bash
#SBATCH --job-name=atlas-math-harness
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=1G
#SBATCH --time=00:05:00
#SBATCH --output=math-harness-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PYTHONDONTWRITEBYTECODE=1 MATH_PREFLIGHT_SPOOL="${BASH_SOURCE[0]}"
python3 - <<'PY'
import hashlib, json, os, pathlib, subprocess, sys
sys.path.insert(0, 'hpc')
from math_baseline_build import digest
root = pathlib.Path.cwd()
manifest = root / 'suite-inputs.json'
assert digest(manifest) == os.environ['SUITE_INPUTS_SHA256']
expected = json.loads(manifest.read_text())
actual = {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob('*'))
          if p.is_file() and p.relative_to(root).parts[0] in ('hpc', 'tests')}
assert actual == expected
assert digest(pathlib.Path(os.environ['MATH_PREFLIGHT_SPOOL'])) == expected['hpc/math_suite_preflight.sbatch']
out = root / 'results' / os.environ['SLURM_JOB_ID']
out.mkdir(parents=True, exist_ok=False)
with (out / 'unittest.log').open('wb') as log:
    run = subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s', 'hpc',
                          '-p', 'test_math*.py', '-v'], stdout=log, stderr=subprocess.STDOUT)
unchanged = all(digest(root / name) == sha for name, sha in expected.items())
report = {'schema': 'atlas-math-harness-preflight-v1',
          'job': os.environ['SLURM_JOB_ID'], 'inputs': expected,
          'input_manifest_sha256': digest(manifest), 'unchanged': unchanged,
          'exit_status': run.returncode, 'log_sha256': digest(out / 'unittest.log'),
          'status': 'HARNESS_PASS_NOT_MATH_VERIFIED' if run.returncode == 0 and unchanged else 'FAIL'}
(out / 'report.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
(out / 'report.sha256').write_text(digest(out / 'report.json') + '\n')
print(report['status'], out, flush=True)
sys.exit(0 if run.returncode == 0 and unchanged else 1)
PY

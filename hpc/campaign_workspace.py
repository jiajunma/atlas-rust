"""Bounded HPC campaign layout and disposable per-job build workspaces.

Durable evidence lives below one campaign root.  Expanded source trees and
Cargo targets live in a job-local temporary directory and are removed when the
driver leaves the context.  A killed job can leave only one precisely named
directory below its own result folder, never another top-level atlas-* tree.
"""
from contextlib import contextmanager
import os
from pathlib import Path
import re
import stat
import tempfile


ACTIVE_CAMPAIGN = "atlas-rust-campaign-20260930"
STAGE_RE = re.compile(r"[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?")
HPC_HOME = Path("/public/home/majj")


def campaign_stage(root, home=None):
    """Return the campaign for `<home>/<campaign>/stages/<stage>` or fail."""
    home = HPC_HOME if home is None else home
    root, home = Path(root).resolve(), Path(home).resolve()
    if (root.parent.name != "stages" or root.parent.parent.parent != home
            or root.parent.parent.name != ACTIVE_CAMPAIGN
            or not STAGE_RE.fullmatch(root.name)):
        raise ValueError("stage must be inside the one active atlas-rust campaign")
    return root.parent.parent


def campaign_storage_root(root):
    """Accept only the already-created exact active campaign root."""
    root = Path(root).resolve()
    home = HPC_HOME.resolve()
    expected = home / ACTIVE_CAMPAIGN
    if root != expected or not root.is_dir():
        raise ValueError("campaign storage requires the existing exact active campaign root")
    return root


def submission_scope(root, home=None):
    """Share one queue ledger and reject every legacy top-level stage."""
    return campaign_stage(root, home)


def _real_directory(path):
    try:
        value = Path(path).lstat()
    except FileNotFoundError as error:
        raise ValueError("required campaign directory is missing") from error
    if not stat.S_ISDIR(value.st_mode):
        raise ValueError("campaign path is not a real directory")


def create_result_folder(root, job_id):
    """Create one numeric result directory without following stage symlinks."""
    if not isinstance(job_id, str) or not job_id.isdecimal():
        raise ValueError("SLURM job id must be decimal")
    root = Path(root).resolve()
    campaign_stage(root)
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe result-directory traversal is unavailable")
    root_descriptor = results_descriptor = job_descriptor = None
    created_job = False
    try:
        root_descriptor = os.open(root, os.O_RDONLY | directory_flag | nofollow)
        try:
            os.mkdir("results", 0o700, dir_fd=root_descriptor)
            os.fsync(root_descriptor)
        except FileExistsError:
            pass
        results_descriptor = os.open(
            "results", os.O_RDONLY | directory_flag | nofollow,
            dir_fd=root_descriptor)
        try:
            os.mkdir(job_id, 0o700, dir_fd=results_descriptor)
            created_job = True
        except FileExistsError as error:
            raise ValueError("result folder already exists") from error
        job_descriptor = os.open(
            job_id, os.O_RDONLY | directory_flag | nofollow,
            dir_fd=results_descriptor)
        if not stat.S_ISDIR(os.fstat(job_descriptor).st_mode):
            raise ValueError("result folder is not a real directory")
        os.fsync(results_descriptor)
        return root / "results" / job_id
    except OSError as error:
        if created_job and results_descriptor is not None:
            try:
                os.rmdir(job_id, dir_fd=results_descriptor)
            except OSError:
                pass
        raise ValueError("result folder could not be created safely") from error
    finally:
        for descriptor in (job_descriptor, results_descriptor, root_descriptor):
            if descriptor is not None:
                os.close(descriptor)


def _result_folder(out):
    out = Path(out).absolute()
    if not out.name.isdecimal() or out.parent.name != "results":
        raise ValueError("temporary workspace requires one numeric job result folder")
    stage = out.parent.parent
    campaign_stage(stage)
    for path in (stage, out.parent, out):
        _real_directory(path)
    return out


@contextmanager
def ephemeral_job_workspace(out, label):
    """Yield an exact disposable directory for source and Cargo target trees."""
    out = _result_folder(out)
    if not STAGE_RE.fullmatch(label):
        raise ValueError("invalid workspace label")
    requested = os.environ.get("SLURM_TMPDIR")
    if requested:
        requested_path = Path(requested)
        if not requested_path.is_absolute():
            raise ValueError("invalid SLURM_TMPDIR")
        base = requested_path.resolve()
        if not base.is_dir():
            raise ValueError("invalid SLURM_TMPDIR")
        try:
            base.relative_to(HPC_HOME.resolve())
        except ValueError:
            pass
        else:
            raise ValueError("SLURM_TMPDIR must not persist below the HPC home")
    else:
        base = out
    prefix = ".atlas-" + label + "-" + out.name + "-"
    with tempfile.TemporaryDirectory(prefix=prefix, dir=base) as directory:
        path = Path(directory).resolve()
        if path.parent != base or not path.name.startswith(prefix):
            raise ValueError("temporary workspace escaped its exact base")
        yield path

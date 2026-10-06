"""Static full-stager inventory and single-active-launcher transition boundary."""
import ast
import hashlib
import json
from pathlib import Path
import unittest


HPC = Path(__file__).resolve().parent
AFTER_DRIVER = "math_ladder_boundary_after.py"
INDEX_DRIVER = "math_ladder_boundary_index.py"
WEYL_CAPTURE_DRIVER = "math_weyl_context_core_capture.py"
WEYL_AFTER_DRIVER = "math_weyl_context_core_after.py"
DRIVER = AFTER_DRIVER
HISTORICAL_DRIVER = "math_ladder_boundary_before.py"
DRIVERS = {
    HISTORICAL_DRIVER: "before_driver",
    AFTER_DRIVER: "driver",
    INDEX_DRIVER: "index_driver",
    WEYL_CAPTURE_DRIVER: "weyl_capture_driver",
    WEYL_AFTER_DRIVER: "weyl_after_driver",
}
STAGED_STAGERS = {
    "stage_weyl_parent_seal.py": "parent",
    "stage_ladder_boundary_before.py": "before",
    "stage_ladder_boundary_after.py": "after",
    "stage_ladder_boundary_index.py": "index",
    "stage_weyl_context_core_capture.py": "weyl_capture",
    "stage_weyl_context_core_after.py": "weyl_after",
}
FROZEN_STAGE_FILENAMES = {
    "stage_cartan_before.py",
    "stage_cartan_class_capture.py",
    "stage_cartan_partition_after.py",
    "stage_cartan_partition_before.py",
    "stage_cartan_permutation.py",
    "stage_completion_after.py",
    "stage_completion_before.py",
    "stage_completion_capture.py",
    "stage_complex_rank_before.py",
    "stage_complex_rank_build.py",
    "stage_cycle_candidate_replay.py",
    "stage_cycle_rank1.py",
    "stage_cycle_rank1_review.py",
    "stage_deform_cross_before.py",
    "stage_deform_cross_build.py",
    "stage_f4_class_trace.py",
    "stage_fpp_before.py",
    "stage_fpp_build.py",
    "stage_full_deform_after.py",
    "stage_full_deform_before.py",
    "stage_gl2_before.py",
    "stage_gl2_build.py",
    "stage_hodge_rank1.py",
    "stage_ladder_boundary_after.py",
    "stage_ladder_boundary_before.py",
    "stage_ladder_boundary_index.py",
    "stage_loading_cost_probe.py",
    "stage_loading_followup.py",
    "stage_loading_rank1.py",
    "stage_math_review_memory_retry.py",
    "stage_merged_diagnostics.py",
    "stage_merged_math.py",
    "stage_overload_after.py",
    "stage_overload_before.py",
    "stage_overload_command_after.py",
    "stage_overload_command_before.py",
    "stage_overload_command_capture.py",
    "stage_overload_command_profile.py",
    "stage_overload_presence_after.py",
    "stage_overload_presence_before.py",
    "stage_overload_profile.py",
    "stage_overload_view_after.py",
    "stage_overload_view_before.py",
    "stage_overload_view_profile.py",
    "stage_parampol_before.py",
    "stage_parampol_build.py",
    "stage_pos_neg_consumer.py",
    "stage_post_view_cost_probe.py",
    "stage_progressive_rank1.py",
    "stage_psp4_cayley_before.py",
    "stage_psp4_cayley_build.py",
    "stage_rank6_block_grid.py",
    "stage_rank6_blocks.py",
    "stage_rank6_forms.py",
    "stage_rank6_graphs.py",
    "stage_rank6_inventory.py",
    "stage_rank6_klv_scale.py",
    "stage_rank6_parameters.py",
    "stage_rank6_replay.py",
    "stage_rank6_shards.py",
    "stage_rank_capacity_union.py",
    "stage_real_form_diagnostics.py",
    "stage_real_form_survey.py",
    "stage_repair_union.py",
    "stage_torus_bitset_build.py",
    "stage_type_equivalence.py",
    "stage_unitarity_forms_rank1.py",
    "stage_unitarity_rank1.py",
    "stage_verified_merge.py",
    "stage_weyl_context_capture.py",
    "stage_weyl_context_core_after.py",
    "stage_weyl_context_core_capture.py",
    "stage_weyl_parent_seal.py",
}
HISTORICAL_STAGERS = FROZEN_STAGE_FILENAMES - set(STAGED_STAGERS)
RETIRED_BUNDLE_SCHEMA = "atlas-retired-stager-bundle-v1"
RETIRED_BUNDLE_ENCODING = "utf-8"
RETIRED_BUNDLE_SHA256 = (
    "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09"
)
RETIRED_BUNDLE_BYTES = 311925
MAX_RETIRED_BUNDLE_BYTES = 1024 * 1024
MAX_RETIRED_SOURCE_BYTES = 128 * 1024
CURRENT_POLICY = "weyl-core-after-active"
POLICIES = {
    "parent-only": {
        "parent": "active", "before": "pending", "after": "pending",
        "index": "pending", "before_driver": "pending", "driver": "pending",
        "index_driver": "pending", "weyl_capture": "pending",
        "weyl_capture_driver": "pending", "weyl_after": "pending",
        "weyl_after_driver": "pending",
    },
    "after-active": {
        "parent": "retired", "before": "pending", "after": "active",
        "index": "pending", "before_driver": "pending", "driver": "active",
        "index_driver": "pending", "weyl_capture": "pending",
        "weyl_capture_driver": "pending", "weyl_after": "pending",
        "weyl_after_driver": "pending",
    },
    "index-active": {
        "parent": "retired", "before": "pending", "after": "pending",
        "index": "active", "before_driver": "pending", "driver": "pending",
        "index_driver": "active", "weyl_capture": "pending",
        "weyl_capture_driver": "pending", "weyl_after": "pending",
        "weyl_after_driver": "pending",
    },
    "weyl-core-before-active": {
        "parent": "retired", "before": "pending", "after": "pending",
        "index": "pending", "before_driver": "pending", "driver": "pending",
        "index_driver": "pending", "weyl_capture": "active",
        "weyl_capture_driver": "active", "weyl_after": "pending",
        "weyl_after_driver": "pending",
    },
    "weyl-core-after-active": {
        "parent": "retired", "before": "pending", "after": "pending",
        "index": "pending", "before_driver": "pending", "driver": "pending",
        "index_driver": "pending", "weyl_capture": "pending",
        "weyl_capture_driver": "pending", "weyl_after": "active",
        "weyl_after_driver": "active",
    },
}

WEYL_EXECUTION_SOURCE_SHA256 = {
    "COMMAND_CONTRACTS": (
        "aa2b69a39208fe301537306f919b42a2fcac43acc68a70c05ce148e53159ffd3"
    ),
    "_group_alive": (
        "08ab1771a1d8c08bf3ad9439b3872c217f0e558e34752d949a8fa1847feb7ca2"
    ),
    "_wait_group_exit": (
        "bbac63911232f4aaa4890dbd3dfef5aca798c18f6c4ccc74fe2b8fb47af83b4c"
    ),
    "_terminate_group": (
        "6a9816603f386f33d02f06d95bfddc560bf4929df134ff62dbecebfcdc70e52d"
    ),
    "_execute_timed": (
        "436988ccd2cbcdeb817e2c4d21bf44a2330a8c58ca9d02a40e958188b69df979"
    ),
    "_command_working_directory": (
        "9f853b4379ef0784f99e9338dd0a3f78bf74e87ffce132d670ba1ac97a2eb59a"
    ),
    "_command_failed_checks": (
        "49a0c77050fede58a6e2e001c923eae93518df4a96cc075bce2f635394efbb3f"
    ),
    "CommandRecordFailure": (
        "bdd6c98812561d0809de07d6a9c0116a18ea46ef51824f6b87d675ecc483bc18"
    ),
    "_command_record": (
        "859975e9f813995bf9a2673f471e57b563b6e7f97f5070aeb3f104fb9a883458"
    ),
    "_stat_identity": (
        "c1d7e5965cb1c95d083e32275c0b96dce6eadc833c78bdc6a37338cd3e1b0c10"
    ),
    "_descriptor_snapshot": (
        "da091f9cdd3d7c82650e6689e542cf4d422a95c88fd1745fbd69d69d52d34dc0"
    ),
    "_open_executable": (
        "2ced06681ac9a231302576fb82b5ad130233100be4f7c3e6f3b3d5edebf6e4ea"
    ),
    "_executable_snapshot": (
        "a819972fc7cf5d75949cb3198f32f5db513d18e5450eb5c85bc7c1211dfcd6c1"
    ),
    "_materialize_built_executable": (
        "f84ec1d3e994425cd57d78847bf1067e6951f99efd7a853dc3e34ceb74292675"
    ),
    "_capture_record": (
        "00dc2cf38e4f79b6887a849a715913653c1036a483e2bfe1c329f1572f741120"
    ),
}
WEYL_CAPTURE_DRIVER_SHA256 = (
    "31f5aef26e14ca669ee9306efc70cc2c4f8f22fd19af11dab5e979a0296966d7"
)
WEYL_CAPTURE_STAGER_SHA256 = (
    "639b5c20d1a1efc68eebb3a240d9d2aa717c1c90757321c6b3445652fad649a6"
)
WEYL_AFTER_STAGER_SHA256 = (
    "7cd7bbed9e3c09d414e5a17246f27277311d137ec8b67ee5fe5b2cfcf16af58e"
)
WEYL_AFTER_DRIVER_SHA256 = (
    "ab99c7a62b8c24dcba74cca2531f00f568cee17cb04702f602d59dbbabec0c3a"
)
WEYL_AFTER_EXECUTION_SOURCE_SHA256 = {
    **WEYL_EXECUTION_SOURCE_SHA256,
    "COMMAND_CONTRACTS": (
        "f4dc66d88ed12c55118effc388a512e3b7a0d17415afc59d620fd374bbd43e1e"
    ),
}

PARENT_IMPORTS = {
    "from contextlib import contextmanager",
    "import fcntl", "import json", "import os", "from pathlib import Path",
    "import re", "import shutil", "import stat", "import sys", "import tempfile",
    "from campaign_blob import store_blob,verify_blob",
    "from campaign_source import digest as safe_digest,file_manifest,store_source_archive,verify_source_archive",
    "from campaign_workspace import _real_directory,campaign_stage,campaign_storage_root,submission_scope",
    "from progressive_submit import exclusive_lock,queue_ids,read_json_file,save,submit_one,validate_confirmed_history",
    "from weyl_parent_seal import MIGRATION_OVERRIDE_NAMES,PIN_NAME,SBATCH,STAGE_LOCK,STAGE_NAME,submission_receipt",
}
STAGER_IMPORTS = {
    "from contextlib import contextmanager",
    "import fcntl", "import hashlib", "import json", "import os",
    "from pathlib import Path", "import re", "import secrets", "import stat", "import sys",
    "from campaign_blob import SCHEMA as BLOB_SCHEMA",
    "from campaign_source import file_manifest",
    "from campaign_workspace import campaign_stage,submission_scope",
    "from progressive_submit import exclusive_lock,queue_ids,read_json_file,save,submit_one,validate_confirmed_history",
    "from weyl_parent_seal import load_parent_seal",
}
DRIVER_IMPORTS = {
    "from contextlib import ExitStack,contextmanager",
    "import fcntl", "import hashlib", "import json", "import os", "from pathlib import Path",
    "import platform", "import re", "import signal", "import stat", "import subprocess",
    "import sys", "import time", "import traceback",
    "from campaign_blob import SCHEMA as BLOB_SCHEMA",
    "from campaign_source import digest,file_manifest,materialize_source_archive",
    "from campaign_workspace import ACTIVE_CAMPAIGN,campaign_stage,create_result_folder,ephemeral_job_workspace",
    "from progressive_submit import queue_ids,read_json_file,save,validate_confirmed_history",
    "from weyl_parent_seal import CAPTURE_CASES,boundary_bytes,load_parent_seal",
}
AFTER_STAGER_IMPORTS = (STAGER_IMPORTS - {
    "from campaign_blob import SCHEMA as BLOB_SCHEMA",
}) | {
    "import copy",
    "from campaign_blob import SCHEMA as BLOB_SCHEMA,verify_blob",
}
AFTER_DRIVER_IMPORTS = (DRIVER_IMPORTS - {
    "from campaign_blob import SCHEMA as BLOB_SCHEMA",
}) | {
    "from campaign_blob import SCHEMA as BLOB_SCHEMA,read_blob",
}
INDEX_STAGER_IMPORTS = {
    "from contextlib import contextmanager",
    "import copy", "import fcntl", "import hashlib", "import json", "import os",
    "from pathlib import Path,PurePosixPath",
    "import re", "import secrets", "import stat", "import sys",
    "from campaign_blob import verify_blob",
    "from campaign_source import file_manifest",
    "from campaign_workspace import campaign_stage,submission_scope",
    "from progressive_submit import exclusive_lock,read_json_file,save,submit_one,validate_confirmed_history",
}
INDEX_DRIVER_IMPORTS = {
    "import hashlib", "import json", "import os", "from pathlib import Path",
    "import re", "import signal", "import subprocess", "import sys", "import time",
    "import traceback",
    "from campaign_blob import verify_blob",
    "from campaign_workspace import ACTIVE_CAMPAIGN,campaign_stage,create_result_folder,ephemeral_job_workspace",
    "from progressive_submit import read_json_file,save",
    "from stage_ladder_boundary_index import EXPECTED_TEST_COUNTS,PIN_NAME,RETIRED_STAGER_BUNDLE_REFERENCE,STAGE_LOCK,STAGE_NAME,confirmed_record,existing_lock,frozen_stage_inputs,submission_receipt,validate_pin,validate_test_counts",
}
WEYL_CAPTURE_STAGER_IMPORTS = {
    "from contextlib import contextmanager",
    "import ast", "import copy", "import fcntl", "import hashlib",
    "import json", "import os", "from pathlib import Path,PurePosixPath",
    "import re", "import secrets", "import stat", "import sys",
    "from campaign_blob import read_blob,verify_blob",
    "from campaign_source import file_manifest,verify_source_archive",
    "from campaign_workspace import submission_scope",
    "import weyl_context_core_regression as regression_contract",
    "from progressive_submit import STAGE_CREATION_CONTRACT_SCHEMA,STAGE_CREATION_MARKER,STAGE_CREATION_RECEIPT,create_fixed_stage,exclusive_lock,read_json_file,save,submit_one,validate_confirmed_history,validate_stage_creation",
}
WEYL_CAPTURE_DRIVER_IMPORTS = {
    "import copy", "import hashlib", "import json", "import math", "import os",
    "from pathlib import Path,PurePosixPath", "import platform", "import re",
    "import signal", "import stat", "import subprocess", "import sys",
    "import time", "import traceback",
    "from campaign_blob import read_blob",
    "from campaign_source import digest,file_manifest,materialize_source_archive",
    "from campaign_workspace import ACTIVE_CAMPAIGN,campaign_stage,create_result_folder,ephemeral_job_workspace",
    "from progressive_submit import read_json_file,validate_stage_creation",
    "from stage_weyl_context_core_capture import ACCEPTED_SOURCE,CATALOG_CASES,CATALOG_PATH,CATALOG_SHA256,EXPECTED_TEST_COUNTS,FINAL_HASHES,PATCH_HASHES,PIN_NAME,PIN_SCHEMA,PARENT_SEAL_REFERENCE,PARENT_SOURCE_OBJECT,RETIRED_STAGER_BUNDLE_REFERENCE,SBATCH,SEALED_FIXTURE_HASHES,STAGE_LOCK,STAGE_NAME,SUBMISSION_ENABLED as STAGER_SUBMISSION_ENABLED,TESTS_ONLY_HASHES,REGRESSION_CATALOG_BYTES,REGRESSION_CATALOG_PATH,REGRESSION_CATALOG_SHA256,REGRESSION_FIXTURE_HASHES,REGRESSION_INSPECTION_BYTES,REGRESSION_INSPECTION_PATH,REGRESSION_INSPECTION_SHA256,REGRESSION_PATCH_HASHES,REGRESSION_PATCH_PATH,REGRESSION_SOURCE,REGRESSION_SOURCE_HASHES,confirmed_record,existing_lock,frozen_stage_inputs,saved_json_sha,submission_receipt,validate_catalog as validate_staged_catalog,validate_capture_v1_failure,validate_capture_v2_failure,validate_capture_v3_failure,validate_capture_v4_failure,validate_capture_v5_failure,validate_capture_v6_failure,validate_capture_v7_failure,validate_capture_v8,validate_before_v1_failure,validate_before_v2_failure,validate_before_v3_failure,validate_parent_objects,validate_pin as validate_stage_pin,validate_predecessor,validate_prior_creation_failure,validate_regression_inputs,validate_stage_topology",
    "from weyl_context_core_contract import CAPTURE_MATURITY,NONACCEPTING_STATUSES,classify_capture,decode_catalog as decode_discovery_catalog,validate_catalog as validate_contract_catalog",
    "from weyl_context_core_regression import BEFORE_REPRODUCED,HARNESS_FAILURE as BEFORE_HARNESS_FAILURE,OBSERVATION_SCHEMA as BEFORE_OBSERVATION_SCHEMA,PROVENANCE_FAILURE as BEFORE_PROVENANCE_FAILURE,UNEXPECTED_PASS,classify_before,decode_catalog as decode_regression_catalog,decode_inspection as decode_regression_inspection,expected_provenance as expected_regression_provenance,validate_artifacts as validate_regression_artifacts",
    "from weyl_parent_seal import boundary_bytes,load_parent_seal",
}
WEYL_AFTER_STAGER_IMPORTS = {
    "from contextlib import contextmanager",
    "import ast", "import copy", "import fcntl", "import hashlib",
    "import json", "import os", "from pathlib import Path,PurePosixPath",
    "import re", "import secrets", "import stat", "import sys",
    "from campaign_blob import read_blob,verify_blob",
    "from campaign_source import file_manifest,verify_source_archive",
    "from campaign_workspace import submission_scope",
    "import weyl_context_core_regression as regression_contract",
    "from progressive_submit import STAGE_CREATION_CONTRACT_SCHEMA,STAGE_CREATION_MARKER,STAGE_CREATION_RECEIPT,create_fixed_stage,exclusive_lock,read_json_file,save,submit_one,validate_confirmed_history,validate_stage_creation",
}
WEYL_AFTER_DRIVER_IMPORTS = {
    "import copy", "import hashlib", "import json", "import math", "import os",
    "from pathlib import Path,PurePosixPath", "import platform", "import re",
    "import signal", "import stat", "import subprocess", "import sys",
    "import time", "import traceback",
    "from campaign_blob import read_blob",
    "from campaign_source import digest,file_manifest,materialize_source_archive",
    "from campaign_workspace import ACTIVE_CAMPAIGN,campaign_stage,create_result_folder,ephemeral_job_workspace",
    "from progressive_submit import read_json_file,validate_stage_creation",
    "from stage_weyl_context_core_after import ACCEPTED_SOURCE,CATALOG_CASES,CATALOG_PATH,CATALOG_SHA256,EXPECTED_TEST_COUNTS,FINAL_HASHES,PATCH_HASHES,PIN_NAME,PIN_SCHEMA,PARENT_SEAL_REFERENCE,PARENT_SOURCE_OBJECT,RETIRED_STAGER_BUNDLE_REFERENCE,SBATCH,SEALED_FIXTURE_HASHES,STAGE_LOCK,STAGE_NAME,SUBMISSION_ENABLED as STAGER_SUBMISSION_ENABLED,TESTS_ONLY_HASHES,REPAIR_PATCH_HASHES,REPAIR_PATCH_PATH,REPAIRED_SOURCE_HASHES,AFTER_SOURCE_FILES,AFTER_SOURCE_MANIFEST_SHA256,REGRESSION_CATALOG_BYTES,REGRESSION_CATALOG_PATH,REGRESSION_CATALOG_SHA256,REGRESSION_FIXTURE_HASHES,REGRESSION_INSPECTION_BYTES,REGRESSION_INSPECTION_PATH,REGRESSION_INSPECTION_SHA256,REGRESSION_PATCH_HASHES,REGRESSION_PATCH_PATH,REGRESSION_SOURCE,REGRESSION_SOURCE_HASHES,confirmed_record,existing_lock,frozen_stage_inputs,saved_json_sha,submission_receipt,validate_catalog as validate_staged_catalog,validate_capture_v1_failure,validate_capture_v2_failure,validate_capture_v3_failure,validate_capture_v4_failure,validate_capture_v5_failure,validate_capture_v6_failure,validate_capture_v7_failure,validate_capture_v8,validate_before_v1_failure,validate_before_v2_failure,validate_before_v3_failure,validate_after_v1_failure,validate_after_v2_failure,validate_after_v3_failure,validate_after_v4_failure,validate_before_v4_result,validate_parent_objects,validate_pin as validate_stage_pin,validate_predecessor,validate_prior_creation_failure,validate_regression_inputs,validate_stage_topology",
    "from weyl_context_core_contract import CAPTURE_MATURITY,NONACCEPTING_STATUSES,classify_capture,decode_catalog as decode_discovery_catalog,validate_catalog as validate_contract_catalog",
    "from weyl_context_core_regression import AFTER_HARNESS_FAILURE,AFTER_PROVENANCE_FAILURE,AFTER_REPRODUCED,AFTER_STILL_FAILING,AFTER_OBSERVATION_SCHEMA,classify_after,decode_catalog as decode_regression_catalog,decode_inspection as decode_regression_inspection,expected_provenance as expected_regression_provenance,validate_artifacts as validate_regression_artifacts",
    "from weyl_parent_seal import boundary_bytes,load_parent_seal",
}
EXPECTED_IMPORTS = {
    "stage_weyl_parent_seal.py": PARENT_IMPORTS,
    "stage_ladder_boundary_before.py": STAGER_IMPORTS,
    "stage_ladder_boundary_after.py": AFTER_STAGER_IMPORTS,
    "stage_ladder_boundary_index.py": INDEX_STAGER_IMPORTS,
    HISTORICAL_DRIVER: (DRIVER_IMPORTS - {"import json"}) | {
        "from stage_ladder_boundary_before import frozen_stage_inputs,submission_receipt",
    },
    AFTER_DRIVER: AFTER_DRIVER_IMPORTS | {
        "from stage_ladder_boundary_after import frozen_stage_inputs,submission_receipt",
    },
    INDEX_DRIVER: INDEX_DRIVER_IMPORTS,
    "stage_weyl_context_core_capture.py": WEYL_CAPTURE_STAGER_IMPORTS,
    WEYL_CAPTURE_DRIVER: WEYL_CAPTURE_DRIVER_IMPORTS,
    "stage_weyl_context_core_after.py": WEYL_AFTER_STAGER_IMPORTS,
    WEYL_AFTER_DRIVER: WEYL_AFTER_DRIVER_IMPORTS,
}

COMMON_STAGER_ASSIGNMENTS = {
    "STAGE_NAME", "PIN_NAME", "PIN_SCHEMA", "STAGE_LOCK",
    "SUBMISSION_ENABLED", "CHECKER_TESTS", "SBATCH", "TEST_HASHES",
    "BASE_STAGE_INPUT_NAMES", "CHILD_ONLY_INPUTS", "STAGE_INPUT_NAMES",
    "SEALED_SHARED_INPUTS", "SUBMISSION_RECORD_KEYS",
}
COMMON_DRIVER_ASSIGNMENTS = {
    "_ACTIVE_CAMPAIGN", "LEGACY_PATH_OPEN_ATTEMPTS", "STAGE_NAME", "PIN_NAME",
    "PIN_SCHEMA", "STAGE_LOCK", "SUBMISSION_ENABLED", "COMMAND_TIMEOUT_SECONDS",
    "COMMAND_KILL_AFTER_SECONDS", "ROOT", "SESSION", "SBATCH",
    "DOMAIN_NAMES", "CORE_NAMES", "TEST_HASHES", "FIXTURE_PATH",
    "ORACLE_STDOUT_PATH", "ORACLE_STDERR_PATH", "FIXTURE_HASHES",
    "BASE_STAGE_INPUT_NAMES", "CHILD_ONLY_INPUTS", "STAGE_INPUT_NAMES",
    "SEALED_SHARED_INPUTS", "CHECKERS", "SUBMISSION_RECORD_KEYS",
}
INDEX_STAGER_ASSIGNMENTS = {
    "STAGE_NAME", "PIN_NAME", "PIN_SCHEMA", "STAGE_LOCK", "SBATCH",
    "SUBMISSION_ENABLED", "EXPECTED_TEST_COUNTS", "INDEX_PATH", "REPORT_PATH",
    "INSPECTION_PATH", "REVIEW_PATH", "BEFORE_EVIDENCE_PATH",
    "ORIGINAL_CAPTURE_PATH", "STAGE_INPUT_NAMES", "PREDECESSOR", "LIFECYCLE",
    "RETIRED_STAGER_BUNDLE_REFERENCE", "SUBMISSION_RECORD_KEYS", "PIN_KEYS",
    "SHA256_PATTERN",
}
INDEX_DRIVER_ASSIGNMENTS = {
    "_ACTIVE_CAMPAIGN", "LEGACY_PATH_OPEN_ATTEMPTS", "SUBMISSION_ENABLED",
    "COMMAND_TIMEOUT_SECONDS", "COMMAND_KILL_AFTER_SECONDS", "SBATCH",
    "CHECK_COMMANDS", "SHA256_PATTERN", "UNITTEST_SUMMARY_PATTERN",
}
WEYL_CAPTURE_STAGER_ASSIGNMENTS = {
    "STAGE_NAME", "PIN_NAME", "PIN_SCHEMA", "STAGE_LOCK", "SBATCH",
    "SUBMISSION_ENABLED", "EXPECTED_TEST_COUNTS", "CHECKER_TESTS",
    "CATALOG_PATH", "CATALOG_SHA256", "CATALOG_CASES",
    "CATALOG_BYTES", "PREDECESSOR_CAMPAIGN_FILES",
    "REGRESSION_CATALOG_BYTES", "REGRESSION_CATALOG_PATH",
    "REGRESSION_CATALOG_SHA256", "REGRESSION_EXPECTED_INVENTORY",
    "REGRESSION_FIXTURE_BYTES", "REGRESSION_FIXTURE_HASHES",
    "REGRESSION_INSPECTION_BYTES", "REGRESSION_INSPECTION_PATH",
    "REGRESSION_INSPECTION_SHA256", "REGRESSION_PATCH_BYTES",
    "REGRESSION_PATCH_HASHES", "REGRESSION_PATCH_PATH", "REGRESSION_RECORD",
    "REGRESSION_RETAINED_CONTROL", "REGRESSION_SELECTOR_TESTS",
    "REGRESSION_SOURCE", "REGRESSION_SOURCE_HASHES",
    "V7_PREDECESSOR", "V7_PREDECESSOR_RECORD", "V7_PREDECESSOR_REFERENCE",
    "V7_PREDECESSOR_STAGE", "V7_PREDECESSOR_STATE",
    "V8_PREDECESSOR", "V8_PREDECESSOR_RECORD", "V8_PREDECESSOR_REFERENCE",
    "V8_PREDECESSOR_STAGE", "V8_PREDECESSOR_STATE",
    "V8_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V1_FAILURE_EVIDENCE",
    "BEFORE_V1_PREDECESSOR_STAGE", "BEFORE_V1_PREDECESSOR_RECORD",
    "BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V1_PREDECESSOR_STATE",
    "BEFORE_V1_PREDECESSOR", "BEFORE_V1_PREDECESSOR_REFERENCE",
    "BEFORE_V2_FAILURE_EVIDENCE",
    "BEFORE_V2_PREDECESSOR_STAGE", "BEFORE_V2_PREDECESSOR_RECORD",
    "BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V2_PREDECESSOR_STATE",
    "BEFORE_V2_PREDECESSOR", "BEFORE_V2_PREDECESSOR_REFERENCE",
    "BEFORE_V3_FAILURE_EVIDENCE",
    "V8_SUBMISSION_EVIDENCE_BYTES", "V8_SUBMISSION_EVIDENCE_PATH",
    "V8_SUBMISSION_EVIDENCE_SHA256",
    "AFTER_REPORT_PATH", "AFTER_INSPECTION_PATH", "AFTER_REVIEW_PATH",
    "ACCEPTANCE_INDEX_PATH", "INDEX_REPORT_PATH", "INDEX_INSPECTION_PATH",
    "ACCEPTED_INDEX_PREDECESSOR", "PREDECESSOR", "PREDECESSOR_REFERENCE",
    "PREDECESSOR_STAGE", "PREDECESSOR_RECORD", "PREDECESSOR_STATE",
    "V1_FAILURE_EVIDENCE", "V1_PREDECESSOR", "V1_PREDECESSOR_STAGE",
    "V1_PREDECESSOR_RECORD", "V1_PREDECESSOR_STATE",
    "V2_FAILURE_EVIDENCE", "V2_PREDECESSOR", "V2_PREDECESSOR_STAGE",
    "V2_PREDECESSOR_RECORD", "V2_PREDECESSOR_STATE",
    "V3_FAILURE_EVIDENCE", "V3_PREDECESSOR", "V3_PREDECESSOR_STAGE",
    "V3_PREDECESSOR_RECORD", "V3_PREDECESSOR_STATE",
    "V3_PREDECESSOR_REFERENCE", "V4_FAILURE_EVIDENCE",
    "V4_PREDECESSOR", "V4_PREDECESSOR_STAGE", "V4_PREDECESSOR_RECORD",
    "V4_PREDECESSOR_STATE", "V4_PREDECESSOR_REFERENCE",
    "V5_FAILURE_EVIDENCE", "V5_PREDECESSOR", "V5_PREDECESSOR_STAGE",
    "V5_PREDECESSOR_RECORD", "V5_PREDECESSOR_STATE",
    "V5_PREDECESSOR_REFERENCE", "V6_FAILURE_EVIDENCE",
    "V6_PREDECESSOR", "V6_PREDECESSOR_STAGE", "V6_PREDECESSOR_RECORD",
    "V6_PREDECESSOR_STATE", "V6_PREDECESSOR_REFERENCE",
    "V7_FAILURE_EVIDENCE",
    "PRIOR_CREATION_FAILURE_EVIDENCE", "PRIOR_CREATION_FAILURE",
    "PARENT_SEAL_REFERENCE", "PARENT_SOURCE_OBJECT",
    "RETIRED_STAGER_BUNDLE_REFERENCE", "PATCH_HASHES", "TESTS_ONLY_HASHES",
    "CURRENT_STAGER_PATH", "FROZEN_LAUNCHER_HASHES",
    "FROZEN_LAUNCHER_STATES",
    "FINAL_HASHES", "SEALED_FIXTURE_HASHES", "ACCEPTED_SOURCE", "LIFECYCLE",
    "STAGE_INPUT_NAMES", "SUBMISSION_RECORD_KEYS", "PIN_KEYS",
    "SHA256_PATTERN", "FORBIDDEN_DURABLE_STAGE_NAMES",
    "ALLOWED_STAGE_DIRECTORIES", "ALLOWED_STAGE_FILES",
    "SLURM_OUTPUT_PATTERN",
}
WEYL_CAPTURE_DRIVER_ASSIGNMENTS = {
    "_ACTIVE_CAMPAIGN", "LEGACY_PATH_OPEN_ATTEMPTS", "SUBMISSION_ENABLED",
    "EXPECTED_STAGE", "REPORT_SCHEMA", "SUCCESS_STATUS", "INCOMPLETE_STATUS",
    "REPORT_SCOPE", "COMMAND_TIMEOUT_SECONDS", "COMMAND_KILL_AFTER_SECONDS",
    "COMMAND_EXIT_GRACE_SECONDS",
    "CHECKER_PYTHON", "EXPECTED_INVOCATIONS", "COMMAND_NAMES", "CHECK_COMMANDS",
    "COMMAND_CONTRACTS", "REPORT_KEYS", "INVOCATION_KEYS",
    "PROVENANCE_KEYS", "OBSERVATION_KEYS", "ARTIFACT_KEYS", "COMMAND_KEYS",
    "CATALOG_RECORD",
    "BEFORE_CLAIM_KEYS", "EXPECTED_COMMAND_EXITS", "REGRESSION_CATALOG_RECORD",
    "REGRESSION_INSPECTION_RECORD", "REGRESSION_KEYS",
    "THREAD_SETTINGS", "ENVIRONMENT_KEYS", "LIMITATIONS", "FORBIDDEN_REPORT_KEYS",
    "COMPLETE_CAPTURE_STATUSES", "ROOT_SYSTEM", "SESSION", "TEST_PATCH",
    "PRODUCTION_PATCH", "BOUNDARY_FIXTURE", "BOUNDARY_STDOUT",
    "BOUNDARY_STDERR", "SHA256_PATTERN", "INVOCATION_ID_PATTERN",
    "MANIFEST_PROGRAM",
}
WEYL_AFTER_STAGER_ASSIGNMENTS = {
    "STAGE_NAME", "PIN_NAME", "PIN_SCHEMA", "STAGE_LOCK", "SBATCH",
    "SUBMISSION_ENABLED", "EXPECTED_TEST_COUNTS", "CHECKER_TESTS",
    "BEFORE_FAILURE_PREDICTED_AFTER_COUNTS", "BEFORE_V4_CHECKER_TOTAL",
    "CATALOG_PATH", "CATALOG_SHA256", "CATALOG_CASES",
    "CATALOG_BYTES", "PREDECESSOR_CAMPAIGN_FILES",
    "REGRESSION_CATALOG_BYTES", "REGRESSION_CATALOG_PATH",
    "REGRESSION_CATALOG_SHA256", "REGRESSION_EXPECTED_INVENTORY",
    "REGRESSION_FIXTURE_BYTES", "REGRESSION_FIXTURE_HASHES",
    "REGRESSION_INSPECTION_BYTES", "REGRESSION_INSPECTION_PATH",
    "REGRESSION_INSPECTION_SHA256", "REGRESSION_PATCH_BYTES",
    "REGRESSION_PATCH_HASHES", "REGRESSION_PATCH_PATH", "REGRESSION_RECORD",
    "REGRESSION_RETAINED_CONTROL", "REGRESSION_SELECTOR_TESTS",
    "REGRESSION_SOURCE", "REGRESSION_SOURCE_HASHES",
    "REPAIR_PATCH_PATH", "REPAIR_PATCH_HASHES", "REPAIR_PATCH_BYTES",
    "REPAIRED_SOURCE_HASHES", "AFTER_SOURCE_FILES",
    "AFTER_SOURCE_MANIFEST_SHA256",
    "V7_PREDECESSOR", "V7_PREDECESSOR_RECORD", "V7_PREDECESSOR_REFERENCE",
    "V7_PREDECESSOR_STAGE", "V7_PREDECESSOR_STATE",
    "V8_PREDECESSOR", "V8_PREDECESSOR_RECORD", "V8_PREDECESSOR_REFERENCE",
    "V8_PREDECESSOR_STAGE", "V8_PREDECESSOR_STATE",
    "V8_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V1_FAILURE_EVIDENCE",
    "BEFORE_V1_PREDECESSOR_STAGE", "BEFORE_V1_PREDECESSOR_RECORD",
    "BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V1_PREDECESSOR_STATE",
    "BEFORE_V1_PREDECESSOR", "BEFORE_V1_PREDECESSOR_REFERENCE",
    "BEFORE_V2_FAILURE_EVIDENCE",
    "BEFORE_V2_PREDECESSOR_STAGE", "BEFORE_V2_PREDECESSOR_RECORD",
    "BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V2_PREDECESSOR_STATE",
    "BEFORE_V2_PREDECESSOR", "BEFORE_V2_PREDECESSOR_REFERENCE",
    "BEFORE_V3_FAILURE_EVIDENCE", "BEFORE_V4_RESULT_EVIDENCE",
    "BEFORE_V3_PREDECESSOR_STAGE", "BEFORE_V3_PREDECESSOR_RECORD",
    "BEFORE_V3_PREDECESSOR_CAMPAIGN_FILES", "BEFORE_V3_PREDECESSOR_STATE",
    "BEFORE_V3_PREDECESSOR", "BEFORE_V3_PREDECESSOR_REFERENCE",
    "BEFORE_V4_PREDECESSOR_STAGE", "BEFORE_V4_PREDECESSOR",
    "AFTER_V1_FAILURE_EVIDENCE", "AFTER_V1_FAILURE_REPORT",
    "AFTER_V1_PREDECESSOR_STAGE", "AFTER_V1_PREDECESSOR",
    "AFTER_V1_ERA_SOURCE_MANIFEST_SHA256",
    "AFTER_V2_FAILURE_EVIDENCE",
    "AFTER_V2_PREDECESSOR_STAGE", "AFTER_V2_PREDECESSOR",
    "AFTER_V2_PREDECESSOR_OUT", "AFTER_V3_PREDECESSOR_STAGE",
    "AFTER_V3_PREDECESSOR", "AFTER_V3_PREDECESSOR_OUT",
    "AFTER_V4_FAILURE_EVIDENCE",
    "AFTER_V3_FAILURE_EVIDENCE",
    "V8_SUBMISSION_EVIDENCE_BYTES", "V8_SUBMISSION_EVIDENCE_PATH",
    "V8_SUBMISSION_EVIDENCE_SHA256",
    "AFTER_REPORT_PATH", "AFTER_INSPECTION_PATH", "AFTER_REVIEW_PATH",
    "ACCEPTANCE_INDEX_PATH", "INDEX_REPORT_PATH", "INDEX_INSPECTION_PATH",
    "ACCEPTED_INDEX_PREDECESSOR", "PREDECESSOR", "PREDECESSOR_REFERENCE",
    "PREDECESSOR_STAGE", "PREDECESSOR_RECORD", "PREDECESSOR_STATE",
    "V1_FAILURE_EVIDENCE", "V1_PREDECESSOR", "V1_PREDECESSOR_STAGE",
    "V1_PREDECESSOR_RECORD", "V1_PREDECESSOR_STATE",
    "V2_FAILURE_EVIDENCE", "V2_PREDECESSOR", "V2_PREDECESSOR_STAGE",
    "V2_PREDECESSOR_RECORD", "V2_PREDECESSOR_STATE",
    "V3_FAILURE_EVIDENCE", "V3_PREDECESSOR", "V3_PREDECESSOR_STAGE",
    "V3_PREDECESSOR_RECORD", "V3_PREDECESSOR_STATE",
    "V3_PREDECESSOR_REFERENCE", "V4_FAILURE_EVIDENCE",
    "V4_PREDECESSOR", "V4_PREDECESSOR_STAGE", "V4_PREDECESSOR_RECORD",
    "V4_PREDECESSOR_STATE", "V4_PREDECESSOR_REFERENCE",
    "V5_FAILURE_EVIDENCE", "V5_PREDECESSOR", "V5_PREDECESSOR_STAGE",
    "V5_PREDECESSOR_RECORD", "V5_PREDECESSOR_STATE",
    "V5_PREDECESSOR_REFERENCE", "V6_FAILURE_EVIDENCE",
    "V6_PREDECESSOR", "V6_PREDECESSOR_STAGE", "V6_PREDECESSOR_RECORD",
    "V6_PREDECESSOR_STATE", "V6_PREDECESSOR_REFERENCE",
    "V7_FAILURE_EVIDENCE",
    "PRIOR_CREATION_FAILURE_EVIDENCE", "PRIOR_CREATION_FAILURE",
    "PARENT_SEAL_REFERENCE", "PARENT_SOURCE_OBJECT",
    "RETIRED_STAGER_BUNDLE_REFERENCE", "PATCH_HASHES", "TESTS_ONLY_HASHES",
    "CURRENT_STAGER_PATH", "FROZEN_LAUNCHER_HASHES",
    "FROZEN_LAUNCHER_STATES",
    "FINAL_HASHES", "SEALED_FIXTURE_HASHES", "ACCEPTED_SOURCE", "LIFECYCLE",
    "STAGE_INPUT_NAMES", "SUBMISSION_RECORD_KEYS", "PIN_KEYS",
    "SHA256_PATTERN", "FORBIDDEN_DURABLE_STAGE_NAMES",
    "ALLOWED_STAGE_DIRECTORIES", "ALLOWED_STAGE_FILES",
    "SLURM_OUTPUT_PATTERN",
}
WEYL_AFTER_DRIVER_ASSIGNMENTS = {
    "_ACTIVE_CAMPAIGN", "LEGACY_PATH_OPEN_ATTEMPTS", "SUBMISSION_ENABLED",
    "EXPECTED_STAGE", "REPORT_SCHEMA", "SUCCESS_STATUS", "INCOMPLETE_STATUS",
    "REPORT_SCOPE", "COMMAND_TIMEOUT_SECONDS", "COMMAND_KILL_AFTER_SECONDS",
    "COMMAND_EXIT_GRACE_SECONDS",
    "CHECKER_PYTHON", "EXPECTED_INVOCATIONS", "COMMAND_NAMES", "CHECK_COMMANDS",
    "COMMAND_CONTRACTS", "REPORT_KEYS", "INVOCATION_KEYS",
    "PROVENANCE_KEYS", "OBSERVATION_KEYS", "ARTIFACT_KEYS", "COMMAND_KEYS",
    "CATALOG_RECORD",
    "CLAIM_KEYS", "EXPECTED_COMMAND_EXITS", "REGRESSION_CATALOG_RECORD",
    "REGRESSION_INSPECTION_RECORD", "REGRESSION_KEYS",
    "THREAD_SETTINGS", "ENVIRONMENT_KEYS", "LIMITATIONS", "FORBIDDEN_REPORT_KEYS",
    "COMPLETE_CAPTURE_STATUSES", "ROOT_SYSTEM", "SESSION", "TEST_PATCH",
    "PRODUCTION_PATCH", "BOUNDARY_FIXTURE", "BOUNDARY_STDOUT",
    "BOUNDARY_STDERR", "SHA256_PATTERN", "INVOCATION_ID_PATTERN",
    "MANIFEST_PROGRAM", "DOMAIN_BUILTINS", "TYPED", "WEYL_SUBGROUP",
}
EXPECTED_ASSIGNMENTS = {
    "stage_weyl_parent_seal.py": set(),
    "stage_ladder_boundary_before.py": COMMON_STAGER_ASSIGNMENTS,
    "stage_ladder_boundary_after.py": COMMON_STAGER_ASSIGNMENTS | {
        "FINAL_HASHES", "PATCH_HASHES", "BEFORE_EVIDENCE_PATH",
        "BEFORE_EVIDENCE_HASH", "BEFORE_REFERENCE", "PARENT_SEAL_REFERENCE",
        "V1_FAILURE_EVIDENCE_PATH", "V1_FAILURE_EVIDENCE_HASH",
        "V1_FAILURE_REFERENCE", "RETIRED_STAGER_BUNDLE_REFERENCE",
        "V2_FAILURE_EVIDENCE_PATH", "V2_FAILURE_EVIDENCE_HASH",
        "V2_FAILURE_REFERENCE", "LIFECYCLE", "PIN_KEYS",
    },
    "stage_ladder_boundary_index.py": INDEX_STAGER_ASSIGNMENTS,
    HISTORICAL_DRIVER: COMMON_DRIVER_ASSIGNMENTS | {"PATCH"},
    AFTER_DRIVER: COMMON_DRIVER_ASSIGNMENTS | {
        "TEST_PATCH", "PRODUCTION_PATCH", "FINAL_HASHES", "PATCH_HASHES",
        "BEFORE_EVIDENCE_PATH", "BEFORE_EVIDENCE_HASH", "BEFORE_REFERENCE",
        "V1_FAILURE_EVIDENCE_PATH", "V1_FAILURE_EVIDENCE_HASH",
        "V1_FAILURE_REFERENCE", "PARENT_SEAL_REFERENCE",
        "RETIRED_STAGER_BUNDLE_REFERENCE",
        "V2_FAILURE_EVIDENCE_PATH", "V2_FAILURE_EVIDENCE_HASH",
        "V2_FAILURE_REFERENCE", "LIFECYCLE", "FULL_STAGER_INVENTORY_PROGRAM",
    },
    INDEX_DRIVER: INDEX_DRIVER_ASSIGNMENTS,
    "stage_weyl_context_core_capture.py": WEYL_CAPTURE_STAGER_ASSIGNMENTS,
    WEYL_CAPTURE_DRIVER: WEYL_CAPTURE_DRIVER_ASSIGNMENTS,
    "stage_weyl_context_core_after.py": WEYL_AFTER_STAGER_ASSIGNMENTS,
    WEYL_AFTER_DRIVER: WEYL_AFTER_DRIVER_ASSIGNMENTS,
}

PROJECT_IMPORTS = {
    "campaign_blob", "campaign_source", "campaign_workspace", "progressive_submit",
    "stage_ladder_boundary_before", "stage_ladder_boundary_after",
    "stage_ladder_boundary_index", "weyl_parent_seal",
    "stage_weyl_context_core_capture", "stage_weyl_context_core_after",
    "weyl_context_core_contract",
    "weyl_context_core_regression",
}


def parsed(source):
    tree = ast.parse(source)
    tree._source_text = source
    return tree


def source_node_sha256(tree, node):
    source = getattr(tree, "_source_text", None)
    if (not isinstance(source, str)
            or not isinstance(getattr(node, "lineno", None), int)
            or not isinstance(getattr(node, "end_lineno", None), int)):
        raise ValueError("source-bound AST node is unavailable")
    lines = source.splitlines(keepends=True)
    if node.lineno < 1 or node.end_lineno < node.lineno \
            or node.end_lineno > len(lines):
        raise ValueError("source-bound AST node range is invalid")
    raw = "".join(lines[node.lineno - 1:node.end_lineno]).encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def function(tree, name):
    matches = [node for node in tree.body
               if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
               and node.name == name]
    if len(matches) != 1:
        raise ValueError("launcher must define exactly one " + name)
    return matches[0]


def definition(tree, name):
    matches = [node for node in tree.body
               if isinstance(
                   node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))
               and node.name == name]
    if len(matches) != 1:
        raise ValueError("launcher must define exactly one " + name)
    return matches[0]


def bool_assignment(tree, name):
    values = []
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
            if isinstance(target, ast.Name) and target.id == name:
                values.append(node.value.value if isinstance(node.value, ast.Constant) else None)
    if len(values) != 1 or type(values[0]) is not bool:
        raise ValueError(name + " must have one literal boolean assignment")
    return values[0]


def literal_assignment(tree, name):
    values = [
        node.value for node in tree.body
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == name
    ]
    if len(values) != 1:
        raise ValueError(name + " must have one literal assignment")
    try:
        return ast.literal_eval(values[0])
    except (ValueError, TypeError) as error:
        raise ValueError(name + " must be a literal assignment") from error


def raises_system_exit(node):
    return (isinstance(node, ast.Raise)
            and node.cause is None
            and isinstance(node.exc, ast.Call)
            and isinstance(node.exc.func, ast.Name)
            and node.exc.func.id == "SystemExit"
            and len(node.exc.args) == 1
            and not node.exc.keywords
            and isinstance(node.exc.args[0], ast.Constant)
            and isinstance(node.exc.args[0].value, str))


def disabled_guard(node):
    return (isinstance(node, ast.If)
            and isinstance(node.test, ast.UnaryOp)
            and isinstance(node.test.op, ast.Not)
            and isinstance(node.test.operand, ast.Name)
            and node.test.operand.id == "SUBMISSION_ENABLED"
            and len(node.body) == 1
            and not node.orelse
            and raises_system_exit(node.body[0]))


def canonical_entry_guard(node):
    compare = node.test if isinstance(node, ast.If) else None
    return (isinstance(compare, ast.Compare)
            and isinstance(compare.left, ast.Name)
            and compare.left.id == "__name__"
            and len(compare.ops) == len(compare.comparators) == 1
            and isinstance(compare.ops[0], ast.Eq)
            and isinstance(compare.comparators[0], ast.Constant)
            and compare.comparators[0].value == "__main__"
            and not node.orelse
            and len(node.body) == 1
            and isinstance(node.body[0], ast.Expr)
            and isinstance(node.body[0].value, ast.Call)
            and isinstance(node.body[0].value.func, ast.Name)
            and node.body[0].value.func.id == "main"
            and not node.body[0].value.args
            and not node.body[0].value.keywords)


def canonical_driver_entry(node):
    compare = node.test if isinstance(node, ast.If) else None
    statement = node.body[0] if isinstance(node, ast.If) and len(node.body) == 1 else None
    outer = statement.exc if isinstance(statement, ast.Raise) else None
    cause = statement.cause if isinstance(statement, ast.Raise) else False
    inner = outer.args[0] if isinstance(outer, ast.Call) and len(outer.args) == 1 else None
    return (isinstance(compare, ast.Compare)
            and isinstance(compare.left, ast.Name)
            and compare.left.id == "__name__"
            and len(compare.ops) == len(compare.comparators) == 1
            and isinstance(compare.ops[0], ast.Eq)
            and isinstance(compare.comparators[0], ast.Constant)
            and compare.comparators[0].value == "__main__"
            and not node.orelse
            and cause is None
            and isinstance(outer, ast.Call)
            and isinstance(outer.func, ast.Name)
            and outer.func.id == "SystemExit"
            and not outer.keywords
            and isinstance(inner, ast.Call)
            and isinstance(inner.func, ast.Name)
            and inner.func.id == "main"
            and not inner.args and not inner.keywords)


def import_signature(node):
    def alias(value):
        return value.name + ((" as " + value.asname) if value.asname else "")

    if isinstance(node, ast.Import):
        return "import " + ",".join(alias(value) for value in node.names)
    if isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
        return "from " + node.module + " import " \
            + ",".join(alias(value) for value in node.names)
    raise ValueError("launcher has a relative or malformed import")


def pure_expression(node):
    if isinstance(node, (ast.Constant, ast.Name)):
        return True
    if (isinstance(node, ast.Attribute)
            and isinstance(node.value, ast.Name)
            and node.value.id == "sys"
            and node.attr == "executable"):
        return True
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return all(pure_expression(item) for item in node.elts)
    if isinstance(node, ast.Dict):
        return all(key is not None and pure_expression(key)
                   for key in node.keys) \
            and all(pure_expression(value) for value in node.values)
    if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Sub, ast.BitOr)):
        return pure_expression(node.left) and pure_expression(node.right)
    if isinstance(node, ast.UnaryOp) \
            and isinstance(node.op, (ast.UAdd, ast.USub, ast.Not, ast.Invert)):
        return pure_expression(node.operand)
    if isinstance(node, (ast.SetComp, ast.ListComp, ast.GeneratorExp)):
        return (pure_expression(node.elt)
                and all(isinstance(generator.target, (ast.Name, ast.Tuple))
                        and pure_expression(generator.iter)
                        and all(pure_expression(value) for value in generator.ifs)
                        and not generator.is_async
                        for generator in node.generators))
    return False


def safe_function(node):
    if not isinstance(node, ast.FunctionDef) or node.returns is not None \
            or node.type_comment is not None:
        return False
    arguments = node.args
    annotated = [*arguments.posonlyargs, *arguments.args, *arguments.kwonlyargs]
    if arguments.vararg is not None:
        annotated.append(arguments.vararg)
    if arguments.kwarg is not None:
        annotated.append(arguments.kwarg)
    if any(argument.annotation is not None or argument.type_comment is not None
           for argument in annotated):
        return False
    if any(not pure_expression(value) for value in arguments.defaults):
        return False
    if any(value is not None and not pure_expression(value)
           for value in arguments.kw_defaults):
        return False
    return (not node.decorator_list
            or (len(node.decorator_list) == 1
                and isinstance(node.decorator_list[0], ast.Name)
                and node.decorator_list[0].id == "contextmanager"))


def exact_hpc_home_assignment(node):
    if not isinstance(node, ast.Assign) or len(node.targets) != 1 \
            or not isinstance(node.targets[0], ast.Name) \
            or node.targets[0].id != "_HPC_HOME":
        return False
    value = node.value
    return (isinstance(value, ast.Call)
            and isinstance(value.func, ast.Name) and value.func.id == "Path"
            and len(value.args) == 1 and not value.keywords
            and isinstance(value.args[0], ast.Constant)
            and value.args[0].value == "/public/home/majj")


def exact_audit_install(node):
    value = node.value if isinstance(node, ast.Expr) else None
    return (isinstance(value, ast.Call)
            and isinstance(value.func, ast.Attribute)
            and isinstance(value.func.value, ast.Name)
            and value.func.value.id == "sys" and value.func.attr == "addaudithook"
            and len(value.args) == 1 and not value.keywords
            and isinstance(value.args[0], ast.Name)
            and value.args[0].id == "_path_audit")


def exact_campaign_guard(node, filename):
    compare = node.test if isinstance(node, ast.If) else None
    statement = node.body[0] if isinstance(node, ast.If) and len(node.body) == 1 else None
    error = statement.exc if isinstance(statement, ast.Raise) else None
    cause = statement.cause if isinstance(statement, ast.Raise) else False
    expected_messages = {
        HISTORICAL_DRIVER: "ladder BEFORE v2 campaign policy changed",
        AFTER_DRIVER: "ladder AFTER v3 campaign policy changed",
        INDEX_DRIVER: "ladder boundary index campaign policy changed",
        WEYL_CAPTURE_DRIVER: "Weyl core capture campaign policy changed",
        WEYL_AFTER_DRIVER: "Weyl core after campaign policy changed",
    }
    return (filename in expected_messages
            and isinstance(compare, ast.Compare)
            and isinstance(compare.left, ast.Name) and compare.left.id == "ACTIVE_CAMPAIGN"
            and len(compare.ops) == len(compare.comparators) == 1
            and isinstance(compare.ops[0], ast.NotEq)
            and isinstance(compare.comparators[0], ast.Name)
            and compare.comparators[0].id == "_ACTIVE_CAMPAIGN"
            and not node.orelse
            and cause is None
            and isinstance(error, ast.Call)
            and isinstance(error.func, ast.Name) and error.func.id == "RuntimeError"
            and len(error.args) == 1 and not error.keywords
            and isinstance(error.args[0], ast.Constant)
            and error.args[0].value == expected_messages[filename])


def exact_full_stager_inventory_call(tree):
    expected = ast.parse(
        'run("full-stager-inventory", '
        '[sys.executable, "-I", "-S", "-B", "-c", '
        'FULL_STAGER_INVENTORY_PROGRAM, str(root / "hpc"), '
        'str(campaign_stage(root)), retired_reference], root / "hpc")\n'
    ).body[0].value
    calls = [
        node for node in ast.walk(function(tree, "main"))
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "run"
        and node.args
        and isinstance(node.args[0], ast.Constant)
        and node.args[0].value == "full-stager-inventory"
    ]
    return (len(calls) == 1
            and ast.dump(calls[0], include_attributes=False)
            == ast.dump(expected, include_attributes=False))


def exact_index_check_commands(tree):
    expected = ast.parse(
        'CHECK_COMMANDS = ['
        '("test-stage-ladder-boundary-index", '
        '[sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover", "-s", "hpc", '
        '"-p", "test_stage_ladder_boundary_index.py", "-v"]),'
        '("test-math-acceptance-index", '
        '[sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover", "-s", "hpc", '
        '"-p", "test_math_acceptance_index.py", "-v"]),'
        '("test-stager-allowlist", '
        '[sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover", "-s", "hpc", '
        '"-p", "test_stager_allowlist.py", "-v"]),'
        ']\n'
    ).body[0].value
    values = [
        node.value for node in tree.body
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "CHECK_COMMANDS"
    ]
    return (len(values) == 1
            and ast.dump(values[0], include_attributes=False)
            == ast.dump(expected, include_attributes=False))


def exact_index_stager_flow(tree):
    body = function(tree, "run_enabled").body
    expected = ast.parse(
        "verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)\n"
    ).body[0]
    verified = [
        offset for offset, node in enumerate(body)
        if ast.dump(node, include_attributes=False)
        == ast.dump(expected, include_attributes=False)
    ]
    mutations = [
        offset for offset, statement in enumerate(body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id in {"install_inputs", "save", "submit_pinned"}
               for node in ast.walk(statement))
    ]
    return len(verified) == 1 and bool(mutations) and verified[0] < min(mutations)


def exact_index_driver_flow(tree):
    main = function(tree, "main")
    expected_confirmed = ast.parse(
        "record = confirmed_record(root, pin, repair_intent=True, "
        "expected_job=job)\n"
    ).body[0]
    expected_publish = ast.parse(
        "success, report, report_sha256 = publish_final_report("
        "out, root, initial_inputs, pin['inputs'], commands, failure, "
        "workspace_absent, exception_log_sha256, report)\n"
    ).body[0]
    expected_verify = ast.parse(
        "verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)\n"
    ).body[0].value
    expected_true = ast.parse(
        "retired_stager_bundle_rechecked = True\n"
    ).body[0]
    assignments = [node for node in ast.walk(main) if isinstance(node, ast.Assign)]
    confirmed = [node for node in assignments
                 if isinstance(node.value, ast.Call)
                 and isinstance(node.value.func, ast.Name)
                 and node.value.func.id == "confirmed_record"]
    published = [node for node in assignments
                 if isinstance(node.value, ast.Call)
                 and isinstance(node.value.func, ast.Name)
                 and node.value.func.id == "publish_final_report"]
    verify_calls = [node for node in ast.walk(main)
                    if isinstance(node, ast.Call)
                    and isinstance(node.func, ast.Name)
                    and node.func.id == "verify_blob"]
    direct_initial = [node for node in main.body
                      if isinstance(node, ast.Expr)
                      and ast.dump(node.value, include_attributes=False)
                      == ast.dump(expected_verify, include_attributes=False)]
    tries = [node for node in main.body if isinstance(node, ast.Try)]
    return (
        len(confirmed) == 1
        and ast.dump(confirmed[0], include_attributes=False)
        == ast.dump(expected_confirmed, include_attributes=False)
        and len(published) == 1
        and ast.dump(published[0], include_attributes=False)
        == ast.dump(expected_publish, include_attributes=False)
        and len(verify_calls) == 2
        and all(ast.dump(call, include_attributes=False)
                == ast.dump(expected_verify, include_attributes=False)
                for call in verify_calls)
        and len(direct_initial) == 1
        and len(tries) == 1
        and len(tries[0].body) >= 2
        and isinstance(tries[0].body[-2], ast.Expr)
        and ast.dump(tries[0].body[-2].value, include_attributes=False)
        == ast.dump(expected_verify, include_attributes=False)
        and ast.dump(tries[0].body[-1], include_attributes=False)
        == ast.dump(expected_true, include_attributes=False)
    )


def exact_index_contract(tree, filename):
    if filename == "stage_ladder_boundary_index.py":
        return (
            literal_assignment(tree, "STAGE_NAME") == "ladder-boundary-index-v1"
            and literal_assignment(tree, "SBATCH")
            == "hpc/math_ladder_boundary_index.sbatch"
            and literal_assignment(tree, "EXPECTED_TEST_COUNTS") == {
                "test-stage-ladder-boundary-index": 20,
                "test-math-acceptance-index": 34,
                "test-stager-allowlist": 7,
            }
            and literal_assignment(tree, "RETIRED_STAGER_BUNDLE_REFERENCE") == {
                "schema": "atlas-campaign-blob-v1",
                "role": "retired-stager-bundle",
                "sha256": (
                    "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09"
                ),
                "bytes": 311925,
            }
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
            and exact_index_stager_flow(tree)
        )
    if filename == INDEX_DRIVER:
        return (
            literal_assignment(tree, "SBATCH")
            == "hpc/math_ladder_boundary_index.sbatch"
            and literal_assignment(tree, "COMMAND_TIMEOUT_SECONDS") == 300
            and literal_assignment(tree, "COMMAND_KILL_AFTER_SECONDS") == 15
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
            and literal_assignment(tree, "UNITTEST_SUMMARY_PATTERN")
            == r"^Ran ([0-9]+) tests? in [^\r\n]+\r?\n\r?\nOK[ \t]*\r?\n?\Z"
            and exact_index_check_commands(tree)
            and exact_index_driver_flow(tree)
        )
    return True


def exact_weyl_core_contract(tree, filename):
    if filename == "stage_weyl_context_core_capture.py":
        return (
            hashlib.sha256(
                getattr(tree, "_source_text", "").encode("utf-8")
            ).hexdigest() == WEYL_CAPTURE_STAGER_SHA256
            and literal_assignment(tree, "STAGE_NAME")
            == "weyl-context-core-before-v4"
            and literal_assignment(tree, "PIN_NAME")
            == "weyl-context-core-before-v4-pin.json"
            and literal_assignment(tree, "PIN_SCHEMA")
            == "atlas-weyl-context-core-before-pin-v4"
            and literal_assignment(tree, "SBATCH")
            == "hpc/math_weyl_context_core_capture.sbatch"
            and literal_assignment(tree, "EXPECTED_TEST_COUNTS") == {
                "test-campaign-stage-creation": 32,
                "test-progressive-submit": 17,
                "test-weyl-context-core-contract": 18,
                "test-weyl-context-core-regression-contract": 17,
                "test-math-weyl-context-core-capture": 28,
                "test-stager-allowlist": 7,
            }
            and literal_assignment(tree, "CHECKER_TESTS") == 119
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
            and literal_assignment(tree, "FORBIDDEN_DURABLE_STAGE_NAMES")
            == ("source", "target", "workspace", "build")
            and literal_assignment(tree, "CURRENT_STAGER_PATH")
            == "hpc/stage_weyl_context_core_capture.py"
            and literal_assignment(
                tree, "PRIOR_CREATION_FAILURE_EVIDENCE") == {
                    "file": (
                        "tests/reference/hpc/"
                        "weyl_context_core_stage_creation_renameat2_failure_2026_10_02.json"
                    ),
                    "sha256": (
                        "74d4114f397c84286d0d9aba55ba88f12f4a0607ca93ebe3ef6eeba5911c7094"
                    ),
                }
            and literal_assignment(tree, "PRIOR_CREATION_FAILURE") == {
                "schema": "atlas-stage-creation-prior-failure-v1",
                "temporary": (
                    ".atlas-publish-"
                    "bd83ec4e79a16385851f0105ba5482376d0af5d360160beed6b23315cc1b6c1e.tmp"
                ),
                "archive": (
                    ".atlas-stage-creation-failure-"
                    "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
                ),
                "sha256": (
                    "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5"
                ),
                "bytes": 5481,
                "mode": "0444",
                "destination": ".atlas-stage-creation-prepared.json",
                "contract_sha256": (
                    "c046e43b744cadab9b4f682fa9283623a1589394950064b2e4634fe2998620d3"
                ),
                "transaction": (
                    ".atlas-stage-creation-c046e43b744cadab9b4f682f.txn"
                ),
            }
            and literal_assignment(tree, "FROZEN_LAUNCHER_HASHES") == {
                "hpc/stage_weyl_parent_seal.py": (
                    "f1c3567ab620269c3836bdc624b9573cc89d519944bfe00a7c24e859fe387fc4"
                ),
                "hpc/stage_ladder_boundary_before.py": (
                    "ff1f492adfa22f342bd0729f3df3885678de8f729a0f3d0775e5738c7ab47932"
                ),
                "hpc/math_ladder_boundary_before.py": (
                    "21dc2207690a8cd0559854eb78725c5e0d901a7e5c1c750d4feb89d55e7b8f9e"
                ),
                "hpc/stage_ladder_boundary_after.py": (
                    "07558c08a25f0d00b49720392b6abe469b2286d38b29cbb63a611cf40930d90d"
                ),
                "hpc/math_ladder_boundary_after.py": (
                    "b6f52b1042393705498767a687175f365c1fd0f491c3f630ca4cf53d2277d0a2"
                ),
                "hpc/stage_ladder_boundary_index.py": (
                    "0af4060de2bc5c451c0dab15b492d258c058d206bb06d151d37efcd57ec2008e"
                ),
                "hpc/math_ladder_boundary_index.py": (
                    "29eb6392458aa0d856ca88a1fba6d5eefc4e265f2e5ec5a5ba1ba41bdcf11c31"
                ),
                "hpc/math_weyl_context_core_capture.py": (
                    "a8f784181cf4ec02d58ce00e5b64c0f10910ef975f65f7b53784be9d540e37ab"
                ),
            }
            and literal_assignment(tree, "FROZEN_LAUNCHER_STATES") == {
                "hpc/stage_weyl_parent_seal.py": "retired",
                "hpc/stage_ladder_boundary_before.py": False,
                "hpc/math_ladder_boundary_before.py": False,
                "hpc/stage_ladder_boundary_after.py": False,
                "hpc/math_ladder_boundary_after.py": False,
                "hpc/stage_ladder_boundary_index.py": False,
                "hpc/math_ladder_boundary_index.py": False,
                "hpc/stage_weyl_context_core_capture.py": True,
                "hpc/math_weyl_context_core_capture.py": True,
            }
            and literal_assignment(tree, "RETIRED_STAGER_BUNDLE_REFERENCE") == {
                "schema": "atlas-campaign-blob-v1",
                "role": "retired-stager-bundle",
                "sha256": (
                    "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09"
                ),
                "bytes": 311925,
            }
            and exact_weyl_capture_stager_flow(tree)
        )
    if filename == WEYL_CAPTURE_DRIVER:
        return (
            hashlib.sha256(
                getattr(tree, "_source_text", "").encode("utf-8")
            ).hexdigest() == WEYL_CAPTURE_DRIVER_SHA256
            and literal_assignment(tree, "EXPECTED_STAGE")
            == (
                "/public/home/majj/atlas-rust-campaign-20260930/stages/"
                "weyl-context-core-before-v4"
            )
            and literal_assignment(tree, "REPORT_SCHEMA")
            == "atlas-weyl-context-core-before-v4"
            and literal_assignment(tree, "CHECKER_PYTHON")
            == "/public/software/anaconda/anaconda3-2022.5/bin/python3.9"
            and literal_assignment(tree, "SUCCESS_STATUS")
            == "WEYL_CONTEXT_BEFORE_EXPECTED_FAILURES_OBSERVED"
            and literal_assignment(tree, "INCOMPLETE_STATUS")
            == "WEYL_CONTEXT_CORE_BEFORE_INCOMPLETE"
            and literal_assignment(tree, "EXPECTED_INVOCATIONS") == (
                ("weyl_context_core_cold_dual", "oracle"),
                ("weyl_context_core_cold_dual", "rust"),
                ("weyl_context_core_prewarmed_dual", "rust"),
                ("weyl_context_core_prewarmed_dual", "oracle"),
            )
            and literal_assignment(tree, "COMMAND_NAMES") == (
                "test-campaign-stage-creation",
                "test-progressive-submit",
                "test-weyl-context-core-contract",
                "test-weyl-context-core-regression-contract",
                "test-math-weyl-context-core-capture",
                "test-stager-allowlist", "rustc-version", "cargo-version",
                "source-reconstruction", "release-build",
                "atlas-core-test-inventory", "weyl-context-regressions",
                "root-ladder-control",
            )
            and weyl_command_cwds(tree) == {
                "test-campaign-stage-creation": ".",
                "test-progressive-submit": ".",
                "test-weyl-context-core-contract": ".",
                "test-weyl-context-core-regression-contract": ".",
                "test-math-weyl-context-core-capture": ".",
                "test-stager-allowlist": ".",
                "rustc-version": ".",
                "cargo-version": ".",
                "source-reconstruction": "workspace",
                "release-build": "workspace/source",
                "atlas-core-test-inventory": "workspace/source",
                "weyl-context-regressions": "workspace/source",
                "root-ladder-control": "workspace/source",
            }
            and "executable_bytes"
            in literal_assignment(tree, "INVOCATION_KEYS")
            and exact_weyl_creation_gate(tree)
            and exact_weyl_execution_binding(tree)
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
        )
    if filename == "stage_weyl_context_core_after.py":
        return (
            hashlib.sha256(
                getattr(tree, "_source_text", "").encode("utf-8")
            ).hexdigest() == WEYL_AFTER_STAGER_SHA256
            and literal_assignment(tree, "STAGE_NAME")
            == "weyl-context-core-after-v5"
            and literal_assignment(tree, "PIN_NAME")
            == "weyl-context-core-after-v5-pin.json"
            and literal_assignment(tree, "PIN_SCHEMA")
            == "atlas-weyl-context-core-after-pin-v5"
            and literal_assignment(tree, "SBATCH")
            == "hpc/math_weyl_context_core_after.sbatch"
            and literal_assignment(tree, "EXPECTED_TEST_COUNTS") == {
                "test-campaign-stage-creation": 32,
                "test-progressive-submit": 17,
                "test-weyl-context-core-contract": 20,
                "test-weyl-context-core-regression-contract": 21,
                "test-math-weyl-context-core-after": 30,
                "test-stager-allowlist": 7,
            }
            and literal_assignment(tree, "CHECKER_TESTS") == 127
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
            and literal_assignment(tree, "FORBIDDEN_DURABLE_STAGE_NAMES")
            == ("source", "target", "workspace", "build")
            and literal_assignment(tree, "CURRENT_STAGER_PATH")
            == "hpc/stage_weyl_context_core_after.py"
            and literal_assignment(tree, "FROZEN_LAUNCHER_HASHES") == {
                "hpc/stage_weyl_parent_seal.py": (
                    "f1c3567ab620269c3836bdc624b9573cc89d519944bfe00a7c24e859fe387fc4"
                ),
                "hpc/stage_ladder_boundary_before.py": (
                    "ff1f492adfa22f342bd0729f3df3885678de8f729a0f3d0775e5738c7ab47932"
                ),
                "hpc/math_ladder_boundary_before.py": (
                    "21dc2207690a8cd0559854eb78725c5e0d901a7e5c1c750d4feb89d55e7b8f9e"
                ),
                "hpc/stage_ladder_boundary_after.py": (
                    "07558c08a25f0d00b49720392b6abe469b2286d38b29cbb63a611cf40930d90d"
                ),
                "hpc/math_ladder_boundary_after.py": (
                    "b6f52b1042393705498767a687175f365c1fd0f491c3f630ca4cf53d2277d0a2"
                ),
                "hpc/stage_ladder_boundary_index.py": (
                    "0af4060de2bc5c451c0dab15b492d258c058d206bb06d151d37efcd57ec2008e"
                ),
                "hpc/math_ladder_boundary_index.py": (
                    "29eb6392458aa0d856ca88a1fba6d5eefc4e265f2e5ec5a5ba1ba41bdcf11c31"
                ),
                "hpc/stage_weyl_context_core_capture.py": (
                    "639b5c20d1a1efc68eebb3a240d9d2aa717c1c90757321c6b3445652fad649a6"
                ),
                "hpc/math_weyl_context_core_capture.py": (
                    "31f5aef26e14ca669ee9306efc70cc2c4f8f22fd19af11dab5e979a0296966d7"
                ),
                "hpc/math_weyl_context_core_after.py": (
                    "ab99c7a62b8c24dcba74cca2531f00f568cee17cb04702f602d59dbbabec0c3a"
                ),
            }
            and literal_assignment(tree, "FROZEN_LAUNCHER_STATES") == {
                "hpc/stage_weyl_parent_seal.py": "retired",
                "hpc/stage_ladder_boundary_before.py": False,
                "hpc/math_ladder_boundary_before.py": False,
                "hpc/stage_ladder_boundary_after.py": False,
                "hpc/math_ladder_boundary_after.py": False,
                "hpc/stage_ladder_boundary_index.py": False,
                "hpc/math_ladder_boundary_index.py": False,
                "hpc/stage_weyl_context_core_capture.py": False,
                "hpc/math_weyl_context_core_capture.py": False,
                "hpc/stage_weyl_context_core_after.py": True,
                "hpc/math_weyl_context_core_after.py": True,
            }
            and literal_assignment(tree, "RETIRED_STAGER_BUNDLE_REFERENCE") == {
                "schema": "atlas-campaign-blob-v1",
                "role": "retired-stager-bundle",
                "sha256": (
                    "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09"
                ),
                "bytes": 311925,
            }
            and exact_weyl_after_stager_flow(tree)
        )
    if filename == WEYL_AFTER_DRIVER:
        return (
            hashlib.sha256(
                getattr(tree, "_source_text", "").encode("utf-8")
            ).hexdigest() == WEYL_AFTER_DRIVER_SHA256
            and literal_assignment(tree, "EXPECTED_STAGE")
            == (
                "/public/home/majj/atlas-rust-campaign-20260930/stages/"
                "weyl-context-core-after-v5"
            )
            and literal_assignment(tree, "REPORT_SCHEMA")
            == "atlas-weyl-context-core-after-v5"
            and literal_assignment(tree, "CHECKER_PYTHON")
            == "/public/software/anaconda/anaconda3-2022.5/bin/python3.9"
            and literal_assignment(tree, "SUCCESS_STATUS")
            == "WEYL_CONTEXT_AFTER_REGRESSIONS_PASS"
            and literal_assignment(tree, "INCOMPLETE_STATUS")
            == "WEYL_CONTEXT_CORE_AFTER_INCOMPLETE"
            and literal_assignment(tree, "EXPECTED_INVOCATIONS") == (
                ("weyl_context_core_cold_dual", "oracle"),
                ("weyl_context_core_cold_dual", "rust"),
                ("weyl_context_core_prewarmed_dual", "rust"),
                ("weyl_context_core_prewarmed_dual", "oracle"),
            )
            and literal_assignment(tree, "COMMAND_NAMES") == (
                "test-campaign-stage-creation",
                "test-progressive-submit",
                "test-weyl-context-core-contract",
                "test-weyl-context-core-regression-contract",
                "test-math-weyl-context-core-after",
                "test-stager-allowlist", "rustc-version", "cargo-version",
                "source-reconstruction", "release-build",
                "atlas-core-test-inventory", "weyl-context-regressions",
                "root-ladder-control",
            )
            and weyl_command_cwds(tree) == {
                "test-campaign-stage-creation": ".",
                "test-progressive-submit": ".",
                "test-weyl-context-core-contract": ".",
                "test-weyl-context-core-regression-contract": ".",
                "test-math-weyl-context-core-after": ".",
                "test-stager-allowlist": ".",
                "rustc-version": ".",
                "cargo-version": ".",
                "source-reconstruction": "workspace",
                "release-build": "workspace/source",
                "atlas-core-test-inventory": "workspace/source",
                "weyl-context-regressions": "workspace/source",
                "root-ladder-control": "workspace/source",
            }
            and "executable_bytes"
            in literal_assignment(tree, "INVOCATION_KEYS")
            and exact_weyl_after_creation_gate(tree)
            and exact_weyl_execution_binding(
                tree, WEYL_AFTER_EXECUTION_SOURCE_SHA256)
            and literal_assignment(tree, "SHA256_PATTERN")
            == r"[0-9a-f]{64}\Z"
        )
    return True


def exact_weyl_creation_gate(tree):
    gates = function(tree, "gates")
    expected_confirm = ast.parse(
        "confirmed_record(root, pin, repair_intent=True, expected_job=job)\n"
    ).body[0].value
    expected_creation = ast.parse(
        "validate_stage_creation(\n"
        "    root, record['stage_creation_sha256'], pin_sha256=pin_sha,\n"
        ")\n"
    ).body[0].value
    expected_receipt = ast.parse(
        "read_json_file(root / 'submission.json')\n"
    ).body[0].value
    expected_v5_failure = ast.parse(
        "validate_capture_v5_failure(root, inputs)\n"
    ).body[0].value
    expected_v6_failure = ast.parse(
        "validate_capture_v6_failure(root, inputs)\n"
    ).body[0].value
    expected_v7_failure = ast.parse(
        "validate_capture_v7_failure(root, inputs)\n"
    ).body[0].value
    expected_v8 = ast.parse(
        "validate_capture_v8(root, inputs)\n"
    ).body[0].value
    expected_before_v1 = ast.parse(
        "validate_before_v1_failure(root, inputs)\n"
    ).body[0].value
    expected_before_v2 = ast.parse(
        "validate_before_v2_failure(root, inputs)\n"
    ).body[0].value
    expected_before_v3 = ast.parse(
        "validate_before_v3_failure(root, inputs)\n"
    ).body[0].value
    expected_catalog = ast.parse(
        "validate_staged_catalog(root, inputs)\n"
    ).body[0].value
    expected_regression = ast.parse(
        "validate_regression_inputs(root, inputs, accepted_source_manifest)\n"
    ).body[0].value

    def exact_calls(expected):
        return [
            node for node in ast.walk(gates)
            if isinstance(node, ast.Call)
            and ast.dump(node, include_attributes=False)
               == ast.dump(expected, include_attributes=False)
        ]

    confirmed = exact_calls(expected_confirm)
    creation = exact_calls(expected_creation)
    receipt = exact_calls(expected_receipt)
    v5_failure = exact_calls(expected_v5_failure)
    v6_failure = exact_calls(expected_v6_failure)
    v7_failure = exact_calls(expected_v7_failure)
    v8 = exact_calls(expected_v8)
    before_v1 = exact_calls(expected_before_v1)
    before_v2 = exact_calls(expected_before_v2)
    before_v3 = exact_calls(expected_before_v3)
    catalog = exact_calls(expected_catalog)
    regression = exact_calls(expected_regression)
    return (len(confirmed) == len(creation) == len(receipt) == 1
            and len(v5_failure) == len(v6_failure) == len(v7_failure) == 1
            and len(v8) == len(before_v1) == len(before_v2) == len(before_v3) == 1
            and len(catalog) == len(regression) == 1
            and confirmed[0].lineno < creation[0].lineno < receipt[0].lineno
            and receipt[0].lineno < v5_failure[0].lineno
            and v5_failure[0].lineno < v6_failure[0].lineno
            and v6_failure[0].lineno < v7_failure[0].lineno
            and v7_failure[0].lineno < v8[0].lineno
            and v8[0].lineno < before_v1[0].lineno
            and before_v1[0].lineno < before_v2[0].lineno
            and before_v2[0].lineno < before_v3[0].lineno
            and before_v3[0].lineno < catalog[0].lineno
            and catalog[0].lineno < regression[0].lineno)


def exact_weyl_after_creation_gate(tree):
    gates = function(tree, "gates")
    expected_confirm = ast.parse(
        "confirmed_record(root, pin, repair_intent=True, expected_job=job)\n"
    ).body[0].value
    expected_creation = ast.parse(
        "validate_stage_creation(\n"
        "    root, record['stage_creation_sha256'], pin_sha256=pin_sha,\n"
        ")\n"
    ).body[0].value
    expected_receipt = ast.parse(
        "read_json_file(root / 'submission.json')\n"
    ).body[0].value
    expected_v5_failure = ast.parse(
        "validate_capture_v5_failure(root, inputs)\n"
    ).body[0].value
    expected_v6_failure = ast.parse(
        "validate_capture_v6_failure(root, inputs)\n"
    ).body[0].value
    expected_v7_failure = ast.parse(
        "validate_capture_v7_failure(root, inputs)\n"
    ).body[0].value
    expected_v8 = ast.parse(
        "validate_capture_v8(root, inputs)\n"
    ).body[0].value
    expected_before_v1 = ast.parse(
        "validate_before_v1_failure(root, inputs)\n"
    ).body[0].value
    expected_before_v2 = ast.parse(
        "validate_before_v2_failure(root, inputs)\n"
    ).body[0].value
    expected_before_v3 = ast.parse(
        "validate_before_v3_failure(root, inputs)\n"
    ).body[0].value
    expected_before_v4 = ast.parse(
        "validate_before_v4_result(root, inputs)\n"
    ).body[0].value
    expected_after_v2 = ast.parse(
        "validate_after_v2_failure(root, inputs)\n"
    ).body[0].value
    expected_after_v3 = ast.parse(
        "validate_after_v3_failure(root, inputs)\n"
    ).body[0].value
    expected_after_v4 = ast.parse(
        "validate_after_v4_failure(root, inputs)\n"
    ).body[0].value
    expected_catalog = ast.parse(
        "validate_staged_catalog(root, inputs)\n"
    ).body[0].value
    expected_regression = ast.parse(
        "validate_regression_inputs(root, inputs, accepted_source_manifest)\n"
    ).body[0].value
    expected_repaired = ast.parse(
        "repaired_source_manifest(regression_source_manifest)\n"
    ).body[0].value

    def exact_calls(expected):
        return [
            node for node in ast.walk(gates)
            if isinstance(node, ast.Call)
            and ast.dump(node, include_attributes=False)
               == ast.dump(expected, include_attributes=False)
        ]

    confirmed = exact_calls(expected_confirm)
    creation = exact_calls(expected_creation)
    receipt = exact_calls(expected_receipt)
    v5_failure = exact_calls(expected_v5_failure)
    v6_failure = exact_calls(expected_v6_failure)
    v7_failure = exact_calls(expected_v7_failure)
    v8 = exact_calls(expected_v8)
    before_v1 = exact_calls(expected_before_v1)
    before_v2 = exact_calls(expected_before_v2)
    before_v3 = exact_calls(expected_before_v3)
    before_v4 = exact_calls(expected_before_v4)
    after_v2 = exact_calls(expected_after_v2)
    after_v3 = exact_calls(expected_after_v3)
    after_v4 = exact_calls(expected_after_v4)
    catalog = exact_calls(expected_catalog)
    regression = exact_calls(expected_regression)
    repaired = exact_calls(expected_repaired)
    return (len(confirmed) == len(creation) == len(receipt) == 1
            and len(v5_failure) == len(v6_failure) == len(v7_failure) == 1
            and len(v8) == len(before_v1) == len(before_v2) == len(before_v3) == 1
            and len(before_v4) == 1
            and len(after_v2) == len(after_v3) == len(after_v4) == 1
            and len(catalog) == len(regression) == len(repaired) == 1
            and confirmed[0].lineno < creation[0].lineno < receipt[0].lineno
            and receipt[0].lineno < v5_failure[0].lineno
            and v5_failure[0].lineno < v6_failure[0].lineno
            and v6_failure[0].lineno < v7_failure[0].lineno
            and v7_failure[0].lineno < v8[0].lineno
            and v8[0].lineno < before_v1[0].lineno
            and before_v1[0].lineno < before_v2[0].lineno
            and before_v2[0].lineno < before_v3[0].lineno
            and before_v3[0].lineno < before_v4[0].lineno
            and before_v4[0].lineno < after_v2[0].lineno
            and after_v2[0].lineno < after_v3[0].lineno
            and after_v3[0].lineno < after_v4[0].lineno
            and after_v4[0].lineno < catalog[0].lineno
            and catalog[0].lineno < regression[0].lineno
            and regression[0].lineno < repaired[0].lineno)


def weyl_command_cwds(tree):
    assignments = [
        node for node in tree.body
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "COMMAND_CONTRACTS"
    ]
    if len(assignments) != 1 or not isinstance(assignments[0].value, ast.Dict):
        return None
    result = {}
    for key, value in zip(assignments[0].value.keys, assignments[0].value.values):
        if (not isinstance(key, ast.Constant) or not isinstance(key.value, str)
                or not isinstance(value, ast.Dict)):
            return None
        cwd_values = [
            item for item_key, item in zip(value.keys, value.values)
            if isinstance(item_key, ast.Constant) and item_key.value == "cwd"
        ]
        if (len(cwd_values) != 1
                or not isinstance(cwd_values[0], ast.Constant)
                or not isinstance(cwd_values[0].value, str)):
            return None
        result[key.value] = cwd_values[0].value
    return result


def exact_weyl_execution_binding(tree, expected_hashes=None):
    if expected_hashes is None:
        expected_hashes = WEYL_EXECUTION_SOURCE_SHA256
    contract_assignments = [
        node for node in tree.body
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "COMMAND_CONTRACTS"
    ]
    if len(contract_assignments) != 1:
        return False
    frozen_nodes = {
        "COMMAND_CONTRACTS": contract_assignments[0],
        **{
            name: definition(tree, name)
            for name in expected_hashes
            if name != "COMMAND_CONTRACTS"
        },
    }
    if ({name: source_node_sha256(tree, node)
         for name, node in frozen_nodes.items()}
            != expected_hashes):
        return False
    command_cwd = function(tree, "_command_working_directory")
    command = function(tree, "_command_record")
    capture = function(tree, "_capture_record")
    execute = function(tree, "_execute_timed")
    names = lambda node: [
        value.arg for value in (*node.args.posonlyargs, *node.args.args)
    ]
    if (names(command_cwd) != ["label", "root", "work"]
            or names(command) != [
                "out", "name", "root", "work", "env", "metrics_path",
                "active",
            ]
            or names(capture) != [
                "out", "plan", "executables", "binary_records", "scripts",
                "env", "input_bytes", "metrics_path", "active", "job",
            ]
            or names(execute) != [
                "argv", "cwd", "env", "input_bytes", "timeout_seconds",
                "metrics_path", "active", "pass_fds",
            ]):
        return False
    expected_locations = ast.parse(
        'locations = {".": root, "workspace": work, '
        '"workspace/source": work / "source"}\n'
    ).body[0]
    locations = [
        node for node in ast.walk(command_cwd)
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "locations"
    ]
    expected_cwd_assignment = ast.parse(
        "cwd = _command_working_directory(cwd_label, root, work)\n"
    ).body[0]
    cwd_assignments = [
        node for node in ast.walk(command)
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "cwd"
    ]
    expected_command_run = ast.parse(
        "_execute_timed(argv, cwd, env, b'', COMMAND_TIMEOUT_SECONDS, "
        "metrics_path, active)\n"
    ).body[0].value
    command_runs = [
        node for node in ast.walk(command)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "_execute_timed"
    ]
    expected_wrapper = ast.parse(
        "wrapper = [\n"
        "    '/bin/bash', '--noprofile', '--norc', '-c',\n"
        "    'exec -a \"$1\" \"/proc/self/fd/$2\"',\n"
        "    'atlas-capture-exec', str(binary), str(descriptor),\n"
        "]\n"
    ).body[0]
    wrappers = [
        node for node in ast.walk(capture)
        if isinstance(node, ast.Assign) and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id == "wrapper"
    ]
    capture_runs = [
        node for node in ast.walk(capture)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "_execute_timed"
    ]
    popen_calls = [
        node for node in ast.walk(execute)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == "subprocess" and node.func.attr == "Popen"
    ]
    expected_capture_run = ast.parse(
        "_execute_timed(wrapper, scripts, env, input_bytes, "
        "plan['timeout_seconds'], metrics_path, active, "
        "pass_fds=(descriptor,))\n"
    ).body[0].value
    expected_popen = ast.parse(
        "subprocess.Popen(\n"
        "    ['/usr/bin/time', '-v', '-o', str(metrics_path), *argv],\n"
        "    cwd=cwd, env=env, stdin=subprocess.PIPE,\n"
        "    stdout=subprocess.PIPE, stderr=subprocess.PIPE,\n"
        "    start_new_session=True, close_fds=True, pass_fds=pass_fds,\n"
        ")\n"
    ).body[0].value
    execute_rebindings = [
        node.id for node in ast.walk(execute)
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store)
        and node.id in {"argv", "cwd", "env", "pass_fds"}
    ]
    execute_defaults = execute.args.defaults
    return (
        len(locations) == 1
        and ast.dump(locations[0], include_attributes=False)
        == ast.dump(expected_locations, include_attributes=False)
        and len(cwd_assignments) == 1
        and ast.dump(cwd_assignments[0], include_attributes=False)
        == ast.dump(expected_cwd_assignment, include_attributes=False)
        and len(command_runs) == 1
        and ast.dump(command_runs[0], include_attributes=False)
        == ast.dump(expected_command_run, include_attributes=False)
        and len(wrappers) == 1
        and ast.dump(wrappers[0], include_attributes=False)
        == ast.dump(expected_wrapper, include_attributes=False)
        and len(capture_runs) == 1
        and ast.dump(capture_runs[0], include_attributes=False)
        == ast.dump(expected_capture_run, include_attributes=False)
        and len(popen_calls) == 1
        and ast.dump(popen_calls[0], include_attributes=False)
        == ast.dump(expected_popen, include_attributes=False)
        and execute_rebindings == []
        and len(execute_defaults) == 1
        and isinstance(execute_defaults[0], ast.Tuple)
        and execute_defaults[0].elts == []
    )


def exact_weyl_capture_stager_flow(tree):
    parent = function(tree, "validate_parent_objects")
    expected_verify = ast.parse(
        "verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)\n"
    ).body[0].value
    verifies = [
        node for node in ast.walk(parent)
        if isinstance(node, ast.Call)
        and ast.dump(node, include_attributes=False)
        == ast.dump(expected_verify, include_attributes=False)
    ]
    run = function(tree, "run_enabled")
    validations = [
        offset for offset, statement in enumerate(run.body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id == "validate_parent_objects"
               for node in ast.walk(statement))
    ]
    failure_validations = [
        offset for offset, statement in enumerate(run.body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id == "validate_prior_creation_failure"
               for node in ast.walk(statement))
    ]
    v1_validations = [
        offset for offset, statement in enumerate(run.body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id == "validate_capture_v1_failure"
               for node in ast.walk(statement))
    ]
    v2_validations = [
        offset for offset, statement in enumerate(run.body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id == "validate_capture_v2_failure"
               for node in ast.walk(statement))
    ]
    v3_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v3_failure"],
        key=lambda node: node.lineno,
    )
    v4_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v4_failure"],
        key=lambda node: node.lineno,
    )
    v5_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v5_failure"],
        key=lambda node: node.lineno,
    )
    v6_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v6_failure"],
        key=lambda node: node.lineno,
    )
    v7_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v7_failure"],
        key=lambda node: node.lineno,
    )
    v8_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_capture_v8"],
        key=lambda node: node.lineno,
    )
    regression_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_regression_inputs"],
        key=lambda node: node.lineno,
    )
    before_v1_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_before_v1_failure"],
        key=lambda node: node.lineno,
    )
    before_v2_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_before_v2_failure"],
        key=lambda node: node.lineno,
    )
    before_v3_validation_calls = sorted(
        [node for node in ast.walk(run)
         if isinstance(node, ast.Call)
         and isinstance(node.func, ast.Name)
         and node.func.id == "validate_before_v3_failure"],
        key=lambda node: node.lineno,
    )
    install_calls = [
        node for node in ast.walk(run)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "install_inputs"
    ]
    creation_calls = [
        node for node in ast.walk(run)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "create_fixed_stage"
    ]
    build_pin_calls = [
        node for node in ast.walk(run)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
        and node.func.id == "build_pin"
    ]
    prior_arguments = [
        keyword.value for call in creation_calls for keyword in call.keywords
        if keyword.arg == "prior_failure"
    ]
    mutations = [
        offset for offset, statement in enumerate(run.body)
        if any(isinstance(node, ast.Call)
               and isinstance(node.func, ast.Name)
               and node.func.id in {
                   "create_fixed_stage", "install_inputs", "save",
                   "submit_pinned",
               }
               for node in ast.walk(statement))
    ]
    return (len(verifies) == 1 and len(validations) == 1
            and len(failure_validations) == 2
            and len(v1_validations) == 2 and len(v2_validations) == 2
            and len(v3_validation_calls) == 2
            and len(v4_validation_calls) == 2
            and len(v5_validation_calls) == 2
            and len(v6_validation_calls) == 2
            and len(v7_validation_calls) == 2
            and len(v8_validation_calls) == 2
            and len(before_v1_validation_calls) == 2
            and len(before_v2_validation_calls) == 2
            and len(before_v3_validation_calls) == 2
            and len(regression_validation_calls) == 2
            and len(install_calls) == 1
            and bool(mutations)
            and validations[0] < min(mutations)
            and failure_validations[0] < min(mutations)
            and v1_validations[0] < min(mutations)
            and v2_validations[0] < min(mutations)
            and len(creation_calls) == 1
            and v3_validation_calls[0].lineno < creation_calls[0].lineno
            and creation_calls[0].lineno < install_calls[0].lineno
            and install_calls[0].lineno < v3_validation_calls[1].lineno
            and len(build_pin_calls) == 1
            and v4_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < v4_validation_calls[1].lineno
            and v4_validation_calls[1].lineno < build_pin_calls[0].lineno
            and v5_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < v5_validation_calls[1].lineno
            and v5_validation_calls[1].lineno < build_pin_calls[0].lineno
            and v6_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < v6_validation_calls[1].lineno
            and v6_validation_calls[1].lineno < build_pin_calls[0].lineno
            and v7_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < v7_validation_calls[1].lineno
            and v7_validation_calls[1].lineno < build_pin_calls[0].lineno
            and v8_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < v8_validation_calls[1].lineno
            and v8_validation_calls[1].lineno < build_pin_calls[0].lineno
            and before_v1_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < before_v1_validation_calls[1].lineno
            and before_v1_validation_calls[1].lineno < build_pin_calls[0].lineno
            and before_v2_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < before_v2_validation_calls[1].lineno
            and before_v2_validation_calls[1].lineno < build_pin_calls[0].lineno
            and before_v3_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < before_v3_validation_calls[1].lineno
            and before_v3_validation_calls[1].lineno < build_pin_calls[0].lineno
            and regression_validation_calls[0].lineno < creation_calls[0].lineno
            and install_calls[0].lineno < regression_validation_calls[1].lineno
            and regression_validation_calls[1].lineno < build_pin_calls[0].lineno
            and v5_validation_calls[0].lineno
                < v6_validation_calls[0].lineno
            and v5_validation_calls[1].lineno
                < v6_validation_calls[1].lineno
            and v6_validation_calls[0].lineno
                < v7_validation_calls[0].lineno
            and v6_validation_calls[1].lineno
                < v7_validation_calls[1].lineno
            and v7_validation_calls[0].lineno < v8_validation_calls[0].lineno
            and v7_validation_calls[1].lineno < v8_validation_calls[1].lineno
            and v8_validation_calls[0].lineno
                < before_v1_validation_calls[0].lineno
            and v8_validation_calls[1].lineno
                < before_v1_validation_calls[1].lineno
            and before_v1_validation_calls[0].lineno
                < before_v2_validation_calls[0].lineno
            and before_v1_validation_calls[1].lineno
                < before_v2_validation_calls[1].lineno
            and before_v2_validation_calls[0].lineno
                < before_v3_validation_calls[0].lineno
            and before_v2_validation_calls[1].lineno
                < before_v3_validation_calls[1].lineno
            and before_v3_validation_calls[0].lineno
                < regression_validation_calls[0].lineno
            and before_v3_validation_calls[1].lineno
                < regression_validation_calls[1].lineno
            and prior_arguments == [])


def exact_weyl_after_stager_flow(tree):
    parent = function(tree, "validate_parent_objects")
    expected_verify = ast.parse(
        "verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)\n"
    ).body[0].value
    verifies = [
        node for node in ast.walk(parent)
        if isinstance(node, ast.Call)
        and ast.dump(node, include_attributes=False)
        == ast.dump(expected_verify, include_attributes=False)
    ]
    run = function(tree, "run_enabled")

    def statement_offsets(names):
        return [
            offset for offset, statement in enumerate(run.body)
            if any(isinstance(node, ast.Call)
                   and isinstance(node.func, ast.Name)
                   and node.func.id in names
                   for node in ast.walk(statement))
        ]

    def call_chain(name):
        return sorted(
            (node for node in ast.walk(run)
             if isinstance(node, ast.Call)
             and isinstance(node.func, ast.Name)
             and node.func.id == name),
            key=lambda node: node.lineno,
        )

    validations = statement_offsets({"validate_parent_objects"})
    failure_validations = statement_offsets(
        {"validate_prior_creation_failure"})
    mutations = statement_offsets(
        {"create_fixed_stage", "install_inputs", "save", "submit_pinned"})
    install_calls = call_chain("install_inputs")
    creation_calls = call_chain("create_fixed_stage")
    build_pin_calls = call_chain("build_pin")
    prior_arguments = [
        keyword.value for call in creation_calls for keyword in call.keywords
        if keyword.arg == "prior_failure"
    ]
    chains = {
        name: call_chain(name)
        for name in (
            "validate_capture_v1_failure", "validate_capture_v2_failure",
            "validate_capture_v3_failure", "validate_capture_v4_failure",
            "validate_capture_v5_failure", "validate_capture_v6_failure",
            "validate_capture_v7_failure", "validate_capture_v8",
            "validate_before_v1_failure", "validate_before_v2_failure",
            "validate_before_v3_failure", "validate_before_v4_result",
            "validate_after_v2_failure", "validate_after_v3_failure",
            "validate_after_v4_failure", "validate_regression_inputs",
        )
    }
    ordered_pairs = (
        ("validate_capture_v5_failure", "validate_capture_v6_failure"),
        ("validate_capture_v6_failure", "validate_capture_v7_failure"),
        ("validate_capture_v7_failure", "validate_capture_v8"),
        ("validate_capture_v8", "validate_before_v1_failure"),
        ("validate_before_v1_failure", "validate_before_v2_failure"),
        ("validate_before_v2_failure", "validate_before_v3_failure"),
        ("validate_before_v3_failure", "validate_before_v4_result"),
        ("validate_before_v4_result", "validate_after_v2_failure"),
        ("validate_after_v2_failure", "validate_after_v3_failure"),
        ("validate_after_v3_failure", "validate_after_v4_failure"),
        ("validate_after_v4_failure", "validate_regression_inputs"),
    )
    return (len(verifies) == 1 and len(validations) == 1
            and len(failure_validations) == 2
            and all(len(chain) == 2 for chain in chains.values())
            and len(install_calls) == 1
            and len(creation_calls) == 1
            and len(build_pin_calls) == 1
            and bool(mutations)
            and validations[0] < min(mutations)
            and failure_validations[0] < min(mutations)
            and creation_calls[0].lineno < install_calls[0].lineno
            and all(chain[0].lineno < creation_calls[0].lineno
                    for chain in chains.values())
            and all(install_calls[0].lineno < chain[1].lineno
                    for chain in chains.values())
            and all(chain[1].lineno < build_pin_calls[0].lineno
                    for chain in chains.values())
            and all(chains[before][0].lineno < chains[after][0].lineno
                    and chains[before][1].lineno < chains[after][1].lineno
                    for before, after in ordered_pairs)
            and prior_arguments == [])


def bounded_module(tree, filename):
    if filename not in EXPECTED_IMPORTS:
        raise ValueError("unknown launcher filename")
    top_level_imports = {
        id(node) for node in tree.body
        if isinstance(node, (ast.Import, ast.ImportFrom))
    }
    for node in ast.walk(tree):
        if (isinstance(node, (ast.Import, ast.ImportFrom))
                and id(node) not in top_level_imports):
            raise ValueError("launcher has a nested import outside the exact allowlist")
        if isinstance(node, ast.Name) and node.id in {"__builtins__", "importlib"}:
            raise ValueError("launcher names a dynamic code-loading surface")
        if isinstance(node, ast.Call):
            if (isinstance(node.func, ast.Name)
                    and node.func.id in {"__import__", "eval", "exec", "compile"}):
                raise ValueError("launcher has a dynamic code-loading call")
            if (isinstance(node.func, ast.Attribute)
                    and node.func.attr in {
                        "__import__", "eval", "exec", "compile",
                        "import_module", "exec_module",
                    }):
                raise ValueError("launcher has a dynamic code-loading attribute call")
    imports = []
    assignments = []
    sys_bytecode = 0
    audit_installs = []
    campaign_guards = []
    entries = []
    function_offsets = {}
    assignment_offsets = {}
    project_import_offsets = []
    for offset, node in enumerate(tree.body):
        if isinstance(node, ast.Expr) and offset == 0 \
                and isinstance(node.value, ast.Constant) \
                and isinstance(node.value.value, str):
            continue
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            signature = import_signature(node)
            imports.append(signature)
            module = node.module if isinstance(node, ast.ImportFrom) else node.names[0].name
            if module in PROJECT_IMPORTS:
                project_import_offsets.append(offset)
            continue
        if isinstance(node, ast.Assign) and len(node.targets) == 1 \
                and isinstance(node.targets[0], ast.Attribute) \
                and isinstance(node.targets[0].value, ast.Name) \
                and node.targets[0].value.id == "sys" \
                and node.targets[0].attr == "dont_write_bytecode" \
                and isinstance(node.value, ast.Constant) and node.value.value is True:
            sys_bytecode += 1
            continue
        if exact_hpc_home_assignment(node) and filename in DRIVERS:
            assignments.append("_HPC_HOME")
            assignment_offsets["_HPC_HOME"] = offset
            continue
        if isinstance(node, ast.Assign) and len(node.targets) == 1 \
                and isinstance(node.targets[0], ast.Name) \
                and node.targets[0].id in EXPECTED_ASSIGNMENTS[filename] \
                and pure_expression(node.value):
            assignments.append(node.targets[0].id)
            assignment_offsets[node.targets[0].id] = offset
            continue
        if isinstance(node, ast.AsyncFunctionDef):
            raise ValueError("launcher has an executable async definition")
        if isinstance(node, ast.ClassDef):
            if (filename in (WEYL_CAPTURE_DRIVER, WEYL_AFTER_DRIVER)
                    and node.name == "CommandRecordFailure"
                    and source_node_sha256(tree, node)
                    == WEYL_EXECUTION_SOURCE_SHA256["CommandRecordFailure"]):
                continue
            raise ValueError("launcher has an executable class definition")
        if isinstance(node, ast.FunctionDef):
            if not safe_function(node):
                raise ValueError("launcher function signature can execute code")
            if node.name in function_offsets:
                raise ValueError("launcher repeats a top-level function definition")
            function_offsets[node.name] = offset
            continue
        if filename in DRIVERS and exact_audit_install(node):
            audit_installs.append(offset)
            continue
        if filename in DRIVERS and exact_campaign_guard(node, filename):
            campaign_guards.append(offset)
            continue
        entry = canonical_driver_entry(node) if filename in DRIVERS else canonical_entry_guard(node)
        if entry:
            entries.append(offset)
            continue
        raise ValueError(
            "launcher has a non-allowlisted module-level action: "
            + filename + ":" + str(node.lineno) + " " + type(node).__name__)

    if (len(imports) != len(EXPECTED_IMPORTS[filename])
            or set(imports) != EXPECTED_IMPORTS[filename]):
        raise ValueError("launcher imports differ from its exact allowlist")
    expected_assignments = EXPECTED_ASSIGNMENTS[filename] \
        | ({"_HPC_HOME"} if filename in DRIVERS else set())
    if len(assignments) != len(expected_assignments) \
            or set(assignments) != expected_assignments:
        raise ValueError("launcher assignments differ from its exact allowlist")
    if entries != [len(tree.body) - 1]:
        raise ValueError("launcher lacks one final canonical entry guard")
    if filename == DRIVER and not exact_full_stager_inventory_call(tree):
        raise ValueError("full-stager inventory command changed")
    if not exact_index_contract(tree, filename):
        raise ValueError("ladder index launcher contract changed")
    if not exact_weyl_core_contract(tree, filename):
        raise ValueError("Weyl core launcher contract changed")
    if filename in DRIVERS:
        if sys_bytecode or len(audit_installs) != 1 or len(campaign_guards) != 1:
            raise ValueError("compute driver module guards changed")
        audit = audit_installs[0]
        required_before_audit = (
            assignment_offsets.get("_ACTIVE_CAMPAIGN", audit),
            assignment_offsets.get("_HPC_HOME", audit),
            assignment_offsets.get("LEGACY_PATH_OPEN_ATTEMPTS", audit),
            function_offsets.get("forbidden_legacy_path", audit),
            function_offsets.get("_path_audit", audit),
        )
        if any(offset >= audit for offset in required_before_audit) \
                or any(offset <= audit for offset in project_import_offsets) \
                or campaign_guards[0] <= max(project_import_offsets, default=-1):
            raise ValueError("compute driver audit or campaign guard order changed")
    elif sys_bytecode != 1 or audit_installs or campaign_guards:
        raise ValueError("login stager module guards changed")


def validate_snapshot(stagers, drivers, policy):
    if (set(stagers) != set(STAGED_STAGERS)
            or set(drivers) != set(DRIVERS)
            or policy not in POLICIES):
        raise ValueError("staged launcher filename or policy set changed")
    expected = POLICIES[policy]
    roles = {}
    for name, role in STAGED_STAGERS.items():
        tree = parsed(stagers[name])
        bounded_module(tree, name)
        entry = [node for node in tree.body if canonical_entry_guard(node)]
        if len(entry) != 1:
            raise ValueError("launcher lacks one canonical module entry guard")
        body = function(tree, "main").body
        if not body:
            raise ValueError("launcher main is empty")
        if role == "parent":
            roles[role] = "retired" if raises_system_exit(body[0]) else "active"
        else:
            if not disabled_guard(body[0]):
                raise ValueError("ladder stager lacks its first-statement guard")
            roles[role] = "active" if bool_assignment(
                tree, "SUBMISSION_ENABLED") else "pending"
    for name, role in DRIVERS.items():
        tree = parsed(drivers[name])
        bounded_module(tree, name)
        body = function(tree, "main").body
        if not body or not disabled_guard(body[0]):
            raise ValueError("ladder compute driver lacks its first-statement guard")
        roles[role] = "active" if bool_assignment(
            tree, "SUBMISSION_ENABLED") else "pending"
    if roles != expected:
        raise ValueError("launcher roles differ from the selected transition snapshot")
    return roles


def sources():
    return {name: (HPC / name).read_text(encoding="utf-8")
            for name in STAGED_STAGERS}


def driver_sources():
    return {name: (HPC / name).read_text(encoding="utf-8")
            for name in DRIVERS}


def _unique_json_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("retired stager bundle contains a duplicate JSON key")
        result[key] = value
    return result


def _reject_json_constant(value):
    raise ValueError("retired stager bundle contains a non-finite value: " + value)


def _canonical_bundle_bytes(value):
    return (json.dumps(
        value, ensure_ascii=True, allow_nan=False, indent=2, sort_keys=True,
    ) + "\n").encode("utf-8")


def canonical_retired_bundle(stagers):
    if not isinstance(stagers, dict) or set(stagers) != HISTORICAL_STAGERS:
        raise ValueError("retired stager bundle filename inventory changed")
    manifest = {}
    sources = {}
    total = 0
    for name in sorted(HISTORICAL_STAGERS):
        source = stagers.get(name)
        if not isinstance(source, str):
            raise ValueError("retired stager source must be UTF-8 text: " + name)
        try:
            raw = source.encode("utf-8", errors="strict")
        except UnicodeEncodeError as error:
            raise ValueError("retired stager source is not strict UTF-8: " + name) from error
        if len(raw) > MAX_RETIRED_SOURCE_BYTES:
            raise ValueError("retired stager source exceeds its size limit: " + name)
        total += len(raw)
        manifest[name] = hashlib.sha256(raw).hexdigest()
        sources[name] = source
    if total > MAX_RETIRED_BUNDLE_BYTES:
        raise ValueError("retired stager sources exceed their aggregate size limit")
    raw = _canonical_bundle_bytes({
        "encoding": RETIRED_BUNDLE_ENCODING,
        "manifest": manifest,
        "schema": RETIRED_BUNDLE_SCHEMA,
        "sources": sources,
    })
    if len(raw) > MAX_RETIRED_BUNDLE_BYTES:
        raise ValueError("canonical retired stager bundle exceeds its size limit")
    return raw


def retired_sources_from_bundle(raw):
    if not isinstance(raw, bytes) or len(raw) > MAX_RETIRED_BUNDLE_BYTES:
        raise ValueError("retired stager bundle must be bounded bytes")
    try:
        text = raw.decode("utf-8", errors="strict")
        value = json.loads(
            text,
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_json_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("retired stager bundle is not strict canonical JSON") from error
    if (not isinstance(value, dict)
            or set(value) != {"encoding", "manifest", "schema", "sources"}
            or value.get("schema") != RETIRED_BUNDLE_SCHEMA
            or value.get("encoding") != RETIRED_BUNDLE_ENCODING
            or not isinstance(value.get("manifest"), dict)
            or not isinstance(value.get("sources"), dict)
            or set(value["manifest"]) != HISTORICAL_STAGERS
            or set(value["sources"]) != HISTORICAL_STAGERS
            or _canonical_bundle_bytes(value) != raw):
        raise ValueError("retired stager bundle shape or canonical bytes changed")
    total = 0
    result = {}
    for name in sorted(HISTORICAL_STAGERS):
        source = value["sources"].get(name)
        wanted = value["manifest"].get(name)
        if (not isinstance(source, str)
                or not isinstance(wanted, str) or len(wanted) != 64
                or any(character not in "0123456789abcdef" for character in wanted)):
            raise ValueError("retired stager bundle entry is invalid: " + name)
        try:
            source_raw = source.encode("utf-8", errors="strict")
        except UnicodeEncodeError as error:
            raise ValueError("retired stager source is not strict UTF-8: " + name) from error
        if (len(source_raw) > MAX_RETIRED_SOURCE_BYTES
                or hashlib.sha256(source_raw).hexdigest() != wanted):
            raise ValueError("retired stager source hash or size changed: " + name)
        total += len(source_raw)
        result[name] = source
    if total > MAX_RETIRED_BUNDLE_BYTES:
        raise ValueError("retired stager sources exceed their aggregate size limit")
    return result


def reconstructed_stage_sources(raw):
    staged = retired_sources_from_bundle(raw)
    staged.update(sources())
    return staged


def bounded_historical_module(tree):
    """Reject executable module actions before the retired main guard."""
    for offset, node in enumerate(tree.body):
        if (offset == 0 and isinstance(node, ast.Expr)
                and isinstance(node.value, ast.Constant)
                and isinstance(node.value.value, str)):
            continue
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            import_signature(node)
            continue
        if (isinstance(node, ast.Assign) and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Attribute)
                and isinstance(node.targets[0].value, ast.Name)
                and node.targets[0].value.id == "sys"
                and node.targets[0].attr == "dont_write_bytecode"
                and isinstance(node.value, ast.Constant)
                and node.value.value is True):
            continue
        if (isinstance(node, ast.Assign) and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name)
                and pure_expression(node.value)):
            continue
        if isinstance(node, ast.FunctionDef) and safe_function(node):
            continue
        if offset == len(tree.body) - 1 and canonical_entry_guard(node):
            continue
        raise ValueError("historical stager has executable module-level behavior")


def validate_historical_inventory(stagers):
    if not isinstance(stagers, dict) or set(stagers) != FROZEN_STAGE_FILENAMES:
        raise ValueError("full stager filename inventory changed")
    for name in sorted(HISTORICAL_STAGERS):
        tree = parsed(stagers[name])
        bounded_historical_module(tree)
        main = function(tree, "main")
        arguments = main.args
        if (not isinstance(main, ast.FunctionDef)
                or main.decorator_list or main.returns is not None
                or arguments.posonlyargs or arguments.args
                or arguments.vararg is not None or arguments.kwonlyargs
                or arguments.kwarg is not None
                or not main.body or not raises_system_exit(main.body[0])
                or not tree.body or not canonical_entry_guard(tree.body[-1])
                or sum(canonical_entry_guard(node) for node in tree.body) != 1):
            raise ValueError("historical stager is not immediately retired: " + name)
    return set(HISTORICAL_STAGERS)


class StagerAllowlist(unittest.TestCase):
    def test_current_filename_and_role_snapshot(self):
        actual_paths = tuple(HPC.glob("stage_*.py"))
        self.assertTrue(all(path.is_file() and not path.is_symlink()
                            for path in actual_paths))
        actual = {path.name for path in actual_paths}
        if actual == FROZEN_STAGE_FILENAMES:
            historical = {
                name: (HPC / name).read_text(encoding="utf-8")
                for name in HISTORICAL_STAGERS
            }
            raw = canonical_retired_bundle(historical)
            self.assertEqual(len(raw), RETIRED_BUNDLE_BYTES)
            self.assertEqual(hashlib.sha256(raw).hexdigest(), RETIRED_BUNDLE_SHA256)
            self.assertEqual(
                validate_historical_inventory({**historical, **sources()}),
                HISTORICAL_STAGERS,
            )
        else:
            self.assertEqual(actual, set(STAGED_STAGERS))
        expected_bundle = {
            "schema": "atlas-campaign-blob-v1",
            "role": "retired-stager-bundle",
            "sha256": RETIRED_BUNDLE_SHA256,
            "bytes": RETIRED_BUNDLE_BYTES,
        }
        for name in (
                "stage_ladder_boundary_after.py", DRIVER,
                "stage_weyl_context_core_capture.py",
                "stage_weyl_context_core_after.py"):
            tree = parsed((HPC / name).read_text(encoding="utf-8"))
            self.assertEqual(
                literal_assignment(tree, "RETIRED_STAGER_BUNDLE_REFERENCE"),
                expected_bundle,
            )
        self.assertEqual(
            validate_snapshot(sources(), driver_sources(), CURRENT_POLICY),
            POLICIES[CURRENT_POLICY],
        )

    def test_ladder_stagers_and_drivers_have_independent_feature_guards(self):
        expected = {
            "stage_ladder_boundary_before.py": False,
            "stage_ladder_boundary_after.py": False,
            "stage_ladder_boundary_index.py": False,
            "stage_weyl_context_core_capture.py": False,
            "stage_weyl_context_core_after.py": True,
            HISTORICAL_DRIVER: False,
            AFTER_DRIVER: False,
            INDEX_DRIVER: False,
            WEYL_CAPTURE_DRIVER: False,
            WEYL_AFTER_DRIVER: True,
        }
        for name, enabled in expected.items():
            tree = parsed((HPC / name).read_text(encoding="utf-8"))
            self.assertIs(bool_assignment(tree, "SUBMISSION_ENABLED"), enabled)
            self.assertTrue(disabled_guard(function(tree, "main").body[0]))

    def test_current_frozen_launchers_cannot_fake_parent_only_snapshot(self):
        with self.assertRaises(ValueError):
            validate_snapshot(sources(), driver_sources(), "parent-only")
        staged = sources()
        drivers = driver_sources()
        parent = staged["stage_weyl_parent_seal.py"]
        closed = "def main():\n    raise SystemExit(\"parent seal is closed\")\n"
        self.assertEqual(parent.count(closed), 1)
        staged["stage_weyl_parent_seal.py"] = parent.replace(
            closed, "def main():\n", 1)
        after = staged["stage_weyl_context_core_after.py"]
        staged["stage_weyl_context_core_after.py"] = after.replace(
            "SUBMISSION_ENABLED = True", "SUBMISSION_ENABLED = False", 1)
        active_driver = drivers[WEYL_AFTER_DRIVER]
        drivers[WEYL_AFTER_DRIVER] = active_driver.replace(
            "SUBMISSION_ENABLED = True", "SUBMISSION_ENABLED = False", 1)
        with self.assertRaises(ValueError):
            validate_snapshot(staged, drivers, "parent-only")

    def test_module_actions_and_canonical_entries_are_bounded(self):
        historical = driver_sources()[HISTORICAL_DRIVER]
        frozen_message = "ladder BEFORE v2 campaign policy changed"
        self.assertEqual(historical.count(frozen_message), 1)
        changed = parsed(historical.replace(
            frozen_message, "ladder BEFORE v3 campaign policy changed", 1))
        with self.assertRaisesRegex(
                ValueError,
                r"math_ladder_boundary_before\.py:55 If$"):
            bounded_module(changed, HISTORICAL_DRIVER)
        for name, source in {**sources(), **driver_sources()}.items():
            tree = parsed(source)
            bounded_module(tree, name)
            entry = canonical_driver_entry if name in DRIVERS else canonical_entry_guard
            self.assertEqual(sum(entry(node) for node in tree.body), 1, name)

    def test_role_guard_and_side_effect_mutations_are_rejected(self):
        staged = sources()
        drivers = driver_sources()
        mutations = [
            ({**staged, "stage_extra.py": "def main():\n pass\n"}, drivers),
            ({**staged, "stage_ladder_boundary_after.py": staged[
                "stage_ladder_boundary_after.py"].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}, drivers),
            ({**staged, "stage_ladder_boundary_index.py": staged[
                "stage_ladder_boundary_index.py"].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}, drivers),
            ({**staged, "stage_weyl_context_core_capture.py": staged[
                "stage_weyl_context_core_capture.py"].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}, drivers),
            ({**staged, "stage_weyl_context_core_after.py": staged[
                "stage_weyl_context_core_after.py"].replace(
                    "SUBMISSION_ENABLED = True", "SUBMISSION_ENABLED = False", 1)}, drivers),
            ({**staged, "stage_weyl_context_core_capture.py": staged[
                "stage_weyl_context_core_capture.py"].replace(
                    "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09",
                    "0" * 64, 1)}, drivers),
            ({**staged, "stage_weyl_context_core_capture.py": staged[
                "stage_weyl_context_core_capture.py"].replace(
                    "    verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)\n",
                    "", 1)}, drivers),
            ({**staged, "stage_ladder_boundary_before.py": staged[
                "stage_ladder_boundary_before.py"].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}, drivers),
            (staged, {**drivers, HISTORICAL_DRIVER: drivers[
                HISTORICAL_DRIVER].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}),
            (staged, {**drivers, AFTER_DRIVER: drivers[AFTER_DRIVER].replace(
                "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "SUBMISSION_ENABLED = False", "SUBMISSION_ENABLED = True", 1)}),
            (staged, {**drivers, WEYL_AFTER_DRIVER: drivers[
                WEYL_AFTER_DRIVER].replace(
                    "SUBMISSION_ENABLED = True", "SUBMISSION_ENABLED = False", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '"workspace/source": work / "source"',
                    '"workspace/source": root', 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "pass_fds=(descriptor,)", "pass_fds=()", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "active, pass_fds=()):", "active, pass_fds=(0,)):", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "            cwd=cwd,\n", "            cwd=Path('/'),\n", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "        argv, cwd, env, b\"\", COMMAND_TIMEOUT_SECONDS,",
                    "        argv, root, env, b\"\", COMMAND_TIMEOUT_SECONDS,",
                    1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "            wrapper, scripts, env, input_bytes,",
                    "            ['/bin/true'], scripts, env, input_bytes,", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '        raise ValueError("invalid timed-command request")\n'
                    "    started =",
                    '        raise ValueError("invalid timed-command request")\n'
                    '    argv[0] = "/bin/true"\n'
                    "    started =", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '        raise ValueError("invalid timed-command request")\n'
                    "    started =",
                    '        raise ValueError("invalid timed-command request")\n'
                    "    env.clear()\n"
                    "    started =", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '        raise ValueError("invalid timed-command request")\n'
                    "    started =",
                    '        raise ValueError("invalid timed-command request")\n'
                    "    alias = argv\n"
                    '    alias[0] = "/bin/true"\n'
                    "    started =", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '        raise ValueError("invalid timed-command request")\n'
                    "    started =",
                    '        raise ValueError("invalid timed-command request")\n'
                    '    list.__setitem__(argv, 0, "/bin/true")\n'
                    "    started =", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "    descriptor = None\n"
                    "    try:\n"
                    "        descriptor, before_identity, before_fingerprint "
                    "= _open_executable(binary)",
                    "    scripts = Path('/')\n"
                    "    descriptor = None\n"
                    "    try:\n"
                    "        descriptor, before_identity, before_fingerprint "
                    "= _open_executable(binary)", 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    '"rustc-version": {"argv": ["rustc", "-vV"], "cwd": "."}',
                    '"rustc-version": {"argv": ["rustc", "--version"], '
                    '"cwd": "."}', 1)}),
            (staged, {**drivers, WEYL_CAPTURE_DRIVER: drivers[
                WEYL_CAPTURE_DRIVER].replace(
                    "    capture_incomplete = False\n\n"
                    "    def interrupted",
                    "    capture_incomplete = False\n"
                    "    COMMAND_CONTRACTS['release-build']['cwd'] = '.'\n\n"
                    "    def interrupted", 1)}),
            (staged, {**drivers, DRIVER: drivers[DRIVER] +
                      "\nopen('/tmp/not-allowed', 'w')\n"}),
            (staged, {**drivers, DRIVER: drivers[DRIVER].replace(
                "import fcntl\n", "import fcntl\nimport webbrowser\n", 1)}),
            ({**staged, "stage_ladder_boundary_after.py": staged[
                "stage_ladder_boundary_after.py"].replace(
                    "def main():\n", "def main():\n    import webbrowser\n", 1)}, drivers),
            (staged, {**drivers, DRIVER: drivers[DRIVER].replace(
                "def main():\n", "def main():\n    __import__('webbrowser')\n", 1)}),
            (staged, {**drivers, DRIVER: drivers[DRIVER].replace(
                "def main():\n",
                "def main():\n    __builtins__['__import__']('webbrowser')\n",
                1)}),
            (staged, {**drivers, DRIVER: drivers[DRIVER].replace(
                "                FULL_STAGER_INVENTORY_PROGRAM,\n",
                "                'print(0)',\n",
                1)}),
            ({**staged, "stage_ladder_boundary_index.py": staged[
                "stage_ladder_boundary_index.py"].replace(
                    "import hashlib\n", "import hashlib\nimport webbrowser\n", 1)}, drivers),
            ({**staged, "stage_ladder_boundary_index.py": staged[
                "stage_ladder_boundary_index.py"].replace(
                    '"test-stager-allowlist": 7',
                    '"test-stager-allowlist": 8', 1)}, drivers),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                "import hashlib\n", "import hashlib\nimport webbrowser\n", 1)}),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                '"test-stager-allowlist",',
                '"test-stager-allowlist-changed",', 1)}),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                'sys.executable, "-I", "-S", "-B"',
                'sys.executable, "-B"', 1)}),
            ({**staged, "stage_ladder_boundary_index.py": staged[
                "stage_ladder_boundary_index.py"].replace(
                    "    verify_blob(campaign, "
                    "RETIRED_STAGER_BUNDLE_REFERENCE)\n", "", 1)}, drivers),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                "expected_job=job)", "expected_job=None)", 1)}),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                " = publish_final_report(", " = run_command(", 1)}),
            (staged, {**drivers, INDEX_DRIVER: drivers[INDEX_DRIVER].replace(
                "    verify_blob(campaign, "
                "RETIRED_STAGER_BUNDLE_REFERENCE)\n", "", 1)}),
        ]
        for changed_stagers, changed_drivers in mutations:
            with self.assertRaises(ValueError):
                validate_snapshot(changed_stagers, changed_drivers, CURRENT_POLICY)

        retired = (
            "def main():\n"
            "    raise SystemExit('historical stager closed')\n\n"
            "if __name__ == '__main__':\n"
            "    main()\n"
        )
        frozen = {name: retired for name in FROZEN_STAGE_FILENAMES}
        for name in STAGED_STAGERS:
            frozen[name] = ""
        self.assertEqual(
            validate_historical_inventory(frozen), HISTORICAL_STAGERS)
        added = dict(frozen)
        added["stage_unreviewed.py"] = retired
        removed = dict(frozen)
        removed.pop("stage_cartan_before.py")
        reactivated = dict(frozen)
        reactivated["stage_cartan_before.py"] = retired.replace(
            "    raise SystemExit(", "    return\n    raise SystemExit(", 1)
        duplicate_main = dict(frozen)
        duplicate_main["stage_cartan_before.py"] += (
            "\ndef main():\n    raise SystemExit('duplicate')\n")
        side_effect = dict(frozen)
        side_effect["stage_cartan_before.py"] = retired.replace(
            "\nif __name__", "\nopen('/tmp/not-allowed', 'w')\n\nif __name__", 1)
        for changed in (added, removed, reactivated, duplicate_main, side_effect):
            with self.assertRaises(ValueError):
                validate_historical_inventory(changed)

    def test_canonical_retired_bundle_reconstructs_exact_inventory(self):
        retired = (
            "def main():\n"
            "    raise SystemExit('historical stager closed')\n\n"
            "if __name__ == '__main__':\n"
            "    main()\n"
        )
        historical = {name: retired for name in HISTORICAL_STAGERS}
        raw = canonical_retired_bundle(historical)
        self.assertEqual(retired_sources_from_bundle(raw), historical)
        reconstructed = reconstructed_stage_sources(raw)
        self.assertEqual(set(reconstructed), FROZEN_STAGE_FILENAMES)
        self.assertEqual(
            validate_historical_inventory(reconstructed), HISTORICAL_STAGERS)

    def test_retired_bundle_rejects_noncanonical_changed_or_active_payloads(self):
        retired = (
            "def main():\n"
            "    raise SystemExit('historical stager closed')\n\n"
            "if __name__ == '__main__':\n"
            "    main()\n"
        )
        historical = {name: retired for name in HISTORICAL_STAGERS}
        raw = canonical_retired_bundle(historical)

        def encode(value):
            return (json.dumps(
                value, ensure_ascii=True, allow_nan=False, indent=2,
                sort_keys=True) + "\n").encode("utf-8")

        value = json.loads(raw)
        mutations = []
        mutations.append(raw.replace(b"{\n", b"{ \n", 1))
        mutations.append(b"\xff" + raw)
        mutations.append(raw.replace(
            b'"schema": "atlas-retired-stager-bundle-v1"',
            b'"schema": "duplicate",\n  "schema": '
            b'"atlas-retired-stager-bundle-v1"', 1))
        changed = dict(value, schema="atlas-retired-stager-bundle-v0")
        mutations.append(encode(changed))
        changed = dict(value, encoding="latin-1")
        mutations.append(encode(changed))
        changed = json.loads(raw)
        changed["sources"].pop(next(iter(sorted(HISTORICAL_STAGERS))))
        mutations.append(encode(changed))
        changed = json.loads(raw)
        changed["sources"]["../stage_escape.py"] = retired
        changed["manifest"]["../stage_escape.py"] = hashlib.sha256(
            retired.encode("utf-8")).hexdigest()
        mutations.append(encode(changed))
        changed = json.loads(raw)
        name = next(iter(sorted(HISTORICAL_STAGERS)))
        changed["sources"][name] += "# changed\n"
        mutations.append(encode(changed))
        for payload in mutations:
            with self.assertRaises(ValueError):
                retired_sources_from_bundle(payload)

        name = next(iter(sorted(HISTORICAL_STAGERS)))
        for source in (
                retired.replace("    raise SystemExit(",
                                "    return\n    raise SystemExit(", 1),
                retired + "\ndef main():\n    raise SystemExit('duplicate')\n"):
            changed = dict(historical, **{name: source})
            reconstructed = reconstructed_stage_sources(
                canonical_retired_bundle(changed))
            with self.assertRaises(ValueError):
                validate_historical_inventory(reconstructed)


if __name__ == "__main__":
    unittest.main()

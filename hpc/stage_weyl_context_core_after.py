"""Prepare one bounded, tests-first Weyl-context core AFTER gate.

The stage is a direct child of the one active campaign.  It reruns the same
frozen gate on the repaired source: the two known regressions must now pass,
with the repaired Rust matching every frozen original golden.  Immutable
harness bytes are installed once, one exact pin is submitted through the
shared campaign ledger, and a repeated invocation can only validate or
recover that receipt.
"""
from contextlib import contextmanager
import ast
import copy
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import secrets
import stat
import sys


sys.dont_write_bytecode = True

from campaign_blob import read_blob, verify_blob
from campaign_source import file_manifest, verify_source_archive
from campaign_workspace import submission_scope
import weyl_context_core_regression as regression_contract
from progressive_submit import (
    STAGE_CREATION_CONTRACT_SCHEMA, STAGE_CREATION_MARKER,
    STAGE_CREATION_RECEIPT, create_fixed_stage, exclusive_lock,
    read_json_file, save, submit_one, validate_confirmed_history,
    validate_stage_creation,
)


STAGE_NAME = "weyl-context-core-after-v5"
PIN_NAME = "weyl-context-core-after-v5-pin.json"
PIN_SCHEMA = "atlas-weyl-context-core-after-pin-v5"
STAGE_LOCK = ".weyl-context-core-after-v5-stage.lock"
SBATCH = "hpc/math_weyl_context_core_after.sbatch"

# The failed predecessor BEFORE stage is immutable.  This changed-input
# successor remains disabled until its driver, progressive creator, allowlist
# and all static tests are frozen together.  The guard in main() precedes argument parsing
# and every filesystem operation.
SUBMISSION_ENABLED = True

EXPECTED_TEST_COUNTS = {
    "test-campaign-stage-creation": 32,
    "test-progressive-submit": 17,
    "test-weyl-context-core-contract": 20,
    "test-weyl-context-core-regression-contract": 21,
    "test-math-weyl-context-core-after": 30,
    "test-stager-allowlist": 7,
}
CHECKER_TESTS = 127

# The before-v1/v2/v3 failure evidences froze this predicted after-gate
# checker table, and the before-v4 report recorded checker total 119.  The
# actual after gate renames the capture checker command to
# test-math-weyl-context-core-after and adds the four classify_after contract
# tests (regression-contract 17 -> 21, total 119 -> 123).  Preserve the
# historical predictions exactly as recorded; never relabel them to match
# the successor.
BEFORE_FAILURE_PREDICTED_AFTER_COUNTS = {
    "test-campaign-stage-creation": 32,
    "test-progressive-submit": 17,
    "test-weyl-context-core-contract": 18,
    "test-weyl-context-core-regression-contract": 17,
    "test-math-weyl-context-core-capture": 28,
    "test-stager-allowlist": 7,
    "total": 119,
}
BEFORE_V4_CHECKER_TOTAL = 119

CATALOG_PATH = "tests/math/generics/weyl_context_core_catalog.json"
CATALOG_SHA256 = (
    "7fb77fecda841962cb98bdeddbdba58df46d673e076f87f2087cd234e2f1a717"
)
CATALOG_BYTES = 1283
CATALOG_CASES = [
    {
        "id": "weyl_context_core_cold_dual",
        "file": "weyl_context_core_cold_dual.atlas",
        "fixture_sha256":
            "4eff8fa08f8490e242f07282125b83cde1daf24a875fe8df4dc7e73e75a0ae99",
        "intent": "accept",
        "timeout_seconds": 45,
        "input_sha256":
            "d94ae61b72215cda32b4ef040221dd81d0dd127866bc334c2ca7f43a23b820ba",
        "input_bytes": 1916,
        "order": ["oracle", "rust"],
    },
    {
        "id": "weyl_context_core_prewarmed_dual",
        "file": "weyl_context_core_prewarmed_dual.atlas",
        "fixture_sha256":
            "f13d704175f702b966790bf82e56c837dd80a594f0dc2dcd0d6b81ba281799a2",
        "intent": "reject",
        "timeout_seconds": 45,
        "input_sha256":
            "3f3bb95cf514aee419678543e135e405285b1d6db10d63eb590e27a94880ee4f",
        "input_bytes": 1386,
        "order": ["rust", "oracle"],
    },
]

REGRESSION_CATALOG_PATH = (
    "tests/math/generics/weyl_context_core_regression_catalog.json"
)
REGRESSION_CATALOG_SHA256 = (
    "6a9f960753bcd2afb575cb5207d4310f21a5694505d02a83f4e4142211eaa634"
)
REGRESSION_CATALOG_BYTES = 5208
REGRESSION_INSPECTION_PATH = (
    "tests/reference/hpc/"
    "math_weyl_context_core_capture_v8_inspection_2026_10_02.json"
)
REGRESSION_INSPECTION_SHA256 = (
    "b2c7f4ece709df3506c3224a57abad896f3ecfcd6625fea97731619f629e1da3"
)
REGRESSION_INSPECTION_BYTES = 11902
V8_SUBMISSION_EVIDENCE_PATH = (
    "tests/reference/hpc/"
    "math_weyl_context_core_capture_v8_submission_2026_10_02.json"
)
V8_SUBMISSION_EVIDENCE_SHA256 = (
    "2ef561f986277f2645f8929b0b95acf30eb1f03accdd3361e89803220dca79fa"
)
V8_SUBMISSION_EVIDENCE_BYTES = 4593

AFTER_REPORT_PATH = (
    "tests/reference/hpc/math_ladder_boundary_after_v3_report_2026_10_01.json"
)
AFTER_INSPECTION_PATH = (
    "tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json"
)
AFTER_REVIEW_PATH = (
    "tests/reference/hpc/"
    "math_ladder_boundary_after_v3_index_review_2026_10_01.json"
)
ACCEPTANCE_INDEX_PATH = (
    "tests/reference/hpc/math_acceptance_index_2026_10_01.json"
)
INDEX_REPORT_PATH = (
    "tests/reference/hpc/math_ladder_boundary_index_v1_report_2026_10_01.json"
)
INDEX_INSPECTION_PATH = (
    "tests/reference/hpc/math_ladder_boundary_index_v1_2026_10_01.json"
)

ACCEPTED_INDEX_PREDECESSOR = {
    "stage": (
        "/public/home/majj/atlas-rust-campaign-20260930/"
        "stages/ladder-boundary-index-v1"
    ),
    "job": "3879103",
    "pin_sha256":
        "0b2d966d8208c36e2be28ef48920de9319fe64848a5e1c10ddbadacdf488fd3c",
    "receipt_sha256":
        "8a68007be0c54e1dd69dd3d599b0ceebefe4659dd1ae29d986d4fcc288821072",
    "report": {
        "file": INDEX_REPORT_PATH,
        "sha256":
            "43cc79e1630b5ec21a24c5aa1a7c411eafffaa9a4c6911333c8752b0dc63cf9d",
    },
    "inspection": {
        "file": INDEX_INSPECTION_PATH,
        "sha256":
            "d9448e5cd53ba4c25e6a0fd91a5fdb16e3b423d48fe58531a50d7d7457afb9ad",
    },
    "campaign_ledger_sha256":
        "b336fb831de28f72e9138284d2c6f07ecfb178f617ebe4bec78e1c82ffb8eaa7",
    "acceptance_index_sha256":
        "801e79c8679dfd4a50ff6736b439fe097c0e0d25e5db6387edc2dfd3316a720f",
    "acceptance_index_head_sha256":
        "1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12",
}
V1_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v1"
)

V1_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v1_failure_2026_10_02.json"
    ),
    "sha256":
        "3ff6e579327536eb6d5686750946d668664f3adbb7e33558b7a3d4a7f528aa9f",
}
V1_PREDECESSOR_RECORD = {
    "stage": V1_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884124",
    "pin_sha256":
        "642f3af98698e3f6bc897b00b732753840a324d6906dc9f49fe68a6072b640c4",
    "stage_creation_sha256":
        "7aa595f1acdff10a0ce8318eced5d0e8c4658fe7b130c6fec5ae8bd9f64ce3e6",
}
V1_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v2",
    "stage": V1_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655296334,
    "stage_tree_sha256":
        "f85df75860e3da76871ba9d9972485ef3ea7f43cc2663b71b28d380c0be47940",
    "stage_tree_files": 80,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2505878,
    "record": V1_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "e0e70a81a2155641a5dc7695c9d27beb171545f7c253adfc02791a85a8e49dba",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "7aa595f1acdff10a0ce8318eced5d0e8c4658fe7b130c6fec5ae8bd9f64ce3e6",
            "bytes": 6137,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v1-pin.json": {
            "sha256":
                "642f3af98698e3f6bc897b00b732753840a324d6906dc9f49fe68a6072b640c4",
            "bytes": 10447,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "8c7b5022cccdeb492db2c83c2f98755500135e4e8b563b3a9b38168a06294b50",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "1a67d465e41e2048344be29fd0af4b46a919b6ed4e779c87e0c6fa400d1b7b64",
            "bytes": 6979,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/report.json": {
            "sha256":
                "52a647ca524621e117e7848a43f2ab370569de73e5c20a7a9b9fafc0390ee2f7",
            "bytes": 29375,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/report.sha256": {
            "sha256":
                "87f4a0fee0b34326fa37c00ce58cd05b3745a346ef7254c8545c9c7e51a001b7",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/test-campaign-stage-creation.stderr": {
            "sha256":
                "429f6850507231f37a7c95b50b208ff10b4804c9be8b2280d3c11b3dedced9be",
            "bytes": 7272,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884124/test-campaign-stage-creation.time": {
            "sha256":
                "b8fd7e4469410ca9e83d49e62ead2c705d162784be131efc8690bdcb470f50ea",
            "bytes": 896,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v1-3884124.out": {
            "sha256":
                "5c138088511be2aba506212c281e97ff1e8dbaf5127a24419e80899f55f9859f",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V1_PREDECESSOR = {
    "stage": V1_PREDECESSOR_STAGE,
    "job": "3884124",
    "pin_sha256":
        "642f3af98698e3f6bc897b00b732753840a324d6906dc9f49fe68a6072b640c4",
    "stage_creation_sha256":
        "7aa595f1acdff10a0ce8318eced5d0e8c4658fe7b130c6fec5ae8bd9f64ce3e6",
    "submission_receipt_sha256":
        "1a67d465e41e2048344be29fd0af4b46a919b6ed4e779c87e0c6fa400d1b7b64",
    "report_sha256":
        "52a647ca524621e117e7848a43f2ab370569de73e5c20a7a9b9fafc0390ee2f7",
    "failure_evidence": V1_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "d59633a873f334faad00e9b030a3a8cc0fdb73e4e6561c019d05451e4873ff52",
    "campaign_ledger_records": 8,
}

V2_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v2"
)
V2_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v2_failure_2026_10_02.json"
    ),
    "sha256":
        "5ec936c2cbae28778df9ab4a1d4076008e4fcc6a54b95a7e6b9f47b550820fab",
}
V2_PREDECESSOR_RECORD = {
    "stage": V2_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884371",
    "pin_sha256":
        "44f3133abd56de4b749f2c97aa658e2d18cbecd1b694dfba70b4a16ff36b87dc",
    "stage_creation_sha256":
        "fa3539d603b273ddf549fa05acd9e6ca3306a54143ce89362cfe69c1e869322e",
}
V2_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v3",
    "stage": V2_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655298694,
    "stage_tree_sha256":
        "43a8347b61d23d275eecb92eec1a0ca2ff3689e33117a7000b28c4eb3df1bd09",
    "stage_tree_files": 91,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2635472,
    "record": V2_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "394cfd2b9aba3f733a937c8cd6feae32a49d84bd61f607fa474e482352af731c",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "fa3539d603b273ddf549fa05acd9e6ca3306a54143ce89362cfe69c1e869322e",
            "bytes": 10853,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v2-pin.json": {
            "sha256":
                "44f3133abd56de4b749f2c97aa658e2d18cbecd1b694dfba70b4a16ff36b87dc",
            "bytes": 10424,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "381c59368f858396e91d64f98040fec197a9c2a68bef5f0d57095462489dee36",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "9f325da651dacba89942dd96578f09eb28f725c9a0ab9428ccb0e7ff9d6cf617",
            "bytes": 6803,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/report.json": {
            "sha256":
                "8ce92ed8ef844472e84b0c369b9c68b2a94c2b1fa0ae0b2fdec5f4cfd8a5782a",
            "bytes": 32873,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/report.sha256": {
            "sha256":
                "35403f46a0953d3e62c061ac9bf27aedf82f813a6574d7e2772968946b959cbe",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-campaign-stage-creation.stderr": {
            "sha256":
                "36f3db49e1b721b31aa9869452d0e8898e6fcdd2051dacc3f91998f5f06d0168",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-campaign-stage-creation.time": {
            "sha256":
                "ada80a44f11486bfda043d300baa787f5e7400fe33bf989bccf549a51b3d5314",
            "bytes": 860,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-progressive-submit.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-progressive-submit.stderr": {
            "sha256":
                "5bdd5f5fb48917e3cfa67f56836c4b3768aa892bd641e695e44f4a2191f387c2",
            "bytes": 1704,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-progressive-submit.time": {
            "sha256":
                "52769e9f0b6322d408cd451052d027e0b633e5e19bf4e39813498ddb89cd3ee1",
            "bytes": 846,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-weyl-context-core-contract.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-weyl-context-core-contract.stderr": {
            "sha256":
                "9faace8f7eae2df8d428884fcbf85ae240ceaf8cc8640c7e6ceb94e5fee008cd",
            "bytes": 2300,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-weyl-context-core-contract.time": {
            "sha256":
                "eac8f21a2c25555d37bf8285b3419cedd79f6ff948dfb0048082e070210ce96b",
            "bytes": 851,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-math-weyl-context-core-capture.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-math-weyl-context-core-capture.stderr": {
            "sha256":
                "60e910f4eece138732e13cc3a5847da588193c2baacdcf33631b994999baf83b",
            "bytes": 6260,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884371/test-math-weyl-context-core-capture.time": {
            "sha256":
                "733f23cb232ac7e55d3be38e9dbd397ed625927739eb5b917014009b385fc6a8",
            "bytes": 902,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v2-3884371.out": {
            "sha256":
                "b514fea6761fb9a67cd291d01f465da9110f2cbe9d9f912483089cae26b810ca",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V2_PREDECESSOR = {
    "stage": V2_PREDECESSOR_STAGE,
    "job": "3884371",
    "pin_sha256":
        "44f3133abd56de4b749f2c97aa658e2d18cbecd1b694dfba70b4a16ff36b87dc",
    "stage_creation_sha256":
        "fa3539d603b273ddf549fa05acd9e6ca3306a54143ce89362cfe69c1e869322e",
    "stage_creation_contract_sha256":
        "f77e7e1425b3b3af5c75b40f021c17751a583c3a60d8b2b9258db0836d03acb1",
    "submission_intent_sha256":
        "381c59368f858396e91d64f98040fec197a9c2a68bef5f0d57095462489dee36",
    "submission_receipt_sha256":
        "9f325da651dacba89942dd96578f09eb28f725c9a0ab9428ccb0e7ff9d6cf617",
    "report_sha256":
        "8ce92ed8ef844472e84b0c369b9c68b2a94c2b1fa0ae0b2fdec5f4cfd8a5782a",
    "failure_evidence": V2_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "3f914057b51234d4be97774a6d2a6360d61eb2e77bea3004335f1e947632c938",
    "campaign_ledger_records": 9,
}

V3_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v3"
)
V3_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v3_failure_2026_10_02.json"
    ),
    "sha256":
        "434f1502dd65038a96b036a2c6eddbe541d84e01a1e7abb446ee86063e038b54",
}
V3_PREDECESSOR_RECORD = {
    "stage": V3_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884456",
    "pin_sha256":
        "a4f4d9e924a201c29d25f73649b1eaa40843efddbb4b21ab78fa0d8bc0871c06",
    "stage_creation_sha256":
        "742ff713a6f865c163f1846c62e0647892ec5d556e9cd1358cbdfad45c87958d",
}
V3_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v4",
    "stage": V3_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655299664,
    "stage_tree_sha256":
        "e3d95aa3b64b10513ccd56a246b8980363d11fd399dceb28cb3859bbc6d25b19",
    "stage_tree_files": 84,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2683734,
    "record": V3_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "prepared.json"
        ): {
            "sha256":
                "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
            "bytes": 13799,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "sealed.json"
        ): {
            "sha256":
                "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "published.json"
        ): {
            "sha256":
                "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "5c3c4481098177e9b138c45e2cde109bc5b20a5695aa592a6b7bdd155982dab4",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "742ff713a6f865c163f1846c62e0647892ec5d556e9cd1358cbdfad45c87958d",
            "bytes": 14038,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v3-pin.json": {
            "sha256":
                "a4f4d9e924a201c29d25f73649b1eaa40843efddbb4b21ab78fa0d8bc0871c06",
            "bytes": 10944,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "973b0944273bb0741bec73408ae2c25bd2686a34bfab1424741bb5e9c892e932",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "2f547384f7ec2632cd604e0b68d9aa8c4673ab8f83010c319476078da7c940de",
            "bytes": 7170,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/report.json": {
            "sha256":
                "64cb617a81dbea65cbc14c3d62e9988921a56a81408a561a86586ef4b61bf520",
            "bytes": 30377,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/report.sha256": {
            "sha256":
                "8c8cbe4ff69502d6d1609145650a464b9e979eadd3bcab31956589309bf599a8",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/test-campaign-stage-creation.stderr": {
            "sha256":
                "adee98007af0a745b0acd8b897231cbdee66e937dff4c57a0a33fea7a837a03d",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884456/test-campaign-stage-creation.time": {
            "sha256":
                "624f8ea4714c371560f51da4dede4ec55afdd05e6cf05c4c6c433ec25acad4b2",
            "bytes": 860,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v3-3884456.out": {
            "sha256":
                "471e0e9fbef6f6b5da35862160e1c6927a99d4dfd4d48a4d9d340d37912638b3",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V3_PREDECESSOR = {
    "stage": V3_PREDECESSOR_STAGE,
    "job": "3884456",
    "pin_sha256":
        "a4f4d9e924a201c29d25f73649b1eaa40843efddbb4b21ab78fa0d8bc0871c06",
    "stage_creation_sha256":
        "742ff713a6f865c163f1846c62e0647892ec5d556e9cd1358cbdfad45c87958d",
    "stage_creation_contract_sha256":
        "782da7645710f555ecd7c1a560f915929aebdb46981028d13dd97f153555ecbd",
    "submission_intent_sha256":
        "973b0944273bb0741bec73408ae2c25bd2686a34bfab1424741bb5e9c892e932",
    "submission_receipt_sha256":
        "2f547384f7ec2632cd604e0b68d9aa8c4673ab8f83010c319476078da7c940de",
    "report_sha256":
        "64cb617a81dbea65cbc14c3d62e9988921a56a81408a561a86586ef4b61bf520",
    "failure_evidence": V3_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "23684daa03b9f8791625514cecd84432ecb99c202b022504d724724b68b36096",
    "campaign_ledger_records": 10,
}
V3_PREDECESSOR_REFERENCE = V3_PREDECESSOR

V4_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v4"
)
V4_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v4_failure_2026_10_02.json"
    ),
    "sha256":
        "051f5292f51dc2ed413a277932fb79556295732a58634c7cdff7cd27fe807517",
}
V4_PREDECESSOR_RECORD = {
    "stage": V4_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884494",
    "pin_sha256":
        "ac12cbda1e1c5dcb75502c20730b8d4e645850d78fe3cee18a09cf454010cabe",
    "stage_creation_sha256":
        "cc0d9cedaa385b4366014c5f3f7d799c9367f4c8eca2f62b6182738949325dc4",
}
V4_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v5",
    "stage": V4_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655301397,
    "stage_tree_sha256":
        "7868842059b3bef7b6d53196f76a48c89105bd2a7d00a27b82fe5446b04f8d5b",
    "stage_tree_files": 95,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2791040,
    "record": V4_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "prepared.json"
        ): {
            "sha256":
                "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
            "bytes": 13799,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "sealed.json"
        ): {
            "sha256":
                "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "published.json"
        ): {
            "sha256":
                "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "prepared.json"
        ): {
            "sha256":
                "92b36d334df1a1a5d5e19ee8921e3bcea8dafd094173ce02da41e3b0854e0873",
            "bytes": 12710,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "sealed.json"
        ): {
            "sha256":
                "2aac4a9a431fe46aca4c7c45ff4170406224ca1eb2c8915a34dbabd30208ff03",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "published.json"
        ): {
            "sha256":
                "94556a39fc41caa9f15f6ece4c8030f87c7cb3a094c964e994e50759e82bed82",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "2534c9f88fa8c1d25672d4d565387dfa18bbac48d2ca50319d9c41aa9ac9f57c",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "cc0d9cedaa385b4366014c5f3f7d799c9367f4c8eca2f62b6182738949325dc4",
            "bytes": 12949,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v4-pin.json": {
            "sha256":
                "ac12cbda1e1c5dcb75502c20730b8d4e645850d78fe3cee18a09cf454010cabe",
            "bytes": 11228,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "be21a7e4781b622d8ea9393a555f2253d7ed07fc66f65163b878367e43e4f9e0",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "110c415e5b19aa135ba372935cac573ed05b1a0ca88d8a62af0de2180f9c6d6b",
            "bytes": 7301,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/report.json": {
            "sha256":
                "3878e1f5cd80781c53f2e3027e7043f8c13f53440318d83c0e1251d15dededc2",
            "bytes": 36445,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/report.sha256": {
            "sha256":
                "3fa54735e5c0836808602d0dcf29a1c7fbf0011443094595da44c55cded3736b",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-campaign-stage-creation.stderr": {
            "sha256":
                "1b989e08309e7550faaf1f065467905c8d176a8d5ff4cfb8eafdfee1a75e39a5",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-campaign-stage-creation.time": {
            "sha256":
                "afcb77506712b17dd41436f0916e934036d3c829675a193dabdf021ba610b6f6",
            "bytes": 860,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-progressive-submit.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-progressive-submit.stderr": {
            "sha256":
                "be6d058619053f2046f73cfb89af03bbdb4837c98fd72f4d13d1aa4f459e1fa5",
            "bytes": 1704,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-progressive-submit.time": {
            "sha256":
                "c71e93fd4c5fe21da20021a85488a5261ed5f946ca421f9e5953cfea9ddbd0ac",
            "bytes": 846,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-weyl-context-core-contract.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-weyl-context-core-contract.stderr": {
            "sha256":
                "9faace8f7eae2df8d428884fcbf85ae240ceaf8cc8640c7e6ceb94e5fee008cd",
            "bytes": 2300,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-weyl-context-core-contract.time": {
            "sha256":
                "15bfca85622bb8d4506a3172dc081778623a4681c80c83d8275b156c5f38929f",
            "bytes": 851,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-math-weyl-context-core-capture.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-math-weyl-context-core-capture.stderr": {
            "sha256":
                "a6d93921f01c5919c77d6dd2f161995325c2e6d29aa2bff5f2cac2539d3cf30a",
            "bytes": 1044,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884494/test-math-weyl-context-core-capture.time": {
            "sha256":
                "d505752129b4939ecf910cae2fea2674be9459f5c29f8aa47456179f533ee78c",
            "bytes": 894,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v4-3884494.out": {
            "sha256":
                "9eea9e147f77216c2562da813eea9a0e620f66cbd10e0902f8365c6a1545e7ca",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V4_PREDECESSOR = {
    "stage": V4_PREDECESSOR_STAGE,
    "job": "3884494",
    "pin_sha256":
        "ac12cbda1e1c5dcb75502c20730b8d4e645850d78fe3cee18a09cf454010cabe",
    "stage_creation_sha256":
        "cc0d9cedaa385b4366014c5f3f7d799c9367f4c8eca2f62b6182738949325dc4",
    "stage_creation_contract_sha256":
        "17a46812009028b764b320f99e2a57119b246f874512536ebac15a5cd92608f4",
    "submission_intent_sha256":
        "be21a7e4781b622d8ea9393a555f2253d7ed07fc66f65163b878367e43e4f9e0",
    "submission_receipt_sha256":
        "110c415e5b19aa135ba372935cac573ed05b1a0ca88d8a62af0de2180f9c6d6b",
    "report_sha256":
        "3878e1f5cd80781c53f2e3027e7043f8c13f53440318d83c0e1251d15dededc2",
    "failure_evidence": V4_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "00b7bc1cb917bfa15d729620f807cff92d2988c375a237d13fe5fdce9320a8ab",
    "campaign_ledger_records": 11,
}
V4_PREDECESSOR_REFERENCE = V4_PREDECESSOR

V5_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v5"
)
V5_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v5_failure_2026_10_02.json"
    ),
    "sha256":
        "5620228b361952f34a5cf5400cfa21fcd1c951131a0a470a4909b0993bfc3db9",
}
V5_PREDECESSOR_RECORD = {
    "stage": V5_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884727",
    "pin_sha256":
        "80d61ee985cbf6a920908d7b754e48654112baa7790611776f7a2cf715c47a13",
    "stage_creation_sha256":
        "1e7d077e43d6d1cec6199e1e77e2842edf314742e4694c58c155179467a9a95f",
}
V5_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v6",
    "stage": V5_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655303288,
    "stage_tree_sha256":
        "6ce9bc7b83a1dbba5f7a40fc22e47da3c7726bcc726ac4453d54ca76aa94d6ee",
    "stage_tree_files": 97,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2876626,
    "record": V5_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "prepared.json"
        ): {
            "sha256":
                "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
            "bytes": 13799,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "sealed.json"
        ): {
            "sha256":
                "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "published.json"
        ): {
            "sha256":
                "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "prepared.json"
        ): {
            "sha256":
                "92b36d334df1a1a5d5e19ee8921e3bcea8dafd094173ce02da41e3b0854e0873",
            "bytes": 12710,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "sealed.json"
        ): {
            "sha256":
                "2aac4a9a431fe46aca4c7c45ff4170406224ca1eb2c8915a34dbabd30208ff03",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "published.json"
        ): {
            "sha256":
                "94556a39fc41caa9f15f6ece4c8030f87c7cb3a094c964e994e50759e82bed82",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "prepared.json"
        ): {
            "sha256":
                "e6307e1efe1b07ba720de39948ac994604a235a00413a5aa8032220f3d5a9843",
            "bytes": 15601,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "sealed.json"
        ): {
            "sha256":
                "7e16e5bf2c8bdbc952ff9afec01860e12ab2f37dca6cf65910faa18214131452",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "published.json"
        ): {
            "sha256":
                "2227f14946639fd06df8f92bde178041be34e88b5feab9f4516ee8e8e00eda71",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "1245860dd55f6f53a09f5b03dd9f0bf6e7752701676e38092c2a8f1294757d4c",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "1e7d077e43d6d1cec6199e1e77e2842edf314742e4694c58c155179467a9a95f",
            "bytes": 15840,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v5-pin.json": {
            "sha256":
                "80d61ee985cbf6a920908d7b754e48654112baa7790611776f7a2cf715c47a13",
            "bytes": 11253,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "392747371545a3b78f51ee462eac4e41f392e4cad955b95acecd04d15a954ee4",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "5651a5490ccd58b9c257d26f0b79f432a01a4acaf97543f583a72e2d5d11ca4c",
            "bytes": 7173,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/report.json": {
            "sha256":
                "b50d7aae9f04d30cc5d8ca7c9ccb85405c412bc8e8e251aa559eb1b1220d785a",
            "bytes": 36496,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/report.sha256": {
            "sha256":
                "c21cc735a26ebb514015b683a6e1f24a420346fe4b815fc9477900bc50d25f60",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-campaign-stage-creation.stderr": {
            "sha256":
                "91aa0d814c79da1f1fbae330cb582a30017341fb8f842ddc19d0cafecf93b132",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-campaign-stage-creation.time": {
            "sha256":
                "a3f78dc4a5c3e9198b2176c2eb468fc823b5822d9c53cf291dbbdd3d05dda072",
            "bytes": 859,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-progressive-submit.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-progressive-submit.stderr": {
            "sha256":
                "7d65d1a244eab2304c3078f97561bf0c584640e81d85d171416d203567f4335d",
            "bytes": 1704,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-progressive-submit.time": {
            "sha256":
                "2d49df27ba56483d591c10f07f1430399003f8ae0dec7f3fe1dc16996e48b78e",
            "bytes": 846,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-weyl-context-core-contract.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-weyl-context-core-contract.stderr": {
            "sha256":
                "81423aa640f48c53c21c111f015084220da520f528d72a974af62e1df64201e5",
            "bytes": 2300,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-weyl-context-core-contract.time": {
            "sha256":
                "e7a3789e6b42933a2e791f9e5b4c03ba32b48591a406c0b498645c8e438e0647",
            "bytes": 851,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-math-weyl-context-core-capture.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-math-weyl-context-core-capture.stderr": {
            "sha256":
                "9d9646c8fa955a67cef4a535fab37bba93625bf50ba683c50083dc70cf4b7108",
            "bytes": 4324,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884727/test-math-weyl-context-core-capture.time": {
            "sha256":
                "95a9cccbc39ebcb69c99e427bdfc363cd371cad60f6d43c68fe8bcda2eed170c",
            "bytes": 903,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v5-3884727.out": {
            "sha256":
                "c4ddeabc94329245142e342cfa8ffbea621428e89fe8c5ad4984c8f97ff42441",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V5_PREDECESSOR = {
    "stage": V5_PREDECESSOR_STAGE,
    "job": "3884727",
    "pin_sha256":
        "80d61ee985cbf6a920908d7b754e48654112baa7790611776f7a2cf715c47a13",
    "stage_creation_sha256":
        "1e7d077e43d6d1cec6199e1e77e2842edf314742e4694c58c155179467a9a95f",
    "stage_creation_contract_sha256":
        "00ab3460ea1591882168cc379c5e264bfc915f473f660190d6231ceace578b69",
    "submission_intent_sha256":
        "392747371545a3b78f51ee462eac4e41f392e4cad955b95acecd04d15a954ee4",
    "submission_receipt_sha256":
        "5651a5490ccd58b9c257d26f0b79f432a01a4acaf97543f583a72e2d5d11ca4c",
    "report_sha256":
        "b50d7aae9f04d30cc5d8ca7c9ccb85405c412bc8e8e251aa559eb1b1220d785a",
    "failure_evidence": V5_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "e5c1ed292453da24720c180dd0702cc031bd125716bf0ce053049e32bcc0d4bc",
    "campaign_ledger_records": 12,
}
V5_PREDECESSOR_REFERENCE = V5_PREDECESSOR

V6_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v6"
)
V6_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v6_failure_2026_10_02.json"
    ),
    "sha256":
        "98661668b2994aed264ed5bd28d0663bbbd8a5d5b434497d4463368803def6ce",
}
V6_PREDECESSOR_RECORD = {
    "stage": V6_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884751",
    "pin_sha256":
        "abb82bdb238ef30663941f81d706bef2c0104e3832abe8def9097d6597878956",
    "stage_creation_sha256":
        "e964703079cb928552dc9e6414bd6bd455d94a20856973db967415b01ffae325",
}
V6_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v7",
    "stage": V6_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655305762,
    "stage_tree_sha256":
        "ec238444f343a2432a9af3088664081957ed19aa5e2ed689ecd7a9ab6cee6cc0",
    "stage_tree_files": 108,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 2970431,
    "record": V6_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "prepared.json"
        ): {
            "sha256":
                "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
            "bytes": 13799,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "sealed.json"
        ): {
            "sha256":
                "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "published.json"
        ): {
            "sha256":
                "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "prepared.json"
        ): {
            "sha256":
                "92b36d334df1a1a5d5e19ee8921e3bcea8dafd094173ce02da41e3b0854e0873",
            "bytes": 12710,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "sealed.json"
        ): {
            "sha256":
                "2aac4a9a431fe46aca4c7c45ff4170406224ca1eb2c8915a34dbabd30208ff03",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "published.json"
        ): {
            "sha256":
                "94556a39fc41caa9f15f6ece4c8030f87c7cb3a094c964e994e50759e82bed82",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "prepared.json"
        ): {
            "sha256":
                "e6307e1efe1b07ba720de39948ac994604a235a00413a5aa8032220f3d5a9843",
            "bytes": 15601,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "sealed.json"
        ): {
            "sha256":
                "7e16e5bf2c8bdbc952ff9afec01860e12ab2f37dca6cf65910faa18214131452",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "published.json"
        ): {
            "sha256":
                "2227f14946639fd06df8f92bde178041be34e88b5feab9f4516ee8e8e00eda71",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "prepared.json"
        ): {
            "sha256":
                "cabfd13f37e08a1e122b1afcc64095529062d0bd8a574052a02374c1c394e9fc",
            "bytes": 16499,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "sealed.json"
        ): {
            "sha256":
                "ce7ab08f63d41d4808f0791aa87d99182c13793ed5b52ae83b96105a293e5568",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "published.json"
        ): {
            "sha256":
                "5b9528bd093b1d671517979b143a1bcc4e26768395dbea2ddfb7e6ab1a60f2a6",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "685822dd43d5cf50a2a6cd4555d14fb69510324f785104f0e10f300de028154a",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "e964703079cb928552dc9e6414bd6bd455d94a20856973db967415b01ffae325",
            "bytes": 16738,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v6-pin.json": {
            "sha256":
                "abb82bdb238ef30663941f81d706bef2c0104e3832abe8def9097d6597878956",
            "bytes": 11404,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "9f2ba0bb36ec25777792542f80e77d17bb8a86d730358a9ead3a6286ce3f8393",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "f1399b73a76241b358a5c5fe3d3b1df4575a8d894ffdbeebab58facec765abab",
            "bytes": 7171,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/report.json": {
            "sha256":
                "503f70735c9b23739520b4d30bd51e8a2c3fd1cfc13da0ac801398132b0893de",
            "bytes": 38975,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/report.sha256": {
            "sha256":
                "4152f51ceffce255964ad3145e18cd7e10348ba03ec6069c897614f085d90dba",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-campaign-stage-creation.stderr": {
            "sha256":
                "7dbe2b13d224c57e773b3ebbaaa42e237b70041590a99cf77097f722f20757bf",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-campaign-stage-creation.time": {
            "sha256":
                "824c5795519091a648441ca795dfb17d38045884f41161f73e6b5c8e7d618c1c",
            "bytes": 859,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-progressive-submit.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-progressive-submit.stderr": {
            "sha256":
                "5e7d9b3414564b47d2d18f0c108f9b1a32ead089f2f33d8a91becc46615716ef",
            "bytes": 1704,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-progressive-submit.time": {
            "sha256":
                "1af02c4d43313d91b03b7d1cc9c6adea650bacaf54b6b7d7542ef8e952016cf7",
            "bytes": 846,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-weyl-context-core-contract.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-weyl-context-core-contract.stderr": {
            "sha256":
                "8634408934711a7ba27040666f8eadd66beeda9e5175e5449761975e56884970",
            "bytes": 2300,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-weyl-context-core-contract.time": {
            "sha256":
                "fe95bbd52a7275bc8e3cdf35d34552ef25cf116723fb60686babb5ffc755a2a7",
            "bytes": 851,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-math-weyl-context-core-capture.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-math-weyl-context-core-capture.stderr": {
            "sha256":
                "ab106c62b2e1187330152d23cad51bbf2ddec3e2fe035913bce199cce06756a3",
            "bytes": 3707,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-math-weyl-context-core-capture.time": {
            "sha256":
                "084d1eef326dc0dff4e06d9e9fd154ca3f7842878aa82fd0d7772737cd44c5b8",
            "bytes": 865,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-stager-allowlist.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-stager-allowlist.stderr": {
            "sha256":
                "66a344f84883913e3849d2c8ad826100dbf156cd0daeffb456679e822fff8dfe",
            "bytes": 830,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/test-stager-allowlist.time": {
            "sha256":
                "ca26aa68687377e96e498880ba43a9370d6b25ffd745fc8573d0926062a0b6cc",
            "bytes": 842,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/rustc-version.stdout": {
            "sha256":
                "f64c55ed9f210d80a9b248206104a47175faf5ec94a093e4ad132549209089fc",
            "bytes": 196,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/rustc-version.stderr": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/rustc-version.time": {
            "sha256":
                "3836d4d332b5536e9baca891145d3777c949008bd6abc31e08d25017765e8c5f",
            "bytes": 733,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/cargo-version.stdout": {
            "sha256":
                "fdb3f2fc57229bfd22c53064c26587a57a856e5ef3f9f5cd5e30d5af2c818a1d",
            "bytes": 320,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/cargo-version.stderr": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884751/cargo-version.time": {
            "sha256":
                "395dbef32a254c5b9f78928ef526fc8a7fe73d047ee552593ac4a27ac10e1d82",
            "bytes": 728,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v6-3884751.out": {
            "sha256":
                "a7de9eefb4f494364b3810c2c37b35302f92fbf24d34aee226fc43d7c85b9638",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V6_PREDECESSOR = {
    "stage": V6_PREDECESSOR_STAGE,
    "job": "3884751",
    "pin_sha256":
        "abb82bdb238ef30663941f81d706bef2c0104e3832abe8def9097d6597878956",
    "stage_creation_sha256":
        "e964703079cb928552dc9e6414bd6bd455d94a20856973db967415b01ffae325",
    "stage_creation_contract_sha256":
        "6a84cef24b252ece488f24e7181e4fe2cef8472726addfca7399946a237d6f6e",
    "submission_intent_sha256":
        "9f2ba0bb36ec25777792542f80e77d17bb8a86d730358a9ead3a6286ce3f8393",
    "submission_receipt_sha256":
        "f1399b73a76241b358a5c5fe3d3b1df4575a8d894ffdbeebab58facec765abab",
    "report_sha256":
        "503f70735c9b23739520b4d30bd51e8a2c3fd1cfc13da0ac801398132b0893de",
    "failure_evidence": V6_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "3c31e158df8ed6b06618cdf2818d6107b1379c8ed0e7bc28db2bdfd162e43c83",
    "campaign_ledger_records": 13,
}
V6_PREDECESSOR_REFERENCE = V6_PREDECESSOR

V7_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v7"
)
V7_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v7_failure_2026_10_02.json"
    ),
    "sha256":
        "9a9f0b103c323cbcc9a53838c4ba94e93d0b5cfc3fe62aea22b7806184066a87",
}
V7_PREDECESSOR_RECORD = {
    "stage": V7_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884780",
    "pin_sha256":
        "a5c41005aee0ffdfce815feb35acc8d95dcef564f7ccd3264977262a503c1805",
    "stage_creation_sha256":
        "328e3c0c4a24b6bba94f17aa6dee54d14fcd5accc081d6febec8035f7f103bd6",
}
V7_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v8",
    "stage": V7_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655307095,
    "stage_tree_sha256":
        "32a44f0679e3220a47ed5ed0dcad7d8a750de3610cd375ec4b05b3677131c8d8",
    "stage_tree_files": 95,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3071613,
    "record": V7_PREDECESSOR_RECORD,
    "campaign_files": {
        ".atlas-stage-creation-prepared.json": {
            "sha256":
                "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
            "bytes": 5898,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-sealed.json": {
            "sha256":
                "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation-published.json": {
            "sha256":
                "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-failure-"
            "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
        ): {
            "sha256":
                "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
            "bytes": 5481,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "prepared.json"
        ): {
            "sha256":
                "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
            "bytes": 10614,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "sealed.json"
        ): {
            "sha256":
                "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v2-"
            "published.json"
        ): {
            "sha256":
                "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "prepared.json"
        ): {
            "sha256":
                "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
            "bytes": 13799,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "sealed.json"
        ): {
            "sha256":
                "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v3-"
            "published.json"
        ): {
            "sha256":
                "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "prepared.json"
        ): {
            "sha256":
                "92b36d334df1a1a5d5e19ee8921e3bcea8dafd094173ce02da41e3b0854e0873",
            "bytes": 12710,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "sealed.json"
        ): {
            "sha256":
                "2aac4a9a431fe46aca4c7c45ff4170406224ca1eb2c8915a34dbabd30208ff03",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v4-"
            "published.json"
        ): {
            "sha256":
                "94556a39fc41caa9f15f6ece4c8030f87c7cb3a094c964e994e50759e82bed82",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "prepared.json"
        ): {
            "sha256":
                "e6307e1efe1b07ba720de39948ac994604a235a00413a5aa8032220f3d5a9843",
            "bytes": 15601,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "sealed.json"
        ): {
            "sha256":
                "7e16e5bf2c8bdbc952ff9afec01860e12ab2f37dca6cf65910faa18214131452",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v5-"
            "published.json"
        ): {
            "sha256":
                "2227f14946639fd06df8f92bde178041be34e88b5feab9f4516ee8e8e00eda71",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "prepared.json"
        ): {
            "sha256":
                "cabfd13f37e08a1e122b1afcc64095529062d0bd8a574052a02374c1c394e9fc",
            "bytes": 16499,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "sealed.json"
        ): {
            "sha256":
                "ce7ab08f63d41d4808f0791aa87d99182c13793ed5b52ae83b96105a293e5568",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v6-"
            "published.json"
        ): {
            "sha256":
                "5b9528bd093b1d671517979b143a1bcc4e26768395dbea2ddfb7e6ab1a60f2a6",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v7-"
            "prepared.json"
        ): {
            "sha256":
                "23326f302ad5030ce8bca84fe082e583b704cf2f5e9459d7b0e831b7cf20d653",
            "bytes": 19544,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v7-"
            "sealed.json"
        ): {
            "sha256":
                "71ac6f4387980ed91d0a159ed210b7afb42dea865551d5e9cbfb80308a0afd96",
            "bytes": 590,
            "mode": "0444",
            "nlink": 1,
        },
        (
            ".atlas-stage-creation-weyl-context-core-capture-v7-"
            "published.json"
        ): {
            "sha256":
                "b5eebd18ddf7db75c206f6469c20f258b1d7fb2cd2fc3f6d2fb99c547102e0a6",
            "bytes": 678,
            "mode": "0444",
            "nlink": 1,
        },
    },
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "b0ac0819f1ac0def9e5d6fac119801ec79d57adcea42d4504c9c6d04609ee73b",
            "bytes": 327,
            "mode": "0444",
            "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "328e3c0c4a24b6bba94f17aa6dee54d14fcd5accc081d6febec8035f7f103bd6",
            "bytes": 19783,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v7-pin.json": {
            "sha256":
                "a5c41005aee0ffdfce815feb35acc8d95dcef564f7ccd3264977262a503c1805",
            "bytes": 11707,
            "mode": "0444",
            "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "e78d1f1ebfcd22271eb5c41ebdf315212853692e095c6791cd2b0bd002fd179d",
            "bytes": 428,
            "mode": "0444",
            "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "9158e691584cdf78e9a07cd0d374fe96d383f51bd039f2de9037c2c767946fff",
            "bytes": 7321,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": 1283,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/report.json": {
            "sha256":
                "d7a8e3e3b779f070feff8dd582dae17635efd97505780f37a9661769fec23243",
            "bytes": 34972,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/report.sha256": {
            "sha256":
                "efca192aa496b8b47a954f3df7ba0b27b9ab7389a1a20ea7b8d05298f36b8e08",
            "bytes": 65,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-campaign-stage-creation.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-campaign-stage-creation.stderr": {
            "sha256":
                "ff73093590166639fd9012e10513266c95e578f3c1092dbe0c3067f74859906b",
            "bytes": 3857,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-campaign-stage-creation.time": {
            "sha256":
                "6ab955469418440aa095b04009b317871a7f2ac2f86a283b19c5377003cd0f33",
            "bytes": 859,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-progressive-submit.stdout": {
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "bytes": 0,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-progressive-submit.stderr": {
            "sha256":
                "7e10b2ca8baaa409332e6227ee4b9bf393e13fa50942daa200b44206435a74ff",
            "bytes": 5312,
            "mode": "0444",
            "nlink": 1,
        },
        "results/3884780/test-progressive-submit.time": {
            "sha256":
                "bbe4d5f9757bda787c236ee05c83aac17b535edba4d4df28e67cb69c4bb1de5d",
            "bytes": 884,
            "mode": "0444",
            "nlink": 1,
        },
        "weyl-context-core-capture-v7-3884780.out": {
            "sha256":
                "b42ab383b6e66b89af8bfecdd7c1ae10db3601bffeffac0431e8b3651d764858",
            "bytes": 115,
            "mode": "0644",
            "nlink": 1,
        },
    },
}
V7_PREDECESSOR = {
    "stage": V7_PREDECESSOR_STAGE,
    "job": "3884780",
    "pin_sha256":
        "a5c41005aee0ffdfce815feb35acc8d95dcef564f7ccd3264977262a503c1805",
    "stage_creation_sha256":
        "328e3c0c4a24b6bba94f17aa6dee54d14fcd5accc081d6febec8035f7f103bd6",
    "stage_creation_contract_sha256":
        "16f7896239dacfd92a9502b7179f5c6a6d2b585108a2959f411da98b1c6d84cf",
    "submission_intent_sha256":
        "e78d1f1ebfcd22271eb5c41ebdf315212853692e095c6791cd2b0bd002fd179d",
    "submission_receipt_sha256":
        "9158e691584cdf78e9a07cd0d374fe96d383f51bd039f2de9037c2c767946fff",
    "report_sha256":
        "d7a8e3e3b779f070feff8dd582dae17635efd97505780f37a9661769fec23243",
    "failure_evidence": V7_FAILURE_EVIDENCE,
    "campaign_ledger_sha256":
        "11c0b626ad22e56c68f1e7d7c0f4d7336574b1ca3afb1374d6d33de06ec55b5e",
    "campaign_ledger_records": 14,
}
V7_PREDECESSOR_REFERENCE = V7_PREDECESSOR

V8_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-capture-v8"
)
V8_PREDECESSOR_RECORD = {
    "stage": V8_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884807",
    "pin_sha256":
        "600ed2f6ea082ea62e945a4cc2805313ba8722ebdfec65548db68ebd2082e28c",
    "stage_creation_sha256":
        "f00aacd1a5a735c8d153fd372876aa2d235fc8ba643042ea95e0983a7e11cd43",
}
V8_PREDECESSOR_CAMPAIGN_FILES = {
    ".atlas-stage-creation-prepared.json": {
        "sha256": "ee64d9b336908df79b85af16fd6fe6191a31500d73f6986af04b9e8d25e799d7",
        "bytes": 5898, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-sealed.json": {
        "sha256": "b4d71b66239c87c6925ad0de034a51764fe400ecbbd5e4194dbf3cf4b87fbaa3",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-published.json": {
        "sha256": "eeb045fed8a0cab82ff710700f4558e482aa3d169390a08ebd185329192c40b0",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    (
        ".atlas-stage-creation-failure-"
        "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
    ): {
        "sha256": "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
        "bytes": 5481, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v2-prepared.json": {
        "sha256": "ec440d939bd9bdc62fd917bdeca289c0dbd243456018ec030a2f72815b7045aa",
        "bytes": 10614, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v2-sealed.json": {
        "sha256": "e3915036d7673e6cec998ed93549be4ed2e74e995eba1a17e96cc95adda77cb0",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v2-published.json": {
        "sha256": "036d741e1bff3288297ba4d7e987dcf47cf6936febd3be143e022009b6f32c1c",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v3-prepared.json": {
        "sha256": "c34413614d3f59af73462f3cbb3038ef1423340474d5335f43473ac3d1cd1c04",
        "bytes": 13799, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v3-sealed.json": {
        "sha256": "55139441a0c3f5a130913aa73aeab72f1ee44bdb76556d223eafeafc52dd3786",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v3-published.json": {
        "sha256": "df4b01435e66eacdf28c9fac1940e0239da863f29cee6c6bb4c05e2af12f4342",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v4-prepared.json": {
        "sha256": "92b36d334df1a1a5d5e19ee8921e3bcea8dafd094173ce02da41e3b0854e0873",
        "bytes": 12710, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v4-sealed.json": {
        "sha256": "2aac4a9a431fe46aca4c7c45ff4170406224ca1eb2c8915a34dbabd30208ff03",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v4-published.json": {
        "sha256": "94556a39fc41caa9f15f6ece4c8030f87c7cb3a094c964e994e50759e82bed82",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v5-prepared.json": {
        "sha256": "e6307e1efe1b07ba720de39948ac994604a235a00413a5aa8032220f3d5a9843",
        "bytes": 15601, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v5-sealed.json": {
        "sha256": "7e16e5bf2c8bdbc952ff9afec01860e12ab2f37dca6cf65910faa18214131452",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v5-published.json": {
        "sha256": "2227f14946639fd06df8f92bde178041be34e88b5feab9f4516ee8e8e00eda71",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v6-prepared.json": {
        "sha256": "cabfd13f37e08a1e122b1afcc64095529062d0bd8a574052a02374c1c394e9fc",
        "bytes": 16499, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v6-sealed.json": {
        "sha256": "ce7ab08f63d41d4808f0791aa87d99182c13793ed5b52ae83b96105a293e5568",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v6-published.json": {
        "sha256": "5b9528bd093b1d671517979b143a1bcc4e26768395dbea2ddfb7e6ab1a60f2a6",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v7-prepared.json": {
        "sha256": "23326f302ad5030ce8bca84fe082e583b704cf2f5e9459d7b0e831b7cf20d653",
        "bytes": 19544, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v7-sealed.json": {
        "sha256": "71ac6f4387980ed91d0a159ed210b7afb42dea865551d5e9cbfb80308a0afd96",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v7-published.json": {
        "sha256": "b5eebd18ddf7db75c206f6469c20f258b1d7fb2cd2fc3f6d2fb99c547102e0a6",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v8-prepared.json": {
        "sha256": "a6dc48041076fb7b1a75d6ff624472642ed7a7a388c7fec8224bf05c8ffb0016",
        "bytes": 16775, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v8-sealed.json": {
        "sha256": "4ba9d36345769dbe56ba2f154972dc23494d3c160ee470862d02fec5c19a6a46",
        "bytes": 590, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-capture-v8-published.json": {
        "sha256": "4e0d511c9a6d648ac1e7059b230177361e71ca43d8858be2382b55681fcfcc73",
        "bytes": 678, "mode": "0444", "nlink": 1,
    },
}
V8_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v9",
    "stage": V8_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655308072,
    "stage_tree_sha256":
        "da98c876de3edc46d12eb21ac605839b1d8ae7a63f1b2e887af24bd35504ee63",
    "stage_tree_files": 134,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3471903,
    "record": V8_PREDECESSOR_RECORD,
    "campaign_files": V8_PREDECESSOR_CAMPAIGN_FILES,
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256": "e9cf7e66e245aba4dcc566bada6333b3f1b89425620092b6e656365965e2ef77",
            "bytes": 327, "mode": "0444", "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256": "f00aacd1a5a735c8d153fd372876aa2d235fc8ba643042ea95e0983a7e11cd43",
            "bytes": 17014, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-capture-v8-pin.json": {
            "sha256": "600ed2f6ea082ea62e945a4cc2805313ba8722ebdfec65548db68ebd2082e28c",
            "bytes": 11622, "mode": "0444", "nlink": 1,
        },
        "submission-intent.json": {
            "sha256": "53813783c18ab716b72baefff2f503752f5d09d1df27c0358ba77cdb5e7a86a7",
            "bytes": 428, "mode": "0444", "nlink": 1,
        },
        "submission.json": {
            "sha256": "6238f1e16ecec7793f74490243bcf0339dcf43560fb8b16e012f0964ec28ccec",
            "bytes": 7083, "mode": "0444", "nlink": 1,
        },
        "results/3884807/catalog.json": {
            "sha256": CATALOG_SHA256,
            "bytes": CATALOG_BYTES, "mode": "0444", "nlink": 1,
        },
        "results/3884807/report.json": {
            "sha256": "e644017f6c040691c91ee56fdaeac09a3e8d834ec9956939c2ba2530c80c5c73",
            "bytes": 324940, "mode": "0444", "nlink": 1,
        },
        "results/3884807/report.sha256": {
            "sha256": "dd51b4341d5c24eaaf9c2e3a343339ead32ad587bf4c986b584ead80661fb5ee",
            "bytes": 65, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-capture-v8-3884807.out": {
            "sha256": "f6256a7e101918dd145765b8489c71656e0a1795ab6fa2d2b82c3c37f021e8aa",
            "bytes": 145, "mode": "0644", "nlink": 1,
        },
    },
}
V8_PREDECESSOR = {
    "stage": V8_PREDECESSOR_STAGE,
    "job": "3884807",
    "pin_sha256":
        "600ed2f6ea082ea62e945a4cc2805313ba8722ebdfec65548db68ebd2082e28c",
    "stage_creation_sha256":
        "f00aacd1a5a735c8d153fd372876aa2d235fc8ba643042ea95e0983a7e11cd43",
    "stage_creation_transaction_sha256":
        "e9cf7e66e245aba4dcc566bada6333b3f1b89425620092b6e656365965e2ef77",
    "submission_intent_sha256":
        "53813783c18ab716b72baefff2f503752f5d09d1df27c0358ba77cdb5e7a86a7",
    "submission_receipt_sha256":
        "6238f1e16ecec7793f74490243bcf0339dcf43560fb8b16e012f0964ec28ccec",
    "report_sha256":
        "e644017f6c040691c91ee56fdaeac09a3e8d834ec9956939c2ba2530c80c5c73",
    "report_bytes": 324940,
    "inspection": {
        "file": REGRESSION_INSPECTION_PATH,
        "sha256": REGRESSION_INSPECTION_SHA256,
        "bytes": REGRESSION_INSPECTION_BYTES,
    },
    "submission_evidence": {
        "file": V8_SUBMISSION_EVIDENCE_PATH,
        "sha256": V8_SUBMISSION_EVIDENCE_SHA256,
        "bytes": V8_SUBMISSION_EVIDENCE_BYTES,
    },
    "stage_tree_sha256":
        "da98c876de3edc46d12eb21ac605839b1d8ae7a63f1b2e887af24bd35504ee63",
    "stage_tree_files": 134,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3471903,
    "campaign_ledger_sha256":
        "fe97c6043a160815a097e09e58da59ecf5c1ce6ea4dd9adc961c7a155ebe979b",
    "campaign_ledger_records": 15,
}
V8_PREDECESSOR_REFERENCE = V8_PREDECESSOR

BEFORE_V1_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v1_failure_2026_10_02.json"
    ),
    "sha256":
        "4ea80b67f853789f345911f2a61d71610bf37e3a2d99b8c77805cde3497bf427",
}
BEFORE_V1_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-before-v1"
)
BEFORE_V1_PREDECESSOR_RECORD = {
    "stage": BEFORE_V1_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884862",
    "pin_sha256":
        "e713b56909d43d3f01d8782ef14d269c69d294b4418e56fba1f1929618ed8e12",
    "stage_creation_sha256":
        "7a3c0d7f5f8efba29abcf399545fbadf85c92a01ba45f2430be001bab11ff51e",
}
BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES = V8_PREDECESSOR_CAMPAIGN_FILES | {
    ".atlas-stage-creation-weyl-context-core-before-v1-prepared.json": {
        "sha256":
            "86e075a8269b26ff605c1698479b46071496c12f78333432a1a7e9f8fc947cb6",
        "bytes": 17669, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v1-sealed.json": {
        "sha256":
            "ecedc64f0ca19cfd3eac23f3eebf5ab721f661ffcf69452530483d522bdc2921",
        "bytes": 589, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v1-published.json": {
        "sha256":
            "8edfd835e7118924cc17a6e6fdfd2b050cc995bb9f726529ca640846a2bbd638",
        "bytes": 677, "mode": "0444", "nlink": 1,
    },
}
BEFORE_V1_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v10",
    "stage": BEFORE_V1_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655309521,
    "stage_tree_sha256":
        "6c6cd4e9cfc39102089c07cf499c1817bd1e3e3b52b5b1b2c8baa155b17251af",
    "stage_tree_files": 134,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3486714,
    "record": BEFORE_V1_PREDECESSOR_RECORD,
    "campaign_files": BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES,
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "194932a52a0cea1a1b205716e0ad491b25f8a955a7d68b4bf905c0eea3f61341",
            "bytes": 327, "mode": "0444", "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "7a3c0d7f5f8efba29abcf399545fbadf85c92a01ba45f2430be001bab11ff51e",
            "bytes": 17907, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v1-pin.json": {
            "sha256":
                "e713b56909d43d3f01d8782ef14d269c69d294b4418e56fba1f1929618ed8e12",
            "bytes": 16515, "mode": "0444", "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "1dc58e9d087bf7c357966337131d34faf83879b6382178c97420e8528dc2c13a",
            "bytes": 427, "mode": "0444", "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "843cc031f097bff437918915d7f58c73761dc2d39f7a95fb9bab0c2d8170399d",
            "bytes": 10715, "mode": "0444", "nlink": 1,
        },
        "overrides/overrides.json": {
            "sha256":
                "df73c17df5b1735692ec18410e97aef57ad7b395c7b95daae2e6132a773dc537",
            "bytes": 6240, "mode": "0444", "nlink": 1,
        },
        "results/3884862/test-math-weyl-context-core-capture.stderr": {
            "sha256":
                "c5cc7e10099130f6755665a1fd53fb59f6681d40e7f38eb7e78d72fc31793446",
            "bytes": 5271, "mode": "0444", "nlink": 1,
        },
        "results/3884862/report.json": {
            "sha256":
                "1972be6c1bb893153d7765b3f778dbb96707f1f90ad327f54a07311c2b57c21c",
            "bytes": 51145, "mode": "0444", "nlink": 1,
        },
        "results/3884862/report.sha256": {
            "sha256":
                "96db6b7ab22a73c86994364363f14ffe175c0c6ee7eb3ad60063ba00332e5f79",
            "bytes": 65, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v1-3884862.out": {
            "sha256":
                "9c9c43609d4aba8ba193fd1a2223cd7e531a0f6fbc92f45633bbd4470cfdc1bc",
            "bytes": 134, "mode": "0644", "nlink": 1,
        },
    },
}
BEFORE_V1_PREDECESSOR = {
    "stage": BEFORE_V1_PREDECESSOR_STAGE,
    "job": "3884862",
    "pin_sha256":
        "e713b56909d43d3f01d8782ef14d269c69d294b4418e56fba1f1929618ed8e12",
    "stage_creation_sha256":
        "7a3c0d7f5f8efba29abcf399545fbadf85c92a01ba45f2430be001bab11ff51e",
    "stage_creation_contract_sha256":
        "e38495b439d89a255555358a047c9e82e99d2e40054a864142e5ef369eb3446d",
    "stage_creation_transaction_sha256":
        "194932a52a0cea1a1b205716e0ad491b25f8a955a7d68b4bf905c0eea3f61341",
    "override_manifest_sha256":
        "df73c17df5b1735692ec18410e97aef57ad7b395c7b95daae2e6132a773dc537",
    "submission_intent_sha256":
        "1dc58e9d087bf7c357966337131d34faf83879b6382178c97420e8528dc2c13a",
    "submission_receipt_sha256":
        "843cc031f097bff437918915d7f58c73761dc2d39f7a95fb9bab0c2d8170399d",
    "report_sha256":
        "1972be6c1bb893153d7765b3f778dbb96707f1f90ad327f54a07311c2b57c21c",
    "report_bytes": 51145,
    "failure_evidence": BEFORE_V1_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "6c6cd4e9cfc39102089c07cf499c1817bd1e3e3b52b5b1b2c8baa155b17251af",
    "stage_tree_files": 134,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3486714,
    "campaign_ledger_sha256":
        "2da7643c27debb664ade23a8d2d5e1ae881a6a45857f4ea5dfdea5311eddf4ad",
    "campaign_ledger_records": 16,
}
BEFORE_V1_PREDECESSOR_REFERENCE = BEFORE_V1_PREDECESSOR

BEFORE_V2_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v2_failure_2026_10_02.json"
    ),
    "sha256":
        "ee4cc4db5b4c1bb64d280ea9e47d85ea9ca8b99356312bfce5c312f30359e80f",
}
BEFORE_V2_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-before-v2"
)
BEFORE_V2_PREDECESSOR_RECORD = {
    "stage": BEFORE_V2_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884880",
    "pin_sha256":
        "bc98d9a033ab1fb0829456ef8a852b0db504a2eeffff6a6e6c377ddad045b750",
    "stage_creation_sha256":
        "82b88fcc782c540055f11ef07e37244e73bec13fd1179c7d0b3c40747978dec9",
}
BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES = \
    BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES | {
    ".atlas-stage-creation-weyl-context-core-before-v2-prepared.json": {
        "sha256":
            "0563d3d7147428b18a430b6d51729c7efb9a1f8d4502612bf6acced10b7b2b3f",
        "bytes": 18839, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v2-sealed.json": {
        "sha256":
            "1e0ed9bb398dd0f820c895225ae6a091206deba2004ab58db6accb79b0d7217e",
        "bytes": 589, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v2-published.json": {
        "sha256":
            "7841d14640dce85af7e2f40620b76546894672ea3d64b1f47064d5ad9dae06c0",
        "bytes": 677, "mode": "0444", "nlink": 1,
    },
}
BEFORE_V2_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v11",
    "stage": BEFORE_V2_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655310146,
    "stage_tree_sha256":
        "c5a83247888efdf74957c315436d1a64ce0f9268f30c0e21fe97503855febac4",
    "stage_tree_files": 139,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3584950,
    "record": BEFORE_V2_PREDECESSOR_RECORD,
    "campaign_files": BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES,
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "72d43d72fb28a13ba77be329dc9ac9d8240d8a1d594e8a8a01fc76e4cfdaa785",
            "bytes": 327, "mode": "0444", "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "82b88fcc782c540055f11ef07e37244e73bec13fd1179c7d0b3c40747978dec9",
            "bytes": 19077, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v2-pin.json": {
            "sha256":
                "bc98d9a033ab1fb0829456ef8a852b0db504a2eeffff6a6e6c377ddad045b750",
            "bytes": 16650, "mode": "0444", "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "ac30c59b893ec9c00c2e493eb1e7c98fe0d3596f7a4f4b21c5cc096ad5e70421",
            "bytes": 427, "mode": "0444", "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "a10bb7b0658cda3c7c9fd92853e8418c1be58d576c6feacf5498ad2edeb7925b",
            "bytes": 10698, "mode": "0444", "nlink": 1,
        },
        "overrides/overrides.json": {
            "sha256":
                "0f66de83019ae828b258a06aa38afe312e166075bb5bc1a685c9752cc7b42542",
            "bytes": 6390, "mode": "0444", "nlink": 1,
        },
        "results/3884880/test-stager-allowlist.stderr": {
            "sha256":
                "b25dbead40c0e2c33d53728206589082c5cb7e4285b7e35aaf09e4f23de15cc5",
            "bytes": 2618, "mode": "0444", "nlink": 1,
        },
        "results/3884880/report.json": {
            "sha256":
                "d38a7ace107be965212a46a4edfd54b1327b5429f0893f3a347320cbc553f827",
            "bytes": 52518, "mode": "0444", "nlink": 1,
        },
        "results/3884880/report.sha256": {
            "sha256":
                "cdcc693bf86c7eb852f3fccc269e48d2555742a69b384883f0ea601fd5b7c1eb",
            "bytes": 65, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v2-3884880.out": {
            "sha256":
                "de1f674482400ff5a46f2d3204b9e6f7b9a5b51af16c6a57eb6664077dae67a5",
            "bytes": 134, "mode": "0644", "nlink": 1,
        },
    },
}
BEFORE_V2_PREDECESSOR = {
    "stage": BEFORE_V2_PREDECESSOR_STAGE,
    "job": "3884880",
    "pin_sha256":
        "bc98d9a033ab1fb0829456ef8a852b0db504a2eeffff6a6e6c377ddad045b750",
    "stage_creation_sha256":
        "82b88fcc782c540055f11ef07e37244e73bec13fd1179c7d0b3c40747978dec9",
    "stage_creation_contract_sha256":
        "e2ca9ba1f4c7fe1a81c84783863f95a1f57a6b307271d060718a04a3d1a79afc",
    "stage_creation_transaction_sha256":
        "72d43d72fb28a13ba77be329dc9ac9d8240d8a1d594e8a8a01fc76e4cfdaa785",
    "override_manifest_sha256":
        "0f66de83019ae828b258a06aa38afe312e166075bb5bc1a685c9752cc7b42542",
    "submission_intent_sha256":
        "ac30c59b893ec9c00c2e493eb1e7c98fe0d3596f7a4f4b21c5cc096ad5e70421",
    "submission_receipt_sha256":
        "a10bb7b0658cda3c7c9fd92853e8418c1be58d576c6feacf5498ad2edeb7925b",
    "report_sha256":
        "d38a7ace107be965212a46a4edfd54b1327b5429f0893f3a347320cbc553f827",
    "report_bytes": 52518,
    "failure_evidence": BEFORE_V2_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "c5a83247888efdf74957c315436d1a64ce0f9268f30c0e21fe97503855febac4",
    "stage_tree_files": 139,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3584950,
    "campaign_ledger_sha256":
        "e7a4ba5736bdbb40b97eb4cc2961c6c5756c1d1cd2aae339b6f99095e7987567",
    "campaign_ledger_records": 17,
}
BEFORE_V2_PREDECESSOR_REFERENCE = BEFORE_V2_PREDECESSOR

BEFORE_V3_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v3_failure_2026_10_02.json"
    ),
    "sha256":
        "fd6dd4ce66b743c2db36429dcb609e67f4e945fc1422b3fd82b24de7bb8654fd",
}
BEFORE_V4_RESULT_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v4_2026_10_02.json"
    ),
    "sha256":
        "bf69999feb945b67f586d92d378cc20d238cb518ef84ad6ce11b54e3e74200d3",
}
# The immutable after-v1 harness failure (job 3890328): its diagnosis record
# and its preserved report.  The after-v2 successor must bind both.
AFTER_V1_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v1_failure_2026_10_03.json"
    ),
    "sha256":
        "4e9ff6f8c685699cd05658234c3cbdbb391b247359f600132a8c2b23387252a5",
}
AFTER_V1_FAILURE_REPORT = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v1_failure_report_2026_10_03.json"
    ),
    "sha256":
        "22b0b152ba964a6efae0953b1e36bf67dbf97565ad857d13a4db0fb9e6599a61",
    "bytes": 259179,
}
# The immutable after-v2 harness failure (job 3890580): sbatch's labels were
# never migrated to v2, so it wrote a v1-named output file into the v2 stage
# and validate_stage_topology rejected the stage before any gate ran.  No
# report exists; the after-v3 successor binds the exact failure record.
AFTER_V2_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v2_failure_2026_10_03.json"
    ),
    "sha256":
        "bb23a01fd6088417bcd763da08465ecdc724dd463eea0cf1b6c15ddc544c413d",
}
# The immutable after-v3 harness failure (job 3899303): the repair patch
# migrated session.rs to context.kernel.system but omitted
# domain_builtins/weyl_subgroup.rs, which exists only in the frozen campaign
# source archive, so the release build failed with E0609 after all 125
# checkers passed.  The after-v4 successor binds the exact failure record.
AFTER_V3_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v3_failure_2026_10_06.json"
    ),
    "sha256":
        "45c8b709cf96fd36981a8fba4476e81aa1b03f7c27b9de0686b4f8e4a580efa2",
}
# The immutable after-v4 harness failure (job 3899885): the completed repair
# passed everything (build, inventory, regressions, goldens) but the gate
# demanded byte-equal stderr across two diagnostic renderers on the reject
# capture.  The after-v5 successor binds the exact failure record.
AFTER_V4_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v4_failure_2026_10_06.json"
    ),
    "sha256":
        "6347708e38a28e9d368bc0d2ea6ae32909b57a520d9a492016a0e31eea0b5e6a",
}
# The frozen before-v4 identity, retained for validate_before_v4_result after
# PREDECESSOR advanced to the failed after-v1 stage (the v3 lesson).
BEFORE_V4_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-before-v4"
)
BEFORE_V4_PREDECESSOR = {
    "stage": BEFORE_V4_PREDECESSOR_STAGE,
    "job": "3886748",
    "pin_sha256":
        "54221cf4545875ec768c09d92753ceba5c5140faeacd895cbf8af36327e18e78",
    "stage_creation_sha256":
        "c9197be0f005da73d110a3aac6d9b0531fe92728dd0f8db37d7c2a8b992307c7",
    "stage_creation_contract_sha256":
        "2affae9a40b11f15ca97ad80e38ca95534ff6294ff43e0c7823ede57202d919a",
    "stage_creation_transaction_sha256":
        "4ae83ab4446267af676897c4b1b1ec2ffcf1ead2db30bca431c0140960dff030",
    "override_manifest_sha256":
        "add7dfe0a9d4f0efa1b7d73333149c934d9f367812b47e30a94f5fbe5ff12ebd",
    "submission_intent_sha256":
        "c94c6cc6b8a28f795bb3f8412c15edd7c2ca24b3611378a6771ce93bc9d10ab2",
    "submission_receipt_sha256":
        "f53f37662aa4607647b9055e15544fdd67e8bac9a328971a919afada93d18798",
    "report_sha256":
        "3d0c7c91f672e6bf41830296864a621ee11bbd260735e37c0441d6c499953bce",
    "report_bytes": 344871,
    "result_evidence": BEFORE_V4_RESULT_EVIDENCE,
    "stage_tree_sha256":
        "1fa45944f8f115cb6de516feb15a2d597b3167ba33050a135add020351bd1ee7",
    "stage_tree_files": 180,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 4151738,
    "campaign_ledger_sha256":
        "5381b3b3a8ffcf8ae1566eb63640719ddec13955f681dbeca5361bbf9321cd5a",
    "campaign_ledger_records": 19,
}
# The frozen after-v1 identity, retained for validate_after_v1_failure after
# PREDECESSOR advanced to the failed after-v2 stage.
AFTER_V1_ERA_SOURCE_MANIFEST_SHA256 = (
    "3f8cf4753f29d33df8273086254a4f09170ada46f05e9c13acfdac2f5ab85c33"
)
AFTER_V1_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-after-v1"
)
AFTER_V1_PREDECESSOR = {
    "stage": AFTER_V1_PREDECESSOR_STAGE,
    "job": "3890328",
    "pin_sha256":
        "396f30f2dae9e52637fbfeb0c8b5510286090b0801da4787742c0f2d2b9d7bb4",
    "stage_creation_sha256":
        "ba22b03c4f87171d1ec1b5eb7aa0b8d6018e15d03054e9d5822eabc1e58771ad",
    "stage_creation_contract_sha256":
        "60597853414483ca7b5c16294df712ca6f6dce245de4c6bb7fe8e4d808aeea9e",
    "stage_creation_transaction_sha256":
        "7ae03a18ae53852a0ab02369e130bfa8c63a9ba8f93b3b8dcc6bbce777533a8c",
    "override_manifest_sha256":
        "07154f8b88155421f46d9b706c3a5ce627271e87ae1aa689cca5b025c4b52ecb",
    "submission_intent_sha256":
        "e17bb18c2cfb36e53b7c3ec242c3e9db339c2ee321b9520a33a195fb7b1e46f4",
    "submission_receipt_sha256":
        "51443573534b25e4f4a0be14d3c8903caf65ef65d45f538af963d129a5aa3908",
    "report_sha256":
        "22b0b152ba964a6efae0953b1e36bf67dbf97565ad857d13a4db0fb9e6599a61",
    "report_bytes": 259179,
    "result_evidence": BEFORE_V4_RESULT_EVIDENCE,
    "stage_tree_sha256":
        "89032144b58339fe9d36b0ce81a8029f50207cb8b9b975fda53d2bb17987e68d",
    "stage_tree_files": 164,
    "stage_tree_directories": 19,
    "stage_tree_bytes": 5021882,
    "campaign_ledger_sha256":
        "09a46422ff808a6df3a2e2d84f7f7d10ecddcb5424bcbef53004e7aeea62b42d",
    "campaign_ledger_records": 20,
}
# The frozen after-v2 identity, retained for validate_after_v2_failure after
# PREDECESSOR advanced to the failed after-v3 stage.
AFTER_V2_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-after-v2"
)
AFTER_V2_PREDECESSOR = {
    "stage": AFTER_V2_PREDECESSOR_STAGE,
    "job": "3890580",
    "pin_sha256":
        "3820fd83e64a9a9b8503eedc379e3a4e9dc62a583ca778d232c358bd1940918a",
    "stage_creation_sha256":
        "e468bf226f31f35177703d4b5ed88ef230a03396a5be5dc94671d6bee94ec29b",
    "stage_creation_contract_sha256":
        "e72995b75215a0d5b6c10cab5ffb7d32cb2d3aeb140bc97dbfe1f825c53f36e8",
    "stage_creation_transaction_sha256":
        "8ce9e1e1e3f20b41757df2dae36e841a21a76d8f04ea9fe789fb7df0d53af6ba",
    "override_manifest_sha256":
        "5ceb77c83fa7de3223ecead1454669bd97344f4fdf8ffe51ceebe11efc043f8a",
    "submission_intent_sha256":
        "f5475d95d9c27aafaf501ca09b37d489f9dff6498730d97d402d842f4ded0b7b",
    "submission_receipt_sha256":
        "a012a125c1540762416d9f4c327dd80ab4a2a8266730687358de2942a2819987",
    "failure_evidence": AFTER_V2_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "587b1d490840d5845dfb28be67168471f20d46b3222b53b462d3e96bc2b25fb6",
    "stage_tree_files": 128,
    "stage_tree_directories": 17,
    "stage_tree_bytes": 4740691,
    "campaign_ledger_sha256":
        "c43f55dbe28cbdcee4ae35d0490842c282a4f6554f8a9973468ed8dfe53a79bd",
    "campaign_ledger_records": 21,
}
# The frozen after-v2 stray sbatch output binding, retained for
# validate_after_v2_failure after PREDECESSOR_STATE advanced to after-v3.
AFTER_V2_PREDECESSOR_OUT = {
    "sha256":
        "c50e274a20b95e488f8f7d03fd42f1d969ce8993aa6f5a8f87d6d0bf5e211f5d",
    "bytes": 847,
}
# The frozen after-v3 identity, retained for validate_after_v3_failure after
# PREDECESSOR advanced to the failed after-v4 stage.
AFTER_V3_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-after-v3"
)
AFTER_V3_PREDECESSOR = {
    "stage": AFTER_V3_PREDECESSOR_STAGE,
    "job": "3899303",
    "pin_sha256":
        "b7f683d27b21fa1f8b98d22f54445e7d8edb82b22ff78a7c89d8420dcc70b52c",
    "stage_creation_sha256":
        "387ecb1f71635488d9dd59d10f8d685120941807e9df768268d340d14b42a2bc",
    "stage_creation_contract_sha256":
        "073f72190eea8a6d2aca50f357ac6471c178b2838f83c620c1b8f8eeca787456",
    "stage_creation_transaction_sha256":
        "3ba2ac45cce29733f6dee6e255d92aad519af38a6a239d10914b69ea748fddc8",
    "override_manifest_sha256":
        "0d554771c461e058430aba8b432bbb9391101367e57ce8f25cb97adb147860ad",
    "submission_intent_sha256":
        "5e5ec63d66dd0cd050614d3a7b921f1aa5a887c09e8becae491333707663189a",
    "submission_receipt_sha256":
        "5dca4bdd283211b3b604222eb336490bbd3c1f566c372fac9595f69ebc0fe911",
    "report_sha256":
        "195cc4fca1d8848bd1617e71591518499dfaf0f951c8a232696f3f26ea776506",
    "report_bytes": 261764,
    "failure_evidence": AFTER_V3_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "0ab62cd025a5b06c51dd484936aa71ebbd63be97f0289ea2d20a25d0594cf023",
    "stage_tree_files": 169,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 4997261,
    "campaign_ledger_sha256":
        "97acd045285aea78cd081c525529d9b0a44b64270b38a11ffac3d480c86b3014",
    "campaign_ledger_records": 22,
}
# The frozen after-v3 sbatch output binding, retained for
# validate_after_v3_failure after PREDECESSOR_STATE advanced to after-v4.
AFTER_V3_PREDECESSOR_OUT = {
    "sha256":
        "509eb46e2dd6be6987203005e5eecec729529c388ce0c47366a602565f826289",
    "bytes": 132,
}
BEFORE_V3_PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-before-v3"
)
BEFORE_V3_PREDECESSOR_RECORD = {
    "stage": BEFORE_V3_PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_capture.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3884903",
    "pin_sha256":
        "a56d700e8b9aff4fea37ae6fc8f8281b9a3447befa20ce5ec4b6290a50857eb7",
    "stage_creation_sha256":
        "72884ef76fca402fee932098abefe4a8d9ce0e9593b6229c0cb514399aeee214",
}
BEFORE_V3_PREDECESSOR_CAMPAIGN_FILES = \
    BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES | {
    ".atlas-stage-creation-weyl-context-core-before-v3-prepared.json": {
        "sha256":
            "9f83056ae8c8306b02e12f8fc1c25a234d63b664e4019d1d0cb8bf826b153d72",
        "bytes": 19742, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v3-sealed.json": {
        "sha256":
            "94c907686adfb63384141d6953c0d3f20df7ce4d6562629ad28d037dd20e49fb",
        "bytes": 589, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v3-published.json": {
        "sha256":
            "1ab6c3d02d4cc97c9a039985ffbd7d657c8ba6bd65a57b2b8fae959bdbd78566",
        "bytes": 677, "mode": "0444", "nlink": 1,
    },
}
BEFORE_V3_PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v12",
    "stage": BEFORE_V3_PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669655310921,
    "stage_tree_sha256":
        "7d9603b2c4081c8ec620fe3530d292f43b8fe6782916749af19391280c44a60e",
    "stage_tree_files": 141,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3676351,
    "record": BEFORE_V3_PREDECESSOR_RECORD,
    "campaign_files": BEFORE_V3_PREDECESSOR_CAMPAIGN_FILES,
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "e6c02b947cbbce421b55f46a9a8632a22b5e2d947de8feb89d8284e6b90c3dec",
            "bytes": 327, "mode": "0444", "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "72884ef76fca402fee932098abefe4a8d9ce0e9593b6229c0cb514399aeee214",
            "bytes": 19980, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v3-pin.json": {
            "sha256":
                "a56d700e8b9aff4fea37ae6fc8f8281b9a3447befa20ce5ec4b6290a50857eb7",
            "bytes": 16823, "mode": "0444", "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "4623a603961033a33450271f153c47a3a5c54a957b3317fb7ff0995d83cba4f5",
            "bytes": 427, "mode": "0444", "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "e710a68d861ad74dc035f77cb578f70ed5ba9176a3d1a387b67c90844d031516",
            "bytes": 10719, "mode": "0444", "nlink": 1,
        },
        "overrides/overrides.json": {
            "sha256":
                "50687cdbc4b193d10accf3fe6d84705737ce8b54c635223a2126e4f480db5258",
            "bytes": 6540, "mode": "0444", "nlink": 1,
        },
        "results/3884903/test-stager-allowlist.stderr": {
            "sha256":
                "a5e4519e716935d2bd04f07c49f1e664bb4667cc530cd230b7d10ab8e22a8c92",
            "bytes": 2618, "mode": "0444", "nlink": 1,
        },
        "results/3884903/report.json": {
            "sha256":
                "7918ed04b11d2719b59916b6fe59d146bc6c622e43eac39c5c4f1643d6d571a1",
            "bytes": 52866, "mode": "0444", "nlink": 1,
        },
        "results/3884903/report.sha256": {
            "sha256":
                "08780ca688567992fb423ab9e0d0e99f96c384515bc9f6b29f057664997ac34d",
            "bytes": 65, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-before-v3-3884903.out": {
            "sha256":
                "899378373894315259c371585513fb5d175908d5eb32dac6c5f41fa7a844c386",
            "bytes": 134, "mode": "0644", "nlink": 1,
        },
    },
}
BEFORE_V3_PREDECESSOR = {
    "stage": BEFORE_V3_PREDECESSOR_STAGE,
    "job": "3884903",
    "pin_sha256":
        "a56d700e8b9aff4fea37ae6fc8f8281b9a3447befa20ce5ec4b6290a50857eb7",
    "stage_creation_sha256":
        "72884ef76fca402fee932098abefe4a8d9ce0e9593b6229c0cb514399aeee214",
    "stage_creation_contract_sha256":
        "5128c3758cc517f2c65353d51b52af9576057fd0a6d6a7cd0d476be49298f0c9",
    "stage_creation_transaction_sha256":
        "e6c02b947cbbce421b55f46a9a8632a22b5e2d947de8feb89d8284e6b90c3dec",
    "override_manifest_sha256":
        "50687cdbc4b193d10accf3fe6d84705737ce8b54c635223a2126e4f480db5258",
    "submission_intent_sha256":
        "4623a603961033a33450271f153c47a3a5c54a957b3317fb7ff0995d83cba4f5",
    "submission_receipt_sha256":
        "e710a68d861ad74dc035f77cb578f70ed5ba9176a3d1a387b67c90844d031516",
    "report_sha256":
        "7918ed04b11d2719b59916b6fe59d146bc6c622e43eac39c5c4f1643d6d571a1",
    "report_bytes": 52866,
    "failure_evidence": BEFORE_V3_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "7d9603b2c4081c8ec620fe3530d292f43b8fe6782916749af19391280c44a60e",
    "stage_tree_files": 141,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 3676351,
    "campaign_ledger_sha256":
        "fec706e87cd1a28c0700b55698a86b8c69553875b89d15274789065cf740e6b9",
    "campaign_ledger_records": 18,
}
BEFORE_V3_PREDECESSOR_REFERENCE = BEFORE_V3_PREDECESSOR
PREDECESSOR_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/"
    "stages/weyl-context-core-after-v4"
)
PREDECESSOR_RECORD = {
    "stage": PREDECESSOR_STAGE,
    "script": "hpc/math_weyl_context_core_after.sbatch",
    "queue_before": [],
    "status": "SUBMITTED",
    "max_outstanding": 10,
    "job": "3899885",
    "pin_sha256":
        "b1855bcab3c70cfb2a83943bd484de4c58e4b5266442b6c61b6a05bd0b825f25",
    "stage_creation_sha256":
        "1b3e32a192c6e2e7136d7ee7bc04f5aeb7c0b9d2df06077933832fa3c4829508",
}
PREDECESSOR_CAMPAIGN_FILES = BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES | {
    ".atlas-stage-creation-weyl-context-core-before-v3-prepared.json": {
        "sha256":
            "9f83056ae8c8306b02e12f8fc1c25a234d63b664e4019d1d0cb8bf826b153d72",
        "bytes": 19742, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v3-sealed.json": {
        "sha256":
            "94c907686adfb63384141d6953c0d3f20df7ce4d6562629ad28d037dd20e49fb",
        "bytes": 589, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v3-published.json": {
        "sha256":
            "1ab6c3d02d4cc97c9a039985ffbd7d657c8ba6bd65a57b2b8fae959bdbd78566",
        "bytes": 677, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v4-prepared.json": {
        "sha256":
            "eaa076df62b5e75d0755f8dec25fc6ce6cd3f0fdf0a19b0422a74b4eb946e7cb",
        "bytes": 20662, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v4-sealed.json": {
        "sha256":
            "227bf9aa28737e1b9c7004d0cf3f8213748025e18f8950742543afb9d03ef150",
        "bytes": 589, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-before-v4-published.json": {
        "sha256":
            "0dc3362461e91ce7329567f2c766047da4d9b1362d70fe8c6e62c703df22f657",
        "bytes": 677, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v1-prepared.json": {
        "sha256":
            "5fe6a718345f50db6b7e86a83f8c9c06ab095738b3c3617706d5304fbb382188",
        "bytes": 21720, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v1-sealed.json": {
        "sha256":
            "d43fe74789db4911d45dc72da2fca6d20f5781e33e55875dd407e423deb32714",
        "bytes": 588, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v1-published.json": {
        "sha256":
            "116b9adb696cf838a9a1e7e3842e0b2ce5b5a9f612b13b9b8dd5cc02831f1849",
        "bytes": 676, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v2-prepared.json": {
        "sha256":
            "343f764decf13700a8a268d1c7562702df8aba2b81f27c1494a9fc00a1241b38",
        "bytes": 22736, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v2-sealed.json": {
        "sha256":
            "2b32ea54de0daf93f4cd12f301e367cfa6fe5ff7596ce1cd91e17d2a52dea68c",
        "bytes": 588, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v2-published.json": {
        "sha256":
            "5ad45e1ba8950f87749ee18bf60bc63a22d877580fd5944aa2430183e3794c54",
        "bytes": 676, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v3-prepared.json": {
        "sha256":
            "e37fa59ac81ee654631c7b518c36a9d724d029a267e1b57cb8b9402fe635885a",
        "bytes": 23208, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v3-sealed.json": {
        "sha256":
            "06cc17ae8c0aea087d2201f965f79b28d6eef2c5b5411252e59732114a8ed0cf",
        "bytes": 588, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v3-published.json": {
        "sha256":
            "0d1599252a3414e89bc953b8aad17a1ccf78bd41aff55eab3af6c388195e6ca8",
        "bytes": 676, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v4-prepared.json": {
        "sha256":
            "b537c268fd183e21e411d2ec140b65b517ff2d7b582bf60acf4f455d03b3cabd",
        "bytes": 24264, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v4-sealed.json": {
        "sha256":
            "f6f2c3381485c86559b8f4109580c06e05964dd370c09b1f0e0497099e16b3b3",
        "bytes": 588, "mode": "0444", "nlink": 1,
    },
    ".atlas-stage-creation-weyl-context-core-after-v4-published.json": {
        "sha256":
            "660214d1d11eaf048838072a411876b13d06080573751822e5986a98db142c86",
        "bytes": 676, "mode": "0444", "nlink": 1,
    },
}
PREDECESSOR_STATE = {
    "schema": "atlas-stage-creation-predecessor-v14",
    "stage": PREDECESSOR_STAGE,
    "stage_device": 3431958692,
    "stage_inode": 162130669806415232,
    "stage_tree_sha256":
        "4d8b6518f49629af5456de6ececf65eab93159684457e2c473d043f49fc0a8a4",
    "stage_tree_files": 196,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 5186847,
    "record": PREDECESSOR_RECORD,
    "campaign_files": PREDECESSOR_CAMPAIGN_FILES,
    "stage_files": {
        ".atlas-stage-creation-transaction.json": {
            "sha256":
                "573e7edef9225a22ff23ebae32c67367723bec9ede6958c235abcf7653838856",
            "bytes": 327, "mode": "0444", "nlink": 1,
        },
        ".atlas-stage-creation.json": {
            "sha256":
                "1b3e32a192c6e2e7136d7ee7bc04f5aeb7c0b9d2df06077933832fa3c4829508",
            "bytes": 24501, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-after-v4-pin.json": {
            "sha256":
                "b1855bcab3c70cfb2a83943bd484de4c58e4b5266442b6c61b6a05bd0b825f25",
            "bytes": 18285, "mode": "0444", "nlink": 1,
        },
        "submission-intent.json": {
            "sha256":
                "46eef1c77208369f2a01346617fecca8f6e76bd2ec74b49fc9a3a55ee47c2d52",
            "bytes": 424, "mode": "0444", "nlink": 1,
        },
        "submission.json": {
            "sha256":
                "89345b025286d969e08e56e287666292e9d794bfca4fe15db927dc6dac8f8c0f",
            "bytes": 10976, "mode": "0444", "nlink": 1,
        },
        "overrides/overrides.json": {
            "sha256":
                "efcd05c46c28fc1778125eb673841318cb3dae1835982a60854f903956c23cf6",
            "bytes": 7664, "mode": "0444", "nlink": 1,
        },
        "weyl-context-core-after-v4-3899885.out": {
            "sha256":
                "3d23a0683925f8ee96733b89916d230b18292f83d99ab33994f2790754e13659",
            "bytes": 132, "mode": "0644", "nlink": 1,
        },
    },
}
PREDECESSOR = {
    "stage": PREDECESSOR_STAGE,
    "job": "3899885",
    "pin_sha256":
        "b1855bcab3c70cfb2a83943bd484de4c58e4b5266442b6c61b6a05bd0b825f25",
    "stage_creation_sha256":
        "1b3e32a192c6e2e7136d7ee7bc04f5aeb7c0b9d2df06077933832fa3c4829508",
    "stage_creation_contract_sha256":
        "f2145ec48160eac574754700c3e351183b02fa64435030734a66af005245614f",
    "stage_creation_transaction_sha256":
        "573e7edef9225a22ff23ebae32c67367723bec9ede6958c235abcf7653838856",
    "override_manifest_sha256":
        "efcd05c46c28fc1778125eb673841318cb3dae1835982a60854f903956c23cf6",
    "submission_intent_sha256":
        "46eef1c77208369f2a01346617fecca8f6e76bd2ec74b49fc9a3a55ee47c2d52",
    "submission_receipt_sha256":
        "89345b025286d969e08e56e287666292e9d794bfca4fe15db927dc6dac8f8c0f",
    "report_sha256":
        "479ddd84b1ddefc16a1f51e5d6fae5feda20405a84ae8b12feb6f418287f4467",
    "report_bytes": 352080,
    "failure_evidence": AFTER_V4_FAILURE_EVIDENCE,
    "stage_tree_sha256":
        "4d8b6518f49629af5456de6ececf65eab93159684457e2c473d043f49fc0a8a4",
    "stage_tree_files": 196,
    "stage_tree_directories": 18,
    "stage_tree_bytes": 5186847,
    "campaign_ledger_sha256":
        "02b5cb975a1fe15c9b5dd57ec3f0b424282f7b075dbf00967024505e8424e91e",
    "campaign_ledger_records": 23,
}
PREDECESSOR_REFERENCE = PREDECESSOR

PRIOR_CREATION_FAILURE_EVIDENCE = {
    "file": (
        "tests/reference/hpc/"
        "weyl_context_core_stage_creation_renameat2_failure_2026_10_02.json"
    ),
    "sha256":
        "74d4114f397c84286d0d9aba55ba88f12f4a0607ca93ebe3ef6eeba5911c7094",
}
PRIOR_CREATION_FAILURE = {
    "schema": "atlas-stage-creation-prior-failure-v1",
    "temporary": (
        ".atlas-publish-"
        "bd83ec4e79a16385851f0105ba5482376d0af5d360160beed6b23315cc1b6c1e.tmp"
    ),
    "archive": (
        ".atlas-stage-creation-failure-"
        "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5.json"
    ),
    "sha256":
        "1588813e1562c7011397876de836db22ac18a8e249da32ef3c5c3ed4e86e28d5",
    "bytes": 5481,
    "mode": "0444",
    "destination": ".atlas-stage-creation-prepared.json",
    "contract_sha256":
        "c046e43b744cadab9b4f682fa9283623a1589394950064b2e4634fe2998620d3",
    "transaction": ".atlas-stage-creation-c046e43b744cadab9b4f682f.txn",
}

PARENT_SEAL_REFERENCE = {
    "schema": "atlas-campaign-blob-v1",
    "role": "weyl-parent-seal",
    "sha256":
        "db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb",
    "bytes": 12488095,
}
PARENT_SOURCE_OBJECT = {
    "schema": "atlas-campaign-source-tar-v1",
    "role": "rust-source",
    "sha256":
        "5133bb32da7e5a92775d5f56ea2035680c363b40f085a8af95eaeebfdbc4536b",
    "bytes": 37672960,
    "files": 1558,
    "source_bytes": 36438334,
}
RETIRED_STAGER_BUNDLE_REFERENCE = {
    "schema": "atlas-campaign-blob-v1",
    "role": "retired-stager-bundle",
    "sha256":
        "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09",
    "bytes": 311925,
}

CURRENT_STAGER_PATH = "hpc/stage_weyl_context_core_after.py"
FROZEN_LAUNCHER_HASHES = {
    "hpc/stage_weyl_parent_seal.py":
        "f1c3567ab620269c3836bdc624b9573cc89d519944bfe00a7c24e859fe387fc4",
    "hpc/stage_ladder_boundary_before.py":
        "ff1f492adfa22f342bd0729f3df3885678de8f729a0f3d0775e5738c7ab47932",
    "hpc/math_ladder_boundary_before.py":
        "21dc2207690a8cd0559854eb78725c5e0d901a7e5c1c750d4feb89d55e7b8f9e",
    "hpc/stage_ladder_boundary_after.py":
        "07558c08a25f0d00b49720392b6abe469b2286d38b29cbb63a611cf40930d90d",
    "hpc/math_ladder_boundary_after.py":
        "b6f52b1042393705498767a687175f365c1fd0f491c3f630ca4cf53d2277d0a2",
    "hpc/stage_ladder_boundary_index.py":
        "0af4060de2bc5c451c0dab15b492d258c058d206bb06d151d37efcd57ec2008e",
    "hpc/math_ladder_boundary_index.py":
        "29eb6392458aa0d856ca88a1fba6d5eefc4e265f2e5ec5a5ba1ba41bdcf11c31",
    "hpc/stage_weyl_context_core_capture.py":
        "639b5c20d1a1efc68eebb3a240d9d2aa717c1c90757321c6b3445652fad649a6",
    "hpc/math_weyl_context_core_capture.py":
        "31f5aef26e14ca669ee9306efc70cc2c4f8f22fd19af11dab5e979a0296966d7",
    "hpc/math_weyl_context_core_after.py":
        "ab99c7a62b8c24dcba74cca2531f00f568cee17cb04702f602d59dbbabec0c3a",
}
FROZEN_LAUNCHER_STATES = {
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

PATCH_HASHES = {
    "hpc/patches/ladder_boundary_tests.patch":
        "69676d60b16591851570f58bc62ddc3bfe3a80b0ec6dad64a8eff36a9f777ca9",
    "hpc/patches/ladder_boundary_fix.patch":
        "cada2e341bfba80ee587acb966413ce8cb6297593d6394b93890e02cdd730e75",
}
TESTS_ONLY_HASHES = {
    "crates/atlas-core/src/session.rs":
        "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
    "crates/atlas-real-group/src/root_system.rs":
        "109076ec05cf25e532fc7a951af0cbf5a725efefb318d35c2172a4528fd999f7",
}
FINAL_HASHES = {
    "crates/atlas-core/src/session.rs":
        "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
    "crates/atlas-real-group/src/root_system.rs":
        "cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9",
}
SEALED_FIXTURE_HASHES = {
    "tests/math/generics/root_ladder_coordinate_boundary.atlas":
        "dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65",
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout":
        "3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80",
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr":
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
ACCEPTED_SOURCE = {
    "job": "3875239",
    "report": {
        "file": AFTER_REPORT_PATH,
        "sha256":
            "771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c",
    },
    "inspection": {
        "file": AFTER_INSPECTION_PATH,
        "sha256":
            "a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15",
    },
    "review": {
        "file": AFTER_REVIEW_PATH,
        "sha256":
            "3c0eed61cc5bf6af809096da73ac4bf1d8217b5cc81954a2e8000e5da44f9b58",
    },
    "source_manifest_sha256":
        "d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213",
    "parent_source_manifest_sha256":
        "b027eee2efe72d0f8a20848f99cd6368c29e3884ce9be16a2e7b5c63577dd0da",
    "parent_files": 1558,
    "final_files": 1561,
    "patches": PATCH_HASHES,
    "tests_only_hashes": TESTS_ONLY_HASHES,
    "final_hashes": FINAL_HASHES,
    "sealed_fixture_hashes": SEALED_FIXTURE_HASHES,
    "oracle": {
        "commit": "7e1b958c7aa9456769cc9cf09ac1542814b4800a",
        "binary_sha256":
            "d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a",
    },
}

REGRESSION_PATCH_PATH = "hpc/patches/weyl_context_core_regressions.patch"
REGRESSION_PATCH_HASHES = {
    REGRESSION_PATCH_PATH:
        "ccd3009892dbeae2f146dff9924efa6d6bede3dca910d026c8f3ab3e1b4c49cf",
}
REGRESSION_PATCH_BYTES = 3994
REPAIR_PATCH_PATH = "hpc/patches/weyl_context_core_repair.patch"
REPAIR_PATCH_HASHES = {
    REPAIR_PATCH_PATH:
        "1ad07e8fedf17162c8282c168d423b7f3dfcfb6e847b0d883b4fb8497dea59f7",
}
REPAIR_PATCH_BYTES = 30047
REPAIRED_SOURCE_HASHES = {
    "crates/atlas-core/src/domain_builtins.rs":
        "e6987e7cfc76674665184085cf639e2eb549ad9d7201bdb9c4f06419a39e6de5",
    "crates/atlas-core/src/typed.rs":
        "614975c5e2d49357b4d9ffff80145a6c69db2fd5aeef23f033faf03f594acc95",
    "crates/atlas-core/src/domain_builtins/weyl_subgroup.rs":
        "86d52b4f7228b300ad7ed0a38c526fd14f14003dfa034e4edf4f0e0102c63780",
}
AFTER_SOURCE_FILES = 1567
AFTER_SOURCE_MANIFEST_SHA256 = (
    "84a3fbfd977c61807fb9da5ef2295ed3057ab1e44c8466958f75e04d94ec78fd"
)
REGRESSION_SOURCE_HASHES = {
    "crates/atlas-core/src/session.rs":
        "969cdb27ccae61ca4fe3e36219801037d475517bee3614fead553024818307e7",
    "crates/atlas-real-group/src/root_system.rs":
        "cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9",
}
REGRESSION_FIXTURE_HASHES = {
    "tests/math/generics/weyl_context_core_cold_dual.atlas":
        "4eff8fa08f8490e242f07282125b83cde1daf24a875fe8df4dc7e73e75a0ae99",
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stdout":
        "7a614e47b75469c441e774cbe46769dcd769c5cc0a2a31cf1868ff9b59a780dc",
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stderr":
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "tests/math/generics/weyl_context_core_prewarmed_dual.atlas":
        "f13d704175f702b966790bf82e56c837dd80a594f0dc2dcd0d6b81ba281799a2",
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stdout":
        "5074aab3290d4ae404bb7abb0b24ccc99036f616abec518c6479e59ec2db021f",
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stderr":
        "ef4404d85f7252a611f9f4e4a5205fe6bbac2c66f48b7378a30a8053f986e157",
}
REGRESSION_FIXTURE_BYTES = {
    "tests/math/generics/weyl_context_core_cold_dual.atlas": 1815,
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stdout": 911,
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stderr": 0,
    "tests/math/generics/weyl_context_core_prewarmed_dual.atlas": 1275,
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stdout": 572,
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stderr": 501,
}
REGRESSION_SELECTOR_TESTS = [
    "session::tests::weyl_context_core_cold_dual_original",
    "session::tests::weyl_context_core_prewarmed_dual_original",
]
REGRESSION_RETAINED_CONTROL = (
    "session::tests::root_ladder_coordinate_boundary_original"
)
REGRESSION_EXPECTED_INVENTORY = 632
REGRESSION_SOURCE = {
    "parent_files": 1561,
    "parent_source_manifest_sha256":
        "d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213",
    "files": 1567,
    "source_manifest_sha256":
        "55f807cadb712377cbf6c0c79250b1d5e739e9757290a24209a086f89632850f",
}
REGRESSION_RECORD = {
    "catalog": {
        "file": REGRESSION_CATALOG_PATH,
        "sha256": REGRESSION_CATALOG_SHA256,
        "bytes": REGRESSION_CATALOG_BYTES,
    },
    "inspection": {
        "file": REGRESSION_INSPECTION_PATH,
        "sha256": REGRESSION_INSPECTION_SHA256,
        "bytes": REGRESSION_INSPECTION_BYTES,
    },
    "patches": REGRESSION_PATCH_HASHES,
    "source_hashes": REGRESSION_SOURCE_HASHES,
    "fixture_hashes": REGRESSION_FIXTURE_HASHES,
    "fixture_bytes": REGRESSION_FIXTURE_BYTES,
    "source": REGRESSION_SOURCE,
    "selector_tests": REGRESSION_SELECTOR_TESTS,
    "retained_control": REGRESSION_RETAINED_CONTROL,
    "expected_inventory": REGRESSION_EXPECTED_INVENTORY,
}

LIFECYCLE = {
    "stage": STAGE_NAME,
    "predecessor_stage": "weyl-context-core-after-v4",
    "changed_input_reasons": [
        (
            "Preserve FINAL FAILED job 3899885 as the immutable direct "
            "predecessor and bind its exact failure evidence, tree, "
            "twenty-three-record ledger, submission record and preserved "
            "report."
        ),
        (
            "Compare the prewarmed reject capture's stderr by ordered error "
            "summaries instead of raw bytes (the after-v4 failure): byte "
            "equality across the oracle's and the Rust CLI's diagnostic "
            "envelopes is unattainable; accept cases keep byte equality."
        ),
        (
            "Migrate the sbatch --job-name/--output labels to after-v5 so "
            "the sbatch output file matches this stage's "
            "SLURM_OUTPUT_PATTERN, pinned by the label checker regression."
        ),
        (
            "Keep the accepted 1561-file base, the 1567-file tests-only "
            "source, both fixtures, all four original-backed goldens, the "
            "completed repair patch and the retained ladder control exactly "
            "fixed."
        ),
        (
            "Rerun the same frozen original/Rust gate with complete "
            "GNU-time-v metrics, the 632-test inventory and retained ladder "
            "control; the two regressions must pass and the repaired Rust "
            "must match every frozen original golden under the reject "
            "case's error-summary comparison."
        ),
    ],
    "retention_class": "ACTIVE_GATE_COMPACT",
    "retirement_condition": (
        "FINAL independent inspection, focused commit and push, zero live or "
        "uncertain job ownership, verified campaign-CAS closure, and separate "
        "explicit exact-path deletion authorization."
    ),
}

STAGE_INPUT_NAMES = {
    "hpc/campaign_blob.py",
    "hpc/campaign_source.py",
    "hpc/campaign_workspace.py",
    "hpc/progressive_submit.py",
    "hpc/weyl_parent_seal.py",
    "hpc/weyl_context_core_contract.py",
    "hpc/test_weyl_context_core_contract.py",
    "hpc/weyl_context_core_regression.py",
    "hpc/test_weyl_context_core_regression.py",
    "hpc/stage_weyl_parent_seal.py",
    "hpc/stage_ladder_boundary_before.py",
    "hpc/stage_ladder_boundary_after.py",
    "hpc/stage_ladder_boundary_index.py",
    "hpc/math_ladder_boundary_before.py",
    "hpc/math_ladder_boundary_after.py",
    "hpc/math_ladder_boundary_index.py",
    "hpc/test_stager_allowlist.py",
    "hpc/test_campaign_stage_creation.py",
    "hpc/test_progressive_submit.py",
    "hpc/stage_weyl_context_core_capture.py",
    "hpc/math_weyl_context_core_capture.py",
    "hpc/math_weyl_context_core_capture.sbatch",
    "hpc/test_math_weyl_context_core_after.py",
    "hpc/stage_weyl_context_core_after.py",
    "hpc/math_weyl_context_core_after.py",
    "hpc/math_weyl_context_core_after.sbatch",
    "hpc/patches/ladder_boundary_tests.patch",
    "hpc/patches/ladder_boundary_fix.patch",
    "hpc/patches/weyl_context_core_repair.patch",
    REGRESSION_PATCH_PATH,
    CATALOG_PATH,
    REGRESSION_CATALOG_PATH,
    "tests/math/generics/weyl_context_core_cold_dual.atlas",
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stdout",
    "tests/math/generics/weyl_context_core_cold_dual.oracle.stderr",
    "tests/math/generics/weyl_context_core_prewarmed_dual.atlas",
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stdout",
    "tests/math/generics/weyl_context_core_prewarmed_dual.oracle.stderr",
    AFTER_REPORT_PATH,
    AFTER_INSPECTION_PATH,
    AFTER_REVIEW_PATH,
    ACCEPTANCE_INDEX_PATH,
    INDEX_REPORT_PATH,
    INDEX_INSPECTION_PATH,
    REGRESSION_INSPECTION_PATH,
    V8_SUBMISSION_EVIDENCE_PATH,
    (
        "tests/reference/hpc/"
        "weyl_context_core_stage_creation_renameat2_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v1_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v2_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v3_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v4_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v5_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v6_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_capture_v7_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v1_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v2_failure_2026_10_02.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_before_v3_failure_2026_10_02.json"
    ),
    "tests/reference/hpc/math_weyl_context_core_before_v4_2026_10_02.json",
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v1_failure_2026_10_03.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v2_failure_2026_10_03.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v3_failure_2026_10_06.json"
    ),
    (
        "tests/reference/hpc/"
        "math_weyl_context_core_after_v4_failure_2026_10_06.json"
    ),
}

SUBMISSION_RECORD_KEYS = {
    "stage", "script", "queue_before", "status", "max_outstanding", "job",
    "pin_sha256", "stage_creation_sha256",
}
PIN_KEYS = {
    "schema", "inputs", "overrides_sha256", "test_counts", "predecessor",
    "accepted_source", "parent_seal_object", "retired_stager_object",
    "catalog", "regression", "lifecycle", "scope", "stage_creation",
}
SHA256_PATTERN = r"[0-9a-f]{64}\Z"
FORBIDDEN_DURABLE_STAGE_NAMES = (
    "source", "target", "workspace", "build",
)
ALLOWED_STAGE_DIRECTORIES = (
    ".incoming", "hpc", "overrides", "results", "tests",
)
ALLOWED_STAGE_FILES = (
    STAGE_CREATION_MARKER, STAGE_CREATION_RECEIPT, STAGE_LOCK, PIN_NAME,
    "submission-intent.json", "submission.json",
)
SLURM_OUTPUT_PATTERN = r"weyl-context-core-after-v5-[0-9]+\.out\Z"


def strict_json_loads(raw):
    """Decode JSON while rejecting duplicate keys and non-finite numbers."""
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate JSON key: " + key)
            result[key] = value
        return result

    def reject_constant(value):
        raise ValueError("non-finite JSON value: " + value)

    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=reject_constant)


def saved_json_sha(value):
    raw = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def canonical_json_sha(value):
    raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def validate_test_counts(value):
    if (not isinstance(value, dict)
            or value != EXPECTED_TEST_COUNTS
            or any(type(count) is not int or count < 1
                   for count in value.values())):
        raise ValueError("Weyl core capture checker counts are not frozen")
    return copy.deepcopy(value)


def require_enabled_launcher():
    if SUBMISSION_ENABLED is not True:
        raise ValueError("Weyl core capture submission remains disabled")
    return validate_test_counts(EXPECTED_TEST_COUNTS)


def _safe_relative(name):
    if not isinstance(name, str) or not name:
        raise ValueError("unsafe stage-relative input path")
    pure = PurePosixPath(name)
    if (pure.is_absolute() or pure.as_posix() != name
            or any(part in ("", ".", "..") for part in pure.parts)):
        raise ValueError("unsafe stage-relative input path")
    return pure.parts


def _directory_flags():
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory is None:
        raise ValueError("safe no-follow directory traversal is unavailable")
    return os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow | directory


def _open_directory(path):
    try:
        descriptor = os.open(path, _directory_flags())
    except OSError as error:
        raise ValueError("required stage directory is missing or unsafe") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("required stage path is not a directory")
    return descriptor


def _open_child_directory(parent, name, create=False):
    if not isinstance(name, str) or name in ("", ".", "..") or "/" in name:
        raise ValueError("unsafe stage directory component")
    if create:
        try:
            os.mkdir(name, 0o755, dir_fd=parent)
        except FileExistsError:
            pass
        # An existing child may be the visible result of an interrupted mkdir
        # whose parent-directory sync never completed.  Replay that durability
        # step before using either a new or recovered directory component.
        os.fsync(parent)
    try:
        descriptor = os.open(name, _directory_flags(), dir_fd=parent)
    except OSError as error:
        raise ValueError("stage directory component is missing or unsafe") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("stage directory component is not a directory")
    return descriptor


def _validate_results_directory(parent):
    descriptor = _open_child_directory(parent, "results")
    try:
        names = sorted(os.listdir(descriptor))
        if len(names) > 1:
            raise ValueError("capture stage has more than one result owner")
        for name in names:
            if not name.isdecimal():
                raise ValueError("capture result directory is not a job id")
            value = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
            if not stat.S_ISDIR(value.st_mode):
                raise ValueError("capture result owner is not a real directory")
    except OSError as error:
        raise ValueError("capture results topology is unsafe") from error
    finally:
        os.close(descriptor)


def validate_stage_topology(root):
    """Reject every durable stage-root path outside the fixed lifecycle."""
    descriptor = _open_directory(root)
    try:
        try:
            names = sorted(os.listdir(descriptor))
        except OSError as error:
            raise ValueError("capture stage topology could not be listed") from error
        slurm_outputs = 0
        for name in names:
            if any(
                    name == stem
                    or name.startswith(stem + "-")
                    or name.startswith(stem + "_")
                    or name.startswith(stem + ".")
                    for stem in FORBIDDEN_DURABLE_STAGE_NAMES):
                raise ValueError("durable source or build tree is forbidden")
            try:
                value = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
            except OSError as error:
                raise ValueError("capture stage entry could not be inspected") from error
            if stat.S_ISDIR(value.st_mode):
                if name not in ALLOWED_STAGE_DIRECTORIES:
                    raise ValueError("unexpected durable capture-stage directory")
                if name == ".incoming":
                    incoming = _open_child_directory(descriptor, name)
                    try:
                        if os.listdir(incoming):
                            raise ValueError(
                                "capture publication scratch is not empty")
                    finally:
                        os.close(incoming)
                elif name == "results":
                    _validate_results_directory(descriptor)
                continue
            if not stat.S_ISREG(value.st_mode) or value.st_nlink != 1:
                raise ValueError("capture stage entry is not a single-link file")
            if name in ALLOWED_STAGE_FILES:
                continue
            if re.fullmatch(SLURM_OUTPUT_PATTERN, name) is not None:
                slurm_outputs += 1
                if slurm_outputs > 1:
                    raise ValueError("capture stage has more than one SLURM output")
                continue
            raise ValueError("unexpected durable capture-stage file")
        return tuple(names)
    finally:
        os.close(descriptor)


@contextmanager
def _open_parent(root, name, create=False):
    descriptors = []
    try:
        current = _open_directory(root)
        descriptors.append(current)
        parts = _safe_relative(name)
        for part in parts[:-1]:
            current = _open_child_directory(current, part, create=create)
            descriptors.append(current)
        yield current, parts[-1]
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def stable_relative_bytes(root, name, *, mode=None, nlink=None):
    """Read one stable regular file without following any path component."""
    descriptor = current_descriptor = None
    try:
        with _open_parent(root, name) as (parent, leaf):
            flags = (os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
                     | getattr(os, "O_NOFOLLOW", 0))
            descriptor = os.open(leaf, flags, dir_fd=parent)
            before = os.fstat(descriptor)
            if (not stat.S_ISREG(before.st_mode)
                    or (mode is not None
                        and stat.S_IMODE(before.st_mode) != mode)
                    or (nlink is not None and before.st_nlink != nlink)):
                raise ValueError("stage input is not the required regular file")
            chunks = []
            while True:
                block = os.read(descriptor, 64 * 1024)
                if not block:
                    break
                chunks.append(block)
            after = os.fstat(descriptor)
            current_descriptor = os.open(leaf, flags, dir_fd=parent)
            current = os.fstat(current_descriptor)
        identity = lambda value: (
            value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
            value.st_ctime_ns, stat.S_IFMT(value.st_mode), value.st_nlink,
        )
        raw = b"".join(chunks)
        if (identity(before) != identity(after)
                or identity(before) != identity(current)
                or len(raw) != before.st_size):
            raise ValueError("stage input changed while it was read")
        return raw
    except OSError as error:
        raise ValueError("stage input could not be read safely") from error
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)


def load_relative_json(root, name, wanted, *, mode=None, nlink=None):
    raw = stable_relative_bytes(root, name, mode=mode, nlink=nlink)
    if hashlib.sha256(raw).hexdigest() != wanted:
        raise ValueError("changed JSON stage input: " + name)
    try:
        value = strict_json_loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON stage input: " + name) from error
    return value


def frozen_stage_inputs(root):
    """Require exact hpc/tests file and directory topology plus frozen modes."""
    root = Path(root)
    for namespace in ("hpc", "tests"):
        prefix = namespace + "/"
        expected = {
            name[len(prefix):] for name in STAGE_INPUT_NAMES
            if name.startswith(prefix)
        }
        expected_directories = set()
        for name in expected:
            for parent in PurePosixPath(name).parents:
                if parent.as_posix() != ".":
                    expected_directories.add(parent.as_posix())
        namespace_root = root / namespace
        manifest = file_manifest(namespace_root)
        directories = set()
        for path in namespace_root.rglob("*"):
            value = path.lstat()
            if stat.S_ISDIR(value.st_mode):
                directories.add(str(path.relative_to(namespace_root)))
        if set(manifest) != expected or directories != expected_directories:
            raise ValueError(
                "installed " + namespace + " namespace differs from the pin")
    result = {}
    for name in sorted(STAGE_INPUT_NAMES):
        raw = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        result[name] = hashlib.sha256(raw).hexdigest()
    return result


def _write_all(descriptor, raw):
    remaining = memoryview(raw)
    while remaining:
        written = os.write(descriptor, remaining)
        if written <= 0:
            raise ValueError("short stage-input write")
        remaining = remaining[written:]


def _atomic_install(root, incoming, name, raw, wanted):
    """Install one immutable single-link input with held directory handles."""
    with _open_parent(root, name, create=True) as (destination, leaf):
        try:
            present = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        except ValueError:
            present = None
        if present is not None:
            if hashlib.sha256(present).hexdigest() != wanted:
                raise ValueError("installed stage input changed: " + name)
            # The prior link may have become visible before its destination
            # directory was synced.  Revalidate bytes, then replay that sync.
            os.fsync(destination)
            return

        incoming_descriptor = _open_directory(incoming)
        temporary_name = ".copy-" + secrets.token_hex(16)
        descriptor = None
        linked = False
        try:
            flags = (os.O_RDWR | os.O_CREAT | os.O_EXCL
                     | getattr(os, "O_CLOEXEC", 0)
                     | getattr(os, "O_NOFOLLOW", 0))
            descriptor = os.open(
                temporary_name, flags, 0o600, dir_fd=incoming_descriptor)
            _write_all(descriptor, raw)
            os.fchmod(descriptor, 0o444)
            os.fsync(descriptor)
            if hashlib.sha256(raw).hexdigest() != wanted:
                raise ValueError("override bytes changed before publication")
            try:
                os.link(
                    temporary_name, leaf,
                    src_dir_fd=incoming_descriptor,
                    dst_dir_fd=destination,
                    follow_symlinks=False,
                )
                linked = True
                os.fsync(destination)
            except FileExistsError:
                present = stable_relative_bytes(
                    root, name, mode=0o444, nlink=1)
                if hashlib.sha256(present).hexdigest() != wanted:
                    raise ValueError("stage input appeared with changed bytes")
                os.fsync(destination)
        finally:
            if descriptor is not None:
                os.close(descriptor)
            try:
                os.unlink(temporary_name, dir_fd=incoming_descriptor)
            except FileNotFoundError:
                pass
            os.fsync(incoming_descriptor)
            os.close(incoming_descriptor)
        if linked:
            final = stable_relative_bytes(root, name, mode=0o444, nlink=1)
            if hashlib.sha256(final).hexdigest() != wanted:
                raise ValueError("published stage input changed: " + name)


def _prepare_incoming(root):
    root_descriptor = _open_directory(root)
    try:
        incoming_descriptor = _open_child_directory(
            root_descriptor, ".incoming", create=True)
        try:
            if os.listdir(incoming_descriptor):
                raise ValueError(
                    "nonempty publication scratch requires manual reconciliation")
        finally:
            os.close(incoming_descriptor)
    finally:
        os.close(root_descriptor)
    return Path(root) / ".incoming"


def install_inputs(root, overrides):
    incoming = _prepare_incoming(root)
    override_root = Path(root) / "overrides"
    for name in sorted(STAGE_INPUT_NAMES):
        raw = stable_relative_bytes(
            override_root, name, mode=0o444, nlink=1)
        if hashlib.sha256(raw).hexdigest() != overrides[name]:
            raise ValueError("changed frozen override: " + name)
        _atomic_install(root, incoming, name, raw, overrides[name])
    if any(incoming.iterdir()):
        raise ValueError("publication scratch is not empty after installation")


def _source_manifest(report):
    manifest = report.get("source_files") if isinstance(report, dict) else None
    if (not isinstance(manifest, dict)
            or len(manifest) != ACCEPTED_SOURCE["final_files"]
            or canonical_json_sha(manifest)
               != ACCEPTED_SOURCE["source_manifest_sha256"]
            or any(not isinstance(name, str)
                   or not isinstance(sha, str)
                   or re.fullmatch(SHA256_PATTERN, sha) is None
                   for name, sha in manifest.items())):
        raise ValueError("accepted post-ladder source manifest changed")
    return manifest


def validate_predecessor(root, inputs):
    """Bind the completed index stage and the accepted source it published."""
    expected_hashes = {
        AFTER_REPORT_PATH: ACCEPTED_SOURCE["report"]["sha256"],
        AFTER_INSPECTION_PATH: ACCEPTED_SOURCE["inspection"]["sha256"],
        AFTER_REVIEW_PATH: ACCEPTED_SOURCE["review"]["sha256"],
        ACCEPTANCE_INDEX_PATH:
            ACCEPTED_INDEX_PREDECESSOR["acceptance_index_sha256"],
        INDEX_REPORT_PATH: ACCEPTED_INDEX_PREDECESSOR["report"]["sha256"],
        INDEX_INSPECTION_PATH:
            ACCEPTED_INDEX_PREDECESSOR["inspection"]["sha256"],
    }
    if any(inputs.get(name) != wanted
           for name, wanted in expected_hashes.items()):
        raise ValueError("Weyl capture predecessor evidence hash changed")
    values = {
        name: load_relative_json(root, name, wanted)
        for name, wanted in expected_hashes.items()
    }
    after = values[AFTER_REPORT_PATH]
    inspection = values[AFTER_INSPECTION_PATH]
    review = values[AFTER_REVIEW_PATH]
    index = values[ACCEPTANCE_INDEX_PATH]
    index_report = values[INDEX_REPORT_PATH]
    index_inspection = values[INDEX_INSPECTION_PATH]
    manifest = _source_manifest(after)
    if (after.get("schema") != "atlas-ladder-boundary-after-v3"
            or after.get("status") != "LADDER_BOUNDARY_AFTER_GATES_PASS"
            or after.get("job") != ACCEPTED_SOURCE["job"]
            or after.get("integrity_rechecked") is not True
            or after.get("source_integrity_rechecked") is not True
            or after.get("parent_seal_object") != PARENT_SEAL_REFERENCE
            or after.get("patch_hashes") != PATCH_HASHES
            or after.get("tests_only_source_hashes") != TESTS_ONLY_HASHES
            or any(manifest.get(name) != sha
                   for name, sha in {**FINAL_HASHES,
                                      **SEALED_FIXTURE_HASHES}.items())):
        raise ValueError("accepted post-ladder execution report changed")
    if (inspection.get("schema")
            != "atlas-ladder-boundary-after-inspection-v3"
            or inspection.get("status") != "LADDER_BOUNDARY_AFTER_ACCEPTED"
            or inspection.get("accounting", {}).get("job")
               != ACCEPTED_SOURCE["job"]
            or inspection.get("artifacts", {}).get("report", {}).get(
                "file_sha256") != ACCEPTED_SOURCE["report"]["sha256"]
            or inspection.get("artifacts", {}).get("source_object_sha256")
               != PARENT_SOURCE_OBJECT["sha256"]):
        raise ValueError("accepted post-ladder inspection changed")
    if (review.get("schema") != "atlas-root-ladder-acceptance-review-v1"
            or review.get("status")
               != "ROOT_LADDER_COORDINATE_BOUNDARY_REVIEWED"
            or review.get("job") != ACCEPTED_SOURCE["job"]
            or review.get("execution_report") != ACCEPTED_SOURCE["report"]
            or review.get("independent_inspection")
               != ACCEPTED_SOURCE["inspection"]
            or review.get("source_manifest_sha256")
               != ACCEPTED_SOURCE["source_manifest_sha256"]):
        raise ValueError("accepted post-ladder independent review changed")
    entries = index.get("entries") if isinstance(index, dict) else None
    if (index.get("schema") != "atlas-math-acceptance-index-v1"
            or not isinstance(entries, list) or len(entries) != 3
            or entries[-1].get("entry_sha256")
               != ACCEPTED_INDEX_PREDECESSOR[
                   "acceptance_index_head_sha256"]
            or entries[-1].get("claim_id")
               != "a1_torus_root_coroot_ladder_coordinate_boundary"
            or entries[-1].get("acceptance") != "accepted"
            or entries[-1].get("status") != "math_pass"
            or entries[-1].get("report") != ACCEPTED_SOURCE["report"]
            or entries[-1].get("review_evidence")
               != ACCEPTED_SOURCE["review"]):
        raise ValueError("accepted-source index tail changed")
    receipt = index_report.get("submission_receipt", {})
    if (index_report.get("schema") != "atlas-ladder-boundary-index-v1"
            or index_report.get("status")
               != "LADDER_BOUNDARY_INDEX_GATES_PASS"
            or index_report.get("job") != ACCEPTED_INDEX_PREDECESSOR["job"]
            or index_report.get("pin_sha256")
               != ACCEPTED_INDEX_PREDECESSOR["pin_sha256"]
            or index_report.get("source_integrity_rechecked") is not True
            or receipt.get("job") != ACCEPTED_INDEX_PREDECESSOR["job"]
            or receipt.get("index", {}).get("head_sha256")
               != ACCEPTED_INDEX_PREDECESSOR[
                   "acceptance_index_head_sha256"]):
        raise ValueError("acceptance-index publication report changed")
    artifacts = index_inspection.get("artifacts", {})
    if (index_inspection.get("schema")
            != "atlas-ladder-boundary-index-inspection-v1"
            or index_inspection.get("status")
               != "LADDER_BOUNDARY_INDEX_ACCEPTED"
            or index_inspection.get("accounting", {}).get("job")
               != ACCEPTED_INDEX_PREDECESSOR["job"]
            or index_inspection.get("accounting", {}).get("state")
               != "COMPLETED"
            or index_inspection.get("accounting", {}).get("exit_code") != "0:0"
            or artifacts.get("pin_sha256")
               != ACCEPTED_INDEX_PREDECESSOR["pin_sha256"]
            or artifacts.get("submission_receipt_sha256")
               != ACCEPTED_INDEX_PREDECESSOR["receipt_sha256"]
            or artifacts.get("campaign_ledger_sha256")
               != ACCEPTED_INDEX_PREDECESSOR["campaign_ledger_sha256"]
            or artifacts.get("report", {}).get("sha256")
               != ACCEPTED_INDEX_PREDECESSOR["report"]["sha256"]):
        raise ValueError("acceptance-index independent inspection changed")
    return copy.deepcopy(manifest)


def validate_prior_creation_failure(root, inputs):
    """Bind the exact failed renameat2 attempt and its preserved residual."""
    reference = PRIOR_CREATION_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("stage-creation failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    invocation = value.get("invocation") if isinstance(value, dict) else None
    remote = value.get("post_failure_remote_state") \
        if isinstance(value, dict) else None
    residual = remote.get("prepared_publication_residual") \
        if isinstance(remote, dict) else None
    probe = remote.get("same_directory_hardlink_probe") \
        if isinstance(remote, dict) else None
    expected_residual = {
        "path": (
            str(Path(PREDECESSOR_STAGE).parent.parent)
            + "/" + PRIOR_CREATION_FAILURE["temporary"]
        ),
        "mode": PRIOR_CREATION_FAILURE["mode"],
        "bytes": PRIOR_CREATION_FAILURE["bytes"],
        "sha256": PRIOR_CREATION_FAILURE["sha256"],
        "destination": PRIOR_CREATION_FAILURE["destination"],
        "contract_sha256": PRIOR_CREATION_FAILURE["contract_sha256"],
        "transaction": PRIOR_CREATION_FAILURE["transaction"],
        "status": "PRESERVED_FOR_EXACT_RECONCILIATION",
    }
    if (value.get("schema")
            != "atlas-weyl-context-core-stage-creation-failure-v1"
            or value.get("classification")
               != "PLATFORM_COMPATIBILITY_FAILURE_BEFORE_HPC_CHECKER_OR_MATHEMATICAL_EXECUTION"
            or not isinstance(invocation, dict)
            or invocation.get("exit_status") != 1
            or invocation.get("exception")
               != "ValueError: stage publication refused by renameat2: Invalid argument"
            or invocation.get("failure_symbol")
               != "progressive_submit._rename_noreplace"
            or not isinstance(remote, dict)
            or remote.get("expanded_queue_jobs") != 0
            or remote.get("campaign_ledger_sha256")
               != ACCEPTED_INDEX_PREDECESSOR["campaign_ledger_sha256"]
            or remote.get("campaign_ledger_records") != 7
            or remote.get("target_stage_present") is not False
            or remote.get("hidden_stage_transaction_present") is not False
            or remote.get("scheduler_contact_reached") is not False
            or remote.get("submission_intent_created") is not False
            or remote.get("submission_receipt_created") is not False
            or residual != expected_residual
            or probe != {
                "path": "/public/home/majj/.wcc-rename-probe.41esTq",
                "source_and_link_same_device_and_inode": True,
                "link_count": 2,
                "mode": "0444",
                "result": "SUPPORTED",
                "path_removed_after_probe": True,
            }):
        raise ValueError("stage-creation failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v1_failure(root, inputs):
    """Bind the immutable v1 checker failure and its exact campaign state."""
    reference = V1_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v1 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    submission = value.get("submission") if isinstance(value, dict) else None
    creation = value.get("stage_creation") \
        if isinstance(value, dict) else None
    report = value.get("report") if isinstance(value, dict) else None
    gate = value.get("failed_gate") if isinstance(value, dict) else None
    causes = gate.get("errors_by_cause") if isinstance(gate, dict) else None
    not_reached = value.get("not_reached") if isinstance(value, dict) else None
    archive_name = next(
        name for name in V1_PREDECESSOR_STATE["campaign_files"]
        if name.startswith(".atlas-stage-creation-failure-")
    )
    if (value.get("schema")
            != "atlas-weyl-context-core-capture-failure-v1"
            or value.get("classification")
               != "HARNESS_FAILURE_BEFORE_BUILD_OR_MATHEMATICAL_EXECUTION"
            or value.get("campaign")
               != str(Path(V1_PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != V1_PREDECESSOR_STAGE
            or not isinstance(submission, dict)
            or submission.get("job") != V1_PREDECESSOR["job"]
            or submission.get("state") != "FAILED"
            or submission.get("exit_code") != "1:0"
            or submission.get("ledger_records_after")
               != V1_PREDECESSOR["campaign_ledger_records"]
            or submission.get("ledger_sha256_after")
               != V1_PREDECESSOR["campaign_ledger_sha256"]
            or submission.get("pin_sha256")
               != V1_PREDECESSOR["pin_sha256"]
            or submission.get("stage_creation_sha256")
               != V1_PREDECESSOR["stage_creation_sha256"]
            or submission.get("submission_receipt_sha256")
               != V1_PREDECESSOR["submission_receipt_sha256"]
            or not isinstance(creation, dict)
            or creation.get("prepared_sha256")
               != V1_PREDECESSOR_STATE["campaign_files"][
                   ".atlas-stage-creation-prepared.json"]["sha256"]
            or creation.get("sealed_sha256")
               != V1_PREDECESSOR_STATE["campaign_files"][
                   ".atlas-stage-creation-sealed.json"]["sha256"]
            or creation.get("published_sha256")
               != V1_PREDECESSOR_STATE["campaign_files"][
                   ".atlas-stage-creation-published.json"]["sha256"]
            or creation.get("target_device")
               != V1_PREDECESSOR_STATE["stage_device"]
            or creation.get("target_inode")
               != V1_PREDECESSOR_STATE["stage_inode"]
            or creation.get("prior_renameat2_residual_consumed") is not True
            or creation.get("prior_failure_archive", {}).get("path")
               != str(Path(V1_PREDECESSOR_STAGE).parent.parent / archive_name)
            or creation.get("prior_failure_archive", {}).get("sha256")
               != V1_PREDECESSOR_STATE["campaign_files"][archive_name]["sha256"]
            or creation.get("publication_residuals_after") != []
            or not isinstance(report, dict)
            or report.get("sha256") != V1_PREDECESSOR["report_sha256"]
            or report.get("status") != "HARNESS_FAILURE"
            or report.get("complete") is not False
            or report.get("acceptance_eligible") is not False
            or report.get("math_gate_released") is not False
            or report.get("cache_gate_released") is not False
            or report.get("commands_recorded") != 0
            or report.get("atlas_invocations") != 0
            or report.get("captures_recorded") != 0
            or not isinstance(gate, dict)
            or gate.get("name") != "test-campaign-stage-creation"
            or gate.get("expected_tests") != 29
            or gate.get("observed_tests") != 29
            or gate.get("passed") != 25
            or gate.get("errors") != 4
            or not isinstance(causes, dict)
            or sorted(causes.get("controlled_environment_has_no_USER", []))
               != sorted([
                   "test_intent_racer_is_preserved_and_never_reaches_sbatch",
                   "test_lock_name_swap_after_squeue_never_reaches_sbatch",
                   "test_uncertain_submission_retains_creation_sha_everywhere",
               ])
            or causes.get("final_validation_left_synthetic_campaign_context")
               != [
                   "test_real_published_stage_reaches_sbatch_after_durable_intent"
               ]
            or not isinstance(not_reached, list)
            or "Cargo release build" not in not_reached
            or "mathematical comparison" not in not_reached
            or value.get("claims_not_granted") != [
                "creator checker acceptance",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v1 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v2_failure(root, inputs):
    """Bind the immutable v2 checker failure before any Atlas execution."""
    reference = V2_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v2 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    submission = value.get("submission") if isinstance(value, dict) else None
    creation = value.get("stage_creation") \
        if isinstance(value, dict) else None
    report = value.get("report") if isinstance(value, dict) else None
    passed = value.get("passed_gates") if isinstance(value, dict) else None
    gate = value.get("failed_gate") if isinstance(value, dict) else None
    causes = gate.get("errors_by_cause") if isinstance(gate, dict) else None
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v2-"
    expected_submission = {
        "job": V2_PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 114,
        "node": "cu006",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 83356,
        "queue_before": [],
        "ledger_records_after": V2_PREDECESSOR["campaign_ledger_records"],
        "ledger_sha256_after": V2_PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": V2_PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": V2_PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            V2_PREDECESSOR["stage_creation_contract_sha256"],
        "submission_intent_sha256":
            V2_PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            V2_PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256":
            V2_PREDECESSOR_STATE["campaign_files"][
                scoped + "prepared.json"]["sha256"],
        "sealed_sha256":
            V2_PREDECESSOR_STATE["campaign_files"][
                scoped + "sealed.json"]["sha256"],
        "published_sha256":
            V2_PREDECESSOR_STATE["campaign_files"][
                scoped + "published.json"]["sha256"],
        "transaction_marker_sha256":
            V2_PREDECESSOR_STATE["stage_files"][
                ".atlas-stage-creation-transaction.json"]["sha256"],
        "target_device": V2_PREDECESSOR_STATE["stage_device"],
        "target_inode": V2_PREDECESSOR_STATE["stage_inode"],
        "publication_residuals_after": [],
    }
    expected_passed = [
        {
            "name": "test-campaign-stage-creation",
            "tests": 32,
            "seconds": 53.95,
            "maxrss_kb": 27456,
            "stderr_sha256":
                "36f3db49e1b721b31aa9869452d0e8898e6fcdd2051dacc3f91998f5f06d0168",
        },
        {
            "name": "test-progressive-submit",
            "tests": 17,
            "seconds": 1.92,
            "maxrss_kb": 24564,
            "stderr_sha256":
                "5bdd5f5fb48917e3cfa67f56836c4b3768aa892bd641e695e44f4a2191f387c2",
        },
        {
            "name": "test-weyl-context-core-contract",
            "tests": 18,
            "seconds": 1.19,
            "maxrss_kb": 19676,
            "stderr_sha256":
                "9faace8f7eae2df8d428884fcbf85ae240ceaf8cc8640c7e6ceb94e5fee008cd",
        },
    ]
    expected_causes = {
        "controlled_environment_values_were_mistaken_for_inherited_values": [
            "test_command_environment_uses_only_ephemeral_target_and_scrubs_inheritance"
        ],
        "build_order_assertion_searched_main_for_top_level_cargo_literal": [
            "test_driver_materializes_source_and_target_only_in_ephemeral_workspace"
        ],
        "stager_order_assertion_did_not_skip_function_docstring": [
            "test_stager_submission_is_prerequisite_bound_single_job_and_fail_closed"
        ],
    }
    expected_not_reached = [
        "test-stager-allowlist",
        "toolchain identity",
        "source reconstruction",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    if (value.get("schema")
            != "atlas-weyl-context-core-capture-failure-v2"
            or value.get("classification")
               != "HARNESS_FAILURE_BEFORE_BUILD_OR_MATHEMATICAL_EXECUTION"
            or value.get("campaign")
               != str(Path(V2_PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != V2_PREDECESSOR_STAGE
            or submission != expected_submission
            or creation != expected_creation
            or not isinstance(report, dict)
            or report.get("path")
               != str(Path(V2_PREDECESSOR_STAGE) / "results/3884371/report.json")
            or report.get("bytes") != 32873
            or report.get("sha256") != V2_PREDECESSOR["report_sha256"]
            or report.get("status") != "HARNESS_FAILURE"
            or report.get("complete") is not False
            or report.get("acceptance_eligible") is not False
            or report.get("math_gate_released") is not False
            or report.get("cache_gate_released") is not False
            or report.get("ephemeral_workspace_removed") is not True
            or report.get("commands_recorded") != 3
            or report.get("atlas_invocations") != 0
            or report.get("captures_recorded") != 0
            or report.get("legacy_path_open_attempts") != []
            or passed != expected_passed
            or not isinstance(gate, dict)
            or gate.get("name") != "test-math-weyl-context-core-capture"
            or gate.get("expected_tests") != 27
            or gate.get("observed_tests") != 27
            or gate.get("passed") != 24
            or gate.get("failures") != 2
            or gate.get("errors") != 1
            or gate.get("seconds") != 47.72
            or gate.get("stderr_sha256")
               != "60e910f4eece138732e13cc3a5847da588193c2baacdcf33631b994999baf83b"
            or gate.get("stdout_sha256")
               != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            or gate.get("time_sha256")
               != "733f23cb232ac7e55d3be38e9dbd397ed625927739eb5b917014009b385fc6a8"
            or causes != expected_causes
            or value.get("not_reached") != expected_not_reached
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no mathematical "
                   "result was produced; the three failures are test-harness "
                   "assertions.")
            or value.get("remediation_contract", {}).get(
                "immutable_failed_stage") != "weyl-context-core-capture-v2"
            or value.get("remediation_contract", {}).get(
                "changed_input_successor_required") is not True
            or value.get("remediation_contract", {}).get(
                "no_same_stage_resubmission") is not True
            or value.get("remediation_contract", {}).get(
                "no_rank_escalation") is not True
            or value.get("claims_not_granted") != [
                "complete checker acceptance",
                "source reconstruction",
                "Cargo build",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v2 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v3_failure(root, inputs):
    """Bind the immutable v3 post-checker harness-observability failure."""
    reference = V3_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v3 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    submission = value.get("submission") if isinstance(value, dict) else None
    creation = value.get("stage_creation") \
        if isinstance(value, dict) else None
    report = value.get("report") if isinstance(value, dict) else None
    passed = value.get("passed_but_unrecorded_gate") \
        if isinstance(value, dict) else None
    failed = value.get("failed_gate") if isinstance(value, dict) else None
    diagnosis = failed.get("independent_diagnosis") \
        if isinstance(failed, dict) else None
    tree = value.get("final_stage_tree") if isinstance(value, dict) else None
    transport = value.get("transport") if isinstance(value, dict) else None
    remediation = value.get("remediation_contract") \
        if isinstance(value, dict) else None
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v3-"
    expected_submission = {
        "job": V3_PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 36,
        "start": "2026-10-02T02:52:44",
        "end": "2026-10-02T02:53:20",
        "node": "cu081",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 87016,
        "queue_before": [],
        "queue_after_reconciliation": [],
        "ledger_records_after": V3_PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 3752,
        "ledger_sha256_after": V3_PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": V3_PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": V3_PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            V3_PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256":
            V3_PREDECESSOR_STATE["stage_files"][
                ".atlas-stage-creation-transaction.json"]["sha256"],
        "submission_intent_sha256":
            V3_PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            V3_PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": V3_PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": V3_PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": V3_PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": V3_PREDECESSOR_STATE["stage_device"],
        "target_inode": V3_PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_not_reached = [
        "test-progressive-submit",
        "test-weyl-context-core-contract",
        "test-math-weyl-context-core-capture",
        "test-stager-allowlist",
        "toolchain identity",
        "source reconstruction",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    if (value.get("schema")
            != "atlas-weyl-context-core-capture-failure-v3"
            or value.get("classification")
               != "HARNESS_OBSERVABILITY_FAILURE_AFTER_CHECKER_PASS_BEFORE_BUILD_OR_MATHEMATICAL_EXECUTION"
            or value.get("campaign")
               != str(Path(V3_PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != V3_PREDECESSOR_STAGE
            or submission != expected_submission
            or creation != expected_creation
            or not isinstance(report, dict)
            or report.get("path")
               != str(Path(V3_PREDECESSOR_STAGE)
                      / "results/3884456/report.json")
            or report.get("bytes") != 30377
            or report.get("sha256") != V3_PREDECESSOR["report_sha256"]
            or report.get("report_sha256_file_sha256")
               != V3_PREDECESSOR_STATE["stage_files"][
                   "results/3884456/report.sha256"]["sha256"]
            or report.get("status") != "HARNESS_FAILURE"
            or report.get("complete") is not False
            or report.get("acceptance_eligible") is not False
            or report.get("math_gate_released") is not False
            or report.get("cache_gate_released") is not False
            or report.get("ephemeral_workspace_removed") is not True
            or report.get("commands_recorded") != 0
            or report.get("atlas_invocations") != 0
            or report.get("captures_recorded") != 0
            or report.get("legacy_path_open_attempts") != []
            or report.get("cleanup_error") is not None
            or not isinstance(passed, dict)
            or passed.get("name") != "test-campaign-stage-creation"
            or passed.get("tests") != 32
            or passed.get("test_seconds") != 27.245
            or passed.get("gnu_time") != {
                "seconds": 28.52,
                "user_seconds": 3.38,
                "system_seconds": 5.19,
                "maxrss_kb": 29096,
                "exit_status": 0,
                "signal": None,
            }
            or passed.get("stderr_sha256")
               != V3_PREDECESSOR_STATE["stage_files"][
                   "results/3884456/test-campaign-stage-creation.stderr"][
                       "sha256"]
            or passed.get("stdout_sha256")
               != V3_PREDECESSOR_STATE["stage_files"][
                   "results/3884456/test-campaign-stage-creation.stdout"][
                       "sha256"]
            or passed.get("time_sha256")
               != V3_PREDECESSOR_STATE["stage_files"][
                   "results/3884456/test-campaign-stage-creation.time"][
                       "sha256"]
            or not isinstance(failed, dict)
            or failed.get("name") != "driver-command-record"
            or failed.get("exception")
               != ("ValueError: command failed or lacks exact GNU time "
                   "metrics: test-campaign-stage-creation")
            or failed.get("traceback_driver_line") != 1323
            or failed.get("combined_rejection_predicate") != [
                "nonzero process return code",
                "timed_out",
                "termination_uncertain",
                "derived signal",
                "missing or invalid exact GNU time metrics",
            ]
            or not isinstance(diagnosis, dict)
            or diagnosis.get("audits") != 2
            or diagnosis.get("gnu_time_parse_failure_excluded") is not True
            or diagnosis.get("invalid_metric_failure_excluded") is not True
            or diagnosis.get("checker_failure_excluded") is not True
            or diagnosis.get("cleanup_left_live_group_excluded") is not True
            or diagnosis.get("most_likely_remaining_predicate")
               != "termination_uncertain"
            or not isinstance(diagnosis.get("proof_limit"), str)
            or not diagnosis["proof_limit"]
            or value.get("not_reached") != expected_not_reached
            or tree != {
                "schema": "atlas-stage-tree-inventory-v1",
                "root_included": False,
                "entry_order": "relative path ascending",
                "canonical_json": "indent=2, sort_keys=true, trailing LF",
                "sha256": V3_PREDECESSOR_STATE["stage_tree_sha256"],
                "files": V3_PREDECESSOR_STATE["stage_tree_files"],
                "directories": V3_PREDECESSOR_STATE[
                    "stage_tree_directories"],
                "bytes": V3_PREDECESSOR_STATE["stage_tree_bytes"],
                "double_scan_and_final_reread_match": True,
            }
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no mathematical "
                   "result was produced; the failure is in harness process-"
                   "group observation after a checker completed successfully.")
            or not isinstance(remediation, dict)
            or remediation.get("tests_first") is not True
            or remediation.get("changed_input_successor_required") is not True
            or remediation.get("immutable_failed_stage")
               != "weyl-context-core-capture-v3"
            or remediation.get("no_same_stage_resubmission") is not True
            or remediation.get("no_rank_escalation") is not True
            or not isinstance(transport, dict)
            or transport.get("manifest_sha256")
               != "47c07c13ff2a4d9d812977d948356ec89b1702211155436d574a4bb8275eb814"
            or transport.get("inputs") != 35
            or transport.get("local_removed")
               != "/tmp/atlas-wcc-v3-payload-20261002.Z1gYTW"
            or transport.get("remote_removed")
               != "/public/home/majj/.wcc-v3-payload-20261002-47c07c13ff2a"
            or value.get("claims_not_granted") != [
                "complete checker acceptance",
                "source reconstruction",
                "Cargo build",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v3 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v4_failure(root, inputs):
    """Bind the immutable v4 Python-3.9 test-module compilation failure."""
    reference = V4_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v4 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    submission = value.get("submission") if isinstance(value, dict) else None
    creation = value.get("stage_creation") \
        if isinstance(value, dict) else None
    report = value.get("report") if isinstance(value, dict) else None
    passed = value.get("passed_gates") if isinstance(value, dict) else None
    failed = value.get("failed_gate") if isinstance(value, dict) else None
    tree = value.get("final_stage_tree") if isinstance(value, dict) else None
    top_log = value.get("top_level_log") \
        if isinstance(value, dict) else None
    remediation = value.get("remediation_contract") \
        if isinstance(value, dict) else None
    review = value.get("independent_review") \
        if isinstance(value, dict) else None
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v4-"
    expected_submission = {
        "job": V4_PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 24,
        "start": "2026-10-02T03:29:10",
        "end": "2026-10-02T03:29:34",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 0,
        "queue_after_reconciliation": [],
        "ledger_records_after": V4_PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 4201,
        "ledger_sha256_after": V4_PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": V4_PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": V4_PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            V4_PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256":
            V4_PREDECESSOR_STATE["stage_files"][
                ".atlas-stage-creation-transaction.json"]["sha256"],
        "submission_intent_sha256":
            V4_PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            V4_PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": V4_PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": V4_PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": V4_PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": V4_PREDECESSOR_STATE["stage_device"],
        "target_inode": V4_PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(V4_PREDECESSOR_STAGE) / "results/3884494/report.json"),
        "bytes": 36445,
        "sha256": V4_PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256":
            V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/report.sha256"]["sha256"],
        "status": "HARNESS_FAILURE",
        "evidence_maturity": "capture_only_unreviewed",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 3,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "legacy_path_open_attempts": [],
    }
    expected_passed = [
        {
            "name": "test-campaign-stage-creation",
            "tests": 32,
            "seconds": 18.97,
            "user_seconds": 3.3,
            "system_seconds": 4.86,
            "maxrss_kb": 27412,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 3857,
            "stderr_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-campaign-stage-creation.stderr"][
                    "sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-campaign-stage-creation.stdout"][
                    "sha256"],
            "time_bytes": 860,
            "time_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-campaign-stage-creation.time"][
                    "sha256"],
        },
        {
            "name": "test-progressive-submit",
            "tests": 17,
            "seconds": 0.42,
            "user_seconds": 0.14,
            "system_seconds": 0.09,
            "maxrss_kb": 24624,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 1704,
            "stderr_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-progressive-submit.stderr"]["sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-progressive-submit.stdout"]["sha256"],
            "time_bytes": 846,
            "time_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-progressive-submit.time"]["sha256"],
        },
        {
            "name": "test-weyl-context-core-contract",
            "tests": 18,
            "seconds": 0.17,
            "user_seconds": 0.11,
            "system_seconds": 0.03,
            "maxrss_kb": 19672,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 2300,
            "stderr_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-weyl-context-core-contract.stderr"][
                    "sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-weyl-context-core-contract.stdout"][
                    "sha256"],
            "time_bytes": 851,
            "time_sha256": V4_PREDECESSOR_STATE["stage_files"][
                "results/3884494/test-weyl-context-core-contract.time"][
                    "sha256"],
        },
    ]
    expected_failed = {
        "name": "test-math-weyl-context-core-capture",
        "expected_tests": 27,
        "tests_started": 0,
        "failed_checks": ["exit_status"],
        "exit_status": 1,
        "timed_out": False,
        "termination_uncertain": False,
        "cleanup_attempted": False,
        "group_alive_after_communicate": False,
        "group_alive_after_cleanup": False,
        "seconds": 0.1,
        "user_seconds": 0.05,
        "system_seconds": 0.01,
        "maxrss_kb": 18484,
        "stderr_bytes": 1044,
        "stderr_sha256": V4_PREDECESSOR_STATE["stage_files"][
            "results/3884494/test-math-weyl-context-core-capture.stderr"][
                "sha256"],
        "stdout_bytes": 0,
        "stdout_sha256": V4_PREDECESSOR_STATE["stage_files"][
            "results/3884494/test-math-weyl-context-core-capture.stdout"][
                "sha256"],
        "time_bytes": 894,
        "time_sha256": V4_PREDECESSOR_STATE["stage_files"][
            "results/3884494/test-math-weyl-context-core-capture.time"][
                "sha256"],
        "exception": "SyntaxError: too many statically nested blocks",
        "path": "hpc/test_math_weyl_context_core_capture.py",
        "line": 2188,
        "cause": (
            "One with statement contains nineteen context managers inside an "
            "outer with containing two managers, exceeding CPython 3.9's "
            "static block-stack limit during module compilation."
        ),
        "v3_process_group_fix_verified": False,
        "v3_process_group_fix_limit": (
            "The target checker module never imported, so none of its process-"
            "group lifecycle regressions ran."
        ),
    }
    expected_not_reached = [
        "test-stager-allowlist",
        "toolchain identity",
        "source reconstruction",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "sha256": V4_PREDECESSOR_STATE["stage_tree_sha256"],
        "files": V4_PREDECESSOR_STATE["stage_tree_files"],
        "directories": V4_PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": V4_PREDECESSOR_STATE["stage_tree_bytes"],
        "double_scan_and_final_reread_match": True,
    }
    expected_top_log = {
        "path": "weyl-context-core-capture-v4-3884494.out",
        "bytes": 115,
        "mode": "0644",
        "sha256": V4_PREDECESSOR_STATE["stage_files"][
            "weyl-context-core-capture-v4-3884494.out"]["sha256"],
    }
    expected_remediation = {
        "tests_first": True,
        "static_block_fix": (
            "Replace the single oversized multi-manager with statement with "
            "contextlib.ExitStack while preserving every patch target, alias, "
            "assertion and call order."
        ),
        "python_target": (
            "The repaired test module must compile and execute under the HPC "
            "Python 3.9 interpreter used by the driver."
        ),
        "changed_input_successor_required": True,
        "immutable_failed_stage": "weyl-context-core-capture-v4",
        "no_same_stage_resubmission": True,
        "no_rank_escalation": True,
    }
    expected_review = {
        "read_only_audits": 2,
        "report_raw_stream_and_tree_hashes_reconciled": True,
        "current_queue_empty": True,
    }
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "passed_gates",
        "failed_gate", "not_reached", "final_stage_tree", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "independent_review", "claims_not_granted",
    }
    if (not isinstance(value, dict) or set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-capture-failure-v4"
            or value.get("observed_at_utc") != "2026-10-01T19:35:16Z"
            or value.get("classification")
               != "HARNESS_TEST_MODULE_COMPILE_FAILURE_BEFORE_BUILD_OR_MATHEMATICAL_EXECUTION"
            or value.get("campaign")
               != str(Path(V4_PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != V4_PREDECESSOR_STAGE
            or submission != expected_submission
            or creation != expected_creation
            or report != expected_report
            or passed != expected_passed
            or failed != expected_failed
            or value.get("not_reached") != expected_not_reached
            or tree != expected_tree
            or top_log != expected_top_log
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no mathematical "
                   "result was produced; the failure is a Python 3.9 test-"
                   "module compilation error.")
            or remediation != expected_remediation
            or review != expected_review
            or value.get("claims_not_granted") != [
                "complete checker acceptance",
                "v3 process-group remediation verification",
                "source reconstruction",
                "Cargo build",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v4 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v5_failure(root, inputs):
    """Bind the immutable v5 three-byte synthetic-stream mismatch."""
    # Freeze this historical validator before the generic predecessor names
    # advance to v6 for the changed-input successor.
    PREDECESSOR_STAGE = V5_PREDECESSOR_STAGE
    PREDECESSOR_STATE = V5_PREDECESSOR_STATE
    PREDECESSOR = V5_PREDECESSOR
    reference = V5_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v5 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    submission = value.get("submission") if isinstance(value, dict) else None
    creation = value.get("stage_creation") \
        if isinstance(value, dict) else None
    report = value.get("report") if isinstance(value, dict) else None
    passed = value.get("passed_gates") if isinstance(value, dict) else None
    failed = value.get("failed_gate") if isinstance(value, dict) else None
    tree = value.get("final_stage_tree") if isinstance(value, dict) else None
    top_log = value.get("top_level_log") \
        if isinstance(value, dict) else None
    remediation = value.get("remediation_contract") \
        if isinstance(value, dict) else None
    transport = value.get("transport") if isinstance(value, dict) else None
    review = value.get("independent_review") \
        if isinstance(value, dict) else None
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v5-"
    expected_submission = {
        "job": PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 37,
        "start": "2026-10-02T04:09:51",
        "end": "2026-10-02T04:10:28",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 78176,
        "queue_after_reconciliation": [],
        "ledger_records_after": PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 4650,
        "ledger_sha256_after": PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256":
            PREDECESSOR_STATE["stage_files"][
                ".atlas-stage-creation-transaction.json"]["sha256"],
        "submission_intent_sha256":
            PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": PREDECESSOR_STATE["stage_device"],
        "target_inode": PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(PREDECESSOR_STAGE) / "results/3884727/report.json"),
        "bytes": 36496,
        "sha256": PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256":
            PREDECESSOR_STATE["stage_files"][
                "results/3884727/report.sha256"]["sha256"],
        "status": "HARNESS_FAILURE",
        "evidence_maturity": "capture_only_unreviewed",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 3,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "legacy_path_open_attempts": [],
    }
    expected_passed = [
        {
            "name": "test-campaign-stage-creation",
            "tests": 32,
            "seconds": 22.45,
            "user_seconds": 3.76,
            "system_seconds": 5.45,
            "maxrss_kb": 26176,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 3857,
            "stderr_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-campaign-stage-creation.stderr"][
                    "sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-campaign-stage-creation.stdout"][
                    "sha256"],
            "time_bytes": 859,
            "time_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-campaign-stage-creation.time"][
                    "sha256"],
        },
        {
            "name": "test-progressive-submit",
            "tests": 17,
            "seconds": 0.42,
            "user_seconds": 0.14,
            "system_seconds": 0.07,
            "maxrss_kb": 24704,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 1704,
            "stderr_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-progressive-submit.stderr"]["sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-progressive-submit.stdout"]["sha256"],
            "time_bytes": 846,
            "time_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-progressive-submit.time"]["sha256"],
        },
        {
            "name": "test-weyl-context-core-contract",
            "tests": 18,
            "seconds": 0.16,
            "user_seconds": 0.11,
            "system_seconds": 0.02,
            "maxrss_kb": 19672,
            "exit_status": 0,
            "termination_uncertain": False,
            "stderr_bytes": 2300,
            "stderr_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-weyl-context-core-contract.stderr"][
                    "sha256"],
            "stdout_bytes": 0,
            "stdout_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-weyl-context-core-contract.stdout"][
                    "sha256"],
            "time_bytes": 851,
            "time_sha256": PREDECESSOR_STATE["stage_files"][
                "results/3884727/test-weyl-context-core-contract.time"][
                    "sha256"],
        },
    ]
    expected_failed = {
        "name": "test-math-weyl-context-core-capture",
        "expected_tests": 27,
        "tests_run": 27,
        "tests_passed": 26,
        "test_failures": 1,
        "test_errors": 0,
        "failed_test":
            "test_invocation_metrics_must_be_exact_finite_and_nonnegative",
        "failed_checks": ["exit_status"],
        "exit_status": 1,
        "timed_out": False,
        "termination_uncertain": False,
        "cleanup_attempted": False,
        "cleanup_uncertain": False,
        "group_alive_after_communicate": False,
        "group_alive_after_cleanup": False,
        "natural_exit_grace_attempted": False,
        "natural_exit_grace_succeeded": False,
        "seconds": 10.19,
        "user_seconds": 2.23,
        "system_seconds": 1.69,
        "maxrss_kb": 39140,
        "stderr_bytes": 4324,
        "stderr_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884727/test-math-weyl-context-core-capture.stderr"][
                "sha256"],
        "stdout_bytes": 0,
        "stdout_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884727/test-math-weyl-context-core-capture.stdout"][
                "sha256"],
        "time_bytes": 903,
        "time_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884727/test-math-weyl-context-core-capture.time"][
                "sha256"],
        "assertion": "AssertionError: 857 != 860",
        "path": "hpc/test_math_weyl_context_core_capture.py",
        "line": 921,
        "cause": (
            "The synthetic copy of the immutable v3 first-checker GNU-time "
            "stream omitted the three bytes '-p ' between the unittest "
            "discovery directory and filename pattern, while still asserting "
            "the original 860-byte length and SHA-256."
        ),
        "synthetic_before": {
            "bytes": 857,
            "sha256":
                "c46842009141dade5b846e2f934c1eb4c6db755cdd16c4dff188c9cf5bd4abd9",
        },
        "immutable_v3_raw_and_synthetic_after": {
            "bytes": 860,
            "sha256":
                "624f8ea4714c371560f51da4dede4ec55afdd05e6cf05c4c6c433ec25acad4b2",
        },
        "v3_process_group_remediation_regressions_verified": True,
        "v3_process_group_verification_basis": (
            "The same 27-test run passed "
            "test_driver_command_and_process_group_boundaries_are_static, "
            "including successful-leader natural grace, persistent-group "
            "cleanup, failed leader, timeout, lingering group and cleanup-"
            "failure branches."
        ),
    }
    expected_not_reached = [
        "test-stager-allowlist",
        "toolchain identity",
        "source reconstruction",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "sha256": PREDECESSOR_STATE["stage_tree_sha256"],
        "files": PREDECESSOR_STATE["stage_tree_files"],
        "directories": PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": PREDECESSOR_STATE["stage_tree_bytes"],
        "double_scan_and_final_reread_match": True,
    }
    expected_top_log = {
        "path": "weyl-context-core-capture-v5-3884727.out",
        "bytes": 115,
        "mode": "0644",
        "sha256": PREDECESSOR_STATE["stage_files"][
            "weyl-context-core-capture-v5-3884727.out"]["sha256"],
    }
    expected_remediation = {
        "tests_first": True,
        "exact_literal_fix": (
            "Insert only b'-p ' into the synthetic v3 Command-being-timed "
            "byte string at the exact position present in the immutable raw "
            "artifact."
        ),
        "proof_obligation": (
            "The repaired literal must be exactly 860 bytes with SHA-256 "
            "624f8ea4714c371560f51da4dede4ec55afdd05e6cf05c4c6c433ec25acad4b2, "
            "and _parse_time must retain the same metrics."
        ),
        "production_driver_change_allowed": False,
        "changed_input_successor_required": True,
        "immutable_failed_stage": "weyl-context-core-capture-v5",
        "no_same_stage_resubmission": True,
        "no_rank_escalation": True,
    }
    expected_transport = {
        "manifest_sha256":
            "09d623a9663d2612787a6408422dc54074dd0ecd6ec6a105ea262fbf621c07ac",
        "inputs": 37,
        "files_including_manifest": 38,
        "transfer_invocations": 1,
        "creator_invocations": 1,
        "local_removed": "/tmp/atlas-wcc-v5-payload-20261002.98dpm59p",
        "remote_removed":
            "/public/home/majj/.wcc-v5-payload-20261002-09d623a9663d",
    }
    expected_review = {
        "read_only_audits": 1,
        "latest_reconciliation_at_utc": "2026-10-01T20:22:02Z",
        "report_raw_stream_and_tree_hashes_reconciled": True,
        "current_queue_empty": True,
        "status": "INDEPENDENT_READ_ONLY_AUDIT_COMPLETE",
    }
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "passed_gates",
        "failed_gate", "not_reached", "final_stage_tree", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "transport", "independent_review",
        "claims_not_granted",
    }
    if (not isinstance(value, dict) or set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-capture-failure-v5"
            or value.get("observed_at_utc")
               != "2026-10-01T20:15:55.174766Z"
            or value.get("classification")
               != "HARNESS_TEST_EXPECTATION_FAILURE_BEFORE_BUILD_OR_MATHEMATICAL_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or submission != expected_submission
            or creation != expected_creation
            or report != expected_report
            or passed != expected_passed
            or failed != expected_failed
            or value.get("not_reached") != expected_not_reached
            or tree != expected_tree
            or top_log != expected_top_log
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no mathematical "
                   "result was produced; the only failure is an incomplete "
                   "synthetic copy of an already frozen GNU-time command line "
                   "in a harness test.")
            or remediation != expected_remediation
            or transport != expected_transport
            or review != expected_review
            or value.get("claims_not_granted") != [
                "complete checker acceptance",
                "source reconstruction",
                "Cargo build",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v5 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v6_failure(root, inputs):
    """Bind the immutable v6 sealed-boundary extraction failure."""
    PREDECESSOR_STAGE = V6_PREDECESSOR_STAGE
    PREDECESSOR_STATE = V6_PREDECESSOR_STATE
    PREDECESSOR = V6_PREDECESSOR
    reference = V6_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v6 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core v6 failure evidence changed")
    submission = value.get("submission")
    creation = value.get("stage_creation")
    report = value.get("report")
    passed = value.get("passed_test_gates")
    toolchains = value.get("passed_toolchain_identity")
    failed = value.get("failed_gate")
    tree = value.get("final_stage_tree")
    top_log = value.get("top_level_log")
    remediation = value.get("remediation_contract")
    transport = value.get("transport")
    review = value.get("independent_review")
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v6-"
    expected_submission = {
        "job": PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 55,
        "start": "2026-10-02T04:52:15",
        "end": "2026-10-02T04:53:10",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 79396,
        "queue_after_reconciliation": [],
        "ledger_records_after": PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 5099,
        "ledger_sha256_after": PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256": PREDECESSOR_STATE[
            "stage_files"][".atlas-stage-creation-transaction.json"][
                "sha256"],
        "submission_intent_sha256":
            PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": PREDECESSOR_STATE["stage_device"],
        "target_inode": PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(PREDECESSOR_STAGE) / "results/3884751/report.json"),
        "bytes": 38975,
        "sha256": PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884751/report.sha256"]["sha256"],
        "status": "HARNESS_FAILURE",
        "evidence_maturity": "capture_only_unreviewed",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 7,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "legacy_path_open_attempts": [],
    }
    expected_gate_keys = {
        "name", "tests", "seconds", "user_seconds", "system_seconds",
        "maxrss_kb", "exit_status", "timed_out", "termination_uncertain",
        "stderr_bytes", "stderr_sha256", "stdout_bytes", "stdout_sha256",
        "time_bytes", "time_sha256",
    }
    empty_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    expected_gate_summaries = [
        ("test-campaign-stage-creation", 32, 24.63, 3.96, 5.79, 25744,
         0, False, False, 3857,
         "7dbe2b13d224c57e773b3ebbaaa42e237b70041590a99cf77097f722f20757bf",
         0, empty_sha, 859,
         "824c5795519091a648441ca795dfb17d38045884f41161f73e6b5c8e7d618c1c"),
        ("test-progressive-submit", 17, 0.44, 0.15, 0.09, 24792,
         0, False, False, 1704,
         "5e7d9b3414564b47d2d18f0c108f9b1a32ead089f2f33d8a91becc46615716ef",
         0, empty_sha, 846,
         "1af02c4d43313d91b03b7d1cc9c6adea650bacaf54b6b7d7542ef8e952016cf7"),
        ("test-weyl-context-core-contract", 18, 0.17, 0.1, 0.03, 19672,
         0, False, False, 2300,
         "8634408934711a7ba27040666f8eadd66beeda9e5175e5449761975e56884970",
         0, empty_sha, 851,
         "fe95bbd52a7275bc8e3cdf35d34552ef25cf116723fb60686babb5ffc755a2a7"),
        ("test-math-weyl-context-core-capture", 27, 10.38, 2.3, 1.76, 39132,
         0, False, False, 3707,
         "ab106c62b2e1187330152d23cad51bbf2ddec3e2fe035913bce199cce06756a3",
         0, empty_sha, 865,
         "084d1eef326dc0dff4e06d9e9fd154ca3f7842878aa82fd0d7772737cd44c5b8"),
        ("test-stager-allowlist", 7, 6.35, 6.25, 0.06, 27400,
         0, False, False, 830,
         "66a344f84883913e3849d2c8ad826100dbf156cd0daeffb456679e822fff8dfe",
         0, empty_sha, 842,
         "ca26aa68687377e96e498880ba43a9370d6b25ffd745fc8573d0926062a0b6cc"),
    ]
    observed_gate_summaries = [
        (row.get("name"), row.get("tests"), row.get("seconds"),
         row.get("user_seconds"), row.get("system_seconds"),
         row.get("maxrss_kb"), row.get("exit_status"),
         row.get("timed_out"), row.get("termination_uncertain"),
         row.get("stderr_bytes"), row.get("stderr_sha256"),
         row.get("stdout_bytes"), row.get("stdout_sha256"),
         row.get("time_bytes"), row.get("time_sha256"))
        for row in passed
    ] if isinstance(passed, list) else None
    expected_toolchain_keys = {
        "name", "identity", "seconds", "user_seconds", "system_seconds",
        "maxrss_kb", "exit_status", "timed_out", "termination_uncertain",
        "stderr_bytes", "stderr_sha256", "stdout_bytes", "stdout_sha256",
        "time_bytes", "time_sha256",
    }
    expected_toolchain_summaries = [
        ("rustc-version", "rustc 1.96.0 (ac68faa20 2026-05-25)",
         5.39, 0.0, 0.06, 31896, 0, False, False, 0, empty_sha, 196,
         "f64c55ed9f210d80a9b248206104a47175faf5ec94a093e4ad132549209089fc",
         733,
         "3836d4d332b5536e9baca891145d3777c949008bd6abc31e08d25017765e8c5f"),
        ("cargo-version", "cargo 1.96.0 (30a34c682 2026-05-25)",
         0.11, 0.0, 0.01, 9580, 0, False, False, 0, empty_sha, 320,
         "fdb3f2fc57229bfd22c53064c26587a57a856e5ef3f9f5cd5e30d5af2c818a1d",
         728,
         "395dbef32a254c5b9f78928ef526fc8a7fe73d047ee552593ac4a27ac10e1d82"),
    ]
    observed_toolchain_summaries = [
        (row.get("name"), row.get("identity"), row.get("seconds"),
         row.get("user_seconds"), row.get("system_seconds"),
         row.get("maxrss_kb"), row.get("exit_status"),
         row.get("timed_out"), row.get("termination_uncertain"),
         row.get("stderr_bytes"), row.get("stderr_sha256"),
         row.get("stdout_bytes"), row.get("stdout_sha256"),
         row.get("time_bytes"), row.get("time_sha256"))
        for row in toolchains
    ] if isinstance(toolchains, list) else None
    expected_failed = {
        "name": "sealed-boundary-fixture-extraction",
        "after_commands": 7,
        "failed_command": None,
        "exception": "ValueError: sealed ladder fixture frame changed",
        "driver_path": "hpc/math_weyl_context_core_capture.py",
        "call_line": 1756,
        "check_line": 1084,
        "source_archive_materialized": True,
        "test_patch_applied": False,
        "production_patch_applied": False,
        "source_manifest_verified": False,
        "cause": (
            "The capture-only helper used the unaccepted short identifier "
            "root_ladder_coordinate_boundary, while the immutable parent "
            "seal and accepted ladder AFTER v3 helper use "
            "generic_root_ladder_coordinate_boundary. Its input suffix also "
            "omitted the accepted leading newline. A second latent mismatch "
            "would have stripped the accepted stdout frame even though "
            "SEALED_FIXTURE_HASHES binds the complete stdout stream."
        ),
        "sealed_input": {
            "bytes": 1896,
            "sha256":
                "dde2e5c1f84d255ef66e01c83deef114f70a22dc22224e87cb186359077df95f",
            "accepted_case_id": "generic_root_ladder_coordinate_boundary",
            "accepted_prefix_bytes": 61,
            "accepted_suffix_bytes": 65,
        },
        "case_id_only_extraction": {
            "bytes": 1771,
            "sha256":
                "2ffeb1664291eeea9872e9ab2726f3ac4c97e89e5e2ddbc54ed61ce30856d5ee",
            "defect": "one unconsumed separator LF",
        },
        "corrected_extraction": {
            "bytes": 1770,
            "sha256": SEALED_FIXTURE_HASHES[
                "tests/math/generics/root_ladder_coordinate_boundary.atlas"],
        },
        "sealed_oracle_stdout": {
            "bytes": 2304,
            "sha256": SEALED_FIXTURE_HASHES[
                "tests/math/generics/"
                "root_ladder_coordinate_boundary.oracle.stdout"],
            "accepted_prefix_bytes": 51,
            "accepted_suffix_bytes": 54,
        },
        "wrongly_sliced_oracle_stdout": {
            "bytes": 2199,
            "sha256":
                "3f37a408e1d9be00b5fe4c363c1dece37041a08332bd263b40dc3d384fd5ff80",
        },
        "sealed_oracle_stderr": {
            "bytes": 0,
            "sha256":
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        },
    }
    expected_not_reached = [
        "sealed boundary fixture installation into the reconstructed source",
        "tests-only patch application",
        "production patch application",
        "source-manifest verification",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "sha256": PREDECESSOR_STATE["stage_tree_sha256"],
        "files": PREDECESSOR_STATE["stage_tree_files"],
        "directories": PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": PREDECESSOR_STATE["stage_tree_bytes"],
        "triple_scan_and_final_reread_match": True,
    }
    expected_remediation = {
        "tests_first": True,
        "new_regression": (
            "Exercise _sealed_boundary_fixtures with the accepted "
            "generic_root_ladder_coordinate_boundary input envelope and "
            "require the complete sealed oracle stdout to be returned "
            "unchanged."
        ),
        "exact_driver_fix": [
            "Use the accepted case id "
            "generic_root_ladder_coordinate_boundary.",
            "Use the accepted input suffix with one leading newline before "
            "the MATH_END print.",
            "Return the full sealed oracle stdout instead of stripping its "
            "accepted frame.",
        ],
        "proof_obligation": (
            "The extracted fixture must be 1,770 bytes at SHA-256 "
            "dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65 "
            "and the returned oracle stdout must remain the complete "
            "2,304-byte stream at SHA-256 "
            "3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80."
        ),
        "other_execution_logic_change_allowed": False,
        "changed_input_successor_required": True,
        "immutable_failed_stage": "weyl-context-core-capture-v6",
        "no_same_stage_resubmission": True,
        "no_rank_escalation": True,
    }
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "passed_test_gates",
        "passed_toolchain_identity", "failed_gate", "not_reached",
        "final_stage_tree", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "transport", "independent_review",
        "claims_not_granted",
    }
    if (set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-capture-failure-v6"
            or value.get("observed_at_utc")
               != "2026-10-01T20:58:05.746305Z"
            or value.get("classification")
               != "HARNESS_SEALED_BOUNDARY_ENVELOPE_CONTRACT_MISMATCH_BEFORE_PATCH_APPLICATION_BUILD_OR_ATLAS_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or submission != expected_submission
            or creation != expected_creation
            or report != expected_report
            or not isinstance(passed, list)
            or any(not isinstance(row, dict)
                   or set(row) != expected_gate_keys for row in passed)
            or observed_gate_summaries != expected_gate_summaries
            or not isinstance(toolchains, list)
            or any(not isinstance(row, dict)
                   or set(row) != expected_toolchain_keys
                   for row in toolchains)
            or observed_toolchain_summaries != expected_toolchain_summaries
            or failed != expected_failed
            or value.get("not_reached") != expected_not_reached
            or tree != expected_tree
            or top_log != {
                "path": "weyl-context-core-capture-v6-3884751.out",
                "bytes": 115,
                "mode": "0644",
                "sha256": PREDECESSOR_STATE["stage_files"][
                    "weyl-context-core-capture-v6-3884751.out"]["sha256"],
            }
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no "
                   "mathematical result was produced; the failure is a "
                   "capture-harness disagreement with an already accepted "
                   "immutable stream envelope.")
            or remediation != expected_remediation
            or transport != {
                "manifest_sha256":
                    "ba8891c009b6bb9de08908fa868705d2c77bc14e0f203631b2ca25972bd90cf6",
                "inputs": 38,
                "files_including_manifest": 39,
                "transfer_invocations": 1,
                "remote_creator_invocations": 1,
                "local_pre_ssh_failures": 1,
                "local_pre_ssh_failure_remote_effect": False,
                "local_removed": "/tmp/atlas-wcc-v6-transport.SBJ9p3",
                "remote_removed":
                    "/public/home/majj/.wcc-v6-payload-20261002-ba8891c009b6",
            }
            or review != {
                "read_only_audits": 2,
                "latest_reconciliation_at_utc": "2026-10-01T21:03:13Z",
                "report_raw_stream_and_tree_hashes_reconciled": True,
                "current_queue_empty": True,
                "status": "INDEPENDENT_READ_ONLY_AUDIT_COMPLETE",
            }
            or value.get("claims_not_granted") != [
                "source reconstruction acceptance",
                "Cargo build",
                "original Atlas execution",
                "Rust Atlas execution",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v6 failure evidence changed")
    return copy.deepcopy(value)


def validate_capture_v7_failure(root, inputs):
    """Bind the immutable v7 synthetic-receipt fixture failure."""
    PREDECESSOR_STAGE = V7_PREDECESSOR_STAGE
    PREDECESSOR_STATE = V7_PREDECESSOR_STATE
    PREDECESSOR = V7_PREDECESSOR
    reference = V7_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core v7 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core v7 failure evidence changed")
    scoped = ".atlas-stage-creation-weyl-context-core-capture-v7-"
    expected_submission = {
        "job": PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 30,
        "start": "2026-10-02T05:48:32",
        "end": "2026-10-02T05:49:02",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 59084,
        "queue_after_reconciliation": [],
        "ledger_records_after": PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 5548,
        "ledger_sha256_after": PREDECESSOR["campaign_ledger_sha256"],
        "pin_sha256": PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256": PREDECESSOR_STATE[
            "stage_files"][".atlas-stage-creation-transaction.json"][
                "sha256"],
        "submission_intent_sha256":
            PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": PREDECESSOR_STATE["stage_device"],
        "target_inode": PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(PREDECESSOR_STAGE) / "results/3884780/report.json"),
        "bytes": 34972,
        "sha256": PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884780/report.sha256"]["sha256"],
        "status": "HARNESS_FAILURE",
        "evidence_maturity": "capture_only_unreviewed",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 1,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "legacy_path_open_attempts": [],
    }
    empty_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    expected_passed = [{
        "name": "test-campaign-stage-creation",
        "tests": 32,
        "seconds": 25.42,
        "user_seconds": 4.06,
        "system_seconds": 6.15,
        "maxrss_kb": 26240,
        "exit_status": 0,
        "timed_out": False,
        "termination_uncertain": False,
        "stderr_bytes": 3857,
        "stderr_sha256":
            "ff73093590166639fd9012e10513266c95e578f3c1092dbe0c3067f74859906b",
        "stdout_bytes": 0,
        "stdout_sha256": empty_sha,
        "time_bytes": 859,
        "time_sha256":
            "6ab955469418440aa095b04009b317871a7f2ac2f86a283b19c5377003cd0f33",
    }]
    expected_failed = {
        "name": "test-progressive-submit",
        "tests": 17,
        "passed": 13,
        "failures": 0,
        "errors": 4,
        "seconds": 0.34,
        "user_seconds": 0.13,
        "system_seconds": 0.07,
        "maxrss_kb": 24832,
        "exit_status": 1,
        "timed_out": False,
        "termination_uncertain": False,
        "stderr_bytes": 5312,
        "stderr_sha256":
            "7e10b2ca8baaa409332e6227ee4b9bf393e13fa50942daa200b44206435a74ff",
        "stdout_bytes": 0,
        "stdout_sha256": empty_sha,
        "time_bytes": 884,
        "time_sha256":
            "bbe4d5f9757bda787c236ee05c83aac17b535edba4d4df28e67cb69c4bb1de5d",
        "failed_checks": ["exit_status"],
        "error_tests": [
            "test_campaign_stages_share_one_ledger",
            "test_same_stage_cannot_submit_twice",
            "test_submission_uses_the_prechecked_script_snapshot",
            "test_uncertain_submission_cannot_retry",
        ],
        "exception": "KeyError: 'stage_device'",
        "production_read": "hpc/progressive_submit.py:2505",
        "fixture_builder": "hpc/test_progressive_submit.py:34-45",
        "cause": (
            "The synthetic validate_stage_creation receipt used by the "
            "isolated progressive-submit tests retained the old schema "
            "subset and omitted stage_device and stage_inode. The "
            "descriptor-bound sbatch path correctly requires those real "
            "receipt fields before comparing the opened stage descriptor "
            "with the publication identity."
        ),
        "production_path_evidence": (
            "The v7 creator used a complete real stage-creation receipt "
            "containing device 3431958692 and inode 162130669655307095 and "
            "successfully obtained the durable sbatch receipt for job "
            "3884780. The defect is confined to the test mock."
        ),
        "actual_stage_creation_tests_passed": 32,
    }
    expected_not_reached = [
        "Weyl-context contract tests",
        "capture-driver tests",
        "stager allowlist tests",
        "Rust and Cargo identity probes",
        "sealed boundary fixture installation",
        "tests-only patch application",
        "production patch application",
        "source-manifest verification",
        "Cargo release build",
        "original Atlas execution",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "inventory_bytes": 25248,
        "sha256": PREDECESSOR_STATE["stage_tree_sha256"],
        "files": PREDECESSOR_STATE["stage_tree_files"],
        "directories": PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": PREDECESSOR_STATE["stage_tree_bytes"],
        "coordinator_triple_scan_and_independent_double_scan_match": True,
    }
    expected_remediation = {
        "tests_first": True,
        "existing_failure_reproduction": (
            "Four existing progressive-submit tests reach the "
            "descriptor-binding branch and all fail on the missing "
            "synthetic receipt identity before the fix."
        ),
        "exact_test_fixture_fix": [
            "Resolve the synthetic stage once and bind its real st_dev as "
            "stage_device.",
            "Bind the same stage's real st_ino as stage_inode.",
            "Keep production direct indexing and fail-closed identity "
            "comparison unchanged.",
            "In the existing script-snapshot test, assert one pass_fds "
            "descriptor, the matching /proc/self/fd cwd, equal "
            "descriptor/cwd/stage identities, clean env, text mode and the "
            "25-second timeout.",
        ],
        "production_submission_algorithm_change_allowed": False,
        "expected_test_counts_after": {
            "test-campaign-stage-creation": 32,
            "test-progressive-submit": 17,
            "test-weyl-context-core-contract": 18,
            "test-math-weyl-context-core-capture": 28,
            "test-stager-allowlist": 7,
            "total": 102,
        },
        "changed_input_successor_required": True,
        "immutable_failed_stage": "weyl-context-core-capture-v7",
        "no_same_stage_resubmission": True,
        "no_rank_escalation": True,
    }
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "passed_test_gates",
        "failed_gate", "not_reached", "final_stage_tree", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "transport", "independent_review",
        "claims_not_granted",
    }
    if (set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-capture-failure-v7"
            or value.get("observed_at_utc")
               != "2026-10-01T21:51:28.975536Z"
            or value.get("classification")
               != "HARNESS_SYNTHETIC_STAGE_CREATION_RECEIPT_IDENTITY_OMISSION_BEFORE_CONTRACT_BUILD_OR_ATLAS_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("submission") != expected_submission
            or value.get("stage_creation") != expected_creation
            or value.get("report") != expected_report
            or value.get("passed_test_gates") != expected_passed
            or value.get("failed_gate") != expected_failed
            or value.get("not_reached") != expected_not_reached
            or value.get("final_stage_tree") != expected_tree
            or value.get("top_level_log") != {
                "path": "weyl-context-core-capture-v7-3884780.out",
                "bytes": 115,
                "mode": "0644",
                "sha256": PREDECESSOR_STATE["stage_files"][
                    "weyl-context-core-capture-v7-3884780.out"]["sha256"],
            }
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no "
                   "mathematical result was produced; the failure is an "
                   "infrastructure test-fixture contract mismatch.")
            or value.get("remediation_contract") != expected_remediation
            or value.get("transport") != {
                "manifest_sha256":
                    "8a7859b6b2f58bd611dca0e26ba419f388c04c9fcf59000e47dbfc984184ca0a",
                "inputs": 39,
                "files_including_manifest": 40,
                "transfer_invocations": 1,
                "remote_creator_invocations": 1,
                "local_removed":
                    "/tmp/atlas-wcc-v7-transport.8a7859b6b2f5.DC7c17",
                "remote_removed": (
                    "/public/home/majj/"
                    ".wcc-v7-payload-20261002-8a7859b6b2f5"
                ),
            }
            or value.get("independent_review") != {
                "read_only_audits": 2,
                "latest_coordinator_reconciliation_at_utc":
                    "2026-10-01T21:51:28.975536Z",
                "report_raw_stream_and_tree_hashes_reconciled": True,
                "durable_receipt_and_ledger_tail_reconciled": True,
                "current_queue_empty": True,
                "transport_absent": True,
                "status": "INDEPENDENT_READ_ONLY_AUDIT_COMPLETE",
            }
            or value.get("claims_not_granted") != [
                "progressive-submit checker pass",
                "Weyl-context contract checker execution",
                "capture checker execution",
                "allowlist checker execution",
                "source reconstruction",
                "Cargo build",
                "original Atlas execution",
                "Rust Atlas execution",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core v7 failure evidence changed")
    return copy.deepcopy(value)


def validate_catalog(root, inputs):
    if inputs.get(CATALOG_PATH) != CATALOG_SHA256:
        raise ValueError("Weyl core catalog hash changed")
    catalog = load_relative_json(root, CATALOG_PATH, CATALOG_SHA256)
    cases = catalog.get("cases") if isinstance(catalog, dict) else None
    if (catalog.get("schema") != "atlas-weyl-context-core-discovery-v1"
            or catalog.get("evidence_maturity")
               != "source_predicted_not_captured"
            or not isinstance(catalog.get("scope"), str)
            or not isinstance(cases, list) or len(cases) != len(CATALOG_CASES)):
        raise ValueError("Weyl core catalog schema changed")
    for expected, observed in zip(CATALOG_CASES, cases):
        contract_case = {
            key: expected[key] for key in (
                "id", "file", "fixture_sha256", "intent", "timeout_seconds")
        }
        if (not isinstance(observed, dict)
                or any(observed.get(key) != value
                       for key, value in contract_case.items())
                or set(observed) != set(contract_case) | {"scope"}
                or not isinstance(observed.get("scope"), str)):
            raise ValueError("Weyl core catalog case changed")
        fixture_name = "tests/math/generics/" + expected["file"]
        raw = stable_relative_bytes(root, fixture_name)
        if (inputs.get(fixture_name) != expected["fixture_sha256"]
                or hashlib.sha256(raw).hexdigest()
                   != expected["fixture_sha256"]
                or not raw.endswith(b"\n")):
            raise ValueError("Weyl core fixture changed")
        source = (
            ("prints(\"MATH_BEGIN " + expected["id"] + "\")\n").encode()
            + raw
            + ("prints(\"MATH_END " + expected["id"] + "\")\nquit\n").encode()
        )
        if (len(source) != expected["input_bytes"]
                or hashlib.sha256(source).hexdigest()
                   != expected["input_sha256"]):
            raise ValueError("Weyl core input envelope changed")
    return copy.deepcopy(catalog)


def validate_capture_v8(root, inputs):
    """Bind the historical complete v8 capture and non-accepting review."""
    PREDECESSOR_STAGE = V8_PREDECESSOR_STAGE
    PREDECESSOR_STATE = V8_PREDECESSOR_STATE
    PREDECESSOR = V8_PREDECESSOR
    references = (
        (REGRESSION_CATALOG_PATH, REGRESSION_CATALOG_SHA256,
         REGRESSION_CATALOG_BYTES),
        (REGRESSION_INSPECTION_PATH, REGRESSION_INSPECTION_SHA256,
         REGRESSION_INSPECTION_BYTES),
        (V8_SUBMISSION_EVIDENCE_PATH, V8_SUBMISSION_EVIDENCE_SHA256,
         V8_SUBMISSION_EVIDENCE_BYTES),
    )
    raw_by_name = {}
    for name, wanted, size in references:
        raw = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        if (inputs.get(name) != wanted or len(raw) != size
                or hashlib.sha256(raw).hexdigest() != wanted):
            raise ValueError("v8 capture evidence bytes changed: " + name)
        raw_by_name[name] = raw
    catalog = regression_contract.decode_catalog(
        raw_by_name[REGRESSION_CATALOG_PATH])
    inspection = regression_contract.decode_inspection(
        raw_by_name[REGRESSION_INSPECTION_PATH], catalog)
    submission = strict_json_loads(raw_by_name[V8_SUBMISSION_EVIDENCE_PATH])
    evidence = inspection["submission_evidence"]
    tree = inspection["artifacts"]["stage_tree"]
    if (inspection["stage"] != PREDECESSOR_STAGE
            or inspection["job"]["id"] != PREDECESSOR["job"]
            or inspection["report"]["sha256"] != PREDECESSOR["report_sha256"]
            or inspection["report"]["bytes"] != PREDECESSOR["report_bytes"]
            or evidence["file"] != V8_SUBMISSION_EVIDENCE_PATH
            or evidence["sha256"] != V8_SUBMISSION_EVIDENCE_SHA256
            or evidence["bytes"] != V8_SUBMISSION_EVIDENCE_BYTES
            or evidence["ledger_records"] != PREDECESSOR["campaign_ledger_records"]
            or evidence["ledger_sha256"] != PREDECESSOR["campaign_ledger_sha256"]
            or tree["sha256"] != PREDECESSOR_STATE["stage_tree_sha256"]
            or tree["files"] != PREDECESSOR_STATE["stage_tree_files"]
            or tree["directories"] != PREDECESSOR_STATE["stage_tree_directories"]
            or tree["file_bytes"] != PREDECESSOR_STATE["stage_tree_bytes"]
            or not isinstance(submission, dict)):
        raise ValueError("historical v8 capture review identity changed")
    # BEFORE v1's immutable direct-predecessor tree and failure record close
    # over these v8 inputs.  This review document cannot replace that chain.
    return copy.deepcopy(inspection)


def validate_before_v1_failure(root, inputs):
    """Bind the immutable BEFORE v1 two-error harness failure."""
    PREDECESSOR_STAGE = BEFORE_V1_PREDECESSOR_STAGE
    PREDECESSOR_STATE = BEFORE_V1_PREDECESSOR_STATE
    PREDECESSOR = BEFORE_V1_PREDECESSOR
    reference = BEFORE_V1_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core BEFORE v1 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core BEFORE v1 failure evidence changed")
    scoped = ".atlas-stage-creation-weyl-context-core-before-v1-"
    empty_sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    expected_submission = {
        "job": PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 47,
        "start": "2026-10-02T07:29:53",
        "end": "2026-10-02T07:30:40",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 87272,
        "queue_after_reconciliation": [],
        "ledger_records_after": PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 6445,
        "ledger_sha256_after": PREDECESSOR["campaign_ledger_sha256"],
        "predecessor_ledger_records": 15,
        "predecessor_ledger_sha256":
            V8_PREDECESSOR["campaign_ledger_sha256"],
        "override_manifest_bytes": 6240,
        "override_manifest_sha256": PREDECESSOR["override_manifest_sha256"],
        "override_manifest_inputs": 50,
        "installed_and_override_copies_reconciled": 100,
        "pin_sha256": PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256":
            PREDECESSOR["stage_creation_transaction_sha256"],
        "submission_intent_sha256":
            PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            PREDECESSOR["submission_receipt_sha256"],
    }
    expected_creation = {
        "prepared_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "sealed_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "published_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "target_device": PREDECESSOR_STATE["stage_device"],
        "target_inode": PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(PREDECESSOR_STAGE) / "results/3884862/report.json"),
        "bytes": PREDECESSOR["report_bytes"],
        "sha256": PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884862/report.sha256"]["sha256"],
        "status": "WEYL_CONTEXT_BEFORE_HARNESS_FAILURE",
        "evidence_maturity": "tests_first_before",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 4,
        "source_files_recorded": 0,
        "binaries_recorded": 0,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "regression_classification_recorded": False,
        "cleanup_error_present": False,
        "legacy_path_open_attempts": [],
    }
    expected_passed = [
        {
            "name": "test-campaign-stage-creation",
            "tests": 32,
            "seconds": 27.47,
            "user_seconds": 4.44,
            "system_seconds": 6.79,
            "maxrss_kb": 25472,
            "exit_status": 0,
            "timed_out": False,
            "termination_uncertain": False,
            "stderr_bytes": 3878,
            "stderr_sha256":
                "5f3248ea474e736d594623b8c1068ef9e38c124fa7110637d40c08bea67fdd07",
            "stdout_bytes": 0,
            "stdout_sha256": empty_sha,
            "time_bytes": 860,
            "time_sha256":
                "14c828d922f234719bd4737337b1e52758af1cde5cd30f3086872565efb50035",
        },
        {
            "name": "test-progressive-submit",
            "tests": 17,
            "seconds": 0.44,
            "user_seconds": 0.15,
            "system_seconds": 0.09,
            "maxrss_kb": 24800,
            "exit_status": 0,
            "timed_out": False,
            "termination_uncertain": False,
            "stderr_bytes": 1704,
            "stderr_sha256":
                "8dba699563c9f33f3ec5e566ecbbf8c69a83961a15e04a6bd31ed6506938a476",
            "stdout_bytes": 0,
            "stdout_sha256": empty_sha,
            "time_bytes": 846,
            "time_sha256":
                "f216010da3c2863911c371622ca87b07bed97e0de6f21c5622b4f695e02647d4",
        },
        {
            "name": "test-weyl-context-core-contract",
            "tests": 18,
            "seconds": 0.17,
            "user_seconds": 0.11,
            "system_seconds": 0.03,
            "maxrss_kb": 21428,
            "exit_status": 0,
            "timed_out": False,
            "termination_uncertain": False,
            "stderr_bytes": 2300,
            "stderr_sha256":
                "9faace8f7eae2df8d428884fcbf85ae240ceaf8cc8640c7e6ceb94e5fee008cd",
            "stdout_bytes": 0,
            "stdout_sha256": empty_sha,
            "time_bytes": 851,
            "time_sha256":
                "9a9934082a52cf19731308b0a0b97bb871bceef4ebaebe7590cb36be664e0415",
        },
        {
            "name": "test-weyl-context-core-regression-contract",
            "tests": 17,
            "seconds": 0.19,
            "user_seconds": 0.14,
            "system_seconds": 0.03,
            "maxrss_kb": 21264,
            "exit_status": 0,
            "timed_out": False,
            "termination_uncertain": False,
            "stderr_bytes": 2306,
            "stderr_sha256":
                "311052aace98eafe69817a3042a00c421a5e1ec806fffc2f4573a4cebd3fcdcd",
            "stdout_bytes": 0,
            "stdout_sha256": empty_sha,
            "time_bytes": 853,
            "time_sha256":
                "2d97df00166118e8825075c246af85f3cf483c50418a05a366105c83fde2089d",
        },
    ]
    expected_failed = {
        "name": "test-math-weyl-context-core-capture",
        "tests": 28,
        "passed": 26,
        "failures": 1,
        "errors": 1,
        "seconds": 13.88,
        "user_seconds": 3.15,
        "system_seconds": 2.42,
        "maxrss_kb": 44412,
        "exit_status": 1,
        "timed_out": False,
        "termination_uncertain": False,
        "cleanup_attempted": False,
        "cleanup_uncertain": False,
        "group_alive_after_communicate": False,
        "group_alive_after_cleanup": False,
        "stderr_bytes": 5271,
        "stderr_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884862/test-math-weyl-context-core-capture.stderr"][
                "sha256"],
        "stdout_bytes": 0,
        "stdout_sha256": empty_sha,
        "time_bytes": 903,
        "time_sha256":
            "b3d07fbad2668b6d81206183c469d1a8c727c991394990301e9c14d7b8773591",
        "failed_checks": ["exit_status"],
        "failure": {
            "test": (
                "test_report_accepts_prediction_difference_but_rejects_"
                "incomplete_capture"
            ),
            "path": "hpc/test_math_weyl_context_core_capture.py",
            "line": 1706,
            "expected": "RUST_SOURCE_PREDICTION_DIFFERED",
            "actual": "BOTH_SOURCE_PREDICTIONS_DIFFERED",
            "cause": (
                "The synthetic complete-report test retained the pre-v8 "
                "one-sided expected classification even though the immutable "
                "v8 capture established that both source predictions differ "
                "for the cold-dual case. This is a stale harness expectation, "
                "not a new Atlas execution."
            ),
        },
        "error": {
            "test": (
                "test_stager_submission_is_prerequisite_bound_single_job_"
                "and_fail_closed"
            ),
            "path": "hpc/test_math_weyl_context_core_capture.py",
            "line": 3130,
            "exception": "ValueError: substring not found",
            "missing_literal": "validate_regression_inputs(payload_root",
            "cause": (
                "The static ordering test searched for a single-line source "
                "substring, while the validated call is split after the "
                "opening parenthesis. The production validator was present "
                "and ran both before stage creation and after input installation."
            ),
        },
    }
    expected_not_reached = [
        "stager allowlist tests",
        "Rust and Cargo identity probes",
        "source reconstruction",
        "Cargo release build",
        "632-test inventory",
        "two named Weyl-context regression tests",
        "retained root-ladder control",
        "original Atlas reruns",
        "Rust Atlas execution",
        "mathematical comparison",
        "performance comparison",
    ]
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "inventory_bytes": 35377,
        "sha256": PREDECESSOR_STATE["stage_tree_sha256"],
        "files": PREDECESSOR_STATE["stage_tree_files"],
        "directories": PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": PREDECESSOR_STATE["stage_tree_bytes"],
        "stable_double_scan_and_independent_rescan_match": True,
    }
    expected_result_artifacts = {
        "files": 26,
        "inventory_schema": (
            "path,bytes,sha256,mode,nlink sorted by path; canonical compact JSON"
        ),
        "inventory_sha256":
            "d779210fe6541316809f231358c3929bba8cf60c10b92c4f43469837b8ce00dd",
        "unexpected_files": [],
        "symlinks": [],
        "multiply_linked_regular_files": [],
        "durable_source_target_workspace_or_build_directories": [],
        "incoming_entries": [],
    }
    expected_remediation = {
        "tests_first": True,
        "exact_test_fixes": [
            (
                "Update the synthetic complete-report expectation for the "
                "cold-dual capture from RUST_SOURCE_PREDICTION_DIFFERED to "
                "BOTH_SOURCE_PREDICTIONS_DIFFERED."
            ),
            (
                "Make the stager-order assertion match the existing multiline "
                "validate_regression_inputs call without changing its "
                "execution order."
            ),
        ],
        "production_rust_change_allowed": False,
        "regression_patch_or_fixture_change_allowed": False,
        "source_manifest_change_allowed": False,
        "expected_test_counts_after": BEFORE_FAILURE_PREDICTED_AFTER_COUNTS,
        "changed_input_successor_required": True,
        "successor_stage": "weyl-context-core-before-v2",
        "immutable_failed_stage": "weyl-context-core-before-v1",
        "no_same_stage_resubmission": True,
        "no_suffix_retry": True,
        "no_rank_escalation": True,
    }
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "passed_test_gates",
        "failed_gate", "not_reached", "final_stage_tree",
        "result_artifacts", "top_level_log", "mathematical_regression_required",
        "mathematical_regression_reason", "remediation_contract", "transport",
        "independent_review", "claims_not_granted",
    }
    if (set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-before-failure-v1"
            or value.get("observed_at_utc") != "2026-10-01T23:35:04Z"
            or value.get("classification")
               != "HARNESS_CAPTURE_DRIVER_TEST_EXPECTATION_AND_STATIC_ORDER_MISMATCH_BEFORE_ALLOWLIST_SOURCE_BUILD_OR_ATLAS_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("submission") != expected_submission
            or value.get("stage_creation") != expected_creation
            or value.get("report") != expected_report
            or value.get("passed_test_gates") != expected_passed
            or value.get("failed_gate") != expected_failed
            or value.get("not_reached") != expected_not_reached
            or value.get("final_stage_tree") != expected_tree
            or value.get("result_artifacts") != expected_result_artifacts
            or value.get("top_level_log") != {
                "path": "weyl-context-core-before-v1-3884862.out",
                "bytes": 134,
                "mode": "0644",
                "sha256": PREDECESSOR_STATE["stage_files"][
                    "weyl-context-core-before-v1-3884862.out"]["sha256"],
            }
            or value.get("mathematical_regression_required") is not False
            or value.get("mathematical_regression_reason")
               != ("No original or Rust Atlas process ran and no mathematical "
                   "result was produced. The two independently established "
                   "Weyl-context regressions and their original-backed fixtures "
                   "remain unchanged; this failure is confined to two Python "
                   "harness expectations.")
            or value.get("remediation_contract") != expected_remediation
            or value.get("transport") != {
                "manifest_sha256": PREDECESSOR["override_manifest_sha256"],
                "inputs": 50,
                "files_including_manifest": 51,
                "remote_path":
                    "/public/home/majj/.weyl-core-before-v1-payload",
                "remote_removed": True,
                "local_temporary_path_created": False,
            }
            or value.get("independent_review") != {
                "read_only_audits": 3,
                "latest_reconciliation_at_utc": "2026-10-01T23:35:04Z",
                "canonical_compact_facts_sha256":
                    "144083de41a530c3a1cd5ec98d104ea6432c4f87fc1d512700d2303ca9930774",
                "report_raw_stream_and_tree_hashes_reconciled": True,
                "durable_receipt_creation_events_and_ledger_tail_reconciled":
                    True,
                "all_stage_input_copies_reconciled": True,
                "current_queue_empty": True,
                "transport_absent": True,
                "status": "INDEPENDENT_READ_ONLY_AUDIT_COMPLETE",
            }
            or value.get("claims_not_granted") != [
                "capture-driver checker pass",
                "stager allowlist checker execution",
                "source reconstruction",
                "Cargo build",
                "original Atlas rerun",
                "Rust Atlas execution",
                "tests-first Weyl-context BEFORE reproduction",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core BEFORE v1 failure evidence changed")
    return copy.deepcopy(value)


def validate_before_v2_failure(root, inputs):
    """Bind the immutable BEFORE v2 two-error allowlist failure."""
    PREDECESSOR_STAGE = BEFORE_V2_PREDECESSOR_STAGE
    PREDECESSOR_STATE = BEFORE_V2_PREDECESSOR_STATE
    PREDECESSOR = BEFORE_V2_PREDECESSOR
    reference = BEFORE_V2_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core BEFORE v2 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "test_totals",
        "passed_test_gates", "failed_gate", "not_reached",
        "final_stage_tree", "result_artifacts", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "transport", "independent_review", "kimi",
        "claims_not_granted",
    }
    expected_submission = {
        "job": PREDECESSOR["job"],
        "state": "FAILED",
        "exit_code": "1:0",
        "elapsed_seconds": 56,
        "start": "2026-10-02T07:52:31",
        "end": "2026-10-02T07:53:27",
        "node": "cu001",
        "allocated_cpus": 2,
        "requested_memory": "8G",
        "batch_maxrss_kb": 86800,
        "queue_after_reconciliation": [],
        "ledger_records_after": PREDECESSOR["campaign_ledger_records"],
        "ledger_bytes_after": 6893,
        "ledger_sha256_after": PREDECESSOR["campaign_ledger_sha256"],
        "predecessor_ledger_records":
            BEFORE_V1_PREDECESSOR["campaign_ledger_records"],
        "predecessor_ledger_sha256":
            BEFORE_V1_PREDECESSOR["campaign_ledger_sha256"],
        "override_manifest_bytes": 6390,
        "override_manifest_sha256": PREDECESSOR["override_manifest_sha256"],
        "override_manifest_inputs": 51,
        "override_manifest_input_bytes": 1716695,
        "changed_inputs_from_predecessor": 8,
        "installed_and_override_copies_reconciled": 102,
        "pin_sha256": PREDECESSOR["pin_sha256"],
        "stage_creation_sha256": PREDECESSOR["stage_creation_sha256"],
        "stage_creation_contract_sha256":
            PREDECESSOR["stage_creation_contract_sha256"],
        "stage_creation_transaction_sha256":
            PREDECESSOR["stage_creation_transaction_sha256"],
        "submission_intent_sha256": PREDECESSOR["submission_intent_sha256"],
        "submission_receipt_sha256":
            PREDECESSOR["submission_receipt_sha256"],
        "submission_receipt_status": "SUBMITTED_NOT_VERIFIED",
    }
    scoped = ".atlas-stage-creation-weyl-context-core-before-v2-"
    expected_creation = {
        "prepared_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "prepared.json"]["sha256"],
        "prepared_bytes": 18839,
        "sealed_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "sealed.json"]["sha256"],
        "sealed_bytes": 589,
        "published_sha256": PREDECESSOR_STATE["campaign_files"][
            scoped + "published.json"]["sha256"],
        "published_bytes": 677,
        "target_device": PREDECESSOR_STATE["stage_device"],
        "target_inode": PREDECESSOR_STATE["stage_inode"],
        "target_mode": "0755",
        "publication_residuals_after": [],
    }
    expected_report = {
        "path": str(Path(PREDECESSOR_STAGE) / "results/3884880/report.json"),
        "bytes": PREDECESSOR["report_bytes"],
        "sha256": PREDECESSOR["report_sha256"],
        "report_sha256_file_bytes": 65,
        "report_sha256_file_sha256": PREDECESSOR_STATE["stage_files"][
            "results/3884880/report.sha256"]["sha256"],
        "status": "WEYL_CONTEXT_BEFORE_HARNESS_FAILURE",
        "evidence_maturity": "tests_first_before",
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
        "source_integrity_rechecked": False,
        "integrity_rechecked": False,
        "ephemeral_workspace_removed": True,
        "commands_recorded": 5,
        "failed_command_recorded": True,
        "source_files_recorded": 0,
        "binaries_recorded": 0,
        "atlas_invocations": 0,
        "captures_recorded": 0,
        "regression_classification_recorded": False,
        "cleanup_error_present": False,
        "legacy_path_open_attempts": [],
    }
    expected_gate_summary = [
        ("test-campaign-stage-creation", 32, 0),
        ("test-progressive-submit", 17, 0),
        ("test-weyl-context-core-contract", 18, 0),
        ("test-weyl-context-core-regression-contract", 17, 0),
        ("test-math-weyl-context-core-capture", 28, 0),
    ]
    passed = value.get("passed_test_gates")
    passed_summary = (
        [(item.get("name"), item.get("tests"), item.get("exit_status"))
         for item in passed]
        if isinstance(passed, list) and all(isinstance(item, dict)
                                            for item in passed)
        else None
    )
    failed = value.get("failed_gate")
    expected_errors = [
        {
            "test": "test_current_filename_and_role_snapshot",
            "path": "hpc/test_stager_allowlist.py",
            "test_line": 1785,
            "rejection_line": 1516,
            "exception": (
                "ValueError: launcher has a non-allowlisted module-level action"
            ),
        },
        {
            "test": "test_module_actions_and_canonical_entries_are_bounded",
            "path": "hpc/test_stager_allowlist.py",
            "test_line": 1827,
            "rejection_line": 1516,
            "exception": (
                "ValueError: launcher has a non-allowlisted module-level action"
            ),
        },
    ]
    expected_first_action = {
        "path": "hpc/stage_weyl_context_core_capture.py",
        "assignment_line": 2587,
        "expression_line": 2588,
        "assignment": "PREDECESSOR_CAMPAIGN_FILES",
        "form": "dictionary display with **V8_PREDECESSOR_CAMPAIGN_FILES unpacking",
        "cause": (
            "The frozen allowlist accepts literal dictionaries and pure "
            "dictionary union, but deliberately does not accept dictionary "
            "unpacking as a module-level pure expression. Both failing tests "
            "encounter this first non-allowlisted assignment."
        ),
    }
    expected_latent_action = {
        "path": "hpc/stage_weyl_context_core_capture.py",
        "assignment_line": 2949,
        "expression_line": 3024,
        "assignment": "STAGE_INPUT_NAMES",
        "form": "set display containing BEFORE_V1_FAILURE_EVIDENCE[\"file\"]",
        "cause": (
            "Independent static AST review found that the exact allowlist also "
            "rejects a module-level subscript expression. It was not reached "
            "because the earlier dictionary-unpack assignment failed first."
        ),
    }
    expected_tree = {
        "schema": "atlas-stage-tree-inventory-v1",
        "root_included": False,
        "entry_order": "relative path ascending",
        "canonical_json": "indent=2, sort_keys=true, trailing LF",
        "inventory_bytes": 36652,
        "sha256": PREDECESSOR_STATE["stage_tree_sha256"],
        "files": PREDECESSOR_STATE["stage_tree_files"],
        "directories": PREDECESSOR_STATE["stage_tree_directories"],
        "bytes": PREDECESSOR_STATE["stage_tree_bytes"],
        "stable_double_scan_and_independent_rescan_match": True,
    }
    expected_counts = BEFORE_FAILURE_PREDICTED_AFTER_COUNTS
    remediation = value.get("remediation_contract")
    if (not isinstance(value, dict) or set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-before-failure-v2"
            or value.get("observed_at_utc") != "2026-10-02T00:03:30Z"
            or value.get("classification")
               != "HARNESS_STAGER_ALLOWLIST_MODULE_ACTION_MISMATCH_BEFORE_SOURCE_BUILD_OR_ATLAS_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("submission") != expected_submission
            or value.get("stage_creation") != expected_creation
            or value.get("report") != expected_report
            or value.get("test_totals") != {
                "tests": 119, "passed": 117, "failures": 0,
                "errors": 2, "ignored": 0,
            }
            or passed_summary != expected_gate_summary
            or not isinstance(failed, dict)
            or failed.get("name") != "test-stager-allowlist"
            or failed.get("tests") != 7 or failed.get("passed") != 5
            or failed.get("failures") != 0 or failed.get("errors") != 2
            or failed.get("exit_status") != 1
            or failed.get("stderr_sha256") != PREDECESSOR_STATE[
                "stage_files"]["results/3884880/test-stager-allowlist.stderr"][
                    "sha256"]
            or failed.get("error_records") != expected_errors
            or failed.get("observed_first_rejected_action")
               != expected_first_action
            or failed.get("latent_next_rejected_action")
               != expected_latent_action
            or value.get("final_stage_tree") != expected_tree
            or value.get("top_level_log") != {
                "path": "weyl-context-core-before-v2-3884880.out",
                "bytes": 134,
                "mode": "0644",
                "sha256": PREDECESSOR_STATE["stage_files"][
                    "weyl-context-core-before-v2-3884880.out"]["sha256"],
            }
            or value.get("mathematical_regression_required") is not False
            or not isinstance(remediation, dict)
            or remediation.get("tests_first") is not True
            or remediation.get("expected_test_counts_after") != expected_counts
            or remediation.get("production_rust_change_allowed") is not False
            or remediation.get("regression_patch_or_fixture_change_allowed")
               is not False
            or remediation.get("source_manifest_change_allowed") is not False
            or remediation.get("changed_input_successor_required") is not True
            or remediation.get("successor_stage")
               != "weyl-context-core-before-v3"
            or remediation.get("immutable_failed_stage")
               != "weyl-context-core-before-v2"
            or remediation.get("no_same_stage_resubmission") is not True
            or remediation.get("no_suffix_retry") is not True
            or remediation.get("no_rank_escalation") is not True
            or value.get("transport") != {
                "manifest_sha256": PREDECESSOR["override_manifest_sha256"],
                "inputs": 51,
                "files_including_manifest": 52,
                "changed_inputs_from_predecessor": 8,
                "remote_path": "/public/home/majj/.weyl-core-before-v2-payload",
                "remote_removed": True,
                "local_temporary_path_created": False,
            }
            or value.get("independent_review") != {
                "read_only_audits": 6,
                "latest_reconciliation_at_utc": "2026-10-02T00:03:30Z",
                "canonical_compact_facts_bytes": 858,
                "canonical_compact_facts_sha256":
                    "490a1fbcb056356f18afe923d6da239ec8fc7bdf6772635ebbafd88c046578bf",
                "report_raw_stream_and_tree_hashes_reconciled": True,
                "durable_receipt_creation_events_and_ledger_tail_reconciled":
                    True,
                "all_stage_input_copies_reconciled": True,
                "current_queue_empty": True,
                "transport_absent": True,
                "status": "INDEPENDENT_READ_ONLY_AUDIT_COMPLETE",
            }
            or value.get("kimi") != {"invocations": 0}
            or value.get("claims_not_granted") != [
                "stager allowlist checker pass",
                "source reconstruction",
                "Cargo build",
                "original Atlas rerun",
                "Rust Atlas execution",
                "tests-first Weyl-context BEFORE reproduction",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core BEFORE v2 failure evidence changed")
    return copy.deepcopy(value)


def validate_before_v3_failure(root, inputs):
    """Bind the immutable BEFORE v3 historical-guard allowlist failure."""
    # This validator is inherited verbatim from the BEFORE-v4 launcher, where
    # the module-level PREDECESSOR named the before-v3 stage.  Here the
    # module-level PREDECESSOR is before-v4, so rebind the three names
    # locally to the frozen before-v3 identity; the body below stays
    # byte-identical to the HPC-verified original apart from the pinned
    # predicted-counts constant.
    PREDECESSOR = BEFORE_V3_PREDECESSOR
    PREDECESSOR_STAGE = BEFORE_V3_PREDECESSOR_STAGE
    PREDECESSOR_STATE = BEFORE_V3_PREDECESSOR_STATE
    reference = BEFORE_V3_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core BEFORE v3 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    expected_keys = {
        "schema", "observed_at_utc", "classification", "campaign", "stage",
        "submission", "stage_creation", "report", "test_totals",
        "passed_test_gates", "failed_gate", "root_cause", "not_reached",
        "final_stage_tree", "result_artifacts", "top_level_log",
        "mathematical_regression_required", "mathematical_regression_reason",
        "remediation_contract", "transport", "independent_review", "kimi",
        "claims_not_granted",
    }
    submission = value.get("submission")
    creation = value.get("stage_creation")
    report = value.get("report")
    passed = value.get("passed_test_gates")
    passed_summary = (
        [(item.get("name"), item.get("tests"), item.get("exit_status"))
         for item in passed]
        if isinstance(passed, list) and all(isinstance(item, dict)
                                            for item in passed)
        else None
    )
    failed = value.get("failed_gate")
    root_cause = value.get("root_cause")
    remediation = value.get("remediation_contract")
    review = value.get("independent_review")
    expected_counts = BEFORE_FAILURE_PREDICTED_AFTER_COUNTS
    if (not isinstance(value, dict) or set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-before-failure-v3"
            or value.get("observed_at_utc") != "2026-10-02T00:27:55Z"
            or value.get("classification")
               != "HARNESS_STAGER_ALLOWLIST_HISTORICAL_DRIVER_LITERAL_MISMATCH_BEFORE_SOURCE_BUILD_OR_ATLAS_EXECUTION"
            or value.get("campaign")
               != str(Path(PREDECESSOR_STAGE).parent.parent)
            or value.get("stage") != PREDECESSOR_STAGE
            or not isinstance(submission, dict)
            or submission.get("job") != PREDECESSOR["job"]
            or submission.get("state") != "FAILED"
            or submission.get("exit_code") != "1:0"
            or submission.get("elapsed_seconds") != 56
            or submission.get("ledger_records_after")
               != PREDECESSOR["campaign_ledger_records"]
            or submission.get("ledger_sha256_after")
               != PREDECESSOR["campaign_ledger_sha256"]
            or submission.get("override_manifest_sha256")
               != PREDECESSOR["override_manifest_sha256"]
            or submission.get("override_manifest_inputs") != 52
            or submission.get("installed_and_override_copies_reconciled")
               != 104
            or submission.get("pin_sha256") != PREDECESSOR["pin_sha256"]
            or submission.get("stage_creation_sha256")
               != PREDECESSOR["stage_creation_sha256"]
            or submission.get("stage_creation_contract_sha256")
               != PREDECESSOR["stage_creation_contract_sha256"]
            or submission.get("stage_creation_transaction_sha256")
               != PREDECESSOR["stage_creation_transaction_sha256"]
            or submission.get("submission_intent_sha256")
               != PREDECESSOR["submission_intent_sha256"]
            or submission.get("submission_receipt_sha256")
               != PREDECESSOR["submission_receipt_sha256"]
            or submission.get("submission_receipt_status")
               != "SUBMITTED_NOT_VERIFIED"
            or not isinstance(creation, dict)
            or creation.get("prepared_sha256") != PREDECESSOR_STATE[
                "campaign_files"][
                    ".atlas-stage-creation-weyl-context-core-before-v3-"
                    "prepared.json"]["sha256"]
            or creation.get("sealed_sha256") != PREDECESSOR_STATE[
                "campaign_files"][
                    ".atlas-stage-creation-weyl-context-core-before-v3-"
                    "sealed.json"]["sha256"]
            or creation.get("published_sha256") != PREDECESSOR_STATE[
                "campaign_files"][
                    ".atlas-stage-creation-weyl-context-core-before-v3-"
                    "published.json"]["sha256"]
            or creation.get("target_device")
               != PREDECESSOR_STATE["stage_device"]
            or creation.get("target_inode")
               != PREDECESSOR_STATE["stage_inode"]
            or creation.get("publication_residuals_after") != []
            or not isinstance(report, dict)
            or report.get("sha256") != PREDECESSOR["report_sha256"]
            or report.get("bytes") != PREDECESSOR["report_bytes"]
            or report.get("status") != "WEYL_CONTEXT_BEFORE_HARNESS_FAILURE"
            or report.get("complete") is not False
            or any(report.get(name) is not False for name in (
                "acceptance_eligible", "math_gate_released",
                "cache_gate_released", "performance_gate_released",
                "rank_gate_released", "source_integrity_rechecked",
                "integrity_rechecked"))
            or report.get("commands_recorded") != 5
            or report.get("source_files_recorded") != 0
            or report.get("binaries_recorded") != 0
            or report.get("atlas_invocations") != 0
            or report.get("captures_recorded") != 0
            or report.get("regression_classification_recorded") is not False
            or value.get("test_totals") != {
                "tests": 119, "passed": 117, "failures": 0,
                "errors": 2, "ignored": 0,
            }
            or passed_summary != [
                ("test-campaign-stage-creation", 32, 0),
                ("test-progressive-submit", 17, 0),
                ("test-weyl-context-core-contract", 18, 0),
                ("test-weyl-context-core-regression-contract", 17, 0),
                ("test-math-weyl-context-core-capture", 28, 0),
            ]
            or not isinstance(failed, dict)
            or failed.get("name") != "test-stager-allowlist"
            or failed.get("tests") != 7 or failed.get("passed") != 5
            or failed.get("failures") != 0 or failed.get("errors") != 2
            or failed.get("exit_status") != 1
            or failed.get("stderr_sha256") != PREDECESSOR_STATE[
                "stage_files"][
                    "results/3884903/test-stager-allowlist.stderr"]["sha256"]
            or [item.get("test") for item in failed.get("error_records", [])]
               != [
                   "test_current_filename_and_role_snapshot",
                   "test_module_actions_and_canonical_entries_are_bounded",
               ]
            or not isinstance(root_cause, dict)
            or root_cause.get("rejected_launcher")
               != "hpc/math_ladder_boundary_before.py"
            or root_cause.get("rejected_node")
               != "top-level campaign guard"
            or root_cause.get("rejected_node_lines") != [55, 56]
            or root_cause.get("historical_driver_actual_message")
               != "ladder BEFORE v2 campaign policy changed"
            or root_cause.get("allowlist_incorrect_expected_message")
               != "ladder BEFORE v3 campaign policy changed"
            or root_cause.get("not_a_rust_mathematical_failure") is not True
            or root_cause.get("active_v3_stager_non_pure_module_assignments")
               != []
            or root_cause.get("active_v3_driver_non_pure_module_assignments")
               != []
            or value.get("not_reached") != [
                "rustc-version", "cargo-version", "source-reconstruction",
                "release-build", "atlas-core-test-inventory",
                "weyl-context-regressions", "root-ladder-control",
            ]
            or value.get("final_stage_tree", {}).get("sha256")
               != PREDECESSOR_STATE["stage_tree_sha256"]
            or value.get("final_stage_tree", {}).get("files")
               != PREDECESSOR_STATE["stage_tree_files"]
            or value.get("final_stage_tree", {}).get("directories")
               != PREDECESSOR_STATE["stage_tree_directories"]
            or value.get("final_stage_tree", {}).get("bytes")
               != PREDECESSOR_STATE["stage_tree_bytes"]
            or value.get("top_level_log", {}).get("sha256")
               != PREDECESSOR_STATE["stage_files"][
                   "weyl-context-core-before-v3-3884903.out"]["sha256"]
            or value.get("mathematical_regression_required") is not False
            or not isinstance(remediation, dict)
            or remediation.get("tests_first") is not True
            or remediation.get("expected_test_counts_after") != expected_counts
            or remediation.get("production_rust_change_allowed") is not False
            or remediation.get("regression_patch_or_fixture_change_allowed")
               is not False
            or remediation.get("source_manifest_change_allowed") is not False
            or remediation.get("historical_driver_change_allowed") is not False
            or remediation.get("allowlist_weakening_allowed") is not False
            or remediation.get("changed_input_successor_required") is not True
            or remediation.get("successor_stage")
               != "weyl-context-core-before-v4"
            or remediation.get("immutable_failed_stage")
               != "weyl-context-core-before-v3"
            or remediation.get("no_same_stage_resubmission") is not True
            or remediation.get("no_suffix_retry") is not True
            or remediation.get("no_rank_escalation") is not True
            or value.get("transport") != {
                "manifest_sha256": PREDECESSOR["override_manifest_sha256"],
                "inputs": 52,
                "files_including_manifest": 53,
                "changed_inputs_from_predecessor": 8,
                "compressed_payload_bytes": 159530,
                "remote_path": "/public/home/majj/.weyl-core-before-v3-payload",
                "remote_removed": True,
                "local_temporary_path_created": False,
            }
            or not isinstance(review, dict)
            or review.get("remote_snapshot_facts_sha256")
               != "7158d01e9dcb3ff94fac20f854d107b57bf4fcdca3442391afdff38735d4197d"
            or review.get("report_raw_stream_and_tree_hashes_reconciled")
               is not True
            or review.get(
                "durable_receipt_creation_events_and_ledger_tail_reconciled")
               is not True
            or review.get("all_104_stage_input_copies_reconciled") is not True
            or review.get("current_queue_empty") is not True
            or review.get("transport_absent") is not True
            or value.get("kimi") != {"invocations": 0}
            or value.get("claims_not_granted") != [
                "stager allowlist checker pass",
                "source reconstruction",
                "Cargo build",
                "original Atlas rerun",
                "Rust Atlas execution",
                "tests-first Weyl-context BEFORE reproduction",
                "Weyl-context mathematical correctness",
                "cache correctness",
                "performance improvement",
                "rank escalation",
            ]):
        raise ValueError("Weyl core BEFORE v3 failure evidence changed")
    return copy.deepcopy(value)


def validate_before_v4_result(root, inputs):
    """Bind the retained BEFORE v4 result: the direct predecessor's success."""
    # This validator is inherited from the after-v1 module, where the
    # module-level PREDECESSOR named the before-v4 stage.  Here PREDECESSOR
    # has advanced to the failed after-v2 stage, so rebind the two names
    # locally to the frozen before-v4 identity; the body below is unchanged.
    PREDECESSOR = BEFORE_V4_PREDECESSOR
    PREDECESSOR_STAGE = BEFORE_V4_PREDECESSOR_STAGE
    reference = BEFORE_V4_RESULT_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core BEFORE v4 result evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core BEFORE v4 result evidence changed")
    expected_keys = {
        "schema", "observed_at_utc", "classification", "job", "stage",
        "report", "checker_suites", "commands", "captures",
        "report_classification", "integrity", "final_state", "scope",
        "limitations",
    }
    job = value.get("job")
    report = value.get("report")
    checker = value.get("checker_suites")
    commands = value.get("commands")
    captures = value.get("captures")
    classification = value.get("report_classification")
    integrity = value.get("integrity")
    final_state = value.get("final_state")
    scope = value.get("scope")
    if (not isinstance(value, dict) or set(value) != expected_keys
            or value.get("schema")
               != "atlas-weyl-context-core-before-inspection-v4"
            or value.get("classification")
               != "INDEPENDENT_INSPECTION_PASS_TESTS_FIRST_BEFORE_RETAINED"
            or not isinstance(job, dict)
            or job.get("id") != PREDECESSOR["job"]
            or job.get("state") != "COMPLETED"
            or job.get("exit_code") != "0:0"
            or job.get("elapsed_seconds") != 450
            or job.get("node") != "cu081"
            or value.get("stage") != PREDECESSOR_STAGE
            or not isinstance(report, dict)
            or report.get("sha256") != PREDECESSOR["report_sha256"]
            or report.get("status")
               != "WEYL_CONTEXT_BEFORE_EXPECTED_FAILURES_OBSERVED"
            or report.get("complete") is not True
            or report.get("acceptance_eligible") is not False
            or report.get("evidence_maturity") != "tests_first_before"
            or not isinstance(checker, dict)
            or checker.get("total") != BEFORE_V4_CHECKER_TOTAL
            or checker.get("all_ok") is not True
            or not isinstance(commands, dict)
            or commands.get("weyl_context_regressions", {}).get(
                "exit_status") != 101
            or commands.get("weyl_context_regressions", {}).get("failed") != 2
            or commands.get("weyl_context_regressions", {}).get(
                "filtered_out") != 630
            or commands.get("weyl_context_regressions", {}).get(
                "failed_tests") != [
                "session::tests::weyl_context_core_cold_dual_original",
                "session::tests::weyl_context_core_prewarmed_dual_original",
            ]
            or commands.get("root_ladder_control", {}).get("exit_status") != 0
            or not isinstance(captures, dict)
            or captures.get("fresh_processes") != 4
            or not isinstance(classification, dict)
            or classification.get("expected_regressions_observed") is not True
            or classification.get("original_goldens_matched") is not True
            or classification.get("retained_control_passed") is not True
            or classification.get("unexpected_regressions_passed") is not False
            or classification.get("inventory_complete") is not True
            or classification.get("metrics_complete") is not True
            or not isinstance(integrity, dict)
            or integrity.get("source_integrity_rechecked") is not True
            or integrity.get("integrity_rechecked") is not True
            or integrity.get("ephemeral_workspace_removed") is not True
            or integrity.get("legacy_path_open_attempts") != []
            or integrity.get("production_change") is not False
            or not isinstance(final_state, dict)
            or final_state.get("queue_empty_after") is not True
            or final_state.get("ledger_records")
               != PREDECESSOR["campaign_ledger_records"]
            or final_state.get("ledger_sha256")
               != PREDECESSOR["campaign_ledger_sha256"]
            or final_state.get("pin_sha256") != PREDECESSOR["pin_sha256"]
            or final_state.get("stage_creation_sha256")
               != PREDECESSOR["stage_creation_sha256"]
            or final_state.get("transports_absent") is not True
            or not isinstance(scope, dict)
            or not isinstance(scope.get("releases"), str)
            or not scope["releases"]
            or (not isinstance(scope.get("not_granted"), list)
                or not scope["not_granted"]
                or any(not isinstance(item, str) or not item
                       for item in scope["not_granted"]))
            or not isinstance(value.get("limitations"), list)
            or not value["limitations"]):
        raise ValueError("Weyl core BEFORE v4 result evidence changed")
    return copy.deepcopy(value)


def validate_after_v1_failure(root, inputs):
    """Bind the immutable after-v1 harness failure (job 3890328)."""
    # This validator was added in the after-v2 module, where the module-level
    # PREDECESSOR named the after-v1 stage.  Here PREDECESSOR is the failed
    # after-v3 stage, so rebind the two names locally to the frozen after-v1
    # identity; the body below is unchanged except that the reconstruction
    # manifest comparison uses the frozen v1-era manifest (the source changed
    # at after-v4).
    PREDECESSOR = AFTER_V1_PREDECESSOR
    PREDECESSOR_STAGE = AFTER_V1_PREDECESSOR_STAGE
    reference = AFTER_V1_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core after-v1 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core after-v1 failure evidence changed")
    reconstruction = value.get("evidence_of_correct_reconstruction") \
        if isinstance(value.get("evidence_of_correct_reconstruction"), dict) \
        else None
    if (value.get("schema") != "atlas-weyl-context-core-after-failure-v1"
            or value.get("job") != PREDECESSOR["job"]
            or value.get("state") != "FAILED"
            or value.get("report_status")
               != "WEYL_CONTEXT_AFTER_HARNESS_FAILURE"
            or value.get("classification")
               != ("HARNESS_DRIVER_EXPECTATION_MISMATCH_AFTER_SOURCE_"
                   "RECONSTRUCTION_BEFORE_BUILD_OR_ATLAS_EXECUTION")
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("pin_sha256") != PREDECESSOR["pin_sha256"]
            or value.get("stage_creation_sha256")
               != PREDECESSOR["stage_creation_sha256"]
            or value.get("report_sha256") != PREDECESSOR["report_sha256"]
            or value.get("report_bytes") != PREDECESSOR["report_bytes"]
            or value.get("mathematical_regression_required") is not False
            or not isinstance(reconstruction, dict)
            or reconstruction.get("computed_manifest_sha256")
               != AFTER_V1_ERA_SOURCE_MANIFEST_SHA256
            or reconstruction.get("expected_after_manifest_sha256")
               != AFTER_V1_ERA_SOURCE_MANIFEST_SHA256
            or reconstruction.get("match") is not True
            or value.get("submission_record")
               != ("tests/reference/hpc/"
                   "math_weyl_context_core_after_v1_submission_2026_10_03.json")):
        raise ValueError("Weyl core after-v1 failure evidence changed")
    return copy.deepcopy(value)


def validate_after_v2_failure(root, inputs):
    """Bind the immutable after-v2 harness failure (job 3890580)."""
    # This validator was added in the after-v3 module, where the module-level
    # PREDECESSOR named the after-v2 stage.  Here PREDECESSOR is the failed
    # after-v3 stage, so rebind the two names locally to the frozen after-v2
    # identity; the body below is unchanged.
    PREDECESSOR = AFTER_V2_PREDECESSOR
    PREDECESSOR_STAGE = AFTER_V2_PREDECESSOR_STAGE
    reference = AFTER_V2_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core after-v2 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core after-v2 failure evidence changed")
    if (value.get("schema") != "atlas-weyl-context-core-after-failure-v2"
            or value.get("job") != PREDECESSOR["job"]
            or value.get("state") != "FAILED"
            or value.get("classification")
               != ("HARNESS_SBATCH_LABEL_MISMATCH_AT_TOPOLOGY_"
                   "VALIDATION_BEFORE_ANY_GATE")
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("pin_sha256") != PREDECESSOR["pin_sha256"]
            or value.get("stage_creation_sha256")
               != PREDECESSOR["stage_creation_sha256"]
            or value.get("report") is not None
            or value.get("out_sha256") != AFTER_V2_PREDECESSOR_OUT["sha256"]
            or value.get("out_bytes") != AFTER_V2_PREDECESSOR_OUT["bytes"]
            or value.get("mathematical_regression_required") is not False):
        raise ValueError("Weyl core after-v2 failure evidence changed")
    return copy.deepcopy(value)


def validate_after_v3_failure(root, inputs):
    """Bind the immutable after-v3 harness failure (job 3899303)."""
    # This validator was added in the after-v4 module, where the module-level
    # PREDECESSOR named the after-v3 stage.  Here PREDECESSOR is the failed
    # after-v4 stage, so rebind the two names locally to the frozen after-v3
    # identity; the body below is unchanged.
    PREDECESSOR = AFTER_V3_PREDECESSOR
    PREDECESSOR_STATE = AFTER_V3_PREDECESSOR
    reference = AFTER_V3_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core after-v3 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core after-v3 failure evidence changed")
    failed_command = value.get("failed_command") \
        if isinstance(value.get("failed_command"), dict) else None
    if (value.get("schema") != "atlas-weyl-context-core-after-failure-v3"
            or value.get("status")
               != "FINAL_FAILED_HARNESS_BUILD_E0609_REPAIR_INCOMPLETE"
            or value.get("job") != PREDECESSOR["job"]
            or value.get("final_state") != "FAILED 1:0 (batch); extern COMPLETED 0:0"
            or value.get("stage") != PREDECESSOR["stage"]
            or value.get("slurm_out_sha256")
               != AFTER_V3_PREDECESSOR_OUT["sha256"]
            or not isinstance(value.get("report"), dict)
            or value["report"].get("sha256") != PREDECESSOR["report_sha256"]
            or value["report"].get("status_field")
               != "WEYL_CONTEXT_AFTER_HARNESS_FAILURE"
            or value["report"].get("ephemeral_workspace_removed") is not True
            or not isinstance(failed_command, dict)
            or failed_command.get("name") != "release-build"
            or failed_command.get("exit_status") != 101
            or value.get("successor_guidance") is None):
        raise ValueError("Weyl core after-v3 failure evidence changed")
    return copy.deepcopy(value)


def validate_after_v4_failure(root, inputs):
    """Bind the immutable after-v4 harness failure (job 3899885)."""
    reference = AFTER_V4_FAILURE_EVIDENCE
    if inputs.get(reference["file"]) != reference["sha256"]:
        raise ValueError("Weyl core after-v4 failure evidence hash changed")
    value = load_relative_json(
        root, reference["file"], reference["sha256"], mode=0o444, nlink=1)
    if not isinstance(value, dict):
        raise ValueError("Weyl core after-v4 failure evidence changed")
    report = value.get("report") \
        if isinstance(value.get("report"), dict) else None
    failed_gate = value.get("failed_gate") \
        if isinstance(value.get("failed_gate"), dict) else None
    outcome = value.get("mathematical_outcome_validated_by_the_report") \
        if isinstance(
            value.get("mathematical_outcome_validated_by_the_report"), dict) \
        else None
    if (value.get("schema") != "atlas-weyl-context-core-after-failure-v4"
            or value.get("status")
               != ("FINAL_FAILED_HARNESS_GATE_OVERASSERTION_PREWARMED_"
                   "STDERR_PRESENTATION")
            or value.get("job") != PREDECESSOR["job"]
            or value.get("stage") != PREDECESSOR_STAGE
            or value.get("slurm_out_sha256") != PREDECESSOR_STATE[
                "stage_files"][
                "weyl-context-core-after-v4-3899885.out"]["sha256"]
            or not isinstance(report, dict)
            or report.get("sha256") != PREDECESSOR["report_sha256"]
            or report.get("status_field")
               != "WEYL_CONTEXT_AFTER_HARNESS_FAILURE"
            or report.get("ephemeral_workspace_removed") is not True
            or not isinstance(outcome, dict)
            or outcome.get("expected_regressions_passed") is not True
            or outcome.get("original_goldens_matched") is not True
            or outcome.get("retained_control_passed") is not True
            or outcome.get("inventory_complete") is not True
            or outcome.get("classification_status")
               != "WEYL_CONTEXT_AFTER_REGRESSIONS_PASS"
            or not isinstance(failed_gate, dict)
            or failed_gate.get("oracle_golden_stderr_sha256")
               != ("ef4404d85f7252a611f9f4e4a5205fe6bbac2c66f48b7378a30a805"
                   "3f986e157")
            or value.get("successor_guidance") is None):
        raise ValueError("Weyl core after-v4 failure evidence changed")
    return copy.deepcopy(value)


def validate_regression_inputs(root, inputs, source_manifest):
    """Derive a tests-only manifest without extracting or executing source."""
    if (not isinstance(source_manifest, dict)
            or len(source_manifest) != ACCEPTED_SOURCE["final_files"]
            or canonical_json_sha(source_manifest)
               != ACCEPTED_SOURCE["source_manifest_sha256"]
            or REGRESSION_SOURCE["parent_files"] != ACCEPTED_SOURCE["final_files"]
            or REGRESSION_SOURCE["parent_source_manifest_sha256"]
               != ACCEPTED_SOURCE["source_manifest_sha256"]):
        raise ValueError("regression base is not the accepted source manifest")
    catalog_raw = stable_relative_bytes(
        root, REGRESSION_CATALOG_PATH, mode=0o444, nlink=1)
    inspection_raw = stable_relative_bytes(
        root, REGRESSION_INSPECTION_PATH, mode=0o444, nlink=1)
    if (inputs.get(REGRESSION_CATALOG_PATH) != REGRESSION_CATALOG_SHA256
            or inputs.get(REGRESSION_INSPECTION_PATH)
               != REGRESSION_INSPECTION_SHA256):
        raise ValueError("regression evidence input binding changed")
    catalog = regression_contract.decode_catalog(catalog_raw)
    regression_contract.decode_inspection(inspection_raw, catalog)
    artifacts = {}
    for name, wanted in REGRESSION_FIXTURE_HASHES.items():
        raw = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        if (inputs.get(name) != wanted
                or hashlib.sha256(raw).hexdigest() != wanted
                or len(raw) != REGRESSION_FIXTURE_BYTES.get(name)
                or name in source_manifest):
            raise ValueError("regression fixture identity or base collision changed")
        basename = PurePosixPath(name).name
        if basename in artifacts:
            raise ValueError("regression fixture basenames are not unique")
        artifacts[basename] = raw
    regression_contract.validate_artifacts(catalog, artifacts)
    for name, wanted in REGRESSION_PATCH_HASHES.items():
        raw = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        if (inputs.get(name) != wanted or hashlib.sha256(raw).hexdigest() != wanted
                or name != REGRESSION_PATCH_PATH or len(raw) != REGRESSION_PATCH_BYTES):
            raise ValueError("regression test patch changed")
    if (set(REGRESSION_SOURCE_HASHES) != set(FINAL_HASHES)
            or REGRESSION_SOURCE_HASHES[
                "crates/atlas-real-group/src/root_system.rs"]
               != FINAL_HASHES["crates/atlas-real-group/src/root_system.rs"]
            or any(source_manifest.get(name) != wanted
                   for name, wanted in FINAL_HASHES.items())):
        raise ValueError("regression source changed outside the session test block")
    result = dict(source_manifest)
    result.update(REGRESSION_SOURCE_HASHES)
    result.update(REGRESSION_FIXTURE_HASHES)
    if (len(result) != REGRESSION_SOURCE["files"]
            or canonical_json_sha(result)
               != REGRESSION_SOURCE["source_manifest_sha256"]):
        raise ValueError("tests-only regression source manifest changed")
    return result


def validate_parent_objects(campaign):
    raw = read_blob(campaign, PARENT_SEAL_REFERENCE)
    try:
        seal = strict_json_loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid parent seal JSON") from error
    if (not isinstance(seal, dict)
            or seal.get("schema") != "atlas-weyl-context-parent-seal-v1"
            or seal.get("status") != "WEYL_CONTEXT_PARENT_SEALED"
            or seal.get("source_object") != PARENT_SOURCE_OBJECT
            or seal.get("source_file_count") != ACCEPTED_SOURCE["parent_files"]
            or seal.get("source_manifest_sha256")
               != ACCEPTED_SOURCE["parent_source_manifest_sha256"]
            or seal.get("binaries", {}).get("oracle", {}).get("sha256")
               != ACCEPTED_SOURCE["oracle"]["binary_sha256"]):
        raise ValueError("accepted parent seal changed")
    verify_source_archive(
        campaign, PARENT_SOURCE_OBJECT, seal.get("source_manifest"))
    verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)
    return copy.deepcopy(PARENT_SOURCE_OBJECT)


def _literal_submission_enabled(raw, label):
    try:
        tree = ast.parse(raw.decode("utf-8"), filename=label)
    except (UnicodeDecodeError, SyntaxError) as error:
        raise ValueError("invalid launcher source: " + label) from error
    values = []
    for node in tree.body:
        if (isinstance(node, ast.Assign) and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name)
                and node.targets[0].id == "SUBMISSION_ENABLED"):
            values.append(
                node.value.value if isinstance(node.value, ast.Constant)
                and type(node.value.value) is bool else None)
    if len(values) != 1 or type(values[0]) is not bool:
        raise ValueError("launcher lacks one literal enable flag: " + label)
    return values[0]


def _parent_launcher_is_retired(raw, label):
    try:
        tree = ast.parse(raw.decode("utf-8"), filename=label)
    except (UnicodeDecodeError, SyntaxError) as error:
        raise ValueError("invalid parent launcher source: " + label) from error
    mains = [
        node for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name == "main"
    ]
    if len(mains) != 1 or not mains[0].body:
        raise ValueError("parent launcher has no unique main")
    first = mains[0].body[0]
    error = first.exc if isinstance(first, ast.Raise) else None
    if (not isinstance(error, ast.Call)
            or not isinstance(error.func, ast.Name)
            or error.func.id != "SystemExit"
            or len(error.args) != 1 or error.keywords
            or not isinstance(error.args[0], ast.Constant)
            or error.args[0].value != "parent seal is closed"):
        raise ValueError("parent launcher is not retired")
    return "retired"


def _running_stager_sha256():
    path = Path(__file__).resolve()
    if path.name != PurePosixPath(CURRENT_STAGER_PATH).name:
        raise ValueError("running Weyl capture stager path changed")
    root = path.parent.parent
    raw = stable_relative_bytes(root, CURRENT_STAGER_PATH)
    return hashlib.sha256(raw).hexdigest()


def validate_launcher_transition(root, inputs):
    if (set(FROZEN_LAUNCHER_HASHES) | {CURRENT_STAGER_PATH}
            != set(FROZEN_LAUNCHER_STATES)
            or not set(FROZEN_LAUNCHER_STATES) <= STAGE_INPUT_NAMES):
        raise ValueError("Weyl core launcher trust set changed")
    expected_hashes = dict(FROZEN_LAUNCHER_HASHES)
    expected_hashes[CURRENT_STAGER_PATH] = _running_stager_sha256()
    states = {}
    for name in FROZEN_LAUNCHER_STATES:
        raw = stable_relative_bytes(root, name)
        observed = hashlib.sha256(raw).hexdigest()
        if observed != inputs.get(name) or observed != expected_hashes[name]:
            raise ValueError("launcher source changed: " + name)
        states[name] = (
            _parent_launcher_is_retired(raw, name)
            if name == "hpc/stage_weyl_parent_seal.py"
            else _literal_submission_enabled(raw, name)
        )
    if states != FROZEN_LAUNCHER_STATES:
        raise ValueError("Weyl core capture is not the sole enabled launcher")
    return copy.deepcopy(states)


def validate_configuration(inputs, overrides_sha256, test_counts):
    if (not isinstance(inputs, dict) or set(inputs) != STAGE_INPUT_NAMES
            or any(not isinstance(name, str)
                   or not isinstance(value, str)
                   or re.fullmatch(SHA256_PATTERN, value) is None
                   for name, value in inputs.items())):
        raise ValueError("Weyl core capture input hashes are not frozen")
    if (not isinstance(overrides_sha256, str)
            or re.fullmatch(SHA256_PATTERN, overrides_sha256) is None):
        raise ValueError("Weyl core capture override manifest is not frozen")
    expected_hashes = {
        **PATCH_HASHES,
        **REGRESSION_PATCH_HASHES,
        **REPAIR_PATCH_HASHES,
        **REGRESSION_FIXTURE_HASHES,
        REGRESSION_CATALOG_PATH: REGRESSION_CATALOG_SHA256,
        REGRESSION_INSPECTION_PATH: REGRESSION_INSPECTION_SHA256,
        V8_SUBMISSION_EVIDENCE_PATH: V8_SUBMISSION_EVIDENCE_SHA256,
        BEFORE_V1_FAILURE_EVIDENCE["file"]:
            BEFORE_V1_FAILURE_EVIDENCE["sha256"],
        BEFORE_V2_FAILURE_EVIDENCE["file"]:
            BEFORE_V2_FAILURE_EVIDENCE["sha256"],
        BEFORE_V3_FAILURE_EVIDENCE["file"]:
            BEFORE_V3_FAILURE_EVIDENCE["sha256"],
        BEFORE_V4_RESULT_EVIDENCE["file"]:
            BEFORE_V4_RESULT_EVIDENCE["sha256"],
    }
    if any(inputs.get(name) != wanted for name, wanted in expected_hashes.items()):
        raise ValueError("Weyl core capture patch bytes changed")
    return copy.deepcopy(inputs), validate_test_counts(test_counts)


def validate_stage_creation_reference(value):
    if (not isinstance(value, dict)
            or set(value) != {"receipt_sha256", "contract_sha256"}
            or any(not isinstance(item, str)
                   or re.fullmatch(SHA256_PATTERN, item) is None
                   for item in value.values())):
        raise ValueError("Weyl core stage-creation reference changed")
    return copy.deepcopy(value)


def build_stage_creation_contract(campaign, inputs, overrides_sha256):
    frozen, _ = validate_configuration(
        inputs, overrides_sha256, EXPECTED_TEST_COUNTS)
    campaign = Path(campaign)
    expected_campaign = Path(PREDECESSOR_STAGE).parent.parent
    if (not campaign.is_absolute() or campaign != expected_campaign
            or campaign.name != "atlas-rust-campaign-20260930"):
        raise ValueError("Weyl core stage creator names another campaign")
    return {
        "schema": STAGE_CREATION_CONTRACT_SCHEMA,
        "campaign": str(campaign),
        "stage_name": STAGE_NAME,
        "predecessor_ledger_sha256": PREDECESSOR["campaign_ledger_sha256"],
        "predecessor_state": copy.deepcopy(PREDECESSOR_STATE),
        "overrides_sha256": overrides_sha256,
        "inputs": frozen,
        "script": {"path": SBATCH, "sha256": frozen[SBATCH]},
        "pin": {
            "path": PIN_NAME,
            "schema": PIN_SCHEMA,
            "stage_creation_key": "stage_creation",
        },
        "lifecycle": copy.deepcopy(LIFECYCLE),
    }


def build_pin(inputs, overrides_sha256, test_counts, stage_creation):
    frozen, counts = validate_configuration(
        inputs, overrides_sha256, test_counts)
    return {
        "schema": PIN_SCHEMA,
        "inputs": frozen,
        "overrides_sha256": overrides_sha256,
        "test_counts": counts,
        "predecessor": copy.deepcopy(PREDECESSOR),
        "accepted_source": copy.deepcopy(ACCEPTED_SOURCE),
        "regression": copy.deepcopy(REGRESSION_RECORD),
        "parent_seal_object": copy.deepcopy(PARENT_SEAL_REFERENCE),
        "retired_stager_object": copy.deepcopy(
            RETIRED_STAGER_BUNDLE_REFERENCE),
        "catalog": {
            "file": CATALOG_PATH,
            "sha256": CATALOG_SHA256,
            "cases": copy.deepcopy(CATALOG_CASES),
            "fresh_processes": 4,
        },
        "lifecycle": copy.deepcopy(LIFECYCLE),
        "stage_creation": validate_stage_creation_reference(stage_creation),
        "scope": (
            "Tests-first A1 Weyl owner/dual BEFORE: unchanged production, "
            "two original-backed regression failures, one retained ladder "
            "control and four fresh original/Rust CLI processes. No "
            "mathematical, cache, performance, parallel, rank or index release."
        ),
    }


def validate_pin(pin):
    if not isinstance(pin, dict) or set(pin) != PIN_KEYS:
        raise ValueError("changed Weyl core capture pin")
    try:
        expected = build_pin(
            pin["inputs"], pin["overrides_sha256"], pin["test_counts"],
            pin["stage_creation"])
    except (KeyError, TypeError, ValueError) as error:
        raise ValueError("changed Weyl core capture pin") from error
    if pin != expected:
        raise ValueError("changed Weyl core capture pin")
    return copy.deepcopy(pin)


def validate_submission_record(record, pin, root=None):
    validate_pin(pin)
    if not isinstance(record, dict) or set(record) != SUBMISSION_RECORD_KEYS:
        raise ValueError("Weyl core capture submission record is not exact")
    try:
        validate_confirmed_history([record])
    except ValueError as error:
        raise ValueError("Weyl core capture submission is not confirmed") from error
    stage = Path(record["stage"])
    if (not stage.is_absolute() or str(stage) != record["stage"]
            or ".." in stage.parts or stage.name != STAGE_NAME
            or record.get("script") != SBATCH
            or record.get("pin_sha256") != saved_json_sha(pin)
            or record.get("stage_creation_sha256")
               != pin["stage_creation"]["receipt_sha256"]):
        raise ValueError("Weyl core capture submission record changed")
    if root is not None and stage != Path(root).resolve():
        raise ValueError("submission record names another stage")
    return copy.deepcopy(record)


def validate_campaign_history(history, root, require_own=False):
    """Require the eighteen-record BEFORE v3 predecessor plus this v4 child."""
    if type(require_own) is not bool:
        raise ValueError("invalid campaign-history requirement")
    validate_confirmed_history(history)
    root = str(Path(root).resolve())
    matches = [row for row in history if row.get("stage") == root]
    if len(matches) > 1 or (require_own and len(matches) != 1):
        raise ValueError("campaign ledger has invalid capture-stage multiplicity")
    if matches:
        if history[-1] != matches[0]:
            raise ValueError("Weyl capture is not the direct campaign successor")
        predecessor = history[:-1]
    else:
        predecessor = history
    if saved_json_sha(predecessor) != PREDECESSOR["campaign_ledger_sha256"]:
        raise ValueError("campaign ledger predecessor changed")
    return copy.deepcopy(matches[0]) if matches else None


def submission_receipt(record, pin, root=None):
    record = validate_submission_record(record, pin, root=root)
    return copy.deepcopy(dict(
        record,
        schema="atlas-weyl-context-core-before-submission-v4",
        status="SUBMITTED_NOT_VERIFIED",
        test_counts=pin["test_counts"],
        predecessor=pin["predecessor"],
        accepted_source=pin["accepted_source"],
        regression=pin["regression"],
        parent_seal_object=pin["parent_seal_object"],
        retired_stager_object=pin["retired_stager_object"],
        catalog=pin["catalog"],
        lifecycle=pin["lifecycle"],
        scope=pin["scope"],
        cargo_builds=1,
        cargo_test_commands=3,
        atlas_processes=4,
        fresh_processes=4,
        acceptance_eligible=False,
        math_gate_released=False,
        cache_gate_released=False,
        performance_gate_released=False,
        rank_gate_released=False,
        storage_policy=(
            "One existing-campaign child. Expanded source, Cargo target, "
            "materialized binaries and scripts are job-local and disposable."
        ),
    ))


@contextmanager
def existing_lock(directory, name):
    """Lock one existing inode; receipt recovery never creates a lock."""
    directory_descriptor = descriptor = current_descriptor = None
    try:
        directory_descriptor = _open_directory(directory)
        flags = (os.O_RDWR | getattr(os, "O_CLOEXEC", 0)
                 | getattr(os, "O_NOFOLLOW", 0))
        descriptor = os.open(name, flags, dir_fd=directory_descriptor)
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("existing submission lock is unsafe")
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        current_descriptor = os.open(
            name, flags, dir_fd=directory_descriptor)
        current = os.fstat(current_descriptor)
        identity = lambda value: (
            value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode),
            value.st_nlink,
        )
        if identity(current) != identity(opened):
            raise ValueError("existing submission lock changed while acquired")
        yield descriptor
    except OSError as error:
        raise ValueError("existing submission lock could not be acquired") from error
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)
        if directory_descriptor is not None:
            os.close(directory_descriptor)


def confirmed_record(root, pin, repair_intent=False, expected_job=None):
    if type(repair_intent) is not bool:
        raise ValueError("invalid capture-intent repair request")
    if (expected_job is not None
            and (not isinstance(expected_job, str)
                 or not expected_job.isdecimal())):
        raise ValueError("compute allocation is not a decimal job ID")
    root = Path(root).resolve()
    campaign = submission_scope(root)
    ledger_path = campaign / ".atlas-progressive-submit.json"
    intent_path = root / "submission-intent.json"
    receipt_path = root / "submission.json"
    with existing_lock(campaign, ".atlas-progressive-submit.lock"):
        history = read_json_file(ledger_path)
        intent = read_json_file(intent_path)
        uncertain_keys = SUBMISSION_RECORD_KEYS - {"job"}
        running_attempt = (
            isinstance(history, list) and bool(history)
            and isinstance(history[-1], dict)
            and history[-1].get("status")
               == "SUBMISSION_INTENT_NOT_CONFIRMED"
        )
        if running_attempt:
            attempt = history[-1]
            if (not repair_intent
                    or expected_job is None
                    or len(history)
                       != PREDECESSOR["campaign_ledger_records"] + 1
                    or set(attempt) != uncertain_keys
                    or intent != attempt):
                raise ValueError(
                    "running BEFORE does not match the exact nineteenth intent")
            record = dict(attempt, status="SUBMITTED", job=expected_job)
            repaired_history = history[:-1] + [record]
            candidate = validate_campaign_history(
                repaired_history, root, require_own=True)
            record = validate_submission_record(candidate, pin, root=root)
            expected_receipt = submission_receipt(record, pin, root=root)
            receipt_exists = os.path.lexists(receipt_path)
            if (receipt_exists
                    and read_json_file(receipt_path) != expected_receipt):
                raise ValueError("Weyl core capture submission receipt changed")
            # Each replacement is atomic.  Ledger first makes every crash
            # window fail closed; a retry can then finish intent and receipt.
            save(ledger_path, repaired_history)
            save(intent_path, record)
            if not receipt_exists:
                save(receipt_path, expected_receipt)
            return record

        candidate = validate_campaign_history(
            history, root, require_own=True)
        record = validate_submission_record(candidate, pin, root=root)
        if expected_job is not None and record.get("job") != expected_job:
            raise ValueError("compute allocation differs from durable record")
        intent_needs_repair = intent != record
        if intent_needs_repair:
            same_attempt = (
                isinstance(intent, dict)
                and set(intent) == uncertain_keys
                and intent.get("status") == "SUBMISSION_INTENT_NOT_CONFIRMED"
                and all(intent.get(key) == record.get(key)
                        for key in uncertain_keys - {"status"})
            )
            if not repair_intent or not same_attempt:
                raise ValueError("capture intent and campaign ledger disagree")
        expected_receipt = None
        receipt_exists = False
        if repair_intent and expected_job is not None:
            expected_receipt = submission_receipt(record, pin, root=root)
            receipt_exists = os.path.lexists(receipt_path)
            if (receipt_exists
                    and read_json_file(receipt_path) != expected_receipt):
                raise ValueError("Weyl core capture submission receipt changed")
        if repair_intent:
            # The submitter may have crashed after replacing the confirmed
            # ledger but before syncing its parent directory.  Visibility is
            # not durability proof, so replay the same validated history
            # before publishing a confirmed intent or either login/compute
            # receipt.  The login recovery has no independent expected_job;
            # its already validated confirmed record supplies that identity.
            save(ledger_path, history)
        if intent_needs_repair:
            save(intent_path, record)
        if expected_receipt is not None and not receipt_exists:
            save(receipt_path, expected_receipt)
        return record


def submit_pinned(root, pin_sha256, stage_creation_sha256):
    if SUBMISSION_ENABLED is not True:
        raise ValueError("Weyl core capture submission remains disabled")
    return submit_one(
        root,
        SBATCH,
        dict(os.environ,
             WEYL_CONTEXT_CORE_CAPTURE_PIN_SHA256=pin_sha256),
        pin_sha256=pin_sha256,
        stage_creation_sha256=stage_creation_sha256,
    )


def _read_override_manifest(root, wanted):
    override_root = Path(root) / "overrides"
    manifest = load_relative_json(
        override_root, "overrides.json", wanted, mode=0o444, nlink=1)
    manifest, _ = validate_configuration(
        manifest, wanted, EXPECTED_TEST_COUNTS)
    expected_tree = dict(manifest)
    expected_tree["overrides.json"] = wanted
    if file_manifest(override_root) != expected_tree:
        raise ValueError("override tree contains changed or extra files")
    return copy.deepcopy(manifest)


def run_enabled(payload_root, overrides_sha256):
    """Publish the frozen payload, then submit its fixed campaign child once."""
    test_counts = require_enabled_launcher()
    if (not isinstance(overrides_sha256, str)
            or re.fullmatch(SHA256_PATTERN, overrides_sha256) is None):
        raise ValueError("invalid override-manifest trust root")
    payload_root = Path(payload_root).resolve()
    campaign = Path(PREDECESSOR_STAGE).parent.parent
    manifest = _read_override_manifest(payload_root, overrides_sha256)
    source_manifest = validate_predecessor(payload_root / "overrides", manifest)
    validate_prior_creation_failure(payload_root / "overrides", manifest)
    validate_capture_v1_failure(payload_root / "overrides", manifest)
    validate_capture_v2_failure(payload_root / "overrides", manifest)
    validate_capture_v3_failure(payload_root / "overrides", manifest)
    validate_capture_v4_failure(payload_root / "overrides", manifest)
    validate_capture_v5_failure(payload_root / "overrides", manifest)
    validate_capture_v6_failure(payload_root / "overrides", manifest)
    validate_capture_v7_failure(payload_root / "overrides", manifest)
    validate_capture_v8(payload_root / "overrides", manifest)
    validate_before_v1_failure(payload_root / "overrides", manifest)
    validate_before_v2_failure(payload_root / "overrides", manifest)
    validate_before_v3_failure(payload_root / "overrides", manifest)
    validate_before_v4_result(payload_root / "overrides", manifest)
    validate_after_v1_failure(payload_root / "overrides", manifest)
    validate_after_v2_failure(payload_root / "overrides", manifest)
    validate_after_v3_failure(payload_root / "overrides", manifest)
    validate_after_v4_failure(payload_root / "overrides", manifest)
    regression_manifest = validate_regression_inputs(
        payload_root / "overrides", manifest, source_manifest)
    validate_catalog(payload_root / "overrides", manifest)
    validate_launcher_transition(payload_root / "overrides", manifest)
    validate_parent_objects(campaign)
    creation_contract = build_stage_creation_contract(
        campaign, manifest, overrides_sha256)
    root, creation_sha256 = create_fixed_stage(
        campaign, payload_root, creation_contract)
    if root != Path(PREDECESSOR_STAGE).parent / STAGE_NAME:
        raise ValueError("stage creator returned another campaign child")
    creation_receipt = validate_stage_creation(
        root, creation_sha256, creation_contract)
    creation_reference = {
        "receipt_sha256": creation_sha256,
        "contract_sha256": creation_receipt["contract_sha256"],
    }
    validate_stage_topology(root)

    with existing_lock(campaign, ".atlas-progressive-submit.lock"):
        history = read_json_file(campaign / ".atlas-progressive-submit.json")
        own_record = validate_campaign_history(history, root)
    if own_record is not None and not (
            os.path.lexists(root / "submission-intent.json")
            or os.path.lexists(root / "submission.json")):
        raise ValueError("campaign ledger record lacks durable stage state")

    with exclusive_lock(root, STAGE_LOCK):
        install_inputs(root, manifest)
        validate_stage_topology(root)
        if frozen_stage_inputs(root) != manifest:
            raise ValueError("installed Weyl capture inputs changed")
        installed_source_manifest = validate_predecessor(root, manifest)
        validate_prior_creation_failure(root, manifest)
        validate_capture_v1_failure(root, manifest)
        validate_capture_v2_failure(root, manifest)
        validate_capture_v3_failure(root, manifest)
        validate_capture_v4_failure(root, manifest)
        validate_capture_v5_failure(root, manifest)
        validate_capture_v6_failure(root, manifest)
        validate_capture_v7_failure(root, manifest)
        validate_capture_v8(root, manifest)
        validate_before_v1_failure(root, manifest)
        validate_before_v2_failure(root, manifest)
        validate_before_v3_failure(root, manifest)
        validate_before_v4_result(root, manifest)
        validate_after_v1_failure(root, manifest)
        validate_after_v2_failure(root, manifest)
        validate_after_v3_failure(root, manifest)
        validate_after_v4_failure(root, manifest)
        if validate_regression_inputs(root, manifest, installed_source_manifest) \
                != regression_manifest:
            raise ValueError("installed regression source manifest changed")
        validate_catalog(root, manifest)
        validate_launcher_transition(root, manifest)
        pin = build_pin(
            manifest, overrides_sha256, test_counts, creation_reference)
        pin_path = root / PIN_NAME
        pin_sha256 = saved_json_sha(pin)
        if os.path.lexists(pin_path):
            if read_json_file(pin_path, pin_sha256) != pin:
                raise ValueError("prepared Weyl capture pin changed")
        else:
            save(pin_path, pin)
        validate_stage_creation(
            root, creation_sha256, creation_contract,
            pin_sha256=pin_sha256)
        receipt_path = root / "submission.json"
        intent_path = root / "submission-intent.json"
        if os.path.lexists(receipt_path):
            record = confirmed_record(root, pin)
            receipt = read_json_file(receipt_path)
            if receipt != submission_receipt(record, pin, root=root):
                raise ValueError("Weyl core capture receipt changed")
        elif os.path.lexists(intent_path):
            record = confirmed_record(root, pin, repair_intent=True)
            receipt = submission_receipt(record, pin, root=root)
            save(receipt_path, receipt)
        else:
            record = submit_pinned(root, pin_sha256, creation_sha256)
            receipt = submission_receipt(record, pin, root=root)
            save(receipt_path, receipt)
        # A compliant launcher cannot interpose another stage because this is
        # the sole enabled entry point.  Recheck the exact direct-successor
        # ledger before reporting a durable receipt.
        confirmed = confirmed_record(root, pin)
        if confirmed != record:
            raise ValueError("confirmed capture record changed after submission")
        validate_stage_topology(root)
        print(json.dumps(receipt, sort_keys=True), flush=True)
        return copy.deepcopy(receipt)


def main():
    if not SUBMISSION_ENABLED:
        raise SystemExit("Weyl core capture launcher is not frozen or enabled")
    if len(sys.argv) != 3:
        raise SystemExit("usage: stage PAYLOAD_ROOT OVERRIDES_SHA256")
    run_enabled(Path(sys.argv[1]), sys.argv[2])


if __name__ == "__main__":
    main()

#!/usr/bin/env bash
set -euo pipefail
scripts/run_controlled_release.sh
scripts/run_live_pipeline.sh
# Native source-build proof is separate and cannot be faked by the REST/API lane.
if [[ "${IOLAUS_REQUIRE_NATIVE_HYDRA:-1}" == "1" ]]; then
  build_root="${IOLAUS_HYDRA_BUILD_ROOT:-${HYDRADB_BUILD_ROOT:-/root/iolaus-hydradb-build}}"
  [[ -f "$build_root/evidence/server-binary.sha256" ]] || { echo "missing native HydraDB source-build proof" >&2; exit 8; }
  [[ -f evidence/native-import/PASS.json ]] || { echo "missing native HydraDB import/count proof" >&2; exit 9; }
fi
echo "IOLAUS_ALL_GATES_PASS"

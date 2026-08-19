#!/usr/bin/env bash
set -euo pipefail
build_root="${IOLAUS_HYDRA_BUILD_ROOT:-${HYDRADB_BUILD_ROOT:-/root/iolaus-hydradb-build}}"
bin="${HYDRADB_BIN_PATH:-}"
[[ -n "$bin" ]] || bin="$(cat "$build_root/HYDRADB_BIN_PATH" 2>/dev/null || true)"
[[ -x "$bin" ]] || { echo "HydraDB binary missing; run build_hydradb_source.sh" >&2; exit 2; }
mkdir -p "$build_root/logs" "$build_root/run"

# Source revisions can change CLI flags. We do not invent them. Supply an exact
# JSON argv array derived from this revision's --help/source.
if [[ -z "${HYDRADB_RUN_ARGV_JSON:-}" ]]; then
  echo "FAIL-CLOSED: HYDRADB_RUN_ARGV_JSON is unset." >&2
  echo "Review $build_root/evidence/server-help.txt and the checked-out source, then set exact argv JSON." >&2
  echo "Example shape only: export HYDRADB_RUN_ARGV_JSON='[\"serve\",\"--config\",\"/path/config\"]'" >&2
  exit 3
fi

args_file="$build_root/run/server-argv.json"
python3 - "$HYDRADB_RUN_ARGV_JSON" "$args_file" <<'PY'
import json,sys
raw=sys.argv[1]; out=sys.argv[2]
xs=json.loads(raw)
if not isinstance(xs,list) or not all(isinstance(x,str) for x in xs):
    raise SystemExit('HYDRADB_RUN_ARGV_JSON must be a JSON string array')
open(out,'w').write(json.dumps(xs,indent=2)+'\n')
PY
mapfile -d '' -t args < <(python3 - "$args_file" <<'PY'
import json,sys
for x in json.load(open(sys.argv[1])):
    sys.stdout.buffer.write(x.encode()+b'\0')
PY
)

nohup "$bin" "${args[@]}" > "$build_root/logs/hydradb.stdout.log" 2> "$build_root/logs/hydradb.stderr.log" &
pid=$!
echo "$pid" > "$build_root/run/hydradb.pid"
sleep 1
kill -0 "$pid" 2>/dev/null || { echo "HydraDB exited during startup" >&2; tail -100 "$build_root/logs/hydradb.stderr.log" >&2; exit 4; }
echo "HYDRADB_PROCESS_STARTED pid=$pid"
echo "This is process-liveness only; run 'iolaus-bench hydra-smoke' before claiming API readiness."

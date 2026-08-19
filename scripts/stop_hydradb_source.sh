#!/usr/bin/env bash
set -euo pipefail
build_root="${IOLAUS_HYDRA_BUILD_ROOT:-${HYDRADB_BUILD_ROOT:-/root/iolaus-hydradb-build}}"
pidfile="$build_root/run/hydradb.pid"
[[ -f "$pidfile" ]] || { echo "no pidfile"; exit 0; }
pid=$(cat "$pidfile")
kill "$pid" 2>/dev/null || true
for _ in {1..20}; do kill -0 "$pid" 2>/dev/null || { rm -f "$pidfile"; echo "stopped"; exit 0; }; sleep .25; done
kill -9 "$pid" 2>/dev/null || true
rm -f "$pidfile"

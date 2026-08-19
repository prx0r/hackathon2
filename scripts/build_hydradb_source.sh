#!/usr/bin/env bash
set -euo pipefail

src="${1:-${HYDRADB_SOURCE:-${HYDRADB_SRC:-/root/hydradb-build}}}"
build_root="${IOLAUS_HYDRA_BUILD_ROOT:-${HYDRADB_BUILD_ROOT:-/root/iolaus-hydradb-build}}"
min_free_gb="${HYDRADB_MIN_FREE_GB:-8}"
mkdir -p "$build_root" "$build_root/tmp" "$build_root/target" "$build_root/evidence"
export TMPDIR="$build_root/tmp"
export CARGO_TARGET_DIR="$build_root/target"

if [[ ! -d "$src" ]]; then echo "HydraDB source not found: $src" >&2; exit 2; fi
free_kb=$(df -Pk "$build_root" | awk 'NR==2{print $4}')
need_kb=$((min_free_gb*1024*1024))
if (( free_kb < need_kb )); then
  echo "FAIL: only $((free_kb/1024/1024))GiB free at $build_root; need >= ${min_free_gb}GiB" >&2
  exit 3
fi

if [[ "${IOLAUS_INSTALL_BUILD_DEPS:-0}" == "1" ]]; then
  if [[ "$(id -u)" == "0" ]]; then SUDO=(); else SUDO=(sudo); fi
  "${SUDO[@]}" apt-get update
  "${SUDO[@]}" apt-get install -y libcypher-parser-dev libgraphblas-dev cmake pkg-config clang build-essential
fi

for cmd in cargo rustc cmake pkg-config git; do command -v "$cmd" >/dev/null || { echo "missing: $cmd" >&2; exit 4; }; done
if ! pkg-config --exists libcypher-parser 2>/dev/null && ! pkg-config --exists cypher-parser 2>/dev/null && ! ldconfig -p 2>/dev/null | grep -qi cypher; then
  echo "Cypher parser library not detected (pkg-config libcypher-parser/cypher-parser or ldconfig)" >&2
  exit 5
fi
# SuiteSparse GraphBLAS package names vary. Require either pkg-config or ldconfig evidence.
if ! pkg-config --exists GraphBLAS 2>/dev/null && ! pkg-config --exists graphblas 2>/dev/null && ! ldconfig -p 2>/dev/null | grep -qi graphblas; then
  echo "GraphBLAS library not detected (pkg-config GraphBLAS/graphblas or ldconfig)" >&2
  exit 6
fi

python3 "$(dirname "$0")/discover_hydradb_source.py" "$src" --out "$build_root/evidence/source-discovery.json"
# If exactly one package declares server-runtime, use it automatically. An explicit
# HYDRADB_PACKAGE always wins. This avoids applying a package feature ambiguously at
# a virtual workspace root.
if [[ -z "${HYDRADB_PACKAGE:-}" ]]; then
  auto_pkg="$(python3 - "$build_root/evidence/source-discovery.json" <<'PY2'
import json,sys
r=json.load(open(sys.argv[1]))
xs=sorted({x.get('package') for x in r.get('server_runtime_feature',[]) if x.get('package')})
print(xs[0] if len(xs)==1 else '')
PY2
)"
  [[ -z "$auto_pkg" ]] || export HYDRADB_PACKAGE="$auto_pkg"
fi
{
  echo "timestamp=$(date -u +%FT%TZ)"
  echo "src=$src"
  echo "git_head=$(git -C "$src" rev-parse HEAD)"
  echo "rustc=$(rustc --version)"
  echo "cargo=$(cargo --version)"
  echo "tmpdir=$TMPDIR"
  echo "cargo_target_dir=$CARGO_TARGET_DIR"
  echo "free_kb_before=$free_kb"
} > "$build_root/evidence/build-env.txt"

cd "$src"
args=(build --release --features server-runtime)
if [[ -n "${HYDRADB_PACKAGE:-}" ]]; then args+=( -p "$HYDRADB_PACKAGE" ); fi
# Never use /tmp for target or temp files.
cargo "${args[@]}" 2>&1 | tee "$build_root/evidence/cargo-build.log"

bin="${HYDRADB_BIN_PATH:-}"
if [[ -z "$bin" ]]; then
  for candidate in "$CARGO_TARGET_DIR/release/graph-node" "$CARGO_TARGET_DIR/release/hydradb" "$CARGO_TARGET_DIR/release/hydra"; do
    [[ -x "$candidate" ]] && { bin="$candidate"; break; }
  done
fi
if [[ -z "$bin" ]]; then
  # Evidence-first fallback: executable files in release root only.
  mapfile -t xs < <(find "$CARGO_TARGET_DIR/release" -maxdepth 1 -type f -perm -111 -printf '%p\n' | sort)
  printf '%s\n' "${xs[@]}" > "$build_root/evidence/release-executables.txt"
  echo "FAIL: could not uniquely identify HydraDB server binary. Set HYDRADB_BIN_PATH after reviewing release-executables.txt" >&2
  exit 7
fi

"$bin" --help > "$build_root/evidence/server-help.txt" 2>&1 || true
sha256sum "$bin" > "$build_root/evidence/server-binary.sha256"
printf '%s\n' "$bin" > "$build_root/HYDRADB_BIN_PATH"

echo "HYDRADB_SOURCE_BUILD_PASS"
echo "binary=$bin"
echo "evidence=$build_root/evidence"

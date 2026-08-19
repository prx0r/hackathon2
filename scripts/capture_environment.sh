#!/usr/bin/env bash
set -euo pipefail
out="${1:-evidence/environment.txt}"
mkdir -p "$(dirname "$out")"
{
  echo "captured_at=$(date -u +%FT%TZ)"
  echo "hostname=$(hostname)"
  echo "uname=$(uname -a)"
  echo "cwd=$(pwd)"
  echo "git_head=$(git rev-parse HEAD 2>/dev/null || echo NO_GIT)"
  echo "cargo=$(cargo --version 2>/dev/null || echo MISSING)"
  echo "rustc=$(rustc --version 2>/dev/null || echo MISSING)"
  echo "docker=$(docker --version 2>/dev/null || echo MISSING)"
  hermes_bin="${HERMES_BIN:-hermes}"
  echo "HERMES_BIN=$hermes_bin"
  echo "hermes=$($hermes_bin --version 2>/dev/null || echo MISSING)"
  echo "HYDRA_DB_API_URL=${HYDRA_DB_API_URL:-${HYDRADB_BASE_URL:-UNSET}}"
  echo "IOLAUS_HYDRA_TENANT=${IOLAUS_HYDRA_TENANT:-${IOLAUS_HYDRA_DATABASE:-UNSET}}"
  echo "HYDRA_DB_API_KEY_SET=$([[ -n ${HYDRA_DB_API_KEY:-${HYDRADB_API_KEY:-}} ]] && echo yes || echo no)"
  echo "---- df ----"; df -h /root /tmp 2>/dev/null || true
  echo "---- memory ----"; free -h 2>/dev/null || true
} > "$out"
cat "$out"

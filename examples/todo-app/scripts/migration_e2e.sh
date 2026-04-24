#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLE_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd -- "${EXAMPLE_DIR}/../.." && pwd)"
WORK_ROOT="${1:-$(mktemp -d "${TMPDIR:-/tmp}/mssql-orm-todo-migrations.XXXXXX")}"
CLI_BIN="${REPO_ROOT}/target/debug/mssql-orm-cli"
MANIFEST_PATH="${EXAMPLE_DIR}/Cargo.toml"

cargo build --manifest-path "${REPO_ROOT}/crates/mssql-orm-cli/Cargo.toml" >/dev/null

mkdir -p "${WORK_ROOT}"

(
    cd "${WORK_ROOT}"
    "${CLI_BIN}" migration add CreateTodoSchema \
        --snapshot-bin model_snapshot \
        --manifest-path "${MANIFEST_PATH}"
    "${CLI_BIN}" migration add VerifyTodoSchemaNoop \
        --snapshot-bin model_snapshot \
        --manifest-path "${MANIFEST_PATH}"
    "${CLI_BIN}" database update > database_update.sql
)

printf 'Migration workspace: %s\n' "${WORK_ROOT}"
printf 'Generated script: %s\n' "${WORK_ROOT}/database_update.sql"

if [[ -n "${MSSQL_ORM_SQLCMD_SERVER:-}" && -n "${MSSQL_ORM_SQLCMD_USER:-}" && -n "${MSSQL_ORM_SQLCMD_PASSWORD:-}" ]]; then
    sqlcmd -S "${MSSQL_ORM_SQLCMD_SERVER}" \
        -U "${MSSQL_ORM_SQLCMD_USER}" \
        -P "${MSSQL_ORM_SQLCMD_PASSWORD}" \
        -d "${MSSQL_ORM_SQLCMD_DATABASE:-tempdb}" \
        -C -b -i "${WORK_ROOT}/database_update.sql"
else
    printf 'MSSQL_ORM_SQLCMD_SERVER/USER/PASSWORD are not set; SQL Server apply step was skipped.\n'
fi

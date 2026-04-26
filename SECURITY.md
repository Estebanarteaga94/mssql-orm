# Security Policy

`mssql-orm` is an early-stage Rust ORM for SQL Server. Treat it as project
software under active development, not as a security-certified data access
layer.

## Supported Versions

There is no published stable release line yet. Security fixes are handled on
the main development branch until versioned releases exist.

## Reporting a Vulnerability

Do not open a public issue for a vulnerability that could expose user data,
credentials, SQL injection paths, migration data loss, or tenant isolation
problems.

Until a private security contact is published, report privately to the current
maintainer through the repository owner's preferred private channel. If no
private channel is available, open a minimal public issue that says only that a
private security report is needed. Do not include exploit details, credentials,
connection strings, database dumps, logs with parameters, or production schema
details in that issue.

Please include, when safe:

- affected crate or module;
- affected public API;
- reproduction steps using test data only;
- expected impact;
- whether the issue requires SQL Server access;
- whether the issue affects generated SQL, raw SQL, migrations, tenant filters,
  soft delete, transactions, pooling, or row mapping.

## Security Boundaries

The project intentionally separates responsibilities:

- `mssql-orm-core` must not depend on Tiberius.
- `mssql-orm-query` builds AST and must not generate SQL strings.
- `mssql-orm-sqlserver` owns SQL Server SQL generation and identifier quoting.
- `mssql-orm-tiberius` owns SQL Server execution through Tiberius.
- `mssql-orm` exposes the public user API.

Security-sensitive changes should preserve those boundaries.

## Data Handling Rules

- Do not commit real connection strings, passwords, tokens, database dumps, or
  production logs.
- Prefer environment variables such as `DATABASE_URL` or
  `MSSQL_ORM_TEST_CONNECTION_STRING` for local integration tests.
- Generated SQL should use parameters for values.
- Raw SQL APIs require the caller to write safe SQL and explicit predicates.
- Raw SQL does not automatically apply ORM-level `tenant` or `soft_delete`
  filters.
- Logs and diagnostics should avoid exposing parameter values by default.

## Security-Sensitive Areas

Review changes carefully when they affect:

- SQL parameter binding and placeholder validation;
- identifier quoting;
- raw SQL APIs;
- `tenant` filtering and fail-closed behavior;
- `soft_delete` behavior;
- migration destructive-change detection;
- transaction boundaries;
- connection pooling;
- row mapping and type conversion;
- tracing or error messages that may include data.

## Guidance for AI Agents

AI agents must not invent security guarantees. If code behavior is unclear,
document it as `Pending verification` and add a task instead of presenting it as
implemented. Agents must not print or commit secrets from local environment
variables or user-provided connection strings.

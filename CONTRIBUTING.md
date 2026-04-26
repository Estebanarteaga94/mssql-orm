# Contributing

Thanks for helping improve `mssql-orm`. This repository is built as a
SQL Server-first Rust ORM with strict crate boundaries and documentation-driven
workflow.

## Project Goals

`mssql-orm` aims to provide:

- code-first entities in Rust;
- a public `DbContext` / `DbSet` API;
- a typed query builder that builds AST, not SQL strings;
- SQL Server compilation in a dedicated crate;
- execution through Tiberius in a dedicated adapter crate;
- code-first migrations;
- clear documentation for users and AI-assisted maintainers.

Do not broaden the project into a multi-database ORM unless the roadmap is
explicitly changed.

## Architecture Rules

Follow these boundaries:

- `mssql-orm-core`: contracts, metadata, values, errors and base traits.
- `mssql-orm-macros`: derives and compile-time metadata generation.
- `mssql-orm-query`: query AST only; no SQL generation.
- `mssql-orm-sqlserver`: SQL Server SQL and DDL compilation.
- `mssql-orm-tiberius`: connection, execution, rows, transactions and Tiberius.
- `mssql-orm-migrate`: snapshots, diff, migration operations and migration IO.
- `mssql-orm-cli`: command-line entry points.
- `mssql-orm`: public API and reexports.

Prefer existing local patterns over new abstractions. Keep changes scoped to the
task being implemented.

## Workflow

Before changing code or docs:

1. Read `docs/instructions.md`.
2. Read `docs/tasks.md`.
3. Read `docs/context.md`.
4. Check recent entries in `docs/worklog.md`.
5. Verify whether the requested behavior already exists in code.

For task-driven work:

1. Move one task to `En Progreso` in `docs/tasks.md`.
2. Implement only that task and the minimum support required.
3. Validate with focused tests and broader checks when appropriate.
4. Update `docs/tasks.md`, `docs/worklog.md` and `docs/context.md`.
5. Commit completed and validated work.

## Documentation Contributions

Documentation should help both normal users and AI agents.

Use English for public documentation unless a task explicitly says otherwise.
Use kebab-case filenames under `docs/`.

Documentation must distinguish:

- implemented and validated behavior;
- experimental behavior;
- planned behavior;
- `Pending verification` when code or tests do not make the claim clear.

Keep the root `README.md` short and navigational. Put detailed explanations in
`docs/`.

## Testing

Run the narrowest useful validation first, then broader checks when the change
can affect shared behavior.

Common commands:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

SQL Server integration tests may require:

```bash
MSSQL_ORM_TEST_CONNECTION_STRING="Server=localhost;Database=tempdb;User Id=SA;Password=...;TrustServerCertificate=True;Encrypt=False;"
```

Do not commit real credentials.

## Pull Request Checklist

- The change follows crate boundaries.
- Public APIs are documented when they change.
- Tests or documentation validation match the risk of the change.
- `docs/tasks.md` and `docs/worklog.md` are updated for task-driven work.
- Experimental behavior is labeled as experimental.
- Planned behavior is not described as available.
- Secrets and local connection strings are not committed.

## Guidance for AI Agents

AI agents should operate as maintainers, not as abstract advisors:

- read the repo before editing;
- do not invent architecture;
- do not document unverified behavior as real;
- preserve user changes already present in the working tree;
- use `rg` for search;
- prefer focused patches;
- leave traceability in `docs/`;
- keep final summaries concrete: task, changes, validation and next step.

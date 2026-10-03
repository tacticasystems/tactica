# Agentic Coding Guidance for Tactica

Guidance for agents (and humans) working in the Tactica repository.

## What this is

Tactica is a military simulation (milsim) unit management platform built to help
communities easier to manage.

## Workspace Layout

```
web/                The Vite + React Web UI for Tactica

crates/             The Rust backend for Tactica

    api/            The REST API server for Tactica

    auth/           The pluggable authentication layer

    db/             The database infrastructure model
    db/model/       The database domain models
    db/schema/      The database schema for use by Diesel

    permissions/    The permissions model for Tactica, represented by bitflags.

    uuid-kinds/     Central registry of UUID newtypes for use across the app.
```

## Build/test/verify

Tasks live in `.mise/config.toml`; `mise tasks` lists them, `mise run <task>`
runs one. Mise is mandatory and pins the Rust toolchain plus required
components.

```
mise run build
mise run test                # hermetic tests: unit/serde/validation, no llm provider, no network
mise run clippy
mise run complexity-check    # cognitive complexity ratchet: offender count must not grow
mise run complexity          # advisory: list every function above the threshold
mise run fmt-check           # check formatting with rustfmt
```

Integration/E2E tests must **never** target the user's own resources.

## Working style here

- This is a phased build. Land one milestone, verify it (cargo test + clippy
  green), then start the next. Don't claim a milestone done without showing the
  passing test output.
- When a milestone is large and well-specified, it's fine to delegate to a
  subagent — but always independently re-run cargo test/clippy before trusting
  the result.
- Commit/push only when the user asks. Work happens on a feature or fix branch.
- Commits should follow [Conventional Commit](https://conventionalcommits.org)

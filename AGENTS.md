# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## Project

`iced_ui` is a component library built on top of [iced-rs]. The library
crate is published to crates.io; a companion demo app showcases every
component in a single kitchen-sink binary.

[iced-rs]: https://github.com/iced-rs/iced

## Workspace Layout

This is a Cargo workspace (edition 2024, resolver 3) with two members:

- `crates/iced_ui/` — the published library crate (`iced_ui`). This is
  the only crate intended for publication to crates.io.
- `crates/demo/` — an internal kitchen-sink application that exercises
  every component in `iced_ui`. Never published.

Workspace `version` and `edition` are inherited via `workspace.package`.

## Hard Rules

These constraints are load-bearing. Do not violate them without an
explicit request from a maintainer.

1. **No derivative component dependencies.** `iced_ui` must not depend
   on derivative iced component libraries such as `iced_aw`. Components
   are to be implemented directly against `iced` primitives.
2. **Keep `iced_ui`'s dependency surface minimal.** Prefer `iced` and
   the standard library. Any new dependency added to `crates/iced_ui/`
   should be justified.
3. **Only `iced_ui` is published.** `demo` is a development aid and
   must stay internal (no `publish = true`, no crates.io metadata).
4. **The demo must stay a kitchen sink.** Every component exposed by
   `iced_ui` should have a corresponding showcase in `crates/demo/`.
5. **Demo showcases defaults.** Component demonstrations in
   `crates/demo/` must show each widget using its default
   configuration. Do not override default values (padding, spacing,
   roundness, etc.) — the demo should reflect the out-of-the-box
   appearance driven by the theme.

## Adding a Component

1. Implement it under `crates/iced_ui/src/`.
2. Re-export it from the library's `lib.rs` so downstream users can
   reach it via the public API.
3. Add a demonstration screen/section to `crates/demo/` that exercises
   the component's public API.
4. Run `just lint && just test` before considering the change done.

## Commands

Use the workspace [`justfile`](./justfile). Common recipes:

- `just` — list all recipes
- `just dev` — run the demo gallery (`cargo run -p demo`)
- `just build` — build the whole workspace
- `just test` — run all workspace tests
- `just lint` — check formatting and run `clippy` with `-D warnings`
- `just fix` — auto-format and apply `clippy --fix`
- `just publish-dry` — dry-run a publish of `iced_ui`

Before opening a PR, run `just lint && just test`.
If `just` is not installed, the recipes map directly to `cargo`
invocations; consult the justfile for the exact commands.

## Conventions

- Member crates inherit `license`, `repository`, and `edition` from the workspace. Each crate declares its own `version`; release-please manages per-crate version bumps based on conventional commits.
- Public API items in `iced_ui` should be documented; run
  `cargo doc -p iced_ui` locally when adding public surface.

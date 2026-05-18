# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## Project

`iced_ui` is a component library built on top of [iced-rs]. The library
crate is published to crates.io; a companion demo app showcases every
component in a single kitchen-sink binary.

[iced-rs]: https://github.com/iced-rs/iced

## Workspace Layout

This is a Cargo workspace (edition 2024, resolver 3) with three members:

- `crates/iced_ui/` — the published library crate (`iced_ui`). This is
  the only crate intended for publication to crates.io.
- `crates/demo/` — an internal kitchen-sink application that exercises
  every component in `iced_ui`. Never published.
- `crates/iced_ui_tests/` — an internal integration-test crate with
  snapshot tests that lock down the visual output of every widget.
  Never published.

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
4. Add a snapshot test in `crates/iced_ui_tests/tests/<widget>.rs`
   covering the widget's default configuration plus any meaningful
   variants. See [Testing](#testing) below.
5. Run `just lint && just test` before considering the change done.

## Testing

Widget quality is enforced by the `iced_ui_tests` crate, which uses
[`iced_test`] — iced's first-party headless testing framework — to
render each widget into an offscreen pixel buffer and compare the
result against a committed PNG reference image.

[`iced_test`]: https://docs.rs/iced_test

### Backend

`just test` (and `cargo test --workspace` invoked through the
justfile) sets `ICED_TEST_BACKEND=tiny-skia`. This pins snapshots to
the CPU `tiny_skia` software renderer for two reasons:

1. **Determinism** — pixel output is identical across hardware,
   driver versions, and CI runners.
2. **Portability** — no GPU is required. CI workers without graphics
   acceleration still produce identical bytes.

Reference images are stored under
`crates/iced_ui_tests/tests/snapshots/` with names of the form
`<test_name>-tiny-skia.png`. The `-tiny-skia` suffix is appended
automatically by `iced_test` based on the active renderer.

### Authoring snapshot tests

Each widget gets its own integration-test file under
`crates/iced_ui_tests/tests/<widget>.rs`. A test file looks like:

```rust
use iced::widget::{row, text};
use iced_test::Error;
use iced_ui::Fab;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot};

#[derive(Debug, Clone)]
enum Message {
    Pressed,
}

#[test]
fn fab_default() -> Result<(), Error> {
    let element = row![Fab::new(text("+")).on_press(Message::Pressed)].padding(20);
    assert_snapshot::<Message>("fab_default", element, DEFAULT_SIZE)
}
```

The `iced_ui_tests` library crate exposes:

- `assert_snapshot(name, element, size)` — render the supplied
  element with the default `iced_ui::Theme` and compare against the
  named snapshot, creating it on first run.
- `build(element, size)` — return a raw `Simulator` for tests that
  need to drive interactions (clicks, key taps) before snapshotting.
- `theme()` — the canonical `iced_ui::Theme` used in tests.
- `DEFAULT_SIZE` / `WIDE_SIZE` / `TALL_SIZE` — standard canvas sizes.

For interaction tests (clicks, keyboard input that emit messages),
use `Simulator::click(...)` and `Simulator::tap_key(...)`, then
inspect the produced messages via `into_messages()`. See
`tests/interactions.rs` for examples.

### Reviewing snapshot changes

When a widget's appearance is intentionally changed:

1. Delete the affected snapshot files in
   `crates/iced_ui_tests/tests/snapshots/`.
2. Run `just test` to regenerate them.
3. Visually inspect the new PNGs (e.g. via `git diff --stat` and an
   image viewer) and commit them alongside the code change.

A failing snapshot test means either the widget regressed visually
or the change is intentional and the reference needs to be updated.

## Commands

Use the workspace [`justfile`](./justfile). Common recipes:

- `just` — list all recipes
- `just dev` — run the demo gallery (`cargo run -p demo`)
- `just build` — build the whole workspace
- `just test` — run all workspace tests with
  `ICED_TEST_BACKEND=tiny-skia` for deterministic snapshots
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

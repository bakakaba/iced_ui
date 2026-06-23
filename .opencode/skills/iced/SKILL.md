---
name: iced
description: Use when writing, reviewing, or optimizing iced-rs GUI code in this workspace. Also use to review any changes that include iced-related modifications. Covers render optimization with lazy/keyed widgets, modular Elm Architecture state patterns, and performance anti-patterns that cause lag or memory leaks.
---

# Iced Performance & Architecture Skill

## Workspace Context

- **iced 0.14** with features: `advanced`, `image`, `svg`
- Reactive rendering: iced 0.14 only redraws when state changes
- Library crate (`crates/iced_ui/`) implements custom widgets against iced primitives
- Type alias used in this workspace: `type Element<'a, Message> = iced::Element<'a, Message, Theme>`
- Theme: `iced_ui::Theme` wraps `iced::Theme` plus spacing/roundness tokens
- Architecture: Elm Architecture — `update(&mut self, msg) -> Task<Message>`, `view(&self) -> Element`

Severity tags below indicate review priority:

- **CRITICAL** — load-bearing rules; violations cause correctness or major perf issues. Block on these.
- **HIGH** — important optimizations or patterns; flag and recommend a fix.
- **MEDIUM** — quality improvements; suggest when relevant.
- **LOW** — polish; mention if convenient.

---

## 1. Render Optimization

### CRITICAL

- **Never do expensive work in `view()`.** `view()` runs on every state
  change. Sorting, filtering, string formatting, and any O(n) or worse
  computation must happen in `update()` instead.

  ```rust
  // BAD: O(n log n) every frame
  fn view(&self) -> Element<'_, Message> {
      let sorted: Vec<_> = self.items.iter().sorted_by_key(|i| &i.name).collect();
      column(sorted.iter().map(|i| text(&i.name))).into()
  }

  // GOOD: pre-computed in update()
  fn update(&mut self, msg: Message) -> Task<Message> {
      match msg {
          Message::ItemAdded(item) => {
              self.items.push(item);
              self.items.sort_by(|a, b| a.name.cmp(&b.name));
          }
          // ...
      }
      Task::none()
  }
  ```

- **Keep widget tree structure stable.** Conditional insertion or removal
  shifts positional diffing and causes child widget state loss (focus,
  scroll position, text-input cursor).

  ```rust
  // BAD: inserting warning shifts text_input's position — loses focus
  let mut col = column![];
  if self.show_warning {
      col = col.push(text("Warning!"));
  }
  col = col.push(text_input("Name", &self.name));

  // GOOD: always occupy the slot; collapse with an empty placeholder
  use iced::widget::Space;
  let warning: Element<'_, Message> = if self.show_warning {
      text("Warning!").into()
  } else {
      Space::new(0, 0).into()
  };
  column![warning, text_input("Name", &self.name)].into()
  ```

### HIGH

- **Use `lazy()` for expensive subtrees with infrequent changes.** The
  closure only re-executes when the `Hash`-able dependency changes.

  ```rust
  use iced::widget::lazy;

  fn view(&self) -> Element<'_, Message> {
      lazy(self.items.len(), |_| {
          column(self.items.iter().map(|i| build_item_row(i)))
      }).into()
  }
  ```

- **Use `keyed::column()` for dynamic lists.** Without keys, inserting or
  removing items causes all siblings to lose state.

  ```rust
  use iced::widget::keyed;

  keyed::column(self.items.iter().map(|item| {
      (item.id, build_item_widget(item))
  }))
  ```

- **Use `canvas::Cache` for custom drawing.** Only call `.clear()` from
  `update()` when the underlying data changes.

  ```rust
  // In update():
  Message::DataChanged(d) => {
      self.data = d;
      self.cache.clear(); // triggers redraw next frame
  }

  // In Program::draw():
  self.cache.draw(renderer, bounds.size(), |frame| {
      // expensive geometry only re-runs when cache is cleared
  })
  ```

### MEDIUM

- Prefer `&str` references and `Cow<'static, str>` over `String`
  allocations in `view()` for static or rarely-changing text.
- Use `responsive(|size| ...)` for layout-dependent branching rather than
  building all variants unconditionally.
- Wrap stable side-panels (sidebars, top bars, footers) in `lazy()` keyed
  on the small subset of state they actually depend on.

---

## 2. State Structure

### CRITICAL

- **Do NOT use the deprecated `Component` trait.** It was deprecated in
  iced 0.13 because it encapsulates state and breaks the single-source-of-
  truth principle. Use nested Elm Architecture with Action enums, or
  implement a custom `Widget` directly.

### HIGH

- **Each logical module owns: a state struct, a `Message` enum, and an
  `update() -> Action` method.** The parent inspects `Action` to coordinate
  transitions and to map child tasks/subscriptions.

  ```rust
  // Module: contacts.rs
  pub struct Contacts { /* module-local state */ }

  pub enum Message {
      Selected(ContactId),
      SearchChanged(String),
  }

  pub enum Action {
      None,
      OpenChat(ContactId),
      Run(Task<Message>),
  }

  impl Contacts {
      pub fn update(&mut self, msg: Message) -> Action {
          match msg {
              Message::Selected(id) => Action::OpenChat(id),
              Message::SearchChanged(q) => {
                  self.query = q;
                  Action::None
              }
          }
      }

      pub fn view(&self) -> Element<'_, Message> { /* ... */ }
  }

  // Parent:
  fn update(&mut self, msg: Message) -> Task<Message> {
      match msg {
          Message::Contacts(m) => match self.contacts.update(m) {
              contacts::Action::OpenChat(id) => { /* transition */ }
              contacts::Action::Run(task) => return task.map(Message::Contacts),
              contacts::Action::None => {}
          },
      }
      Task::none()
  }

  fn view(&self) -> Element<'_, Message> {
      self.contacts.view().map(Message::Contacts)
  }
  ```

- **Pre-compute derived data in `update()`; store the result in state.**
  Formatted strings, filtered lists, computed layouts — all belong in
  `update()`, never in `view()`.

- **Compose via `Element::map()`, `Task::map()`, `Subscription::map()`.**
  These are the glue between parent and child message types.

### MEDIUM

- Keep `Message` variants lightweight — carry IDs or indices, not cloned
  data structures. The framework requires `Message: Clone` in many spots,
  and bloated messages multiply that cost.
- Group related fields into sub-structs to clarify ownership boundaries
  (e.g., `dialog_state: DialogState` rather than `dialog_open: bool` plus
  five sibling fields).
- Derive `Default` on state structs to enable clean initialization and
  composition with `iced::application(State::default, update, view)`.

### LOW

- Prefer enums over booleans for mutually-exclusive UI states (e.g.,
  `enum Loading { Idle, Working, Failed(String) }` over three booleans).

---

## 3. Performance Anti-patterns

| Severity | Anti-pattern                                                         | Fix                                                                |
| -------- | -------------------------------------------------------------------- | ------------------------------------------------------------------ |
| CRITICAL | Blocking `update()` with I/O or heavy CPU work                       | `Task::perform(async_fn(), Message::Done)`                         |
| CRITICAL | Subscription identity changes each frame (new closure captures)      | Function pointers or `Subscription::run_with(hash_key, fn)`        |
| HIGH     | Large dynamic lists without keys                                     | `keyed::column()` with stable IDs                                  |
| HIGH     | Canvas redraws every frame                                           | `canvas::Cache` plus selective `.clear()` in `update()`            |
| HIGH     | Sorting/filtering/formatting in `view()`                             | Move to `update()`, store the result                               |
| MEDIUM   | Unbounded buffers in subscription streams                            | Bounded channels; return `Subscription::none()` when inactive      |
| MEDIUM   | Allocating `Vec`/`String` per frame in `view()` loops                | Cache in state; use `Cow<'static, str>` for static text            |
| LOW      | Widget state loss from tree restructuring                            | Stable structure; placeholder `Space`; keyed widgets               |

### CRITICAL examples

```rust
// BAD: blocks the UI thread — every other widget freezes until I/O completes
fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::Save => {
            std::fs::write("data.json", self.serialize()).ok(); // BLOCKS
            Task::none()
        }
        _ => Task::none(),
    }
}

// GOOD: offload to an async task; UI stays responsive
fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::Save => {
            let data = self.serialize();
            Task::perform(
                async move { tokio::fs::write("data.json", data).await },
                |result| Message::SaveComplete(result.is_ok()),
            )
        }
        _ => Task::none(),
    }
}
```

```rust
// BAD: closure identity changes each frame — subscription restarts every time
fn subscription(&self) -> Subscription<Message> {
    let config = self.config.clone();
    Subscription::run(move || watch_files(config)) // identity unstable
}

// GOOD: stable identity via run_with; the runtime keeps the same stream alive
fn subscription(&self) -> Subscription<Message> {
    Subscription::run_with(self.config_hash, watch_files_stream)
}
```

---

## 4. Custom Widget Implementation (`crates/iced_ui/src/`)

Specific guidance for the components that live in this library crate:

- **CRITICAL: Theme values must reach the widgets that use them.** Where a
  widget consumes a theme token (spacing, roundness, text size, color
  group), changing that token on the `Theme` *must* visibly affect the
  widget. Do not silently fall back to `Theme::DEFAULT_*` as the final
  resolved value. The common trap is `Widget::layout()`, which has no
  `&Theme` parameter: resolving `Space::sx`/`Roundness::sx` there against a
  hardcoded default means the widget ignores a customized theme. Use the
  **`Cell<T>` cache pattern** — write the theme-resolved value in `draw()`
  (which receives `&Theme`), read it in `layout()`, and seed the cell with
  the matching `Theme::DEFAULT_*` constant so the first frame (before any
  draw) is reasonable. See `tabs/`, `tree/`, `list/`, and `snackbar/` for
  the canonical implementation; `progress/` and `spinner/` cache the
  spacing base this way.

  ```rust
  struct State {
      // ...
      spacing: Cell<u8>, // seeded with Theme::DEFAULT_SPACING
  }
  // draw(): state.spacing.set(theme.spacing());
  // layout(): let s = tree.state.downcast_ref::<State>().spacing.get();
  //           let px = MY_TOKEN.resolve(s);
  ```

  If a theme value genuinely cannot be threaded through to where it is
  needed, **document why in a code comment and explicitly notify the user**
  (call it out in your summary) rather than quietly defaulting.

- **Implement `Widget::diff()` correctly.** Call `tree.diff_children()` (or

  the appropriate variant) with the new children slice. Without this, child
  widget state is reset on every frame.
- **Do not cache view output inside widgets.** Caching is the *consumer's*
  responsibility via `lazy()`. Widgets should be cheap to reconstruct so
  the framework's diffing remains correct.
- **Use `tree::State` for internal mutable widget state** (hover tracking,
  open/closed flags, animation timers). Access via
  `tree.state.downcast_mut::<MyState>()`.
- **Return an accurate `Tag` from `Widget::tag()`** — this is how the diff
  engine identifies widget type. Mismatched tags cause full state reset.
- **Avoid allocations in `layout()` and `draw()`.** These run on every
  frame the widget is visible. Pre-compute sizes where possible; prefer
  stack arrays over `Vec` for small fixed-size child lists.
- **Test snapshots after layout/style changes.** `crates/iced_ui_tests/`
  uses `iced_test` with the `tiny-skia` backend for deterministic pixel
  comparisons. Regenerate snapshots intentionally (delete + `just test`)
  rather than tweaking thresholds.
- **Keep each widget self-contained in its module (HIGH).** A widget's
  struct, `Widget` impl, style/`Catalog`, and internal helpers belong in
  one top-level module under `src/<widget>/`. Use private submodules
  (`style`, `core`, `grid`, `overlay`) for internals, but do not nest
  public widget modules or split a widget across crate-root siblings.
  Conceptually related widgets may share one top-level module over a
  private engine (e.g. `datetime_input::core`), with every sibling widget
  at the same top level. See AGENTS.md Hard Rules #6–#7.

---

## 5. Overlay Implementation

Guidance for widgets that render floating content via `Widget::overlay()`.

### CRITICAL

- **Position overlays relative to host bounds, not the viewport.**
  `Widget::overlay()` receives `viewport` but that's the full window.
  Compute host bounds from `layout.bounds() + translation`:

  ```rust
  fn overlay<'b>(&'b mut self, tree, layout, renderer, viewport, translation) {
      let mut host_bounds = layout.bounds();
      host_bounds.x += translation.x;
      host_bounds.y += translation.y;
      // Pass host_bounds to your overlay struct
  }
  ```

  In the overlay's `layout()`, position the root node at the host's
  absolute position and size it to host dimensions (or smaller). Child
  nodes use positions relative to the root.

- **Size overlay root to only the interactive region.** If the overlay
  root covers the full host area, iced's runtime checks
  `mouse_interaction()` against those bounds and may block cursor
  pass-through to the widget tree underneath. Size the root to tightly
  wrap only the rendered overlay content (e.g., notification bars, menu
  panels, tooltip bubbles). This ensures clicks outside the overlay
  reach the host's interactive widgets.

- **Recompute hover state on `ButtonPressed`, not just `CursorMoved`.**
  After layout shifts (items added/removed between frames), cached
  hover state (which ID is "hovered") becomes stale — it references
  positions/IDs from the previous layout. Always recalculate what's
  under the cursor when a click starts:

  ```rust
  Event::Mouse(mouse::Event::CursorMoved { .. })
  | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
      // Recompute hover from cursor + current layout on BOTH events.
      self.update_hover(cursor, &layouts);

      if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
          self.pressed = self.hovered.clone();
      }
  }
  ```

  Without this, rapidly clicking after a dismiss (which shifts the
  layout) fires events for the wrong target or doesn't fire at all.

### HIGH

- **Theme values are unavailable in `Overlay::layout()`.** The
  `Overlay::layout(&mut self, renderer, bounds)` signature has no
  `theme` parameter. Use `Cell<T>` in the widget's tree state to cache
  theme-derived values. Update in `Overlay::draw()` (which receives
  the theme via `&self`), read in `Overlay::layout()` (via `&mut self`
  access to state). Initialize with `Theme::DEFAULT_*` constants for
  the first-frame fallback.

  ```rust
  // In SnackbarState:
  spacing: Cell<u8>,  // initialized to Theme::DEFAULT_SPACING

  // In Overlay::draw():
  self.state.spacing.set(theme.spacing());

  // In Overlay::layout():
  let margin = Space::sx(2.0).resolve(self.state.spacing.get());
  ```

- **`overlay::Group` does not short-circuit events.** All children in a
  group receive every event via `Group::update()`. Only
  `shell.capture_event()` prevents propagation to the base widget tree
  — group children cannot block each other.

- **Overlay `mouse_interaction()` controls cursor pass-through.** Return
  `Interaction::None` when the cursor is not over any interactive
  overlay element. If you return non-`None`, the iced runtime sets
  `base_cursor = Cursor::Unavailable` for the widget tree, and
  interactive widgets underneath (buttons, inputs) won't respond to
  that event.

### MEDIUM

- **Clean up stale state when the overlay's data changes.** If the
  overlay tracks per-item state (timers, hover flags keyed by ID),
  remove entries for items no longer present in the data. Otherwise
  stale entries accumulate and may cause ID mismatches.

- **Use `shell.request_redraw()` for animated overlays.** Auto-dismiss
  countdowns, progress indicators, or any time-varying content needs
  explicit redraw requests — iced won't repaint without state changes.

### Frame Lifecycle Reference

Within a single frame in `user_interface.rs`:

1. `Widget::overlay()` — collect overlay elements from widget tree
2. `Overlay::layout()` — compute overlay positioning
3. For each event in batch: `Overlay::update()` → check `event_status`
4. `Overlay::mouse_interaction()` — determines `base_cursor` for widget tree
5. For each non-captured event: `Widget::update()` (receives `base_cursor`)
6. `Overlay::draw()` + `Widget::draw()` — render

Messages are collected during steps 3 + 5 and processed **after** the
frame completes. The app's `update()` runs, then `view()` rebuilds the
widget tree for the next frame.

Key implications:
- Overlays see events **before** the widget tree.
- `shell.capture_event()` in overlay prevents the widget tree from
  receiving that specific event.
- `mouse_interaction() != None` makes cursor unavailable to widgets.
- `Overlay::draw()` runs on the same frame as `Overlay::layout()` —
  values written to state in `draw()` are available in `layout()` on
  the **next** frame only.

---

## Review Checklist

When reviewing a change that touches iced code, walk this list:

1. Does any new code in `view()` allocate, sort, filter, or format?
   → Move to `update()`. (CRITICAL)
2. Does the widget tree structure change conditionally between frames?
   → Use stable structure with `Space` placeholders. (CRITICAL)
3. Is there a dynamic list of items? Does it use `keyed::column()`? (HIGH)
4. Is there a `Canvas` without a `Cache`? (HIGH)
5. Does `update()` do synchronous I/O or heavy CPU work?
   → Wrap in `Task::perform()`. (CRITICAL)
6. Is `subscription()` returning a closure-built `Subscription::run`?
   → Use a function pointer or `run_with(hash, fn)`. (CRITICAL)
7. Is the new code using the deprecated `Component` trait?
   → Refactor to nested Elm Architecture. (CRITICAL)
8. For new custom widgets: is `diff()` calling `diff_children()`?
   Is `tag()` distinguishing the widget type? (CRITICAL)
9. Are `Message` variants carrying large cloned payloads?
   → Use IDs/indices instead. (MEDIUM)
10. For overlay widgets: is the overlay positioned relative to host bounds
    (not viewport)? Is the root node sized minimally? (CRITICAL)
11. Does the overlay's `mouse_interaction()` return `None` when the cursor
    is not over an interactive overlay element? (CRITICAL)
12. Does the overlay recompute hover state on `ButtonPressed` (not just
    `CursorMoved`) to handle layout shifts between frames? (CRITICAL)
13. Does the widget resolve a theme token (spacing, roundness, text size,
    color) against a hardcoded `Theme::DEFAULT_*` as the final value —
    especially in `layout()`? → Thread the live theme through (e.g. the
    `Cell<T>` cache written in `draw`, read in `layout`); only default for
    the seed/first frame. If truly impossible, document and notify. (CRITICAL)

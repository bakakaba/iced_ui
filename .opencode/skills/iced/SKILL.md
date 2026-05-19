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

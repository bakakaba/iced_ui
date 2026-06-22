//! Snapshot and interaction tests for the [`iced_ui::tree::Tree`] widget.

use iced::Length;
use iced::widget::text;
use iced_test::Error;
use iced_ui::tree;
use iced_ui_tests::{DEFAULT_SIZE, assert_snapshot, build};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Toggle(u32),
}

/// Builds a small sample tree. `root_expanded` controls whether the
/// "Fruits" branch (id 1) is open.
fn sample(root_expanded: bool) -> tree::Tree<'static, u32, Message> {
    tree::Tree::new()
        .node(
            tree::Node::new(1, text("Fruits"))
                .expanded(root_expanded)
                .push(tree::Node::new(10, text("Apple")))
                .push(tree::Node::new(11, text("Banana"))),
        )
        .node(tree::Node::new(2, text("Vegetables")).push(tree::Node::new(20, text("Carrot"))))
        .on_toggle(Message::Toggle)
        .width(Length::Fixed(240.0))
}

#[test]
fn tree_collapsed() -> Result<(), Error> {
    assert_snapshot("tree_collapsed", sample(false), DEFAULT_SIZE)
}

#[test]
fn tree_expanded() -> Result<(), Error> {
    // The expanded branch reveals indentation derived from the theme
    // spacing token.
    assert_snapshot("tree_expanded", sample(true), DEFAULT_SIZE)
}

#[test]
fn tree_custom_indicator() -> Result<(), Error> {
    let element = tree::Tree::new()
        .node(
            tree::Node::new(1u32, text("Folder"))
                .expanded(true)
                .push(tree::Node::new(10, text("File A")))
                .push(tree::Node::new(11, text("File B"))),
        )
        .expanded_indicator(|| text("-").into())
        .collapsed_indicator(|| text("+").into())
        .on_toggle(Message::Toggle)
        .width(Length::Fixed(240.0));

    assert_snapshot("tree_custom_indicator", element, DEFAULT_SIZE)
}

#[test]
fn tree_leaf_click_does_not_toggle() -> Result<(), Error> {
    // Clicking a leaf node's label must never emit a toggle: only
    // nodes that have children expose an interactive indicator.
    let mut sim = build(sample(true), DEFAULT_SIZE);
    sim.click("Apple").ok();

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        !messages.iter().any(|m| matches!(m, Message::Toggle(_))),
        "leaf click should not toggle, got {messages:?}"
    );
    Ok(())
}

#[test]
fn tree_indicator_click_emits_toggle() -> Result<(), Error> {
    use iced_test::simulator::click;

    // The first root node ("Fruits", id 1) has children, so its
    // disclosure indicator is interactive. It sits at the top-left of
    // the widget, just inside the outer padding. Click that region and
    // assert a toggle for id 1 is emitted.
    let mut sim = build(sample(true), DEFAULT_SIZE);

    // Force an initial layout/draw pass so widget bounds are computed
    // before we drive raw cursor events.
    sim.snapshot(&iced_ui_tests::theme())?;

    sim.point_at(iced::Point::new(16.0, 26.0));
    let _ = sim.simulate(click());

    let messages = sim.into_messages().collect::<Vec<_>>();
    assert!(
        messages.contains(&Message::Toggle(1)),
        "expected Toggle(1) from indicator click, got {messages:?}"
    );
    Ok(())
}

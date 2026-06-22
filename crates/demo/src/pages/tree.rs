use std::collections::HashSet;

use iced::Length;
use iced::widget::{column, text};
use iced_ui::text::Text;
use iced_ui::tree;

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Toggle(u32),
}

pub(crate) struct TreePage {
    expanded: HashSet<u32>,
}

impl Default for TreePage {
    fn default() -> Self {
        // Start with the two top-level folders expanded so the default
        // showcase reveals the indentation and indicators.
        Self {
            expanded: HashSet::from([1, 2]),
        }
    }
}

impl super::PageView for TreePage {
    type Msg = Msg;
    const LABEL: &'static str = "Tree";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Toggle(id) => {
                if !self.expanded.remove(&id) {
                    self.expanded.insert(id);
                }
            }
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let is = |id: u32| self.expanded.contains(&id);

        let example = tree::Tree::new()
            .node(
                tree::Node::new(1, text("Fruits"))
                    .expanded(is(1))
                    .push(tree::Node::new(10, text("Apple")))
                    .push(tree::Node::new(11, text("Banana")))
                    .push(
                        tree::Node::new(12, text("Berries"))
                            .expanded(is(12))
                            .push(tree::Node::new(120, text("Strawberry")))
                            .push(tree::Node::new(121, text("Blueberry"))),
                    ),
            )
            .node(
                tree::Node::new(2, text("Vegetables"))
                    .expanded(is(2))
                    .push(tree::Node::new(20, text("Carrot")))
                    .push(tree::Node::new(21, text("Potato"))),
            )
            .on_toggle(Msg::Toggle)
            .width(Length::Fixed(260.0));

        column![
            Text::h1("Tree"),
            text("A hierarchical tree of expandable nodes. Click a chevron to toggle.").size(14),
            Text::h2("Default"),
            example,
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}

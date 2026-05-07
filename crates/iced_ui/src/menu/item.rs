//! Entries that a [`Menu`] may contain and their builders.

use iced::Font;

use super::shortcut::KeyBinding;

/// An optional icon rendered at the leading edge of an [`Item`].
///
/// Icons are rendered as text, which lets you use either:
///
/// - a single **glyph from an icon font** — call
///   [`Icon::from_char`] with a `char` and pass your icon font via
///   [`Icon::font`]; or
/// - a **short text fragment** — call [`Icon::from_text`] with a
///   string, which will be rendered using the menu's default font
///   unless overridden.
///
/// _Note_: In v1 icons are always rendered as text. Supporting arbitrary
/// [`Element`](iced::Element)s as icons is planned for a follow-up
/// release.
#[derive(Debug, Clone)]
pub struct Icon {
    pub(super) content: String,
    pub(super) font: Option<Font>,
}

impl Icon {
    /// Creates an [`Icon`] from a single glyph. Typically used together
    /// with [`Icon::font`] to select an icon font.
    pub fn from_char(glyph: char) -> Self {
        Self {
            content: glyph.to_string(),
            font: None,
        }
    }

    /// Creates an [`Icon`] from a short text fragment.
    pub fn from_text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font: None,
        }
    }

    /// Sets the font used to render the icon. This is typically an icon
    /// font such as Font Awesome or a custom glyph font.
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }
}

impl From<char> for Icon {
    fn from(glyph: char) -> Self {
        Self::from_char(glyph)
    }
}

impl From<&str> for Icon {
    fn from(text: &str) -> Self {
        Self::from_text(text)
    }
}

impl From<String> for Icon {
    fn from(text: String) -> Self {
        Self::from_text(text)
    }
}

/// A single selectable row inside a [`Menu`].
///
/// An [`Item`] has a text label and optionally:
///
/// - a leading [`Icon`],
/// - a trailing [`KeyBinding`] shortcut,
/// - a message to publish when it is activated.
///
/// Items without an `on_press` message are inert (they render but
/// cannot be activated). Use [`Item::enabled`] to visually disable an
/// item while keeping it in the list.
pub struct Item<Message> {
    pub(super) label: String,
    pub(super) icon: Option<Icon>,
    pub(super) shortcut: Option<KeyBinding>,
    pub(super) on_press: Option<Message>,
    pub(super) enabled: bool,
    pub(super) checked: bool,
}

impl<Message> Item<Message> {
    /// Creates a new [`Item`] with the given label and no other
    /// properties set.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            shortcut: None,
            on_press: None,
            enabled: true,
            checked: false,
        }
    }

    /// Sets the leading icon for this [`Item`].
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the trailing keyboard shortcut for this [`Item`].
    ///
    /// The shortcut is rendered as a right-aligned label on the row.
    /// To actually bind the shortcut at runtime, feed the list of
    /// `(KeyBinding, Message)` pairs from
    /// [`MenuBar::shortcuts`](crate::menu::MenuBar::shortcuts) into
    /// [`crate::menu::shortcuts`] and merge the result into your
    /// application's subscription.
    pub fn shortcut(mut self, binding: impl Into<KeyBinding>) -> Self {
        self.shortcut = Some(binding.into());
        self
    }

    /// Sets the message to publish when the item is activated.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Enables or disables the item. Disabled items render with a
    /// dimmed color and do not fire their `on_press` message nor
    /// their shortcut.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Marks this item as checked, drawing a check glyph in the
    /// leading icon column. Useful for toggleable items so the menu
    /// communicates the current state.
    ///
    /// When an item has both a [custom icon](Self::icon) and is
    /// checked, the check glyph takes precedence over the icon for
    /// the duration the item is checked.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub(super) fn is_activatable(&self) -> bool {
        self.enabled && self.on_press.is_some()
    }
}

/// A visual divider between groups of items inside a menu.
#[derive(Debug, Clone, Copy)]
pub struct Separator;

/// A menu of [`Item`]s, separators and nested sub-[`Menu`]s.
///
/// When placed at the top level of a
/// [`MenuBar`](crate::menu::MenuBar), a [`Menu`] renders its label in
/// the bar and opens as a dropdown when activated.
///
/// When placed inside another [`Menu`], it becomes a submenu entry
/// that opens sideways on hover or activation.
pub struct Menu<Message> {
    pub(super) label: String,
    pub(super) entries: Vec<Entry<Message>>,
}

impl<Message> Menu<Message> {
    /// Creates a new, empty [`Menu`] with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            entries: Vec::new(),
        }
    }

    /// Appends an [`Item`] to this menu.
    pub fn push(mut self, item: impl Into<Entry<Message>>) -> Self {
        self.entries.push(item.into());
        self
    }

    /// Appends a [`Separator`] to this menu.
    ///
    /// Convenience shortcut for `menu.push(Separator)`.
    pub fn separator(self) -> Self {
        self.push(Separator)
    }

    /// Appends a nested sub-[`Menu`] to this menu.
    pub fn submenu(self, submenu: Menu<Message>) -> Self {
        self.push(submenu)
    }
}

/// An entry within a [`Menu`]: an [`Item`], a [`Separator`], or a
/// nested [`Menu`].
pub enum Entry<Message> {
    /// An activatable item.
    Item(Item<Message>),
    /// A horizontal divider.
    Separator,
    /// A nested submenu.
    Submenu(Menu<Message>),
}

impl<Message> From<Item<Message>> for Entry<Message> {
    fn from(item: Item<Message>) -> Self {
        Entry::Item(item)
    }
}

impl<Message> From<Separator> for Entry<Message> {
    fn from(_: Separator) -> Self {
        Entry::Separator
    }
}

impl<Message> From<Menu<Message>> for Entry<Message> {
    fn from(menu: Menu<Message>) -> Self {
        Entry::Submenu(menu)
    }
}

/// Walks the menu tree and returns every `(binding, message)` pair
/// reachable from the given list of top-level menus — including items
/// inside nested submenus, but excluding disabled items and items
/// without an `on_press`.
pub(super) fn collect_shortcuts<Message: Clone>(
    menus: &[Menu<Message>],
) -> Vec<(KeyBinding, Message)> {
    fn visit<Message: Clone>(menu: &Menu<Message>, out: &mut Vec<(KeyBinding, Message)>) {
        for entry in &menu.entries {
            match entry {
                Entry::Item(item) => {
                    if !item.enabled {
                        continue;
                    }
                    if let (Some(binding), Some(msg)) = (&item.shortcut, &item.on_press) {
                        out.push((binding.clone(), msg.clone()));
                    }
                }
                Entry::Submenu(sub) => visit(sub, out),
                Entry::Separator => {}
            }
        }
    }

    let mut out = Vec::new();
    for menu in menus {
        visit(menu, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_shortcuts_walks_nested_submenus() {
        let menu = Menu::<u32>::new("File")
            .push(Item::new("New").shortcut(KeyBinding::ctrl('n')).on_press(1))
            .push(Separator)
            .push(
                Menu::new("Recent")
                    .push(Item::new("a").shortcut(KeyBinding::ctrl('r')).on_press(2))
                    .push(
                        Item::new("b")
                            .shortcut(KeyBinding::ctrl('b'))
                            .on_press(3)
                            .enabled(false),
                    ),
            );

        let bindings = collect_shortcuts(&[menu]);
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].1, 1);
        assert_eq!(bindings[1].1, 2);
    }

    #[test]
    fn collect_shortcuts_ignores_items_without_on_press() {
        let menu =
            Menu::<u32>::new("Menu").push(Item::new("no-message").shortcut(KeyBinding::ctrl('z')));
        assert!(collect_shortcuts(&[menu]).is_empty());
    }
}

//! Keyboard shortcuts for menu items.
//!
//! This module defines the [`KeyBinding`] type — a typed, displayable
//! shortcut used by [`Item::shortcut`] — and the [`shortcuts`]
//! subscription helper that dispatches a collected list of bindings to
//! their corresponding messages whenever the application is focused.
//!
//! [`Item::shortcut`]: crate::menu::Item::shortcut

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use iced::Subscription;
use iced::event::{self, Event};
use iced::keyboard::{self, Key, Modifiers, key::Named};

/// A typed keyboard shortcut that can both be displayed next to a menu
/// item and be pattern-matched against runtime [`keyboard::Event`]s.
///
/// The shortcut's [`Display`] impl renders a platform-aware label such as
/// `Ctrl+Shift+Z` (non-macOS) or `⌘⇧Z` (macOS), suitable for showing on
/// the right side of a menu row.
///
/// [`Display`]: fmt::Display
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    /// The modifier keys that must be held for the shortcut to fire.
    pub modifiers: Modifiers,
    /// The non-modifier key that triggers the shortcut.
    pub key: Key,
}

impl KeyBinding {
    /// Creates a new binding from the given modifiers and key.
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// A binding that uses the platform "command" modifier — `Ctrl` on
    /// most platforms, `⌘` on macOS — plus the given character.
    pub fn command(character: char) -> Self {
        Self {
            modifiers: Modifiers::COMMAND,
            key: Key::Character(character.to_lowercase().to_string().into()),
        }
    }

    /// A binding that uses `Ctrl` plus the given character, regardless of
    /// platform. Prefer [`Self::command`] for "platform primary" bindings.
    pub fn ctrl(character: char) -> Self {
        Self {
            modifiers: Modifiers::CTRL,
            key: Key::Character(character.to_lowercase().to_string().into()),
        }
    }

    /// Returns a new binding with [`Modifiers::SHIFT`] added.
    pub fn shift(mut self) -> Self {
        self.modifiers |= Modifiers::SHIFT;
        self
    }

    /// Returns a new binding with [`Modifiers::ALT`] added.
    pub fn alt(mut self) -> Self {
        self.modifiers |= Modifiers::ALT;
        self
    }

    /// Returns a new binding with [`Modifiers::CTRL`] added.
    pub fn with_ctrl(mut self) -> Self {
        self.modifiers |= Modifiers::CTRL;
        self
    }

    /// Returns a new binding with [`Modifiers::LOGO`] added.
    pub fn logo(mut self) -> Self {
        self.modifiers |= Modifiers::LOGO;
        self
    }

    /// Returns `true` if the given modifiers and key match this binding.
    pub fn matches(&self, modifiers: Modifiers, key: &Key) -> bool {
        self.modifiers == modifiers && keys_equal(&self.key, key)
    }
}

fn keys_equal(a: &Key, b: &Key) -> bool {
    match (a, b) {
        (Key::Character(a), Key::Character(b)) => a.eq_ignore_ascii_case(b.as_str()),
        (Key::Named(a), Key::Named(b)) => a == b,
        (Key::Unidentified, Key::Unidentified) => true,
        _ => false,
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Compact, symbol-only rendering on every platform. Glyphs are
        // rendered without separators between them; the key (if any)
        // also follows immediately after the last modifier.
        let mut first = true;
        let mut write_mod = |f: &mut fmt::Formatter<'_>, s: &str| -> fmt::Result {
            first = false;
            f.write_str(s)
        };

        if self.modifiers.contains(Modifiers::CTRL) {
            write_mod(f, "⌃")?;
        }
        if self.modifiers.contains(Modifiers::ALT) {
            write_mod(f, "⌥")?;
        }
        if self.modifiers.contains(Modifiers::SHIFT) {
            write_mod(f, "⇧")?;
        }
        if self.modifiers.contains(Modifiers::LOGO) {
            let logo = if cfg!(target_os = "macos") {
                "⌘"
            } else {
                "❖"
            };
            write_mod(f, logo)?;
        }

        let _ = first;

        match &self.key {
            Key::Character(c) => {
                for ch in c.chars() {
                    for upper in ch.to_uppercase() {
                        f.write_fmt(format_args!("{upper}"))?;
                    }
                }
                Ok(())
            }
            Key::Named(named) => f.write_str(named_label(*named)),
            Key::Unidentified => f.write_str("?"),
        }
    }
}

fn named_label(named: Named) -> &'static str {
    match named {
        Named::Enter => "Enter",
        Named::Tab => "Tab",
        Named::Space => "Space",
        Named::Backspace => "Backspace",
        Named::Delete => "Delete",
        Named::Escape => "Esc",
        Named::ArrowLeft => "←",
        Named::ArrowRight => "→",
        Named::ArrowUp => "↑",
        Named::ArrowDown => "↓",
        Named::Home => "Home",
        Named::End => "End",
        Named::PageUp => "PgUp",
        Named::PageDown => "PgDn",
        Named::Insert => "Ins",
        Named::F1 => "F1",
        Named::F2 => "F2",
        Named::F3 => "F3",
        Named::F4 => "F4",
        Named::F5 => "F5",
        Named::F6 => "F6",
        Named::F7 => "F7",
        Named::F8 => "F8",
        Named::F9 => "F9",
        Named::F10 => "F10",
        Named::F11 => "F11",
        Named::F12 => "F12",
        _ => "?",
    }
}

/// Produces a [`Subscription`] that listens for keyboard events and
/// dispatches the matching message whenever one of the provided
/// shortcuts is pressed.
///
/// Callers typically assemble the list of `(KeyBinding, Message)` pairs
/// via [`MenuBar::shortcuts`](crate::menu::MenuBar::shortcuts) and then
/// merge the returned subscription into their application's
/// `subscription` function.
pub fn shortcuts<Message>(bindings: Vec<(KeyBinding, Message)>) -> Subscription<Message>
where
    Message: Clone + Send + Sync + 'static,
{
    if bindings.is_empty() {
        return Subscription::none();
    }

    let set = BindingSet(Arc::new(bindings));

    event::listen()
        .with(set)
        .filter_map(|(set, event)| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => set
                .0
                .iter()
                .find(|(binding, _)| binding.matches(modifiers, &key))
                .map(|(_, message)| message.clone()),
            _ => None,
        })
}

/// Wrapper that implements [`Hash`], [`Clone`], [`Send`], [`Sync`] and
/// `'static` on top of a `Vec<(KeyBinding, Message)>` without
/// requiring the `Message` itself to implement `Hash` — the hash
/// comes from the bindings alone, which is enough to identify the
/// subscription.
#[derive(Debug)]
struct BindingSet<Message>(Arc<Vec<(KeyBinding, Message)>>);

impl<Message> Clone for BindingSet<Message> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<Message> Hash for BindingSet<Message> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for (binding, _) in self.0.iter() {
            binding.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_character_case_insensitively() {
        let binding = KeyBinding::ctrl('n');
        assert!(binding.matches(Modifiers::CTRL, &Key::Character("N".into()),));
        assert!(binding.matches(Modifiers::CTRL, &Key::Character("n".into()),));
        assert!(!binding.matches(Modifiers::empty(), &Key::Character("n".into()),));
        assert!(!binding.matches(Modifiers::CTRL, &Key::Character("m".into()),));
    }

    #[test]
    fn matches_named_key() {
        let binding = KeyBinding::new(Modifiers::empty(), Key::Named(Named::F1));
        assert!(binding.matches(Modifiers::empty(), &Key::Named(Named::F1),));
        assert!(!binding.matches(Modifiers::empty(), &Key::Named(Named::F2),));
    }

    #[test]
    fn shift_adds_shift_modifier() {
        let binding = KeyBinding::ctrl('z').shift();
        assert!(binding.modifiers.contains(Modifiers::CTRL));
        assert!(binding.modifiers.contains(Modifiers::SHIFT));
    }

    #[test]
    fn display_non_empty() {
        // Just make sure Display doesn't panic and yields a non-empty label
        // for a variety of inputs.
        assert!(!KeyBinding::ctrl('n').to_string().is_empty());
        assert!(
            !KeyBinding::new(Modifiers::empty(), Key::Named(Named::Escape))
                .to_string()
                .is_empty()
        );
        assert!(!KeyBinding::command('s').shift().to_string().is_empty());
    }

    #[test]
    fn display_uses_symbol_modifiers() {
        // We deliberately render modifiers as compact glyphs on every
        // platform; the textual words `Ctrl`, `Alt`, `Shift`, `Super`
        // should not appear anywhere in the formatted output.
        let s = KeyBinding::ctrl('z').shift().alt().to_string();
        assert!(!s.contains("Ctrl"));
        assert!(!s.contains("Shift"));
        assert!(!s.contains("Alt"));
        assert!(!s.contains("Super"));
        assert!(!s.contains('+'));
        assert!(s.contains('⌃'));
        assert!(s.contains('⌥'));
        assert!(s.contains('⇧'));
        assert!(s.ends_with('Z'));
    }
}

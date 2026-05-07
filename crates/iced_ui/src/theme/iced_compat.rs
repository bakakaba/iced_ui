//! Catalog delegations that let `iced_ui::Theme` style the iced
//! built-in widgets.
//!
//! Every catalog impl here forwards to the corresponding impl on
//! `iced::Theme`, using the [`Theme::colors`] field as the underlying
//! source. Application code therefore only ever needs to handle
//! `iced_ui::Theme`; iced built-ins like `text`, `container`,
//! `pick_list`, `checkbox`, `slider` and `scrollable` keep their
//! native look while remaining tied to the same theme as `iced_ui`'s
//! own widgets.
//!
//! [`Theme::colors`]: crate::Theme::colors

use iced::theme as iced_theme;
use iced::widget::{checkbox, container, overlay, pick_list, scrollable, slider, text};

use crate::Theme;

impl iced_theme::Base for Theme {
    fn default(preference: iced_theme::Mode) -> Self {
        Self::from(<iced::Theme as iced_theme::Base>::default(preference))
    }

    fn mode(&self) -> iced_theme::Mode {
        self.colors.mode()
    }

    fn base(&self) -> iced_theme::Style {
        self.colors.base()
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(self.colors.palette())
    }

    fn name(&self) -> &str {
        self.colors.name()
    }
}

impl text::Catalog for Theme {
    type Class<'a> = text::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| text::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        class(self)
    }
}

impl container::Catalog for Theme {
    type Class<'a> = container::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| container::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        class(self)
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = checkbox::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme: &Theme, status| checkbox::primary(&theme.colors, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        class(self, status)
    }
}

impl slider::Catalog for Theme {
    type Class<'a> = slider::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme: &Theme, status| slider::default(&theme.colors, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: slider::Status) -> slider::Style {
        class(self, status)
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = scrollable::StyleFn<'a, Theme>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme: &Theme, status| scrollable::default(&theme.colors, status))
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        class(self, status)
    }
}

impl overlay::menu::Catalog for Theme {
    type Class<'a> = overlay::menu::StyleFn<'a, Theme>;

    fn default<'a>() -> <Self as overlay::menu::Catalog>::Class<'a> {
        Box::new(|theme: &Theme| overlay::menu::default(&theme.colors))
    }

    fn style(&self, class: &<Self as overlay::menu::Catalog>::Class<'_>) -> overlay::menu::Style {
        class(self)
    }
}

impl pick_list::Catalog for Theme {
    type Class<'a> = pick_list::StyleFn<'a, Theme>;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        Box::new(|theme: &Theme, status| pick_list::default(&theme.colors, status))
    }

    fn style(
        &self,
        class: &<Self as pick_list::Catalog>::Class<'_>,
        status: pick_list::Status,
    ) -> pick_list::Style {
        class(self, status)
    }
}

mod badge;
mod button;
mod card;
mod chip;
mod colors;
mod dialog;
mod divider;
mod fab;
mod icon_button;
mod inputs;
mod list;
mod menu;
mod navigation_bar;
mod navigation_drawer;
mod navigation_rail;
mod overview;
mod screen;
mod segmented_button;
mod slide_sheet;
mod snackbar;
mod tabs;
mod text;
mod top_app_bar;

use std::fmt::Debug;

use crate::Element;
pub(crate) use crate::state::Action;
use crate::state::ActionLog;

/// Trait implemented by every page. All page metadata and behavior is
/// co-located in the page's own module.
pub(super) trait PageView: Default {
    type Msg: Debug + Clone;

    const LABEL: &'static str;

    fn update(&mut self, _msg: Self::Msg) -> Action {
        Action::None
    }

    fn view(&self, log: &ActionLog) -> Element<'_, Self::Msg>;
}

/// Generates `Page` enum, `ActivePage` enum, `pages::Message` enum,
/// and all dispatch logic from a single grouped page list.
macro_rules! pages {
    (
        $(
            $group:ident {
                $( $(#[$meta:meta])* $variant:ident ( $page_ty:ty ) ),* $(,)?
            }
        )*
    ) => {
        /// Flat enum used for navigation identity, labels, and group membership.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub(crate) enum Page {
            $($( $(#[$meta])* $variant, )*)*
        }

        impl Page {
            pub(crate) fn label(self) -> &'static str {
                match self {
                    $($( Self::$variant => <$page_ty as PageView>::LABEL, )*)*
                }
            }

            $(
                pub(crate) const $group: &[Self] = &[
                    $( Self::$variant, )*
                ];
            )*
        }

        /// The active page — state is embedded in the variant. Created fresh
        /// on navigation, dropped when navigating away.
        pub(crate) enum ActivePage {
            $($( $variant($page_ty), )*)*
        }

        impl Default for ActivePage {
            fn default() -> Self {
                Self::navigate(Page::default())
            }
        }

        impl ActivePage {
            pub(crate) fn navigate(page: Page) -> Self {
                match page {
                    $($( Page::$variant => Self::$variant(<$page_ty>::default()), )*)*
                }
            }

            pub(crate) fn page(&self) -> Page {
                match self {
                    $($( Self::$variant(_) => Page::$variant, )*)*
                }
            }

            pub(crate) fn update(&mut self, msg: Message) -> Action {
                match (self, msg) {
                    $($( (Self::$variant(state), Message::$variant(m)) => state.update(m), )*)*
                    _ => Action::None,
                }
            }

            pub(crate) fn view(&self, log: &ActionLog) -> Element<'_, Message> {
                match self {
                    $($( Self::$variant(state) => state.view(log).map(Message::$variant), )*)*
                }
            }
        }

        /// Page message enum — one variant per page wrapping that page's `Msg` type.
        #[derive(Debug, Clone)]
        pub(crate) enum Message {
            $($( $variant(<$page_ty as PageView>::Msg), )*)*
        }
    };
}

pages! {
    SHOWCASE {
        #[default]
        Overview(overview::OverviewPage),
    }
    WIDGETS {
        Badge(badge::BadgePage),
        Button(button::ButtonPage),
        Card(card::CardPage),
        Chip(chip::ChipPage),
        Colors(colors::ColorsPage),
        Dialog(dialog::DialogPage),
        Divider(divider::DividerPage),
        Fab(fab::FabPage),
        IconButton(icon_button::IconButtonPage),
        Inputs(inputs::InputsPage),
        List(list::ListPage),
        Menu(menu::MenuPage),
        NavigationBar(navigation_bar::NavigationBarPage),
        NavigationDrawer(navigation_drawer::NavigationDrawerPage),
        NavigationRail(navigation_rail::NavigationRailPage),
        Screen(screen::ScreenPage),
        SegmentedButton(segmented_button::SegmentedButtonPage),
        SlideSheet(slide_sheet::SlideSheetPage),
        Snackbar(snackbar::SnackbarPage),
        Tabs(tabs::TabsPage),
        Text(text::TextPage),
        TopAppBar(top_app_bar::TopAppBarPage),
    }
}

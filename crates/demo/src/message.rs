use crate::pages::Page;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum PaletteField {
    Background,
    Text,
    Primary,
    Success,
    Warning,
    Danger,
}

impl PaletteField {
    pub(crate) const ALL: [Self; 6] = [
        Self::Background,
        Self::Text,
        Self::Primary,
        Self::Success,
        Self::Warning,
        Self::Danger,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::Text => "Text",
            Self::Primary => "Primary",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Danger => "Danger",
        }
    }

    pub(crate) fn get(self, palette: &iced::theme::Palette) -> iced::Color {
        match self {
            Self::Background => palette.background,
            Self::Text => palette.text,
            Self::Primary => palette.primary,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
        }
    }

    pub(crate) fn set(self, palette: &mut iced::theme::Palette, color: iced::Color) {
        match self {
            Self::Background => palette.background = color,
            Self::Text => palette.text = color,
            Self::Primary => palette.primary = color,
            Self::Success => palette.success = color,
            Self::Warning => palette.warning = color,
            Self::Danger => palette.danger = color,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) enum Action {
    New,
    Open,
    OpenRecent(u8),
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleSidebar,
    About,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Triggered(Action),
    ThemeSelected(iced::Theme),
    CustomizeToggled(bool),
    PaletteFieldChanged {
        field: PaletteField,
        color: iced::Color,
    },
    InformationColorChanged(iced::Color),
    RoundnessChanged(u8),
    SpacingChanged(u8),
    Navigate(Page),
    // Interactive demo messages
    IconButtonToggled,
    ChipToggled,
    SegmentSelected(usize),
    OpenDialog,
    CloseDialog,
    DialogConfirmed,
    ShowSnackbar,
    HideSnackbar,
    ToggleBottomSheet,
    CloseBottomSheet,
    TabSelected(usize),
    ToggleDrawer,
    CloseDrawer,
    DrawerItemSelected(usize),
    FabPressed,
    PickerColorChanged(iced::Color),
    Noop,
}

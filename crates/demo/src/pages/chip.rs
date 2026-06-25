use iced::Color;
use iced::widget::{column, row, text};
use iced_ui::chip::{Chip, ChipColor, ChipSize};
use iced_ui::text::Text;

use crate::Element;
use crate::state::ActionLog;

const FILTERS: [&str; 3] = ["Vegetarian", "Vegan", "Gluten-free"];

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    ToggleFilter(usize),
    Removed(usize),
    ToggleSelected,
    RemoveCombined,
}

#[derive(Default)]
pub(crate) struct ChipPage {
    filters: [bool; 3],
    removed: [bool; 3],
    combined_selected: bool,
    combined_removed: bool,
}

impl super::PageView for ChipPage {
    type Msg = Msg;
    const LABEL: &'static str = "Chip";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::ToggleFilter(i) => self.filters[i] = !self.filters[i],
            Msg::Removed(i) => self.removed[i] = true,
            Msg::ToggleSelected => self.combined_selected = !self.combined_selected,
            Msg::RemoveCombined => self.combined_removed = true,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        // Colors are static showcases (no handler => no pointer/hover).
        let default = Chip::new("Add event");
        let primary = Chip::new("Primary").color(ChipColor::Primary);
        let secondary = Chip::new("Secondary").color(ChipColor::Secondary);
        let success = Chip::new("Success").color(ChipColor::Success);
        let information = Chip::new("Information").color(ChipColor::Information);
        let warning = Chip::new("Warning").color(ChipColor::Warning);
        let danger = Chip::new("Danger").color(ChipColor::Danger);
        let foreground = Chip::new("Foreground").color(ChipColor::Foreground);
        let custom = Chip::new("Custom").color(ChipColor::Custom(Color::from_rgb(0.55, 0.2, 0.75)));

        // Sizes use the default (no color) outlined pill.
        let small = Chip::new("Small").size(ChipSize::Sm);
        let medium = Chip::new("Medium").size(ChipSize::Md);
        let large = Chip::new("Large").size(ChipSize::Lg);

        // Toggle-only: clicking the body toggles; color shows selection.
        let mut toggles = row![].spacing(12);
        for (i, label) in FILTERS.iter().enumerate() {
            let mut chip = Chip::new(*label).on_toggle(Msg::ToggleFilter(i));
            if self.filters[i] {
                chip = chip.color(ChipColor::Primary);
            }
            toggles = toggles.push(chip);
        }

        // Remove-only: the pointer appears only over the x button.
        let mut removables = row![].spacing(12);
        for (i, label) in FILTERS.iter().enumerate() {
            if !self.removed[i] {
                removables = removables.push(
                    Chip::new(*label)
                        .color(ChipColor::Secondary)
                        .on_remove(Msg::Removed(i)),
                );
            }
        }

        // Toggle + removable: body toggles, x button removes.
        let mut combined = row![].spacing(12);
        if !self.combined_removed {
            let mut chip = Chip::new("Assignee")
                .on_toggle(Msg::ToggleSelected)
                .on_remove(Msg::RemoveCombined);
            if self.combined_selected {
                chip = chip.color(ChipColor::Primary);
            }
            combined = combined.push(chip);
        }

        // Disabled: rendered faded and non-interactive even with
        // handlers present. Shown in both the outlined and filled looks.
        let disabled_toggle_outlined = Chip::new("Toggle")
            .on_toggle(Msg::ToggleSelected)
            .enabled(false);
        let disabled_toggle_filled = Chip::new("Toggle")
            .color(ChipColor::Primary)
            .on_toggle(Msg::ToggleSelected)
            .enabled(false);
        let disabled_combined_outlined = Chip::new("Assignee")
            .on_toggle(Msg::ToggleSelected)
            .on_remove(Msg::RemoveCombined)
            .enabled(false);
        let disabled_combined_filled = Chip::new("Assignee")
            .color(ChipColor::Primary)
            .on_toggle(Msg::ToggleSelected)
            .on_remove(Msg::RemoveCombined)
            .enabled(false);

        column![
            text("Pill-shaped chips. Default is an outlined pill; a color fills it.").size(14),
            Text::h2("Colors"),
            row![default, primary, secondary, success, information].spacing(12),
            row![warning, danger, foreground, custom].spacing(12),
            Text::h2("Sizes"),
            row![small, medium, large].spacing(12),
            Text::h2("Interactive"),
            text("Toggle").size(12),
            toggles,
            text("Removable").size(12),
            removables,
            text("Toggle + removable").size(12),
            combined,
            Text::h2("Disabled"),
            text("Toggle").size(12),
            row![disabled_toggle_outlined, disabled_toggle_filled].spacing(12),
            text("Toggle + removable").size(12),
            row![disabled_combined_outlined, disabled_combined_filled].spacing(12),
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}

use iced::widget::{column, row, text};
use iced_ui::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use iced_ui::text::Text;
use iced_ui::{DateInput, DateTimeInput, TimeInput};

use crate::Element;
use crate::state::ActionLog;

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
}

pub(crate) struct DateTimePage {
    date: NaiveDate,
    time: NaiveTime,
    datetime: NaiveDateTime,
}

impl Default for DateTimePage {
    fn default() -> Self {
        let date = NaiveDate::from_ymd_opt(2026, 6, 12).expect("valid date");
        let time = NaiveTime::from_hms_opt(9, 30, 0).expect("valid time");
        Self {
            date,
            time,
            datetime: date.and_time(time),
        }
    }
}

impl super::PageView for DateTimePage {
    type Msg = Msg;
    const LABEL: &'static str = "Date & Time";

    fn update(&mut self, msg: Msg) -> super::Action {
        match msg {
            Msg::Date(value) => self.date = value,
            Msg::Time(value) => self.time = value,
            Msg::DateTime(value) => self.datetime = value,
        }
        super::Action::None
    }

    fn view(&self, _log: &ActionLog) -> Element<'_, Msg> {
        let date_input = DateInput::new(self.date).on_change(Msg::Date);

        let time_input = TimeInput::new(self.time).on_change(Msg::Time);

        let datetime_input = DateTimeInput::new(self.datetime).on_change(Msg::DateTime);

        column![
            text("Date, time, and combined date-time inputs with picker popups.").size(14),
            Text::h2("Date Input"),
            row![column![date_input, text(format!("Value: {}", self.date)).size(12),].spacing(4),],
            Text::h2("Time Input"),
            row![
                column![
                    time_input,
                    text(format!("Value: {}", self.time.format("%H:%M"))).size(12),
                ]
                .spacing(4),
            ],
            Text::h2("Date-Time Input"),
            row![
                column![
                    datetime_input,
                    text(format!("Value: {}", self.datetime.format("%Y-%m-%d %H:%M"))).size(12),
                ]
                .spacing(4),
            ],
        ]
        .spacing(16)
        .padding(20)
        .into()
    }
}

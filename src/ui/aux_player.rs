use super::styles::*;
use crate::{App, Message, PickerTarget};
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Font, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub fn aux_players_panel(&self) -> Element<'_, Message> {
        let header = row![
            container(row![text("AUX PLAYERS").size(10)])
                .padding([6, 12])
                .style(panel_style(accent_purple(), accent_purple_border())),
            Space::with_width(Length::Fill),
            text(Bootstrap::CaretDownFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(13)
                .style(text_color(rgb(7, 13, 17))),
        ]
        .height(Length::Fixed(30.0))
        .align_y(Alignment::Center);

        let players = column![self.aux_row(0), self.aux_row(1), self.aux_row(2),]
            .spacing(1)
            .height(Length::Fill);

        container(column![header, players].height(Length::Fill))
            .height(Length::Fill)
            .style(panel_style(rgb(31, 46, 55), rgb(13, 22, 28)))
            .into()
    }

    pub fn aux_row(&self, index: usize) -> Element<'_, Message> {
        let loaded = self.aux_slots.get(index).and_then(Option::as_ref);
        let playing = self.aux_is_active(index);
        let looping = self.aux_loops.get(index).copied().unwrap_or(false);
        let title = loaded
            .map(|track| format!("{} - {}", track.artist, track.title))
            .unwrap_or_else(|| format!("AUX {} empty", index + 1));
        let (elapsed, remaining, total) = self.aux_timing(index);
        let (progress_filled, progress_empty) = self.aux_progress_parts(index);

        container(
            row![
                self.small_square_button(
                    text(Bootstrap::StopFill.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(13)
                        .into(),
                    rgb(59, 80, 94),
                    Message::AuxStop(index)
                ),
                self.small_square_button(
                    text(Bootstrap::PlayFill.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(13)
                        .into(),
                    if playing {
                        rgb(184, 42, 46)
                    } else {
                        rgb(59, 80, 94)
                    },
                    Message::AuxPlay(index)
                ),
                self.small_square_button(
                    text(Bootstrap::ArrowRepeat.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(13)
                        .into(),
                    if looping {
                        accent_purple()
                    } else {
                        rgb(59, 80, 94)
                    },
                    Message::AuxToggleLoop(index)
                ),
                container(
                    column![
                        text(title).size(11).style(text_color(rgb(219, 232, 240))),
                        row![
                            text(fmt_hms(elapsed))
                                .size(10)
                                .font(Font::MONOSPACE)
                                .style(text_color(rgb(46, 210, 249))),
                            text(fmt_hms(remaining))
                                .size(10)
                                .font(Font::MONOSPACE)
                                .style(text_color(rgb(46, 210, 249))),
                            text(fmt_hms(total))
                                .size(10)
                                .font(Font::MONOSPACE)
                                .style(text_color(rgb(167, 186, 199))),
                        ]
                        .spacing(10),
                        row![
                            container(Space::with_height(Length::Fixed(4.0)))
                                .width(Length::FillPortion(progress_filled))
                                .style(panel_style(accent_purple_hover(), rgb(109, 104, 101))),
                            container(Space::with_height(Length::Fixed(4.0)))
                                .width(Length::FillPortion(progress_empty))
                                .style(panel_style(rgb(35, 37, 38), rgb(109, 104, 101))),
                        ]
                        .spacing(0),
                    ]
                    .spacing(1)
                )
                .padding([0, 8])
                .width(Length::Fill),
                self.small_square_button(
                    text(Bootstrap::Upload.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(13)
                        .into(),
                    rgb(59, 80, 94),
                    Message::OpenTrackPicker(PickerTarget::Aux(index))
                ),
            ]
            .spacing(1)
            .width(Length::Fill)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(block_style(if playing {
            rgb(102, 58, 64)
        } else {
            rgb(63, 84, 97)
        }))
        .into()
    }
}

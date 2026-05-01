use super::styles::*;
use crate::{App, InstantView, Message, PickerTarget};
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Font, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub fn instant_panel(&self) -> Element<'_, Message> {
        let tabs = row![
            self.instant_tab(
                "SEARCH",
                self.instant_view == InstantView::Search,
                Message::ShowSearch
            ),
            self.instant_tab(
                "INSTANT PLAYERS",
                self.instant_view == InstantView::InstantPlayers,
                Message::ShowInstantPlayers,
            ),
            Space::with_width(Length::Fill),
            text(Bootstrap::CaretDownFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(13)
                .style(text_color(rgb(12, 18, 22))),
        ]
        .align_y(Alignment::Center)
        .height(Length::Fixed(31.0));

        let active_body = match self.instant_view {
            InstantView::Search => self.search_panel(),
            InstantView::InstantPlayers => self.instant_players_panel(),
        };

        let instant_area = column![tabs, active_body]
            .height(Length::Fill)
            .width(Length::Fill);

        container(instant_area)
            .height(Length::Fill)
            .style(panel_style(rgb(32, 47, 57), rgb(13, 22, 28)))
            .into()
    }

    pub fn instant_tab(
        &self,
        label: &'static str,
        active: bool,
        message: Message,
    ) -> Element<'_, Message> {
        let bg = if active {
            accent_purple()
        } else {
            rgb(52, 70, 82)
        };
        let border = if active {
            accent_purple_border()
        } else {
            rgb(27, 41, 50)
        };

        button(row![text(format!("{label}")).size(10)].align_y(Alignment::Center))
            .padding([7, 12])
            .on_press(message)
            .style(move |_, _| button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::WHITE,
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    pub fn instant_players_panel(&self) -> Element<'_, Message> {
        let mut grid = column![]
            .spacing(1)
            .height(Length::Fill)
            .width(Length::Fill);
        for row_index in 0..2 {
            let mut player_row = row![].spacing(1).height(Length::Fill);
            for column_index in 0..5 {
                let slot_index = row_index * 5 + column_index;
                player_row = player_row.push(self.instant_cell(slot_index, rgb(47, 63, 73)));
            }
            grid = grid.push(player_row);
        }

        let bottom_tools = row![
            self.small_square_button(
                text(Bootstrap::FloppyFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantSave
            ),
            self.small_square_button(
                text(Bootstrap::FilePlusFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantNewPage
            ),
            self.small_square_button(
                text(Bootstrap::TrashFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantDeletePage
            ),
            self.small_square_button(
                text(Bootstrap::ChevronDoubleLeft.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantPreviousPage
            ),
            Space::with_width(Length::Fill),
            text(self.active_instant_page_name())
                .size(10)
                .style(text_color(rgb(210, 223, 232))),
            Space::with_width(Length::Fill),
            self.small_square_button(
                text(Bootstrap::ChevronDoubleRight.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantNextPage
            ),
            self.small_square_button(
                text(Bootstrap::StopFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13)
                    .into(),
                rgb(52, 72, 84),
                Message::InstantStop
            ),
        ]
        .spacing(1)
        .align_y(Alignment::Center)
        .height(Length::Fixed(36.0));

        column![grid, bottom_tools]
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn instant_cell(&self, slot_index: usize, bg: Color) -> Element<'_, Message> {
        let loaded = self.instant_slots.get(slot_index).and_then(Option::as_ref);
        let playing = self.active_instant_slot == Some(slot_index);
        let cell_bg = if playing { rgb(184, 42, 46) } else { bg };
        let content: Element<_> = if let Some(track) = loaded {
            column![
                text(format!("{} - {}", track.artist, track.title))
                    .size(12)
                    .style(text_color(Color::WHITE))
                    .align_x(Horizontal::Center),
                text(fmt_dur(
                    self.instant_duration_display(slot_index, track.duration)
                ))
                .size(12)
                .font(Font::MONOSPACE)
                .style(text_color(rgb(235, 243, 248)))
                .align_x(Horizontal::Center),
            ]
            .spacing(4)
            .align_x(Alignment::Center)
            .into()
        } else {
            text((slot_index + 1).to_string())
                .size(12)
                .style(text_color(Color::WHITE))
                .align_x(Horizontal::Center)
                .into()
        };

        let cell = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(5)
            .style(panel_style(cell_bg, rgb(5, 12, 15)));

        if loaded.is_some() {
            button(cell)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .on_press(Message::InstantSlotPressed(slot_index))
                .style(move |_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed if !playing => {
                            rgb(58, 82, 97)
                        }
                        _ => cell_bg,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        color: rgb(5, 12, 15),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            button(cell)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .on_press(Message::OpenTrackPicker(PickerTarget::Instant(slot_index)))
                .style(move |_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed => rgb(58, 82, 97),
                        _ => bg,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        color: rgb(5, 12, 15),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }
    }

    pub fn small_square_button(
        &self,
        icon: Element<'static, Message>,
        bg: Color,
        message: Message,
    ) -> Element<'_, Message> {
        button(
            container(icon)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(Length::Fixed(38.0))
        .height(Length::Fill)
        .padding(0)
        .on_press(message)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                _ => bg,
            })),
            text_color: rgb(235, 243, 248),
            border: Border {
                color: rgb(29, 43, 52),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

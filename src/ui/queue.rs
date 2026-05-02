use super::styles::*;
use crate::{App, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text, Column, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub fn queue_panel(&self) -> Element<'_, Message> {
        let mut rows = Column::new().spacing(0);

        for (index, entry) in self.queue_entries.iter().enumerate() {
            let selected = self.selected_queue_index == Some(index);
            let previewing = self.previewing_queue_id == Some(entry.id);
            let scheduled = entry
                .scheduled_at
                .as_deref()
                .unwrap_or("--:--:--")
                .to_string();
            rows = rows.push(self.song_slot(
                index + 1,
                self.queue_entry_title(entry),
                self.queue_entry_meta(entry),
                scheduled,
                fmt_dur(entry.duration),
                index % 2 == 0,
                selected,
                previewing,
                Message::QueueRowSelected(index),
                Message::QueuePlayNow(index),
                Message::QueuePreviewToggle(entry.id),
            ));
        }

        let conductor = scrollable(rows).width(Length::Fill).height(Length::Fill);

        let autodj_label = if self.autodj_enabled {
            "AUTO MIX"
        } else {
            "MANUAL"
        };
        let side = column![
            self.side_mode(
                autodj_label,
                self.autodj_enabled,
                Some(Message::ToggleAutoDj)
            ),
            self.side_mode("INSERT", true, Some(Message::QueueInsertTrack)),
            self.side_mode("REPLACE", true, Some(Message::QueueReplaceTrack)),
            self.side_mode("REMOVE", false, Some(Message::QueueRemoveEntry)),
            self.side_mode("CLEAR", false, Some(Message::QueueClearAll)),
        ]
        .spacing(1)
        .width(Length::Fixed(78.0))
        .height(Length::Fill);

        let toolbar = row![
            self.toolbar_cell(
                text(Bootstrap::ArrowUp.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                Some(Message::QueueMoveUp),
            ),
            self.toolbar_cell(
                text(Bootstrap::ArrowBarUp.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                Some(Message::QueueMoveTop),
            ),
            self.toolbar_cell(
                text(Bootstrap::ArrowDown.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                Some(Message::QueueMoveDown),
            ),
            self.toolbar_cell(
                text(Bootstrap::ArrowBarDown.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                Some(Message::QueueMoveBottom),
            ),
            self.toolbar_cell(
                text(Bootstrap::FoldertwoOpen.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                None,
            ),
            self.toolbar_cell(
                text(Bootstrap::FloppyFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .into(),
                None,
            ),
        ]
        .spacing(1)
        .height(Length::Fixed(32.0));

        let main = column![conductor, toolbar]
            .height(Length::Fill)
            .width(Length::Fill);

        container(row![main, side].height(Length::Fill))
            .height(Length::Fill)
            .style(panel_style(rgb(26, 39, 48), rgb(12, 20, 25)))
            .into()
    }

    pub fn song_slot(
        &self,
        index: usize,
        title: String,
        meta: String,
        scheduled: String,
        duration: String,
        accented: bool,
        selected: bool,
        previewing: bool,
        on_press: Message,
        play_now_msg: Message,
        preview_msg: Message,
    ) -> Element<'_, Message> {
        let bg = if selected {
            rgb(112, 52, 204)
        } else if accented {
            accent_purple_dark()
        } else {
            accent_purple_deep()
        };
        let border_color = if selected {
            accent_lavender()
        } else {
            rgb(18, 31, 39)
        };
        let border_width = if selected { 2.0 } else { 1.0 };
        let accent = if index == 1 {
            accent_purple_hover()
        } else if selected {
            rgb(190, 141, 255)
        } else {
            accent_purple()
        };
        let speaker_color = if previewing {
            rgb(52, 195, 255)
        } else {
            rgb(140, 170, 190)
        };

        let speaker = button(
            container(
                text(Bootstrap::VolumeUpFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .style(text_color(speaker_color)),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .width(Length::Fixed(38.0))
        .height(Length::Fixed(24.0))
        .padding(0)
        .on_press(preview_msg)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => rgb(30, 50, 65),
                _ => Color::TRANSPARENT,
            })),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        });

        let right_col = column![
            speaker,
            Space::with_height(Length::Fill),
            text(duration)
                .size(10)
                .style(text_color(rgb(192, 211, 224))),
        ]
        .align_x(Alignment::Center)
        .padding([4, 0])
        .width(Length::Fixed(46.0))
        .height(Length::Fill);

        let number_button = button(
            container(
                text(index.to_string())
                    .size(18)
                    .style(text_color(Color::WHITE)),
            )
            .width(Length::Fixed(60.0))
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(block_style(accent)),
        )
        .width(Length::Fixed(60.0))
        .height(Length::Fill)
        .padding(0)
        .on_press(play_now_msg)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => accent_purple_hover(),
                _ => accent,
            })),
            border: Border {
                color: rgb(18, 31, 39),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        let inner = row![
            number_button,
            column![
                text(title).size(12).style(text_color(rgb(224, 239, 249))),
                text(meta).size(9).style(text_color(accent_lavender())),
                text(scheduled)
                    .size(10)
                    .style(text_color(rgb(221, 237, 73))),
            ]
            .spacing(2)
            .padding([4, 5])
            .width(Length::Fill),
            right_col,
            Space::with_width(Length::Fixed(8.0)),
        ]
        .height(Length::Fill)
        .align_y(Alignment::Center);

        button(inner)
            .width(Length::Fill)
            .height(Length::Fixed(52.0))
            .padding(0)
            .on_press(on_press)
            .style(move |_, _| button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn queue_entry_title(&self, entry: &crate::db::QueueEntry) -> String {
        match (entry.artist_name.trim(), entry.title.trim()) {
            ("", "") => format!("Queue item {}", entry.id),
            ("", title) => title.to_string(),
            (artist, "") => artist.to_string(),
            (artist, title) => format!("{artist} - {title}"),
        }
    }

    fn queue_entry_meta(&self, entry: &crate::db::QueueEntry) -> String {
        let mut parts = vec![
            format!("Intro {}", fmt_dur(entry.intro)),
            format!("Cue {}/{}", fmt_dur(entry.cue_in), fmt_dur(entry.cue_out)),
        ];

        if entry.fixed_time {
            parts.push(String::from("Fixed"));
        }

        if entry.priority > 0 {
            parts.push(format!("Priority {}", entry.priority));
        }

        if entry.track_id.is_none() {
            parts.push(String::from("No track"));
        }

        parts.join("  •  ")
    }

    pub fn side_mode(
        &self,
        label: &'static str,
        active: bool,
        message: Option<Message>,
    ) -> Element<'_, Message> {
        let bg = if active {
            accent_purple()
        } else {
            rgb(65, 84, 96)
        };
        let inner = container(text(label).size(10).style(text_color(rgb(226, 238, 245))))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(panel_style(bg, rgb(24, 37, 45)));

        if let Some(msg) = message {
            button(inner)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .on_press(msg)
                .style(move |_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed => accent_purple_hover(),
                        _ => bg,
                    })),
                    border: Border {
                        color: rgb(24, 37, 45),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            inner.into()
        }
    }

    pub fn toolbar_cell(
        &self,
        icon: Element<'static, Message>,
        message: Option<Message>,
    ) -> Element<'_, Message> {
        let inner = container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(|_| container::Style {
                text_color: Some(rgb(205, 216, 224)),
                background: Some(Background::Color(rgb(53, 72, 84))),
                border: Border {
                    color: rgb(30, 44, 53),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            });

        if let Some(msg) = message {
            button(inner)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .on_press(msg)
                .style(|_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                        _ => rgb(53, 72, 84),
                    })),
                    border: Border {
                        color: rgb(30, 44, 53),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            inner.into()
        }
    }
}

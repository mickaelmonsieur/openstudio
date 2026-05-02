use super::styles::*;
use crate::{App, Message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Padding};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub fn deck_header(&self, compact: bool) -> Element<'_, Message> {
        let playing = self.ui_playing();
        let elapsed = self.elapsed();
        let elapsed_str = if elapsed.is_zero() {
            String::from("--:--")
        } else {
            fmt_hms(elapsed)
        };
        let remaining = self
            .transport_duration()
            .map(|total| fmt_hms(total.saturating_sub(elapsed)))
            .unwrap_or_else(|| String::from("--:--"));

        let (intro_str, intro_color) = self
            .current_queue_entry
            .as_ref()
            .filter(|e| !e.intro.is_zero() && elapsed < e.intro)
            .map(|e| (fmt_hms(e.intro.saturating_sub(elapsed)), rgb(255, 185, 50)))
            .unwrap_or_else(|| (String::from("--:--"), rgb(124, 147, 163)));

        let (outro_str, outro_color) = self
            .current_queue_entry
            .as_ref()
            .filter(|e| !e.outro.is_zero())
            .and_then(|e| {
                let end = if e.cue_out > std::time::Duration::ZERO && e.cue_out < e.duration {
                    e.cue_out
                } else {
                    e.duration
                };
                let outro_start = end.saturating_sub(e.outro);
                if elapsed >= outro_start && elapsed <= end {
                    Some((fmt_hms(end.saturating_sub(elapsed)), rgb(255, 100, 70)))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| (String::from("--:--"), rgb(124, 147, 163)));

        let play_bg = if playing {
            rgb(30, 124, 202)
        } else {
            rgb(46, 137, 220)
        };
        let play_icon: Element<'static, Message> = if playing {
            text(Bootstrap::PauseFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(16)
                .into()
        } else {
            text(Bootstrap::PlayFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(16)
                .into()
        };

        let mut play_button = button(
            container(play_icon)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0)
        .style(transport_style(
            play_bg,
            rgb(72, 161, 235),
            rgb(112, 188, 255),
        ));
        play_button = play_button.on_press(Message::TogglePlay);

        let mut stop_button = button(
            container(
                text(Bootstrap::StopFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(13),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0)
        .style(transport_style(
            rgb(68, 101, 128),
            rgb(216, 66, 62),
            rgb(105, 140, 164),
        ));
        stop_button = stop_button.on_press(Message::Stop);

        let mut restart_button = button(
            container(
                text(Bootstrap::SkipStartFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0)
        .style(transport_style(
            rgb(61, 94, 121),
            rgb(78, 121, 153),
            rgb(105, 140, 164),
        ));
        restart_button = restart_button.on_press(Message::Restart);

        let rewind_button = self.transport_seek_button(
            text(Bootstrap::RewindFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(15)
                .into(),
            -5000,
        );
        let forward_button = self.transport_seek_button(
            text(Bootstrap::FastForwardFill.to_string())
                .font(BOOTSTRAP_FONT)
                .size(15)
                .into(),
            5000,
        );
        let cover = container(
            text(Bootstrap::ImageAlt.to_string())
                .font(BOOTSTRAP_FONT)
                .size(24)
                .style(text_color(rgb(84, 108, 122))),
        )
        .width(Length::Fixed(58.0))
        .height(Length::Fixed(58.0))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(panel_style(rgb(8, 12, 15), rgb(31, 43, 51)));

        let now_playing = row![
            cover,
            column![
                text(self.track_title())
                    .size(15)
                    .style(text_color(rgb(197, 220, 235))),
                text(self.track_artist())
                    .size(11)
                    .style(text_color(rgb(125, 154, 171))),
                text(self.file_name())
                    .size(11)
                    .style(text_color(accent_lavender_muted())),
            ]
            .spacing(3)
            .width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let timing = row![
            self.time_box("INTRO", intro_str, intro_color),
            self.time_box("OUTRO", outro_str, outro_color),
            self.time_box("ELAPSED", elapsed_str, rgb(80, 220, 120)),
            self.time_box("REMAINING", remaining, rgb(52, 206, 251)),
            self.time_box("HOUR", self.current_hour.clone(), rgb(244, 239, 38)),
        ]
        .spacing(22)
        .align_y(Alignment::Center);

        let controls = row![
            play_button,
            stop_button,
            restart_button,
            rewind_button,
            forward_button
        ]
        .spacing(9)
        .align_y(Alignment::Center);

        let deck: Element<_> = if compact {
            column![
                row![controls, now_playing.width(Length::Fill)]
                    .spacing(12)
                    .align_y(Alignment::Center),
                row![Space::with_width(Length::Fill), timing].align_y(Alignment::Center),
            ]
            .spacing(5)
            .into()
        } else {
            row![
                controls,
                now_playing.width(Length::FillPortion(4)),
                Space::with_width(Length::FillPortion(1)),
                timing,
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .into()
        };

        container(deck)
            .height(Length::Fixed(if compact { 112.0 } else { 68.0 }))
            .padding([5, 8])
            .style(|_| container::Style {
                background: Some(Background::Color(rgb(54, 72, 84))),
                border: Border {
                    color: rgb(22, 32, 39),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    pub fn transport_seek_button(
        &self,
        icon: Element<'static, Message>,
        offset: i64,
    ) -> button::Button<'static, Message> {
        button(
            container(icon)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0)
        .style(transport_style(
            rgb(61, 94, 121),
            rgb(78, 121, 153),
            rgb(105, 140, 164),
        ))
        .on_press(Message::Seek(offset))
    }

    pub fn time_box(
        &self,
        label: &'static str,
        value: impl Into<String>,
        color: Color,
    ) -> Element<'_, Message> {
        container(
            column![
                text(label)
                    .size(9)
                    .style(text_color(rgb(111, 135, 151)))
                    .align_x(Horizontal::Center),
                text(value.into())
                    .size(17)
                    .font(Font::MONOSPACE)
                    .style(text_color(color))
                    .align_x(Horizontal::Center),
            ]
            .spacing(2)
            .align_x(Alignment::Center),
        )
        .width(Length::Shrink)
        .into()
    }

    pub fn progress_strip(&self) -> Element<'_, Message> {
        let elapsed = self.elapsed().as_millis();
        let filled = self
            .transport_duration()
            .map(|total| {
                let total = total.as_millis().max(1);
                ((elapsed * 1000) / total).min(1000) as u16
            })
            .unwrap_or(0);
        let filled = filled.max(1);
        let empty = 1000_u16.saturating_sub(filled).max(1);

        column![
            row![
                container(Space::with_height(Length::Fixed(4.0)))
                    .width(Length::FillPortion(filled))
                    .style(block_style(accent_purple_hover())),
                container(Space::with_height(Length::Fixed(4.0)))
                    .width(Length::FillPortion(empty))
                    .style(block_style(rgb(17, 25, 30))),
            ],
            row![
                self.queue_vu_meter(),
                // text("• 80S").size(10).style(text_color(rgb(245, 105, 57))),
                // text("• DAILY SWEEPERS").size(10).style(text_color(rgb(232, 239, 246))),
                // Space::with_width(Length::Fixed(60.0)),
                // text("✖ NO SWEEPER").size(10).style(text_color(rgb(91, 226, 75))),
                // text("✖ NO VOICETRACK").size(10).style(text_color(rgb(91, 226, 75))),
                Space::with_width(Length::Fill),
                container(
                    text(self.current_date.clone())
                        .size(12)
                        .style(text_color(accent_lavender()))
                )
                .padding(Padding {
                    right: 12.0,
                    ..Padding::ZERO
                }),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .height(Length::Fixed(34.0)),
        ]
        .into()
    }

    pub fn queue_vu_meter(&self) -> Element<'_, Message> {
        let (left, right) = self.queue_meter_levels();

        container(
            column![self.vu_meter_row("L", left), self.vu_meter_row("R", right),]
                .spacing(2)
                .width(Length::Fill),
        )
        .width(Length::Fixed(154.0))
        .height(Length::Fixed(20.0))
        .padding([2, 4])
        .style(panel_style(rgb(12, 19, 23), rgb(29, 43, 52)))
        .into()
    }

    fn vu_meter_row(&self, label: &'static str, level: f32) -> Element<'_, Message> {
        const SEGMENTS: usize = 30;

        let active = ((level.clamp(0.0, 1.0) * SEGMENTS as f32).round() as usize).min(SEGMENTS);
        let mut segments = row![].spacing(1).height(Length::Fixed(6.0));

        for index in 0..SEGMENTS {
            let ratio = (index + 1) as f32 / SEGMENTS as f32;
            let lit = index < active;
            let color = if !lit {
                rgb(24, 34, 39)
            } else if ratio < 0.72 {
                rgb(45, 214, 75)
            } else if ratio < 0.88 {
                rgb(238, 205, 54)
            } else {
                rgb(231, 62, 57)
            };

            segments = segments.push(
                container(Space::with_height(Length::Fixed(6.0)))
                    .width(Length::Fixed(3.0))
                    .height(Length::Fixed(6.0))
                    .style(block_style(color)),
            );
        }

        row![
            text(label)
                .size(8)
                .font(Font::MONOSPACE)
                .style(text_color(rgb(136, 162, 178))),
            segments,
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .height(Length::Fixed(7.0))
        .into()
    }
}

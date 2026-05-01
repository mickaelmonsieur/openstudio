use super::styles::*;
use crate::{audio, db, App, Message, PickerTarget, TrackPickerState, SEARCH_PAGE_SIZE};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{window, Alignment, Background, Border, Color, Element, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub fn track_picker_window(
        &self,
        window_id: window::Id,
        picker: &TrackPickerState,
    ) -> Element<'_, Message> {
        let search_bar = text_input("Search", &picker.search_query)
            .on_input(move |value| Message::PickerSearchChanged(window_id, value))
            .padding(7)
            .size(13)
            .width(Length::Fill);

        let filters = row![
            self.picker_filter_select(
                self.search_categories.clone(),
                picker.search_category.clone(),
                move |value| Message::PickerCategorySelected(window_id, value),
            ),
            self.picker_filter_select(
                self.picker_visible_subcategories(picker),
                picker.search_subcategory.clone(),
                move |value| Message::PickerSubcategorySelected(window_id, value),
            ),
            self.picker_filter_select(
                self.search_genres.clone(),
                picker.search_genre.clone(),
                move |value| Message::PickerGenreSelected(window_id, value),
            ),
        ]
        .spacing(1)
        .height(Length::Fixed(31.0));

        let header = self.picker_result_row(
            window_id,
            picker,
            [
                String::from("Artist"),
                String::from("Title"),
                String::from("Intro"),
                String::from("Duration"),
                String::from("Date Modified"),
                String::from("Date Added"),
            ],
            true,
            0,
            0,
        );

        let visible_tracks: Vec<_> = self
            .search_tracks
            .iter()
            .filter(|track| self.picker_track_matches(picker, track))
            .skip(picker.page_start)
            .take(SEARCH_PAGE_SIZE)
            .collect();
        let total_rows = self.filtered_picker_total(picker);
        let visible_end = (picker.page_start + visible_tracks.len()).min(total_rows);

        let mut rows = column![header].spacing(0);
        for (row_offset, track) in visible_tracks.iter().enumerate() {
            let row_index = picker.page_start + row_offset;
            rows = rows.push(self.picker_result_row(
                window_id,
                picker,
                [
                    track.artist_name.clone(),
                    track.title.clone(),
                    fmt_hms(track.intro),
                    fmt_hms(track.duration),
                    track.updated_at.clone(),
                    track.created_at.clone(),
                ],
                false,
                row_index,
                track.id,
            ));
        }

        let target = match picker.target {
            PickerTarget::Instant(index) => format!("INSTANT {}", index + 1),
            PickerTarget::Aux(index) => format!("AUX {}", index + 1),
        };

        let title = container(text(target).size(12).style(text_color(rgb(226, 238, 245))))
            .height(Length::Fixed(30.0))
            .width(Length::Fill)
            .padding([0, 10])
            .align_y(Vertical::Center)
            .style(panel_style(accent_purple(), accent_purple_border()));

        let table = scrollable(rows).width(Length::Fill).height(Length::Fill);
        let navigation = self.picker_navigation_bar(window_id, picker, visible_end, total_rows);

        container(column![title, search_bar, filters, table, navigation].spacing(1))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(6)
            .style(block_style(rgb(13, 22, 28)))
            .into()
    }

    fn picker_navigation_bar(
        &self,
        window_id: window::Id,
        picker: &TrackPickerState,
        visible_end: usize,
        total_rows: usize,
    ) -> Element<'_, Message> {
        let range = if total_rows == 0 {
            String::from("0-0/0")
        } else {
            format!("{}-{}/{}", picker.page_start + 1, visible_end, total_rows)
        };

        row![
            self.picker_nav_button(
                text(Bootstrap::SkipStartFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::PickerFirstPage(window_id),
                rgb(55, 76, 90),
            ),
            self.picker_nav_button(
                text(Bootstrap::ChevronDoubleLeft.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::PickerPreviousPage(window_id),
                rgb(55, 76, 90),
            ),
            container(text(range).size(11).style(text_color(rgb(221, 230, 237))))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(block_style(rgb(60, 80, 93))),
            self.picker_nav_button(
                text(Bootstrap::ChevronDoubleRight.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::PickerNextPage(window_id),
                rgb(55, 76, 90),
            ),
            self.picker_nav_button(
                text(Bootstrap::SkipEndFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::PickerLastPage(window_id),
                rgb(55, 76, 90),
            ),
            self.picker_nav_button(
                text(Bootstrap::StopFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::Player(audio::PlayerId::Preview, audio::PlayerCommand::Stop),
                rgb(62, 83, 97),
            ),
            self.picker_nav_button(
                text(Bootstrap::PlayFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::PickerPreviewPlay(window_id),
                rgb(62, 83, 97),
            ),
            self.picker_nav_button(
                text(Bootstrap::FastForwardFill.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .into(),
                Message::Player(
                    audio::PlayerId::Preview,
                    audio::PlayerCommand::SeekRelative(5000),
                ),
                rgb(62, 83, 97),
            ),
        ]
        .spacing(1)
        .height(Length::Fixed(40.0))
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .into()
    }

    fn picker_nav_button(
        &self,
        icon: Element<'static, Message>,
        message: Message,
        bg: Color,
    ) -> Element<'_, Message> {
        button(
            container(icon)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
        .width(Length::Fixed(44.0))
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
                radius: 1.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn picker_filter_select<'a>(
        &'a self,
        options: Vec<db::FilterOption>,
        selected: db::FilterOption,
        on_selected: impl Fn(db::FilterOption) -> Message + 'a,
    ) -> Element<'a, Message> {
        pick_list(options, Some(selected), on_selected)
            .width(Length::Fill)
            .padding([0, 8])
            .text_size(12)
            .style(search_pick_list_style)
            .into()
    }

    fn picker_result_row(
        &self,
        window_id: window::Id,
        picker: &TrackPickerState,
        cells: [String; 6],
        header: bool,
        index: usize,
        track_id: i32,
    ) -> Element<'_, Message> {
        let selected = picker.selected_track_id == Some(track_id);
        let bg = if header {
            rgb(24, 132, 211)
        } else if selected {
            rgb(81, 93, 155)
        } else if index.is_multiple_of(2) {
            rgb(31, 47, 56)
        } else {
            rgb(28, 42, 50)
        };

        let [artist, title, intro, duration, modified, added] = cells;
        let content = row![
            self.picker_table_cell(artist, Length::FillPortion(5), header),
            self.picker_table_cell(title, Length::FillPortion(5), header),
            self.picker_table_cell(intro, Length::Fixed(86.0), header),
            self.picker_table_cell(duration, Length::Fixed(94.0), header),
            self.picker_table_cell(modified, Length::Fixed(158.0), header),
            self.picker_table_cell(added, Length::Fixed(158.0), header),
        ]
        .spacing(1)
        .height(Length::Fill);

        if header {
            container(content)
                .height(Length::Fixed(24.0))
                .style(block_style(bg))
                .into()
        } else {
            button(content)
                .width(Length::Fill)
                .height(Length::Fixed(31.0))
                .padding(0)
                .on_press(Message::PickerRowPressed(window_id, track_id))
                .style(move |_, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered | button::Status::Pressed if !selected => {
                            rgb(47, 69, 83)
                        }
                        _ => bg,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        color: if selected {
                            rgb(125, 173, 236)
                        } else {
                            Color::TRANSPARENT
                        },
                        width: if selected { 1.0 } else { 0.0 },
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }
    }

    fn picker_table_cell(
        &self,
        value: String,
        width: Length,
        header: bool,
    ) -> Element<'_, Message> {
        container(
            text(value)
                .size(if header { 10 } else { 12 })
                .wrapping(Wrapping::None)
                .style(text_color(if header {
                    rgb(192, 210, 221)
                } else {
                    Color::WHITE
                })),
        )
        .width(width)
        .height(Length::Fill)
        .padding([0, 6])
        .align_y(Vertical::Center)
        .style(block_style(if header {
            rgb(31, 55, 69)
        } else {
            Color::TRANSPARENT
        }))
        .into()
    }
}

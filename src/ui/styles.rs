use iced::widget::{button, container, pick_list, text};
use iced::{Background, Border, Color, Theme};

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn accent_purple() -> Color {
    rgb(122, 78, 214)
}

pub fn accent_purple_dark() -> Color {
    rgb(58, 36, 83)
}

pub fn accent_purple_deep() -> Color {
    rgb(44, 30, 63)
}

pub fn accent_purple_hover() -> Color {
    rgb(148, 96, 236)
}

pub fn accent_purple_border() -> Color {
    rgb(96, 56, 166)
}

pub fn accent_lavender() -> Color {
    rgb(205, 186, 255)
}

pub fn accent_lavender_muted() -> Color {
    rgb(177, 157, 214)
}

pub fn panel_style(bg: Color, border: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

pub fn block_style(bg: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    }
}

pub fn text_color(color: Color) -> impl Fn(&Theme) -> text::Style {
    move |_| text::Style { color: Some(color) }
}

pub fn search_pick_list_style(_: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: rgb(230, 230, 230),
        placeholder_color: rgb(180, 180, 180),
        handle_color: rgb(180, 185, 190),
        background: Background::Color(rgb(62, 63, 66)),
        border: Border {
            color: match status {
                pick_list::Status::Active => rgb(41, 143, 221),
                pick_list::Status::Hovered | pick_list::Status::Opened => rgb(70, 172, 245),
            },
            width: 1.0,
            radius: 1.0.into(),
        },
    }
}

pub fn transport_style(
    active_bg: Color,
    hover_bg: Color,
    border: Color,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let disabled = matches!(status, button::Status::Disabled);
        button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => hover_bg,
                button::Status::Disabled => rgb(42, 57, 66),
                _ => active_bg,
            })),
            text_color: if disabled {
                rgb(119, 137, 148)
            } else {
                Color::WHITE
            },
            border: Border {
                color: if disabled { rgb(62, 78, 88) } else { border },
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

pub fn fmt_dur(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}", s / 60, s % 60)
}

pub fn fmt_hms(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

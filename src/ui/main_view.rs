use super::styles::{
    accent_purple, block_style, panel_style, rgb, search_pick_list_style, text_color,
};
use crate::app_audio_config::COMPRESSOR_PRESETS;
use crate::{
    app_helpers::{login_input_id, pass_input_id},
    audio, App, ConfigField, DbField, DeviceTarget, Dialog, LoginField, Message, WindowKind,
};
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, responsive, row, stack, text,
    text_input, vertical_slider, Space,
};
use iced::{window, Alignment, Background, Border, Color, Element, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

impl App {
    pub(crate) fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(WindowKind::TrackPicker(picker)) = self.windows.get(&window_id) {
            return self.track_picker_window(window_id, picker);
        }

        let middle: Element<_> = if self.is_locked {
            container(
                column![
                    text(Bootstrap::LockFill.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(48)
                        .style(text_color(rgb(80, 100, 115))),
                    text("Studio verrouillé")
                        .size(16)
                        .style(text_color(rgb(100, 125, 145))),
                    text("Cliquez sur le cadenas pour vous identifier.")
                        .size(12)
                        .style(text_color(rgb(70, 90, 105))),
                ]
                .spacing(12)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(block_style(rgb(7, 11, 13)))
            .into()
        } else {
            column![
                self.progress_strip(),
                responsive(|size| self.main_stage(size.width < 980.0)),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        let content: Element<_> = column![self.deck_header(false), middle, self.footer_bar(),]
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if self.dialog.is_some() {
            stack([content, self.dialog_overlay()]).into()
        } else {
            content
        }
    }

    fn main_stage(&self, compact: bool) -> Element<'_, Message> {
        let content: Element<_> = if compact {
            column![
                self.queue_panel(),
                self.instant_panel(),
                self.aux_players_panel()
            ]
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            row![
                container(self.queue_panel())
                    .width(Length::FillPortion(7))
                    .height(Length::Fill),
                column![
                    container(self.instant_panel())
                        .width(Length::Fill)
                        .height(Length::FillPortion(7)),
                    container(self.aux_players_panel())
                        .width(Length::Fill)
                        .height(Length::FillPortion(3)),
                ]
                .spacing(4)
                .width(Length::FillPortion(12))
                .height(Length::Fill),
            ]
            .spacing(4)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(block_style(rgb(7, 11, 13)))
            .into()
    }

    fn footer_bar(&self) -> Element<'_, Message> {
        let db_status = self.db_status_display();
        let status_color = if self.db.is_some() {
            rgb(221, 230, 237)
        } else {
            rgb(255, 190, 120)
        };
        let db_icon_color = if self.db.is_some() {
            rgb(100, 140, 170)
        } else {
            rgb(255, 160, 80)
        };
        let icon_btn = |icon: Bootstrap, msg: Option<Message>, color: Color| {
            let t = text(icon.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .style(text_color(color));
            let b = button(t).padding([0, 10]).style(|_, _| button::Style {
                background: None,
                ..Default::default()
            });
            if let Some(m) = msg {
                b.on_press(m)
            } else {
                b
            }
        };

        let active_color = rgb(100, 140, 170);
        let inactive_color = rgb(70, 90, 105);

        let authenticated = self.current_user_role >= 1;
        let admin_enabled =
            !self.is_locked && self.db.is_some() && matches!(self.current_user_role, 1 | 2);
        let cfg_btn = icon_btn(
            Bootstrap::GearFill,
            admin_enabled.then_some(Message::ConfigOpen),
            if admin_enabled {
                active_color
            } else {
                inactive_color
            },
        );
        let audio_btn = icon_btn(
            Bootstrap::Sliders,
            admin_enabled.then_some(Message::AudioProcessingOpen),
            if admin_enabled {
                active_color
            } else {
                inactive_color
            },
        );
        let encoder_btn = icon_btn(
            Bootstrap::CloudUploadFill,
            admin_enabled.then_some(Message::StreamEncoderOpen),
            if admin_enabled {
                active_color
            } else {
                inactive_color
            },
        );
        let db_btn = icon_btn(
            Bootstrap::DatabaseFill,
            (!self.is_locked).then_some(Message::DbConfigOpen),
            if self.is_locked {
                inactive_color
            } else {
                db_icon_color
            },
        );
        let about_btn = icon_btn(
            Bootstrap::QuestionCircleFill,
            Some(Message::AboutOpen),
            active_color,
        );
        let lock_btn = icon_btn(
            if self.is_locked {
                Bootstrap::LockFill
            } else {
                Bootstrap::UnlockFill
            },
            Some(Message::LockToggle),
            if self.is_locked {
                rgb(220, 100, 80)
            } else {
                inactive_color
            },
        );
        let user_label = text(format!("User: {}", self.current_user_login))
            .size(11)
            .style(text_color(if authenticated {
                rgb(100, 200, 120)
            } else {
                rgb(100, 125, 145)
            }));
        let auto_mix_color = if self.autodj_enabled {
            rgb(221, 230, 237)
        } else {
            rgb(125, 154, 171)
        };
        let section_label =
            |label: &'static str| text(label).size(12).style(text_color(rgb(160, 180, 195)));

        container(
            row![
                container(
                    row![
                        section_label("DB:"),
                        text(db_status).size(13).style(text_color(status_color))
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .width(Length::FillPortion(5))
                .height(Length::Fill)
                .padding([0, 12])
                .center_y(Length::Fill),
                container(Space::with_width(Length::Fixed(1.0)))
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(block_style(rgb(37, 54, 64))),
                container(
                    row![
                        section_label("AUTO MIX:"),
                        text(self.auto_mix_status.clone())
                            .size(13)
                            .style(text_color(auto_mix_color))
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .width(Length::FillPortion(7))
                .height(Length::Fill)
                .padding([0, 12])
                .center_y(Length::Fill),
                row![
                    user_label,
                    lock_btn,
                    cfg_btn,
                    audio_btn,
                    encoder_btn,
                    db_btn,
                    about_btn
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(block_style(rgb(55, 75, 89)))
        .into()
    }

    fn dialog_overlay(&self) -> Element<'_, Message> {
        let dialog = match &self.dialog {
            Some(Dialog::About) => container(
                column![
                    text("OpenStudio")
                        .size(18)
                        .style(text_color(rgb(226, 238, 245))),
                    text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .size(13)
                        .style(text_color(rgb(180, 200, 212))),
                    text("Professional broadcast radio software")
                        .size(12)
                        .style(text_color(rgb(160, 180, 195))),
                    text("Copyright Mickael Monsieur © 2026")
                        .size(12)
                        .style(text_color(rgb(160, 180, 195))),
                    row![
                        Space::with_width(Length::Fill),
                        self.dialog_button("Close", Message::DialogCancel, accent_purple()),
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(12),
            )
            .width(Length::Fixed(360.0))
            .padding(18)
            .style(panel_style(rgb(31, 46, 55), accent_purple())),
            Some(Dialog::SaveInstantPage { name }) => container(
                column![
                    text("Save Instant Page")
                        .size(14)
                        .style(text_color(rgb(226, 238, 245))),
                    text_input("Page name", name)
                        .on_input(Message::InstantSaveNameChanged)
                        .on_submit(Message::InstantSaveConfirm)
                        .padding(8)
                        .size(13)
                        .width(Length::Fill),
                    row![
                        Space::with_width(Length::Fill),
                        self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                        self.dialog_button("Save", Message::InstantSaveConfirm, accent_purple()),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(12),
            )
            .width(Length::Fixed(420.0))
            .padding(16)
            .style(panel_style(rgb(31, 46, 55), accent_purple())),
            Some(Dialog::EditDbConfig {
                host,
                port,
                database,
                user,
                password,
                psql_path,
                connection_status,
                create_status,
                delete_status,
                delete_confirm,
            }) => {
                let lbl = |s: &'static str| text(s).size(11).style(text_color(rgb(160, 180, 195)));
                let field = |label: &'static str, val: &str, f: DbField| {
                    column![
                        lbl(label),
                        text_input("", val)
                            .on_input(move |v| Message::DbConfigFieldChanged(f.clone(), v))
                            .padding(7)
                            .size(13)
                            .width(Length::Fill),
                    ]
                    .spacing(4)
                };
                let section_label =
                    |s: &'static str| text(s).size(10).style(text_color(rgb(100, 130, 150)));
                let conn_status_el: Element<_> = match connection_status {
                    Some(Ok(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(100, 220, 130)))
                        .into(),
                    Some(Err(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(230, 100, 100)))
                        .into(),
                    None => Space::with_width(Length::Fill).into(),
                };
                let create_status_el: Element<_> = match create_status {
                    Some(Ok(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(100, 220, 130)))
                        .into(),
                    Some(Err(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(230, 100, 100)))
                        .into(),
                    None => Space::with_width(Length::Fill).into(),
                };
                let delete_status_el: Element<_> = match delete_status {
                    Some(Ok(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(100, 220, 130)))
                        .into(),
                    Some(Err(msg)) => text(msg.as_str())
                        .size(11)
                        .style(text_color(rgb(230, 100, 100)))
                        .into(),
                    None => Space::with_width(Length::Fill).into(),
                };
                let delete_confirm_el: Element<_> = if *delete_confirm {
                    container(
                        column![
                            text("DANGER: permanently delete this database?")
                                .size(13)
                                .style(text_color(rgb(255, 210, 210))),
                            text(format!(
                                "This will drop \"{}\" and destroy all OpenStudio data inside it. There is no undo.",
                                database
                            ))
                            .size(11)
                            .style(text_color(rgb(245, 170, 170))),
                            row![
                                Space::with_width(Length::Fill),
                                self.dialog_button(
                                    "Keep Database",
                                    Message::DbConfigCancelDeleteDatabase,
                                    rgb(62, 83, 97)
                                ),
                                self.dialog_button(
                                    "Drop Database",
                                    Message::DbConfigDeleteDatabase,
                                    rgb(180, 35, 35)
                                ),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(10),
                    )
                    .padding(10)
                    .style(panel_style(rgb(50, 18, 18), rgb(190, 45, 45)))
                    .into()
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                };
                container(
                    column![
                        text("Database Settings")
                            .size(14)
                            .style(text_color(rgb(226, 238, 245))),
                        section_label("CONNECTION"),
                        row![
                            field("Host", host, DbField::Host),
                            field("Port", port, DbField::Port).width(Length::Fixed(80.0)),
                        ]
                        .spacing(8),
                        row![
                            field("User", user, DbField::User),
                            column![
                                lbl("Password"),
                                text_input("", password)
                                    .on_input(|v| Message::DbConfigFieldChanged(
                                        DbField::Password,
                                        v
                                    ))
                                    .secure(true)
                                    .padding(7)
                                    .size(13)
                                    .width(Length::Fill),
                            ]
                            .spacing(4),
                        ]
                        .spacing(8),
                        row![
                            self.dialog_button(
                                "Test Connection",
                                Message::DbConfigTestConnection,
                                rgb(30, 80, 110)
                            ),
                            conn_status_el,
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        section_label("DATABASE"),
                        row![
                            field("Database", database, DbField::Database),
                            column![
                                Space::with_height(Length::Fixed(16.0)),
                                self.dialog_button(
                                    "Create Database",
                                    Message::DbConfigCreateDatabase,
                                    rgb(30, 90, 60)
                                ),
                            ]
                            .spacing(4),
                        ]
                        .spacing(8)
                        .align_y(Alignment::End),
                        create_status_el,
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button(
                                "Delete database",
                                Message::DbConfigAskDeleteDatabase,
                                rgb(180, 35, 35)
                            ),
                        ]
                        .align_y(Alignment::Center),
                        delete_status_el,
                        delete_confirm_el,
                        section_label("psql PATH"),
                        field("psql binary", psql_path, DbField::PsqlPath),
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button(
                                "Save & Reconnect",
                                Message::DbConfigSave,
                                accent_purple()
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(12),
                )
                .width(Length::Fixed(500.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            Some(Dialog::EditConfig {
                auto_mix_on_start,
                auto_play_on_start,
                start_locked,
                preload,
                fade_out_duration_ms,
                stop_fade_duration_ms,
                timezone,
                device_deck,
                device_instant,
                device_aux,
                device_preview,
            }) => {
                let general_fieldset_label = container(
                    text("General")
                        .size(11)
                        .style(text_color(rgb(160, 180, 195))),
                )
                .padding([3, 8]);

                let general_fieldset_body = column![row![
                    text("Timezone")
                        .size(13)
                        .style(text_color(rgb(226, 238, 245))),
                    Space::with_width(Length::Fill),
                    pick_list(
                        self.timezone_options.clone(),
                        Some(timezone.clone()),
                        Message::ConfigTimezoneChanged,
                    )
                    .padding(6)
                    .text_size(13)
                    .width(Length::Fixed(220.0))
                    .style(search_pick_list_style),
                ]
                .spacing(6)
                .align_y(Alignment::Center),]
                .spacing(10)
                .padding([10, 12]);

                let automix_fieldset_label = container(
                    text("AUTO MIX")
                        .size(11)
                        .style(text_color(rgb(160, 180, 195))),
                )
                .padding([3, 8]);

                let automix_fieldset_body = column![
                    checkbox("Enable AUTO MIX on startup", *auto_mix_on_start)
                        .on_toggle(|_| Message::ConfigToggle(ConfigField::AutoMixOnStart))
                        .size(14)
                        .text_size(13),
                    checkbox("Enable AUTO PLAY on startup", *auto_play_on_start)
                        .on_toggle(|_| Message::ConfigToggle(ConfigField::AutoPlayOnStart))
                        .size(14)
                        .text_size(13),
                    checkbox("Start Locked", *start_locked)
                        .on_toggle(|_| Message::ConfigToggle(ConfigField::StartLocked))
                        .size(14)
                        .text_size(13),
                    row![
                        text("Preload")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", preload)
                            .on_input(Message::ConfigPreloadChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("s").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    row![
                        text("Fade Out Duration")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", fade_out_duration_ms)
                            .on_input(Message::ConfigFadeOutDurationChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("ms").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    row![
                        text("Stop Fade Out Duration")
                            .size(13)
                            .style(text_color(rgb(226, 238, 245))),
                        Space::with_width(Length::Fill),
                        text_input("", stop_fade_duration_ms)
                            .on_input(Message::ConfigStopFadeDurationChanged)
                            .padding(6)
                            .size(13)
                            .width(Length::Fixed(70.0)),
                        text("ms").size(13).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                ]
                .spacing(10)
                .padding([10, 12]);

                let general_fieldset =
                    container(column![general_fieldset_label, general_fieldset_body].spacing(0))
                        .style(|_| container::Style {
                            border: Border {
                                color: rgb(62, 83, 97),
                                width: 1.0,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        });

                let automix_fieldset =
                    container(column![automix_fieldset_label, automix_fieldset_body].spacing(0))
                        .style(|_| container::Style {
                            border: Border {
                                color: rgb(62, 83, 97),
                                width: 1.0,
                                radius: 3.0.into(),
                            },
                            ..Default::default()
                        });

                let device_options: Vec<String> = std::iter::once(String::from("(Default)"))
                    .chain(self.audio_devices.iter().cloned())
                    .collect();
                let mk_device_row = |label: &'static str,
                                     stored: &str,
                                     target: DeviceTarget,
                                     opts: Vec<String>|
                 -> Element<'_, Message> {
                    let selected: String = if stored.is_empty() {
                        String::from("(Default)")
                    } else {
                        stored.to_string()
                    };
                    row![
                        text(label).size(11).style(text_color(rgb(160, 180, 195))),
                        Space::with_width(Length::Fill),
                        pick_list(opts, Some(selected), move |name: String| {
                            let v = if name == "(Default)" {
                                String::new()
                            } else {
                                name
                            };
                            Message::ConfigDeviceChanged(target, v)
                        })
                        .text_size(12)
                        .padding(6)
                        .width(Length::Fixed(220.0))
                        .style(search_pick_list_style),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
                };
                let audio_fieldset_label = container(
                    text("Audio Devices")
                        .size(11)
                        .style(text_color(rgb(160, 180, 195))),
                )
                .padding([3, 8]);
                let audio_fieldset_body = column![
                    mk_device_row(
                        "Deck",
                        device_deck,
                        DeviceTarget::Deck,
                        device_options.clone()
                    ),
                    mk_device_row(
                        "Instant Player",
                        device_instant,
                        DeviceTarget::Instant,
                        device_options.clone()
                    ),
                    mk_device_row(
                        "Aux Player",
                        device_aux,
                        DeviceTarget::Aux,
                        device_options.clone()
                    ),
                    mk_device_row(
                        "Preview",
                        device_preview,
                        DeviceTarget::Preview,
                        device_options
                    ),
                ]
                .spacing(10)
                .padding([10, 12]);
                let audio_fieldset = container(
                    column![audio_fieldset_label, audio_fieldset_body].spacing(0),
                )
                .style(|_| container::Style {
                    border: Border {
                        color: rgb(62, 83, 97),
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                });

                container(
                    column![
                        text("Settings")
                            .size(14)
                            .style(text_color(rgb(226, 238, 245))),
                        general_fieldset,
                        automix_fieldset,
                        audio_fieldset,
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button("Save", Message::ConfigSave, accent_purple()),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(16),
                )
                .width(Length::Fixed(480.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            Some(Dialog::AudioProcessing {
                processing_bypassed,
                input_volume,
                compressor_mode,
                compressor_preset,
                attack,
                ratio,
                threshold,
                gain,
                release,
                eq_enabled,
                eq_gains,
                agc_preset,
            }) => {
                let label =
                    |s: &'static str| text(s).size(11).style(text_color(rgb(160, 180, 195)));
                let value_label =
                    |s: String| text(s).size(11).style(text_color(rgb(226, 238, 245)));
                let numeric_field = |caption: &'static str,
                                     value: &str,
                                     on_input: fn(String) -> Message|
                 -> Element<'_, Message> {
                    column![
                        label(caption),
                        text_input("", value)
                            .on_input(on_input)
                            .padding(5)
                            .size(12)
                            .width(Length::Fixed(72.0)),
                    ]
                    .spacing(3)
                    .into()
                };

                let input_body: Element<_> = row![
                    label("Volume"),
                    text_input("", &format!("{input_volume:.0}"))
                        .on_input(|value| {
                            value
                                .parse::<f32>()
                                .ok()
                                .map(|v| {
                                    Message::AudioProcessingInputVolumeChanged(v.clamp(0.0, 100.0))
                                })
                                .unwrap_or(Message::NoOp)
                        })
                        .padding(5)
                        .size(12)
                        .width(Length::Fixed(58.0)),
                    text("0=mute, 100=max")
                        .size(11)
                        .style(text_color(rgb(160, 180, 195))),
                    Space::with_width(Length::Fill),
                    vertical_slider(
                        0.0..=100.0,
                        *input_volume,
                        Message::AudioProcessingInputVolumeChanged
                    )
                    .height(Length::Fixed(56.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into();

                let compressor_modes = vec!["By Preset".to_string(), "Custom Values".to_string()];
                let compressor_presets = COMPRESSOR_PRESETS
                    .iter()
                    .map(|preset| preset.name.to_string())
                    .collect::<Vec<_>>();
                let compressor_body: Element<_> = column![
                    row![
                        label("Mode"),
                        pick_list(
                            compressor_modes,
                            Some(compressor_mode.clone()),
                            Message::AudioProcessingModeChanged,
                        )
                        .padding(5)
                        .text_size(12)
                        .width(Length::Fixed(140.0))
                        .style(search_pick_list_style),
                        label("Preset"),
                        pick_list(
                            compressor_presets,
                            Some(compressor_preset.clone()),
                            Message::AudioProcessingPresetChanged,
                        )
                        .padding(5)
                        .text_size(12)
                        .width(Length::Fill)
                        .style(search_pick_list_style),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        numeric_field("Attack", attack, Message::AudioProcessingAttackChanged),
                        numeric_field("Ratio", ratio, Message::AudioProcessingRatioChanged),
                        numeric_field(
                            "Threshold",
                            threshold,
                            Message::AudioProcessingThresholdChanged
                        ),
                        numeric_field("Gain", gain, Message::AudioProcessingGainChanged),
                        numeric_field("Release", release, Message::AudioProcessingReleaseChanged),
                    ]
                    .spacing(10),
                ]
                .spacing(10)
                .into();

                let eq_freqs = [
                    "32", "63", "125", "250", "500", "1k", "2k", "4k", "8k", "16k",
                ];
                let mut eq_slider_row =
                    iced::widget::Row::new().spacing(12).align_y(Alignment::End);
                for (idx, freq) in eq_freqs.iter().enumerate() {
                    let current = eq_gains.get(idx).copied().unwrap_or(0.0);
                    let band = column![
                        value_label(format!("{current:.0}dB")),
                        vertical_slider(-15.0..=15.0, current, move |v| {
                            Message::AudioProcessingEqGainChanged(idx, v)
                        })
                        .height(Length::Fixed(108.0)),
                        label(freq),
                    ]
                    .spacing(4)
                    .align_x(Alignment::Center)
                    .width(Length::Fixed(52.0));
                    eq_slider_row = eq_slider_row.push(band);
                }
                let eq_scale = column![
                    text("+15dB").size(11).style(text_color(rgb(160, 180, 195))),
                    Space::with_height(Length::Fill),
                    text("0dB").size(11).style(text_color(rgb(160, 180, 195))),
                    Space::with_height(Length::Fill),
                    text("-15dB").size(11).style(text_color(rgb(160, 180, 195))),
                ]
                .height(Length::Fixed(108.0));
                let eq_body: Element<_> = column![
                    row![
                        Space::with_width(Length::Fill),
                        checkbox("Enable", *eq_enabled)
                            .on_toggle(Message::AudioProcessingEqEnabledChanged)
                            .size(14)
                            .text_size(12),
                    ]
                    .align_y(Alignment::Center),
                    row![
                        column![
                            Space::with_height(Length::Fixed(19.0)),
                            eq_scale,
                            Space::with_height(Length::Fixed(17.0)),
                        ],
                        eq_slider_row,
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                ]
                .spacing(8)
                .into();

                let agc_presets = audio::AgcPreset::ALL
                    .iter()
                    .map(|preset| preset.as_str().to_string())
                    .collect::<Vec<_>>();
                let agc_body: Element<_> = row![
                    label("Automatic Gain Control (AGC) Preset"),
                    pick_list(
                        agc_presets,
                        Some(agc_preset.clone()),
                        Message::AudioProcessingAgcPresetChanged,
                    )
                    .padding(5)
                    .text_size(12)
                    .width(Length::Fixed(260.0))
                    .style(search_pick_list_style),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into();

                container(
                    column![
                        row![
                            text(Bootstrap::Sliders.to_string())
                                .font(BOOTSTRAP_FONT)
                                .size(16)
                                .style(text_color(rgb(100, 140, 170))),
                            text("Audio Processing")
                                .size(18)
                                .style(text_color(rgb(226, 238, 245))),
                            Space::with_width(Length::Fill),
                            checkbox("Bypass", *processing_bypassed)
                                .on_toggle(Message::AudioProcessingBypassChanged)
                                .size(14)
                                .text_size(12),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        self.audio_processing_fieldset("In Chain Volume", input_body),
                        self.audio_processing_fieldset("Compressor Audio", compressor_body),
                        self.audio_processing_fieldset("10 Band Equalizer", eq_body),
                        self.audio_processing_fieldset("AGC", agc_body),
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button(
                                "Save",
                                Message::AudioProcessingSave,
                                accent_purple()
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(12),
                )
                .width(Length::Fixed(700.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            Some(Dialog::StreamEncoder {
                bitrate,
                sample_rate,
                channels,
                encoder_type,
                server_host,
                server_port,
                password,
                mountpoint,
                reconnect_seconds,
                error,
            }) => {
                let label =
                    |s: &'static str| text(s).size(11).style(text_color(rgb(160, 180, 195)));
                let field_label = |s: &'static str| {
                    container(label(s))
                        .width(Length::Fixed(128.0))
                        .align_x(Alignment::End)
                };
                let disabled_text = |s: String, width: f32| -> Element<'_, Message> {
                    container(
                        row![
                            text(s).size(12).style(text_color(rgb(135, 155, 168))),
                            Space::with_width(Length::Fill),
                            text("▾").size(10).style(text_color(rgb(105, 122, 134))),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(width))
                    .padding([5, 8])
                    .style(|_| container::Style {
                        background: Some(Background::Color(rgb(36, 47, 55))),
                        border: Border {
                            color: rgb(54, 70, 82),
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
                };
                let read_only_field = |s: String, width: f32| -> Element<'_, Message> {
                    container(text(s).size(12).style(text_color(rgb(135, 155, 168))))
                        .width(Length::Fixed(width))
                        .padding([5, 8])
                        .style(|_| container::Style {
                            background: Some(Background::Color(rgb(36, 47, 55))),
                            border: Border {
                                color: rgb(54, 70, 82),
                                width: 1.0,
                                radius: 2.0.into(),
                            },
                            ..Default::default()
                        })
                        .into()
                };
                let input_field = |value: &str,
                                   width: f32,
                                   on_input: fn(String) -> Message|
                 -> Element<'_, Message> {
                    text_input("", value)
                        .on_input(on_input)
                        .padding(5)
                        .size(12)
                        .width(Length::Fixed(width))
                        .into()
                };
                let password_error: Element<_> = if let Some(error) = error {
                    row![
                        Space::with_width(Length::Fixed(138.0)),
                        text(error.clone())
                            .size(11)
                            .style(text_color(rgb(220, 100, 80))),
                    ]
                    .into()
                } else {
                    Space::new(Length::Shrink, Length::Shrink).into()
                };
                let channel_options = vec![String::from("Mono"), String::from("Stéréo")];
                let encoding_body: Element<_> = column![
                    row![
                        field_label("Encoder Type"),
                        disabled_text(encoder_type.clone(), 132.0),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Bitrate"),
                        input_field(bitrate, 64.0, Message::StreamEncoderBitrateChanged),
                        text("kbps").size(11).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Samplerate"),
                        input_field(sample_rate, 82.0, Message::StreamEncoderSampleRateChanged),
                        text("Hz").size(11).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Channels"),
                        pick_list(
                            channel_options,
                            Some(channels.clone()),
                            Message::StreamEncoderChannelsChanged,
                        )
                        .padding(5)
                        .text_size(12)
                        .width(Length::Fixed(132.0))
                        .style(search_pick_list_style),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(9)
                .into();
                let server_body: Element<_> = column![
                    row![
                        field_label("Server Host"),
                        read_only_field(server_host.clone(), 300.0),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Server Port"),
                        read_only_field(server_port.clone(), 82.0),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Mountpoint"),
                        input_field(mountpoint, 190.0, Message::StreamEncoderMountpointChanged),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        field_label("Password"),
                        text_input("", password)
                            .secure(true)
                            .on_input(Message::StreamEncoderPasswordChanged)
                            .padding(5)
                            .size(12)
                            .width(Length::Fixed(190.0)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    password_error,
                    row![
                        field_label("Reconnect"),
                        input_field(
                            reconnect_seconds,
                            64.0,
                            Message::StreamEncoderReconnectChanged
                        ),
                        text("sec").size(11).style(text_color(rgb(160, 180, 195))),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(9)
                .into();

                container(
                    column![
                        row![
                            text(Bootstrap::CloudUploadFill.to_string())
                                .font(BOOTSTRAP_FONT)
                                .size(16)
                                .style(text_color(rgb(100, 140, 170))),
                            text("Streaming Encoder")
                                .size(18)
                                .style(text_color(rgb(226, 238, 245))),
                            Space::with_width(Length::Fill),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        self.audio_processing_fieldset("Encoding", encoding_body),
                        self.audio_processing_fieldset("Server", server_body),
                        row![
                            Space::with_width(Length::Fill),
                            self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                            self.dialog_button("OK", Message::StreamEncoderSave, accent_purple()),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(14),
                )
                .width(Length::Fixed(620.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), accent_purple()))
            }
            Some(Dialog::Login {
                login,
                password,
                error,
                focus_index,
            }) => {
                let lbl = |s: &'static str| text(s).size(11).style(text_color(rgb(160, 180, 195)));
                let error_row: Element<_> = if let Some(msg) = error {
                    text(msg.clone())
                        .size(12)
                        .style(text_color(rgb(220, 100, 80)))
                        .into()
                } else {
                    Space::new(Length::Shrink, Length::Shrink).into()
                };
                let unlock_focused = *focus_index == 2;
                let cancel_focused = *focus_index == 3;
                let btn_unlock = button(text("Déverrouiller").size(12))
                    .padding([7, 14])
                    .on_press(Message::LoginSubmit)
                    .style(move |_, status: button::Status| button::Style {
                        background: Some(Background::Color(match status {
                            button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                            _ => accent_purple(),
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            color: if unlock_focused {
                                Color::WHITE
                            } else {
                                rgb(29, 43, 52)
                            },
                            width: if unlock_focused { 1.5 } else { 1.0 },
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    });
                let btn_cancel = button(text("Annuler").size(12))
                    .padding([7, 14])
                    .on_press(Message::DialogCancel)
                    .style(move |_, status: button::Status| button::Style {
                        background: Some(Background::Color(match status {
                            button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                            _ => rgb(62, 83, 97),
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            color: if cancel_focused {
                                Color::WHITE
                            } else {
                                rgb(29, 43, 52)
                            },
                            width: if cancel_focused { 1.5 } else { 1.0 },
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    });
                container(
                    column![
                        row![
                            text(Bootstrap::LockFill.to_string())
                                .font(BOOTSTRAP_FONT)
                                .size(16)
                                .style(text_color(rgb(160, 180, 195))),
                            text("Déverrouillage requis")
                                .size(14)
                                .style(text_color(rgb(226, 238, 245))),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        column![
                            lbl("Login"),
                            text_input("", login)
                                .id(login_input_id())
                                .on_input(|v| Message::LoginFieldChanged(LoginField::Login, v))
                                .padding(7)
                                .size(13)
                                .width(Length::Fill),
                        ]
                        .spacing(4),
                        column![
                            lbl("Mot de passe"),
                            text_input("", password)
                                .id(pass_input_id())
                                .on_input(|v| Message::LoginFieldChanged(LoginField::Password, v))
                                .secure(true)
                                .padding(7)
                                .size(13)
                                .width(Length::Fill),
                        ]
                        .spacing(4),
                        error_row,
                        row![Space::with_width(Length::Fill), btn_cancel, btn_unlock,]
                            .spacing(8)
                            .align_y(Alignment::Center),
                    ]
                    .spacing(12),
                )
                .width(Length::Fixed(360.0))
                .padding(16)
                .style(panel_style(rgb(31, 46, 55), rgb(220, 100, 80)))
            }
            Some(Dialog::ConfirmClose { .. }) => container(
                column![
                    text("Close OpenStudio?")
                        .size(14)
                        .style(text_color(rgb(226, 238, 245))),
                    text("This will cut the broadcast.")
                        .size(12)
                        .style(text_color(rgb(220, 180, 100))),
                    row![
                        Space::with_width(Length::Fill),
                        self.dialog_button("Cancel", Message::DialogCancel, rgb(62, 83, 97)),
                        self.dialog_button("Quit", Message::ConfirmQuit, rgb(180, 55, 45)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(16),
            )
            .width(Length::Fixed(340.0))
            .padding(20)
            .style(panel_style(rgb(31, 46, 55), rgb(180, 55, 45))),
            None => container(Space::new(Length::Shrink, Length::Shrink)),
        };

        if self.dialog.is_none() {
            return Space::new(Length::Shrink, Length::Shrink).into();
        }

        mouse_area(
            container(dialog)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
                    ..Default::default()
                }),
        )
        .on_press(Message::NoOp)
        .into()
    }

    fn audio_processing_fieldset<'a>(
        &self,
        title: &'static str,
        body: Element<'a, Message>,
    ) -> Element<'a, Message> {
        container(
            column![
                container(text(title).size(11).style(text_color(rgb(160, 180, 195))))
                    .padding([3, 8]),
                container(body).padding([10, 12]),
            ]
            .spacing(0),
        )
        .style(|_| container::Style {
            border: Border {
                color: rgb(62, 83, 97),
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn dialog_button(
        &self,
        label: &'static str,
        message: Message,
        bg: Color,
    ) -> Element<'_, Message> {
        button(text(label).size(12))
            .padding([7, 14])
            .on_press(message)
            .style(move |_, status| button::Style {
                background: Some(Background::Color(match status {
                    button::Status::Hovered | button::Status::Pressed => rgb(73, 98, 115),
                    _ => bg,
                })),
                text_color: Color::WHITE,
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

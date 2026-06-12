use crate::{
    app_constants::{INSTANT_PLAYER_ID, PREVIEW_PLAYER_ID, QUEUE_PLAYER_IDS},
    app_helpers::{duration_ms, loaded_track_status, queue_track_status},
    audio, rest, App,
};

impl App {
    pub(crate) fn handle_rest_command(&mut self, command: rest::RestCommand) {
        use rest::RestCommand;

        let reply = match command {
            RestCommand::Status(reply) => {
                let _ = reply.send(rest::RestReply::ok("Status", self.rest_status()));
                return;
            }
            RestCommand::SetAutomix(enabled, reply) => {
                self.set_auto_dj_enabled(enabled);
                (
                    reply,
                    rest::RestReply::ok("Automix updated", self.rest_status()),
                )
            }
            RestCommand::DeckPlay(reply) => {
                if self.any_queue_active() {
                    for player_id in QUEUE_PLAYER_IDS {
                        if self.audio.player(player_id).is_active() {
                            self.audio.handle(player_id, audio::PlayerCommand::Resume);
                        }
                    }
                } else {
                    self.load_next_from_queue(self.current_queue_player_id);
                }
                (reply, rest::RestReply::ok("Deck play", self.rest_status()))
            }
            RestCommand::DeckPause(reply) => {
                for player_id in QUEUE_PLAYER_IDS {
                    if self.audio.player(player_id).is_active() {
                        self.audio.handle(player_id, audio::PlayerCommand::Pause);
                    }
                }
                (reply, rest::RestReply::ok("Deck pause", self.rest_status()))
            }
            RestCommand::DeckPlayPause(reply) => {
                if self.any_queue_active() {
                    for player_id in QUEUE_PLAYER_IDS {
                        if self.audio.player(player_id).is_active() {
                            self.audio
                                .handle(player_id, audio::PlayerCommand::TogglePause);
                        }
                    }
                } else {
                    self.load_next_from_queue(self.current_queue_player_id);
                }
                (
                    reply,
                    rest::RestReply::ok("Deck play/pause", self.rest_status()),
                )
            }
            RestCommand::DeckStop(reply) => {
                let duration = std::time::Duration::from_millis(
                    self.app_config.stop_fade_duration_ms.max(0) as u64,
                );
                let had_active = self.any_queue_active();
                for player_id in QUEUE_PLAYER_IDS {
                    if self.audio.player(player_id).is_active() {
                        self.audio
                            .handle(player_id, audio::PlayerCommand::SoftStop(duration));
                    }
                }
                if had_active {
                    self.deck_soft_stopping = true;
                }
                self.preloaded_queue_entry = None;
                (reply, rest::RestReply::ok("Deck stop", self.rest_status()))
            }
            RestCommand::DeckRestart(reply) => {
                if self.audio.player(self.current_queue_player_id).is_active() {
                    self.audio
                        .handle(self.current_queue_player_id, audio::PlayerCommand::Restart);
                }
                (
                    reply,
                    rest::RestReply::ok("Deck restart", self.rest_status()),
                )
            }
            RestCommand::DeckSeek(offset_ms, reply) => {
                self.audio.handle(
                    self.current_queue_player_id,
                    audio::PlayerCommand::SeekRelative(offset_ms),
                );
                (reply, rest::RestReply::ok("Deck seek", self.rest_status()))
            }
            RestCommand::DeckQueuePlay(one_based_id, reply) => {
                let index = one_based_id - 1;
                if index >= self.queue_entries.len() {
                    (reply, rest::RestReply::error("Queue item does not exist."))
                } else {
                    self.play_queue_entry_now(index);
                    (
                        reply,
                        rest::RestReply::ok("Queue item started", self.rest_status()),
                    )
                }
            }
            RestCommand::DeckPreviewPlay(one_based_id, reply) => {
                if self.play_queue_preview_at_position(one_based_id, false) {
                    (
                        reply,
                        rest::RestReply::ok("Deck preview started", self.rest_status()),
                    )
                } else {
                    (
                        reply,
                        rest::RestReply::error("Queue preview item cannot be played."),
                    )
                }
            }
            RestCommand::DeckPreviewToggle(one_based_id, reply) => {
                if self.play_queue_preview_at_position(one_based_id, true) {
                    (
                        reply,
                        rest::RestReply::ok("Deck preview toggled", self.rest_status()),
                    )
                } else {
                    (
                        reply,
                        rest::RestReply::error("Queue preview item cannot be toggled."),
                    )
                }
            }
            RestCommand::DeckPreviewStop(reply) => {
                self.stop_preview();
                self.previewing_queue_id = None;
                (
                    reply,
                    rest::RestReply::ok("Deck preview stopped", self.rest_status()),
                )
            }
            RestCommand::DeckPreviewSeek(offset_ms, reply) => {
                self.audio.handle(
                    PREVIEW_PLAYER_ID,
                    audio::PlayerCommand::SeekRelative(offset_ms),
                );
                (
                    reply,
                    rest::RestReply::ok("Deck preview seek", self.rest_status()),
                )
            }
            RestCommand::InstantPlay(one_based_id, reply) => {
                let index = one_based_id - 1;
                if index >= self.instant_slots.len() {
                    (
                        reply,
                        rest::RestReply::error("Instant slot does not exist."),
                    )
                } else if self.instant_slots[index].is_none() {
                    (reply, rest::RestReply::error("Instant slot is empty."))
                } else {
                    self.play_instant_slot(index);
                    (
                        reply,
                        rest::RestReply::ok("Instant slot started", self.rest_status()),
                    )
                }
            }
            RestCommand::InstantStop(one_based_id, reply) => {
                let index = one_based_id - 1;
                if index >= self.instant_slots.len() {
                    (
                        reply,
                        rest::RestReply::error("Instant slot does not exist."),
                    )
                } else {
                    self.stop_instant_slot(index);
                    (
                        reply,
                        rest::RestReply::ok("Instant slot stopped", self.rest_status()),
                    )
                }
            }
            RestCommand::InstantSetLoop(one_based_id, enabled, reply) => {
                let index = one_based_id - 1;
                if index >= self.instant_slots.len() {
                    (
                        reply,
                        rest::RestReply::error("Instant slot does not exist."),
                    )
                } else {
                    self.set_instant_loop(index, enabled);
                    (
                        reply,
                        rest::RestReply::ok("Instant loop updated", self.rest_status()),
                    )
                }
            }
            RestCommand::AuxPlay(one_based_id, reply) => {
                let index = one_based_id - 1;
                if index >= self.aux_slots.len() {
                    (reply, rest::RestReply::error("Aux player does not exist."))
                } else if self.aux_slots[index].is_none() {
                    (reply, rest::RestReply::error("Aux player is empty."))
                } else {
                    self.play_aux_slot(index);
                    (
                        reply,
                        rest::RestReply::ok("Aux player started", self.rest_status()),
                    )
                }
            }
            RestCommand::AuxStop(one_based_id, reply) => {
                let index = one_based_id - 1;
                if index >= self.aux_slots.len() {
                    (reply, rest::RestReply::error("Aux player does not exist."))
                } else {
                    self.stop_aux_slot(index);
                    (
                        reply,
                        rest::RestReply::ok("Aux player stopped", self.rest_status()),
                    )
                }
            }
            RestCommand::AuxSetLoop(one_based_id, enabled, reply) => {
                let index = one_based_id - 1;
                if index >= self.aux_loops.len() {
                    (reply, rest::RestReply::error("Aux player does not exist."))
                } else {
                    self.set_aux_loop(index, enabled);
                    (
                        reply,
                        rest::RestReply::ok("Aux loop updated", self.rest_status()),
                    )
                }
            }
        };

        let _ = reply.0.send(reply.1);
    }

    fn rest_status(&self) -> rest::RestStatus {
        let deck_snapshot = self.audio.player(self.current_queue_player_id).snapshot();
        let preview_snapshot = self.audio.player(PREVIEW_PLAYER_ID).snapshot();
        let instant_snapshot = self.audio.player(INSTANT_PLAYER_ID).snapshot();

        rest::RestStatus {
            automix: rest::AutomixStatus {
                enabled: self.autodj_enabled,
                label: self.auto_mix_status.clone(),
            },
            deck: rest::DeckStatus {
                active: self.any_queue_active(),
                playing: self.ui_playing(),
                current_player: Self::player_id_label(self.current_queue_player_id).to_string(),
                position_ms: duration_ms(deck_snapshot.position),
                duration_ms: deck_snapshot.duration.map(duration_ms),
                current: self.current_queue_entry.as_ref().map(queue_track_status),
            },
            preview: rest::PreviewStatus {
                active: self.audio.player(PREVIEW_PLAYER_ID).is_active(),
                playing: self.audio.player(PREVIEW_PLAYER_ID).is_playing(),
                queue_id: self.previewing_queue_id,
                position_ms: duration_ms(preview_snapshot.position),
                duration_ms: preview_snapshot.duration.map(duration_ms),
            },
            instant: rest::InstantStatus {
                active_slot: self.active_instant_slot.map(|index| index + 1),
                active: self.audio.player(INSTANT_PLAYER_ID).is_active(),
                playing: self.audio.player(INSTANT_PLAYER_ID).is_playing(),
                slots: self.instant_rest_slots(&instant_snapshot),
            },
            aux: self.aux_rest_slots(),
            queue: self
                .queue_entries
                .iter()
                .enumerate()
                .map(|(index, entry)| rest::QueueItemStatus {
                    id: index + 1,
                    queue_id: entry.id,
                    track: Some(queue_track_status(entry)),
                })
                .collect(),
        }
    }

    fn instant_rest_slots(
        &self,
        instant_snapshot: &audio::PlayerSnapshot,
    ) -> Vec<rest::SlotPlayerStatus> {
        self.instant_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                let active = self.active_instant_slot == Some(index)
                    && self.audio.player(INSTANT_PLAYER_ID).is_active();
                rest::SlotPlayerStatus {
                    id: index + 1,
                    loaded: slot.is_some(),
                    active,
                    playing: active && self.audio.player(INSTANT_PLAYER_ID).is_playing(),
                    loop_enabled: self.instant_loops.get(index).copied().unwrap_or(false),
                    position_ms: if active {
                        duration_ms(instant_snapshot.position)
                    } else {
                        0
                    },
                    duration_ms: slot.as_ref().map(|track| duration_ms(track.duration)),
                    track: slot.as_ref().map(loaded_track_status),
                }
            })
            .collect()
    }

    fn aux_rest_slots(&self) -> Vec<rest::SlotPlayerStatus> {
        self.aux_slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                let player_id = Self::aux_player_id(index);
                let snapshot = player_id.map(|id| self.audio.player(id).snapshot());
                let active = player_id.is_some_and(|id| self.audio.player(id).is_active());
                let playing = player_id.is_some_and(|id| self.audio.player(id).is_playing());
                rest::SlotPlayerStatus {
                    id: index + 1,
                    loaded: slot.is_some(),
                    active,
                    playing,
                    loop_enabled: self.aux_loops.get(index).copied().unwrap_or(false),
                    position_ms: snapshot
                        .as_ref()
                        .map(|snapshot| duration_ms(snapshot.position))
                        .unwrap_or(0),
                    duration_ms: slot.as_ref().map(|track| duration_ms(track.duration)),
                    track: slot.as_ref().map(loaded_track_status),
                }
            })
            .collect()
    }

    fn player_id_label(player_id: audio::PlayerId) -> &'static str {
        match player_id {
            audio::PlayerId::QueueA => "queue_a",
            audio::PlayerId::QueueB => "queue_b",
            audio::PlayerId::Instant => "instant",
            audio::PlayerId::Aux1 => "aux_1",
            audio::PlayerId::Aux2 => "aux_2",
            audio::PlayerId::Aux3 => "aux_3",
            audio::PlayerId::Preview => "preview",
        }
    }
}

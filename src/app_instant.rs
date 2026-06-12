use crate::{
    app_constants::INSTANT_PLAYER_ID, audio, App, Dialog, InstantPage, LoadedTrack, PickerTarget,
};

impl App {
    pub(crate) fn load_instant_pages_from_db(&mut self) {
        let Some(db) = self.db.clone() else {
            return;
        };

        match db.instant_pages() {
            Ok(pages) if pages.is_empty() => {
                self.instant_pages = vec![InstantPage::default()];
                self.active_instant_page = 0;
                self.instant_slots = vec![None; 10];
                self.instant_loops = vec![false; 10];
            }
            Ok(pages) => {
                self.instant_pages = pages
                    .into_iter()
                    .map(|page| InstantPage {
                        id: Some(page.id),
                        name: page.name,
                    })
                    .collect();
                self.active_instant_page =
                    self.active_instant_page.min(self.instant_pages.len() - 1);
                self.load_active_instant_slots();
            }
            Err(error) => self.status = format!("Instant pages unavailable: {error}"),
        }
    }

    pub(crate) fn load_active_instant_slots(&mut self) {
        self.stop_instant();
        self.instant_slots = vec![None; 10];
        self.instant_loops = vec![false; 10];

        let Some(db) = self.db.clone() else {
            return;
        };
        let Some(page_id) = self.active_instant_page_id() else {
            return;
        };

        match db.instant_slots(page_id) {
            Ok(slots) => {
                for slot in slots {
                    if slot.slot_index < self.instant_slots.len() {
                        self.instant_slots[slot.slot_index] = self.loaded_track(slot.track_id);
                    }
                }
            }
            Err(error) => self.status = format!("Instant slots unavailable: {error}"),
        }
    }

    pub(crate) fn active_instant_page_id(&self) -> Option<i32> {
        self.instant_pages
            .get(self.active_instant_page)
            .and_then(|page| page.id)
    }

    pub(crate) fn active_instant_page_name(&self) -> String {
        self.instant_pages
            .get(self.active_instant_page)
            .map(|page| page.name.clone())
            .unwrap_or_else(|| String::from("Default"))
    }

    pub(crate) fn open_save_instant_dialog(&mut self) {
        let current = self.active_instant_page_name();
        let name = if current == "Default" {
            String::new()
        } else {
            current
        };
        self.dialog = Some(Dialog::SaveInstantPage { name });
    }

    pub(crate) fn new_instant_page(&mut self) {
        self.stop_instant();
        self.instant_pages.push(InstantPage::default());
        self.active_instant_page = self.instant_pages.len() - 1;
        self.instant_slots = vec![None; 10];
        self.instant_loops = vec![false; 10];
        self.dialog = Some(Dialog::SaveInstantPage {
            name: String::new(),
        });
        self.status = String::from("New instant page");
    }

    pub(crate) fn save_instant_page(&mut self) {
        let Some(db) = self.db.clone() else {
            self.status = String::from("Disconnected (instant page not saved)");
            return;
        };

        let name = match &self.dialog {
            Some(Dialog::SaveInstantPage { name }) => {
                let trimmed = name.trim();
                if trimmed.is_empty() {
                    "Default".to_string()
                } else {
                    trimmed.chars().take(64).collect()
                }
            }
            None
            | Some(Dialog::About)
            | Some(Dialog::EditDbConfig { .. })
            | Some(Dialog::EditConfig { .. })
            | Some(Dialog::AudioProcessing { .. })
            | Some(Dialog::StreamEncoder { .. })
            | Some(Dialog::Login { .. })
            | Some(Dialog::ConfirmClose { .. }) => self.active_instant_page_name(),
        };

        let page_id = match self.active_instant_page_id() {
            Some(page_id) => {
                if let Err(error) = db.update_instant_page_name(page_id, &name) {
                    self.status = format!("Instant page save failed: {error}");
                    return;
                }
                page_id
            }
            None => match db.insert_instant_page(&name) {
                Ok(page_id) => page_id,
                Err(error) => {
                    self.status = format!("Instant page creation failed: {error}");
                    return;
                }
            },
        };

        if let Err(error) = db.clear_instant_slots(page_id) {
            self.status = format!("Instant slots clear failed: {error}");
            return;
        }

        for (slot_index, slot) in self.instant_slots.iter().enumerate() {
            if let Some(track) = slot {
                if let Err(error) = db.insert_instant_slot(page_id, slot_index, track.id) {
                    self.status = format!("Instant slot save failed: {error}");
                    return;
                }
            }
        }

        if let Some(page) = self.instant_pages.get_mut(self.active_instant_page) {
            page.id = Some(page_id);
            page.name = name.clone();
        } else {
            self.instant_pages.push(InstantPage {
                id: Some(page_id),
                name: name.clone(),
            });
            self.active_instant_page = self.instant_pages.len() - 1;
        }

        self.dialog = None;
        self.status = format!("Instant page saved: {name}");
    }

    pub(crate) fn delete_active_instant_page(&mut self) {
        let Some(page_id) = self.active_instant_page_id() else {
            self.instant_slots = vec![None; 10];
            self.instant_loops = vec![false; 10];
            self.status = String::from("Instant page is empty");
            return;
        };
        let Some(db) = self.db.clone() else {
            self.status = String::from("Disconnected (instant page not deleted)");
            return;
        };

        if let Err(error) = db.delete_instant_page(page_id) {
            self.status = format!("Instant page delete failed: {error}");
            return;
        }

        self.status = format!("Instant page deleted: {}", self.active_instant_page_name());
        self.load_instant_pages_from_db();
        if self.instant_pages.is_empty() {
            self.instant_pages = vec![InstantPage::default()];
            self.active_instant_page = 0;
            self.instant_slots = vec![None; 10];
            self.instant_loops = vec![false; 10];
        }
    }

    pub(crate) fn show_previous_instant_page(&mut self) {
        if self.instant_pages.is_empty() {
            return;
        }
        self.active_instant_page = if self.active_instant_page == 0 {
            self.instant_pages.len() - 1
        } else {
            self.active_instant_page - 1
        };
        self.load_active_instant_slots();
    }

    pub(crate) fn show_next_instant_page(&mut self) {
        if self.instant_pages.is_empty() {
            return;
        }
        self.active_instant_page = (self.active_instant_page + 1) % self.instant_pages.len();
        self.load_active_instant_slots();
    }

    pub(crate) fn assign_loaded_track(&mut self, target: PickerTarget, track: LoadedTrack) {
        match target {
            PickerTarget::Instant(index) => {
                if let Some(slot) = self.instant_slots.get_mut(index) {
                    *slot = Some(track);
                }
            }
            PickerTarget::Aux(index) => {
                if let Some(slot) = self.aux_slots.get_mut(index) {
                    *slot = Some(track);
                }
            }
        }
    }

    pub(crate) fn play_instant_slot(&mut self, index: usize) {
        let Some(track) = self.instant_slots.get(index).and_then(Option::as_ref) else {
            return;
        };
        let path = track.path.clone();
        let cue_in = track.cue_in;

        self.audio.handle(
            INSTANT_PLAYER_ID,
            audio::PlayerCommand::Load { path, cue_in },
        );
        self.audio
            .handle(INSTANT_PLAYER_ID, audio::PlayerCommand::Play);
        self.active_instant_slot = Some(index);
    }

    pub(crate) fn stop_instant(&mut self) {
        self.audio
            .handle(INSTANT_PLAYER_ID, audio::PlayerCommand::Stop);
        self.active_instant_slot = None;
    }

    pub(crate) fn stop_instant_slot(&mut self, index: usize) {
        if self.active_instant_slot == Some(index) {
            self.stop_instant();
        }
    }

    pub(crate) fn set_instant_loop(&mut self, index: usize, enabled: bool) {
        if let Some(looping) = self.instant_loops.get_mut(index) {
            *looping = enabled;
        }
    }

    pub(crate) fn sync_instant_active_slot(&mut self) {
        if let Some(index) = self.active_instant_slot {
            if !self.audio.player(INSTANT_PLAYER_ID).is_active() {
                if self.instant_loops.get(index).copied().unwrap_or(false) {
                    self.play_instant_slot(index);
                } else {
                    self.active_instant_slot = None;
                }
            }
        }
    }

    pub(crate) fn instant_duration_display(
        &self,
        slot_index: usize,
        total: std::time::Duration,
    ) -> std::time::Duration {
        if self.active_instant_slot == Some(slot_index) {
            let elapsed = self
                .audio
                .player(INSTANT_PLAYER_ID)
                .snapshot()
                .position
                .min(total);
            total.saturating_sub(elapsed)
        } else {
            total
        }
    }
}

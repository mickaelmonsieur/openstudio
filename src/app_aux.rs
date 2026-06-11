use crate::{audio, App, AUX_PLAYER_IDS};

impl App {
    pub(crate) fn aux_player_id(index: usize) -> Option<audio::PlayerId> {
        AUX_PLAYER_IDS.get(index).copied()
    }

    pub(crate) fn aux_active_flags(&self) -> [bool; 3] {
        AUX_PLAYER_IDS.map(|id| self.audio.player(id).is_active())
    }

    pub(crate) fn aux_is_active(&self, index: usize) -> bool {
        Self::aux_player_id(index).is_some_and(|id| self.audio.player(id).is_active())
    }

    pub(crate) fn aux_progress_parts(&self, index: usize) -> (u16, u16) {
        let Some(player_id) = Self::aux_player_id(index) else {
            return (1, 1000);
        };
        let snapshot = self.audio.player(player_id).snapshot();
        let filled = snapshot
            .duration
            .map(|duration| {
                let total = duration.as_millis().max(1);
                ((snapshot.position.as_millis() * 1000) / total).min(1000) as u16
            })
            .unwrap_or(0)
            .max(1);
        let empty = 1000_u16.saturating_sub(filled).max(1);
        (filled, empty)
    }

    pub(crate) fn aux_timing(
        &self,
        index: usize,
    ) -> (
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
    ) {
        let total = self
            .aux_slots
            .get(index)
            .and_then(Option::as_ref)
            .map(|track| track.duration)
            .unwrap_or_default();
        let elapsed = Self::aux_player_id(index)
            .map(|player_id| self.audio.player(player_id).snapshot().position)
            .unwrap_or_default()
            .min(total);
        let remaining = total.saturating_sub(elapsed);

        (elapsed, remaining, total)
    }

    pub(crate) fn play_aux_slot(&mut self, index: usize) {
        let Some(player_id) = Self::aux_player_id(index) else {
            return;
        };
        let Some(track) = self.aux_slots.get(index).and_then(Option::as_ref) else {
            return;
        };
        let path = track.path.clone();
        let cue_in = track.cue_in;

        self.audio
            .handle(player_id, audio::PlayerCommand::Load { path, cue_in });
        self.audio.handle(player_id, audio::PlayerCommand::Play);
    }

    pub(crate) fn stop_aux_slot(&mut self, index: usize) {
        if let Some(player_id) = Self::aux_player_id(index) {
            self.audio.handle(player_id, audio::PlayerCommand::Stop);
        }
    }

    pub(crate) fn toggle_aux_loop(&mut self, index: usize) {
        if let Some(looping) = self.aux_loops.get_mut(index) {
            *looping = !*looping;
        }
    }

    pub(crate) fn set_aux_loop(&mut self, index: usize, enabled: bool) {
        if let Some(looping) = self.aux_loops.get_mut(index) {
            *looping = enabled;
        }
    }

    pub(crate) fn sync_aux_loops(&mut self, was_active: [bool; 3]) {
        for (index, was_active) in was_active.into_iter().enumerate() {
            if !was_active || self.aux_is_active(index) {
                continue;
            }
            if self.aux_loops.get(index).copied().unwrap_or(false) {
                self.play_aux_slot(index);
            }
        }
    }
}

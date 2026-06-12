use crate::audio;

pub(crate) const ANY_CATEGORY: &str = "Any Category";
pub(crate) const ANY_SUBCATEGORY: &str = "Any Subcategory";
pub(crate) const ANY_GENRE: &str = "Any Genre";
pub(crate) const SEARCH_PAGE_SIZE: usize = 50;

pub(crate) const QUEUE_PLAYER_IDS: [audio::PlayerId; 2] =
    [audio::PlayerId::QueueA, audio::PlayerId::QueueB];
pub(crate) const PREVIEW_PLAYER_ID: audio::PlayerId = audio::PlayerId::Preview;
pub(crate) const INSTANT_PLAYER_ID: audio::PlayerId = audio::PlayerId::Instant;
pub(crate) const AUX_PLAYER_IDS: [audio::PlayerId; 3] = [
    audio::PlayerId::Aux1,
    audio::PlayerId::Aux2,
    audio::PlayerId::Aux3,
];

pub(crate) const METER_TICK_MS: u64 = 100;
pub(crate) const METER_DECAY_PER_SECOND: f32 = 0.32;

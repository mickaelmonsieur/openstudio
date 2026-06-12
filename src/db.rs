use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use postgres::{Client, Config, NoTls, Row};
use serde::{Deserialize, Serialize};

pub type SharedDatabase = Arc<Database>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_mix_on_start: bool,
    pub auto_play_on_start: bool,
    pub preload: i32,
    pub fade_out_duration_ms: i32,
    pub stop_fade_duration_ms: i32,
    pub timezone: String,
    pub device_deck: String,
    pub device_instant: String,
    pub device_aux: String,
    pub device_preview: String,
    pub start_locked: bool,
    pub audio_processing_bypassed: bool,
    pub audio_master_volume_percent: f32,
    pub audio_eq_enabled: bool,
    pub audio_eq_gains: Vec<f32>,
    pub audio_compressor_mode: String,
    pub audio_compressor_preset: String,
    pub audio_compressor_attack_ms: f32,
    pub audio_compressor_ratio: f32,
    pub audio_compressor_threshold_db: f32,
    pub audio_compressor_gain_db: f32,
    pub audio_compressor_release_ms: f32,
    pub audio_agc_preset: String,
    pub encoder_bitrate: i32,
    pub encoder_sample_rate: i32,
    pub encoder_channels: i32,
    pub encoder_type: String,
    pub encoder_server_host: String,
    pub encoder_server_port: i32,
    pub encoder_password: String,
    pub encoder_mountpoint: String,
    pub encoder_reconnect_seconds: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_mix_on_start: false,
            auto_play_on_start: false,
            preload: 10,
            fade_out_duration_ms: 2500,
            stop_fade_duration_ms: 1000,
            timezone: String::from("Europe/Paris"),
            device_deck: String::new(),
            device_instant: String::new(),
            device_aux: String::new(),
            device_preview: String::new(),
            start_locked: false,
            audio_processing_bypassed: false,
            audio_master_volume_percent: 100.0,
            audio_eq_enabled: false,
            audio_eq_gains: vec![0.0; 10],
            audio_compressor_mode: String::from("Custom Values"),
            audio_compressor_preset: String::from("Soft 2"),
            audio_compressor_attack_ms: 25.0,
            audio_compressor_ratio: 4.0,
            audio_compressor_threshold_db: -27.0,
            audio_compressor_gain_db: 0.0,
            audio_compressor_release_ms: 1500.0,
            audio_agc_preset: String::from("Disabled"),
            encoder_bitrate: 128,
            encoder_sample_rate: 44100,
            encoder_channels: 2,
            encoder_type: String::from("LAME MP3"),
            encoder_server_host: String::from("openstudio.entrypoint.belstream.net"),
            encoder_server_port: 80,
            encoder_password: String::new(),
            encoder_mountpoint: String::from("/live"),
            encoder_reconnect_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterOption {
    pub id: Option<i32>,
    pub parent_id: Option<i32>,
    pub name: String,
}

impl FilterOption {
    pub fn all(name: impl Into<String>) -> Self {
        Self {
            id: None,
            parent_id: None,
            name: name.into(),
        }
    }

    pub fn matches_id(&self, id: Option<i32>) -> bool {
        self.id.is_none() || self.id == id
    }
}

impl fmt::Display for FilterOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct SearchTrack {
    pub id: i32,
    pub artist_name: String,
    pub title: String,
    pub path: String,
    pub duration: Duration,
    pub intro: Duration,
    pub outro: Duration,
    pub cue_in: Duration,
    pub cue_out: Duration,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct InstantPage {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InstantSlot {
    pub slot_index: usize,
    pub track_id: i32,
}

#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub id: i32,
    pub track_id: Option<i32>,
    pub artist_name: String,
    pub title: String,
    pub duration: Duration,
    pub intro: Duration,
    pub outro: Duration,
    pub cue_in: Duration,
    pub cue_out: Duration,
    pub scheduled_at: Option<String>,
    pub fixed_time: bool,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

#[derive(Debug)]
pub enum DbError {
    Config(std::io::Error),
    Json(serde_json::Error),
    Postgres(postgres::Error),
    LockPoisoned,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "database configuration unreadable: {error}"),
            Self::Json(error) => write!(f, "database configuration invalid: {error}"),
            Self::Postgres(error) => write!(f, "PostgreSQL: {error}"),
            Self::LockPoisoned => write!(f, "database connection lock is poisoned"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<postgres::Error> for DbError {
    fn from(error: postgres::Error) -> Self {
        Self::Postgres(error)
    }
}

pub struct Database {
    client: Mutex<Client>,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}

impl Database {
    pub fn connect_from_file(path: impl AsRef<Path>) -> Result<SharedDatabase, DbError> {
        let config = DatabaseConfig::from_file(path)?;
        let mut pg_config = Config::new();
        pg_config
            .host(&config.host)
            .port(config.port)
            .dbname(&config.database)
            .user(&config.user)
            .password(&config.password);

        let mut client = pg_config.connect(NoTls)?;
        Self::ensure_config_schema(&mut client)?;
        Ok(Arc::new(Self {
            client: Mutex::new(client),
        }))
    }

    fn ensure_config_schema(client: &mut Client) -> Result<(), DbError> {
        client.batch_execute(
            "
            CREATE EXTENSION IF NOT EXISTS pgcrypto;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS preload INTEGER NOT NULL DEFAULT 10;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS fade_out_duration_ms INTEGER NOT NULL DEFAULT 2500;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS stop_fade_duration_ms INTEGER NOT NULL DEFAULT 1000;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS timezone TEXT NOT NULL DEFAULT 'Europe/Paris';

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_bitrate INTEGER NOT NULL DEFAULT 128;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_sample_rate INTEGER NOT NULL DEFAULT 44100;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_channels INTEGER NOT NULL DEFAULT 2;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_type TEXT NOT NULL DEFAULT 'LAME MP3';

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_server_host TEXT NOT NULL DEFAULT 'openstudio.entrypoint.belstream.net';

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_server_port INTEGER NOT NULL DEFAULT 80;

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_password TEXT NOT NULL DEFAULT '';

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_mountpoint TEXT NOT NULL DEFAULT '/live';

            ALTER TABLE configurations
            ADD COLUMN IF NOT EXISTS encoder_reconnect_seconds INTEGER NOT NULL DEFAULT 10;

            CREATE TABLE IF NOT EXISTS automix_log (
                id        INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                message   TEXT        NOT NULL,
                logged_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
            );

            CREATE INDEX IF NOT EXISTS idx_automix_log_logged_at
            ON automix_log (logged_at);
            ",
        )?;
        Ok(())
    }

    pub fn search_tracks_page(
        &self,
        query: &str,
        category_id: Option<i32>,
        subcategory_id: Option<i32>,
        genre_id: Option<i32>,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<SearchTrack>, usize), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let query = query.trim();
        let offset = offset as i64;
        let limit = limit as i64;

        let count_row = client.query_one(
            "
            SELECT COUNT(*)
            FROM tracks t
            LEFT JOIN artists a ON a.id = t.artist_id
            LEFT JOIN subcategories s ON s.id = t.subcategory_id
            WHERE t.active = TRUE
              AND ($1 = '' OR CONCAT_WS(' ', COALESCE(a.name, ''), t.title) ILIKE '%' || $1 || '%')
              AND ($2::integer IS NULL OR s.category_id = $2)
              AND ($3::integer IS NULL OR t.subcategory_id = $3)
              AND ($4::integer IS NULL OR t.genre_id = $4)
            ",
            &[&query, &category_id, &subcategory_id, &genre_id],
        )?;
        let total: i64 = count_row.get(0);

        let rows = client.query(
            "
            SELECT
                t.id,
                COALESCE(a.name, '') AS artist_name,
                t.title,
                t.path,
                t.duration::double precision AS duration,
                t.intro::double precision AS intro,
                t.outro::double precision AS outro,
                t.cue_in::double precision AS cue_in,
                COALESCE(t.cue_out, t.duration)::double precision AS cue_out,
                to_char(t.updated_at, 'FMMM/FMDD/YYYY HH24:MI:SS') AS updated_at,
                to_char(t.created_at, 'FMMM/FMDD/YYYY HH24:MI:SS') AS created_at
            FROM tracks t
            LEFT JOIN artists a ON a.id = t.artist_id
            LEFT JOIN subcategories s ON s.id = t.subcategory_id
            WHERE t.active = TRUE
              AND ($1 = '' OR CONCAT_WS(' ', COALESCE(a.name, ''), t.title) ILIKE '%' || $1 || '%')
              AND ($2::integer IS NULL OR s.category_id = $2)
              AND ($3::integer IS NULL OR t.subcategory_id = $3)
              AND ($4::integer IS NULL OR t.genre_id = $4)
            ORDER BY a.name NULLS LAST, t.title
            LIMIT $5 OFFSET $6
            ",
            &[
                &query,
                &category_id,
                &subcategory_id,
                &genre_id,
                &limit,
                &offset,
            ],
        )?;

        Ok((
            rows.into_iter().map(search_track_from_row).collect(),
            total.max(0) as usize,
        ))
    }

    pub fn search_track(&self, track_id: i32) -> Result<Option<SearchTrack>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "
            SELECT
                t.id,
                COALESCE(a.name, '') AS artist_name,
                t.title,
                t.path,
                t.duration::double precision AS duration,
                t.intro::double precision AS intro,
                t.outro::double precision AS outro,
                t.cue_in::double precision AS cue_in,
                COALESCE(t.cue_out, t.duration)::double precision AS cue_out,
                to_char(t.updated_at, 'FMMM/FMDD/YYYY HH24:MI:SS') AS updated_at,
                to_char(t.created_at, 'FMMM/FMDD/YYYY HH24:MI:SS') AS created_at
            FROM tracks t
            LEFT JOIN artists a ON a.id = t.artist_id
            LEFT JOIN subcategories s ON s.id = t.subcategory_id
            WHERE t.active = TRUE AND t.id = $1
            ",
            &[&track_id],
        )?;

        Ok(rows.into_iter().next().map(search_track_from_row))
    }

    pub fn categories(&self) -> Result<Vec<FilterOption>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query("SELECT id, name FROM categories ORDER BY id", &[])?;
        Ok(rows
            .into_iter()
            .map(|row| FilterOption {
                id: Some(row.get("id")),
                parent_id: None,
                name: row.get("name"),
            })
            .collect())
    }

    pub fn subcategories(&self) -> Result<Vec<FilterOption>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "
            SELECT id, category_id, name
            FROM subcategories
            WHERE hidden = FALSE
            ORDER BY category_id, id
            ",
            &[],
        )?;
        Ok(rows
            .into_iter()
            .map(|row| FilterOption {
                id: Some(row.get("id")),
                parent_id: Some(row.get("category_id")),
                name: row.get("name"),
            })
            .collect())
    }

    pub fn genres(&self) -> Result<Vec<FilterOption>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query("SELECT id, name FROM genres ORDER BY name", &[])?;
        Ok(rows
            .into_iter()
            .map(|row| FilterOption {
                id: Some(row.get("id")),
                parent_id: None,
                name: row.get("name"),
            })
            .collect())
    }

    pub fn queue_entries(&self, timezone: &str) -> Result<Vec<QueueEntry>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "
            SELECT
                q.id,
                q.track_id,
                q.fixed_time,
                q.cue_in::double precision AS cue_in,
                q.cue_out::double precision AS cue_out,
                COALESCE(t.intro, 0)::double precision AS intro,
                COALESCE(t.outro, 0)::double precision AS outro,
                COALESCE(a.name, '') AS artist_name,
                COALESCE(t.title, '') AS title,
                COALESCE(t.duration, 0)::double precision AS duration,
                to_char(q.scheduled_at AT TIME ZONE $1, 'HH24:MI:SS') AS scheduled_at
            FROM queue q
            LEFT JOIN tracks t ON t.id = q.track_id
            LEFT JOIN artists a ON a.id = t.artist_id
            WHERE q.played = FALSE
              AND (q.scheduled_at IS NULL OR q.scheduled_at >= NOW())
            ORDER BY q.scheduled_at NULLS LAST, q.priority, q.id
            LIMIT 50
            ",
            &[&timezone],
        )?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let duration: f64 = row.get("duration");
                let cue_in: f64 = row.get("cue_in");
                let cue_out: f64 = row.get("cue_out");
                let intro: f64 = row.get("intro");
                let outro: f64 = row.get("outro");
                QueueEntry {
                    id: row.get("id"),
                    track_id: row.get("track_id"),
                    artist_name: row.get("artist_name"),
                    title: row.get("title"),
                    duration: seconds_to_duration(duration),
                    intro: seconds_to_duration(intro),
                    outro: seconds_to_duration(outro),
                    cue_in: seconds_to_duration(cue_in),
                    cue_out: seconds_to_duration(cue_out),
                    scheduled_at: row.get("scheduled_at"),
                    fixed_time: row.get("fixed_time"),
                }
            })
            .collect())
    }

    pub fn instant_pages(&self) -> Result<Vec<InstantPage>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query("SELECT id, name FROM instant_pages ORDER BY id", &[])?;
        Ok(rows
            .into_iter()
            .map(|row| InstantPage {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect())
    }

    pub fn instant_slots(&self, page_id: i32) -> Result<Vec<InstantSlot>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "
            SELECT slot_index, track_id
            FROM instant_slots
            WHERE page_id = $1
            ORDER BY slot_index
            ",
            &[&page_id],
        )?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let slot_index: i16 = row.get("slot_index");
                InstantSlot {
                    slot_index: slot_index.max(0) as usize,
                    track_id: row.get("track_id"),
                }
            })
            .collect())
    }

    pub fn insert_instant_page(&self, name: &str) -> Result<i32, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let row = client.query_one(
            "INSERT INTO instant_pages (name) VALUES ($1) RETURNING id",
            &[&name],
        )?;
        Ok(row.get("id"))
    }

    pub fn update_instant_page_name(&self, page_id: i32, name: &str) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute(
            "UPDATE instant_pages SET name = $1 WHERE id = $2",
            &[&name, &page_id],
        )?;
        Ok(())
    }

    pub fn delete_instant_page(&self, page_id: i32) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute("DELETE FROM instant_pages WHERE id = $1", &[&page_id])?;
        Ok(())
    }

    pub fn clear_instant_slots(&self, page_id: i32) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute("DELETE FROM instant_slots WHERE page_id = $1", &[&page_id])?;
        Ok(())
    }

    pub fn insert_queue_entry(&self, track_id: i32) -> Result<i32, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let row = client.query_one(
            "
            INSERT INTO queue (track_id, cue_in, cue_out)
            SELECT id, cue_in, COALESCE(cue_out, duration)
            FROM tracks WHERE id = $1
            RETURNING id
            ",
            &[&track_id],
        )?;
        Ok(row.get("id"))
    }

    pub fn replace_queue_entry(&self, queue_id: i32, track_id: i32) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute(
            "
            UPDATE queue SET
                track_id = t.id,
                cue_in = t.cue_in,
                cue_out = COALESCE(t.cue_out, t.duration),
                updated_at = NOW()
            FROM tracks t
            WHERE t.id = $1 AND queue.id = $2
            ",
            &[&track_id, &queue_id],
        )?;
        Ok(())
    }

    pub fn delete_queue_entry(&self, id: i32) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute(
            "UPDATE queue SET played = TRUE, updated_at = NOW() WHERE id = $1",
            &[&id],
        )?;
        Ok(())
    }

    pub fn mark_track_played(&self, track_id: i32) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute(
            "
            WITH played_track AS (
                UPDATE tracks
                SET last_played_at = NOW()
                WHERE id = $1
                RETURNING artist_id
            )
            UPDATE artists
            SET last_broadcast_at = NOW()
            WHERE id IN (
                SELECT artist_id
                FROM played_track
                WHERE artist_id IS NOT NULL
            )
            ",
            &[&track_id],
        )?;
        Ok(())
    }

    pub fn insert_play_log(
        &self,
        track_id: i32,
        played_duration: Duration,
        count_commercial_campaign: bool,
    ) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let mut transaction = client.transaction()?;
        let played_duration = played_duration.as_secs_f32();
        transaction.execute(
            "
            INSERT INTO play_log (track_id, played_duration)
            VALUES ($1, $2)
            ",
            &[&track_id, &played_duration],
        )?;

        if count_commercial_campaign {
            transaction.execute(
                "
                UPDATE campaigns cp
                SET
                    broadcast_count = cp.broadcast_count + 1,
                    last_aired_at = NOW()
                FROM campaign_tracks ct
                JOIN tracks t ON t.id = ct.track_id
                WHERE cp.id = ct.campaign_id
                  AND ct.track_id = $1
                  AND t.track_type_id = 9
                ",
                &[&track_id],
            )?;
        }

        transaction.commit()?;
        Ok(())
    }

    pub fn insert_automix_log(&self, message: &str) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        client.execute(
            "
            INSERT INTO automix_log (message)
            VALUES ($1)
            ",
            &[&message],
        )?;
        Ok(())
    }

    pub fn load_config(&self) -> Result<AppConfig, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let row = client.query_one("SELECT * FROM configurations LIMIT 1", &[])?;
        let default_cfg = AppConfig::default();
        let eq_gains_json: String = row
            .try_get("audio_eq_gains")
            .unwrap_or_else(|_| String::from("[0,0,0,0,0,0,0,0,0,0]"));
        let mut audio_eq_gains = serde_json::from_str::<Vec<f32>>(&eq_gains_json)
            .unwrap_or_else(|_| default_cfg.audio_eq_gains.clone());
        audio_eq_gains.resize(10, 0.0);
        audio_eq_gains.truncate(10);
        Ok(AppConfig {
            auto_mix_on_start: row.get("auto_mix_on_start"),
            auto_play_on_start: row.get("auto_play_on_start"),
            preload: row.get("preload"),
            fade_out_duration_ms: row.get("fade_out_duration_ms"),
            stop_fade_duration_ms: row.get("stop_fade_duration_ms"),
            timezone: row.get("timezone"),
            device_deck: row.get("device_deck"),
            device_instant: row.get("device_instant"),
            device_aux: row.get("device_aux"),
            device_preview: row.get("device_preview"),
            start_locked: row.get("start_locked"),
            audio_processing_bypassed: row
                .try_get("audio_processing_bypassed")
                .unwrap_or(default_cfg.audio_processing_bypassed),
            audio_master_volume_percent: row
                .try_get("audio_master_volume_percent")
                .unwrap_or(default_cfg.audio_master_volume_percent),
            audio_eq_enabled: row
                .try_get("audio_eq_enabled")
                .unwrap_or(default_cfg.audio_eq_enabled),
            audio_eq_gains,
            audio_compressor_mode: row
                .try_get("audio_compressor_mode")
                .unwrap_or(default_cfg.audio_compressor_mode),
            audio_compressor_preset: row
                .try_get("audio_compressor_preset")
                .unwrap_or(default_cfg.audio_compressor_preset),
            audio_compressor_attack_ms: row
                .try_get("audio_compressor_attack_ms")
                .unwrap_or(default_cfg.audio_compressor_attack_ms),
            audio_compressor_ratio: row
                .try_get("audio_compressor_ratio")
                .unwrap_or(default_cfg.audio_compressor_ratio),
            audio_compressor_threshold_db: row
                .try_get("audio_compressor_threshold_db")
                .unwrap_or(default_cfg.audio_compressor_threshold_db),
            audio_compressor_gain_db: row
                .try_get("audio_compressor_gain_db")
                .unwrap_or(default_cfg.audio_compressor_gain_db),
            audio_compressor_release_ms: row
                .try_get("audio_compressor_release_ms")
                .unwrap_or(default_cfg.audio_compressor_release_ms),
            audio_agc_preset: row
                .try_get("audio_agc_preset")
                .unwrap_or(default_cfg.audio_agc_preset),
            encoder_bitrate: row
                .try_get("encoder_bitrate")
                .unwrap_or(default_cfg.encoder_bitrate),
            encoder_sample_rate: row
                .try_get("encoder_sample_rate")
                .unwrap_or(default_cfg.encoder_sample_rate),
            encoder_channels: row
                .try_get("encoder_channels")
                .unwrap_or(default_cfg.encoder_channels),
            encoder_type: row
                .try_get("encoder_type")
                .unwrap_or(default_cfg.encoder_type),
            encoder_server_host: row
                .try_get("encoder_server_host")
                .unwrap_or(default_cfg.encoder_server_host),
            encoder_server_port: row
                .try_get("encoder_server_port")
                .unwrap_or(default_cfg.encoder_server_port),
            encoder_password: row
                .try_get("encoder_password")
                .unwrap_or(default_cfg.encoder_password),
            encoder_mountpoint: row
                .try_get("encoder_mountpoint")
                .unwrap_or(default_cfg.encoder_mountpoint),
            encoder_reconnect_seconds: row
                .try_get("encoder_reconnect_seconds")
                .unwrap_or(default_cfg.encoder_reconnect_seconds),
        })
    }

    pub fn timezones(&self) -> Result<Vec<String>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "
            SELECT name
            FROM pg_timezone_names
            WHERE name NOT LIKE 'posix/%'
              AND name NOT LIKE 'right/%'
            ORDER BY name
            ",
            &[],
        )?;
        Ok(rows.into_iter().map(|row| row.get("name")).collect())
    }

    pub fn save_config(&self, cfg: &AppConfig) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let audio_eq_gains = serde_json::to_string(&cfg.audio_eq_gains)
            .unwrap_or_else(|_| String::from("[0,0,0,0,0,0,0,0,0,0]"));
        client.execute(
            "
            UPDATE configurations
            SET auto_mix_on_start = $1,
                auto_play_on_start = $2,
                preload = $3,
                fade_out_duration_ms = $4,
                stop_fade_duration_ms = $5,
                timezone = $6,
                device_deck = $7,
                device_instant = $8,
                device_aux = $9,
                device_preview = $10,
                start_locked = $11,
                audio_processing_bypassed = $12,
                audio_master_volume_percent = $13,
                audio_eq_enabled = $14,
                audio_eq_gains = $15,
                audio_compressor_mode = $16,
                audio_compressor_preset = $17,
                audio_compressor_attack_ms = $18,
                audio_compressor_ratio = $19,
                audio_compressor_threshold_db = $20,
                audio_compressor_gain_db = $21,
                audio_compressor_release_ms = $22,
                audio_agc_preset = $23,
                encoder_bitrate = $24,
                encoder_sample_rate = $25,
                encoder_channels = $26,
                encoder_type = $27,
                encoder_server_host = $28,
                encoder_server_port = $29,
                encoder_password = $30,
                encoder_mountpoint = $31,
                encoder_reconnect_seconds = $32
            ",
            &[
                &cfg.auto_mix_on_start,
                &cfg.auto_play_on_start,
                &cfg.preload,
                &cfg.fade_out_duration_ms,
                &cfg.stop_fade_duration_ms,
                &cfg.timezone,
                &cfg.device_deck,
                &cfg.device_instant,
                &cfg.device_aux,
                &cfg.device_preview,
                &cfg.start_locked,
                &cfg.audio_processing_bypassed,
                &cfg.audio_master_volume_percent,
                &cfg.audio_eq_enabled,
                &audio_eq_gains,
                &cfg.audio_compressor_mode,
                &cfg.audio_compressor_preset,
                &cfg.audio_compressor_attack_ms,
                &cfg.audio_compressor_ratio,
                &cfg.audio_compressor_threshold_db,
                &cfg.audio_compressor_gain_db,
                &cfg.audio_compressor_release_ms,
                &cfg.audio_agc_preset,
                &cfg.encoder_bitrate,
                &cfg.encoder_sample_rate,
                &cfg.encoder_channels,
                &cfg.encoder_type,
                &cfg.encoder_server_host,
                &cfg.encoder_server_port,
                &cfg.encoder_password,
                &cfg.encoder_mountpoint,
                &cfg.encoder_reconnect_seconds,
            ],
        )?;
        Ok(())
    }

    pub fn check_credentials(
        &self,
        login: &str,
        password: &str,
    ) -> Result<Option<(String, i16)>, DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let rows = client.query(
            "SELECT login, role_id FROM users
             WHERE login = $1
               AND password_hash = crypt($2, password_hash)
               AND active = TRUE",
            &[&login, &password],
        )?;
        Ok(rows
            .first()
            .map(|row| (row.get::<_, String>("login"), row.get::<_, i16>("role_id"))))
    }

    pub fn insert_instant_slot(
        &self,
        page_id: i32,
        slot_index: usize,
        track_id: i32,
    ) -> Result<(), DbError> {
        let mut client = self.client.lock().map_err(|_| DbError::LockPoisoned)?;
        let slot_index = slot_index as i16;
        client.execute(
            "
            INSERT INTO instant_slots (page_id, slot_index, track_id)
            VALUES ($1, $2, $3)
            ",
            &[&page_id, &slot_index, &track_id],
        )?;
        Ok(())
    }
}

impl DatabaseConfig {
    fn from_file(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let raw = fs::read_to_string(path).map_err(DbError::Config)?;
        serde_json::from_str(&raw).map_err(DbError::Json)
    }
}

fn seconds_to_duration(seconds: f64) -> Duration {
    if seconds.is_finite() {
        Duration::from_secs_f64(seconds.max(0.0))
    } else {
        Duration::ZERO
    }
}

fn search_track_from_row(row: Row) -> SearchTrack {
    let duration: f64 = row.get("duration");
    let intro: f64 = row.get("intro");
    let outro: f64 = row.get("outro");
    let cue_in: f64 = row.get("cue_in");
    let cue_out: f64 = row.get("cue_out");

    SearchTrack {
        id: row.get("id"),
        artist_name: row.get("artist_name"),
        title: row.get("title"),
        path: row.get("path"),
        duration: seconds_to_duration(duration),
        intro: seconds_to_duration(intro),
        outro: seconds_to_duration(outro),
        cue_in: seconds_to_duration(cue_in),
        cue_out: seconds_to_duration(cue_out),
        updated_at: row.get("updated_at"),
        created_at: row.get("created_at"),
    }
}

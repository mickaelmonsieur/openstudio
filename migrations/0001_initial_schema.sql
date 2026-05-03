-- OpenStudio — initial schema

-- ── Reference tables (no dependencies) ──────────────────────────────────────

CREATE TABLE templates (
    id   INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(32) NOT NULL
);

CREATE TABLE categories (
    id        INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name      VARCHAR(32) NOT NULL,
    protected BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE TABLE artists (
    id                INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name              VARCHAR(64) NOT NULL,
    last_broadcast_at TIMESTAMPTZ,
    CONSTRAINT uq_artists_name UNIQUE (name)
);

CREATE TABLE stations (
    id           INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         VARCHAR(64)  NOT NULL,
    library_path VARCHAR(255) NOT NULL DEFAULT '',
    CONSTRAINT uq_stations_name UNIQUE (name)
);

CREATE TABLE genres (
    id   INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    CONSTRAINT uq_genres_name UNIQUE (name)
);

-- ── Category hierarchy ───────────────────────────────────────────────────────

CREATE TABLE subcategories (
    id          INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    category_id INTEGER     NOT NULL REFERENCES categories (id),
    name        VARCHAR(32) NOT NULL,
    hidden      BOOLEAN     NOT NULL DEFAULT FALSE,
    protected   BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_subcategories_category ON subcategories (category_id);

-- ── Template slots ──────────────────────────────────────────────────────────
-- track_protection / artist_protection: minimum seconds between replays

CREATE TABLE template_slots (
    id                INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id       INTEGER     NOT NULL REFERENCES templates (id),
    position          INTEGER     NOT NULL DEFAULT 0 CHECK (position >= 0),
    category_id       INTEGER     NOT NULL REFERENCES categories (id),
    subcategory_id    INTEGER     REFERENCES subcategories (id),
    comment           VARCHAR(64) NOT NULL DEFAULT '',
    track_protection  INTEGER     NOT NULL DEFAULT 3600,
    artist_protection INTEGER     NOT NULL DEFAULT 3600
);

CREATE INDEX idx_template_slots_order ON template_slots (template_id, position, id);

-- ── Media library ───────────────────────────────────────────────────────────

CREATE TABLE tracks (
    id             INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    artist_id      INTEGER     REFERENCES artists (id),
    genre_id       INTEGER     REFERENCES genres (id),
    title          VARCHAR(64) NOT NULL DEFAULT '',
    album          VARCHAR(64) NOT NULL DEFAULT '',
    year           SMALLINT,
    duration       REAL        NOT NULL DEFAULT 0,
    sample_rate    INTEGER     NOT NULL DEFAULT 44100,
    cue_in         REAL        NOT NULL DEFAULT 0,
    cue_out        REAL,
    intro          REAL        NOT NULL DEFAULT 0,
    outro          REAL        NOT NULL DEFAULT 0,
    hook_in        REAL        NOT NULL DEFAULT 0,
    hook_out       REAL        NOT NULL DEFAULT 0,
    loop_in        REAL        NOT NULL DEFAULT 0,
    loop_out       REAL        NOT NULL DEFAULT 0,
    path           TEXT        NOT NULL DEFAULT '' UNIQUE,
    subcategory_id INTEGER     REFERENCES subcategories (id),
    active         BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_played_at TIMESTAMPTZ
);

CREATE INDEX idx_tracks_artist   ON tracks (artist_id);
CREATE INDEX idx_tracks_genre    ON tracks (genre_id);
CREATE INDEX idx_tracks_category ON tracks (subcategory_id, active);
CREATE INDEX idx_tracks_title    ON tracks (title);

-- ── Advertising ─────────────────────────────────────────────────────────────

CREATE TABLE sectors (
    id   INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    CONSTRAINT uq_sectors_name UNIQUE (name)
);

CREATE TABLE advertisers (
    id           INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name         VARCHAR(255) NOT NULL,
    sector_id    INTEGER      REFERENCES sectors (id),
    address      TEXT,
    vat_number   VARCHAR(32),
    notes        TEXT,
    active       BOOLEAN      NOT NULL DEFAULT TRUE,
    client_since DATE,
    CONSTRAINT uq_advertisers_name UNIQUE (name)
);

CREATE TABLE contacts (
    id            INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    advertiser_id INTEGER      NOT NULL REFERENCES advertisers (id),
    name          VARCHAR(128) NOT NULL,
    role          VARCHAR(64),
    phone         VARCHAR(32),
    email         VARCHAR(128),
    primary_contact BOOLEAN    NOT NULL DEFAULT FALSE,
    notes         TEXT
);

CREATE INDEX idx_contacts_advertiser ON contacts (advertiser_id);

CREATE TABLE campaigns (
    id               INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    advertiser_id    INTEGER      NOT NULL REFERENCES advertisers (id),
    name             VARCHAR(255) NOT NULL DEFAULT '',
    total_broadcasts INTEGER      NOT NULL DEFAULT 0,
    broadcast_count  INTEGER      NOT NULL DEFAULT 0,
    station_id       INTEGER      REFERENCES stations (id),
    active           BOOLEAN      NOT NULL DEFAULT TRUE,
    encoded_at       TIMESTAMPTZ,
    start_date       DATE,
    end_date         DATE,
    last_aired_at    TIMESTAMPTZ
);

CREATE TABLE campaign_tracks (
    id          INTEGER  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    campaign_id INTEGER  NOT NULL REFERENCES campaigns (id),
    track_id    INTEGER  NOT NULL REFERENCES tracks (id),
    position    SMALLINT NOT NULL DEFAULT 1,
    CONSTRAINT uq_campaign_track_position UNIQUE (campaign_id, position),
    CONSTRAINT uq_campaign_track_track    UNIQUE (track_id)
);

CREATE INDEX idx_campaign_tracks_campaign ON campaign_tracks (campaign_id);
CREATE INDEX idx_campaign_tracks_track    ON campaign_tracks (track_id);

-- ── Clock events ────────────────────────────────────────────────────────────

CREATE TABLE clock_events (
    id          INTEGER  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    hour        SMALLINT NOT NULL DEFAULT 0  CHECK (hour   BETWEEN 0 AND 23),
    minute      SMALLINT NOT NULL DEFAULT 0  CHECK (minute BETWEEN 0 AND 59),
    second      SMALLINT NOT NULL DEFAULT 0  CHECK (second BETWEEN 0 AND 59),
    template_id INTEGER  REFERENCES templates (id),
    priority    SMALLINT NOT NULL DEFAULT 0,
    duration    REAL     NOT NULL DEFAULT 0
);

CREATE INDEX idx_clock_events_hour ON clock_events (hour);

-- ── Programming schedule ────────────────────────────────────────────────────

CREATE TABLE schedules (
    id          INTEGER  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    from_hour   SMALLINT NOT NULL DEFAULT 0  CHECK (from_hour BETWEEN 0 AND 23),
    to_hour     SMALLINT NOT NULL DEFAULT 23 CHECK (to_hour   BETWEEN 0 AND 23),
    monday      BOOLEAN  NOT NULL DEFAULT FALSE,
    tuesday     BOOLEAN  NOT NULL DEFAULT FALSE,
    wednesday   BOOLEAN  NOT NULL DEFAULT FALSE,
    thursday    BOOLEAN  NOT NULL DEFAULT FALSE,
    friday      BOOLEAN  NOT NULL DEFAULT FALSE,
    saturday    BOOLEAN  NOT NULL DEFAULT FALSE,
    sunday      BOOLEAN  NOT NULL DEFAULT FALSE,
    template_id INTEGER  NOT NULL REFERENCES templates (id)
);

-- ── Play queue ──────────────────────────────────────────────────────────────

CREATE TABLE queue (
    id           INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    track_id     INTEGER     REFERENCES tracks (id),
    cue_in       REAL        NOT NULL DEFAULT 0,
    cue_out      REAL        NOT NULL DEFAULT 0,
    stretch_rate REAL        NOT NULL DEFAULT 1.0,
    played       BOOLEAN     NOT NULL DEFAULT FALSE,
    priority     SMALLINT    NOT NULL DEFAULT 0,
    fixed_time   BOOLEAN     NOT NULL DEFAULT FALSE,
    scheduled_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_queue_scheduled ON queue (scheduled_at, priority);

-- ── Instant player pages ────────────────────────────────────────────────────

CREATE TABLE instant_pages (
    id         INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name       VARCHAR(64) NOT NULL DEFAULT 'Default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE instant_slots (
    id         INTEGER  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    page_id    INTEGER  NOT NULL REFERENCES instant_pages (id) ON DELETE CASCADE,
    slot_index SMALLINT NOT NULL CHECK (slot_index BETWEEN 0 AND 9),
    track_id   INTEGER  NOT NULL REFERENCES tracks (id),
    CONSTRAINT uq_instant_slots_page_slot UNIQUE (page_id, slot_index)
);

CREATE INDEX idx_instant_slots_page ON instant_slots (page_id);
CREATE INDEX idx_instant_slots_track ON instant_slots (track_id);

-- ── Play log — what was actually aired ──────────────────────────────────────

CREATE TABLE play_log (
    id              INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    track_id        INTEGER     REFERENCES tracks (id),
    station_id      INTEGER     REFERENCES stations (id),
    played_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    played_duration REAL        NOT NULL DEFAULT 0
);

CREATE INDEX idx_play_log_played_at ON play_log (played_at);
CREATE INDEX idx_play_log_track     ON play_log (track_id);
CREATE INDEX idx_play_log_station   ON play_log (station_id, played_at);

-- ── Auto mix log — status messages shown in the footer ─────────────────────

CREATE TABLE automix_log (
    id        INTEGER     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    message   TEXT        NOT NULL,
    logged_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX idx_automix_log_logged_at ON automix_log (logged_at);

-- ── Users ───────────────────────────────────────────────────────────────────

CREATE TABLE users_roles (
    id   SMALLINT    GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name VARCHAR(32) NOT NULL,
    CONSTRAINT uq_users_roles_name UNIQUE (name)
);

CREATE TABLE users (
    id            INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    login         VARCHAR(32)  NOT NULL,
    password_hash VARCHAR(255) NOT NULL DEFAULT '',
    active        BOOLEAN      NOT NULL DEFAULT FALSE,
    role_id       SMALLINT     NOT NULL DEFAULT 4 REFERENCES users_roles(id),
    CONSTRAINT uq_users_login UNIQUE (login)
);

-- ── User audit log ──────────────────────────────────────────────────────────

CREATE TABLE users_actions (
    id           INTEGER      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id      INTEGER      NOT NULL REFERENCES users (id),
    action       VARCHAR(64)  NOT NULL DEFAULT '',
    performed_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_actions_user ON users_actions (user_id);

-- ── App configuration (singleton) ───────────────────────────────────────────

CREATE TABLE configurations (
    id                   INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    auto_mix_on_start    BOOLEAN NOT NULL DEFAULT false,
    auto_play_on_start   BOOLEAN NOT NULL DEFAULT false,
    preload              INTEGER NOT NULL DEFAULT 10,
    fade_out_duration_ms INTEGER NOT NULL DEFAULT 2500,
    stop_fade_duration_ms INTEGER NOT NULL DEFAULT 1000,
    timezone             TEXT    NOT NULL DEFAULT 'Europe/Paris'
);

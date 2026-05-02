# OpenStudio — Database Schema

## Overview

The schema is organized in layers: reference data → media library → scheduling → playback → users.

```
formats ──┐
           ├──► templates ──► clock_events
categories ┤              └──► schedules
           │
subcategories ──► tracks ──► queue
artists ─────────────────┘
                              play_log
                              campaigns ◄── stations
                              campaigns ◄── advertisers ◄── sectors
                              campaigns ◄── campaign_tracks ──► tracks
                                             advertisers └──► contacts

users ──► user_actions
```

---

## Reference tables

### `formats`
Programming formats available on the station. Defines the overall broadcast style that a template belongs to.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(32) | Format name (e.g. `SEMAINE`, `WEEKEND-80`, `PUB`) |

Seed values: `PUB`, `TOP HORAIRE`, `SEMAINE`, `WEEKEND-80`.

---

### `categories`
Top-level content categories. Every track and template belongs to one category.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(32) | Category name |

Seed values: `Jingles`, `Music`, `Intervention`, `PubIn`, `PubOut`, `Filler`, `Top of Hour`, `Pub`.

---

### `subcategories`
Second-level classification under a category. For example, the `Music` category contains subcategories like `FR-1980`, `PowerPlay`, etc. The `hidden` flag controls whether the subcategory is visible in the UI.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `category_id` | INTEGER | FK → `categories` |
| `name` | VARCHAR(32) | Subcategory name |
| `hidden` | BOOLEAN | Hide from public UI (default `false`) |

---

### `artists`
Artist registry. Each track references an artist. `last_broadcast_at` is updated whenever a track from this artist is played — used to enforce the artist protection rule.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(64) | Artist name (unique) |
| `last_broadcast_at` | TIMESTAMPTZ | Last time a track from this artist was aired |

---

### `stations`
Radio stations managed by the system. Used to scope advertising campaigns and local ads to a specific station.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(64) | Station name |

---

### `genres`
Musical genre reference list. Each track can optionally be linked to a genre. The list is seeded with 337 standard genres (ID3 tag standard + extended). Stored as a table rather than a plain string on `tracks` to ensure consistency and allow grouping by genre.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(50) | Genre name (unique) |

---

## Programming templates

### `templates`
A template (formerly "canvas") defines a programming slot: what category/subcategory of content to play, and how long to wait before replaying the same track or the same artist. Templates are referenced by the schedule and clock events.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `format_id` | INTEGER | FK → `formats` |
| `category_id` | INTEGER | FK → `categories` — content type to pick from |
| `subcategory_id` | INTEGER | FK → `subcategories` — optional narrower filter |
| `comment` | VARCHAR(64) | Free label (e.g. `1ER DISQUE`, `RETOUR PUB`) |
| `track_protection` | INTEGER | Minimum seconds before the same track can play again |
| `artist_protection` | INTEGER | Minimum seconds before the same artist can play again |

---

## Media library

### `tracks`
The central media library. Every audio file (music, jingle, ad, intervention…) has one row here. The `subcategory_id` determines in which programming slots the track is eligible to play; the parent category is derived from the subcategory via join. `active = false` disables a track without deleting it.

Audio playback metadata (`bpm`, `intro`, `fade_in`, `fade_out`) is stored here so the player can prepare transitions without reading the file at runtime. `fade_out = NULL` means "no cue override"; automation uses `duration` as the effective fade-out reference.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `artist_id` | INTEGER | FK → `artists` |
| `genre_id` | INTEGER | FK → `genres` (optional) |
| `title` | VARCHAR(64) | Track title |
| `album` | VARCHAR(64) | Album name |
| `year` | SMALLINT | Release year |
| `duration` | REAL | Duration in seconds |
| `sample_rate` | INTEGER | Native sample rate (Hz, default 44100) |
| `bpm` | REAL | Tempo |
| `intro` | REAL | Duration of the intro in seconds (before the vocal starts) |
| `fade_in` | REAL | Fade-in duration in seconds |
| `fade_out` | REAL | Optional fade-out cue in seconds; `NULL` means use `duration` |
| `path` | TEXT | Absolute path to the audio file |
| `subcategory_id` | INTEGER | FK → `subcategories` (category derived via join) |
| `active` | BOOLEAN | Whether the track is enabled for scheduling |
| `created_at` | TIMESTAMPTZ | Insertion date |
| `updated_at` | TIMESTAMPTZ | Last modification date |
| `last_played_at` | TIMESTAMPTZ | Last time the track was aired |

---

## Advertising

### `sectors`
Industry sector reference list. Used to classify advertisers and group campaigns by market segment. Avoids free-text inconsistencies.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(64) | Sector name (unique) |

Seed values: Automotive, Construction & Real Estate, Consumer Goods & Retail, Education & Training, Energy & Utilities, Finance & Insurance, Food & Beverage, Government & Public Sector, Healthcare & Wellness, Hospitality & Tourism, Legal & Accounting, Manufacturing & Industry, Media & Communications, Non-profit & Associations, Pharmaceutical, Services & Consulting, Technology, Telecommunications, Transport & Logistics, Other.

---

### `advertisers`
Advertiser registry (mini-CRM). Creating an advertiser once avoids duplicates and allows retrieving all campaigns for a given client with a simple join.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `name` | VARCHAR(255) | Company name (unique) |
| `sector_id` | INTEGER | FK → `sectors` |
| `address` | TEXT | Billing address |
| `vat_number` | VARCHAR(32) | VAT / company registration number |
| `notes` | TEXT | Free-text notes (relationship history, preferences) |
| `active` | BOOLEAN | Active client (default `true`) |
| `client_since` | DATE | Date of first contract |

---

### `contacts`
People at an advertiser's company. One advertiser can have multiple contacts (decision maker, accountant, marketing manager…). `primary_contact` flags the default person to reach.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `advertiser_id` | INTEGER | FK → `advertisers` |
| `name` | VARCHAR(128) | Contact person's full name |
| `role` | VARCHAR(64) | Job title / role (e.g. `Marketing Manager`) |
| `phone` | VARCHAR(32) | Phone number |
| `email` | VARCHAR(128) | Email address |
| `primary_contact` | BOOLEAN | Main point of contact (default `false`) |
| `notes` | TEXT | Free-text notes about this person |

---

### `campaigns`
An advertising campaign links an advertiser to one or more spots for a given station, over a date range. `broadcast_count` is incremented each time any spot from the campaign airs; when it reaches `total_broadcasts` the campaign is considered exhausted. `active = false` pauses the campaign manually.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `advertiser_id` | INTEGER | FK → `advertisers` |
| `name` | VARCHAR(255) | Campaign name |
| `total_broadcasts` | INTEGER | Total number of airings contracted |
| `broadcast_count` | INTEGER | Number of airings already done |
| `station_id` | INTEGER | FK → `stations` |
| `active` | BOOLEAN | Campaign enabled |
| `encoded_at` | TIMESTAMPTZ | When the campaign was registered |
| `start_date` | DATE | Campaign start date |
| `end_date` | DATE | Campaign end date |
| `last_aired_at` | TIMESTAMPTZ | Last broadcast timestamp |

---

### `campaign_tracks`
Junction table between a campaign and its spots. `position` defines the rotation order — the player cycles through spots 1 → 2 → 3 → … → 1. A campaign with a single spot simply has one row at position 1.

`track_id` is unique across the whole table: a spot belongs to exactly one campaign. This makes the reverse lookup unambiguous — given a `track_id` from `play_log`, a single join on `campaign_tracks` yields the parent campaign, which can then have its `broadcast_count` incremented.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `campaign_id` | INTEGER | FK → `campaigns` |
| `track_id` | INTEGER | FK → `tracks` (unique — one spot, one campaign) |
| `position` | SMALLINT | Rotation order (unique per campaign) |

---

## Scheduling

### `clock_events`
Defines fixed clock triggers such as top-of-hour events and ad breaks. Each row specifies a clock time (`hour:minute:second`) and the template to use for that slot. The `duration` field gives the planned length of the event in seconds.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `hour` | SMALLINT | Hour of the trigger (0–23) |
| `minute` | SMALLINT | Minute of the trigger (0–59) |
| `second` | SMALLINT | Second of the trigger (0–59) |
| `template_id` | INTEGER | FK → `templates` — content to play at this slot |
| `priority` | SMALLINT | Priority if multiple slots overlap |
| `duration` | REAL | Planned break duration in seconds |

---

### `schedules`
Maps time ranges and weekdays to a programming template. The scheduler uses this table to determine which template governs music selection at any given moment of the week. Multiple rows can coexist for different time slots or day combinations.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `from_hour` | SMALLINT | Start hour (0–23) |
| `to_hour` | SMALLINT | End hour (0–23) |
| `monday` … `sunday` | BOOLEAN | Days on which this schedule applies |
| `template_id` | INTEGER | FK → `templates` — programming template to apply (required) |

---

## Playback queues

### `queue`
The live play queue. The unified play queue — contains everything: music, jingles, ads. The engine reads entries in `scheduled_at` order, breaking ties with `priority`. `played` is set to `true` once the track starts.

`fixed_time = true` means the entry must play exactly at `scheduled_at` (e.g. top-of-hour jingle); `fixed_time = false` allows the engine to slide the entry slightly to accommodate transitions.

Audio metadata (`bpm`, `intro`, `fade_in`, `fade_out`) is copied from `tracks` during automatic scheduling, but can differ from `tracks` when a slot is scheduled manually — allowing per-occurrence overrides without touching the library. When `tracks.fade_out` is `NULL`, queue generation stores `tracks.duration` as the effective `fade_out`.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `track_id` | INTEGER | FK → `tracks` |
| `sample_rate` | INTEGER | Sample rate at playback time |
| `bpm` | REAL | Tempo |
| `intro` | REAL | Intro duration (seconds) |
| `fade_in` | REAL | Fade-in duration (seconds) |
| `fade_out` | REAL | Fade-out offset (seconds) |
| `played` | BOOLEAN | Whether the track has started playing |
| `priority` | SMALLINT | Higher value = higher priority when resolving conflicts (default 0) |
| `fixed_time` | BOOLEAN | Must play exactly at `scheduled_at`, no sliding (default `false`) |
| `scheduled_at` | TIMESTAMPTZ | Planned play time |
| `created_at` | TIMESTAMPTZ | When the entry was added to the queue |
| `updated_at` | TIMESTAMPTZ | Last modification |

---

## Logs

### `play_log`
Immutable record of everything that has aired. One row per broadcast event. Used for royalty reporting (SACEM/SABAM) and replay history.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `track_id` | INTEGER | FK → `tracks` |
| `station_id` | INTEGER | FK → `stations` |
| `played_at` | TIMESTAMPTZ | Exact broadcast timestamp |
| `played_duration` | REAL | Duration actually played in seconds (may differ from `tracks.duration` if cut short) |

---

## Users

### `users`
System accounts. `role` controls access level (`0` = viewer, `1` = admin). Passwords are stored as bcrypt hashes (via pgcrypto). Inactive accounts (`active = false`) cannot log in.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `login` | VARCHAR(32) | Unique login name |
| `password_hash` | VARCHAR(255) | bcrypt hash (pgcrypto `crypt()`) |
| `active` | BOOLEAN | Account enabled (default `false`) |
| `role` | SMALLINT | Permission level |

---

### `user_actions`
Audit log of all user actions in the system. Every significant operation is recorded here with the acting user and a timestamp. The `action` field contains a short code or description of the operation performed.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `user_id` | INTEGER | FK → `users` |
| `action` | VARCHAR(64) | Action performed |
| `performed_at` | TIMESTAMPTZ | Timestamp |

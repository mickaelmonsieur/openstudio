# OpenStudio

Professional broadcast radio software.

## Installation

### macOS

Download the two DMG files from the [latest release](../../releases/latest):

- `OpenStudio_x.x.x_aarch64.dmg` — the audio player
- `OpenStudio Admin-x.x.x.dmg` — the admin interface *(only needed on the machine that manages the library)*

Open each DMG and drag the app to your Applications folder.

> **First launch warning:** macOS will display "unidentified developer" because the app is not notarized.
> Go to **System Settings → Privacy & Security → Open Anyway**.
>

**PostgreSQL 18 is required.** On macOS, use [Postgres.app](https://postgresapp.com/downloads.html) — it is a universal binary (Intel + Apple Silicon). The EnterpriseDB installer is x86-64 only and runs under Rosetta.

---

### Windows

Download the two installers from the [latest release](../../releases/latest):

- `OpenStudio_x.x.x_x64-setup.exe` — the audio player
- `OpenStudio Admin Setup x.x.x.exe` — the admin interface *(only needed on the machine that manages the library)*

Run each installer. Windows may show a SmartScreen warning for unsigned apps — click **More info → Run anyway**.

**PostgreSQL 18 (x86-64) is required.** Download and install it from [enterprisedb.com](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads).

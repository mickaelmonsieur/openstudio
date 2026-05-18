# OpenStudio

Professional broadcast radio software.

[![Build Status](https://github.com/mickaelmonsieur/OpenStudio/actions/workflows/release.yml/badge.svg)](https://github.com/mickaelmonsieur/OpenStudio/actions/workflows/release.yml)
[![Latest Stable Release](https://img.shields.io/github/v/release/mickaelmonsieur/OpenStudio?label=latest%20stable%20release&sort=semver)](https://github.com/mickaelmonsieur/OpenStudio/releases/latest)
[![GPL-3.0 License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENCE)
[![Open Source](https://img.shields.io/badge/open%20source-GPL--3.0-brightgreen.svg)](LICENCE)
[![Ko-fi](https://img.shields.io/badge/ko--fi-support-ff5e5b?logo=kofi&logoColor=white)](https://ko-fi.com/Y8Y5MXCW)

## Screenshots

Player:

![Player](docs/images/player.png)

Admin in Web UI:

![Admin in Web UI](docs/images/admin-playlists.png)

## Platform Support

OpenStudio is developed primarily for macOS. The Windows version is expected to follow the macOS version, but it may need extra testing on real broadcast setups. If you hit a Windows-specific problem, please open a [GitHub Issue](https://github.com/mickaelmonsieur/OpenStudio/issues) or send a [Pull Request](https://github.com/mickaelmonsieur/OpenStudio/pulls).

## Installation

Install the components in this order, regardless of your operating system:

1. Install PostgreSQL.
2. Install OpenStudio.
3. Initialize the database from OpenStudio.
4. Install OpenStudio Admin.

OpenStudio must be installed and used to initialize the database before installing or launching OpenStudio Admin.

## Default Seed Users

The seeded database includes these default application users:

| Role | Login | Password |
| --- | --- | --- |
| Admin | `admin` | `admin123` |
| Manager | `manager` | `changeme123` |
| User | `user` | `user` |

Admin rights are required to modify the OpenStudio application configuration. Database Settings are the exception: they can be opened and changed without an application login or password.

### macOS

Download the two DMG files from the [latest release](../../releases/latest):

- `OpenStudio_x.x.x_aarch64.dmg` — the audio player
- `OpenStudio Admin-x.x.x.dmg` — the admin interface *(only needed on the machine that manages the library)*

Open the OpenStudio DMG and drag the app to your Applications folder. Launch OpenStudio, open **Database Settings**, then create or initialize the database.

After the database is ready, open the OpenStudio Admin DMG and drag the app to your Applications folder.

> **First launch warning:** macOS will display "unidentified developer" because the app is not notarized.
> Go to **System Settings → Privacy & Security → Open Anyway**.
>

**PostgreSQL 18 is required.** On macOS, use [Postgres.app](https://postgresapp.com/downloads.html) — it is a universal binary (Intel + Apple Silicon). The EnterpriseDB installer is x86-64 only and runs under Rosetta.

---

### Windows

Download the two installers from the [latest release](../../releases/latest):

- `OpenStudio_x.x.x_x64-setup.exe` — the audio player
- `OpenStudio Admin Setup x.x.x.exe` — the admin interface *(only needed on the machine that manages the library)*

Run the OpenStudio installer first. Launch OpenStudio, open **Database Settings**, then create or initialize the database.

After the database is ready, run the OpenStudio Admin installer. Windows may show a SmartScreen warning for unsigned apps — click **More info → Run anyway**.

**PostgreSQL 18 (x86-64) is required.** Download and install it from [enterprisedb.com](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads).

---

## 💼 Professional support

I'm available for consulting for radio stations interested in deploying OpenStudio in production.

👉 [mickael.be](https://www.mickael.be)

---

## ☕ Buy me a coffee

If OpenStudio helps your radio workflow, consider buying me a coffee!

[![ko-fi](https://www.ko-fi.com/img/donate_sm.png)](https://ko-fi.com/Y8Y5MXCW)

---

## 📄 Licence

GNU General Public License v3.0

https://www.gnu.org/licenses/gpl-3.0.en.html

See [LICENCE](LICENCE).

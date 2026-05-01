# CLAUDE.md — Lecteur FLAC Rust / iced

## Stack

- **Langage** : Rust (edition 2021)
- **GUI** : `iced` (architecture Elm / MVU)
- **Décodage audio** : `symphonia` (feature `flac` uniquement)
- **Sortie audio** : `cpal`
- **Base de données** : `sqlx` (PostgreSQL, runtime tokio)
- **Async** : `tokio`

```toml
[dependencies]
iced       = "0.13"
symphonia  = { version = "0.5", default-features = false, features = ["flac"] }
cpal       = "0.15"
sqlx       = { version = "0.7", features = ["postgres", "runtime-tokio", "macros"] }
tokio      = { version = "1", features = ["full"] }
```

---

## Architecture MVU (iced)

Respecter strictement le pattern Elm :

```
struct App { /* state */ }

#[derive(Debug, Clone)]
enum Message { /* events */ }

impl Application for App {
    fn update(&mut self, message: Message) -> Command<Message> { ... }
    fn view(&self) -> Element<Message> { ... }
}
```

- **Pas de mutation d'état hors de `update()`**
- Les appels async (DB, I/O fichier) passent par `Command::perform()`
- Un seul `Message` enum, variantes explicites

---

## Audio

- Décodage FLAC via `symphonia` : utiliser `SymphoniaDecoder`, ne pas instancier de probe inutile
- Sortie via `cpal` : thread audio séparé, communication avec le thread UI par `std::sync::mpsc`
- **Ne jamais bloquer le thread UI** — tout I/O audio est asynchrone ou sur thread dédié
- Resampling uniquement si la carte son ne supporte pas le sample rate natif du fichier

---

## Base de données

- Connexion via `PgPool` (pool de connexions, jamais de connexion unique globale)
- Requêtes avec `sqlx::query!` ou `sqlx::query_as!` (vérification compile-time)
- Migrations dans `migrations/` (format `sqlx migrate`)
- Variables de connexion dans `.env` (jamais hardcodées) :

```
DATABASE_URL=postgres://user:password@localhost/flac_player
```

---

## Conventions code

- `cargo fmt` + `cargo clippy -- -D warnings` doivent passer sans erreur
- Pas de `unwrap()` hors tests — utiliser `?` ou `.expect("message explicite")`
- Modules : `ui/`, `audio/`, `db/`, `model/`
- Les types de domaine (`Track`, `Album`, `Playlist`) dans `model/`
- Pas de dépendance directe entre `audio/` et `ui/` — passer par des messages

---

## Build & Package macOS

```bash
cargo build --release
cargo packager --release
```

- Produit `target/release/OpenStudio.app` et `target/release/OpenStudio_0.1.0_aarch64.dmg`
- La signature ad-hoc est configurée dans `[package.metadata.packager.macos]` (`signing-identity = "-"`)
- La notarisation est skippée (pas de compte Apple Developer)
- Distribution : l'utilisateur doit autoriser dans Réglages système → Confidentialité et sécurité
- Via AirDrop : aucun dialogue (pas d'attribut quarantine)

---

## Ce qu'on n'utilise pas

- Pas de `rodio` (redondant avec cpal + symphonia)
- Pas de `egui` (choix arrêté sur iced)
- Pas de `claxon` ni `flac-sys` (symphonia suffit)
- Pas d'ORM (diesel, sea-orm) — sqlx brut uniquement
- Pas de Tauri, pas de WebView

# OpenStudio

Lecteur audio FLAC professionnel — Rust / iced.

## Build & Package (macOS)

### 1. Compiler

```bash
cargo build --release
```

### 2. Packager (app + dmg signés)

```bash
cargo packager --release
```

Produit dans `target/release/` :
- `OpenStudio.app` — application signée (ad-hoc)
- `OpenStudio_0.1.0_aarch64.dmg` — image disque signée avec raccourci Applications

La signature ad-hoc est configurée dans `Cargo.toml` (`[package.metadata.packager.macos]`).  
La notarisation Apple est skippée (nécessite un compte Apple Developer à 99 €/an).

### 3. Distribution

Envoyer le `.dmg` au destinataire.  
À la première ouverture, macOS affiche "développeur non identifié" :  
→ **Réglages système → Confidentialité et sécurité → Ouvrir quand même**

> AirDrop évite ce dialogue (pas d'attribut quarantine).

# OpenStudio — Schéma de base de données

## Vue d'ensemble

Le schéma est organisé en couches : données de référence → médiathèque → programmation → diffusion → utilisateurs.

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

## Tables de référence

### `formats`
Les formats de programmation disponibles sur la station. Définit le style général d'antenne auquel un template appartient.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(32) | Nom du format (ex. `SEMAINE`, `WEEKEND-80`, `PUB`) |

Valeurs initiales : `PUB`, `TOP HORAIRE`, `SEMAINE`, `WEEKEND-80`.

---

### `categories`
Catégories de contenu de premier niveau. Chaque piste et chaque template appartient à une catégorie.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(32) | Nom de la catégorie |

Valeurs initiales : `Jingles`, `Music`, `Intervention`, `PubIn`, `PubOut`, `Filler`, `Top of Hour`, `Pub`.

---

### `subcategories`
Classification de second niveau à l'intérieur d'une catégorie. Par exemple, la catégorie `Music` contient des sous-catégories comme `FR-1980`, `PowerPlay`, etc. Le flag `hidden` contrôle la visibilité dans l'interface.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `category_id` | INTEGER | FK → `categories` |
| `name` | VARCHAR(32) | Nom de la sous-catégorie |
| `hidden` | BOOLEAN | Masquer dans l'interface publique (défaut `false`) |

---

### `artists`
Registre des artistes. Chaque piste référence un artiste. `last_broadcast_at` est mis à jour à chaque diffusion d'une piste de cet artiste — utilisé pour appliquer la règle de protection artiste.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(64) | Nom de l'artiste (unique) |
| `last_broadcast_at` | TIMESTAMPTZ | Dernière diffusion d'une piste de cet artiste |

---

### `stations`
Stations radio gérées par le système. Permet de rattacher les campagnes publicitaires et les spots locaux à une antenne précise.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(64) | Nom de la station |

---

### `genres`
Référentiel des genres musicaux. Chaque piste peut optionnellement être liée à un genre. La liste est pré-remplie avec 337 genres standards (norme ID3 + extensions). Stocké en table plutôt qu'en texte libre sur `tracks` pour garantir la cohérence et permettre les regroupements par genre.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(50) | Nom du genre (unique) |

---

## Templates de programmation

### `templates`
Un template (anciennement « canvas ») définit un créneau de programmation : quelle catégorie/sous-catégorie de contenu diffuser, et combien de temps attendre avant de rediffuser la même piste ou le même artiste. Les templates sont référencés par la grille de diffusion et les événements horloge.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `format_id` | INTEGER | FK → `formats` |
| `category_id` | INTEGER | FK → `categories` — type de contenu à piocher |
| `subcategory_id` | INTEGER | FK → `subcategories` — filtre plus fin (optionnel) |
| `comment` | VARCHAR(64) | Étiquette libre (ex. `1ER DISQUE`, `RETOUR PUB`) |
| `track_protection` | INTEGER | Délai minimum en secondes avant de rediffuser la même piste |
| `artist_protection` | INTEGER | Délai minimum en secondes avant de rediffuser le même artiste |

---

## Médiathèque

### `tracks`
La médiathèque centrale. Chaque fichier audio (musique, jingle, pub, intervention…) possède une ligne ici. Le `subcategory_id` détermine dans quels créneaux de programmation la piste est éligible ; la catégorie parente se déduit de la sous-catégorie par jointure. `active = false` désactive une piste sans la supprimer.

Les métadonnées de lecture (`bpm`, `intro`, `fade_in`, `fade_out`) sont stockées ici pour que le lecteur puisse préparer les transitions sans relire le fichier au moment de la diffusion. `fade_out = NULL` signifie « pas de cue spécifique » ; l'automate utilise alors `duration` comme référence effective.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `artist_id` | INTEGER | FK → `artists` |
| `genre_id` | INTEGER | FK → `genres` (optionnel) |
| `title` | VARCHAR(64) | Titre de la piste |
| `album` | VARCHAR(64) | Nom de l'album |
| `year` | SMALLINT | Année de sortie |
| `duration` | REAL | Durée en secondes |
| `sample_rate` | INTEGER | Fréquence d'échantillonnage native (Hz, défaut 44100) |
| `bpm` | REAL | Tempo |
| `intro` | REAL | Durée de l'intro en secondes (avant l'entrée vocale) |
| `fade_in` | REAL | Durée du fondu entrant en secondes |
| `fade_out` | REAL | Cue optionnel du fondu sortant en secondes ; `NULL` signifie utiliser `duration` |
| `path` | TEXT | Chemin absolu vers le fichier audio |
| `subcategory_id` | INTEGER | FK → `subcategories` (catégorie déduite par jointure) |
| `active` | BOOLEAN | Piste activée pour la programmation |
| `created_at` | TIMESTAMPTZ | Date d'ajout |
| `updated_at` | TIMESTAMPTZ | Date de dernière modification |
| `last_played_at` | TIMESTAMPTZ | Dernière date de diffusion |

---

## Publicité

### `sectors`
Référentiel des secteurs d'activité. Permet de classifier les annonceurs et de regrouper les campagnes par segment de marché. Évite les incohérences de saisie libre.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(64) | Nom du secteur (unique) |

Valeurs initiales : Automotive, Construction & Real Estate, Consumer Goods & Retail, Education & Training, Energy & Utilities, Finance & Insurance, Food & Beverage, Government & Public Sector, Healthcare & Wellness, Hospitality & Tourism, Legal & Accounting, Manufacturing & Industry, Media & Communications, Non-profit & Associations, Pharmaceutical, Services & Consulting, Technology, Telecommunications, Transport & Logistics, Other.

---

### `advertisers`
Référentiel des annonceurs (mini-CRM). Créer un annonceur une seule fois évite les doublons et permet de retrouver toutes ses campagnes par une simple jointure.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `name` | VARCHAR(255) | Nom de l'entreprise (unique) |
| `sector_id` | INTEGER | FK → `sectors` |
| `address` | TEXT | Adresse de facturation |
| `vat_number` | VARCHAR(32) | Numéro de TVA / SIRET |
| `notes` | TEXT | Notes libres (historique relation, préférences) |
| `active` | BOOLEAN | Client actif (défaut `true`) |
| `client_since` | DATE | Date du premier contrat |

---

### `contacts`
Interlocuteurs chez un annonceur. Un annonceur peut avoir plusieurs contacts (décideur, comptable, responsable marketing…). `primary_contact` désigne l'interlocuteur principal.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `advertiser_id` | INTEGER | FK → `advertisers` |
| `name` | VARCHAR(128) | Nom complet du contact |
| `role` | VARCHAR(64) | Titre / fonction (ex. `Responsable marketing`) |
| `phone` | VARCHAR(32) | Numéro de téléphone |
| `email` | VARCHAR(128) | Adresse email |
| `primary_contact` | BOOLEAN | Interlocuteur principal (défaut `false`) |
| `notes` | TEXT | Notes libres sur cette personne |

---

### `campaigns`
Une campagne publicitaire lie un annonceur à un ou plusieurs spots pour une station donnée, sur une plage de dates. `broadcast_count` est incrémenté à chaque passage d'un spot de la campagne ; quand il atteint `total_broadcasts`, la campagne est considérée épuisée. `active = false` met la campagne en pause manuellement.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `advertiser_id` | INTEGER | FK → `advertisers` |
| `name` | VARCHAR(255) | Nom de la campagne |
| `total_broadcasts` | INTEGER | Nombre total de passages contractés |
| `broadcast_count` | INTEGER | Nombre de passages déjà effectués |
| `station_id` | INTEGER | FK → `stations` |
| `active` | BOOLEAN | Campagne active |
| `encoded_at` | TIMESTAMPTZ | Date d'enregistrement de la campagne |
| `start_date` | DATE | Date de début |
| `end_date` | DATE | Date de fin |
| `last_aired_at` | TIMESTAMPTZ | Dernier passage à l'antenne |

---

### `campaign_tracks`
Table de jointure entre une campagne et ses spots. `position` définit l'ordre de rotation — le lecteur enchaîne les spots 1 → 2 → 3 → … → 1. Une campagne avec un seul spot a simplement une ligne à la position 1.

`track_id` est unique sur l'ensemble de la table : un spot appartient à exactement une campagne. Cela rend la requête inverse non-ambiguë — à partir d'un `track_id` issu de `play_log`, une seule jointure sur `campaign_tracks` retrouve la campagne parente, dont on peut alors incrémenter `broadcast_count`.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `campaign_id` | INTEGER | FK → `campaigns` |
| `track_id` | INTEGER | FK → `tracks` (unique — un spot, une campagne) |
| `position` | SMALLINT | Ordre de rotation (unique par campagne) |

---

## Grilles

### `clock_events`
Définit les déclenchements fixes de l'horloge radio, comme les tops horaires et les écrans publicitaires. Chaque ligne spécifie une heure (`hour:minute:second`) et le template à utiliser pour ce créneau. Le champ `duration` donne la durée prévue de l'événement en secondes.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `hour` | SMALLINT | Heure du déclenchement (0–23) |
| `minute` | SMALLINT | Minute du déclenchement (0–59) |
| `second` | SMALLINT | Seconde du déclenchement (0–59) |
| `template_id` | INTEGER | FK → `templates` — contenu à diffuser sur ce créneau |
| `priority` | SMALLINT | Priorité en cas de chevauchement |
| `duration` | REAL | Durée prévue de l'écran en secondes |

---

### `schedules`
Associe des plages horaires et des jours de la semaine à un template de programmation. Le planificateur consulte cette table pour déterminer quel template régit la sélection musicale à chaque instant de la semaine. Plusieurs lignes peuvent coexister pour des créneaux ou des combinaisons de jours différents.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `from_hour` | SMALLINT | Heure de début (0–23) |
| `to_hour` | SMALLINT | Heure de fin (0–23) |
| `monday` … `sunday` | BOOLEAN | Jours d'application |
| `template_id` | INTEGER | FK → `templates` — template de programmation à appliquer (obligatoire) |

---

## Files de lecture

### `queue`
La file de lecture en direct. File de lecture unifiée — contient tout : musiques, jingles, publicités. Le moteur consomme les entrées dans l'ordre de `scheduled_at`, en départageant les conflits par `priority`. `played` passe à `true` dès que la piste démarre.

`fixed_time = true` signifie que l'entrée doit partir exactement à `scheduled_at` (ex. jingle de top horaire) ; `fixed_time = false` autorise le moteur à décaler légèrement l'entrée pour absorber les transitions.

Les métadonnées audio (`bpm`, `intro`, `fade_in`, `fade_out`) sont copiées depuis `tracks` lors de la planification automatique, mais peuvent différer de `tracks` lors d'une planification manuelle — permettant des ajustements ponctuels sans modifier la médiathèque. Quand `tracks.fade_out` vaut `NULL`, la génération de queue stocke `tracks.duration` comme `fade_out` effectif.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `track_id` | INTEGER | FK → `tracks` |
| `sample_rate` | INTEGER | Fréquence d'échantillonnage à la lecture |
| `bpm` | REAL | Tempo |
| `intro` | REAL | Durée de l'intro (secondes) |
| `fade_in` | REAL | Durée du fondu entrant (secondes) |
| `fade_out` | REAL | Décalage du fondu sortant (secondes) |
| `played` | BOOLEAN | Piste démarrée |
| `priority` | SMALLINT | Priorité en cas de conflit d'heure (défaut 0, valeur plus haute = priorité plus haute) |
| `fixed_time` | BOOLEAN | Doit partir exactement à `scheduled_at`, sans glissement (défaut `false`) |
| `scheduled_at` | TIMESTAMPTZ | Heure prévue de lecture |
| `created_at` | TIMESTAMPTZ | Date d'ajout à la file |
| `updated_at` | TIMESTAMPTZ | Dernière modification |

---

## Journaux

### `play_log`
Enregistrement immuable de tout ce qui a été diffusé. Une ligne par événement de diffusion. Utilisé pour le reporting des droits d'auteur (SACEM/SABAM) et l'historique des passages.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `track_id` | INTEGER | FK → `tracks` |
| `station_id` | INTEGER | FK → `stations` |
| `played_at` | TIMESTAMPTZ | Horodatage exact de la diffusion |
| `played_duration` | REAL | Durée réellement jouée en secondes (peut différer de `tracks.duration` si coupé) |

---

## Utilisateurs

### `users`
Comptes système. `role` contrôle le niveau d'accès (`0` = consultation, `1` = admin). Les mots de passe sont stockés sous forme de hash bcrypt (via pgcrypto). Les comptes inactifs (`active = false`) ne peuvent pas se connecter.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `login` | VARCHAR(32) | Identifiant unique |
| `password_hash` | VARCHAR(255) | Hash bcrypt (pgcrypto `crypt()`) |
| `active` | BOOLEAN | Compte activé (défaut `false`) |
| `role` | SMALLINT | Niveau de permission |

---

### `user_actions`
Journal d'audit de toutes les actions utilisateurs dans le système. Chaque opération significative est enregistrée ici avec l'utilisateur concerné et un horodatage. Le champ `action` contient un code court ou une description de l'opération effectuée.

| Colonne | Type | Description |
|---------|------|-------------|
| `id` | INTEGER | Clé primaire |
| `user_id` | INTEGER | FK → `users` |
| `action` | VARCHAR(64) | Action effectuée |
| `performed_at` | TIMESTAMPTZ | Horodatage |

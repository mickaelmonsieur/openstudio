
INSERT INTO templates (id, name) VALUES
    (1, 'PUB'),
    (2, 'TOP HORAIRE'),
    (3, 'SEMAINE'),
    (4, 'HIT ONLY');
SELECT setval('templates_id_seq', 4);

INSERT INTO stations (id, name, library_path) VALUES
    (1, 'DEMO', '/Users/Shared/OpenStudio/Library/demo');
SELECT setval('stations_id_seq', 1);

INSERT INTO artists (id, name, last_broadcast_at) VALUES
    (1, 'Mylène Farmer', NULL),
    (2, 'ABC', NULL),
    (3, 'Taylor Swift', NULL),
    (4, 'Texas', NULL),
    (5, 'Madonna', NULL),
    (6, 'Melanie C', NULL),
    (7, 'Roxette', NULL),
    (8, 'The Cure', NULL),
    (9, 'Radio Contact', NULL);
SELECT setval('artists_id_seq', 9);

INSERT INTO tracks (
    id, artist_id, genre_id, title, album, year, duration, sample_rate,
    cue_in, cue_out, intro, outro, hook_in, hook_out, loop_in, loop_out,
    path, subcategory_id, active
) VALUES
    (1,  1,  99, 'XXL',                         'Anamorphosee',        1995, 260.38858, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Mylène Farmer - XXL.flac',                                      19, TRUE),
    (2,  2, 333, 'The Look Of Love, Pt.1',      'The Lexicon Of Love', 1982, 209.53334, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/ABC - The Look Of Love, Pt.1.flac',                              19, TRUE),
    (3,  3,  99, 'Cruel Summer',                'Lover',               2019, 178.42667, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Taylor Swift - Cruel Summer.flac',                               19, TRUE),
    (4,  4,  99, 'Getaway',                     'Red Book',            2005, 233.64000, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Texas - Getaway.flac',                                           19, TRUE),
    (5,  5,  99, 'Frozen',                      'Ray Of Light',        1998, 367.33334, 44100, 18, NULL,     0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Madonna - Frozen.flac',                                          19, TRUE),
    (6,  6,  99, 'Never Be The Same Again',     'Northern Star',       2000, 294.20000, 44100, 0, NULL,     25, 28, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Melanie C - Never Be The Same Again.flac',                       19, TRUE),
    (7,  7,  99, 'The Look',                    'Look Sharp!',         1988, 237.32000, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/Roxette - The Look.flac',                                        19, TRUE),
    (8,  8, 332, 'Lullaby',                     'Disintegration',      1989, 248.97333, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/DECK0/The Cure - Lullaby.flac',                                        19, TRUE),
    (9,  9, NULL, 'Avec elle profitons du w-e', '',                    2000,  20.05005, 44100, 0, 15.249319, 0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - Avec elle profitons du w-e.flac',         2, TRUE),
    (10, 9, NULL, 'C''est le w-e quel bonheur', '',                    2000,  14.38147, 44100, 0, 10.631020, 0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - C''est le w-e quel bonheur.flac',          2, TRUE),
    (11, 9, NULL, 'Laissons nous vivre, c''est le w-e', '',            2000,  18.16923, 44100, 0, 13.315396, 0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - Laissons nous vivre, c''est le w-e.flac',  2, TRUE),
    (12, 9, NULL, 'Le w-e, avec elle, je me sens bien', '',            2000,  16.73249, 44100, 0, NULL,      0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - Le w-e, avec elle, je me sens bien.flac', 2, TRUE),
    (13, 9, NULL, 'Quel bonheur, c''est le w-e', '',                   2000,  12.73576, 44100, 0,  8.905600, 0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - Quel bonheur, c''est le w-e.flac',         2, TRUE),
    (14, 9, NULL, 'Vive le w-e, vive la musique', '',                  2000,  16.54964, 44100, 0, 11.080453, 0,  0, 0, 0, 0, 0, '/Users/Shared/OpenStudio/Library/demo/JINGLES/RADIO CONTACT - Vive le w-e, vive la musique.flac',        2, TRUE);
SELECT setval('tracks_id_seq', 14);


-- Table: template_slots (62 rows)
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (4, 3, 1, 2, 12, '1ER DISQUE', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (5, 3, 2, 2, 13, '2EME DISQUE (Annees 2000)', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (6, 3, 3, 1, 14, 'RETOUR PUB', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (7, 3, 4, 2, 15, '1ER DISQUE', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (8, 3, 5, 2, 16, 'SOUVENIR1', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (9, 3, 6, 1, 17, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (10, 3, 7, 2, 18, 'SOUVENIR2', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (11, 3, 8, 1, 19, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (12, 3, 9, 2, 20, '20 MINUTES', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (13, 3, 10, 1, 21, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (14, 3, 11, 2, 12, 'SEUL AVANT PUB (Annee 2010)', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (15, 3, 12, 1, 13, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (16, 3, 13, 2, 14, '31 MINUTES', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (17, 3, 14, 1, 15, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (18, 3, 15, 2, 16, 'SOUVENIR1', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (19, 3, 16, 1, 17, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (20, 3, 17, 2, 18, 'SOUVENIR2', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (21, 3, 18, 1, 19, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (22, 3, 19, 2, 20, '41 MINUTES', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (23, 3, 20, 1, 21, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (24, 3, 21, 2, 12, 'SOUVENIR1', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (25, 3, 22, 1, 13, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (26, 3, 23, 2, 14, 'SOUVENIR2', 9000, 600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (27, 3, 24, 1, 15, '', 600, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (28, 3, 25, 2, 16, 'FIN HEURE', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (29, 3, 26, 2, 17, 'CD SECOURS', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (30, 3, 27, 2, 18, 'CD SECOURS', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (31, 3, 28, 2, 19, 'CD SECOURS', 9000, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (32, 2, 1, 7, NULL, 'Top Horaire', 0, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (33, 1, 1, 4, NULL, 'Pub In', 0, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (34, 1, 2, 8, NULL, 'ECRAN PUB', 0, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (35, 1, 3, 5, NULL, 'Pub Out', 0, 0);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (36, 4, 1, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (37, 4, 30, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (38, 4, 29, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (39, 4, 28, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (40, 4, 27, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (41, 4, 26, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (42, 4, 25, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (43, 4, 24, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (44, 4, 23, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (45, 4, 22, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (46, 4, 21, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (47, 4, 20, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (48, 4, 19, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (49, 4, 18, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (50, 4, 17, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (51, 4, 16, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (52, 4, 15, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (53, 4, 14, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (54, 4, 13, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (55, 4, 12, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (56, 4, 11, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (57, 4, 10, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (58, 4, 9, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (59, 4, 8, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (60, 4, 7, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (61, 4, 6, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (62, 4, 5, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (63, 4, 4, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (64, 4, 3, 2, 21, '', 3600, 3600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (65, 4, 2, 2, 21, '', 3600, 3600);
SELECT setval(pg_get_serial_sequence('"template_slots"', 'id'), MAX(id)) FROM "template_slots";

-- event_type=2: recurring every day (days_mask=127) at a specific hour — one row per hour for TOP HORAIRE
INSERT INTO clock_events (id, name, event_type, days_mask, hours_mask, hour, minute, second, priority, duration, is_fixed) VALUES
    ( 1, 'TOP HORAIRE', 2, 127, 0,  0, 0, 0, 0, 10, TRUE),
    ( 2, 'TOP HORAIRE', 2, 127, 0,  1, 0, 0, 0, 10, TRUE),
    ( 3, 'TOP HORAIRE', 2, 127, 0,  2, 0, 0, 0, 10, TRUE),
    ( 4, 'TOP HORAIRE', 2, 127, 0,  3, 0, 0, 0, 10, TRUE),
    ( 5, 'TOP HORAIRE', 2, 127, 0,  4, 0, 0, 0, 10, TRUE),
    ( 6, 'TOP HORAIRE', 2, 127, 0,  5, 0, 0, 0, 10, TRUE),
    ( 7, 'TOP HORAIRE', 2, 127, 0,  6, 0, 0, 0, 10, TRUE),
    ( 8, 'TOP HORAIRE', 2, 127, 0,  7, 0, 0, 0, 10, TRUE),
    ( 9, 'TOP HORAIRE', 2, 127, 0,  8, 0, 0, 0, 10, TRUE),
    (10, 'TOP HORAIRE', 2, 127, 0,  9, 0, 0, 0, 10, TRUE),
    (11, 'TOP HORAIRE', 2, 127, 0, 10, 0, 0, 0, 10, TRUE),
    (12, 'TOP HORAIRE', 2, 127, 0, 11, 0, 0, 0, 10, TRUE),
    (13, 'TOP HORAIRE', 2, 127, 0, 12, 0, 0, 0, 10, TRUE),
    (14, 'TOP HORAIRE', 2, 127, 0, 13, 0, 0, 0, 10, TRUE),
    (15, 'TOP HORAIRE', 2, 127, 0, 14, 0, 0, 0, 10, TRUE),
    (16, 'TOP HORAIRE', 2, 127, 0, 15, 0, 0, 0, 10, TRUE),
    (17, 'TOP HORAIRE', 2, 127, 0, 16, 0, 0, 0, 10, TRUE),
    (18, 'TOP HORAIRE', 2, 127, 0, 17, 0, 0, 0, 10, TRUE),
    (19, 'TOP HORAIRE', 2, 127, 0, 18, 0, 0, 0, 10, TRUE),
    (20, 'TOP HORAIRE', 2, 127, 0, 19, 0, 0, 0, 10, TRUE),
    (21, 'TOP HORAIRE', 2, 127, 0, 20, 0, 0, 0, 10, TRUE),
    (22, 'TOP HORAIRE', 2, 127, 0, 21, 0, 0, 0, 10, TRUE),
    (23, 'TOP HORAIRE', 2, 127, 0, 22, 0, 0, 0, 10, TRUE),
    (24, 'TOP HORAIRE', 2, 127, 0, 23, 0, 0, 0, 10, TRUE);
-- event_type=3: recurring every day at every hour (hours_mask=16777215) — PUB slots
INSERT INTO clock_events (id, name, event_type, days_mask, hours_mask, hour, minute, second, priority, duration, is_fixed) VALUES
    (25, 'PUB', 3, 127, 16777215, 0,  5,  0, 0, 240, FALSE),
    (26, 'PUB', 3, 127, 16777215, 0, 27,  0, 0, 240, FALSE),
    (27, 'PUB', 3, 127, 16777215, 0, 47,  0, 0, 240, FALSE);
SELECT setval('clock_events_id_seq', 27);

-- action_type=1: Template
INSERT INTO event_actions (id, event_id, action_type, template_id, track_id) VALUES
    ( 1,  1, 1, 2, NULL),
    ( 2,  2, 1, 2, NULL),
    ( 3,  3, 1, 2, NULL),
    ( 4,  4, 1, 2, NULL),
    ( 5,  5, 1, 2, NULL),
    ( 6,  6, 1, 2, NULL),
    ( 7,  7, 1, 2, NULL),
    ( 8,  8, 1, 2, NULL),
    ( 9,  9, 1, 2, NULL),
    (10, 10, 1, 2, NULL),
    (11, 11, 1, 2, NULL),
    (12, 12, 1, 2, NULL),
    (13, 13, 1, 2, NULL),
    (14, 14, 1, 2, NULL),
    (15, 15, 1, 2, NULL),
    (16, 16, 1, 2, NULL),
    (17, 17, 1, 2, NULL),
    (18, 18, 1, 2, NULL),
    (19, 19, 1, 2, NULL),
    (20, 20, 1, 2, NULL),
    (21, 21, 1, 2, NULL),
    (22, 22, 1, 2, NULL),
    (23, 23, 1, 2, NULL),
    (24, 24, 1, 2, NULL),
    (25, 25, 1, 1, NULL),
    (26, 26, 1, 1, NULL),
    (27, 27, 1, 1, NULL);
SELECT setval('event_actions_id_seq', 27);

INSERT INTO schedules (id, from_hour, to_hour, days_mask, template_id) VALUES
    (1, 0, 23, 127, 4);
SELECT setval('schedules_id_seq', 1);

INSERT INTO "advertisers" ("id", "name", "sector_id", "address", "vat_number", "notes", "active", "client_since") VALUES
(1,	'CARRELAGES PIRARD',	2,	'Rue du Travail 1, 4460 Grâce-Hollogne',	NULL,	'FAKE CUSTOMER',	'1',	'2000-01-01');

INSERT INTO "contacts" ("id", "advertiser_id", "name", "role", "phone", "email", "primary_contact", "notes") VALUES
(1,	1,	'Monsieur Dracula',	'Manager',	'+32475151230',	'hello@pirard.local',	'1',	NULL);

INSERT INTO "campaigns" ("id", "advertiser_id", "name", "total_broadcasts", "broadcast_count", "station_id", "active", "encoded_at", "start_date", "end_date", "last_aired_at") VALUES
(1,	1,	'HALLOWEEN 2026',	10000,	0,	1,	'1',	NULL,	'2026-01-01',	'2026-12-31',	NULL);
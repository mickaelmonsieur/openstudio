-- OpenStudio — seed data

CREATE EXTENSION IF NOT EXISTS pgcrypto;

INSERT INTO genres (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1,'Acapella'),(2,'Acid'),(3,'Acid Jazz'),(4,'Acid Punk'),(5,'Acoustic'),
    (6,'Alternative'),(7,'Alternative Rock'),(8,'Ambient'),(9,'Anime'),(10,'Avantgarde'),
    (11,'Ballad'),(12,'Bass'),(13,'Beat'),(14,'Bebob'),(15,'Big Band'),
    (16,'Black Metal'),(17,'Bluegrass'),(18,'Blues'),(19,'Booty Bass'),(20,'BritPop'),
    (21,'Cabaret'),(22,'Celtic'),(23,'Chamber Music'),(24,'Chanson'),(25,'Chorus'),
    (26,'Christian Gangsta Rap'),(27,'Christian Rap'),(28,'Christian Rock'),(29,'Classic Rock'),(30,'Classical'),
    (31,'Club'),(32,'Club - House'),(33,'Comedy'),(34,'Contemporary Christian'),(35,'Country'),
    (36,'Crossover'),(37,'Cult'),(38,'Dance'),(39,'Dance Hall'),(40,'Darkwave'),
    (41,'Death Metal'),(42,'Disco'),(43,'Dream'),(44,'Drum & Bass'),(45,'Drum Solo'),
    (46,'Duet'),(47,'Easy Listening'),(48,'Electronic'),(49,'Ethnic'),(50,'Euro-House'),
    (51,'Euro-Techno'),(52,'Eurodance'),(53,'Fast Fusion'),(54,'Folk'),(55,'Folk-Rock'),
    (56,'Folklore'),(57,'Freestyle'),(58,'Funk'),(59,'Fusion'),(60,'Game'),
    (61,'Gangsta'),(62,'Goa'),(63,'Gospel'),(64,'Gothic'),(65,'Gothic Rock'),
    (66,'Grunge'),(67,'Hard Rock'),(68,'Hardcore'),(69,'Heavy Metal'),(70,'Hip-Hop'),
    (71,'House'),(72,'Humour'),(73,'Indie'),(74,'Industrial'),(75,'Instrumental'),
    (76,'Instrumental Pop'),(77,'Instrumental Rock'),(78,'JPop'),(79,'Jazz'),(80,'Jazz+Funk'),
    (81,'Jungle'),(82,'Latin'),(83,'Lo-Fi'),(84,'Meditative'),(85,'Merengue'),
    (86,'Metal'),(87,'Musical'),(88,'National Folk'),(89,'Native US'),(90,'Negerpunk'),
    (91,'New Age'),(92,'New Wave'),(93,'Noise'),(94,'Oldies'),(95,'Opera'),
    (96,'Other'),(97,'Polka'),(98,'Polsk Punk'),(99,'Pop'),(100,'Pop-Folk'),
    (101,'Pop/Funk'),(102,'Porn Groove'),(103,'Power Ballad'),(104,'Pranks'),(105,'Primus'),
    (106,'Progressive Rock'),(107,'Psychadelic'),(108,'Psychedelic Rock'),(109,'Punk'),(110,'Punk Rock'),
    (111,'R&B'),(112,'Rap'),(113,'Rave'),(114,'Reggae'),(115,'Retro'),
    (116,'Revival'),(117,'Rhythmic Soul'),(118,'Rock'),(119,'Rock & Roll'),(120,'Salsa'),
    (121,'Samba'),(122,'Satire'),(123,'Showtunes'),(124,'Ska'),(125,'Slow Jam'),
    (126,'Slow Rock'),(127,'Sonata'),(128,'Soul'),(129,'Sound Clip'),(130,'Soundtrack'),
    (131,'Southern Rock'),(132,'Space'),(133,'Speech'),(134,'Swing'),(135,'Symphonic Rock'),
    (136,'Symphony'),(137,'Synthpop'),(138,'Tango'),(139,'Techno'),(140,'Techno-Industrial'),
    (141,'Terror'),(142,'Thrash Metal'),(143,'Top 40'),(144,'Trailer'),(145,'Trance'),
    (146,'Tribal'),(147,'Trip-Hop'),(148,'Vocal'),(149,'General Pop Vocal'),(150,'Teen Boy Band'),
    (151,'Dance Pop'),(152,'Unknown'),(153,'Other Alternative Hip-Hop; Rap'),(154,'Pop Soul'),(155,'Alternative Pop'),
    (156,'Classic Hard Rock'),(157,'Teen Rock'),(158,'Alternative Pop Singer-Songwriter'),(159,'General Alternative Rock'),(160,'Latin House'),
    (161,'Idol Pop Vocals'),(162,'General Hip-Hop; Rap'),(163,'Contemporary R&B'),(164,'Urban AC'),(165,'General Pop'),
    (166,'Euro Pop'),(167,'French Pop'),(168,'Inconnu'),(169,'Downtempo'),(170,'Pop; rock'),
    (171,'Variété française'),(172,'Caribbean Pop'),(173,'General Club Dance'),(174,'Ambient Trance'),(175,'Urban Crossover'),
    (176,'Garage Rock Revival'),(177,'Kids'),(178,'R&B; Soul'),(179,'General Folk Rock'),(180,'Tech House'),
    (181,'Data & Other'),(182,'General Mainstream Rock'),(183,'French Hip-Hop; Rap'),(184,'General World'),(185,'Hip Hop; Rap'),
    (186,'Post-Punk Revival'),(187,'Euro House'),(188,'General Teen Pop'),(189,'Levensleid'),(190,'Adult Alternative Rock'),
    (191,'Française'),(192,'Electroklash'),(193,'Pop Female Singer-Songwriter'),(194,'Traditional U.S. Folk'),(195,'American Trad. Rock'),
    (196,'Chill Out'),(197,'Autres'),(198,'Sports Themes'),(199,'Brit Rock'),(200,'Neo-Soul'),
    (201,'RnB'),(202,'Rap Metal'),(203,'Post-Modern Electronic Pop'),(204,'Indie Dance'),(205,'Noise Pop'),
    (206,'Pop Electronica'),(207,'Pop Male Singer-Songwriter'),(208,'Club-House'),(209,'General Latin Pop'),(210,'groove'),
    (211,'General Indie Pop'),(212,'Electro'),(213,'Reggaetón'),(214,'Italian Pop'),(215,'East Coast Rap'),
    (216,'Rap français'),(217,'Ambient Electronica'),(218,'Hip-Hop; Rap'),(219,'Funky House'),(220,'Power Pop'),
    (221,'General Progressive House'),(222,'Adult Alternative Pop'),(223,'Emo'),(224,'world music'),(225,'Pop Punk'),
    (226,'Alternative Rap-Rock'),(227,'Variiti Frangaise'),(228,'General Easy Listening'),(229,'Audio Book: Children & Young Adult'),(230,'Other Reggae'),
    (231,'Slow'),(232,'Rock; Pop'),(233,'General Punk'),(234,'General Latin Rock'),(235,'General House'),
    (236,'Hip-Hop | Rap | R&B'),(237,'Classic Prog'),(238,'Latino'),(239,'Raggae'),(240,'Mento'),
    (241,'Electronica'),(242,'Conscious Hip-Hop; Rap'),(243,'AlternRock'),(244,'AlternRock Alt. Rock'),(245,'Synth Pop'),
    (246,'Soca'),(247,'Rockabilly Revival'),(248,'R & B'),(249,'African Hip-Hop; Rap'),(250,'Iles, Antilles'),
    (251,'Alt. Rock'),(252,'General Dream Pop'),(253,'Tech-house'),(254,'Tribal House'),(255,'New Wave Quirk'),
    (256,'Northeast African'),(257,'General Indie Rock'),(258,'General Latin Hip-Hop; Rap'),(259,'Christian R&B'),(260,'Classic House'),
    (261,'NEW'),(262,'Neo-Psychedelic'),(263,'Classic Pop-Rock'),(264,'Trip Hop'),(265,'Stage Musicals'),
    (266,'General Hard Rock'),(267,'Rap Reggae'),(268,'Folktronica'),(269,'Post-Grunge Alternative Rock'),(270,'Pre-Grunge Alternative Rock'),
    (271,'Soft Jazz Vocals'),(272,'genre'),(273,'General Film Music'),(274,'Chanson française'),(275,'Corsican Polyphony'),
    (276,'Southern Rap'),(277,'Brithop'),(278,'Chanson Rock'),(279,'Acoustic Pop'),(280,'Black Music'),
    (281,'Art & Synth Punk'),(282,'rap et hip hop'),(283,'General Rap; Hip-Hop'),(284,'Toasting'),(285,'Alternative Dance'),
    (286,'ELECTRO HOUSE'),(287,'Zouk'),(288,'Trance Pop'),(289,'Drum ''n'' Bass'),(290,'Pop rock'),
    (291,'Electronica; Dance'),(292,'General Techno'),(293,'Rap & Hip-Hop'),(294,'Teen Girl Group'),(295,'Rap Hip-Hop'),
    (296,'Contemporary U.S. Folk'),(297,'Brit Pop'),(298,'General Country'),(299,'Rock Singer-Songwriter'),(300,'Top100'),
    (301,'Stoner Rock'),(302,'Worldbeat'),(303,'Folk Pop'),(304,'Underground Rock'),(305,'Malian'),
    (306,'Colombian'),(307,'U.K. Garage'),(308,'Turkish Pop'),(309,'Ethno-Lounge Electronica'),(310,'Turntablism'),
    (311,'Dance R&B'),(312,'Afrikaans'),(313,'Grime'),(314,'New Wave Pop'),(315,'General Children''s Music'),
    (316,'Pop Jazz'),(317,'Pop Metal'),(318,'J-Pop'),(319,'Dutch Pop'),(320,'General Spoken'),
    (321,'MPB'),(322,'Art Rock'),(323,'Contemporary Era Solo Instrumental'),(324,'Audio Book: Mystery & Thrillers'),(325,'Celtic Rock'),
    (326,'AOR Classic Rock'),(327,'Rock Oldies'),(328,'Pop Standards'),(329,'Jangle Pop'),(330,'Pop Reggae'),
    (331,'Unclassifiable'),(332,'General Post-Punk'),(333,'New Romantic'),(334,'General Blues'),(335,'Modern Jazz'),
    (336,'80s Dance'),(337,'Autre');
SELECT setval('genres_id_seq', 337);

INSERT INTO sectors (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1,  'Automotive'),
    (2,  'Construction & Real Estate'),
    (3,  'Consumer Goods & Retail'),
    (4,  'Education & Training'),
    (5,  'Energy & Utilities'),
    (6,  'Finance & Insurance'),
    (7,  'Food & Beverage'),
    (8,  'Government & Public Sector'),
    (9,  'Healthcare & Wellness'),
    (10, 'Hospitality & Tourism'),
    (11, 'Legal & Accounting'),
    (12, 'Manufacturing & Industry'),
    (13, 'Media & Communications'),
    (14, 'Non-profit & Associations'),
    (15, 'Pharmaceutical'),
    (16, 'Services & Consulting'),
    (17, 'Technology'),
    (18, 'Telecommunications'),
    (19, 'Transport & Logistics'),
    (20, 'Other');
SELECT setval('sectors_id_seq', 20);

INSERT INTO formats (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1, 'PUB'),
    (2, 'TOP HORAIRE'),
    (3, 'SEMAINE'),
    (4, 'WEEKEND-80');
SELECT setval('formats_id_seq', 4);

INSERT INTO stations (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1, 'DEMO');
SELECT setval('stations_id_seq', 1);

INSERT INTO categories (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1, 'Jingles'),
    (2, 'Music'),
    (3, 'Intervention'),
    (4, 'PubIn'),
    (5, 'PubOut'),
    (6, 'Filler'),
    (7, 'Top of Hour'),
    (8, 'Pub');
SELECT setval('categories_id_seq', 8);

INSERT INTO subcategories (id, category_id, name, hidden) OVERRIDING SYSTEM VALUE VALUES
    (1,  1, 'Jingles',    TRUE),
    (2,  1, 'Jin.W-E',    TRUE),
    (3,  1, 'Jin.Ete',    TRUE),
    (4,  1, 'Jin.Hiver',  TRUE),
    (5,  1, 'Accaps',     TRUE),
    (6,  1, 'Tapis',      TRUE),
    (7,  1, 'Promos',     TRUE),
    (8,  1, 'Hitmix',     TRUE),
    (9,  1, 'Liners',     FALSE),
    (10, 1, 'Divers',     FALSE),
    (11, 2, 'PowerPlay',  FALSE),
    (12, 2, 'FR-1930',    FALSE),
    (13, 2, 'FR-1940',    FALSE),
    (14, 2, 'FR-1950',    FALSE),
    (15, 2, 'FR-1960',    FALSE),
    (16, 2, 'FR-1970',    FALSE),
    (17, 2, 'FR-1980',    FALSE),
    (18, 2, 'FR-1990',    FALSE),
    (19, 2, 'FR-2000',    FALSE),
    (20, 2, 'FR-2010',    FALSE),
    (21, 2, 'FR-2020',    FALSE);
SELECT setval('subcategories_id_seq', 21);

INSERT INTO artists (id, name) OVERRIDING SYSTEM VALUE VALUES
    (1, 'Mylène Farmer'),
    (2, 'ABC'),
    (3, 'Taylor Swift'),
    (4, 'Texas'),
    (5, 'Madonna'),
    (6, 'Melanie C'),
    (7, 'Roxette'),
    (8, 'The Cure');
SELECT setval('artists_id_seq', 8);

INSERT INTO tracks (
    id, artist_id, genre_id, title, album, year, duration, sample_rate,
    bpm, intro, fade_in, fade_out, path, subcategory_id, active
) OVERRIDING SYSTEM VALUE VALUES
    (1, 1,  99, 'XXL',                    'Anamorphosee',      1995, 260.388571, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Mylène Farmer - XXL.flac',                     19, TRUE),
    (2, 2, 333, 'The Look Of Love, Pt.1', 'The Lexicon Of Love',1982, 209.533333, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/ABC - The Look Of Love, Pt.1.flac',             19, TRUE),
    (3, 3,  99, 'Cruel Summer',           'Lover',             2019, 178.426667, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Taylor Swift - Cruel Summer.flac',              19, TRUE),
    (4, 4,  99, 'Getaway',                'Red Book',          2005, 233.640000, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Texas - Getaway.flac',                          19, TRUE),
    (5, 5,  99, 'Frozen',                 'Ray Of Light',      1998, 367.333333, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Madonna - Frozen.flac',                         19, TRUE),
    (6, 6,  99, 'Never Be The Same Again','Northern Star',     2000, 294.200000, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Melanie C - Never Be The Same Again.flac',      19, TRUE),
    (7, 7,  99, 'The Look',               'Look Sharp!',       1988, 237.320000, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/Roxette - The Look.flac',                       19, TRUE),
    (8, 8, 332, 'Lullaby',                'Disintegration',    1989, 248.973333, 44100, 0, 0, 0, NULL, '/Users/mickael/Music/The Cure - Lullaby.flac',                       19, TRUE);
SELECT setval('tracks_id_seq', 8);

WITH RECURSIVE seed_start (scheduled_at) AS (
    SELECT NOW()
),
seeded_queue (position, track_id, scheduled_at, next_scheduled_at) AS (
    SELECT
        0,
        1,
        s.scheduled_at,
        s.scheduled_at + (t.duration * INTERVAL '1 second')
    FROM seed_start s
    JOIN tracks t ON t.id = 1

    UNION ALL

    SELECT
        q.position + 1,
        ((q.position + 1) % 8) + 1,
        q.next_scheduled_at,
        q.next_scheduled_at + (t.duration * INTERVAL '1 second')
    FROM seeded_queue q
    JOIN tracks t ON t.id = ((q.position + 1) % 8) + 1
    JOIN seed_start s ON TRUE
    WHERE q.next_scheduled_at < s.scheduled_at + INTERVAL '2 hours'
)
INSERT INTO queue (track_id, sample_rate, bpm, intro, fade_in, fade_out, scheduled_at)
SELECT
    t.id,
    t.sample_rate,
    t.bpm,
    t.intro,
    t.fade_in,
    GREATEST(t.duration - 5, 0),
    q.scheduled_at
FROM seeded_queue q
JOIN tracks t ON t.id = q.track_id
ORDER BY q.position;

INSERT INTO templates (id, format_id, category_id, subcategory_id, comment, track_protection, artist_protection) OVERRIDING SYSTEM VALUE VALUES
    (1,   1, 8, NULL, 'PUB',                           600,    0),
    (2,   2, 7, NULL, 'TOP HORAIRE',                   600,    0),
    (3,   3, 2, NULL, 'SEMAINE',                      9000, 3600),
    (4,   3, 2,   12, '1ER DISQUE',                   9000, 3600),
    (5,   3, 2,   13, '2EME DISQUE (Annees 2000)',     9000, 3600),
    (6,   3, 1,   14, 'RETOUR PUB',                    600,    0),
    (7,   3, 2,   15, '1ER DISQUE',                   9000, 3600),
    (8,   3, 2,   16, 'SOUVENIR1',                    9000, 3600),
    (9,   3, 1,   17, '',                               600,    0),
    (10,  3, 2,   18, 'SOUVENIR2',                    9000, 3600),
    (11,  3, 1,   19, '',                               600,    0),
    (12,  3, 2,   20, '20 MINUTES',                   9000, 3600),
    (13,  3, 1,   21, '',                               600,    0),
    (14,  3, 2,   12, 'SEUL AVANT PUB (Annee 2010)',  9000, 3600),
    (15,  3, 1,   13, '',                               600,    0),
    (16,  3, 2,   14, '31 MINUTES',                   9000, 3600),
    (17,  3, 1,   15, '',                               600,    0),
    (18,  3, 2,   16, 'SOUVENIR1',                    9000, 3600),
    (19,  3, 1,   17, '',                               600,    0),
    (20,  3, 2,   18, 'SOUVENIR2',                    9000, 3600),
    (21,  3, 1,   19, '',                               600,    0),
    (22,  3, 2,   20, '41 MINUTES',                   9000, 3600),
    (23,  3, 1,   21, '',                               600,    0),
    (24,  3, 2,   12, 'SOUVENIR1',                    9000, 3600),
    (25,  3, 1,   13, '',                               600,    0),
    (26,  3, 2,   14, 'SOUVENIR2',                    9000,  600),
    (27,  3, 1,   15, '',                               600,    0),
    (28,  3, 2,   16, 'FIN HEURE',                    9000, 3600),
    (29,  3, 2,   17, 'CD SECOURS',                   9000, 3600),
    (30,  3, 2,   18, 'CD SECOURS',                   9000, 3600),
    (31,  3, 2,   19, 'CD SECOURS',                   9000, 3600),
    (32,  2, 7, NULL, 'Top H',                          600,  600),
    (33,  1, 4, NULL, 'Pub In',                         600,  600),
    (34,  1, 8, NULL, 'ECRAN PUB',                      600,  600),
    (35,  1, 5, NULL, 'Pub Out',                        600,  600);
SELECT setval('templates_id_seq', 35);

-- row 121 (hour=0 min=0 sec=0) is a catch-all sentinel; template_id NULL
INSERT INTO ad_schedule (id, hour, minute, second, template_id, priority, duration) OVERRIDING SYSTEM VALUE VALUES
    (1,   0, 59, 45, 2, 0, 10),
    (2,   1, 59, 45, 2, 0, 10),
    (3,   2, 59, 45, 2, 0, 10),
    (4,   3, 59, 45, 2, 0, 10),
    (5,   4, 59, 45, 2, 0, 10),
    (6,   5, 59, 45, 2, 0, 10),
    (7,   6, 59, 45, 2, 0, 10),
    (8,   7, 59, 45, 2, 0, 10),
    (9,   8, 59, 45, 2, 0, 10),
    (10,  9, 59, 45, 2, 0, 10),
    (11, 10, 59, 45, 2, 0, 10),
    (12, 11, 59, 45, 2, 0, 10),
    (13, 12, 59, 45, 2, 0, 10),
    (14, 13, 59, 45, 2, 0, 10),
    (15, 14, 59, 45, 2, 0, 10),
    (16, 15, 59, 45, 2, 0, 10),
    (17, 16, 59, 45, 2, 0, 10),
    (18, 17, 59, 45, 2, 0, 10),
    (19, 18, 59, 45, 2, 0, 10),
    (20, 19, 59, 45, 2, 0, 10),
    (21, 20, 59, 45, 2, 0, 10),
    (22, 21, 59, 45, 2, 0, 10),
    (23, 22, 59, 45, 2, 0, 10),
    (24, 23, 59, 45, 2, 0, 10),
    (25,  0,  5,  0, 1, 0, 60),
    (26,  0, 27,  0, 1, 0, 60),
    (28,  0, 47,  0, 1, 0, 60),
    (29,  1,  5,  0, 1, 0, 60),
    (30,  1, 27,  0, 1, 0, 60),
    (32,  1, 47,  0, 1, 0, 60),
    (33,  2,  5,  0, 1, 0, 60),
    (34,  2, 27,  0, 1, 0, 60),
    (36,  2, 47,  0, 1, 0, 60),
    (37,  3,  5,  0, 1, 0, 60),
    (38,  3, 27,  0, 1, 0, 60),
    (40,  3, 47,  0, 1, 0, 60),
    (41,  4,  5,  0, 1, 0, 60),
    (42,  4, 27,  0, 1, 0, 60),
    (44,  4, 47,  0, 1, 0, 60),
    (45,  5,  5,  0, 1, 0, 60),
    (46,  5, 27,  0, 1, 0, 60),
    (48,  5, 47,  0, 1, 0, 60),
    (49,  6,  5,  0, 1, 0, 60),
    (50,  6, 27,  0, 1, 0, 60),
    (52,  6, 47,  0, 1, 0, 60),
    (53,  7,  5,  0, 1, 0, 60),
    (54,  7, 27,  0, 1, 0, 60),
    (56,  7, 47,  0, 1, 0, 60),
    (57,  8,  5,  0, 1, 0, 60),
    (58,  8, 27,  0, 1, 0, 60),
    (60,  8, 47,  0, 1, 0, 60),
    (61,  9,  5,  0, 1, 0, 60),
    (62,  9, 27,  0, 1, 0, 60),
    (64,  9, 47,  0, 1, 0, 60),
    (65, 10,  5,  0, 1, 0, 60),
    (66, 10, 27,  0, 1, 0, 60),
    (68, 10, 47,  0, 1, 0, 60),
    (69, 11,  5,  0, 1, 0, 60),
    (70, 11, 27,  0, 1, 0, 60),
    (72, 11, 47,  0, 1, 0, 60),
    (73, 12,  5,  0, 1, 0, 60),
    (74, 12, 27,  0, 1, 0, 60),
    (76, 12, 47,  0, 1, 0, 60),
    (77, 13,  5,  0, 1, 0, 60),
    (78, 13, 27,  0, 1, 0, 60),
    (80, 13, 47,  0, 1, 0, 60),
    (81, 14,  5,  0, 1, 0, 60),
    (82, 14, 27,  0, 1, 0, 60),
    (84, 14, 47,  0, 1, 0, 60),
    (85, 15,  5,  0, 1, 0, 60),
    (86, 15, 27,  0, 1, 0, 60),
    (88, 15, 47,  0, 1, 0, 60),
    (89, 16,  5,  0, 1, 0, 60),
    (90, 16, 27,  0, 1, 0, 60),
    (92, 16, 47,  0, 1, 0, 60),
    (93, 17,  5,  0, 1, 0, 60),
    (94, 17, 27,  0, 1, 0, 60),
    (96, 17, 47,  0, 1, 0, 60),
    (97, 18,  5,  0, 1, 0, 60),
    (98, 18, 27,  0, 1, 0, 60),
    (100,18, 47,  0, 1, 0, 60),
    (101,19,  5,  0, 1, 0, 60),
    (102,19, 27,  0, 1, 0, 60),
    (104,19, 47,  0, 1, 0, 60),
    (105,20,  5,  0, 1, 0, 60),
    (106,20, 27,  0, 1, 0, 60),
    (108,20, 47,  0, 1, 0, 60),
    (109,21,  5,  0, 1, 0, 60),
    (110,21, 27,  0, 1, 0, 60),
    (112,21, 47,  0, 1, 0, 60),
    (113,22,  5,  0, 1, 0, 60),
    (114,22, 27,  0, 1, 0, 60),
    (116,22, 47,  0, 1, 0, 60),
    (117,23,  5,  0, 1, 0, 60),
    (118,23, 27,  0, 1, 0, 60),
    (120,23, 47,  0, 1, 0, 60),
    (121, 0,  0,  0, NULL, 0, 0);
SELECT setval('ad_schedule_id_seq', 121);

INSERT INTO schedules (id, from_hour, to_hour, monday, tuesday, wednesday, thursday, friday, saturday, sunday, template_id) OVERRIDING SYSTEM VALUE VALUES
    (1, 0, 23, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, 3);
SELECT setval('schedules_id_seq', 1);

INSERT INTO users (id, login, password_hash, active, role) OVERRIDING SYSTEM VALUE VALUES
    (1, 'admin', crypt('admin123', gen_salt('bf')), TRUE, 1);
SELECT setval('users_id_seq', 1);

INSERT INTO configurations (auto_mix_on_start, auto_play_on_start, preload, fade_out_duration_ms)
VALUES (false, false, 10, 2500);

-- OpenStudio — seed data

CREATE EXTENSION IF NOT EXISTS pgcrypto;

INSERT INTO genres (id, name) VALUES
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

INSERT INTO sectors (id, name) VALUES
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

INSERT INTO templates (id, name) VALUES
    (1, 'PUB'),
    (2, 'TOP HORAIRE'),
    (3, 'SEMAINE'),
    (4, 'HIT ONLY');
SELECT setval('templates_id_seq', 4);

INSERT INTO stations (id, name, library_path) VALUES
    (1, 'DEMO', '/Users/Shared/OpenStudio/Library/demo');
SELECT setval('stations_id_seq', 1);

INSERT INTO categories (id, name, protected) VALUES
    (1, 'Jingles',      TRUE),
    (2, 'Music',        TRUE),
    (3, 'Intervention', TRUE),
    (4, 'PubIn',        TRUE),
    (5, 'PubOut',       TRUE),
    (6, 'Filler',       TRUE),
    (7, 'Top of Hour',  TRUE),
    (8, 'Pub',          TRUE);
SELECT setval('categories_id_seq', 8);

INSERT INTO subcategories (id, category_id, name, hidden, protected) VALUES
    (1,  1, 'Jingles',    FALSE,  FALSE),
    (2,  1, 'Jin.W-E',    FALSE,  FALSE),
    (3,  1, 'Jin.Ete',    FALSE,  FALSE),
    (4,  1, 'Jin.Hiver',  FALSE,  FALSE),
    (5,  1, 'Accaps',     FALSE,  FALSE),
    (6,  1, 'Tapis',      FALSE,  FALSE),
    (7,  1, 'Promos',     FALSE,  FALSE),
    (8,  1, 'Hitmix',     FALSE,  FALSE),
    (9,  1, 'Liners',     FALSE, FALSE),
    (10, 1, 'Divers',     FALSE, FALSE),
    (11, 2, 'PowerPlay',  FALSE, FALSE),
    (12, 2, 'FR-1930',    FALSE, FALSE),
    (13, 2, 'FR-1940',    FALSE, FALSE),
    (14, 2, 'FR-1950',    FALSE, FALSE),
    (15, 2, 'FR-1960',    FALSE, FALSE),
    (16, 2, 'FR-1970',    FALSE, FALSE),
    (17, 2, 'FR-1980',    FALSE, FALSE),
    (18, 2, 'FR-1990',    FALSE, FALSE),
    (19, 2, 'FR-2000',    FALSE, FALSE),
    (20, 2, 'FR-2010',    FALSE, FALSE),
    (21, 2, 'FR-2020',    FALSE, FALSE),
    (22, 3, 'Intervention', FALSE, FALSE),
    (23, 4, 'PubIn',        FALSE, FALSE),
    (24, 5, 'PubOut',       FALSE, FALSE),
    (25, 6, 'Filler',       FALSE, FALSE),
    (26, 7, 'Top of Hour',  FALSE, FALSE),
    (27, 8, 'Pub',          FALSE, FALSE);
SELECT setval('subcategories_id_seq', 27);

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
TRUNCATE TABLE "template_slots" RESTART IDENTITY CASCADE;
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
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (32, 2, 1, 7, NULL, 'Top Horaire', 600, 600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (33, 1, 1, 4, NULL, 'Pub In', 600, 600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (34, 1, 2, 8, NULL, 'ECRAN PUB', 600, 600);
INSERT INTO "template_slots" ("id", "template_id", "position", "category_id", "subcategory_id", "comment", "track_protection", "artist_protection") VALUES (35, 1, 3, 5, NULL, 'Pub Out', 600, 600);
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
INSERT INTO clock_events (id, event_type, days_mask, hours_mask, hour, minute, second, priority, duration) VALUES
    ( 1, 2, 127, 0,  0, 59, 45, 0, 10),
    ( 2, 2, 127, 0,  1, 59, 45, 0, 10),
    ( 3, 2, 127, 0,  2, 59, 45, 0, 10),
    ( 4, 2, 127, 0,  3, 59, 45, 0, 10),
    ( 5, 2, 127, 0,  4, 59, 45, 0, 10),
    ( 6, 2, 127, 0,  5, 59, 45, 0, 10),
    ( 7, 2, 127, 0,  6, 59, 45, 0, 10),
    ( 8, 2, 127, 0,  7, 59, 45, 0, 10),
    ( 9, 2, 127, 0,  8, 59, 45, 0, 10),
    (10, 2, 127, 0,  9, 59, 45, 0, 10),
    (11, 2, 127, 0, 10, 59, 45, 0, 10),
    (12, 2, 127, 0, 11, 59, 45, 0, 10),
    (13, 2, 127, 0, 12, 59, 45, 0, 10),
    (14, 2, 127, 0, 13, 59, 45, 0, 10),
    (15, 2, 127, 0, 14, 59, 45, 0, 10),
    (16, 2, 127, 0, 15, 59, 45, 0, 10),
    (17, 2, 127, 0, 16, 59, 45, 0, 10),
    (18, 2, 127, 0, 17, 59, 45, 0, 10),
    (19, 2, 127, 0, 18, 59, 45, 0, 10),
    (20, 2, 127, 0, 19, 59, 45, 0, 10),
    (21, 2, 127, 0, 20, 59, 45, 0, 10),
    (22, 2, 127, 0, 21, 59, 45, 0, 10),
    (23, 2, 127, 0, 22, 59, 45, 0, 10),
    (24, 2, 127, 0, 23, 59, 45, 0, 10),
-- event_type=3: recurring every day at every hour (hours_mask=16777215) — PUB slots
    (25, 3, 127, 16777215, 0,  5,  0, 0, 60),
    (26, 3, 127, 16777215, 0, 27,  0, 0, 60),
    (27, 3, 127, 16777215, 0, 47,  0, 0, 60);
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

INSERT INTO users_roles (id, name) VALUES
    (1, 'SuperAdmin'),
    (2, 'Admin'),
    (3, 'Manager'),
    (4, 'User');
SELECT setval('users_roles_id_seq', 4);

INSERT INTO users (id, login, password_hash, active, role_id) VALUES
    (1, 'admin', crypt('admin123', gen_salt('bf')), TRUE, 1);
SELECT setval('users_id_seq', 1);

INSERT INTO configurations (
    auto_mix_on_start,
    auto_play_on_start,
    preload,
    fade_out_duration_ms,
    stop_fade_duration_ms,
    timezone
)
VALUES (false, false, 10, 2500, 1000, 'Europe/Paris');

INSERT INTO "advertisers" ("id", "name", "sector_id", "address", "vat_number", "notes", "active", "client_since") VALUES
(1,	'CARRELAGES PIRARD',	2,	'Rue du Travail 1, 4460 Grâce-Hollogne',	NULL,	'FAKE CUSTOMER',	'1',	'2000-01-01');

INSERT INTO "contacts" ("id", "advertiser_id", "name", "role", "phone", "email", "primary_contact", "notes") VALUES
(1,	1,	'Monsieur Dracula',	'Manager',	'+32475151230',	'hello@pirard.local',	'1',	NULL);

INSERT INTO "campaigns" ("id", "advertiser_id", "name", "total_broadcasts", "broadcast_count", "station_id", "active", "encoded_at", "start_date", "end_date", "last_aired_at") VALUES
(1,	1,	'HALLOWEEN 2026',	10000,	0,	1,	'1',	NULL,	'2026-01-01',	'2026-12-31',	NULL);


# Open Studio Admin Todo

* Ad
* Voicetracks
* Sweepers

---

Dans la Generation de Playlist (/playlists bouton Generate) , quand un slot_type "ad_break" est rencontré dans les itérations du template_slots,

un track de type 14 "Filler" de la durée (240s ici) OU PLUS GRAND sera ajouté. (si plus LONG on le diminue -> cue_out).

Tu fais une recherche alétoire sur les filler, pas toujours prendre le premier !

----

Dans Automation, en dessous de Playlist Editor, une page "Ad Scheduler".
Cette page viendra remplir les écrans pub qui sont vides de "campaigns" (fichiers via campaigns_tracks) actives et diminuera soit en partie le Filler (diminuer son cue_out) soit le supprimera totalement si le total des pub fait la durée de l'écran. (240=240). Les publicités se mettent DEVANT le Filler.

Ex: Si on a 2 x 30 secondes de pub 240 - 60 = 180 secondes encore à jouer de Filler pour fermer l'écran.

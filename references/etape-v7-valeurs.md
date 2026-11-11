# Étape V7 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape V7 (le rendu de la page : l'image nette des flocons de Reiter, chaque case dessinée comme un hexagone lisse, par `rendu_hex::rendre` ; quatre exportations
neuves du module ; la page à deux canvas) attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque identifiant (`V-V7-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_v7_*.rs`).
Elle est séparée des résultats figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du prototype de l'écrivain.** Les seules valeurs tirées du prototype sont les
mesures dites mesurées (V-V7-40 à V-V7-44), qu'aucun test ne fige. Une valeur vient d'une convention que la doc de `src/vitrine.rs` fixe (dite telle, avec sa raison), d'une arithmétique entière refaite à la main,
d'un calcul **fait à la main** sur les cinq arrêts du dégradé de `rendu_hex`, d'un outil indépendant (nommé) ou d'un test qui recalcule le champ de la doc d'après les masses d'un `Flocon` construit sans passer par la vitrine.

**Outil indépendant** (Python 3.13, dans `essai-v7-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `v7_ref.py` est un second code, écrit d'après la doc seule de `src/vitrine.rs` (« L'image nette »,
« Le champ dessiné ») et de `src/rendu_hex.rs` (la géométrie, les échantillons, l'arrondi cubique, le dégradé, le pixel), sans lire le Rust de ces deux fichiers : le modèle de Reiter en entiers Python
(`modele_independant.py`, écrit d'après la doc de `flocon.rs` pour l'étape 3 et validé alors contre le prototype Rust), le champ de la doc, et le rendu net (recopié de `g1_ref.py`, écrit d'après la doc de `rendu_hex.rs` et
validé à G1). `python v7_ref.py valeurs` écrit E, les pixels du germe et les quatre empreintes de V-V7-13 ; `python v7_ref.py germe` écrit la couleur du germe à sept instants (V-V7-11).

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Tout test de V7 est **exact** : égalité d'entiers, de chaînes, d'octets. Aucun test n'a de tolérance, aucune propriété « vraie en moyenne » n'est testée : le rendu est un calcul entier déterministe (`rendu_hex`), le flocon de Reiter
   n'a aucun hasard, et la vitrine ne fait que les relier.
2. **L'image de la vitrine est celle de `rendu_hex::rendre` appliqué au champ de la doc.** Le test recalcule le champ lui-même (`etape_v7_commun::champ_de_la_doc`, trois opérations sur les `brut` des masses d'un `Flocon` de la
   bibliothèque) et appelle `rendre` ; la bibliothèque `rendu_hex` est éprouvée à part (G1 : un rendu naïf du test, des calculs à la main, un code Python) : V7 teste ce qui est à lui, le **champ**, l'**ordre** des cases, les **réglages**
   (`cote`, `sur`, la vue de rayon `R`, `plein = 4 E`), les **bornes** et les **codes**, le **tampon**.
3. Un code d'erreur, une borne, la table des modèles sont une **convention de la doc** : la doc fixe la table (les codes 0 à 6, leur ordre de vérification, 2048, 4 et 4096), le test la recopie à la main. Aucune de ces valeurs ne vient de
   l'étude d'un article : l'image est un rendu, pas une mesure (`AGENTS.md`, règle 3 : jamais une preuve de physique).
4. **Aucune valeur publiée, aucune valeur de `D`** : V7 ne mesure rien ; les tests de Z1 et D3 qui refusent `1,71`, `1,433`, `1,263` et « D = » suivi d'un chiffre dans la page jouent toujours sur la page de V7.
5. Les tailles des tests viennent du **coût mesuré** (V-V7-40 à V-V7-42) : des images de 62 à 128 pixels pour presque tous les cas, les deux images de la page (384 × 2 et 1024 × 3) une fois chacune, les deux bornes de
   4 096 échantillons de côté une fois chacune.
6. **Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests de la page sont de la **forme** (régression de texte), sauf `lecture_js_lit_l_image_nette_sur_un_faux_module`
   (comparé à un calcul à la main sur un faux module qui change d'état) ; ceux du module sont **justes** et ne passent qu'une fois `docs/givre.wasm` reconstruit.

---

## La table de la doc (`etape_v7_commun::MODELES_NETS`)

**V-V7-01 : les trois flocons et leur échelle `E`** (`chaque_modele_est_dessine_avec_son_champ`, `etape_v7_commun`).
- Valeur : modèle 1 (plaque) : rayon 30, durée 400, `α = 1`, `β = 0`, `γ = 0,05` ; modèle 2 (étoile) : rayon 40, durée 120, `β = 0,45`, `γ = 0,1` ; modèle 3 (dendrite) : rayon 60, durée 800, `β = 0,4`, `γ = 0,001`.
  `E = γ.brut × durée` : `214 748 365 × 400 = 85 899 346 000` ; `429 496 730 × 120 = 51 539 607 600` ; `4 294 967 × 800 = 3 435 973 600`. `plein = 4 E`.
- Source : la table de la doc de `src/vitrine.rs` (« Les huit modèles », comparée aux scénarios d'exemple par les tests de l'étape 5) ; `γ.brut` est `Parametres::lire("0.05")` = `round(0,05 × 2^32)` = 214 748 365, `"0.1"` = 429 496 730
  (la doc de `lire_decimal` le dit), `"0.001"` = `round(4 294 967,296)` = 4 294 967 ; les produits sont de l'arithmétique entière refaite à la main et confirmés par `v7_ref.py valeurs` (la ligne « E modèle »). Le test relit aussi
  `Parametres::lire(…).gamma.0 × durée` et l'égale à `E` écrit à la main.
- Tolérance : aucune. Critère : l'égalité exacte.

## L'image nette de la vitrine (`etape_v7_vitrine.rs`)

**V-V7-02 : le champ de la doc et l'image** (`l_image_nette_est_celle_de_rendre_pour_le_champ_de_la_doc`, `l_image_d_une_page_est_celle_de_la_doc`).
- Valeur : pour chaque flocon, à plusieurs instants et plusieurs réglages, le tampon rempli par `givre_nette_preparer(cote, sur)` est, octet pour octet, `rendre(R, champ, { cote, rayon_vue: R, sur, plein: 4 E })` avec
  `champ(case) = 0` si `brut < UN` et `3 (brut − UN) + E` sinon, dans l'ordre canonique de `Hexagone` (`Flocon::masses`). Instants : plaque 0, 1, 37, 400 pas ; étoile 0, 5, 60, 120 ; dendrite 0, 150, 800, 10 000 (qui dépasse
  la durée). Réglages : `(62, 2)`, `(96, 3)`, `(128, 1)`, `(2, 1)` ; et les deux images de la page, `(384, 2)` pour l'étoile jusqu'au bout, `(1024, 3)` pour la dendrite jusqu'au bout.
- Source : la doc de `src/vitrine.rs` (« Le champ dessiné ») pour le champ, celle de `src/rendu_hex.rs` pour `rendre` ; **raison du champ** (une convention de la doc, discutée dans `MODELE.md`, étape V7) : `3 (brut − UN) + E` met une case
  tout juste gelée au premier arrêt du dégradé (`t = 256`, un bleu clair visible sur le blanc) et le germe à la fin du scénario au dernier (`t = 1024`) ; la masse d'une case gelée mesure son âge (elle gagne `γ` par pas).
- Tolérance : aucune. Critère : l'égalité exacte des `3 × cote²` octets.

**V-V7-03 : les bornes** (`les_bornes_et_les_codes_d_erreur`).
- Valeur : `NETTE_COTE_MAX = 2048`, `NETTE_SUR_MAX = 4`, `NETTE_GRILLE_MAX = 4096` ; `(2, 1)`, `(2048, 2)` et `(1024, 4)` rendent 0 et l'image de `rendre` (la borne 4096 est **incluse**) ; `cote` impair, 0, 1, 2049, 2050, 4096, `u32::MAX` :
  code 3 ; `sur` 0, 5, 16, `u32::MAX` : code 4 ; `(2048, 3)`, `(1366, 3)`, `(2048, 4)`, `(1026, 4)` : code 5.
- Source : convention de la doc (« Les bornes »), mesurée (V-V7-40 à V-V7-42) : 4096 échantillons de côté (16,8 millions d'échantillons) coûtent 0,24 à 0,30 s en natif au calme ; `(1366, 3)` (4098) est le plus petit produit pair
  qui dépasse 4096 avec un `cote` pair et un `sur` de 3 (4097 est impair : aucun couple permis ne le donne, une borne placée en 4097 ne se distingue pas de 4096).
- Tolérance : aucune.

**V-V7-04 : les codes et leur ordre** (`les_bornes_et_les_codes_d_erreur`, `jamais_de_panique_pour_un_cote_ou_un_sur_absurde`).
- Valeur : les vérifications se font dans l'ordre : rien démarré (2), modèle sur une grille carrée (1), `cote` (3), `sur` (4), `cote × sur` (5) ; le premier refus rend son code ; le code 6 (`rendre` a refusé) n'est jamais rendu. Exemples :
  `(2050, 5)` : 3 (le côté avant le suréchantillonnage) ; `(1000, 5)` : 4 (le suréchantillonnage avant le nombre d'échantillons) ; `(0, 0)` : 3 ; pour la DLA et le claquage, `(0, 0)` : 1 (le modèle avant les réglages). Un jeu de 10 côtés
  × 7 `sur` × 8 modèles, `u32::MAX` compris : le code de la table, jamais une panique.
- Source : convention de la doc (la table et l'ordre), écrite à la main dans le test (`code_de_la_doc`) ; raison de l'ordre : le refus le moins cher d'abord, et un seul code par cas absurde rend l'erreur déterministe. Le code 2 (rien n'est
  démarré) ne se teste que sur un module neuf (Node, V-V7-20) : en natif l'état global est déjà posé par les autres tests du processus.
- Tolérance : aucune.

**V-V7-05 : une préparation ne change rien à l'état** (`une_preparation_ne_change_rien_a_l_etat`).
- Valeur : après 37, 100 pas et jusqu'au bout, avant et après une préparation réussie, deux refusées et une réussie de plus petit côté : temps, durée, sites, rayon, compacité (bits), huit mots de l'empreinte, empreinte, dimensions et
  **chaque pixel de l'image d'un pixel par case** égaux à `e5::natif` (le code natif de l'étape 5) ; la suite du scénario jouée après une préparation donne l'état natif de la durée entière.
- Source : la doc (« Rien de ce qui existe ne change ») ; la référence est le code natif de l'étape 5, qui n'appelle jamais `vitrine`. Tolérance : aucune.

**V-V7-06 : le tampon appartient à l'état qu'il montre** (`le_tampon_appartient_a_l_etat_qu_il_montre`).
- Valeur : vide après `givre_demarrer` et après des pas ; `(62, 2)` : `62 × 62` ; `givre_avancer(0)` le laisse ; `givre_avancer(1)` (qui joue) l'efface ; un `givre_demarrer` refusé (modèle 9) le laisse ; un `givre_demarrer` qui réussit l'efface ;
  une préparation réussie remplace la précédente ; un refus l'efface ; un scénario fini garde son tampon (`givre_avancer(1)` et `(100)` rendent 0) ; deux préparations de suite donnent le même tampon.
- Source : convention de la doc (« Le tampon appartient à l'état qu'il montre »), parallèle à la mesure gardée de Z1 (V-Z1-xx : « la mesure gardée appartient à l'amas qu'elle a mesuré »). Tolérance : aucune.

**V-V7-07 : un modèle sur une grille carrée n'a pas d'image nette** (`les_modeles_carres_n_ont_pas_d_image_nette`).
- Valeur : la DLA (modèle 0, graine 4, 60 particules) et le claquage (modèles 4 à 7, graine 5, 30 sites) : code 1 pour `(62, 2)`, `(0, 0)`, `(u32::MAX, u32::MAX)` ; tampon vide ; l'image d'un pixel par case (chaque pixel), l'état et
  l'empreinte égaux à ceux du code natif (`e5::natif`, `d3::natif_claquage`).
- Source : convention de la doc : les modèles carrés gardent leurs cases-carrés (la décision d'André, `plan-v7.md` : « les modèles à grille carrée gardent leurs cases-carrés et leur `pixelated` »). Tolérance : aucune.

**V-V7-08 : chaque modèle est dessiné avec son champ** (`chaque_modele_est_dessine_avec_son_champ`).
- Valeur : pour les modèles 1, 2 et 3 au temps 0 et à la fin, `(96, 3)` donne l'image de `rendre` pour **ce** modèle (rayon, `E`, champ) ; les trois images finales sont deux à deux différentes (SHA-256).
- Source : V-V7-01, V-V7-02. Tolérance : aucune.

**V-V7-09 : avant tout démarrage, en natif** (`avant_tout_demarrage_la_preparation_rend_le_code_2`, `tests/etape_v7_neuf.rs` ; ajout du juge).
- Valeur : dans un processus neuf (le fichier n'a qu'un test : aucun `givre_demarrer` avant lui), largeur et hauteur du tampon 0, tout pixel `u32::MAX` ; `givre_nette_preparer` rend 2 pour `(62, 2)`, `(0, 0)`, `(61, 2)`, `(62, 9)`, `(2048, 4)`,
  `(u32::MAX, u32::MAX)` (« rien n'est démarré » est vérifié avant le modèle et les réglages) ; un `givre_demarrer` refusé (modèle 9) n'y change rien ; puis `givre_demarrer(2, 0)` : tampon vide, `(62, 2)` rend 0 et `62 × 62` ; puis `givre_demarrer(0, 1)` :
  tampon vide, code 1 (un modèle carré démarré après un flocon ne garde pas son image).
- Source : convention de la doc (« Avant tout démarrage », la table des codes et leur ordre). Chaque fichier de `tests/` est un programme à part : c'est ce qui rend le code 2 visible en natif. Tolérance : aucune.

**Ajouts du juge aux valeurs ci-dessus** (mêmes sources, mêmes critères) : V-V7-05 : après les lectures de l'état et un `givre_mesurer` (0 pour un flocon), le tampon est encore l'image `(64, 2)` de cet état (lire n'efface rien) ;
V-V7-06 : `(64, 3)` remplace `(64, 1)` à 60 pas (le même côté, un autre suréchantillonnage : l'image est recalculée, et les deux images diffèrent) ; V-V7-07 : la DLA est démarrée après un flocon dont l'image est prête, son tampon est vide dès le démarrage.

## Ce que l'image montre, à la main (`etape_v7_rendu.rs`)

**V-V7-10 : le germe seul, quatre pixels** (`le_germe_seul_quatre_pixels_a_la_main`).
- Valeur : au temps 0, `(62, 2)` : pour la plaque (`R = 30`) et l'étoile (`R = 40`), les seuls pixels qui ne sont pas blancs sont `(30, 30)`, `(31, 30)`, `(30, 31)`, `(31, 31)`, tous `(241, 246, 251)` ; pour la dendrite (`R = 60`),
  aucun pixel n'est non blanc.
- Source : calcul à la main. Les échantillons sont aux `X = (2 a + 1 − 124)(2 R + 1) / 248` ; les plus proches du centre sont à `± (2 R + 1) / 248` en `X` et en `Y` (0,246 ; 0,327 ; 0,488 pour `R = 30, 40, 60`), la case centrale est un hexagone
  à plat de demi-hauteur 1/2 et de sommets à `± 0,577` (`|Y| <= 1/2`, `|Y| <= √3 (0,577 − |X|)`) : `(0,246, 0,246)` : `√3 × 0,331 = 0,573 >= 0,246` dedans ; `(0,327, 0,327)` : `0,434 >= 0,327` dedans ; `(0,488, 0,488)` : `0,154 < 0,488`
  dehors, et les échantillons suivants (à `± 3 (2 R + 1) / 248`) sont dehors dans les trois cas. Un pixel central a donc 1 échantillon sur 4 dans la case : `(3 × 255 + 198 + 2) / 4 = 241`, `(3 × 255 + 219 + 2) / 4 = 246` (246,5
  tronqué), `(3 × 255 + 239 + 2) / 4 = 251` (251,5 tronqué). `(198, 219, 239)` est le premier arrêt du dégradé (`t = 256`, la valeur `E` sur `plein = 4 E`). Confirmé par `v7_ref.py valeurs` (les quatre pixels, les deux modèles).
- Tolérance : aucune. Critère : exactement ces pixels (et ces valeurs).

**V-V7-11 : la couleur du germe à quatre instants** (`le_germe_a_quatre_instants`).
- Valeur : pixel `(305, 305)` de l'image `(610, 1)` (centre du germe, 10 pixels par case) : plaque 0 pas : `(198, 219, 239)` ; 100 pas : `(130, 185, 220)` ; 200 pas : `(70, 144, 198)` ; 400 pas : `(8, 48, 107)` ; étoile 0 : `(198, 219, 239)`,
  60 : `(70, 144, 198)`, 120 : `(8, 48, 107)` ; dendrite 800 : `(8, 48, 107)` ; les coins `(0, 0)` et `(609, 609)` sont blancs.
- Source : calcul à la main. Le germe gagne `γ` à chaque pas et ne reçoit rien (ses voisines sont réceptives, leur masse qui diffuse est 0) : `brut − UN = γ.brut × t` exact, donc `champ = E + 3 γ.brut t` et `t_degrade = 256 + 768 γ.brut t / E` : 256,
  448 (`t = 100 = durée / 4`), 640 (`durée / 2`), 1024 (la durée). `t = 448` : `s = 1`, `f = 192` : `(198 × 64 + 107 × 192 + 128) >> 8 = 130`, `(219 × 64 + 174 × 192 + 128) >> 8 = 185`, `(239 × 64 + 214 × 192 + 128) >> 8 = 220` ;
  `t = 640` : `s = 2`, `f = 128` : `(107 × 128 + 33 × 128 + 128) >> 8 = 70`, `(174 × 128 + 113 × 128 + 128) >> 8 = 144`, `(214 × 128 + 181 × 128 + 128) >> 8 = 198` ; `t = 1024` : le dernier arrêt `(8, 48, 107)` ; les arrêts sont ceux de
  `rendu_hex::ARRETS` (la doc). Le pixel `(305, 305)` : la vue de rayon 30 montre 61 cases sur 610 pixels, le pixel est à `0,05` du centre de la case `(0, 0)` : dedans. Confirmé par `v7_ref.py germe` (les sept cas, y compris l'étoile à 60 pas).
- Tolérance : aucune.

**V-V7-12 : la couleur suit la masse, case par case, le long d'un bras** (`la_couleur_suit_la_masse_le_long_d_un_bras`).
- Valeur : plaque après 400 pas, image `(610, 1)` : le pixel `(305, 305 ∓ 10 q)` est la couleur du dégradé pour le champ de la doc de la case `(±q, 0)`, pour `q` de 0 à 30 ; le cristal a 21 cases gelées sur le bras (de 0 à 20), la case `(21, 0)` est blanche.
- Source : géométrie de `rendu_hex` (le bras `(1, 0)` va vers le haut : `Y = −q`, une case par 10 pixels ; le pixel est à `0,05` du centre de la case) ; champ de la doc recalculé à la main d'après `Flocon::masses` et `Hexagone::index` ; `couleur_du_degrade`
  (G1). Tolérance : aucune.
- **Le champ de ce bras à la main** (ajout du juge ; le test l'exige case par case, le rayon 20 n'est donc plus une valeur mesurée) : la plaque a `β = 0`, aucune vapeur : rien ne diffuse, une case ne gagne que `γ` par pas, et seulement une fois réceptive
  (gelée ou voisine d'une case gelée). `γ.brut = 214 748 365` : `20 γ.brut = 4 294 967 300 >= 2^32 = 4 294 967 296` et `19 γ.brut = 4 080 218 935 < 2^32` : l'anneau 1, réceptif dès le temps 0, gèle au pas 20 ; l'anneau `q`, réceptif au pas `20 (q − 1)`,
  gèle au pas `20 q`. À 400 pas : rayon 20 (21 cases sur le bras), `brut(q) = (420 − 20 q) × 214 748 365` pour `q` de 1 à 20 (400 γ pour l'anneau 1, 20 γ pour l'anneau 20), `champ(q) = 3 (brut(q) − 2^32) + E` ; le germe `2^32 + 400 γ.brut` vaut `4 E` ;
  la case 21 (réceptive au pas 400 seulement) a la masse 0 : champ 0, blanche. Le test égale ces valeurs à celles tirées de `Flocon::masses`.

**V-V7-13 : les empreintes de référence** (`l_empreinte_des_images_de_reference`).
- Valeur : SHA-256 des octets R, V, B : modèle 2 après 60 pas, `(96, 3)` : `218e8fec69be27a8bb7e9a1179ac65909d55d8470b3aecdf2f424febf9f6b427` ; modèle 1 après 400 pas, `(64, 2)` :
  `87ad3c3a4cbe267243acf65c6cf3b4ff3666dafd5ef0c18c170a47b95263f673` ; modèle 3 après 800 pas, `(64, 2)` : `bb77ac2f0423598f3bfcb810c79da05f879861d1d18c2ca94db23a2436359ae4` ; modèle 2 au temps 0, `(96, 3)` :
  `030775df7e12d5d0376ab30678f565e12adb2826ca3880b5c66c6a24e3780bb9`.
- Source : `v7_ref.py valeurs` (Python 3.13, `hashlib.sha256`) : le modèle de Reiter, le champ et le rendu d'après la doc seule. **Étiquette : régression** (une valeur de l'outil, pas d'un article), **recoupée** : le prototype Rust, la vitrine
  et `rendre` donnent les mêmes octets, du premier coup. Tolérance : aucune. Le test égale trois choses : ce que la vitrine rend, ce que `rendre` rend pour le champ de la doc, et la valeur écrite.

## Le module sous Node (`etape_v7_module.rs`, `tests/etape_v7/nette-wasm.mjs`)

**V-V7-20 : le module lit l'image nette comme le natif** (`node_lit_l_image_nette_comme_le_natif`).
- Valeur : pour dix cas (modèle, graine, pas, côté, sur) : `(1, 1, 400, 62, 2)`, `(1, 7, 0, 62, 2)`, `(2, 0, 37, 96, 3)`, `(2, 5, 120, 128, 1)`, `(3, u32::MAX, 500, 100, 2)`, `(3, 1, 10 000, 64, 4)`, `(2, 3, 60, 384, 2)`, `(0, 1, 150, 64, 2)`,
  `(4, 3, 40, 64, 2)`, `(6, 2, 30, 62, 2)`, la sortie entière du script (15 lignes) est celle que `attendu_du_script_nette` donne : sur un module neuf `2,0,0,4294967295` ; l'état (temps, durée, sites, rayon, empreinte, image d'un pixel par case)
  lu **avant** et **après** la préparation, identique ; le code, les dimensions et les SHA-256 de ce que `lireImageNette` rend (RGBA) et de ce que `givre_nette_pixel` lit (R, V, B), égaux à ceux de `rendre` pour le champ de la doc ;
  cinq pixels hors image à `u32::MAX` ; onze refus (`3,3,3,3,3,3,4,4,4,5,5` pour un flocon, onze `1` pour un modèle carré) ; un refus efface le tampon ; un `givre_avancer(1)` l'efface (sauf scénario fini) ; un `givre_demarrer` l'efface.
- Source : V-V7-02 à V-V7-07 (le natif) ; le script lit l'état **avant** les appels qui le changent (leçon 21) et a été éprouvé sous un faux module qui **change d'état** (le rapport de l'écrivain) ; le module commité doit être reconstruit.
- Tolérance : aucune. **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**

**V-V7-21 : l'image de la fin** (`node_lit_l_image_de_la_fin`). La dendrite jusqu'au bout, `(1024, 3)` : même script, même exigence (le temps sous Node est mesuré par le pilote, V-V7-44).

**V-V7-22 et V-V7-23 : les arguments faux et la validité du JavaScript** (`le_script_refuse_des_arguments_faux`, `les_scripts_sont_du_javascript_valide`) : forme, comme à l'étape D3.

## La page (`etape_v7_page.rs`)

**V-V7-30, V-V7-31, V-V7-32 et V-V7-33 : la forme** (`la_page_a_les_deux_canvas_et_la_legende`, `le_canvas_des_modeles_hexagonaux_n_est_pas_pixelise`, `app_js_dessine_l_image_nette_sur_son_canvas`, `les_reglages_de_l_image_nette_sont_dans_les_bornes`).
- Valeur : deux canvas (`vitrine`, visible au départ ; `vitrine-nette`, caché au départ) et une légende cachée qui dit « hexagone » ; **`image-rendering` sans lissage (`pixelated`, `crisp-edges`, `optimize-contrast`) n'est écrit que pour `#vitrine`**
  (une liste blanche de sélecteurs : `#vitrine` ou `canvas#vitrine`), et il en reste une ; aucun attribut `style` ni aucun script n'y touche ; `app.js` appelle `lireImageNette(m, fin ? NETTE_COTE_FIN : NETTE_COTE_ANIMATION, fin ?
  NETTE_SUR_FIN : NETTE_SUR_ANIMATION)` dans `dessiner`, qui écrit `.hidden` d'après `imagesCachees` sans négation, pose l'image nette sur le canvas des hexagones et l'autre sur l'autre ; `dessiner(false)` au démarrage, `dessiner(joue === 0)`
  à l'arrêt ; les quatre constantes : côtés pairs de 2 à 2048, `sur` de 1 à 4, produits d'au plus 4096, l'animation d'au plus 1024 échantillons de côté, la fin plus fine.
- **Ajouts du juge** (des fautes de page qui passaient) : aucune règle qui **lisse** ne nomme `#vitrine` (écrite plus loin, elle l'emporterait), `image-rendering` ne porte jamais `!important`, pas de `<link rel="stylesheet">` ni de `@import` ; les scripts n'écrivent ni
  `cssText`, `setProperty`, `setAttribute`, `className`, `classList` ni `.id =` ; `dessiner` contient, aux espaces près, `if (nette.code === 0) { poser(<canvas des hexagones>, <son contexte>, nette); } else { poser(<canvas carré>, <son contexte>, lireImage(m)); }`
  (une condition inversée, ou l'image d'un pixel par case posée sur le canvas des hexagones, passait) ; `poser` écrit `.width = largeur` et `.height = hauteur` avant `putImageData(new ImageData(rgba, largeur, hauteur), 0, 0)` (sans quoi l'image de la fin, 1024 pixels,
  serait coupée dans un canvas de 384) ; l'image de l'animation a au moins 256 pixels de côté et `sur >= 2` (plus petite, elle serait agrandie plus de trois fois ; V-V7-44 : `(384, 2)` tient sous Node).
- Source : conventions de la doc de `src/vitrine.rs` (« La page ») et du plan : le prototype de la page fait foi (les mains le recopient à l'octet) ; **la limite de 1024 échantillons de côté pour l'animation** vient du coût (V-V7-41 : 1 million
  d'échantillons : 18 ms en natif). Tolérance : aucune.

**V-V7-34 : `lireImageNette` et `imagesCachees` sur un faux module** (`lecture_js_lit_l_image_nette_sur_un_faux_module`, `tests/etape_v7/lecture-pure.mjs`).
- Valeur : un faux module d'une image **non carrée** (`cote` de large, `cote + 2` de haut) de couleur `((3x + 5y + sur) & 255, (7x + 11y) & 255, (13x + 17y + cote) & 255)` qui change d'état à chaque préparation ; `lireImageNette(faux, 6, 2)` :
  code 0, `6x8`, le SHA-256 des octets RGBA du calcul à la main ; `lireImageNette(faux, 4, 3)` : `4x6`, celui de ces réglages-là ; `lireImageNette(faux, 5, 1)` : code 3, `0x0`, un tableau vide, aucun pixel lu ; les appels sont
  `preparer(6,2), largeur, hauteur, preparer(4,3), largeur, hauteur, preparer(5,1)` ; `imagesCachees(0)` : `carree` vrai, `nette` faux ; pour 1, 2, 3, 6, `u32::MAX` : `carree` faux, `nette` vrai.
- Source : calcul à la main (les formules ci-dessus, recopiées dans le test). Tolérance : aucune.

## Mesuré (rien de cela n'est figé par un test)

**V-V7-40 : le coût natif de `rendre`** (`examples` de l'écrivain, profil `dev` à `opt-level = 3`, une machine de 12 fils, au calme ; `essai-v7-outils/v7_mesures.rs`, sortie `mesure-natif-calme.txt`) : de 15 à 18 ns par échantillon dès `sur >= 2`
(30 ns à `sur = 1` : le coût d'un pixel s'ajoute), indépendant du rayon (30, 40, 60 : les trois courbes se confondent) ; 4096 échantillons de côté (16,8 millions) : 241 à 268 ms pour `(1024, 4)`, 290 à 304 ms pour `(2048, 2)` ; la limite
de la doc est donc de 0,3 s au calme, 0,7 s chargé (le poste était partagé : la série précédente, 17 à 27 ms pour `(384, 2)`, de 25 à 50 ns par échantillon). `(2048, 4)` (8192 de côté, hors borne) coûterait 0,94 à 0,99 s.
**V-V7-41 : le coût d'une image de la page** : `(384, 2)` (589 824 échantillons) 10,3 à 10,8 ms au calme + 1,8 ms de lecture pixel par pixel (147 456 appels natifs) ; `(1024, 3)` (9,4 millions) 143 à 162 ms + 13 ms de lecture ;
`(512, 2)` (1 million) 18,4 à 18,8 ms. **V-V7-42 : le coût de la lecture** : 12 ns par pixel natif (appel et verrou). **V-V7-43 : le cristal** : plaque à 400 pas rayon 20 (1 261 cases gelées), étoile à 120 rayon 30 (2 263), dendrite à 800
rayon 39 (1 513) ; masses des cases gelées : plaque de 1,000 à 21,000, étoile de 1,028 à 13,000, dendrite de 1,004 à 1,840 ; la masse maximale est celle du germe `1 + γ T` (21, 13, 1,8) sauf pour la dendrite, où une case gelée tard par
dépassement atteint 1,84 (c'est pourquoi le champ sature à `plein` au lieu de supposer le germe au maximum). **V-V7-44 : le temps sous Node (wasm)**, mesuré par le pilote sur la VM de preuve (Linux, Node 20, le module reconstruit avec le prototype, `temps-wasm.mjs`) : l'image d'animation `(384, 2)` 25,7 ms à préparer et 28 ms avec la lecture
pixel par pixel ; celle de la fin `(1024, 3)` 375 ms et 388 ms avec la lecture ; la borne `(2048, 2)` 770 ms : la borne de départ (une image d'animation sous 50 ms, la fin sous 1 s) est tenue. **V-V7-45 : la mesure native refaite par le juge** (son poste, 7 essais,
par la vitrine) : `(384, 2)` 13,2 à 14,2 ms (médianes des trois flocons) et 2,5 ms de lecture ; `(1024, 3)` 190 à 193 ms et 18 ms ; `(1024, 4)` 330 à 347 ms ; `(2048, 2)` 358 à 369 ms et 71 ms ; 20 à 24 ns par échantillon dès `sur >= 2`, 33 ns à `sur = 1` ;
`givre_avancer(5)` 0,39, 0,74 et 1,82 ms (plaque, étoile, dendrite, au milieu du scénario). **V-V7-46 : le champ en chiffres** (le `t` du dégradé des cases gelées à la fin du scénario, mesuré) : plaque 1 261 cases, de 256 à 1024, médiane 486 ; étoile 2 263 cases, de 257
à 1024, médiane 488 ; dendrite 1 513 cases, de 259 à 1024, médiane 521, 7 cases saturées (la masse d'une case gelée tard peut dépasser celle du germe).

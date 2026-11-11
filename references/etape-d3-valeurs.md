# Étape D3 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape D3 (le claquage diélectrique sur la page : les entrées de la bibliothèque qui rendent une erreur, les modèles 4 à 7 de la vitrine, la courbe `D(η)` calculée un amas
par appel, la page) attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque identifiant (`V-D3-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_d3_*.rs`). Elle est séparée des
résultats figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du prototype de l'écrivain.** Les seules valeurs tirées du prototype sont les mesures dites mesurées
(V-D3-40 à V-D3-44), qu'aucun test ne fige. Une valeur vient d'une convention que la doc de `src/courbe.rs` ou de `src/vitrine.rs` fixe (dite telle, avec sa raison), d'une arithmétique entière refaite à la main, d'un calcul
**naïf** écrit dans les tests (`etape_d2_commun`, d'après la doc seule de D2, qui n'appelle ni `courbe` ni `correlation`), d'un outil indépendant (nommé), ou d'une publication lue.

**Outils indépendants** (Python 3.13, dans `essai-d3-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `d1_ref.py` (étape D1, copié) fait croître les amas de claquage (PCG32, relaxation rouge-noir,
tirage) d'après la doc de `src/claquage.rs`, validé à D1 contre le prototype Rust et un second prototype ; `d2_ref.py` (étape D2, copié) donne la fenêtre des petits amas, la corrélation cumulée `C(r)` en fractions exactes et
la dimension en `decimal` à 60 chiffres, d'après la doc de `src/courbe.rs` seule. `d2_ref.py amas <η> <graine> <demi-côté> <sites>` donne la dimension de chacun des quatre modèles de la vitrine à la graine 1 (V-D3-35).

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Tout test de D3 est **exact** : égalité d'entiers, de chaînes, d'octets ou de bits (`f64::to_bits`). La doc fixe chaque opération et leur ordre ; une implémentation fidèle donne les mêmes bits. Seule exception : V-D3-35
   (une dimension jugée contre l'outil Python indépendant à `1e-12`, la précision de `math::logarithme` étant `5e-16` en relatif).
2. Les entrées neuves de la bibliothèque sont jugées contre **D2** : un point, une dimension, un demi-côté égaux **au bit** à ceux de `point_de_la_courbe`, `dimension_d_un_amas` et `demi_cote_de_la_courbe` quand rien n'échoue,
   et contre le calcul naïf de D2 pour une partie des cas (le coût d'un amas de claquage interdit de tout recalculer trois fois).
3. Un code d'erreur, un texte d'erreur, une borne sont une **convention de la doc** : la doc fixe la table, le test la recopie à la main. Les bornes de la bibliothèque sont celles de D1 (`ETA_MAX = 8`, `DEMI_COTE_MAX = 255`,
   `PARTICULES_MAX = 100 000`) et de la règle de D2 (`ETA_COURBE_MAX = 3`).
4. **Aucune valeur publiée de `D(η)` n'est un test** : ni Hastings 2001 (`η = 2 → 1,433`, `η = 3 → 1,263`), ni la DLA à 1,71. Seule leur TRANSCRIPTION dans `courbe.rs` est testée (D2, V-D2-11) ; D3 teste que le module et la page
   les **lisent** dans la bibliothèque (V-D3-22, V-D3-33) et ne les écrivent pas. Un test « proche de 1,433 » se règlerait pour passer.
5. Les tailles des tests viennent du **coût mesuré** (`MODELE.md`, « Le coût », étape D3) : 40 sites pour la plupart des courbes, 100 (la borne) pour deux cas, les modèles joués en entier seulement pour `η = 0` et `η = 3`.
6. **Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests de la page sont de la **forme** (régression de texte), sauf `lecture_pure_dit_ce_que_la_doc_dit`.

---

## La bibliothèque (`etape_d3_erreurs.rs`)

**V-D3-01 : chaque erreur est provoquée à la main** (`chaque_erreur_est_provoquee_a_la_main`).
- Valeur : `GrainesInsuffisantes { donnees: 0 }` et `{ 1 }` ; `EtaTropGrand { eta, maximum: 8 }` pour `η` de 9, 100, `u32::MAX` (et `maximum: 3` pour le demi-côté, `η` de 4, 5, 8, 9, `u32::MAX`) ; `DemiCoteHorsBorne` de 256, 1000,
  `u32::MAX` ; `TailleHorsBorne` de 100 001, 1 000 000, `usize::MAX` ; `FrontiereTouchee { graine }` (`η = 1`, taille 40, demi-côté 3 : le domaine a 29 sites ; `η = 0`, taille 100, demi-côté 5 : 81 sites) ; `FenetreVide { graine }` pour
  une taille de 0, 1 et 5 ; les bornes incluses (`η = 8`, demi-côté 255, taille 100 000) ne sont pas leur erreur de borne.
- Source : convention de la doc de `src/courbe.rs` (« Les entrées qui rendent une erreur ») ; les bornes sont celles de D1 (`ETA_MAX`, `DEMI_COTE_MAX`, `PARTICULES_MAX`, relues dans `src/claquage.rs`) ; le nombre de sites d'un domaine
  est un compte de cases `x² + y² <= d²` (29 pour `d = 3`, 81 pour `d = 5` : A000328 de l'OEIS, 1, 5, 13, 29, 49, 81…, lue à Z1) ; un amas de moins de 8 d'étendue n'a pas de fenêtre (D2, V-D2-01).
- Tolérance : aucune. Critère : l'égalité exacte de l'`Err`.

**V-D3-02 : l'ordre des vérifications** (`l_ordre_des_verifications`).
- Valeur : pour un point, les graines puis `η`, puis le demi-côté, puis la taille ; pour une graine, `η`, demi-côté, taille ; pour le demi-côté, `η` puis taille. Chaque cas donne une erreur de paramètre sans faire croître d'amas.
- Source : convention de la doc (« L'ordre des vérifications ») ; raison : une erreur de paramètre se rend avant d'avoir payé le coût d'un amas, et l'ordre fixé rend l'erreur déterministe quand plusieurs paramètres sont absurdes.
- Tolérance : aucune.

**V-D3-03 : le demi-côté essayé** (`le_demi_cote_essaye_est_celui_de_la_regle`).
- Valeur : `min(255, A + B · taille / 100)` avec `A = [30, 32, 22, 12]`, `B = [10, 8, 18, 36]`, en entiers, pour `η` de 0 à 3 et les tailles 0, 1, 7, 50, 99, 100, 101, 250, 300, 1000, 20 000, 100 000 (par exemple `η = 0`, 100 sites :
  `30 + 10 · 100 / 100 = 40` ; `η = 3`, 100 sites : `12 + 36 = 48` ; `η = 3`, 1000 sites : `12 + 360 = 372`, plafonné à 255) ; `TailleHorsBorne` pour `usize::MAX / 10 + 1`, où la règle de D2 déborde.
- Source : la règle de D2 (`references/etape-d2-valeurs.md`, V-D2-xx : convention de la doc, arithmétique entière refaite à la main) ; le test l'écrit en dur (`etape_d2_commun::demi_cote_doc`).
- Tolérance : aucune.

**V-D3-04 : le point essayé est celui de D2** (`le_point_essaye_egale_le_point_de_d2_a_to_bits_pres`).
- Valeur : `eta`, `taille`, `graines`, les bits de la moyenne et de l'erreur-type de `essayer_point_de_la_courbe` égaux à ceux de `point_de_la_courbe` et (`η = 1`, `η = 3`, et les cas d'`η = 0`) à ceux du calcul naïf
  `etape_d2_commun::point_naif` ; cas : `η` de 0 à 3 à 40 sites, graines `[1, 2]` ; `η = 2` à demi-côté 33 ; `η = 0` graines `[5, 5, 6]` et `[3, 1, 2]` à 60 sites ; `η = 0` huit graines.
- Source : D2 (V-D2-05 à V-D2-09 : le point, la moyenne et l'erreur-type) ; le calcul naïf de D2 est écrit d'après la doc seule et validé contre `d2_ref.py` à D2.
- Tolérance : aucune (bits).

**V-D3-05 : la dimension d'une graine** (`la_dimension_d_une_graine_est_celle_du_calcul_naif`).
- Valeur : pour `η` de 0 à 3, à 40 sites, graines 1 à 3, `essayer_dimension_d_une_graine` = `dimension_naive` (naïf de D2) = `dimension_d_un_amas`, au bit près ; les trois dimensions de `η = 1` (graines 1 à 3) et les quatre de la graine 1
  (`η` de 0 à 3) sont différentes entre elles.
- Source : D2 (V-D2-02 à V-D2-04), validé contre `d2_ref.py`. Tolérance : aucune. Ajout du juge : `η = 5` (graine 1, 40 sites, demi-côté 40) est accepté et donne la dimension naïve de cet `η`, différente de celle de `η = 3` (un `η` plafonné à 3 dans la croissance, mutant M03 du juge, survivait).

**V-D3-06 : la frontière touchée au dernier site** (`la_frontiere_touchee_au_dernier_site_est_acceptee`).
- Valeur : pour `η = 0`, demi-côté 12, graine 4, soit `n` le nombre de sites au moment où `Claquage::touche_la_frontiere` devient vrai (trouvé dans le test par le code natif de D1) : `taille = n` est acceptée, `n + 1` donne
  `FrontiereTouchee { graine: 4 }`.
- Source : D1 (« L'arrêt » : le site qui touche est ajouté, c'est le pas suivant qui rend `None`) et D2 (« un amas qui touche la frontière EXACTEMENT au dernier site est accepté »). Tolérance : aucune.

**V-D3-07 : le texte de chaque erreur** (`le_texte_de_chaque_erreur`).
- Valeur : les six textes de la table de la doc de `ErreurCourbe` (« au moins deux graines sont nécessaires, 1 données », « η = 9 : au-delà du maximum 8 », « demi-côté 256 : au-delà du maximum 255 », « taille 100001 : au-delà du maximum
  100000 », « la graine 7 : l'amas touche la frontière avant d'atteindre sa taille », « la graine 3 : aucune dimension (fenêtre vide ou C(r) nul) »).
- Source : convention de la doc. Tolérance : aucune. Critère : l'égalité exacte des chaînes.

**V-D3-08 : aucun cas absurde ne panique** (`aucun_cas_absurde_ne_panique`).
- Valeur : aucune panique (`catch_unwind`) pour `η` de 0, 9, `u32::MAX`, demi-côté de 0, 1, 255, 256, `u32::MAX`, taille de 0, 1, 5, 100 001, `usize::MAX`, graines `[]`, `[1]`, `[1, 2]` ; `demi_cote = 0` : le germe touche la
  frontière (`taille = 5` donne la frontière, `taille = 1` la fenêtre vide) ; `η = 8` accepté.
- Source : convention de la doc (« jamais une panique, jamais un débordement silencieux ») ; le cas `usize::MAX / 10 + 1` est celui que la relecture de la PR 29 a signalé (`demi_cote_de_la_courbe` déborde). Tolérance : aucune.

**V-D3-09 : la mesure des petits amas** (`la_mesure_des_petits_amas_est_le_calcul_naif`).
- Valeur : pour un trait de 30 et de 61 sites, un carré de 12 de côté, une croix de bras 15 et trois amas de claquage de 60 sites (`η` = 0, 2, 3, graine 2, demi-côté 40) : `rayons` = `2..=max(4, e / 6)` ; `valeurs` = `C(r)` du calcul
  naïf de D2 ; `dimension` = `dimension_naive` = `dimension_d_un_amas` (bits). `None` pour une liste vide, un site, un trait de 5 sites (étendue 4 : fenêtre vide) et deux sites à distance 20 (`C(r) = 0`).
- Source : D2 (fenêtre : V-D2-01 ; `C(r)` : V-D2-02, recalculé par `d2_ref.py` en fractions exactes). Tolérance : aucune.

## Les modèles 4 à 7 et la courbe dans la vitrine (`etape_d3_vitrine.rs`)

**V-D3-10 : la table des quatre modèles** (`les_quatre_modeles_de_claquage_au_depart`).
- Valeur : modèle 4 : `η = 0`, 400 sites, demi-côté 70, durée 399 ; modèle 5 : `η = 1`, 250, 52, 249 ; modèle 6 : `η = 2`, 150, 49, 149 ; modèle 7 : `η = 3`, 100, 48, 99 ; au temps 0 un site, rayon 0, compacité `NaN`, image de 512 × 512,
  l'état de `Claquage::new(graine, demi-côté, η)` (graines 1, `0xDEAD_BEEF`, `u32::MAX`) ; les trois empreintes d'un modèle sont différentes.
- Source : convention de la doc de `src/vitrine.rs` (« Les huit modèles »). Le demi-côté est la règle de D2 (V-D3-03) : `30 + 10 · 400 / 100 = 70`, `32 + 8 · 250 / 100 = 52`, `22 + 18 · 150 / 100 = 49`, `12 + 36 · 100 / 100 = 48`.
  Les tailles viennent du coût mesuré (V-D3-40). L'état natif est celui de la bibliothèque de D1 (empreintes figées à D1 : `references/claquage-*-graine-1.empreinte`).
- Tolérance : aucune.

**V-D3-11 : l'état par morceaux** (`l_etat_suit_le_natif_par_morceaux`). Valeur : l'état natif de `Claquage::new(7, demi-côté, η)` après 1, 1, 7, 20, 20 et 45 unités (morceaux 1, 0, 6, 13, 0, 25) : temps, durée, image (chaque
pixel, après 1, 20 et 45 unités), sites, rayon, compacité `NaN`, huit mots et empreinte entière ; l'état d'un seul appel de 45 est le même. Source : D1 (« la dynamique ne dépend pas du découpage ») et la doc. Tolérance : aucune.

**V-D3-12 : l'arrêt à la durée** (`le_claquage_s_arrete_a_sa_duree`). Valeur : `η = 0` (399) et `η = 3` (99) : `givre_avancer(u32::MAX)` rend la durée, puis 0 ; l'état est celui du natif joué jusqu'au bout. Source : la doc
(« `givre_avancer` ne joue jamais au-delà »). Tolérance : aucune.

**V-D3-13 : rien derrière un claquage** (`un_claquage_ne_laisse_rien_derriere_lui`). Valeur : la DLA de graine 1 jouée jusqu'au bout après un claquage a l'empreinte de `references/dla-graine-1.empreinte` ; la plaque,
celle de `references/reiter-plaque.empreinte` ; le modèle 8 est refusé sans rien changer. Source : ces deux fichiers (régressions de notre code, recoupées par `etape_5_vitrine`). Tolérance : aucune.

**V-D3-14 : la mesure du claquage** (`la_vitrine_mesure_le_claquage_comme_la_bibliotheque`). Valeur : pour `η = 0` (graine 1, 400 sites), `η = 1` (graine 2, 60 sites), `η = 2` (graine 3, 60 sites), `η = 3` (graine 4, 100 sites) :
nombre de points, rayons (2, 3, 4, …), `C(r)` et dimension du calcul **naïf** de D2 (bits). Source : D2 (V-D2-01, V-D2-02, V-D2-04). Tolérance : aucune.

**V-D3-15 : pas de mesure** (`pas_de_mesure_du_claquage`). Valeur : un site : pas de mesure ; `n0 - 1` sites (`n0` le premier nombre de sites d'étendue 8 au moins, trouvé par le code natif pour `η = 0` et `η = 2`, graine 1) : pas de
mesure ; `n0` sites : 3 points (rayons 2 à 4). Source : la fenêtre de D2 (V-D2-01 : vide pour `e <= 7`, `[2, 3, 4]` pour `8 <= e <= 29`). Tolérance : aucune.

**V-D3-16 : la mesure suit l'amas** (`la_mesure_du_claquage_suit_l_amas`). Valeur : comme V-Z1-25 pour la DLA : `givre_avancer(0)` ne l'efface pas, `givre_avancer(1)` l'efface, un `givre_avancer` qui ne joue rien (fin du scénario) ne l'efface
pas, `givre_demarrer` l'efface. Source : la doc, et Z1 (V-Z1-25). Tolérance : aucune.

**V-D3-17 : la courbe égale la bibliothèque** (`la_courbe_egale_la_bibliotheque_bit_a_bit`). Valeur : `givre_courbe_demarrer(40, 3)` rend 0, `fait` 0, `total` 12 ; chaque pas rend 0 et ajoute 1 à `fait` ; la moyenne et l'erreur-type d'un `η`
sont `NaN` jusqu'à son 3e, 6e, 9e, 12e pas puis égales, au bit, à `essayer_point_de_la_courbe(η, [1, 2, 3], 40, règle)` (et au calcul naïf de D2 pour `η = 1` et `3`) ; un travail de 100 sites et 2 graines donne les points de la
bibliothèque pour `η = 1` et `3`. Source : la doc (« Le travail ») et D2. Tolérance : aucune.

**V-D3-18 : l'ordre des graines et des `η`** (`les_graines_et_les_eta_dans_l_ordre`). Valeur : `η = 0` est fini après exactement `graines` pas ; sa moyenne est celle des graines `[1, 2]` (différente de celle de `[0, 1]` et `[2, 3]`,
vérifié) ; huit graines à 36 sites : les bits de la bibliothèque, dont la moyenne dans l'ordre inverse diffère pour au moins un `η` (vérifié). Source : la doc. Tolérance : aucune.
Mesuré en préparant le test : à 36 sites les graines 1 à 8 des quatre `η` ont toutes une dimension ; à 32 sites, la graine 1 de `η = 0` et la graine 5 de `η = 1` n'en ont pas (étendue < 8), à 30 sites aussi la graine 7 de `η = 0`.

**V-D3-19 : les codes d'erreur** (`les_codes_d_erreur_de_la_courbe`). Valeur : `taille` hors de 1 à 100 → 1 ; `graines` hors de 2 à 8 → 2 (la taille d'abord) ; aucun travail après un refus ; `givre_courbe_pas` sans travail → 5 ; (100, 8) → total 32 ;
(1, 2) → 8 ; (5, 3) et (1, 2) : le premier pas rend 4, `fait` reste 0, le travail est arrêté. Source : la table de la doc de `src/vitrine.rs` (« Les codes d'erreur »). Tolérance : aucune. Les codes 3 et 6 ne sont pas atteignables avec les bornes
(voir `MODELE.md`).

**V-D3-20 : un nouveau démarrage efface** (`un_nouveau_demarrage_efface_le_travail`). **V-D3-21 : indépendance du modèle et de la courbe** (`le_modele_et_la_courbe_sont_independants`). Valeur : la doc (« Un nouveau `givre_courbe_demarrer`
efface l'ancien travail ; `givre_demarrer` ne touche pas au travail »). Tolérance : aucune.

**V-D3-22 : les repères** (`les_reperes_viennent_de_la_bibliotheque`). Valeur : `givre_courbe_repere(η)` = `1,71`, `1,433`, `1,263` pour `η = 1, 2, 3` (au bit) et `NaN` pour `η = 0`, 4, 5, 100, `u32::MAX` ; le texte de `src/vitrine.rs`
n'écrit aucune de ces valeurs. Source : les repères de D2 (V-D2-11 : Hastings, PRL 87, 175502 (2001), arXiv:cond-mat/0103312v2, tableau II, colonne « full set » ; Mathiesen, Jensen, Bakke, arXiv:0712.1739, section IV pour 1,71), **non relus ici**
(D2 les a lus). **Repères, non testés** : aucun test ne compare un `D` à une de ces valeurs. Tolérance : aucune.

**V-D3-23 : aucun paramètre absurde ne fait paniquer** (`aucun_parametre_absurde_ne_fait_paniquer`). Valeur : aucune panique (une panique abattrait le binaire de test) pour les paramètres absurdes des huit fonctions de la courbe et de `givre_demarrer`. Source : la doc.

## Le module sous Node (`etape_d3_module.rs`)

**V-D3-24 : Node lit le claquage** (`node_lit_le_claquage_comme_le_natif`). Valeur : la sortie entière de `modeles-wasm.mjs` pour (4, 1, 399), (5, 2, 60), (6, `0xDEAD_BEEF`, 45), (7, `u32::MAX`, 99), (7, 3, 0), (4, 5, 1000) : l'état de
`Claquage::new` natif, l'empreinte, le SHA-256 de l'image, la mesure du calcul naïf de D2 (bits), `apres`. Source : D1, D2 et la doc. Tolérance : aucune. **Ne passe qu'une fois le module reconstruit.**

**V-D3-25 : Node calcule la courbe** (`node_calcule_la_courbe_comme_le_natif`). Valeur : la sortie entière de `courbe-wasm.mjs` pour (40, 3), (36, 8) et (5, 2) : les codes de refus `1,1,1,2,2,2,2,1`, l'état sans travail `0,0,1;5`, l'ordre des amas
(`η:graine`), les points de `essayer_point_de_la_courbe` et leurs repères, un pas de plus, un nouveau démarrage. Source : la doc. Tolérance : aucune. **Ne passe qu'une fois le module reconstruit.**

**V-D3-26 : les scripts refusent des arguments faux** (forme).

## La page (`etape_d3_page.rs`)

**V-D3-27 à V-D3-33 et V-D3-37** : la forme de la page (éléments, phrases, blocs du zinc, un amas par tour, unité par unité, appui sur `lecture.js`, aucune valeur publiée, le curseur `η` relie `relancer`). Source : la consigne de la doc de tête de `etape_d3_page.rs`. Ce sont des tests de forme. Ajout du juge à V-D3-30 : les deux fonctions de la courbe (le démarrage et le pas) appellent `texteAvancement(` (le pas qui écrit l'avancement lui-même, mutant P10 du juge, survivait).

**V-D3-34 : la lecture pure** (`lecture_pure_dit_ce_que_la_doc_dit`). Valeur, calculée à la main : `numeroDuModele` : 0, 1, 2, 3 pour les choix « 0 » à « 3 » et `4 + η` pour « claquage » ; `blocsCaches` : le zinc caché pour « claquage » seulement, le repère de `η` et le curseur cachés pour tout autre choix ; `formater` : `1,234` (1,2344), `1,235`
(1,2346), `0,000`, `2,000` ; « η = 2, graine 3 sur 6 » pour 14 amas faits sur 24 (le 15e amas : `⌊14 / 6⌋ = 2`, `14 mod 6 + 1 = 3`) ; l'écart `+0,100` pour `1,6 - 1,5` et `−0,125` pour `1,125 - 1,25` ; « pas de repère publié pour η = 0 » pour
un repère `NaN`. Tolérance : aucune.

## Un outil indépendant (`etape_d3_vitrine.rs`)

**V-D3-35 : la dimension des quatre modèles, graine 1, contre Python** (`la_dimension_des_quatre_modeles_est_celle_de_python`).
- Valeur : `D` de `d2_ref.py amas <η> 1 <demi-côté> <sites>` : modèle 4 (`η = 0`, 400 sites, demi-côté 70, étendue 28) 1,89281132677495956025 ; modèle 5 (`η = 1`, 250, 52, étendue 32) 1,74493588341019327296 ; modèle 6 (`η = 2`, 150, 49, étendue 33)
  1,59607404588882589411 ; modèle 7 (`η = 3`, 100, 48, étendue 29) 1,38303932787124472016. Le test les écrit au `f64` le plus proche (1,8928113267749596 ; 1,7449358834101933 ; 1,5960740458888258 ; 1,3830393278712447).
- Source : `d1_ref.py` (le claquage, écrit d'après la doc de `src/claquage.rs` seule, validé à D1 contre le prototype Rust et un second prototype) et `d2_ref.py` (la fenêtre des petits amas, `C(r)` en fractions exactes, la dimension en `decimal` à 60 chiffres),
  lancés ici le 10/11 ; sorties brutes dans `essai-d3-outils/python-modele-eta{0,1,2,3}.txt`. Le prototype Rust donne les mêmes valeurs (à `1e-12`, vérifié par ce test).
- Tolérance : `1e-12` (la précision annoncée de `math::logarithme` est `5e-16` en relatif ; la pente de 3 à 5 points ne l'amplifie pas). Critère : `|D - D Python| <= 1e-12` et `givre_sites()` égal aux sites visés.

## Les tests des étapes 5 et Z1 amendés

**V-D3-36** (amendements, pas de valeur) : voir le rapport de l'écrivain, « Les tests des étapes 5 et Z1 que j'ai amendés ».

## Les mesures (dites mesurées, jamais figées par un test)

**V-D3-40 : le coût d'un modèle et d'un amas** — voir `MODELE.md` (« Le coût », étape D3) et `essai-d3-outils/mesure-modeles-120.txt`, `mesure-courbe-100-6.txt`. **V-D3-41** : la marge au bord (rayon maximal contre demi-côté) sur 120 graines. **V-D3-42** : la
dimension mesurée des modèles à leur taille et son écart au repère. **V-D3-43** : la courbe de la page (100 sites, 6 graines). **V-D3-44** : la sensibilité de la fenêtre aux petites tailles (à 32 sites, des graines sans dimension).
Les valeurs sont dans `MODELE.md`.

**V-D3-45 (juge, mesuré, jamais figé par un test)** : tout l'espace de la courbe de la page (`η` 0 à 3 × graines 1 à 8 × taille 1 à 100, 3200 amas, natif) : aucune frontière, aucune autre erreur, aucune panique ; fenêtre vide jusqu'à 32 sites seulement. **V-D3-46 (juge, mesuré)** : les modèles 4 à 7 sur les graines 1 à 2000, 300, 500, 500 et 7001 à 7050 : aucune frontière, aucune panique ; rayon maximal 24, 30, 37, 45 (marge 46, 22, 12, **3**) ; temps et pire unité dans `MODELE.md` (« Les modèles 4 à 7 et leurs bornes »). Outil : `essai-d3-juge-outils/juge_fouille.rs`, sortie `sorties/fouille.txt`.

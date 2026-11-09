# Étape D2 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape D2 (la courbe `D(η)` du claquage diélectrique, à 100 sites, à côté des points publiés de Hastings 2001) attend, d'où elle vient,
avec quelle tolérance et quel critère de réussite. Chaque identifiant (`V-D2-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_d2_*.rs`). Elle est séparée des résultats
figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du prototype de l'écrivain**. Les seules valeurs tirées du prototype sont les bandes
statistiques de la courbe (V-D2-18 à V-D2-20, étiquetées « régression ») et les mesures de coût et de sensibilité, dites mesurées. Une valeur vient d'une publication lue (dite lue, ou dite non lue),
d'un calcul exact fait par un outil indépendant (nommé), ou d'une convention que la doc de `src/courbe.rs` fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-d2-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `d2_ref.py`, écrit d'après la **doc seule** de `src/courbe.rs`, sans
lire le code Rust : l'étendue, la fenêtre des petits amas, la corrélation cumulée `C(r)` en **fractions exactes** (les paires comptées une à une, le disque compté case par case), la dimension
`2 + pente(ln r, ln C)` en `decimal` à 60 chiffres (logarithmes exacts), la moyenne, la variance corrigée et l'erreur-type en `decimal`, et les formes (trait, diagonale, carré, disque, croix,
fractale de Vicsek, tapis de Sierpinski) ; `d1_ref.py` (étape D1, copié) fait croître les amas de claquage (PCG32, relaxation rouge-noir, tirage) d'après la doc de `src/claquage.rs`, validé à D1
contre le prototype Rust et un second prototype ; `chercher_stats.py` cherche des listes de valeurs où chaque faute de la moyenne et de l'erreur-type change les bits. Le prototype Rust et `d2_ref.py`
donnent les mêmes dimensions à `1e-15` près sur tous les cas listés ci-dessous.

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Les tests « exacts » (la fenêtre, les bits de la moyenne et de l'erreur-type, le CSV, les pixels de l'image, les refus) n'ont **aucune** tolérance : égalité d'entiers, de chaînes, de couleurs ou de bits
   (`f64::to_bits`). La doc fixe chaque opération et leur ordre ; une implémentation fidèle donne les mêmes bits.
2. Une dimension est jugée contre l'outil Python indépendant (`decimal`, logarithmes exacts) à `1e-12` : la précision annoncée de `math::logarithme` est `5e-16` en relatif, et la pente de 3 à 15 points
   ne l'amplifie pas au-delà de `1e-14` (mesuré : l'écart prototype / Python est de l'ordre de `1e-15`). Aucune autre tolérance n'est réglée.
3. Une propriété vraie seulement en moyenne (l'ordre et la baisse de `D(η)`) se teste en moyenne sur 24 graines fixes (1 à 24), avec une borne `moyenne mesurée - 3 σ / √24` (pour un écart de deux séries :
   `σ_écart`), mesurée sur **170 graines** : 1 à 120 (cinq blocs de 24) et 1001 à 1050 (deux blocs de 25, graines neuves) ; les sept blocs doivent tous la respecter. Aucune graine n'est choisie ; aucune borne
   n'est élargie (arrondie vers l'extérieur au chiffre significatif utile seulement).
4. **Aucune valeur publiée de `D(η)` n'est un test** : ni Hastings 2001 (`η = 2 → 1,433`, `η = 3 → 1,263`), ni la DLA à 1,71. Un test « proche de 1,433 » se règlerait pour passer (la leçon du zinc : la
   fenêtre seule déplace `D` de ±0,05, plus que l'écart mesuré à Hastings à 300 sites ; voir `MODELE.md`). Seule la TRANSCRIPTION des repères (V-D2-11) et l'arithmétique de l'écart (V-D2-12) sont testées.
5. Les tailles de test viennent du **coût mesuré** (`MODELE.md`, « Le coût ») : 100 sites pour la courbe, de 40 à 100 pour les cas exacts.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests `régression :` de l'étape D2 sont `etape_d2_ordre.rs::{la_dimension_baisse_avec_eta,
la_dimension_de_chaque_eta_est_dans_sa_bande, courbe_de_100_sites_figee}`.

---

## La fenêtre et la dimension (`etape_d2_exact.rs`)

**V-D2-01 : la fenêtre suit sa formule** (`fenetre_suit_sa_formule`).
- Valeur : `2..=max(4, e / 6)`, vide si ce dernier rayon dépasse `e / 2` ou 4096 ; à la main : vide pour `e <= 7`, `[2, 3, 4]` de 8 à 29, `[2..=5]` de 30 à 35, `[2..=6]` de 36 à 41, `[2..=7]` à 42, `[2..=49]` à 299 ;
  4095 rayons (`2..=4096`) à `e = 24 581`, vide à `e = 24 582` (`24 582 / 6 = 4097`) ; vide pour une étendue de `2^32 - 1` (sans panique).
- Source : convention de la doc (la formule et ses constantes `2`, `4`, `6` ; leur choix est dans `MODELE.md`, § « La fenêtre des petits amas »), arithmétique entière recalculée à la main (`299 / 6 = 49`,
  `24 581 / 6 = 4096`).
- Tolérance : aucune. Critère : égalité sur les 301 étendues de 0 à 300 et sur les cas à la main.

**V-D2-02 : la fenêtre ne dépend que de l'étendue** (`fenetre_ne_depend_que_de_l_etendue`). Étendues 8, 29, 30, 36, 60 ; cinq familles (deux sites, trait, L, cadre, nuage pseudo-aléatoire de 42 sites) × cinq
transformations (identité, translation, quart de tour, miroir, ordre inversé) + un site de plus : la même fenêtre. Source : la doc (« ne dépend que de l'étendue »). Tolérance : aucune.

**V-D2-03 : la dimension d'un amas est la corrélation sur la fenêtre** (`dimension_egale_la_correlation_sur_la_fenetre`, `dimension_est_none_quand_rien_n_est_mesurable`).
- Valeur : `dimension_d_un_amas(sites)` égale, à `to_bits` près, `correlation::dimension_correlation(sites, &fenêtre)` et un calcul naïf écrit dans `tests/etape_d2_commun/mod.rs` (paires comptées une à une,
  disque compté case par case, une division, `logarithme`, pente dans l'ordre de `mesures::pente`), pour six amas de claquage et six formes ; `None` pour : une liste vide, un site, sept sites en ligne (étendue 6),
  trois sites éloignés (`C(r) = 0` partout), `(0,0) (3,0) (50,0) (53,0)` (`C(2) = 0` seulement : une seule `C` nulle suffit), dix copies d'un site plus un site à distance 20 ; `Some` pour `(0,0) (2,0) (50,0) (52,0)`.
- Source : la doc (le naïf répète sa formule) ; les cas `None` sont des conséquences de la doc, comptées à la main (par exemple `(0,0) (3,0) (50,0) (53,0)` : étendue 53, dernier rayon `max(4, 8) = 8`, aucune paire
  à distance 1 ou 2 : `C(1) = C(2) = 0`).
- Tolérance : aucune (bits). Critère : égalité sur tous les cas.

**V-D2-04 : les dimensions de douze formes** (`dimension_des_formes_contre_l_outil_independant`).
Valeurs (outil Python `d2_ref.py formes`, `decimal` à 60 chiffres, ici à 20 chiffres ; sortie : `essai-d2-outils/formes_python.txt`) :

| forme | sites | étendue | fenêtre | `D` (formule de la doc) |
|---|---|---|---|---|
| trait de 40 | 40 | 39 | 2 à 6 | 0,91439743840635344080 |
| trait de 100 | 100 | 99 | 2 à 16 | 0,95069663289600982666 |
| trait de 300 | 300 | 299 | 2 à 49 | 0,96263433723286901094 |
| diagonale de 30 | 30 | 29 | 2 à 4 | 1,01442216063154429890 |
| carré 12 × 12 | 144 | 11 | 2 à 4 | 1,77469428606446259515 |
| carré 25 × 25 | 625 | 24 | 2 à 4 | 1,89879643836882331157 |
| disque `x² + y² <= 81` | 253 | 18 | 2 à 4 | 1,85669608154283987329 |
| croix de bras 9 | 37 | 18 | 2 à 4 | 1,14239058444035756910 |
| Vicsek niveau 3 | 125 | 26 | 2 à 4 | 1,48627979036320957879 |
| Vicsek niveau 4 | 625 | 80 | 2 à 13 | 1,48454307574721250413 |
| tapis de Sierpinski niveau 2 | 64 | 8 | 2 à 4 | 1,57984179410546682388 |
| tapis de Sierpinski niveau 3 | 512 | 26 | 2 à 4 | 1,79973139141809206546 |


(Source, tolérance et critère : communs à V-D2-04 et V-D2-05, écrits après V-D2-05.)

**V-D2-05 : les dimensions de quatre amas de claquage** (même test) : (`d1_ref.py` fait croître l'amas, `d2_ref.py amas` le mesure ; sortie `amas_python.txt`) `η = 1`, graine 3, `d = 16`, 40 sites : `1,41141998168969426559` ; `η = 2`, graine 5, `d = 20`, 40 sites : `1,44466870921962927584` ;
`η = 3`, graine 2, `d = 24`, 40 sites : `1,46789603090557125513` ; `η = 0`, graine 7, `d = 40`, 100 sites : `1,77017244388245259222`.
- Source (V-D2-04 et V-D2-05) : calcul exact par `d2_ref.py` (outil indépendant, nommé), d'après la doc seule. **Ce ne sont pas les dimensions exactes des formes** (un trait de 100 sites n'est pas à 1 : il est à 0,951) mais ce que la
  FORMULE donne ; l'erreur de la fenêtre aux dimensions de similarité (`ln 5 / ln 3 = 1,4650`, `ln 8 / ln 3 = 1,8928`) est dans `MODELE.md`.
- Tolérance (V-D2-04 et V-D2-05) : `1e-12`. Critère : `|D - valeur| <= 1e-12` pour chaque cas.

**V-D2-06 : la moyenne et l'erreur-type à la main et en bits** (`moyenne_et_erreur_type_a_la_main_et_en_bits`).
- Valeurs : `[1, 3]` → `(2, 1)` exactement (variance corrigée `(1 + 1) / 1 = 2`, `sqrt(2 / 2) = 1`) ; `[-1, 1]` → `(0, 1)` ; `[0,5 ; 0,5 ; 0,5]` → `(0,5 ; 0)` ; `[1, 2, 3, 4]` → moyenne `2,5`, variance `5/3`, erreur-type
  `sqrt(5/12) = 0,6454972243679028` ; `[1.103, 1.962, 1.04, 1.737, 1.679]` → `(1.5042000000000002, 0.18313585121433762)` ; `[1.887, 1.306, 1.898]` → `(1.697, 0.19552578687562755)`.
- Source : calcul à la main (les quatre premiers) ; Python (`float`, mêmes opérations dans le même ordre : sommes de gauche à droite, `/ n`, `/ (n - 1)`, `sqrt(variance / n)` ; `math.sqrt` est l'arrondi correct comme le `sqrt`
  de Rust) pour les deux dernières, que `chercher_stats.py` a trouvées parce que **chacune des quatre fautes** (moyenne à l'envers, carrés à l'envers, `sqrt(v) / sqrt(n)`, diviseur `n`) y donne d'autres bits ;
  le test recalcule ces fautes et exige qu'elles diffèrent.
- Tolérance : aucune (bits). Critère : égalité à `to_bits` près.

**V-D2-07 : le point de la courbe égal au calcul naïf** (`point_de_la_courbe_egal_au_calcul_naif`).
- Valeurs : cinq cas, bits du naïf (`tests/etape_d2_commun/mod.rs`) et valeurs de Python (`d2_ref.py point`, sorties `point_eta*.txt`) à `1e-12` : `η = 0`, 100 sites, `d = 40`, graines `[40, 6, 14, 24, 36, 10, 26, 4]` :
  moyenne `1,75940721196141779617`, erreur-type `0,01124606999129963763` (liste choisie pour que l'ordre inverse change les bits de la moyenne ET de l'erreur-type : le test le vérifie) ; graines `[3, 1, 4, 1, 5]`
  (la graine 1 répétée compte deux fois) : `1,76019032163920491708`, `0,00376707437205083079` ; `η = 1`, 40 sites, `d = 16`, `[2, 7, 11, 5]` : `1,56606965996878420184`, `0,04467801867048171457` ;
  `η = 2`, 40 sites, `d = 20`, `[5, 3, 9, 1, 6]` : `1,38484773965738595822`, `0,04578514750363673127` ; `η = 3`, 40 sites, `d = 24`, `[4, 10, 2, 8]` : `1,46788132072924277009`, `0,00984339510346256369`.
- Source : le naïf répète la doc ; Python (`decimal` à 60 chiffres pour D, la moyenne et l'erreur-type) recoupe, avec l'amas de `d1_ref.py`.
- Tolérance : aucune pour le naïf (bits) ; `1e-12` pour Python. Critère : égalité.

**V-D2-08 : le demi-côté de la courbe** (`demi_cote_de_la_courbe_suit_la_regle`). Valeur : `min(255, A + B * taille / 100)`, `A = [30, 32, 22, 12]`, `B = [10, 8, 18, 36]` (entiers) ; à 100 sites `40, 40, 40, 48`, à 300
sites `60, 56, 76, 120` ; `η = 3` à 674 sites : 254, à 675 : 255, à 700 : 255 ; `η = 2` à 101 sites : 40 ; `η = 4` : panique (« η »). Source : convention de la doc (la règle est MESURÉE, `MODELE.md`, § « Le demi-côté » : le
choix des `A` et `B` est le nôtre, la valeur testée est l'arithmétique). Tolérance : aucune.

**V-D2-09 : la courbe donne les points dans l'ordre donné** (`courbe_donne_les_points_dans_l_ordre_donne`). `courbe(&[3, 0, 2, 0], &[1, 2, 3], 60)` : quatre points d'`η` 3, 0, 2, 0 (ni trié ni dédoublonné),
chacun égal bit pour bit au naïf avec `demi_cote_doc(eta, 60)` ; `courbe(&[], …)` vide ; `courbe(&[2], …)` est le point d'`η = 2` de la grande courbe. Source : la doc. Tolérance : aucune.

**V-D2-10 : les refus** (`moyenne_et_erreur_type_refuse_*`, `un_point_demande_deux_graines`, `l_amas_arrete_a_la_frontiere_est_dit`, `une_fenetre_vide_est_dite`). Panique : moins de deux valeurs (« au moins deux ») ; moins de deux graines
(« au moins deux graines ») ; un amas qui touche la frontière avant `taille` (« frontière » : `η = 1`, `d = 3`, 40 sites ; `η = 0`, `d = 5`, 100 sites) ; une fenêtre vide (« fenêtre » : `taille` 1 ou 5 à `η = 0`). Source :
la doc (« Les refus »). Tolérance : aucune (le message contient le mot).

## Les repères publiés (`etape_d2_fautes.rs`, `etape_d2_exemple.rs`)

**V-D2-11 : la transcription des repères** (`reperes_publies_transcrits`). **Testée** : `HASTINGS_2001_TABLE_II = [(2.0, 1.433), (3.0, 1.263)]` (bits, dans cet ordre), `DLA_RAPPEL = 1.71`, `publie(η)` = `Some(1.71)`,
`Some(1.433)`, `Some(1.263)` pour `η = 1, 2, 3` et `None` pour 0 et 4 à 8.
- Source : **M. B. Hastings, *Fractal to Nonfractal Phase Transition in the Dielectric Breakdown Model*, arXiv:cond-mat/0103312v2 (1 juin 2001), Phys. Rev. Lett. 87, 175502 (2001)** : le **PDF de l'arXiv v2**
  est téléchargé et son texte extrait le 09/11 (`pypdf`, extrait dans les notes) ; lu : le **tableau II** (« For various values of η, calculated dimension based on different parts of data set ; also, rms fluctuations in
  F1 after last step, normalized by F1 »), colonnes « η, full set, last 1/2, last 1/4, last 1/8, rms » : `η = 2` : `1.433`, `1.4256`, `1.435`, `1.4522`, `.039` ; `η = 3` : `1.263`, `1.264`, `1.262`, `1.243`, `.056` ;
  `η = 3,5` : `1.170`, `1.143`, `1.155`, `1.162`, `.078` ; `η = 4` : `1.128`, `1.090`, `1.078`, `1.071`, `.072` ; `η = 4,5` : `1.101`, `1.090`, `1.066`, `1.035`, `.039` ; `η = 5` : `1.068`, `1.030`, `1.025`, `1.009`, `.046`.
  **Aucune barre d'erreur** (la dernière colonne est la fluctuation rms de `F_1` après le pas final). Méthode et taille (section « Results ») : applications conformes itérées (« free of grid effects and boundary effects »,
  introduction), `D` par régression linéaire de `log F_1` contre `log n` sur la moyenne des réalisations (`F_1 ∝ n^{1/D}`) ; `η = 3, 3,5, 4, 4,5, 5` : 13 à 14 réalisations de `N = 10 000` particules, `T = 500` ; `η = 2` :
  8 réalisations de `N = 15 000`, `T = 200`. Introduction : « ranging from 2 (η = 0, the Eden model), to roughly 1.7 (η = 1, DLA), to 1 ». **Seules les valeurs `η = 2` et `3`, colonne « full set », sont transcrites en constantes** ; les
  autres cases ne sont pas dans le code.
- Source de `1,71` : **J. Mathiesen, M. H. Jensen, J. Ø. Bakke, *Dimensions, maximal growth sites and optimization in the dielectric breakdown model*, arXiv:0712.1739, section IV** (ar5iv, lu par l'écrivain le 09/11) : « the value of
  the dimension decreases smoothly with η, from the DLA value D=1.71 for η=1 down towards D∼1 for η→∞ » ; aucune valeur de `η = 2` ou `3` dans le texte ni les légendes (seulement `D = 1,10` pour `η = 4`, légende de la figure 3) ;
  **la revue et l'année ne sont pas lues**. Hastings, lui, écrit « roughly 1.7 » : **`1,71` n'est pas une mesure de Hastings**.
- Tolérance : aucune (bits). Critère : égalité. **Un repère transposé échoue** (`(2.0, 1.263)`).

**V-D2-12 : l'arithmétique de l'écart** (`ecart_et_reperes_affiches`). `ecart(point, publie)` est `point.moyenne - publie` à `to_bits` près pour `(1,5 ; 1,433)`, `(1,2 ; 1,263)`, `(1,433 ; 1,433)`, `(1,7 ; 1,71)` ; positif quand le banc est au-dessus,
négatif dessous, nul à égalité. Source : l'arithmétique (une soustraction). Tolérance : aucune. **Ce test ne compare jamais une mesure à un repère.**

## Le CSV et l'image (`etape_d2_exemple.rs`)

**V-D2-13 : le CSV à la main** (`csv_a_la_main`). Sept points écrits à la main donnent le texte du test (`1.650000,0.010000,1.710,-0.060000` pour `η = 1` : `1,65 - 1,71 = -0,06` ; `1.500000,0.012500,1.433,0.067000` : `1,5 - 1,433 = 0,067` ;
`1.350000,0.012000,1.263,0.087000` : `1,35 - 1,263 = 0,087` ; `1.2345678` écrit `1.234568`, `0.0000004` écrit `0.000000`) ; **ajout du juge** : `η = 1`, moyenne `1.7600015`, erreur `0.0000005` donne `1,100,24,1.760001,0.000000,1.710,0.050002`
(source : le formatage `{:.6}` de Rust, qui arrondit la valeur décimale EXACTE du double : `0.0000005` est le double `4,99999999999999977…e-7`, sous la moitié ; et l'écart `1.7600015 - 1.71`
est calculé sur la moyenne non arrondie, `0,0500015…` en double, écrit `0.050002` ; vérifié par un petit programme `rustc` du juge, `essai-d2-juge-outils/arrondi.rs` et `ecart.rs`, pas par le prototype). Source : la doc (six décimales, trois pour le repère) et l'arithmétique à la main. Tolérance : aucune (chaîne).

**V-D2-14 : la forme du CSV d'une courbe réelle** (`csv_d_une_courbe_reelle`) : l'en-tête, quatre lignes de sept champs, `D` et `erreur` égaux aux six décimales de la moyenne et de l'erreur-type du point, `publie` vide pour `η = 0`, `ecart =
D - publie` à six décimales. Forme seulement (aucune valeur de `D` attendue). Tolérance : aucune.

**V-D2-15 : l'image à la main** (`image_a_la_main`). 640 × 320 ; fond blanc aux coins ; centres, coins des carrés, barres et traits de bout en bleu `[31, 119, 180]` ; EXACTEMENT 75 pixels rouges `[214, 39, 40]` (trois repères de 25) ;
tout pixel bleu dans une colonne `x(η) ± 2`. Formules : `x(η) = 40 + 70 η`, `y(D) = clamp(20 + round((2.2 - D) * 200.0), 0, 319)` ; à la main : `y(1,75) = 110`, `y(1,5) = 160`, `y(1,35) = 190`, `y(3,0) = 0`, `y(-0,5) = 319` ; **ajout du juge** : `y(1,0275) = 255` (`(2.2 - 1.0275) * 200.0` est un peu au-dessus de `234,5` ;
`440 - 200 × 1,0275` en double est un peu au-dessous et donnerait 254 ; vérifié par `essai-d2-juge-outils/y.rs`), un point `η = 4` sans barre d'erreur à cette valeur.
Source : la doc (convention) ; décodeur PNG indépendant de `tests/common/mod.rs` (étape 0). Tolérance : aucune.

**V-D2-16 : l'image d'une courbe réelle** (`image_d_une_courbe_reelle`) : se relit, 640 × 320, 75 pixels rouges, centres bleus. Forme seulement.

**V-D2-17 : le déterminisme** (`meme_graine_meme_courbe`) : mêmes graines, même CSV et même PNG, octet pour octet ; graines différentes, autre CSV. Source : la doc. Tolérance : aucune.

## L'ordre et la baisse (`etape_d2_ordre.rs`, étiquetés « régression »)

**V-D2-18 : `D(0) > D(1) > D(2) > D(3)`, chaque écart grand devant l'erreur-type** (`la_dimension_baisse_avec_eta`). 24 graines (1 à 24), 100 sites, demi-côtés `40, 40, 40, 48`. **Mesuré (prototype)**, dimension moyenne
et écart-type d'UNE graine σ sur les graines 1 à 120 : `η = 0` : `1,7591` (σ `0,0219`) ; `η = 1` : `1,6576` (`0,0583`) ; `η = 2` : `1,4890` (`0,0698`) ; `η = 3` : `1,3496` (`0,0709`) ; sur 50 graines neuves (1001 à 1050) :
`1,7594` (`0,0238`) ; `1,6732` (`0,0562`) ; `1,5094` (`0,0642`) ; `1,3468` (`0,0583`). Écarts `D(η) - D(η + 1)` par bloc : de `0,082` à `0,114`, de `0,156` à `0,189`, de `0,119` à `0,179` ; en unités de `sqrt(e_a² + e_b²)` : au moins `6,1` dans
les sept blocs. Bornes : écarts au moins `0,058`, `0,11`, `0,08` (`moyenne des écarts - 3 σ_écart / √24`, sur 170 graines) et au moins 4 fois l'erreur-type combinée. Source : aucune publication (l'ordre est celui de la théorie,
pas les valeurs). **Le juge a refait la mesure** (second code écrit d'après la doc seule, mêmes bits que le prototype) sur 100 graines neuves (7001 à 7050, 9001 à 9050), huit blocs de 24 :
`D` moyen `1,7601`, `1,6616`, `1,5056`, `1,3585` (σ d'une graine `0,0205`, `0,0524`, `0,0720`, `0,0677`) ; écarts des blocs d'au moins `0,086`, `0,129`, `0,109` (bornes `0,058`, `0,11`, `0,08` : tenues) ;
`z` d'au moins `6,7`, `6,5`, `4,2` (borne 4 : tenue, de peu pour `D(2) - D(3)`). Sur les graines 1 à 24 : `z` = `8,2`, `8,2`, `7,0`.

**V-D2-19 : les bandes de `D` de chaque `η`** (`la_dimension_de_chaque_eta_est_dans_sa_bande`) : `[1,746 ; 1,773]`, `[1,626 ; 1,698]`, `[1,452 ; 1,538]`, `[1,305 ; 1,392]` (moyenne sur 170 graines ± 3 σ / √24, arrondie vers l'extérieur).
Source : aucune publication ; **ces bandes ne sont pas des valeurs publiées**. Juge (huit blocs de 24 graines neuves) : tous dedans ; marges les plus faibles `+0,0064` (bas de `η = 0`) et
`+0,0017` (haut de `η = 2`, bloc 7027 à 7050 : `1,5363`). Les bandes tiennent ; elles ne sont pas élargies (les graines du test sont fixes : seule une faute peut les faire sortir).

**V-D2-20 : la courbe de 100 sites sur les graines 1 à 24, figée** (`courbe_de_100_sites_figee`) : le CSV `0,100,24,1.757889,0.003558,,` ; `1,100,24,1.657425,0.011674,1.710,-0.052575` ; `2,100,24,1.498010,0.015516,1.433,0.065010` ;
`3,100,24,1.358243,0.012358,1.263,0.095243`. Source : le prototype (`cargo run --example courbe_d2 -- <dossier> 100 24`). **Régression : ne prouve que la stabilité** ; la colonne `ecart` est l'arithmétique `D - publie` de notre sortie, pas une
comparaison.

## Les repères NON testés (lus, ou dits non lus)

**V-D2-21 : les repères publiés, NON TESTÉS.**
- **Hastings 2001, tableau II** : lu (V-D2-11). Ses autres colonnes (`1.4256`, `1.435`, `1.4522`, `1.264`, `1.262`, `1.243`) et lignes (`η >= 3,5`) ne sont pas dans le code ; **la transition à `η = 4` (D = 1) est un débat ouvert** :
  hors de D2, à ne pas tester.
- **La DLA à 1,71** (Mathiesen et coll.) et à `1,713 ± 0,003` (Davidovitch, Levermann, Procaccia, arXiv:cond-mat/0008053, un calcul sur de petits amas : lu à Z1, non relu ici) : repères, jamais comparés.
- **`D = 2` pour `η = 0`** (introduction de Hastings, lue) : `η = 0` n'a pas de repère dans la sortie (`publie` vide) ; à 100 sites un amas plein mesure `1,76` (effet de bord), à 300 sites `1,88`.
- **Phys. Rev. Lett. 87, 175502 (2001)** : la référence de la revue est celle que la page de l'arXiv (arxiv.org/abs/cond-mat/0103312, « Journal reference », lue le 09/11 ; v1 du 14 mars 2001, v2 du 1er juin 2001) donne ; elle n'est pas imprimée dans le PDF de l'arXiv v2, et **le numéro de la revue lui-même n'a pas été ouvert** : les valeurs lues sont celles de l'arXiv v2, pas de la version imprimée.
- **Niemeyer, Pietronero, Wiesmann (PRL 52, 1033, 1984)** : résumé seul (D1), texte intégral non lu ; **Eden (1961)** : non lu.

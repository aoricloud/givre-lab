# Étape D1 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape D1 (le claquage diélectrique : le champ de Laplace et la croissance en `φ^η` sur la grille carrée)
attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque identifiant (`V-D1-01`…) est cité dans la doc des tests qui l'emploient
(`tests/etape_d1_*.rs`). Elle est séparée des résultats figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient
du prototype de l'écrivain**. Les seules valeurs tirées du prototype sont les bandes statistiques de la forme des amas (V-D1-16 à V-D1-18, étiquetées
« régression ») et les mesures de coût et de sensibilité, dites mesurées. Une valeur vient d'une publication lue (dite lue, ou dite non lue), d'un calcul exact fait
par un outil indépendant (nommé), ou d'une convention que la doc de `src/claquage.rs` fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-d1-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `d1_ref.py`, écrit d'après la
**doc seule** de `src/claquage.rs`, sans lire le code Rust : PCG32 (transcription de `pcg_basic.c`, vecteur officiel retrouvé), le simulateur naïf du claquage
(dictionnaires, boucles simples : sur-relaxation rouge-noir, candidates, tirage, écriture canonique, SHA-256 par `hashlib`), la solution EXACTE du problème de
Laplace discret en rationnels (`fractions`, élimination de Gauss exacte) et la fabrication de graines à premier tirage `0,25`, `0,5`, `0,75`. `faire_valeurs.py`
écrit les fragments Rust collés dans les tests (aucune valeur recopiée à la main : une faute de transcription a pourtant été faite et vue par un test, voir le rapport).
Le prototype Rust et Python donnent les mêmes sites et les mêmes empreintes sur tous les cas listés ci-dessous.

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Les tests « exacts » (sites, candidates, bits des poids et des potentiels, écriture canonique, empreintes, CSV) n'ont **aucune** tolérance : égalité d'entiers,
   de chaînes ou de bits (`f64::to_bits`). La doc fixe chaque opération et leur ordre ; une implémentation fidèle donne les mêmes bits.
2. Un champ de potentiel est jugé par rapport à la solution EXACTE du problème de Laplace discret avec la tolérance que la doc DÉDUIT du seuil d'arrêt :
   résidu `<= 2 SEUIL` (V-D1-31) et écart à la solution exacte `<= 2 SEUIL (d + 1)²` (V-D1-29). Aucune tolérance n'est réglée.
3. Une propriété de forme vraie seulement en moyenne (compacité, ordre en `η`, voisinage de la DLA) se teste en moyenne sur 24 graines fixes (1 à 24), avec une
   bande `moyenne ± 3 σ / √24` (pour une différence de deux séries : `± 3 σ_diff / √24`) mesurée sur 120 graines (1 à 120, cinq blocs de 24) ; le bloc de
   24 graines des tests (les graines 1 à 24) et les quatre autres doivent y tomber. Aucune graine n'est choisie ; aucune bande n'est élargie (arrondie vers
   l'extérieur au chiffre significatif utile seulement).
4. **Aucune valeur publiée de `D(η)` n'est un test** : ni Hastings 2001 (`η = 2 → 1,433`, `η = 3 → 1,263`), ni la DLA à 1,713. Un test « proche de 1,433 » à
   100 sites se réglerait pour passer (le prototype est loin : voir `MODELE.md`). Ces repères sont dans V-D1-19, lus ou dits non lus.
5. Les tailles de test viennent du **coût mesuré** (`MODELE.md`, « Le coût ») : de 100 à 2000 sites. À ces tailles la dimension est bien en dessous de sa
   valeur limite ; les tests de forme ne prouvent que l'ordre et la compacité.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests `régression :` de l'étape D1 sont
`etape_d1_dimensions.rs::{eden_est_plein, eta_un_est_proche_de_la_dla_mais_plus_compact, plus_eta_monte_plus_l_amas_est_fin}` et
`etape_d1_scenario.rs::regression_references_claquage_graine_1`.

---

## Le simulateur, les bits (`etape_d1_exact.rs`)

**V-D1-01 : un amas égal au simulateur naïf** (`amas_egal_au_simulateur_naif`).
- Valeur : sites, rayon, arrêt, candidates et BITS de leurs poids, BITS du potentiel de chaque site de la boîte, état du générateur ; `η` = 0 à 3 (6 graines,
  `demi_cote` 14, à 1, 4, 10, 20, 30 sites), `η` = 4, 5, 8 (`demi_cote` 10, 12 sites), jusqu'à l'arrêt (`demi_cote` 5, 15 cas ; ajouté par le juge : `demi_cote` 2, 3, 4 avec les graines 21 à 60 et 6 avec
  les graines 21 à 40, `η` = 0, 1, 2, comparés à chaque pas : un mutant qui oubliait la ligne `y = d` des candidates passait).
- Source : le simulateur naïf de `tests/etape_d1_commun/mod.rs`, écrit d'après la section « La dynamique, à la lettre » de la doc (calcul exact par un code écrit à
  part, dans le test) ; sa fidélité à la doc est recoupée par Python (V-D1-10). Il n'emploie que `givre_lab::alea::Alea` (étape 0).
- Tolérance : aucune (bits). Critère : égalité sur tous les cas.

**V-D1-02 : le même, plus long** (`amas_egal_au_simulateur_naif_plus_long`) : `demi_cote` 24, `η` = 1, 2, 3, 3 graines, 80 sites. Même source, même critère.

**V-D1-03 : le germe seul** (`germe_seul_quatre_candidates_symetriques`).
- Valeur : candidates `(0,-1) (-1,0) (1,0) (0,1)` dans cet ordre ; poids `1.0` à `η = 0` ; égaux à `1e-13` près pour `η` = 1 à 3 (mesuré : `1,7e-16`) ; premier site
  = candidate d'indice `⌊4 t⌋`, `t` premier `suivant_f64` (400 graines).
- Source : la doc (règle d'énumération et règle du tirage) et le calcul à la main de `⌊4 t⌋` (à `η = 0` : `u = 4 t` exact, première somme `k + 1 > u`).
- Tolérance : exacte à `η = 0` ; `1e-13` pour l'égalité des poids (mesuré `1,7e-16` : l'ordre d'addition des quatre voisins) ; on écarte les `t` à moins de `1e-9` d'un
  quart (où l'arrondi peut décider) pour `η >= 1`. Mesuré : de 95 à 103 premiers sites par candidate sur 400 graines.

**V-D1-04 : les égalités du tirage** (`egalites_du_tirage`).
- Valeur : trois graines FABRIQUÉES `12457697844934659938`, `8335436391877801038`, `8760127956802293904` dont le premier `suivant_f64` vaut EXACTEMENT `0,25`, `0,5`,
  `0,75` ; à `η = 0`, `u = 1,0`, `2,0`, `3,0` et le premier site est la candidate d'indice 1, 2, 3 (règle `>` strict).
- Source : construction (inversion de l'amorçage de PCG32 : `d1_ref.py fabriquer`, 8 s ; le test vérifie `suivant_f64() == t`) ; empreintes de 12 sites calculées par Python.
- Tolérance : aucune. Critère : un `>=` donnerait les indices 0, 1, 2.

**V-D1-05 : `η = 0` sans champ, indépendant du domaine** (`eta_zero_sans_champ_et_independant_du_domaine`) : `potentiel` rend `None` ; 6 graines, 150 sites, `demi_cote` 30
et 45 donnent les mêmes sites (rayon des amas d'Eden de 150 sites : de l'ordre de 10) ; à `η = 1` les amas diffèrent (mesuré : 6 sur 6, dès le 12e site au plus tôt).
Source : la doc. Tolérance : aucune.

**V-D1-06 : l'arrêt** (`arret_a_la_frontiere`) : `demi_cote` 0 : le germe touche ; `demi_cote` 1 : exactement 2 sites ; `demi_cote` 2 à 4 : le dernier site touche, aucun
avant ; un `None` ne consomme aucun tirage. Source : la doc (règle « touche la frontière » : un voisin `(x + dx, y + dy)` avec `> d²`, comparaison entière) et le calcul
à la main du cas `d = 1` (les quatre voisins du germe sont à distance 1 ; leurs voisins en `(2, 0)`… sont hors du domaine). Tolérance : aucune.

**V-D1-07 : les bornes** (`bornes_et_constantes`) : `ETA_MAX = 8`, `DEMI_COTE_MAX = 255`, `PARTICULES_MAX = 100000`, `PASSES_MAX = 100000` (la garde de terminaison : mesuré, 354 passes à froid à `d = 60`, 1316 à `d = 255`), `SEUIL = 2^-20` (bits), `surrelaxation(d) = 2 - 6/(d + 6)` :
exactement `1,0`, `1,5`, `1,75`, `1,875`, `1,9375` pour `d` = 0, 6, 18, 42, 90 (calcul à la main : `6/6 = 1`, `6/12 = 0,5`, `6/24 = 0,25`, `6/48 = 0,125`, `6/96 = 0,0625`, tous exacts en `f64`).
Source : la doc (convention). Tolérance : aucune.

**V-D1-08 : la croissance par morceaux** (`croissance_par_morceaux_et_proprietes`) : `croitre_jusqu_a(10)` puis `(25)` égale `(25)` (sites, écriture, bits du potentiel). Source : la doc.

**V-D1-09 : l'écriture canonique** (`ecriture_canonique_a_la_main`) : 44 octets pour le germe seul (`demi_cote` 5, `η` 1 puis 0, graine 7 :
`d1a0bdb386a42205 6d00000000000000` est l'état de `Alea::new(7)`, valeur de l'étape 0) et 68 octets pour quatre sites (graine 1, `demi_cote` 9, `η` 1 : état final
`3dce35a13968f346`). Empreintes `7720013b…a82b`, `30576b4a…727b`, `8d350a34…55e6` (SHA-256, Python `hashlib` sur les octets que `d1_ref.py` écrit). Tolérance : aucune.

**V-D1-10 : les valeurs du code Python indépendant** (`valeurs_du_code_independant`) : neuf (graine, `d`, `η`, `n`) pour `η` = 0, 1, 2, 3, 4, 8 : douze premiers sites, rayon,
empreinte (voir le test). Source : `d1_ref.py premiers` (via `faire_valeurs.py`). Tolérance : aucune.

## Le champ (`etape_d1_potentiel.rs`)

**V-D1-11 : le germe seul est la solution exacte** (`germe_seul_solutions_exactes`). Valeurs : les FRACTIONS exactes de Laplace discret pour `d` = 1 à 6 (`3/4` à `d = 1` ;
`7/11`, `9/11`, `10/11` à `d = 2` ; `93/167`, `237/334`, `135/167`, `144/167`, `311/334`, `159/167` à `d = 3` ; au-delà : voir le test, `faire_valeurs.py`).
Source : `d1_ref.py exact` (rationnels, élimination de Gauss exacte) ; `d = 1` et `d = 2` aussi à la main (`4a = c + 2b`, `4b = 2 + 2a`, `4c = 3 + a`). Tolérance : `2 SEUIL (d + 1)²`
(V-D1-29) ; mesuré : l'écart réel est de `4e-8` (`d = 1`) à `1,1e-6` (`d = 6`), soit 80 à 170 fois sous la tolérance.

**V-D1-12 : le champ contre une solution directe** (`champ_contre_la_solution_directe`) : élimination de Gauss dense en `f64` écrite dans le test (aucune itération), `η` = 1, 2, 3,
`demi_cote` 8 et 10, 3, 8 et 15 sites. Tolérance `2 SEUIL (d + 1)²` ; mesuré : l'écart maximal est `2,9e-6` contre `1,5e-4` (`d = 8`) et `2,3e-4` (`d = 10`).

**V-D1-13 : les propriétés du champ** (`proprietes_du_champ_a_chaque_pas`) : `φ = 0` sur l'amas, `φ = 1` hors du domaine, `0 <= φ <= 1`, résidu `<= 2 SEUIL` à chaque pas. Mesuré :
le résidu atteint `0,60 SEUIL` au plus.

**V-D1-14 : la symétrie et la croissance du champ du germe seul** (`germe_seul_symetrique_et_croissant`) : huit images à `1e-9` (mesuré : `1,3e-15`) ; `φ(k, 0)` croît de plus de `1e-3` à chaque pas
(mesuré : la plus petite différence est `0,010`).

**V-D1-29 : la borne de la doc sur l'écart à la solution exacte** (convention déduite, pas mesurée). Soit `r_i = φ_i - moyenne(voisins)` le résidu et `e = φ - φ*` l'écart à la
solution exacte `φ*` de `4 φ_i - Σ voisins = (nombre de voisins hors domaine)`. Alors `L e = 4 r` (`L = 4 I - A`). La fonction `q = (K - x² - y²) / 4` vérifie `L q = 1` et `q >= 0` sur les
sites du domaine et sur la marge si `K >= (d + 1)²` ; par le principe de comparaison, `|e| <= 4 max|r| q <= max|r| (d + 1)²`. Avec `max|r| <= 2 SEUIL` : `|e| <= 2 SEUIL (d + 1)²`. Source :
principe du maximum discret (démonstration ci-dessus, vérifiée numériquement : V-D1-11, V-D1-12).

**V-D1-30 : `u < W`** (la règle du tirage n'a pas de cas du dernier site). `t <= 1 - 2^-53` donc `t W <= W (1 - 2^-53)`, qui est à au moins un demi-pas de `W` ; l'arrondi
au plus proche donne `u <= W - ulp(W)/2 < W` (à `W` puissance de 2 le produit est exact). Source : calcul (IEEE 754 binaire64, arrondi au plus proche). La cumulative finale vaut `W` exactement
(mêmes additions dans le même ordre), donc une candidate dépasse toujours `u`.

**V-D1-31 : le résidu après la dernière passe** (convention déduite). Dernière passe : changement `< SEUIL` pour tout site libre. Un site de couleur 1 (mis à jour en dernier) a pour
résidu `|ω - 1| / ω × (changement) < SEUIL` ; un site de couleur 0 a vu ses voisins changer chacun de moins de `SEUIL`, donc sa moyenne de moins de `SEUIL` : résidu
`< SEUIL (|ω - 1| / ω + 1) < 2 SEUIL`. Un site que le `clamp` a retenu a un résidu inférieur à `SEUIL`. Mesuré : `0,60 SEUIL` au plus.

**V-D1-32 : le total `W` sommé dans l'ordre canonique** (`somme_des_poids_dans_l_ordre_canonique` ; ajouté par le juge). Valeur : les deux premiers sites de douze
germes seuls (`demi_cote` 2 à 12, `η` 1 à 3) avec les trois graines fabriquées de V-D1-04 (premier tirage exactement `0,25`, `0,5`, `0,75`). Source : `d1_ref.py premiers <graine> <d> <eta> 3`
(Python indépendant, rejoué par le juge) et le simulateur naïf ; le juge a vérifié (outil `juge_m44.rs`) que dans ces douze états la somme à l'envers diffère de `W` d'un ulp et
choisit un autre site. Tolérance : aucune. Critère : la faute « `W` sommé à l'envers » (M44 de l'écrivain, J56 du juge) échoue ici.

## La directive et `lancer` (`etape_d1_scenario.rs` ; section ajoutée par le juge : l'écrivain citait V-D1-20 à V-D1-28 dans les tests sans les écrire ici)

**V-D1-20 : `parametres`** (`parametres_lit_la_directive`). Valeurs : `Ok(None)`, `Ok(Some((η, n)))` et les lignes refusées du test. Source : la doc de `claquage::parametres` (convention,
calquée sur `dla::particules`) et `Directive::entier` (étape 0 : `99999999999999999999` dépasse `u64::MAX`). Tolérance : aucune.

**V-D1-21 : trois petits scénarios à la main** (`lancer_a_la_main`). Valeurs : les cinq empreintes et CSV du test (`8511b9cf…f88b`, `20c44f18…8673`, `2c81f3df…23fc`, `60ce2ca7…0070`,
`3dd17156…05cb6`). Source : `d1_ref.py scenario <graine> <largeur> <hauteur> <eta> <n>` (Python indépendant ; rejoué par le juge le 09/11 : mêmes empreintes et CSV) ; l'aperçu est
`dla::amas_colore` (étape 1) du même amas. Tolérance : aucune (chaînes et octets).

**V-D1-22 : les trois scénarios du dépôt** (`exemples_du_depot`) : `boite 61 61` + `claquage 0 600`, `boite 61 61` + `claquage 1 150`, `boite 81 81` + `claquage 2 120`, graine 1 :
`939571b6…d903`, `6c912e3e…8c11`, `f4f572d5…bd68` et les CSV du test. Source : `d1_ref.py scenario` (rejoué par le juge). Les fichiers `scenarios/claquage-{0,1,2}.txt` sont à
écrire par la vague du code avec exactement ces deux lignes (et des commentaires). Tolérance : aucune.

**V-D1-23 : la forme du CSV** (`mesures_empreinte_et_apercu_suivent_la_doc`) : reconstruite dans le test depuis les sites (rayon entier, giration par sommes entières exactes, six
décimales, puissances de 2 puis `n`). Source : la doc de la directive (convention, celle de `dla`). Tolérance : aucune.

**V-D1-24 : déterminisme et graines** (`determinisme_et_graines`) : des égalités et des inégalités d'empreintes, sans valeur. Source : la doc. Tolérance : aucune.

**V-D1-25 : les refus** (`refus_et_arret`) : les lignes du test. Source : la doc (« Les refus », la plus petite ligne refusée ; la règle existante « la DLA l'emporte » de
`lancer`, étape 2d). Tolérance : aucune.

**V-D1-26 et V-D1-27 : le binaire** (`binaire_claquage`, `binaire_refus_et_acceptation`) : code de sortie, trois fichiers, message `ligne 2` sans `panicked`. Source : le contrat du
binaire de l'étape 0 (inchangé) et l'empreinte de V-D1-22. Tolérance : aucune.

**V-D1-28 : les références d'empreinte** (`regression_references_claquage_graine_1`) : `references/claquage-{0,1,2}-graine-1.empreinte`, figées par la vague du code après V-D1-22.
Régression : ne prouve que la stabilité.

## La forme des amas (`etape_d1_dimensions.rs`, étiquetés « régression »)

**V-D1-16 : l'amas d'Eden (`η = 0`) est plein** (`eden_est_plein`). 24 graines, 2000 sites, `demi_cote` 50. Valeurs mesurées sur 120 graines : dimension de masse-rayon
(rayons 6 à `rayon / 2`, depuis le germe) moins celle du disque plein de mêmes rayons (`sites_du_disque`) : moyenne `-0,0020`, `σ = 0,0023`, de `-0,0148` à `+0,0015` ; blocs de 24 :
`-0,0013` à `-0,0025`. Fraction du disque de rayon 10 occupée : moyenne `0,9999`, au moins `0,9968` ; rayon 16 : `0,9983`, au moins `0,986`. Tolérances : par graine `[-0,02 ; 0,005]`,
`>= 0,99`, `>= 0,975` ; moyenne `[-0,004 ; 0,0005]` (`-0,0020 ± 3 σ/√24`, arrondie vers l'extérieur). Source du critère « compact, dimension 2 » : Hastings, Phys. Rev. Lett. 87, 175502
(2001), introduction (lue, voir V-D1-19) pour « D = 2 à `η = 0` (modèle d'Eden) » ; la compacité de l'amas d'Eden en tant que telle (rayon et rugosité) : **À VÉRIFIER** (aucune
source lue en texte intégral : Wikipédia « Eden growth model » ne le dit pas ; la valeur mesurée par le prototype, `D = 2,008`, n'est qu'une régression). Le juge : la rugosité
de la frontière d'un amas d'Eden est dans la classe KPZ à (1 + 1) dimensions (K. A. Takeuchi, J. Stat. Mech. P05007, 2012, arXiv:1203.2483 : résumé lu, pour un Eden hors réseau ;
les exposants n'y sont pas donnés) ; Jullien et Botet, J. Phys. A 18, 2279 (1985) : NON LU. Mesuré par le juge sur 72 graines neuves (5001 à 5072) : écart au disque `-0,0029`
(σ `0,0034`, de `-0,0163` à `0,0000`), blocs de 24 `-0,0029`, `-0,0027`, `-0,0031` ; disque 10 : au moins `0,9969` ; disque 16 : au moins `0,9875` : dans les tolérances.

**V-D1-17 : `η = 1` proche de la DLA mais plus compact** (`eta_un_est_proche_de_la_dla_mais_plus_compact`). 24 graines, 300 sites, `demi_cote` 45, rayons de corrélation 2 à 8. Mesuré sur 120
graines : `η = 1` corrélation `1,6767` (σ `0,041`), masse-rayon `1,614` (σ `0,168`), `1/β` `1,800` (σ `0,158`), rayon `23,5` ; DLA de l'étape 1 : `1,6027` (`0,043`), `1,580` (`0,150`),
`1,687` (`0,128`), `27,6`. Écarts `η = 1 - DLA` : corrélation `+0,074` (blocs de 24 : `+0,056` à `+0,095`), masse-rayon `+0,034`, `1/β` `+0,11`, rayon `-4,1`. Bandes : voir le test
(chacune `écart ± 3 σ_diff / √24`). Source : aucune publication (contrôle croisé de notre propre code) ; le sens de l'écart est confirmé à 600 et 1000 sites (voir `MODELE.md`).
Le juge, sur 100 graines neuves (5001 à 5100) : corrélation du claquage `1,6797` (σ `0,044`, de `1,527` à `1,776`), DLA `1,6085` ; écarts `+0,071`, `+0,027`, `+0,066`, `-3,9` : dans les
bandes. La bande PAR GRAINE `[1,55 ; 1,80]` de l'écrivain est ÔTÉE (une graine neuve donne `1,527` : ce n'est pas une propriété du modèle). La cause de l'écart est mesurée
(`MODELE.md`) : avec l'amas ET ses sites périmètre absorbants (ce que fait le marcheur de l'étape 1), la croissance redonne la DLA (`1,6129` contre `1,6085` à 300 sites).

**V-D1-18 : plus `η` monte, plus l'amas est fin** (`plus_eta_monte_plus_l_amas_est_fin`). 24 graines, 100 sites, `demi_cote` 40, rayons de corrélation 2 à 6. Mesuré sur 120 graines : corrélation
`1,6060`, `1,4548`, `1,3370` pour `η` = 1, 2, 3 ; `1/β` `1,767`, `1,503`, `1,314` ; rayon `13,1`, `17,1`, `23,0`. Écarts de moyennes et bandes : voir le test. Source : aucune publication (l'ordre
est celui de la théorie, pas les valeurs : D2).

**V-D1-19 : les repères publiés, NON TESTÉS.**
Relus par le juge le 09/11 (WebFetch) : Hastings (ar5iv, tableau II et introduction : conformes à ce qui suit, sans barres d'erreur), le résumé de Niemeyer et coll. (journals.aps.org :
ni dimension, ni `η`, ni méthode), Mathiesen et coll. (ar5iv : « D = 1,71 pour η = 1 », aucune valeur de `η = 2` ou `3` dans le texte ni les légendes ; seul `D = 1,10` pour `η = 4`, légende de la figure 3).
Un `grep` du juge sur `tests/etape_d1_*.rs` : aucune de ces valeurs n'est dans un test.
- **Niemeyer, Pietronero, Wiesmann, Phys. Rev. Lett. 52, 1033 (1984), « Fractal Dimension of Dielectric Breakdown »** (https://doi.org/10.1103/PhysRevLett.52.1033) : le **résumé** de l'éditeur
  seulement est lu (journals.aps.org/prl/abstract/10.1103/PhysRevLett.52.1033) : « le modèle stochastique non trivial le plus simple de claquage diélectrique conduit naturellement à des structures fractales ;
  des décharges planes sont étudiées en détail et comparées à des expériences » ; **aucune valeur de dimension ni d'`η` n'y figure, aucune méthode de grille ou de bord**. **Texte intégral NON LU**
  (derrière un accès payant). Ce que le plan écrit de ce papier (`p ∝ φ^η`) est donc repris de l'énoncé du brief, non relu ici.
- **Hastings, Phys. Rev. Lett. 87, 175502 (2001), arXiv:cond-mat/0103312** (lu par ar5iv : https://ar5iv.labs.arxiv.org/html/cond-mat/0103312) : **tableau II**, « pour diverses valeurs de `η`, dimension calculée
  d'après différentes parties du jeu de données » : `η = 2` : `1,433` (jeu entier), `1,4256`, `1,435`, `1,4522` (dernière moitié, quart, huitième), rms `0,039` ; `η = 3` : `1,263`, `1,264`, `1,262`, `1,243`, rms `0,056` ;
  `η = 3,5` : `1,170` ; `η = 4` : `1,128` ; `η = 4,5` : `1,101` ; `η = 5` : `1,068`. **Pas de barres d'erreur** (la dernière colonne est la fluctuation rms de `F_1` à la fin). Méthode : **applications conformes itérées, pas un
  réseau** (« sans effets de grille ni de bord »), `D` par régression de `F_1 ∝ n^(1/D)`. L'introduction (lue) : la dimension du modèle va « de 2 (`η = 0`, le modèle d'Eden) à environ 1,7 (`η = 1`, DLA) à 1 ». **La
  valeur publiée n'est pas atteinte par le prototype** à ces tailles et sur ce réseau (à 100 sites : corrélation `1,455` pour `η = 2` et `1,337` pour `η = 3`, sur des rayons 2 à 6) : le plan dit que D2 compare, D1 ne le fait pas.
- **Mathiesen, Jensen, Bakke, « Dimensions, maximal growth sites and optimization in the dielectric breakdown model »** (arXiv:0712.1739 ; DOI de la version publiée 10.1103/PhysRevE.77.066203 selon la recherche, non
  relu) : lu par ar5iv : le texte donne « la valeur de la DLA `D = 1,71` pour `η = 1` » (section IV) et trace `D(η)` sur `[1, 5]` en figure 4 sans en donner les nombres dans le texte ni dans la légende ; `η = 2` et `η = 3` :
  NON LUS (lisibles seulement sur la figure). L'année 2007 du brief : **non confirmée** (l'identifiant arXiv `0712` est de décembre 2007 ; l'année de la revue n'a pas été lue).
- **Davidovitch, Levermann, Procaccia, arXiv:cond-mat/0008053 (PRE 62, R5919)** : `1,713 ± 0,003` est un résultat de CALCUL de renormalisation sur de petits amas, pas une mesure sur un grand amas (lu à l'étape Z1 ; non relu ici).
- **Amas d'Eden compact (dimension 2)** : lu seulement dans l'introduction de Hastings 2001 (ci-dessus). L'article original d'Eden (1961, « A two-dimensional growth process », Proc. Fourth Berkeley Symposium, vol. 4,
  pp. 223-239) : **NON LU**. La rugosité de la frontière d'un amas d'Eden : **À VÉRIFIER**.

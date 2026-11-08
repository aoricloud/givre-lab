# Étape Z1 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape Z1 (la corrélation densité-densité `C(r)` sur la DLA) attend, d'où elle vient, avec quelle
tolérance et quel critère de réussite. Chaque identifiant (`V-Z1-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_z1_*.rs`). Elle est
séparée des résultats figés de notre propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du prototype de
l'écrivain**. Les seules valeurs tirées du prototype sont les bandes statistiques de la DLA (V-Z1-18 à V-Z1-21, étiquetées « régression ») et les
mesures de sensibilité de la fenêtre, dites mesurées. Une valeur vient d'une publication lue (dite lue, ou dite non lue), d'un calcul exact fait par un
outil indépendant (nommé), ou d'une convention que la doc d'un module fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-z1-outils/` des notes de la session ; le code Rust n'est jamais l'outil) : `z1_ref.py` (un compteur
NAÏF des paires ordonnées par distance au carré, double boucle, écrit d'après la doc seule de `src/correlation.rs` sans lire le code Rust ; un second
compteur par déplacements et intersection d'ensembles ; les rationnels `fractions` ; les logarithmes `decimal` à 60 chiffres ; les formes) ; ses sorties
brutes `z1_ref_petits.txt`, `z1_ref_formes.txt`. Le prototype Rust et Python donnent les mêmes dimensions de forme à `1e-9` (huit formes) et le même tableau de
paires sur les six petits amas à la main.

**Critères écrits avant les calculs** (ce qu'on accepte comme source et comme tolérance) :

1. Les tests « exacts » (entiers, fractions, formules fermées) n'ont **aucune** tolérance : égalité d'entiers, ou égalité de bits quand le résultat est un
   seul quotient de deux entiers exacts (l'arrondi correct de la fraction).
2. Une dimension qui vaut « exactement » une valeur calculée à part (un petit amas, une forme) a la tolérance `1e-12` (cas à la main : l'erreur du
   `logarithme` est `5e-16` en relatif) ou `1e-9` (formes à plusieurs centaines de rayons, deux bibliothèques de logarithme différentes).
3. La fidélité à la dimension d'une forme (1, 2, `ln 5 / ln 3`, `ln 8 / ln 3`) est une tolérance **mesurée** sur le prototype : le pire écart observé sur les
   formes de la liste, plus un quart (V-Z1-14). Elle ne se règle pas sur la DLA.
4. Une propriété de la DLA vraie seulement en moyenne se teste en moyenne, sur 24 graines fixes, avec une bande `moyenne ± 3 σ / √24` mesurée sur 120
   graines ; les moyennes des blocs de 24 graines doivent y tomber toutes (V-Z1-18 à V-Z1-21). Aucune graine n'est choisie.
5. **La valeur publiée 1,713 ± 0,003 (Davidovitch, Levermann, Procaccia 2000) et le zinc 1,66 ± 0,03 (Matsushita et coll. 1984) ne sont pas des tests**
   (V-Z1-35 et V-Z1-36 : repères). Un test « dans la barre du zinc » se réglerait pour passer. Une taille d'amas de zinc qu'on n'a pas lue n'est pas inventée
   (V-Z1-37).
6. La fenêtre d'ajustement est fixée par une formule de l'amas avant toute mesure de la DLA (V-Z1-11) : celle que le plan suggérait (de 4 à environ le
   quart du rayon de l'amas, ici `étendue / 8`) ; la sensibilité à ce choix est mesurée sur les formes et sur la DLA, rapportée (`MODELE.md`, étape Z1), et
   n'a servi à rien d'autre. **Ce n'est pas la meilleure fenêtre sur les formes** : parmi onze fenêtres essayées, `2 à étendue / 8` a un écart moyen de 0,031
   aux formes contre 0,044 pour `4 à étendue / 8` (qui est dans un palier de 0,03 à 0,045 avec les fenêtres dont le haut va de `étendue / 10` à `étendue / 6`
   et le bas de 2 à 8) ; le bas de la fenêtre à 4 (et non 2) est un choix du plan, motivé par l'échelle de réseau des petits `r`, non mesuré.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les tests `régression :` de l'étape Z1 sont
`etape_z1_dla.rs::{trois_mesures_a_4000_sites, trois_mesures_a_10000_sites, la_courbe_decroit}` et
`etape_z1_vitrine.rs::le_plafond_de_la_mesure`.

---

## Le disque plein

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-01 | `sites_du_disque(r)` pour `r = 0..=12` : `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441` ; `r = 100` : 31 417 ; 370 : 430 085 ; 400 : 502 625 ; 1000 : 3 141 549 ; 4096 : 52 706 921 | OEIS A000328, « Number of points of norm <= n^2 in square lattice » (**lue** le 8/11/2026 : termes 0 à 19 `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441, 529, 613, 709, 797, 901, 1009, 1129`, le bord compris) ; recomptée case par case par Python pour `r <= 1000` (`disque_naif`), par colonnes avec `isqrt` au-delà (`disque`) | — | égalité d'entiers ; le compteur naïf du test (double boucle) égale la fonction pour tout `r` de 0 à 60 |
| V-Z1-02 | `r = 36 974` : 4 294 797 601 ; `r = 36 975` : 4 295 030 009 (le premier rayon dont le disque dépasse `u32::MAX = 4 294 967 295`) ; 65 535 : 13 492 624 829 ; 65 536 : 13 493 036 169 ; 100 000 : 31 415 925 457 ; `2^20` : 3 454 217 649 829 | Python, comptage par colonnes (`z1_ref.py disque`), le premier rayon trouvé par dichotomie | — | égalité d'entiers ; `r > 2^20` : panique (convention de la doc : le coût est d'ordre `r`) |

## Les paires par distance au carré

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-03 | deux sites `(0,0)`, `(3,4)` (`rayon_max 5`) : `{25: 2}` ; trois alignés `(0,0)`, `(1,0)`, `(3,0)` (3) : `{1: 2, 4: 2, 9: 2}` ; le L `(0,0)`, `(1,0)`, `(0,1)` (2) : `{1: 4, 2: 2}` ; trois fois `(5,5)` et `(6,5)` (1) : `{0: 6, 1: 6}` ; carré 2 × 2 (2) : `{1: 8, 2: 4}` ; carré 3 × 3 (3) : `{1: 24, 2: 16, 4: 12, 5: 16, 8: 4}` | compteur naïf de Python (`paires_naives`, double boucle sur les paires ordonnées), recompté à la main ; la somme du carré 3 × 3 est `9 × 8 = 72` | — | égalité de tableaux d'entiers |
| V-Z1-04 | pour 392 combinaisons (taille 0, 1, 2, 5, 40, 150, 300 × demi-côté 1, 4, 12, 40 × `rayon_max` 0, 1, 2, 3, 7, 12, 30 × deux graines) : le tableau du compteur naïf | simulateur naïf écrit dans le test (`etape_z1_commun::paires_naives` : double boucle sur `i != j`, `i128`), ancré sur les tableaux de V-Z1-03 | — | égalité de tableaux d'entiers |
| V-Z1-05 | cas limites : vide et un site → zéros de `r² + 1` éléments ; `rayon_max 0` → les répétitions ; somme `n (n - 1)` quand le rayon contient tout ; `i32::MIN` et `i32::MAX` sans débordement, seule la paire proche compte ; `RAYON_MAX = 4096` accepté (`4096² + 1` éléments), 4097 refusé | doc de `paires_par_distance2` (« Panique » et corps) ; calcul à la main | — | égalité / panique |

## `C(r)`

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-06 | un site `0, 0, 0` ; deux sites `(0,0)`, `(3,4)` : `0, 0, 0, 0, 1/80` ; trois alignés : `1/6, 1/9, 1/14` ; le L : `1/3, 1/6` ; trois fois `(5,5)` et `(6,5)` : `3/8` ; carré 2 × 2 : `1/2, 1/4` ; carré 3 × 3 : `2/3, 13/27, 2/7` | `fractions.Fraction(cumul, n × (disque - 1))` de Python (`z1_ref.py petits`) | — (une division de deux entiers exacts : l'arrondi correct de la fraction, égalité de bits avec `p / q` écrit en `f64`) | égalité de bits |
| V-Z1-07 | carré plein de côté `s` (`s = 1..=12` et 20) : `cumul(r) = Σ (s - |dx|)(s - |dy|)` sur les déplacements non nuls de longueur `<= r` ; trait de `L` sites (`L = 2, 3, 10, 101`) : `Σ_{dx=1..r} 2 (L - dx)` | formules fermées de la doc de `correlation` (le déplacement `(dx, dy)` apparaît `(s - |dx|)(s - |dy|)` fois dans un carré plein : décompte direct), calculées par des boucles dans le test, le disque compté case par case | — | égalité de bits, rayons de 1 à `2 s + 1` (au-delà du carré le cumul est constant) |
| V-Z1-08 | carré 100 × 100 (10 000 sites), `C(r)` pour `r = 141..=400` vaut `99 990 000 / (10 000 × (disque(r) - 1))` ; `C(370) = 9999/430084` (dénominateur 4 300 840 000), `C(400) = 9999/502624` (5 026 240 000) | cumul constant `n (n - 1)` (la diagonale est `99 √2 = 140,007`) ; Python (`z1_ref.py petits`) ; le dénominateur dépasse `2^32 = 4 294 967 296` dès `r = 370` | — | égalité de bits |

## La dimension, la fenêtre, la mesure

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-09 | carré 2 × 2, rayons `[1, 2]` : exactement 1 ; carré 3 × 3, `[1, 2]` : `1,5305147166987798` ; `[1, 2, 3]` : `1,2612502168406583` ; carré 5 × 5, `[1, 2, 3, 4]` : `1,5560783714914212` ; trait de 101 sites, `[2, 4, 8, 16]` : `0,9469942034868164` | Python, `fractions` pour les `C(r)` exacts et `decimal` (60 chiffres) pour `ln` et les moindres carrés (`z1_ref.py petits`) | `1e-12` | `|obtenu - attendu| <= 1e-12` ; même valeur avec les rayons dans le désordre |
| V-Z1-10 | panique : moins de deux rayons, rayons tous égaux, rayon nul, rayon `> RAYON_MAX`, aucun site, `C(r) = 0` ; accepté : `[1, 2]` d'un carré 2 × 2, `[2, 2, 3]` | doc de `dimension_correlation` (« Panique ») | — | `catch_unwind` / résultat fini |
| V-Z1-11 | `etendue` : vide et un site 0 ; `(0,0)`, `(3,10)` : 10 ; `i32::MIN` et `i32::MAX` : `4 294 967 295`. `fenetre` : 4 à `étendue / 8` si au moins 6 rayons : étendue 71 vide, 72 à 79 `4..=9`, 80 `4..=10`, 242 `4..=30`, 32 768 `4..=4096`, 32 776 vide | **convention** de la doc de `correlation` (« La fenêtre d'ajustement »), choisie avant toute mesure de la DLA d'après les formes (voir la règle 6) ; ces tailles sont des calculs entiers | — | égalité de listes |
| V-Z1-12 | trait de 201 sites : fenêtre `4..=25`, dimension `0,9596965250374998` ; `mesure` rend `None` pour l'étendue 71, pour deux sites à distance 100, un seul site, vide, et ne panique jamais | Python (`d_de_la_forme(trait(201))`) ; doc de `mesure` | `1e-12` | `|obtenu - attendu| <= 1e-12` ; valeurs et dimension égales de bits aux fonctions de `correlation` |

## Les formes

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-13 | dimension de la fenêtre de la doc : trait de 243 sites `0,9617828589570572` ; trait de 729 `0,9718711000676697` ; diagonale de 243 `1,0687334819006453` ; carré 81 × 81 `1,9283629575414903` ; Vicsek en croix niveau 5 `1,4436788606250689` ; en X niveau 5 `1,5088551993791248` ; tapis niveau 4 `1,829664145714223`, niveau 5 `1,8408485578358925` | Python (`z1_ref.py formes` : le compteur par déplacements, `fractions`, `decimal` 60 chiffres), d'après la doc seule ; les formes sont fabriquées dans le test (même construction que `etape_1_mesures`) | `1e-9` | `|obtenu - attendu| <= 1e-9` |
| V-Z1-14 | écart à la dimension de la forme (1, 2, `ln 5 / ln 3 = 1,46497352071792717…`, `ln 8 / ln 3 = 1,89278926071437231…`) | dimensions de similarité de la construction « 5 carrés sur 9 » et « 8 carrés sur 9 » (Wikipedia, « Vicsek fractal » et « Sierpiński carpet », déjà citées par V-BOITE-04, **non relues** ici) ; écarts mesurés sur le prototype (fenêtre de la doc) : trait 243 : -0,038, 729 : -0,028, diagonale +0,069, carré 81 -0,072, Vicsek croix niveau 5 -0,021 (niveau 6 : -0,014), X +0,044, tapis niveau 4 -0,063, niveau 5 -0,052 (niveau 6 : -0,043) | `0,09` (le pire écart observé, 0,072, plus un quart) | `|obtenu - dimension de la forme| <= 0,09` |
| V-Z1-15 | `mesure(sites).dimension` est, au bit près, `dimension_correlation(sites, &fenetre(sites))` | identité de la doc de `mesure` | — | égalité de bits |
| V-Z1-16 | invariance EXACTE des tableaux de paires, de `C(r)`, de l'étendue, de la fenêtre et de la mesure par translation (`(137, -2048)`, `(±2^30)`), quart/trois quarts de tour, demi-tour, miroir, échange des axes, ordre inversé et mélangé | propriété de la définition (distances entre sites et comptes entiers) ; transformations écrites dans le test | — | égalité de tableaux et de bits |
| V-Z1-17 | fenêtres du Vicsek de niveaux 4, 5, 6 : `4..=10`, `4..=30`, `4..=91` (étendues 80, 242, 728) ; trait de 243 sites, Vicsek et tapis de niveau 5 : la même fenêtre ; un amas allégé de son tiers intérieur : la même | calcul entier de V-Z1-11 | — | égalité de listes |

## La DLA du banc (régression : bandes mesurées sur NOTRE modèle)

Mesuré sur le prototype, 120 graines (1 à 120), `Amas::new(graine, 400)` ; moyenne et écart-type d'UNE graine (`z1_mesures dla`, sorties
`dla-4000-g1-120.txt`, `dla-10000-g31-120.txt`, `dla-10000-g1-30-detail.txt`) :

| taille | corrélation | masse-rayon (6 à R/2) | 1/β | corr − mr | corr − 1/β | mr − 1/β |
|---|---|---|---|---|---|---|
| 4000 sites | 1,6486 (0,0215) | 1,6737 (0,047) | 1,7101 (0,060) | −0,0251 (0,046) | −0,0615 (0,061) | −0,0364 (0,044) |
| 10 000 sites | 1,6587 (0,0152) | 1,6803 (0,035) | 1,7127 (0,054) | −0,0216 (0,037) | −0,0540 (0,058) | −0,0324 (0,044) |

Les 250 graines de l'étape 1 donnaient 1,670 (σ 0,047) pour la masse-rayon à 4000 sites : le prototype de cette étape retrouve 1,6737 (σ 0,047) sur
120 graines, ce qui recoupe le banc de l'étape 1.

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-18 | 4000 sites, graines 1 à 24 : corrélation moyenne dans `[1,635 ; 1,662]` | `1,6486 ± 3 × 0,0215 / √24` ; les cinq moyennes de blocs de 24 graines (1,6427 ; 1,6479 ; 1,6516 ; 1,6537 ; 1,6472) y tombent | bande mesurée | `1,635 <= moyenne <= 1,662` |
| V-Z1-19 | 4000 sites : `|corr - mr| <= 0,055`, `|corr - 1/β| <= 0,10`, `|mr - 1/β| <= 0,065` ; chaque graine : trois dimensions dans `[1,45 ; 1,95]` | `|moyenne| + 3 σ/√24` (0,0535 ; 0,0989 ; 0,0635) ; les blocs de 24 donnent au plus 0,042 ; 0,087 ; 0,045 ; sur 240 amas les dimensions vont de 1,568 à 1,909 | bandes mesurées | écarts de moyennes sous les trois bornes |
| V-Z1-20 | 10 000 sites, graines 1 à 24 : corrélation moyenne dans `[1,649 ; 1,668]` ; mêmes trois écarts | `1,6587 ± 3 × 0,0152 / √24` ; écarts `|moyenne| + 3 σ/√24` : 0,044 ; 0,090 ; 0,059 ; blocs de 24 : au plus 0,035 ; 0,072 ; 0,037 | bandes mesurées | idem |
| V-Z1-21 | 4000 sites : la courbe moyenne `C(r)` est strictement décroissante de `r = 1` à 60 ; `C(1)` moyen entre 0,5 et 0,6 ; chaque graine : `C(60) < C(4)` | mesuré : `C(1) = 0,546`, `C(4) = 0,335`, `C(26) = 0,17`, `C(60) = 0,127` (24 graines) ; sur 120 graines chaque courbe est même strictement décroissante de 1 à 60 | — | inégalités |
| V-Z1-22 | un amas de DLA de 1500 sites (graine 3) : la somme de `paires_par_distance2(sites, 2 R + 1)` est `1500 × 1499` ; la mesure est la même (bits) après translation de `(500, -300)`, ordre inversé, ordre mélangé | identité (toutes les paires) ; invariances de V-Z1-16 | — | égalité |

## La vitrine

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-23 | `givre_mesurer`, `givre_correlation_rayon`, `givre_correlation_valeur`, `givre_dimension` égales à `correlation::mesure` de l'amas natif (graines 1 à 2999 particules, `0xDEAD_BEEF` à 1500, 7 à 800, `u32::MAX` à 2000) ; hors courbe : rayon 0, valeur `NaN` | doc de `vitrine` (« La mesure de la corrélation ») | — | égalité de bits |
| V-Z1-24 | pas de mesure (retour 0, rayon 0 → 0, valeur et dimension `NaN`) pour les trois flocons au temps 0 et au bout, la DLA à un site, et pour tout amas d'étendue `< 72` ; mesure (6 points) dès le premier amas d'étendue 72 | idem ; `n0` trouvé dans le test | — | égalité |
| V-Z1-25 | la mesure gardée survit à `givre_avancer(0)` et à un `givre_avancer` qui ne joue rien (scénario fini), est effacée par `givre_avancer(1)` et par `givre_demarrer` | idem | — | égalité |
| V-Z1-26 | `MESURE_SITES_MAX = 10 000` ; temps natif (release) de `mesure` : 0,008 s à 2000 sites, 0,018 s à 3000, 0,025 s à 4000, 0,040 s à 5000, 0,17 s à 10 000, 0,69 s à 20 000, 1,67 s à 30 000 | doc de `vitrine` ; mesuré sur le prototype (`temps-natif.txt`, une graine, meilleur de trois) ; le module WebAssembly **n'a pas été mesuré** | — | égalité à la constante |

## La page (forme) et le module (Node)

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-Z1-27 | éléments : `#mesurer` (bouton « Mesurer D »), `#courbe` (canvas), `#resultat`, `#barre-zinc`, `#marque-d`, `#repere-zinc`, `#compatible`, `#hors-barre`, `#taille-zinc` | consigne de la page (en-tête de `etape_z1_page.rs`) | — | présence et sorte |
| V-Z1-28 | `#repere-zinc` dit `zinc`, `Matsushita`, `1984`, `1,66`, `0,03` ; `#compatible` = « compatible, ce qui ne prouve pas que le zinc suit la DLA » ; `#taille-zinc` = « taille d'amas du zinc non relevée » | plan de Z1 ; V-Z1-35 et V-Z1-37 | — | égalité de textes |
| V-Z1-29 | aucune valeur de D dans `index.html`, `app.js` ni `lecture.js` : pas de « D = » suivi d'un chiffre dans le HTML, « mesuré par le banc : D = » suivi de `${` dans `app.js`, pas de « 1,71 » ni « 1.71 » | consigne du brief de l'étape (« ne pas geler une valeur de D dans le texte ») | — | absence |
| V-Z1-30 | un seul appel de `givre_mesurer()` dans `app.js`, dans une `function nom` qui n'appelle ni `requestAnimationFrame` ni `givre_avancer`, installée par `addEventListener("click", nom)` | consigne du brief (« mesure à la demande, jamais à chaque image ») | — | forme |
| V-Z1-31 | taille d'amas des feuilles de zinc : **NON RELEVÉE** | voir V-Z1-37 | — | la page dit « taille d'amas du zinc non relevée » |
| V-Z1-32 | `lecture.js` exporte `lireCorrelation(m)` ; `app.js` l'importe ; avec `etape-z1`, la page emploie exactement les quinze noms `givre_*` (onze + quatre) avec leurs arités ; sans, onze ou quinze | doc de `vitrine` | — | forme |
| V-Z1-33 | la sortie de `tests/etape_z1/correlation-wasm.mjs` (module commité + `lecture.js`) est celle du code natif : points, rayons, `C(r)` et dimension au bit près, contrat « avant tout démarrage », mesure effacée après `givre_avancer(1)` sauf scénario fini | code natif (`Amas` + `correlation::mesure`) | — | égalité de la sortie entière (**ne passe qu'une fois `docs/givre.wasm` reconstruit** ; non exécutable sans Node) |
| V-Z1-34 | le script sort en code 1, sans rien écrire sur la sortie standard, pour treize arguments faux | doc du script | — | code de sortie |
| V-Z1-39 | `#compatible` et `#hors-barre` portent l'attribut `hidden` dans `index.html` ; `app.js` les demande par `getElementById` et écrit `.hidden` | consigne de la page (en-tête de `etape_z1_page.rs` : `compatible` seulement si `\|D - 1,66\| <= 0,03`, `hors-barre` sinon), ajoutée par le juge | — | forme (la condition elle-même n'est pas éprouvée : DOM) |
| V-Z1-38 | les tests de l'étape 5 amendés acceptent le module et la page soit tels qu'ils sont (onze exportations) soit étendus (quinze) sans `etape-z1`, et n'acceptent que l'étendu avec | consigne du brief ; vérifié sur cinq modules synthétiques (onze, quinze, quatorze, seize exportations, une mauvaise signature : `essai-z1-outils/faire_faux_wasm.py`) | — | voir le rapport |

## Les repères (jamais des tests)

| Identifiant | Valeur | Source | Lu / non lu |
|---|---|---|---|
| V-Z1-35 | zinc : `D = 1,66 ± 0,03`, « Hausdorff dimension averaged over many examples », en accord avec la DLA 2D ; la corrélation densité-densité calculée « on the digitized patterns of the photographs » | M. Matsushita, M. Sano, Y. Hayakawa, H. Honjo, Y. Sawada, *Fractal Structures of Zinc Metal Leaves Grown by Electrodeposition*, Phys. Rev. Lett. 53, 286 (1984), **résumé** de l'éditeur (https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.53.286, lu le 8/11/2026) | **résumé lu seulement** ; le texte intégral n'est pas ouvert : ni la forme exacte de `C(r)` (par couronne ou cumulée), ni la fenêtre de `r`, ni la taille des amas ne sont lues |
| V-Z1-36 | 1,713 ± 0,003 (±0,005 avec la forme révisée du texte) : la dimension asymptotique de la DLA, **calculée** par renormalisation du premier coefficient de Laurent d'une application conforme, sur de petits amas de 1 à 20 particules (le résumé : une théorie « applicable from very small (2-3 particles) to asymptotically large » amas ; ±0,003 vient d'une extrapolation en `1/(n + α)`, ±0,005 de la forme corrigée sans passage à la limite) ; non une mesure de masse ni de corrélation sur un amas | B. Davidovitch, A. Levermann, I. Procaccia, *Convergent Calculation of the Asymptotic Dimension of Diffusion Limited Aggregates: Scaling and Renormalization of Small Clusters*, arXiv:cond-mat/0008053 (3/8/2000) ; publié Phys. Rev. E 62, R5919 | résumé et texte lus (via ar5iv, le 8/11/2026 ; relus par le juge le même jour : arXiv et ar5iv) ; le détail des équations n'est pas relu |
| V-Z1-37 | taille des amas de zinc (nombre de pixels, diamètre, résolution des photographies) et fenêtre de `r` du 1,66 : **NON RELEVÉES** | essayés : le résumé de l'éditeur (aucune taille), ADS (refus 405), Semantic Scholar (refus 403), DTIC ADA169495 (refus 403), cinq recherches : le texte intégral de 1984 n'est pas ouvert ; une thèse citée par une recherche (ajustement de la corrélation sur `5 <= r <= 50`, 1,68 ± 0,05) est celle de son auteur, pas de Matsushita ; le juge a refait une recherche (le 8/11/2026) : ni le résumé, ni scilit, ni les pages secondaires ne donnent la taille | non lu |

## Ce que cette étape ne fixe pas par une source

Que la forme cumulée de `C(r)` choisie ici soit celle de Matsushita : non lu. Que la fenêtre `4 à étendue / 8` soit la bonne pour la DLA : aucune source ne la dit ; elle
est mesurée sur des formes. Que `D` de la corrélation à 4000 et 10 000 sites (1,65 et 1,66) soit celle de la DLA infinie : elle ne l'est pas (voir `MODELE.md`).

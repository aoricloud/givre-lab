# Étape 0 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 0 attend, d'où elle vient, avec quelle tolérance et quel critère de
réussite. Chaque identifiant (`V-ALEA-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_0_*.rs`). Elle est
séparée des résultats figés de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune
valeur de cette fiche ne vient de notre code**. Une valeur y vient d'une publication lue, d'un calcul exact fait par un outil
indépendant (nommé), ou d'une convention que la documentation du module fixe (dite telle, avec la raison).

**Outils indépendants** : Python 3.13 (`decimal` à 50 ou 80 chiffres, `fractions` en rationnels exacts, entiers de taille
quelconque, `hashlib`, `zlib`) ; `sha256sum` (coreutils, Git pour Windows). Le code Rust n'est jamais l'outil. Les scripts de
calcul (`ref_alea.py`, `valeurs_alea.py`, `valeurs_autres.py`, `valeurs_fixe.py`, `erreur_ln.py`, `constantes_ln.py`) sont dans
les notes de la session (`essai-0-outils/`).

**Tolérance, règle générale** : les tests exacts (entiers, octets, chaînes) n'ont **aucune** tolérance. Seul le logarithme en a
une (V-LOG-01 et V-LOG-05), et elle a été mesurée avant d'être fixée, jamais élargie pour un cas.

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les deux seuls tests
`régression :` de l'étape 0 sont `regression_reference_vide_graine_1` et `regression_reference_du_binaire`.

---

## alea (PCG32, graine et flux)

Sources : M. E. O'Neill, PCG ; le code C de référence `pcg_basic.c` et la démonstration `pcg32-demo.c`
(`github.com/imneme/pcg-c-basic`, lus le 7/11/2026) ; la page « Using the Minimal C Implementation »
(`pcg-random.org/using-pcg-c-basic.html`, lue le 7/11/2026). `pcg32-demo.c` amorce avec `pcg32_srandom_r(&rng, 42u, 54u)` et,
à chaque ronde, tire 6 fois `pcg32_random_r`, puis 65 fois `pcg32_boundedrand_r(&rng, 2)` (pièces : `H` si non nul), puis 33 fois
`pcg32_boundedrand_r(&rng, 6)` (dés, plus un).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-ALEA-01 | six premiers tirages de (42, 54) : `0xa15c02b7 0x7b47f409 0xba1d3330 0x83d2f293 0xbfa4784b 0xcbed606e` | page pcg-random.org ci-dessus, ronde 1, ligne « 32bit » (lue en entier). Recoupée : Rosetta Code, « Pseudo-random numbers/PCG32 » (résultat de recherche) donne en décimal `2707161783 2068313097 3122475824 2211639955 3215226955` pour les cinq premiers, égaux à ces hexadécimaux | aucune (entiers) | égalité exacte des six `u32` |
| V-ALEA-02 | les 65 pièces après les 6 tirages : `HHTTTHTHHHTHTTTHHHHHTTTHHHTHTHTHTTHTTTHHHHHHTTTTHHTTTTTHTTTTTTTHT` | même page, ronde 1, ligne « Coins », lue en entier par le juge (page HTML téléchargée le 8/11/2026, texte brut) ; le modèle V-ALEA-04 et le modèle indépendant du juge donnent la même chaîne | aucune | égalité exacte des 65 caractères |
| V-ALEA-03 | les 33 dés après les 65 pièces : `3 4 1 1 2 2 3 2 4 3 2 4 3 3 5 2 3 1 3 1 5 1 4 1 5 6 4 6 6 2 6 3 3` | même page, ronde 1, ligne « Rolls », lue en entier par le juge ; les deux modèles donnent les mêmes | aucune | égalité exacte des 33 nombres |
| V-ALEA-04 | état (8 octets petit-boutistes) et incrément après amorçage de (42, 54), (0, 0), (`u64::MAX`, `u64::MAX`), (123456789, 987654321) ; quatre premiers tirages et état après eux de ces quatre couples ; état après un tirage de (42, 54) ; premiers tirages des flux 0, 1, 54, 55 de la graine 42 et de (54, 42) ; état après amorçage de `new(7)`, `new(1)`, `new(u64::MAX)` | calcul exact en entiers par un modèle Python (`ref_alea.py`) de `pcg32_srandom_r`, `pcg32_random_r` du C de référence ; **le modèle reproduit d'abord V-ALEA-01 à 03 sans écart**, puis seulement produit ces valeurs. Valeurs listées dans `tests/etape_0_alea.rs` | aucune | égalité exacte |
| V-ALEA-05 | numérateurs de `suivant_f64` de (42, 54) : `5677329748551934`, `6548306420333150`, `6742819640802732` ; `suivant_u64` : `0xA15C02B77B47F409`, `0xBA1D333083D2F293`, `0xBFA4784BCBED606E` | arithmétique entière sur V-ALEA-01 : les `u64` sont les paires de tirages mises bout à bout (premier en poids fort), les numérateurs sont ces `u64` décalés de 11 bits (`>> 11`) | aucune ; `numérateur · 2^-53` est exact en `f64` | égalité exacte |
| V-ALEA-06 | `borne(n)` pour `n` = 7, 3000000000, 2147483649, 4294967295 : 24 résultats chacun, graine 42 flux 54 | modèle Python de `pcg32_boundedrand_r` (`seuil = (-n) mod 2^32 mod n`, rejet de `r < seuil`) ; 10 rejets sur 34 tirages pour 3000000000, 16 sur 40 pour 2147483649 | aucune | égalité exacte |
| V-ALEA-07 | quatre graines fabriquées telles que le premier tirage brut vaille exactement le seuil ou le seuil moins un (n = 3000000000 : graines 17401024240561992018 puis 2631533607250342226 ; n = `u32::MAX` : 4099389142363320658 puis 18396926780031676754), et le résultat et l'état après `borne(n)` | construction : on inverse l'amorçage (`etat = (inc + graine) · M + inc` donne la graine pour un état voulu ; l'état voulu est construit bit à bit pour que la sortie XSH-RR soit la valeur voulue, avec `rot = 0`) ; vérifié par le modèle V-ALEA-04 (premier tirage = valeur voulue) ; résultats et états par le même modèle | aucune | égalité exacte : teste le bord `r >= seuil` |
| V-ALEA-08 | après 100 000 tirages de (42, 54) : XOR `0x842705F9`, somme mod 2^64 `214430088307331`, dernier `1187223689`, état ; après 1 000 tirages de (`u64::MAX`, `u64::MAX`) : XOR `0xAAE84890`, dernier `323556842`, état | modèle Python V-ALEA-04 | aucune | égalité exacte |
| V-ALEA-09 | `borne(1)` rend 0 et consomme exactement un tirage ; pour `n = 2^k` (k = 1, 2, 8, 16, 31), `borne(n) = tirage & (n − 1)` (200 tirages chacun) | définition : seuil `(-n) mod n = 0` pour une puissance de deux (aucun rejet) et `r % 2^k = r & (2^k − 1)` | aucune | égalité exacte avec un second générateur cloné qui tire sans `borne` ; ancrage : le premier tirage officiel `0xA15C02B7` (V-ALEA-01) masqué donne `1`, `0xB7`, `0x02B7`, `0x215C02B7` pour `n` = 2, 2^8, 2^16, 2^31 |
| V-ALEA-10 | `borne(0)` panique | « Panique : si n vaut 0 » dans la doc de `borne` (la référence C divise par zéro dans ce cas) | — | `#[should_panic]` |
| V-ALEA-11 | deux générateurs de même graine sont égaux ; un tirage les distingue ; un clone suit la même suite ; graines ou flux différents donnent des générateurs différents | doc de `Alea` : l'égalité est celle des deux `u64` | aucune | égalité / inégalité |

## reseau : la grille carrée, le disque, la bande

Source des valeurs : la **convention** écrite dans la documentation de `reseau` (ordre des voisins, `y * largeur + x`, reste
euclidien) : ce sont des décisions de conception, non des valeurs publiées ; la raison : un ordre fixe et documenté pour que les
mains et les étapes suivantes s'y accordent. Les cas sont écrits à la main ou recomptés par une définition naïve réécrite dans
le test.

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-GRILLE-01 | `VOISINS4 = [(1,0),(0,1),(-1,0),(0,-1)]`, `VOISINS8 = [(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1),(0,-1),(1,-1)]` | convention de la doc de `reseau` | aucune | égalité |
| V-GRILLE-02 | dimensions et nombre de cases de 5×4, 1×1, 1×7 ; `4096 × 4096 = 16777216` | définition | aucune | égalité |
| V-GRILLE-03 | `index(x, y) = y * largeur + x` : `index(2, 3) = 17`, `index(4, 3) = 19` sur 5×4 ; `position` inverse | définition (convention de la doc) | aucune | égalité sur toute la grille |
| V-GRILLE-04 | paniques : `index` hors grille (dont `x = largeur`), `position(cases)`, `new` à dimension nulle ou produit débordant, `voisins*` hors grille | « panique » écrit dans la doc de chaque fonction | — | `catch_unwind` rend une erreur |
| V-GRILLE-05 | `contient` : bords, négatifs, `isize::MIN` et `isize::MAX` | définition | aucune | égalité |
| V-GRILLE-06 | voisins à 4 et à 8, cases intérieures, de bord, de coin, grilles 1×1 et 1×3 ; symétrie ; rangs pairs de `voisins8` = `voisins4` | définition par la doc ; dénombrement naïf réécrit dans le test | aucune | égalité sur chaque case de 7×6 |
| V-GRILLE-07 | `enrouler_x` = reste euclidien : `-1 → l - 1`, `l → 0`, `-8 → 6` pour `l = 7` | définition ; comparé à `((x % l) + l) % l` en entiers | aucune | égalité |
| V-GRILLE-08 | `dans_disque(dx, dy, r)` ⇔ `dx² + dy² <= r²` : triplets pythagoriciens `(3,4,5)`, `(5,12,13)`, `(8,15,17)`, `(7,24,25)`, `(20,21,29)` dedans à r, dehors à r − 1 | arithmétique entière exacte (les triplets sont vérifiés par `a² + b² = c²`) ; définition réécrite dans le test sur 101 × 101 × 61 cas | aucune | égalité |
| V-GRILLE-09 | nombre de points entiers du disque de rayon 0 à 20 : `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441, 529, 613, 709, 797, 901, 1009, 1129, 1257` | OEIS A000328, « Number of points of norm <= n^2 in square lattice » (lue le 7/11/2026) ; recoupée par un dénombrement exact en Python | aucune | égalité |
| V-GRILLE-10 | `dans_disque(i32::MIN, i32::MIN, u32::MAX)` vrai ; `(i32::MAX, i32::MAX)` dans le disque de rayon 3037000499, pas dans 3037000498 | entiers exacts en Python : `2·(2^31 − 1)² = 9223372028264841218`, `3037000498² = 9223372024852248004`, `3037000499² = 9223372030926249001`, `(2^32 − 1)² = 18446744065119617025 >= 2^63` | aucune | égalité |

## reseau : la grille hexagonale

Sources : Red Blob Games, « Hexagonal Grids » (`redblobgames.com/grids/hexagons/`, lu le 7/11/2026) : coordonnées cubiques
`q + r + s = 0`, axiales `(q, r)` avec `s = -q - r`, distance `max(|q|, |r|, |s|)`, rotation de 60° « à droite » `[q, r, s] → [-r, -s, -q]`,
carte hexagonale de rayon N par `q` de −N à N et `r` de `max(-N, -q-N)` à `min(N, -q+N)`, `1 + 3N(N+1)` cases ; OEIS A003215
(lue le 7/11/2026). L'**ordre des directions** (`(1,0),(0,1),(-1,1),(-1,0),(0,-1),(1,-1)`) et l'**ordre canonique des cases** (`r`
puis `q` croissants) sont des conventions de la doc : l'ordre des directions est choisi pour que `tourner60` envoie la direction `k`
sur la direction `k + 1` (la formule de rotation est celle de Red Blob Games, imposée par le plan de l'étape).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-HEX-01 | `DIRECTIONS_HEX = [(1,0),(0,1),(-1,1),(-1,0),(0,-1),(1,-1)]` | convention de la doc, vérifiée par V-HEX-07 | aucune | égalité |
| V-HEX-02 | nombre de cases `3R(R+1)+1` : 1, 7, 19, 37, 61, 91, 127, 169, 217, 271, 331, 397 pour R = 0 à 11 ; `3·32768·32769 + 1` pour R = 32768 | OEIS A003215 ; Red Blob Games | aucune | égalité |
| V-HEX-03 | `contient` ⇔ `max(|q|, |r|, |s|) <= R` ; cas `R = 2` écrits à la main ; définition réécrite pour `q, r ∈ [−9, 9]`, R ∈ [0, 6] | Red Blob Games (définition de la distance et de la carte) | aucune | égalité |
| V-HEX-04 | ordre canonique pour R = 0, 1, 2 écrit en toutes lettres (1, 7, 19 cases) ; énumération naïve R ≤ 12 | convention de la doc (l'énumération à deux boucles imbriquées de Red Blob Games, avec `r` en boucle externe) | aucune | `position(i)` et `index` égaux à l'énumération |
| V-HEX-05 | paniques : `index(R, 1)`, `index(R+1, 0)`, `position(cases)`, `new(32769)` | « panique » écrit dans la doc | — | `catch_unwind` |
| V-HEX-06 | voisins : cas écrits à la main sur R = 2 (centre, deux coins, un côté), R = 0 et R = 1 ; définition réécrite sur R = 5 ; 6 voisins à l'intérieur, 4 sur un côté, 3 dans un coin | définition par la doc ; dénombrement naïf | aucune | égalité |
| V-HEX-07 | `tourner60(q, r) = (-r, q + r)` : chaîne `(1,0) → (0,1) → (-1,1) → (-1,0) → (0,-1) → (1,-1) → (1,0)` ; formule cubique `(q, r, s) → (-r, -s, -q)` ; six rotations = identité, trois = `(-q, -r)`, bijection de l'hexagone, distance conservée | Red Blob Games, « rotate 60° right » (lu) ; propriétés démontrées en entiers | aucune (entiers) | égalité exacte sur tout l'hexagone de rayon 6, et sur `(100, -37)` |
| V-HEX-08 | distance : `((0,0),(3,-5)) = 5`, `((2,-3),(-1,4)) = 7`, `((0,0),(1,1)) = 2`, `2^29`, `2^30` ; égale au plus court chemin par parcours en largeur sur R = 10 ; `6d` cases à distance `d` | Red Blob Games (formule de distance) ; parcours en largeur écrit dans le test (n'emploie que `voisins`) ; `6d` : propriété classique des anneaux hexagonaux | aucune | égalité |

## math : racine entière

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-RACINE-01 | `r² <= n < (r+1)²` pour `n ∈ [0, 70000]`, pour `k²−1, k², k²+2k, k²+2k+1` avec `k` ∈ [0, 3000], puissances de 2, quatre plus grands `u32` et 5 000 `u32` pseudo-aléatoires | définition de la partie entière de la racine, vérifiée en `u128` | aucune | propriété vraie |
| V-RACINE-02 | `racine(u64::MAX) = 4294967295` ; `4294967295² = 18446744065119617025` ; `racine(2^63) = 3037000499` | entiers exacts en Python (`math.isqrt`) | aucune | égalité |

## math : logarithme

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-LOG-01 | 26 couples `(x, ln x)` listés dans `tests/etape_0_math.rs` (`2`, `10`, `0.5`, `3`, `7`, `0.1`, `1e-10`, `123456.789`, `±1e300`, `f64::MAX`, `f64::MIN_POSITIVE`, `5e-324`, `1 ± 2^-52`…) | `ln` exact du `f64` donné, calculé en 50 chiffres décimaux (`decimal.Decimal.ln`), arrondi au `f64` le plus proche | relative `5e-16` | `|obtenu − attendu| <= 5e-16 · |attendu|`. Mesure : le prototype fait au pire `3,0e-16` (moyenne `5,4e-17`, écart-type `4,2e-17`) ; la valeur écrite a elle-même jusqu'à `1,1e-16` d'erreur d'arrondi ; `5e-16` est la précision annoncée dans la doc, fixée après mesure |
| V-LOG-02 | `logarithme(1.0) = 0` exactement ; `logarithme(1 + 2^-52) > 0` et `logarithme(1 - 2^-53) < 0` | la doc : « exactement 0 » (série à t = 0) ; le signe de `ln` de part et d'autre de 1 | aucune | égalité ; signes |
| V-LOG-03 | NaN pour `0.0`, `-0.0`, négatifs, `-∞`, NaN ; `+∞` pour `+∞` | convention de la doc (la mathématique voudrait `-∞` pour 0 ; le choix NaN est celui du plan de l'étape) | — | `is_nan` ; égalité à `+∞` |
| V-LOG-04 | logarithmes de huit sous-normaux (`5e-324` : `-744.4400719213812` ; `1e-323`, `1.5e-323`, `0x3E8`, `0x40000000`, `0x100000`, `0x0008000000000000`, `0x000FFFFFFFFFFFFF` : `-708.3964185322641`) et de `1e-310` : `-713.8013788281542` | `decimal.Decimal.ln` à 50 chiffres | relative `5e-16` | idem V-LOG-01 |
| V-LOG-05 | 13 402 points fixes (`common::points_logarithme`) : écart à `f64::ln` du système | `f64::ln` du système (permis dans un test seulement) ; vérifié lui-même contre le logarithme exact : `f64::ln` fait au pire `1,1e-16` | relative `5e-16` | `|logarithme(x) − x.ln()| <= 5e-16 · |x.ln()|` pour tout point, `logarithme(1) = 0`. Mesure du prototype : `3,2e-16` au pire contre `f64::ln` (Windows, UCRT), `3,0e-16` contre l'exact, atteint juste sous `√2/2` ; rejoué par le juge sous Linux (glibc, Rust 1.99.1) : mêmes bits de `logarithme` sur 38 402 points, `3,203e-16` au pire contre le `f64::ln` de Linux (qui diffère de celui de Windows en 7 points sur 38 402, chacun à `1,11e-16` au plus de l'exact) ; la marge couvre l'erreur d'une autre bibliothèque mathématique (au plus 1 ulp, `2,2e-16` relatif). Jamais élargie pour un cas |

## math : la table EXP_NEG

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-EXP-01 | `EXP_NEG[k] = round(2^32 · e^(−k/16))` pour k = 0 à 365 ; `EXP_NEG[0] = 4294967296`, `EXP_NEG[16] = 1580030169`, `EXP_NEG[365] = 1` ; `exp_neg(k) = 0` pour `k >= 366` | **deux calculs indépendants du Rust** : (1) Python `decimal` à 80 chiffres, `Decimal(-k/16).exp() · 2^32`, arrondi demi-supérieur ; (2) Python `fractions` : série de Taylor de `e^(−1/16)` à 60 termes en rationnels exacts, puissances exactes, arrondi `floor(v + 1/2)` ; les deux donnent les mêmes 366 entiers et `round(2^32 · e^(−366/16)) = 0`. La valeur la plus proche d'un demi en est à `3,75e-4` unités (aucune ambiguïté d'arrondi) | aucune | égalité exacte des 366 entrées (la liste complète est ci-dessous) |
| V-EXP-02 | `EXP_NEG[k+1] <= EXP_NEG[k]` ; strictement décroissante tant que la valeur dépasse `2^20` ; `|EXP_NEG[k+16]·e − EXP_NEG[k]| <= e/2 + 1/2 + 1e-5` | propriété de `e^(−(k+16)/16) = e^(−k/16)/e` : chaque entrée est à au plus une demi-unité de sa valeur exacte, d'où `e/2 + 1/2` ; `1e-5` couvre l'erreur de `e` en `f64` et du produit (au plus `1e-6`) | `1e-5` unités du dernier chiffre (raison ci-contre) | inégalité ; ancrages `EXP_NEG[0] = 2^32` et `EXP_NEG[365] = 1` (valeurs de V-EXP-01) pour qu'une table de zéros ne passe pas |

Liste complète de V-EXP-01 (k de 0 à 365, six par ligne) :

```text
4294967296, 4034748382, 3790295335, 3560652950, 3344923893, 3142265200,
2951884975, 2773039306, 2605029347, 2447198598, 2298930330, 2159645183,
2028798896, 1905880184, 1790408741, 1681933360, 1580030169, 1484300980,
1394371730, 1309891017, 1230528733, 1155974766, 1085937795, 1020144150,
958336741, 900274053, 845729205, 794489063, 746353404, 701134137,
658654567, 618748704, 581260615, 546043815, 512960693, 481881975,
452686223, 425259351, 399494189, 375290060, 352552385, 331192315,
311126387, 292276193, 274568073, 257932835, 242305474, 227624928,
213833830, 200878294, 188707693, 177274472, 166533955, 156444172,
146965699, 138061498, 129696774, 121838844, 114457002, 107522402,
101007949, 94888187, 89139202, 83738531, 78665070, 73898994,
69421681, 65215634, 61264418, 57552595, 54065659, 50789987,
47712777, 44822006, 42106378, 39555281, 37158748, 34907413,
32792480, 30805684, 28939262, 27185921, 25538809, 23991491,
22537920, 21172416, 19889645, 18684592, 17552550, 16489094,
15490071, 14551575, 13669939, 12841720, 12063679, 11332778,
10646160, 10001141, 9395203, 8825976, 8291237, 7788897,
7316991, 6873677, 6457222, 6065999, 5698479, 5353225,
5028890, 4724205, 4437980, 4169096, 3916503, 3679214,
3456302, 3246895, 3050176, 2865375, 2691771, 2528685,
2375479, 2231556, 2096353, 1969341, 1850025, 1737938,
1632641, 1533725, 1440801, 1353507, 1271502, 1194466,
1122097, 1054113, 990247, 930251, 873890, 820944,
771205, 724480, 680586, 639352, 600615, 564226,
530041, 497928, 467760, 439419, 412796, 387786,
364292, 342220, 321486, 302008, 283711, 266521,
250374, 235204, 220954, 207567, 194991, 183177,
172079, 161653, 151859, 142659, 134015, 125896,
118268, 111103, 104371, 98048, 92107, 86527,
81284, 76360, 71733, 67387, 63304, 59469,
55866, 52481, 49301, 46314, 43508, 40872,
38396, 36070, 33884, 31831, 29903, 28091,
26389, 24790, 23288, 21877, 20552, 19307,
18137, 17038, 16006, 15036, 14125, 13269,
12465, 11710, 11001, 10334, 9708, 9120,
8567, 8048, 7561, 7103, 6672, 6268,
5888, 5531, 5196, 4882, 4586, 4308,
4047, 3802, 3571, 3355, 3152, 2961,
2781, 2613, 2455, 2306, 2166, 2035,
1912, 1796, 1687, 1585, 1489, 1399,
1314, 1234, 1159, 1089, 1023, 961,
903, 848, 797, 749, 703, 661,
621, 583, 548, 515, 483, 454,
427, 401, 376, 354, 332, 312,
293, 275, 259, 243, 228, 214,
201, 189, 178, 167, 157, 147,
138, 130, 122, 115, 108, 101,
95, 89, 84, 79, 74, 70,
65, 61, 58, 54, 51, 48,
45, 42, 40, 37, 35, 33,
31, 29, 27, 26, 24, 23,
21, 20, 19, 18, 17, 16,
15, 14, 13, 12, 11, 11,
10, 9, 9, 8, 8, 7,
7, 6, 6, 6, 5, 5,
5, 4, 4, 4, 4, 3,
3, 3, 3, 3, 3, 2,
2, 2, 2, 2, 2, 2,
2, 1, 1, 1, 1, 1,
1, 1, 1, 1, 1, 1,
1, 1, 1, 1, 1, 1,
```

## math : la virgule fixe Fixe (Q31.32)

Source : l'arrondi `floor(valeur exacte + 1/2)` calculé en rationnels exacts (`fractions.Fraction`) par `valeurs_fixe.py`, pour des
entrées écrites à la main (cas limites) et tirées par `random.Random(20261107)`. Convention de la doc : au plus proche, demi-cas vers
+∞ (partout, y compris la conversion depuis un flottant). Aucun résultat ne vient du Rust. Tolérance : aucune (entiers).

| Identifiant | Valeur | Critère |
|---|---|---|
| V-FIXE-01 | `UN = 4294967296` (brut), `entier(n) = n · 2^32` y compris `i32::MIN` et `i32::MAX` | égalité |
| V-FIXE-02 | 60 cas `rapport(num, den) = floor(num·2^32/den + 1/2)` (dont `1/3 → 1431655765`, `2/3 → 2863311531`, `-1/3 → -1431655765`, `1/2^33 → 1`, `-1/2^33 → 0`, `3/2^33 → 2`, `-3/2^33 → -1`, dénominateurs négatifs, extrêmes) | égalité |
| V-FIXE-03 | 60 cas `a * b = floor(a·b/2^32 + 1/2)` (dont les demi-cas `Fixe(±2^31) * Fixe(1)`, et `Fixe(i64::MAX) * UN`) | égalité |
| V-FIXE-04 | 60 cas `div_entier = floor(brut/n + 1/2)`, `n` positif ou négatif (dont `5/2 → 3`, `-5/2 → -2`, `-7/2 → -3`, `i64::MAX/2`, `i64::MIN/3`) ; moyenne de six voisins : `(3+5+5+6+8+9)/6 = 6`, `21/6 → 4`, `-21/6 → -3`, `UN/6 → 715827883` | égalité |
| V-FIXE-05 | 48 conversions `depuis_f64(x) = floor(x·2^32 + 1/2)` données par les bits de `x`, dont `0.5·2^-32 → 1`, `-0.5·2^-32 → 0`, `1.5·2^-32 → 2`, `-1.5·2^-32 → -1`, `0.1`, `-2147483647.5`, et (ajout du juge) `±(2^20 + 2^-32)` → `±4503599627370497`, `2^20 + 3·2^-32` → `4503599627370499` (là où `(x·2^32 + 0.5).floor()` en flottant se trompe d'une unité) | égalité |
| V-FIXE-06 | `vers_f64(brut) = brut/2^32` exactement pour `|brut| <= 2^53` (14 cas donnés par les bits) ; aller-retour sur 2 000 champs bruts de moins de `2^53` ; `Fixe(i64::MAX).vers_f64() = 2^31` (arrondi de `i64::MAX as f64`) | égalité des bits |
| V-FIXE-07 | `+`, `-`, opposé exacts sur les champs bruts ; l'ordre est celui des valeurs | égalité |
| V-FIXE-08 | paniques : débordement de `+`, `-`, opposé de `i64::MIN`, `*` à `2^32`, `rapport` par zéro ou hors intervalle, `div_entier` par 0 et `i64::MIN / -1`, `depuis_f64` de NaN, ∞, `±2^31`, `1e300` ; bornes incluses sans panique | « jamais de rebouclage silencieux » (doc de `Fixe`) ; `catch_unwind` |

## scenario

Source : la **définition du format** écrite dans la documentation de `scenario` : c'est une décision de conception de l'étape 0
(avec les bornes `COTE_MAX = 4096` et `RAYON_MAX = 2047`), non une valeur publiée. Raison des choix : un format que les étapes
suivantes étendent sans modifier ce module (toute directive bien formée est gardée), des erreurs numérotées par ligne, des entiers
écrits sans ambiguïté (chiffres ASCII seulement). Les cas sont des textes écrits à la main dont le résultat se lit sans calcul.
Tolérance : aucune. Critère : égalité exacte du résultat ou de l'erreur.

| Identifiant | Ce qu'il fixe |
|---|---|
| V-SCEN-01 | `boite 64 48` → `Boite { largeur: 64, hauteur: 48 }`, `hexagone 3` → `Hexagone { rayon: 3 }`, sans autre directive |
| V-SCEN-02 | lignes vides, blanches, commentaires (même indentés), séparateurs multiples, `\r` final |
| V-SCEN-03 | bornes : `1`, `4096`, `2047` acceptés ; `0`, `4097`, `2048` et les entiers qui dépassent un `u32` refusés ; zéros de tête permis |
| V-SCEN-04 | entiers en chiffres ASCII seulement (`+5`, `-5`, `5.0`, `0x10`, `1e2`, `5_0`, `٣` refusés) ; nombre d'arguments exact |
| V-SCEN-05 | une seule forme ; la seconde est la ligne fautive |
| V-SCEN-06 | noms de directive `[a-z0-9_]+` ; `Boite`, `boite-2`, `é`… refusés |
| V-SCEN-07 | numéro de ligne à partir de 1, lignes ignorées comptées ; la première ligne fautive l'emporte |
| V-SCEN-08 | `Manquante("forme")` sans forme (fichier vide, commentaires, autres directives) ; les mêmes textes suivis de `hexagone 1` se lisent |
| V-SCEN-09 | les autres directives sont gardées, dans l'ordre, avec ligne, nom, arguments ; `directive(nom)` rend la première |
| V-SCEN-10 | un `#` en fin de ligne n'est pas un commentaire |
| V-SCEN-11 | `Directive::entier` : chiffres ASCII, `u64::MAX` accepté, erreur `LigneInvalide(ligne de la directive)` sinon |
| V-SCEN-12 | `Directive::flottant` : `-?chiffres(.chiffres)?` ; valeurs arrondies au plus proche (`0.1 = 0x3FB999999999999A`, `123456789.123456789 = 0x419D6F34547E6B75`, `-3.14159265358979323846 = 0xC00921FB54442D18`, calculées par `float()` de Python, correctement arrondi) ; refus de `1e3`, `+1`, `.5`, `5.`, `inf`, `nan`… |
| V-SCEN-13 | `scenarios/vide.txt` est une boîte 64 × 48 sans autre directive |

## empreinte (SHA-256)

Sources : NIST, exemples SHA-256 : `SHA256.pdf` (« One Block Message Sample » : `abc` ; « Two Block Message Sample » :
`abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq`), `SHA2_Additional.pdf` (« SHA-256 Test Data »), FIPS 180-2 annexe B.3 ;
tous lus dans le texte des PDF le 7/11/2026 (flux décompressés avec `zlib` de Python). Tolérance : aucune (octets).

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-SHA-01 | `""` → `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | CAVP du NIST, `SHA256ShortMsg.rsp` (`shabytetestvectors.zip`, CAVS 11.0, csrc.nist.gov), entrée `Len = 0`, lue par le juge le 8/11/2026 ; recoupée par Python `hashlib` | égalité |
| V-SHA-02 | `abc` → `ba7816bf 8f01cfea 414140de 5dae2223 b00361a3 96177a9c b410ff61 f20015ad` | `SHA256.pdf`, « Message Digest is » du premier exemple | égalité |
| V-SHA-03 | message de 448 bits → `248d6a61 d20638b8 e5c02693 0c3e6039 a33ce459 64ff2167 f6ecedd4 19db06c1` | `SHA256.pdf`, second exemple | égalité |
| V-SHA-04 | un million de `a` → `cdc76e5c 9914fb92 81a1c7e2 84d73e67 f1809a48 a497200e 046d39cc c7112cd0` | FIPS 180-2, annexe B.3 | égalité |
| V-SHA-05 | `SHA2_Additional.pdf`, « SHA-256 Test Data » : #1 1 octet `0xbd` → `68325720…732b` ; #2 4 octets `c98c8e55` → `7abc22c0…d9504` ; #3 55 octets nuls → `02779466…8ef7` ; #4 56 → `d4817aa5…9fbb` ; #5 57 → `65a16cb7…1785` ; #6 64 → `f5a5fd42…fb4b` ; #7 1000 → `541b3e9d…8a53` (empreintes complètes dans le test) | `SHA2_Additional.pdf`, section « SHA-256 Test Data » (lue) ; recoupées par `hashlib` | égalité |
| V-SHA-06 | `a` répété 55, 56, 63, 64, 65, 119, 120 fois | Python `hashlib` ; recoupé par `sha256sum` (mêmes empreintes). Longueurs : frontières du bourrage (55 : tient dans un bloc ; 56 : en demande un second ; 64 ; 119/120 la même frontière au second bloc) | égalité |
| V-SHA-07 | les 256 octets `0..=255` → `40aff2e9…4880` | Python `hashlib` | égalité |
| V-SHA-08 | `h = sha256(h ‖ "a"×n)` pour n de 0 à 300 → `8c913ede…b24c` | Python `hashlib` | égalité |
| V-SHA-09 | les 65 vecteurs « SHA-256 ShortMsg » (0 à 512 bits, octets quelconques) et le premier « LongMsg » (1304 bits) du CAVP | CAVP du NIST, `SHA256ShortMsg.rsp` et `SHA256LongMsg.rsp` (`shabytetestvectors.zip`, CAVS 11.0), copiés tels quels par le juge le 8/11/2026 ; les 129 vecteurs des deux fichiers recoupés par Python `hashlib` | égalité |

## image : CRC-32, Adler-32, PNG

Sources : catalogue de CRC de R. Williams (`reveng.sourceforge.io/crc-catalogue/17plus.htm`, lu le 7/11/2026) : CRC-32/ISO-HDLC,
polynôme `0x04c11db7` réfléchi, init `0xffffffff`, xorout `0xffffffff`, **check `0xcbf43926`** ; article « Adler-32 » de Wikipédia
anglais (lu) : `Wikipedia` → `0x11E60398`, `A = 920`, `B = 4582`, modulo 65521 ; RFC 1950 (lu) §2.2 (en-tête CMF/FLG : `0x78 0x01`
vérifie `(CMF·256 + FLG) mod 31 = 0`, Adler-32 en fin de flux, grand-boutiste) ; RFC 1951 (lu) §3.2.3 (BFINAL, BTYPE) et §3.2.4
(bloc stocké : `LEN`, `NLEN` complément à un, petit-boutiste, 65535 octets au plus) ; spécification PNG, 3e édition, W3C
(`w3.org/TR/png-3/`, lue : §5.2 signature `89 50 4E 47 0D 0A 1A 0A`, §5.3 chunks : longueur, type, données, CRC sur type et données,
§11.2.1 IHDR de 13 octets, type de couleur 2 = RVB, §9.2 filtre 0 « None », §7.3 un octet de filtre par ligne). Les autres valeurs
viennent de Python `zlib` (`crc32`, `adler32`, `decompress`). Tolérance : aucune.

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-IMG-01 | `crc32("123456789") = 0xCBF43926`, `crc32("") = 0`, `crc32("IEND") = 0xAE426082` | catalogue (check) ; `zlib.crc32` | égalité |
| V-IMG-02 | `adler32("Wikipedia") = 0x11E60398`, `adler32("") = 1` | Wikipédia ; RFC 1950 | égalité |
| V-IMG-03 | `crc32(0..=255) = 0x29058C73` ; `adler32(0..=255) = 0xADF67F81` ; `adler32('a'×5552) = 0x47333829`, `×5553 = 0x7FBD388A` ; `adler32(0xFF×100000) = 0x149A302C` ; motif de 102 400 octets (`(i·7+3) mod 256`) : `crc32 = 0x2CD27A54`, `adler32 = 0xC63D43AA` ; comparés aussi à un CRC bit par bit et un Adler à modulo par octet écrits dans le test | Python `zlib` | égalité |
| V-IMG-04 | les 75 octets du PNG 2 × 1 (rouge, bleu), un par un (table dans le test) ; IEND `00 00 00 00 49 45 4E 44 AE 42 60 82` | octets posés à la main d'après la spécification PNG et RFC 1950/1951 ; CRC de IHDR `7B40E8DD` et d'IDAT `5536BAC7`, Adler `070001FF` par `zlib` ; `zlib.decompress` relit le flux IDAT et retrouve `00 FF 00 00 00 00 FF` | égalité |
| V-IMG-05 | petites images 1×1, 2×1, 1×2, 3×5, 17×13, 64×48 relues par le décodeur du test : un bloc, `IDAT = 2 + 5 + L + 4` | structure de la spécification ; décodeur du test avec son propre CRC et son propre Adler | égalité |
| V-IMG-06 | blocs stockés aux limites : `L` = 65534, 65535, 65536, 131070, 131071 → `[65534]`, `[65535]`, `[65535, 1]`, `[65535, 65535]`, `[65535, 65535, 1]` ; seul le dernier est final ; pas de bloc vide | RFC 1951 §3.2.4 (65535 octets au plus) ; la coupe en morceaux de 65535, dernier plein quand `L` est un multiple, est la convention de la doc de `ecrire_png` | égalité |
| V-IMG-07 | `ecrire_png` panique pour largeur ou hauteur nulle, ou longueur ≠ `3·l·h` | « panique » écrit dans la doc | `catch_unwind` |

## rendu

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-REND-01 | premiers pixels de la graine 42 : `A1 A1 A1`, `7B 7B 7B`, `BA BA BA` | octet de poids fort de V-ALEA-01 (`0xa1`, `0x7b`, `0xba`) | égalité |
| V-REND-02 | 8 × 5 de graine 42 : `161, 123, 186, 131, 191, 203, 191, 129, 230, 249, 50, 29, 237, 56, 43, 28, 162, 132, 206, 178, 145, 85, 71, 72, 177, 194, 26, 121, 95, 30, 133, 41, 109, 195, 160, 228, 226, 64, 142, 227` ; 5 × 8 donne les mêmes octets | modèle Python V-ALEA-04 (`>> 24`) ; les six premiers sont ceux de V-ALEA-01 | égalité |
| V-REND-03 | 3 × 2 de graine 7 : `164, 108, 25, 183, 185, 125` ; graine 1 : `155, 12, …` ; longueurs `3·l·h` | modèle Python V-ALEA-04 | égalité |

## lancer

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-LANC-01 | empreintes de neuf cas dont l'écriture canonique est écrite octet par octet dans le test : `boite 3 2` graine 7 → `f637e360…5530` (octets `4749565245303030 00 03000000 02000000 d1a0bdb386a42205 6d00000000000000`), `boite 2 3` → `968a78c8…0d56`, `boite 64 48` graines 1 et 2 → `5a21e2ae…f613` et `584cfbd2…778a`, `boite 1 1` graine 0 → `c242f3b6…b909`, `boite 4096 1` graine `u64::MAX` → `421fddb2…3421`, `hexagone 1`/`3` graine 7 → `ef2dffc8…316a`/`a282a3db…9788`, `hexagone 2047` graine 42 → `4d443e2a…44fc` | écriture canonique posée à la main d'après la doc de `lancer` (étiquette, forme, `u32` petit-boutiste, état `Alea::new(graine).octets()` par le modèle V-ALEA-04) ; SHA-256 par Python `hashlib` | égalité |
| V-LANC-02 | mesures : `temps,cases\n0,<cases>\n` : 6, 3072, 4096, 1, 7, 37, 12576769 | définition (`largeur·hauteur` ; `3R(R+1)+1`, OEIS A003215 : `3·2047·2048 + 1 = 12576769`) | égalité |
| V-LANC-03 | aperçu relu par le décodeur du test : dimensions (3×2 ; 5×5 pour `hexagone 2` ; 64×48) et pixels = `>> 24` des tirages (graine 7 : `164, 108, 25, 183, 185, 125, 208, 195, 140, 168, 18, 10, 203, 225, 110, 227, 61, 98, 86, 119, 224, 68, 49, 70, 57`) ; SHA-256 des 9 216 octets de pixels de 64×48 graine 1 : `ad8e00dc…6a74` | modèle Python V-ALEA-04 ; `hashlib` | égalité |
| V-LANC-04 | déterminisme, 20 graines → 20 empreintes distinctes, formes distinctes | propriétés exactes | vérifié |
| V-LANC-05 | directive restante → `DirectiveInconnue(ligne de la première)` | doc de `lancer` (décision de l'étape 0, parce que les étapes suivantes feront consommer leurs directives) | égalité |

## binaire

| Identifiant | Ce qu'il fixe | Source |
|---|---|---|
| V-BIN-01 | succès : code 0, dossier créé avec ses parents, trois fichiers exactement, empreinte de V-LANC-01 suivie de `\n`, mesures, PNG relu | doc de `src/main.rs` ; valeurs de V-LANC-01 à 03 |
| V-BIN-02 | options dans tous les ordres ; graine `18446744073709551615` | doc ; `boite 4096 1` graine max → `421fddb2…3421` (V-LANC-01) |
| V-BIN-03 | `hexagone 2` : aperçu 5 × 5, 19 cases | V-LANC-02 et 03 |
| V-BIN-04 | un dossier existant voit ses fichiers remplacés ; `scenarios/vide.txt` graines 1 et 2 | V-LANC-01 |
| V-BIN-05 | erreurs d'arguments et de fichier : code 1, message non vide sans `panicked`, rien créé | doc de `src/main.rs` (décision : code 1 et non « non nul », pour distinguer une erreur propre d'une panique, code 101) |
| V-BIN-06 | erreurs de scénario : `ligne n` dans le message (ou `forme`), rien écrit même si le dossier existe | doc de `src/main.rs` |
| V-BIN-07 | référence `references/vide-graine-1.empreinte` (régression, produite par la vague du code) | résultat figé de notre code : ne prouve que la stabilité |

## regles

| Identifiant | Ce qu'il fixe | Source |
|---|---|---|
| V-REG-01 | le détecteur de fautes voit `x.ln()`, `f64::exp(…)`, `<f64>::exp`, une fonction interdite déclarée dans un bloc `extern`, `link_name`, `HashMap`… et ne voit pas les commentaires, chaînes, identifiants voisins, champs (`self.log`), ni une fonction exportée `extern "C" fn` | textes écrits dans le test |
| V-REG-02 | aucun appel `f64` interdit, aucune `HashMap`/`HashSet`, aucune horloge ni fil dans `src/` | `AGENTS.md`, règle 2 |
| V-REG-03 | aucune dépendance, aucun paquet externe dans `Cargo.lock`, pas de `.cargo/` | `AGENTS.md`, règles 1 et 8 |

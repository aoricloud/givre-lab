# Étape 1 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 1 attend, d'où elle vient, avec quelle tolérance et quel critère de réussite.
Chaque identifiant (`V-PENTE-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_1_*.rs`). Elle est séparée des résultats
figés de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune valeur « juste » de cette fiche
ne vient du prototype de l'écrivain** ; les seules valeurs tirées d'un prototype sont les bandes statistiques (V-STAT-01 et V-STAT-02), dites
mesurées, et V-STAT-02 est étiquetée « régression ». Une valeur y vient d'une publication lue (dite lue, ou dite non lue), d'un calcul exact fait par un outil
indépendant (nommé), ou d'une convention que la documentation d'un module fixe (dite telle, avec sa raison).

**Outils indépendants** (Python 3.13, dans `essai-1-outils/` des notes de la session ; le code Rust n'est jamais l'outil) :
`valeurs_mesures.py` (rationnels `fractions`, 60 chiffres `decimal`) ; `ref_alea.py` (le modèle PCG32 de l'étape 0, qui reproduit le vecteur
officiel) ; `ref_dla.py` (**un troisième code** : la DLA écrite en Python d'après la doc seule de `src/dla.rs`, avec `set` et listes, sans
rien d'optimisé) ; `valeurs_dla.py` (les SHA-256 `hashlib`, les CSV, les images). Les sorties brutes : `valeurs_mesures.txt`, `valeurs_dla.txt`,
`valeurs_dla_grand.txt`. `ref_dla.py` et le prototype Rust donnent les MÊMES sites et le MÊME état du générateur sur 21 couples (graine, n)
(graines 1, 2, 3, 7, 42, 1000, 99999 ; n = 30, 200, 600) et la même empreinte pour le scénario d'exemple (3000 sites).

**Tolérance, règle générale** : les tests exacts (entiers, octets, chaînes, SHA-256) n'ont **aucune** tolérance. Les mesures qui valent
exactement une valeur à l'arrondi près ont `1e-12` (l'erreur mesurée est `0` à `2e-16` : voir `essai-1-outils/erreurs-mesures.txt` ; `1e-12`
laisse la place à un `logarithme` de précision `5e-16`, le seul écart qu'une autre implémentation fidèle puisse introduire) ; les deux mesures
statistiques ont une bande mesurée (V-STAT-01, V-STAT-02).

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les deux tests `régression :` de l'étape 1
sont `etape_1_lancer.rs::regression_reference_dla_graine_1` et `etape_1_dla_statistique.rs::dimension_de_masse_rayon`.

**Rejoué par le juge** (`essai-1-juge.md`) : un second prototype écrit d'après la doc des stubs seule (sans lire celui de l'écrivain ; autres
structures : grille rangée par colonnes, anneau par racines entières, anneaux gardés dans un `BTreeMap`) passe les 42 tests et donne les mêmes
empreintes ; un troisième simulateur du juge, hors du dépôt, donne les mêmes sites sur trois graines à 20 000 sites ; les sites, les empreintes
et les mesures sont identiques au bit près sous Linux (Rust 1.99.1) et sous Windows.

---

## pente : les moindres carrés

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-PENTE-01 | `(0,1) (1,3) (2,4) (3,8)` : pente `11/5 = 2,2` ; `x` contre `y` : `11/26` ; deux points extrêmes : `7/3` | rationnels exacts (Python `fractions`, `pente_exacte`) | `1e-12` | `|pente - 2,2| <= 1e-12`, écart à `7/3` supérieur à `0,1`, `pente(ys, xs)` égal à `11/26` |
| V-PENTE-02 | `x = 2, -1, 1/2, 4, 3`, `y = 1, 5, 5/2, -3, 0` : `-231/158 = -1,4620253164556962` | idem | `1e-12` | égalité à `1e-12` |
| V-PENTE-03 | droite exacte `y = 3x - 2` sur `x = 1/2, 3/2, 4, 9` : 3 ; `y` constant : 0 | calcul exact | `1e-12` | égalité à `1e-12` |
| V-PENTE-04 | invariance par permutation, par translation des `y` (`+1000`), homothétie des `y` (`×7` : `15,4`), et `x` autour de `10^8` : toujours `2,2` | propriétés des moindres carrés ; le cas `10^8` : `Σx² > 2^53`, une formule non centrée `nΣxy - ΣxΣy` n'y garde aucun chiffre, la doc fixe la formule centrée | `1e-12` (permutation), `1e-10` (translation, homothétie), `1e-9` (`10^8`) | égalité à la tolérance |
| V-PENTE-05 | longueurs différentes, un seul point, tous les `x` égaux : panique | doc de `pente` (« Panique ») | — | `catch_unwind` renvoie une erreur |

## Bitmap

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-BMP-01 | image neuve éteinte ; `(x, y)` n'est pas `(y, x)` ; hors de l'image : panique ; dimension 0 : panique | doc de `Bitmap` | — | égalité / panique |
| V-BMP-02 | `[(0,0),(2,1),(-1,3)]` → 4×4, centre `(1,0)` ; `[(3,4)]` → 4×5, centre `(0,0)` ; `[(-2,-3)]` → 3×4, centre `(2,3)` ; vide → 1×1, centre `(0,0)` | convention de `Bitmap::depuis_sites` (boîte englobante des sites et de l'origine ; le centre est le pixel de l'origine, c'est-à-dire du germe) ; calculs à la main | — | égalité exacte |

## boîtes : le comptage

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-BOITE-01 | image 7×5, pixels `(0,0)(1,0)(6,4)(3,2)(4,2)(6,0)(0,4)` : `N(c)` pour `c = 1, 2, 3, 4, 7, 8, 100` : `7, 6, 5, 4, 1, 1, 1` | comptage à la main de `(x // c, y // c)`, recompté par Python | — | égalité exacte |
| V-BOITE-02 | image 4×1, pixels `(1,0)(2,0)` : `N = 2, 2, 1, 1` pour `c = 1, 2, 3, 4` (le côté 2 chevauche la frontière : 2 boîtes) | comptage à la main | — | égalité exacte |
| V-BOITE-03 | pente de `ln N` contre `-ln c` pour `N = 7, 6, 5, 4`, `c = 1, 2, 3, 4` : `0,385410608714005728…` | Python `decimal` (60 chiffres), moindres carrés | `1e-12` | égalité à `1e-12` ; côtés dans le désordre : même valeur |
| V-BOITE-04 | segments (horizontal, vertical, diagonale) de 243 pixels : 1 ; carré plein 243² : 2 ; Vicsek niveau 5 (croix et X) : `ln 5 / ln 3 = 1,46497352071792717…` ; tapis niveau 5 : `ln 8 / ln 3 = 1,89278926071437231…` ; comptes `N(3^j)` : Vicsek `5^(5-j)`, tapis `8^(5-j)`, segment `243/3^j`, plein `(243/3^j)²` | **valeurs publiées** : la dimension de Hausdorff de la fractale de Vicsek est `log 5 / log 3 ≈ 1,46497` (construction : cinq carrés sur neuf, le centre et quatre voisins) et celle du tapis de Sierpinski `log 8 / log 3 ≈ 1,8928` (huit carrés sur neuf), Wikipedia « Vicsek fractal » et « Sierpiński carpet » lus le 8/11/2026 (la source originale de Vicsek n'a pas été lue). **Pourquoi elles sont exactes ici** (démonstration, pas une lecture) : pour un côté `3^j`, la boîte `(i, j)` est exactement un sous-carré de niveau `5 - j` ; la fractale de niveau 5 est l'union de 5 copies (8 pour le tapis) de niveau 4 réduites d'un tiers ; par récurrence `N(3^j) = 5^(5-j)`, et les points `(-ln 3^j, ln 5^(5-j))` sont alignés de pente `ln 5 / ln 3`. Constantes à 17 chiffres : Python `decimal` | `1e-12` | égalité aux comptes (exacts) puis à `1e-12` |
| V-BOITE-05 | Vicsek niveau 4 sur 81×81 avec `1, 3, 9, 27` : comptes `625, 125, 25, 5`, pente `ln 5 / ln 3` ; niveau 5 sur 324×324 (décalage 0) : mêmes comptes ; carré plein 64² avec `1, 2, 4, 8, 16` : 2 | idem V-BOITE-04 | `1e-12` | idem |
| V-BOITE-06 | Vicsek niveau 5 sur 324×324 décalé de `(27, 54)` : comptes `3125, 625, 125, 25, 10`, pente `1,33878757000363568` ; décalé de `(1, 0)` : `3125, 938, 188, 38, 8`, pente `1,37825327337435781` | comptage naïf `(x // c, y // c)` et `decimal` par Python (`valeurs_mesures.py`) ; **ce ne sont pas des dimensions** : le décalage fausse le comptage exprès, pour que seule une alignement sur le coin de l'IMAGE donne ces comptes | `1e-12` | égalité aux comptes puis à `1e-12` |

## masse-rayon

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-MASSE-01 | 7 sites `(0,0)(3,4)(4,4)(-3,-4)(5,0)(0,5)(6,0)` : centre `(0,0)`, rayons `0,1,2,4,5,6,7` → `1,1,1,1,5,7,7` ; centre `(3,4)`, rayons `0,1,4,5,10` → `1,2,3,6,7` ; centre `(-2,1)`, rayons `0,4,5,6,7,8` → `0,1,2,4,5,6` ; doublon compté deux fois ; coordonnées `i32::MIN/MAX` sans débordement (`r = 2^31` : 1 site ; `r = u32::MAX` : 3) | comptage à la main de `dx² + dy² <= r²`, recompté par Python ; le bord est dedans par la doc | — | égalité exacte |
| V-MASSE-02 | disque plein de rayon 100 (31 417 sites) : `N(r)`, `r = 0..10` : `1,5,13,29,49,81,113,149,197,253,317` ; `N(6,8,12,17,25,35,50,70) = 113,197,441,901,1961,3853,7845,15373` ; pente sur ces rayons `2,00613023065543171` | **valeur publiée** : la suite A000328 de l'OEIS, « Number of points of norm <= n^2 in square lattice », `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317, 377, 441, 529, 613` (lue le 8/11/2026 : elle recoupe `N(6) = 113`, `N(8) = 197`, `N(12) = 441`) ; les autres comptes : Python (double boucle) ; la pente : `decimal` sur ces comptes | comptes exacts ; pente `1e-12` ; écart à 2 : au plus `0,01` | égalité aux comptes ; `|pente - 2,0061302…| <= 1e-12` ; `|pente - 2| < 0,01` (la discrétisation du disque : `N(r) = πr² + O(r^(2/3))`) |
| V-MASSE-03 | le même disque translaté de `(37, -12)`, mesuré autour de son centre : mêmes comptes, même pente | invariance par translation | idem | idem |
| V-MASSE-04 | segment `(x, 0)`, `x = -100..100`, `r = 4, 8, 16, 32, 64` : `N = 9, 17, 33, 65, 129` (`2r + 1`), pente `0,96175094797399986` ; segment vertical : idem | `2r + 1` (comptage), `decimal` (pente) | `1e-12` ; écart à 1 : au plus `0,04` | égalité aux comptes puis à `1e-12` ; `|pente - 1| < 0,04` (la pente d'un segment discret est `2r/(2r+1)`, un peu sous 1 à ces petits rayons) |
| V-MASSE-05 | points `(ln r, ln N)` du disque de rayon 100 pour `r = 6, 8, 12` : `(1,79175946922805500, 4,72738781871234057)`, `(2,07944154167983593, 5,28320372873798851)`, `(2,48490664978800031, 6,08904487544684599)` ; l'ordre des rayons donné est conservé | Python `decimal` (`ln` de `6, 8, 12, 113, 197, 441`) | `1e-12` | égalité à `1e-12` |
| V-MASSE-06 | rayon nul, disque sans site, un seul rayon, rayons égaux : panique | doc | — | panique |

## rayon de giration

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-GIR-01 | un site : 0 ; `(0,0),(3,4)` : `2,5` ; deux sites confondus : 0 ; carré plein de côté 5 : `2` (exact) ; côté 2 : `sqrt(1/2)` ; côté 13 : `sqrt(28)` ; `(0,0),(1,0),(0,1)` : `2/3` ; `(10,0),(10,1),(11,0),(30,40)` : `sqrt(2943/8) = 19,1800677788166327` ; liste vide : panique | rationnels exacts (`rg2` en `Fraction`) puis racine `decimal` ; la formule fermée du carré `R_g² = (s² - 1)/6` : variance d'une loi uniforme sur `s` entiers, `(s² - 1)/12`, par coordonnée, deux coordonnées | exact là où le résultat est exact (`==`), `1e-14` à `1e-15` ailleurs | égalité |
| V-GIR-02 | `R_g² = (s² - 1)/6` pour `s = 1..40` ; translation de `(-1000, 2500)` : MÊME flottant au bit près ; carré 7 translaté : `sqrt(8)` ; `(-2^30,-2^30),(2^30,2^30)` : `1518500249,98802485` | formule fermée ci-dessus ; la translation : les sommes sont des entiers exacts (la doc le fixe), donc le même `num` ; l'extrême : `Fraction` exacte `2305843009213693952` puis racine `decimal` | `1e-12` relatif (`R_g²`), bit à bit (translation), `1e-5` absolu (extrême : `1,5e9` à `1e-15` relatif) | égalité |

## dla : la dynamique

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-DLA-01 | premiers sites (12) des graines 1, 2, 3 et `6848047025811403090`, `10997253461852598610` ; SHA-256 de l'écriture canonique (`demi_cote` 300, 200 sites) des graines 1, 2, 3, 7, 42, 1000 et (60 sites) des deux graines fabriquées | **le troisième code** `ref_dla.py` (la doc de `src/dla.rs`, section « La dynamique, à la lettre », sur le modèle PCG32 `ref_alea.py`), SHA-256 par `hashlib` sur les octets écrits par Python. Les deux graines fabriquées : `ref_alea.graine_pour_premiere_sortie(15, 54)` et `(16, 54)` : le premier tirage vaut 15 et 16 ; l'anneau `R = 5` a 40 points, `2^32 mod 40 = 16` : 15 est rejeté, 16 accepté | aucune | égalité exacte |
| V-DLA-05 | `Amas::sites()` et l'état du générateur égaux à ceux du simulateur naïf du test, pour 12 graines (1 à 12) à 30, 100 et 200 sites (le même amas grandi par étapes) et 3 graines (101 à 103) à 600 sites, avec deux `demi_cote` différents | **la doc seule** : le simulateur naïf est écrit dans le test d'après la section « La dynamique, à la lettre » (un anneau énuméré à chaque lancement, un ensemble de sites, `Alea::borne` appelé comme la doc le dit). Il est de la même main que la doc : c'est pourquoi V-DLA-01 le recoupe par un code Python | aucune | égalité exacte des sites et des 16 octets de l'état |
| V-DLA-02 | propriétés de 10 amas de 150 sites : germe en premier, sites distincts, chacun a un voisin par un côté parmi les précédents, dans le réseau, `contient`, `rayon`, `ajouter` rend le dernier | doc de `Amas` (« La dynamique », « Le rayon de l'amas ») ; propriétés vraies pour toute graine par construction du modèle | — | égalité |
| V-DLA-03 | `croitre_jusqu_a` sans effet si `n` est déjà atteint ; même graine même amas ; huit graines, huit empreintes ; `demi_cote` hors de la dynamique | doc de `Amas` | — | égalité |
| V-DLA-04 | `MARGE = 5`, `FACTEUR_ABANDON = 2` | convention de la doc, celle des premières simulations sur grille : « The launching radius is typically taken to be RL = RB + 5 », et l'abandon peut venir dès `RK = 2RB` (Y. L. Loh, arXiv:1407.2586, 2014, section I, texte intégral lu par le juge le 8/11/2026 ; il cite Witten et Sander 1981 et 1983 pour les deux) ; le simulateur naïf et Python les emploient en dur. Le juge a mesuré que ce choix ne change pas les mesures statistiques à l'erreur-type près (V-STAT-02) | — | égalité |
| V-ANN-01 | `anneau(0) = [(0,0)]`, `anneau(1)` (8 points), `anneau(2)` (16 points), longueurs `R = 0..15` : `1,8,16,20,24,40,36,48,56,56,68,64,80,92,88,96` | énumération à la main (`R² <= d² < (R+1)²`), longueurs comptées par Python (`ref_dla.anneau`) | — | égalité exacte |
| V-ANN-02 | `anneau(R)` égale la double boucle naïve pour `R = 0..300, 777, 1000, 2047, 2900` | l'énumération naïve écrite dans le test d'après la doc | — | même liste, même ordre |
| V-CAP-01 | réseau `demi_cote = 3` : `essayer_ajouter` finit par `None`, au plus 49 sites, tous dans `|x|,|y| <= 3` ; sur 20 graines, chacun des quatre bords (`x = 3`, `x = -3`, `y = 3`, `y = -3`) est atteint par au moins 3 amas ; `demi_cote = 0` : `None` d'emblée | doc (« La capacité du réseau ») ; le seuil de 3 sur 20 par bord : **mesuré** sur les deux prototypes (identiques, la dynamique étant fixée) : 9, 12, 7 et 7 amas sur 20 ; un bord exclu par erreur n'en donnerait aucun | seuil 3/20 par bord (mesuré 7 à 12) | égalité / majorants |
| V-CAP-02 | 60 sites dans un réseau de 49 cases : panique contenant « capacité » | principe des tiroirs (49 cases) | — | `should_panic` |
| V-CAP-03 | `ajouter` à `demi_cote = 0` : panique « capacité » ; `Amas::new(1, 2048)` : panique ; `DEMI_COTE_MAX = 2047` | doc | — | panique |
| V-CAN-01 | germe seul (`demi_cote` 5, graine 7) : `47495652453030310500000001000000 0000000000000000 d1a0bdb386a422056d00000000000000`, SHA-256 `3ff6c507…5238` ; quatre sites de la graine 1 (`demi_cote` 9) : 64 octets, état `91fe57e8fe428c356d00000000000000`, SHA-256 `70b0b2a7…d7af` | octets écrits un à un d'après la doc ; l'état du générateur vient du modèle PCG de l'étape 0 (`Alea::new(7)` : `d1a0bdb386a42205`, repris du test de l'étape 0 ; état après quatre particules de la graine 1 : `ref_dla.py`) ; SHA-256 `hashlib` | aucune | égalité exacte |
| V-DIR-01 | `particules` : `Ok(None)`, `Ok(Some(3000))`, `dla 05` → 5, bornes 1 et `1000000`, refus de `0`, `1000001`, sans argument, deux arguments, `+5`, `-5`, `5.0`, `x`, 20 chiffres ; seconde `dla` fautive ; première fautive | doc de `particules` (la forme est celle de `Directive::entier`, étape 0) ; `PARTICULES_MAX = 1 000 000` : convention (un million de sites, au plus la mémoire d'une borne de 4096) | — | égalité |
| V-DIR-02 | `demi_cote_de_la_boite` : `(401,401) → 200`, `(401,300) → 149`, `(400,400) → 199`, `(2,9) → 0`, `(1,1) → 0`, `(4096,4096) → 2047` | convention `(min - 1) / 2` : le germe est le centre de la plus grande boîte carrée centrée de côté impair qui tient dans la boîte | — | égalité |
| V-COL-01 | `couleur_du_rang` : 5 sites → la palette ; 9 sites → 255,255,255 / 203,243,255 / 150,230,255 / 105,185,255 / 60,140,255 / 100,110,248 / 140,80,240 / 198,85,200 / 255,90,160 ; 3 sites ; 2 sites ; 1 site | la formule de la doc, recalculée par Python (`couleur`) et à la main pour le rang 1 de 9 : `(255·4 + 150·4 + 4)/8 = 203` ; la palette est une convention (couleurs choisies pour se détacher du fond) | — | égalité exacte |
| V-COL-02 | `amas_colore` sur `(0,0)(0,-1)(0,1)(-1,1)(-1,2)` : 5×5, pixels `(2,2),(2,1),(2,3),(1,3),(1,4)` ; échelle 3 (15×15, 45 pixels colorés), 8 (40×40) ; SHA-256 des octets RVB `53586e11…1803`, `41ca91e2…4573`, `2407fb44…f7d0` ; germe seul : 512×512 blanc | image écrite à la main (cinq pixels) ; SHA-256 des images de `ref_dla.py::image` (la doc de `amas_colore`) par `hashlib` | aucune | égalité exacte |

## lancer et le binaire

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-LDLA-01 | `boite 41 41` + `dla 40`, graine 3 : empreinte `ae05f58a…43ee`, CSV `2,1,0.500000 / 4,3,1.118034 / 8,5,1.727534 / 16,5,2.492959 / 32,8,3.737218 / 40,9,4.295856`, PNG 510×510, SHA-256 des octets RVB `b66c262c…3ff5` | `valeurs_dla.py` : l'amas de `ref_dla.py`, ses rayons entiers, sa giration en `float` (sommes entières exactes, une division, `math.sqrt`) formatée `%.6f` (la même chose que `{:.6}` : l'arrondi exact du flottant) ; l'image : `image()` de ce script | aucune | égalité exacte |
| V-LDLA-02 | `scenarios/dla.txt`, graine 1 : empreinte `c1d90557…0ee6`, CSV de 12 lignes (de `2,1,0.500000` à `3000,110,50.238930`), PNG 434×434, SHA-256 RVB `2fc30fdb…5711` | `valeurs_dla.py grand` : le troisième code sur 3000 sites (`valeurs_dla_grand.txt`) | aucune | égalité exacte. **Le prototype Rust a donné les mêmes valeurs avant la comparaison** : l'accord est celui de deux codes écrits séparément |
| V-LDLA-03 | CSV, empreinte et aperçu pour `n = 1, 2, 3, 4, 5, 7, 8, 9, 16, 17, 64, 100` et 3 graines : reconstruits dans le test depuis les sites d'un `Amas`, par la doc de `lancer` (l'en-tête, les puissances de 2, la ligne finale sans doublon, `{:.6}`) | doc de `lancer` ; l'`Amas` est jugé par ailleurs (V-DLA-01) | — | égalité exacte |
| V-LDLA-04 | mêmes entrées mêmes sorties ; deux graines, deux empreintes, deux CSV, deux PNG ; `dla 60` et `dla 61` ; `dla` et le scénario vide | déterminisme de la doc | — | égalité / inégalité |
| V-LDLA-05 | les refus : ligne de la plus petite directive refusée (inconnue, `dla` fautive, seconde `dla`, `dla` sur un `hexagone`, amas hors de la boîte) ; `boite 1 1` + `dla 1` : le germe seul | doc de `lancer` ; le débordement de `boite 5 5` + `dla 100` : principe des tiroirs (25 cases) | — | égalité |
| V-LDLA-06 | le binaire sur `scenarios/dla.txt` : code 0, trois fichiers, octet pour octet ceux de `lancer` | doc de `main.rs` (étape 0, inchangée) | — | égalité |
| V-LDLA-07 | `dla 0` et un débordement : code 1, `ligne 2`, pas de `panicked`, rien d'écrit | doc de `main.rs` (une erreur de `lancer` : « directive inconnue : ligne n ») | — | égalité |
| V-LDLA-08 | `references/dla-graine-1.empreinte` = l'empreinte de `scenarios/dla.txt`, graine 1, puis `\n` | **régression** : produite par NOTRE code (et déjà égale à V-LDLA-02) | — | égalité |

## Les propriétés statistiques

La dynamique étant fixée par la doc, toute implémentation fidèle donne les mêmes nombres : ces bandes ne mesurent pas un écart entre
implémentations ; elles disent ce que le modèle donne à 4000 sites, sur deux séries de 20 graines (1 à 20, 101 à 120).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-STAT-01 | exposant moyen du rayon de giration `β` (`N = 125 … 4000`) sur 20 graines : dans `[0,572 ; 0,598]`, soit `1/β` dans `[1,672 ; 1,748]` | **publié** : Meakin, Phys. Rev. A 27, 604 (1983), résumé lu sur le site de l'APS : en dimension 2, `D/d = 0,847 ± 0,01` (`D = 1/β = 1,694 ± 0,02`) ; Tolman et Meakin, Phys. Rev. A 40, 428 (1989), résumé lu : `D = 1,715 ± 0,004` (hors réseau, jusqu'à `10^6` particules) ; Y. L. Loh, arXiv:1407.2586 (2014), texte intégral lu : les petits amas circulaires sur grille carrée ont `β = 0,585 = 1/1,71`. **Largeur mesurée** : écrivain, graines 1 à 30 : moyenne `0,5850`, écart-type `0,0196`, min `0,5489`, max `0,6185`, bande `± 3 σ / √20` ; juge, 200 autres graines (201 à 400) : moyenne `0,5848`, écart-type `0,0203`, dix moyennes de 20 graines de `0,5747` à `0,5906` | bande à `3 σ / √20` (mesurée, jamais élargie) ; elle contient les trois valeurs publiées | moyenne de chaque série dans la bande (mesuré : `0,5880` et `0,5861`) |
| V-STAT-02 | dimension de masse-rayon moyenne `D_mr` (rayons 6 à `rayon / 2`, depuis le germe) sur 20 graines : dans `[1,642 ; 1,699]` | **mesuré sur NOTRE modèle** (test « régression ») : écrivain, graines 1 à 30 : moyenne `1,6707`, écart-type `0,0424`, min `1,6015`, max `1,7853` ; juge, graines 201 à 400 : `1,6686`, `0,0480`, dix moyennes de 20 graines de `1,6591` à `1,6842`. **Écart à la valeur publiée** (1,71 : Tolman et Meakin 1989 ; Sander, Contemp. Phys. 41, 203, 2000, texte intégral lu : la masse-rayon « fits very well to D2 = 1.71 ») : 0,04 en dessous. **Expliqué par le juge, sans source qui donne la valeur à cette taille** : avec un simulateur indépendant, `D_mr` vaut `1,663 ± 0,007` (4000 sites, 40 graines), `1,682 ± 0,006` (10 000, 30), `1,689 ± 0,006` (30 000, 24) : elle monte vers 1,71 avec la taille ; elle est la même, à l'erreur-type près, avec un abandon à `10 R` (`1,661`), une marge de 20 (`1,670`) ou le retour exact du marcheur par le noyau de Poisson, sans abandon (`1,667` ; `1,674` et `1,685` à 10 000 et 30 000 sites) : un effet de taille finie, pas une faute des règles | bande à `3 σ / √20` (mesurée, jamais élargie) | moyenne de chaque série dans la bande (mesuré : `1,6625` et `1,6796`) |
| V-STAT-03 (non testé) | dimension de comptage de boîtes de l'amas (côtés 2 à 32) : `1,490` à 4000 sites, `1,496` à 10 000 et à 30 000 (juge) ; `1,41` à `1,55` selon la plage de côtés (écrivain) | **aucune source** pour cette valeur (Sander 2000 : la dimension de boîtes de la DLA est « close to 1.7 » ; Grebenkov et Beliaev, arXiv:1708.01514, 2017 : `1,666 ± 0,004` sur des amas de `4·10^7` particules, boîtes de `2^2` à `2^13`) ; elle ne monte pas avec la taille sur cette plage de côtés : c'est une propriété de l'estimateur | — | **rapportée, pas testée** : le juge a retiré l'assertion (une bande tirée de notre code, sans source, n'est pas un test « juste ») ; `dimension_boites` est jugée sur des images exactes (V-BOITE-01 à 06) |

## Ce qui a été lu, et ce qui ne l'a pas été (relu par le juge le 8/11/2026)

- **Witten et Sander, Phys. Rev. Lett. 47, 1400 (1981)** : seul le **résumé** a été lu (page de l'APS) : « Density correlations within the
  model aggregates fall off with distance with a fractional power law » ; le rayon de giration « has power-law behavior ». Le texte intégral
  (le tableau des exposants, la taille des amas) n'a pu être ouvert ni par l'écrivain ni par le juge (article payant ; les copies trouvées
  sont hors d'accès). Les nombres que des moteurs de recherche attribuent à l'article (un exposant de corrélation proche de 0,34, soit
  `D = 2 - A ≈ 1,66`, des amas de 3600 particules) sont **non vérifiés** et n'entrent dans aucun test ni aucune doc du jeu.
  **L'affirmation du plan, « l'article de 1981 donne une valeur un peu plus basse que 1,71 », reste non vérifiée** : rien du jeu ne s'y appuie.
  Ce qu'un auteur en dit, lu : T. A. Witten, « Citation Classic », Current Contents, 3 mai 1993 (texte intégral) : un amas dont la dimension
  fractale « is about 1.7 in two dimensions », le marcheur allant sur un réseau.
- **Witten et Sander, Phys. Rev. B 27, 5686 (1983)** : résumé seul ; aucun nombre ; la dimension ne dépend pas des détails à courte portée.
- **Meakin, Phys. Rev. A 27, 604 (1983)** : résumé lu (V-STAT-01).
- **Tolman et Meakin, Phys. Rev. A 40, 428 (1989)** : résumé lu : `1,715 ± 0,004` en 2D hors réseau ; l'anisotropie du réseau pour `d <= 4`.
- **Ball et Witten, Phys. Rev. A 29, 2966 (1984)** : résumé lu : une borne inférieure `D >= d - 1` ; n'entre dans aucun test.
- **Sander, Contemp. Phys. 41, 203 (2000)** et **Loh, arXiv:1407.2586 (2014)** : textes intégraux lus (V-STAT-01, V-STAT-02, V-DLA-04) ;
  Loh : sur grille carrée l'amas devient anisotrope au-delà d'environ `10^5` sites et `β` y dérive vers 2/3 (borne de Kesten), hors de la
  taille testée ici.
- **Somfai, Ball, Bowler et Sander, arXiv:cond-mat/0210637 (2003)** : texte intégral ; corrections de taille hors réseau
  `R = R̂ N^(1/D)(1 + R̃ N^(-ν))`, `D = 1,711` ; **non employé dans un test** (hors réseau).
- **Vicsek** (1983 et 1992) : la source d'origine de la fractale n'a pas été lue ; la dimension citée vient de Wikipedia et de la
  démonstration de V-BOITE-04.

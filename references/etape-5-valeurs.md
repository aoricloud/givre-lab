# Étape 5, la vitrine : les valeurs attendues

Cette fiche donne, pour chaque valeur que les tests de l'étape 5 attendent, sa source, sa tolérance et son critère de réussite.
Elle est écrite avec les critères **avant** les calculs (la première section) ; les valeurs viennent après.

## Les critères (fixés avant tout calcul)

1. **La vitrine ne calcule rien de neuf.** Elle pilote `dla::Amas` et `flocon::Flocon`, déjà éprouvés par les étapes 1 et 3.
   Tout ce qu'elle expose (temps, durée, image, compteurs, compacité, mots de l'empreinte) est donc comparé à l'état
   **natif** que la bibliothèque donne pour le même modèle, la même graine et le même temps (`tests/etape_5_commun`,
   fonction `natif`, qui n'appelle jamais `vitrine`). Le critère est une **égalité exacte** : entiers, octets, mots de
   32 bits, bits des `f64` (`to_bits`). **Aucune tolérance, aucune statistique** : il n'y a pas de hasard à mesurer, et une
   graine donne toujours le même état.
2. **Des valeurs de source indépendante du code natif**, pour qu'un défaut commun à la vitrine et au natif (une image mal
   rangée, des octets dans le mauvais ordre) se voie : des calculs exacts faits **d'après les docs** de `dla::amas_colore` et
   de `Flocon::apercu`, en Python (`valeurs_etape5.py`, entiers et fractions, rien du code Rust) ; des bornes de calcul fermé ;
   les fichiers de référence du dépôt.
3. **Les résultats figés de notre code** (`references/dla-graine-1.empreinte`, `references/reiter-plaque.empreinte`) ne sont
   pas des valeurs d'un autre outil : ce sont des régressions (étiquette `régression`) ; ils sont recoupés par le calcul natif
   dans le même test, et ne prouvent que la stabilité.
4. **Les seuls nombres mesurés** sont des plafonds de taille (le module sous 1 Mio, la page sous 200 Ko hors module) : la
   valeur observée du prototype est dans le rapport ; le plafond est celui de la feuille de route et du brief, pas une
   tolérance réglée sur le prototype.
5. **La page n'a pas de valeur attendue** : ses tests sont de la forme (ce que dit le fichier), jamais une preuve de
   comportement. Voir « Ce que cela n'établit pas ».

## Les valeurs

| Id | Valeur attendue | Source | Tolérance | Critère de réussite |
|---|---|---|---|---|
| V-VIT-01 | La table des modèles : 0 = DLA (`boite 401 401`, `dla 3000`, durée 2999, `demi_cote` 200) ; 1 = plaque (`hexagone 30`, `reiter 1 0 0.05 400`) ; 2 = étoile (`hexagone 40`, `reiter 1 0.45 0.1 120`) ; 3 = dendrite (`hexagone 60`, `reiter 1 0.4 0.001 800`) | Les fichiers `scenarios/dla.txt`, `reiter-plaque.txt`, `reiter-etoile.txt`, `reiter-dendrite.txt`, lus ; `demi_cote_de_la_boite(401, 401) = (401 - 1) / 2 = 200` (doc de `dla`) ; FEUILLE-DE-ROUTE.md, étape 5 : « la page montre deux des modèles fusionnés » avec les paramètres des scénarios d'exemple | aucune (texte et entiers) | `lire_scenario` des quatre fichiers redonne exactement ces formes, arguments et nombres de pas |
| V-VIT-02 | DLA de graine 1 jouée jusqu'au bout (2999 particules, 3000 sites) : empreinte `c1d905579070373fc8f718e8f75fb8ff1f7824e89331f292517900ceb3e00ee6` | `references/dla-graine-1.empreinte` (régression de notre code, figée par l'étape 1) ET `Amas::new(1, 200)` à 3000 sites, calculé dans le test | aucune | les huit mots lus par la vitrine, mis bout à bout, égalent cette chaîne et celle du natif |
| V-VIT-03 | Plaque jouée jusqu'au bout (400 pas) : empreinte `4ecf6083efb496b4e3f2031c4ca347773199653c8cebc821eaea3eabbae675f1` | `references/reiter-plaque.empreinte` (régression, étape 3) ET `Flocon::new(30, (1, 0, 0.05))` à 400 pas | aucune | idem |
| V-VIT-04 | DLA au temps 0 : image de 512 × 512 pixels, **tous** `0xFFFFFF`, 1 site, rayon 0 | Doc de `amas_colore` : le germe seul donne `m = 0`, `cote = 2m + 1 = 1`, `echelle = max(1, 512 / 1) = 512`, `largeur = 512` ; une seule couleur, `PALETTE[0]` (blanc) ; `couleur_du_rang(0, 1)` = `PALETTE[0]`. Vérifié par `valeurs_etape5.py` (`amas_colore([(0, 0)], 512)`) | aucune | `{0xFFFFFF : 262144}` exactement |
| V-VIT-05 | DLA après **une** particule (elle colle à un voisin du germe : 4 sites possibles selon la graine) : image de 510 × 510 ; 28 900 pixels `0xFFFFFF` (le germe), 28 900 pixels `0xFF5AA0` (la particule), 202 300 pixels `0x0D1117` (le fond) ; 2 sites, rayon 1, temps 1 | Doc de `amas_colore` : `m = 1`, `cote = 3`, `echelle = 512 / 3 = 170`, `largeur = 510` ; blocs de 170 × 170 = 28 900 pixels ; `510² = 260 100` ; fond `FOND = (13, 17, 23)` = `0x0D1117` ; `couleur_du_rang(1, 2)` : `d = 1`, `s = 4`, `k = min(4, 3) = 3`, `f = 1`, canal `(a * 0 + b * 1 + 0) / 1 = b`, donc `PALETTE[4] = (255, 90, 160)` = `0xFF5AA0` ; rayon `ceil(sqrt(1)) = 1`. Vérifié par `valeurs_etape5.py` pour les quatre voisins | aucune | les trois nombres exacts, pour quatre graines |
| V-VIT-06 | Flocons au temps 0 : le germe est le pixel `(R, R)`, `0xEBF5FF` ; le fond (les cases à `β`) est `0x000000` (plaque, `β = 0`), `0x003972` (étoile, `β = 0.45`, `brut = 1932735283`, `niveau = 114`), `0x003265` (dendrite, `β = 0.4`, `brut = 1717986918`, `niveau = 101`) ; le pixel `(0, R)` est la case `(-R, 0)` de couleur `β` ; le pixel `(0, 0)` est noir (aucune case) ; nombre de pixels : plaque 3720 noirs + 1, étoile 1640 noirs + 4920 de fond + 1, dendrite 3660 noirs + 10 980 de fond + 1 | Doc de `Flocon::apercu` : germe gelé `(235, 245, 255)` ; sinon `niveau = (brut * 255) >> 32`, `(0, niveau / 2, niveau)` ; la case `(q, r)` est le pixel `(q + R + floor(r / 2), r + R)` ; coins `(0, 0, 0)` ; doc de `lire_decimal` : `brut = floor(N * 2^32 / 10^k + 1/2)`. Calculé par `valeurs_etape5.py` (Fractions, entiers). Cases : `3R(R + 1) + 1` = 2791, 4921, 10 981 (doc de `reseau`/`flocon`) | aucune | `{couleur : nombre}` exactement égal ; pour `β = 0.4`, `0.4 * 255 = 102` mais `brut` est arrondi sous `0.4 * 2^32`, donc `niveau = 101` : le test voit un calcul fait en flottants |
| V-VIT-07 | Après 2999 particules, le rayon de l'amas est au moins 27 | Calcul fermé : les 3000 sites sont distincts et dans le disque de rayon `rayon` (doc de `Amas::rayon`), qui contient au plus `(2 rayon + 1)²` points entiers ; `(2 rayon + 1)² >= 3000` donne `rayon >= 26,9` | inégalité stricte, sans réglage | `givre_rayon() >= 27` |
| V-VIT-08 | La compacité d'un flocon vaut exactement 1 au temps 0 (une case gelée sur `3 * 0 * 1 + 1 = 1` case) et, pour la plaque, après 400 pas (hexagone plein) | Doc de `Flocon::compacite` (« elle vaut 1 pour un hexagone plein (une plaque) ») ; calcul fermé de l'étape 3 (`etape_3_exact` : compacité de la plaque exactement 1, 0 pointe) ; au temps 0 : `Fixe::rapport(1, 1) = UN`, `UN.vers_f64() = 1.0` | aucune (bits) | `givre_compacite().to_bits() == 1.0_f64.to_bits()` |
| V-VIT-09 | `u32::MAX` n'est jamais une couleur empaquetée `0x00RRGGBB` | Calcul : le plus grand `0x00RRGGBB` est `0x00FFFFFF < 0xFFFFFFFF` | aucune | `givre_pixel` hors de l'image vaut `u32::MAX`, et toute couleur lue dans l'image est `<= 0xFFFFFF` |
| V-VIT-10 | Le module : 0 importation ; exporte exactement `memory` et les onze fonctions `givre_*` de la liste, avec leurs signatures ; moins de 1 Mio | Spécification WebAssembly (sections de types, d'importations, de fonctions et d'exportations ; la mémoire est l'exportation de genre 2) ; plafond de la feuille de route et du brief | plafond 1 048 576 octets | lecture du module par le lecteur de binaire de `etape_5_commun` |
| V-VIT-11 | La page : moins de 200 Ko (204 800 octets) hors module ; aucune adresse extérieure | Plafond du brief ; Mycelium Lab, étape 4 (mêmes motifs d'adresses) | plafond 204 800 octets | somme des tailles des fichiers de `docs/` hors `givre.wasm` |
| V-VIT-12 | `docs/lecture.js` (ajout du juge) : `lireEmpreinte(m)` = l'empreinte native (64 caractères) ; `lireImage(m)` = les dimensions natives et un `Uint8ClampedArray` dont le SHA-256 est celui des octets natifs R, V, B suivis de 255, pixel par pixel ; `lireGraine` rend `0, 1, 42, 7, 4294967295` pour `"0", "1", " 42 ", "007", "4294967295"` et `null` pour `"4294967296", "-1", "", " ", "abc", "1.5", "1e3", "0x10", "+7"` | Le code natif (`etape_5_commun::natif`), comme V-VIT-02 ; pour la graine, un calcul exact d'après la règle écrite en tête de `tests/etape_5_page.rs` (`trim()`, chiffres ASCII seulement, au plus `2^32 - 1`) ; le format RGBA est celui que demande `ImageData` (MDN : `ImageData(dataArray, width)`, un `Uint8ClampedArray` de `4 × largeur × hauteur` octets) | aucune | sortie de `tests/etape_5/lecture-wasm.mjs` égale, ligne pour ligne, à celle calculée par le test ; cinq cas (DLA 1 au bout, grande graine à 77, étoile à 37, dendrite au bout, plaque au temps 0) |

## Ce que les sources disent (lu pour cette étape)

- **MDN, `WebAssembly.instantiate()`** (developer.mozilla.org, JavaScript_interface/instantiate_static) : la forme à deux
  arguments, `instantiate(bufferSource, importObject)`, rend une promesse d'un objet `{ module, instance }` ; `importObject` est
  facultatif ; chaque importation déclarée par le module exige une propriété de l'objet, sinon `LinkError`. Le cas « aucune
  importation » n'est pas décrit explicitement sur la page : on passe `{}` (c'est ce que font les scripts et la page).
- **WebAssembly JavaScript Interface, `ToJSValue`** (webassembly.github.io/spec/js-api, lu à la main sur la seconde moitié de la
  page) : un `i32` rendu par une fonction exportée devient un **Nombre signé** (« Let i32 be signed_32(u32) »), d'où le `>>> 0` des
  mots de l'empreinte et des couleurs ; un `f64` NaN devient `NaN`, avec une note qui dit que des charges utiles de NaN peuvent
  rester observables : les scripts écrivent donc `NaN` en toutes lettres et n'écrivent les 8 octets que pour un nombre.
- **La référence de Rust, fonctions** (doc.rust-lang.org/reference/items/functions.html, « Unwinding ») : avec `panic=unwind`, une
  panique qui atteint un ABI non déroulant comme `extern "C"` devient un abandon (abort) ; d'où l'absence de toute panique
  possible dans les onze fonctions. **La référence de Rust, ABI** (doc.rust-lang.org/reference/abi.html, `no_mangle`) :
  `#[unsafe(no_mangle)]` (obligatoirement `unsafe` à partir de l'édition 2024) donne au symbole le nom de l'élément et l'exporte.
- **La spécification WebAssembly noyau, format binaire** (relue par le juge : webassembly.github.io/spec/core/binary/modules.html
  et types.html ; syntax/modules.html) : en-tête `\0asm` puis la version `01 00 00 00` ; sections numérotées (0 personnalisée,
  1 types, 2 importations, 3 fonctions, 7 exportations, 10 code, 11 données) ; un type de fonction est `0x60`, le vecteur des
  paramètres, le vecteur des résultats ; `i32` = `0x7F`, `f64` = `0x7C` ; une exportation est un nom et un index externe dont
  l'octet dit le genre (0 fonction, 1 table, 2 mémoire, 3 globale) ; « chaque exportation est désignée par un nom unique » ;
  les indices des importations précèdent ceux des définitions dans le même espace (d'où l'exigence « aucune importation »
  avant d'indexer la section des fonctions par l'indice exporté). Le lecteur de `etape_5_commun` dit la même chose.
- **WebAssembly JavaScript Interface, `ToJSValue` et `ToWebAssemblyValue`** (relus par le juge) : un `i64` devient un `BigInt`
  (un module qui rendrait un `u64` ferait échouer `>>> 0`) ; un argument JavaScript devient un `i32` par `ToInt32` (une graine
  `4294967295` passe donc en `u32::MAX`).
- **Node, « A note on process I/O »** (nodejs.org/api/process.html, relu par le juge) : l'écriture sur un tube est
  **asynchrone sous POSIX, synchrone sous Windows** ; `process.exit()` peut tronquer une écriture en cours ; `process.exitCode`
  laisse le processus finir : c'est ce que font les trois scripts.
- **Non lu** : un exemple officiel de `cdylib` pour `wasm32-unknown-unknown` (la commande `cargo rustc … --crate-type cdylib`
  est éprouvée : le juge l'a rejouée, voir son rapport).

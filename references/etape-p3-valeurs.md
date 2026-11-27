# Étape P3 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape P3 (la pointe de la dendrite de P2 selon l'anisotropie `ε` : le contour
`φ = 0`, le rayon par une parabole, la vitesse, la constante de sélection `σ*`) attend, d'où elle vient, avec quelle tolérance et quel
critère. Chaque identifiant (`V-P3-…`) est cité dans la doc des tests (`tests/etape_p3_*.rs`). **Aucune valeur « juste » ne vient du code
qu'elle juge.** Les tests `régression` (un résultat de notre code, figé) sont dits tels : le sens de variation en temps court
(`V-P3-SENS-01`) et la dendrite pleine à la borne (`V-P3-STAB-01`).

**Aucune valeur publiée n'est une valeur de cette fiche** : aucune source ouverte ne donne de vitesse, de rayon ou de `σ*` chiffrés en 2D
selon `ε` (voir « Ce qui a été lu »). Le sens de variation est celui que Karma et Rappel lisent en 3D (PRL 77, 4050, 1996, table I) ; la théorie 2D n'a pas été lue ; nos chiffres sont des repères de NOTRE modèle
(`MODELE.md`, « Étape P3 »).

**Outils indépendants** : `cercle.py` (recopié en entier à la fin de cette fiche) (Python 3.13, `decimal` à 60 chiffres : la
parabole des moindres carrés et la règle des lignes appliquées à un cercle) ; la mesure naïve de `tests/etape_p3_commun/mod.rs` (écrite
d'après la doc de `src/pointe.rs`, sur les champs, sans rien tirer du module sauf ses types) ; un second prototype Rust écrit d'après la
seule doc du contrat (itérateurs, `windows`, `fold`, `map_while`), qui donne les bits du premier sur tous les tests.

## Les critères, écrits avant les mesures

- La mesure (contour, règle des lignes, parabole, vitesse, `σ*`) est fixée au bit par la doc : un calcul exact (rationnels dyadiques) ou
  hors de Rust là où on peut, l'égalité au bit avec la mesure naïve sur des états réels partout ailleurs.
- Le sens : à `Δ`, `d0`, `D`, graine, domaine et temps fixés, quand `ε` monte, la pointe va plus vite, son rayon baisse, `σ*` monte ; à
  deux pas de grille ; l'écart entre deux `ε` voisins doit dépasser nettement l'écart entre les deux grilles (le seuil, lui, est mesuré :
  voir `V-P3-SENS-01`). Le régime est dit tel qu'il est (ici : non établi).
- La stabilité : aucune divergence à `dt = dt_borne` sur toute la plage de `ε` ; sinon, arrêt (ce serait une faute de P2).

## Ce qui a été lu, et ce qui ne l'a pas été

- **Lu (le juge, PDF d'arXiv patt-sol/9607005 v1, 29 juillet 1996, texte extrait par `pdftotext`)** : A. Karma et W.-J. Rappel,
  « Numerical Simulation of Three-Dimensional Dendritic Growth », Phys. Rev. Lett. 77, 4050 (1996). **En 3D**, `Δ = 0,45`, réseau
  200 × 200 × 400, pas de grille 0,6 à 0,8 (`W0 = 1`), `D` de 0,5 à 4, `dt` de 0,01 à 0,08 ; `ε4` de 0,0066 à 0,047. Table I (page 2),
  colonnes `ε4`, `P`, `σ*` (simulations), puis `P`, `σ*` (solvabilité numérique) et `P_Iv`, `σ*` (linéaire) :
  0,0066 : 0,426 ; 0,015 — 0,0123 : 0,360 ; 0,036 — 0,0171 : 0,312 ; 0,063 — 0,0265 : 0,236 ; 0,16 — 0,0369 : 0,159 ; 0,47 —
  0,0470 : 0,093 ; 1,72. **`σ*` croît et le Péclet `P = ρV/(2D)` décroît quand `ε` monte** ; la figure 2 trace `σ*` contre `ε4` à
  `Δ = 0,45`. Ce sont des valeurs 3D : aucune n'est comparée à notre 2D. Le texte (page 2) : les valeurs de la table sont indépendantes
  de `d0/W0` « within an accuracy of 10% ».
- **Lu (le juge, PDF d'arXiv cond-mat/9710167)** : N. Provatas, N. Goldenfeld, J. Dantzig, « Efficient Computation of Dendritic
  Microstructures using Adaptive Mesh Refinement » (PRL 80, 3308, 1998). En 2D, `ε = 0,05` seulement (`Δ` = 0,45 ; 0,55 ; 0,65 ; 0,7) :
  la vitesse de pointe rejoint la solvabilité (figure 3, des courbes) ; **aucune valeur selon `ε`**.
- **Lu par P2, non relu par le juge de P3** : Y.-T. Kim, N. Provatas, N. Goldenfeld, J. Dantzig, arXiv cond-mat/9810189 (le modèle, la
  table II : `Δ = 0,65`, `d0 = 0,5`, `D = 1`, `ε = 0,05`, `Ṽ = 0,0469`). P3 n'en tire que les réglages de la graine.
- **Non ouvert** : A. Karma et W.-J. Rappel, Phys. Rev. E 57, 4323 (1998) (le 2D chiffré, s'il y est) : pas sur arXiv sous ce titre ni
  sous un autre trouvé ; une copie sur un site de partage de documents existe, elle n'a pas été employée. Les théories de la solvabilité
  2D (Kessler et Levine, Ben Amar et Pomeau, Barbieri et Langer) : non ouvertes ; aucun `σ*(ε)` 2D n'est chiffré ici, et le sens
  croissant n'est lu à la source qu'en 3D (la table I de Karma et Rappel).

## V-P3-CONTOUR-01 : le contour à la main

Champs faits à la main (`champ_par_lignes` : la ligne `i` vaut `clamp((x_i - x_j) / 2, -1, 1)`), au pas 0,5 : tout est dyadique, donc
l'interpolation de la doc rend `x_i` exactement. La parabole `x = 10 - y² / 8` aux centres `y = (i + 0,5) / 2` : lignes 0 à 17 exactes,
18 à 23 sans croisement ; ligne 0 = `position_pointe` = 9,9921875 ; un front droit (7,25) ; les inégalités (`0,75`, `0,75 + 0,5 · (0,5 /
0,75)`, le premier croisement 0,5, `None` pour un zéro à gauche) ; `DomaineEntier`. L'ordre des opérations : 300 champs tirés, bits de
`contour_naif` ; l'interpolation par la droite diffère sur au moins 30 lignes. Tolérance : aucune.

## V-P3-PARAB-01, V-P3-PARAB-02 : la parabole à la main

Huit points dyadiques de `x = 10 - y² / 8` : `(10, 4)` exactement (chaque somme, chaque moyenne sur 8, chaque produit est exact) ;
`x = -2 - y²` sur `y = 0..3` : `(-2 ; 0,5)` exactement ; trois points : `1e-12`. Ordre des opérations : 300 jeux, bits de `parabole_naive`
; les sommes brutes diffèrent sur au moins 30 jeux. Refus : la doc. Tolérance : aucune, ou `1e-12` (division par 3).

## V-P3-RAYON-01 à 04 : la règle des lignes

Profondeur de la ligne `i` sur la parabole exacte : `(i² + i) / 32` (calcul exact : `x0 - x_i = (y_i² - y_0²) / 8`) ; à 1,75 : 8 lignes,
`(10, 4)` exactement ; 1,7 : 7 ; 2,25 : 9 ; 100 : 18 ; au pas 0,25, profondeur 1 : `(i² + i) / 128 <= 1`, 11 lignes. Le cercle de rayon
8 (V-P3-RAYON-02), par `cercle.py` (décimal à 60 chiffres) :

| profondeur | lignes | sommet | rayon |
|---|---|---|---|
| 1 | 8 | 8,005 963 261 271 136 | 7,555 419 403 737 679 |
| 2 | 11 | 8,025 548 311 556 944 | 7,100 258 720 076 413 |
| 4 | 14 | 8,090 097 127 426 222 | 6,370 238 610 025 476 |

Tolérance : `1e-10` en relatif (la racine et les rampes arrondissent ; observé : moins de `1e-14`). Le rayon ajusté d'un cercle n'est pas
son rayon : la parabole des moindres carrés dépend de la profondeur.

## V-P3-VIT-01/02, V-P3-SIGMA-01/02 : la vitesse et `σ*` à la main

`(1 ; 2) -> (3 ; 5)` : 1,5 ; `σ*(0,5 ; 1 ; 4 ; 0,125) = 0,5` (avec `ρ` pour `ρ²` : 2) ; `σ*(0,25 ; 2 ; 0,5 ; 4) = 1` ; exacts. Tolérance :
aucune (`1e-12` pour `σ*(0,5 ; 1 ; 5,77 ; 0,0816)`).

## V-P3-NAIF-01 : la mesure égale à la mesure naïve, bit pour bit

16 graines rondes tirées (réglages dans le test), six relevés, six profondeurs : contour, mesure (ou erreur), vitesse, `σ*`, au bit près.
Observé : au-dessus des minima du test (300 mesures, 50 `TropPeuDeLignes`, 200 vitesses, 150 `σ*`). Tolérance : aucune.

## V-P3-FAUTE-01 : la table des fautes

Dix fautes (`Faute` dans `etape_p3_commun`), deux vues (`[à la main, naïf]`), table tirée de la mesure naïve contre la mesure naïve fautive (la colonne « à la main » n'appelle jamais la bibliothèque ; seul `parabole` des huit points exacts la touche), affirmée en entier. Trois
fautes ne sont vues que par les cas à la main (profondeur stricte, règle sans arrêt, dernier croisement : une dendrite réelle en temps
court ne les distingue pas), deux que par les bits (sommes brutes, interpolation par la droite : les mêmes formules en réels).

## V-P3-SENS-01 : le sens de variation en temps court (`régression`, une mesure de notre modèle)

Réglages : la graine de la table II de Kim et coll. (`W0 = 1`, `d0 = 0,5`, `D = 1`, `Δ = 0,65`, rayon 15, `u` joué), quart de 60,
`dt = 25 / ⌈25 / dt_borne⌉`, mesures à `t = 125` et `t = 150`, profondeur 2. Observé (le prototype et le second prototype, mêmes bits) :

| `dx` | `ε` | position (`t = 150`) | lignes | rayon | vitesse (125 → 150) | `σ*` |
|---|---|---|---|---|---|---|
| 0,8 | 0 | 34,2768 | 15 | 37,339 | 0,082 470 | 0,008 697 |
| 0,8 | 0,02 | 34,9976 | 13 | 27,684 | 0,086 724 | 0,015 045 |
| 0,8 | 0,04 | 36,0391 | 10 | 16,724 | 0,093 998 | 0,038 038 |
| 0,8 | 0,06 | 37,8239 | 7 | 7,465 | 0,106 680 | 0,168 194 |
| 0,4 | 0 | 34,6711 | 29 | 34,664 | 0,083 405 | 0,009 978 |
| 0,4 | 0,02 | 35,4535 | 25 | 25,093 | 0,088 515 | 0,017 943 |
| 0,4 | 0,04 | 36,6250 | 19 | 14,571 | 0,097 252 | 0,048 429 |
| 0,4 | 0,06 | 38,6062 | 13 | 6,418 | 0,111 489 | 0,217 737 |

Tolérance sur les valeurs : `1e-9` en relatif (le calcul est déterministe au bit ; la marge ne couvre que l'écriture décimale) ; les lignes
exactement. Ordre strict à chaque pas de grille. **L'écart contre la grille** : pour chaque paire voisine, `|ΔQ| / max(|Q(0,8) -
Q(0,4)|)` aux deux `ε` de la paire, sur les six cas (trois paires, deux grilles) :

| grandeur | observé : min ; moyenne ; max (6 cas) | exigé | marge |
|---|---|---|---|
| position | 1,58 ; 1,98 ; 2,53 | 1,5 | 5 % |
| vitesse | 2,24 ; 2,62 ; 2,96 | 2 | 12 % |
| rayon | 3,58 ; 3,93 ; 4,30 | 2 | 79 % |
| `σ*` | 2,19 ; 2,69 ; 3,42 | 2 | 10 % |

Ce que vaut l'écart entre les grilles comme erreur de grille (hors des tests, `dx = 0,2`, même cas) : `|Q(0,4) - Q(0,2)|` est 2,8 à 6,3
fois plus petit que `|Q(0,8) - Q(0,4)|` (ordre 2 : 4), donc l'écart entre les grilles majore l'erreur au pas 0,4 d'un facteur d'environ 1,8 à 5, si la convergence est régulière ; à
`dx = 0,2` : position 34,7604 ; 35,5572 ; 36,7540 ; 38,7700, rayon 34,121 ; 24,598 ; 14,228 ; 6,147, vitesse 0,083 72 ; 0,088 95 ;
0,098 00 ; 0,112 50, `σ*` 0,010 26 ; 0,018 58 ; 0,050 41 ; 0,235 3. Domaine : un quart de 96 au lieu de 60 change chaque grandeur de
moins de 1,5 % de l'écart entre les grilles (position : 1,2e-3 au plus). Le régime n'est pas établi (la vitesse baisse encore).

## V-P3-STAB-01 : la dendrite pleine à la borne (`régression`)

`ε` = 0 ; 0,06 ; 0,0665, `dx = 0,8`, quart de 60 × 60, `dt` = la borne exacte de P2, 3000 pas : aucune erreur, `|φ| <= 1 + 1e-12` aux
relevés (observé : 1 exactement), la pointe se mesure et avance. Hors des tests (le juge, la référence de P2 sans le refus du pas) :

| `dt / dt_borne` | états tirés (quart de 16 × 16, 8 familles, 3000 pas, 7 tranches de `ε` de 0 à `1/15`, 5600 états par tranche) | dendrite pleine (2 réglages × 2 pas de grille × 6 `ε`, 10 000 pas) |
|---|---|---|
| 1 | 0 divergence sur 39 200, dépassement 0 exactement | 0 sur 24, `|φ| - 1 = 0` |
| 1,1 | 0 sur 39 200 | 0 sur 24 |
| 1,25 | 5 sur 39 200 (2, 0, 3, 0, 0, 0, 0 par tranche, de `ε` faible à fort) | 0 sur 24 |
| 1,5 | 794 sur 39 200 (196, 145, 133, 106, 101, 60, 53) | 0 sur 24 |

La réserve de la borne ne fond pas quand `ε` monte : les divergences au-delà de la borne baissent avec `ε` (une explication possible,
non vérifiée : la raideur `(1 - ε)(1 + 15 ε)` que la borne emploie majore d'autant plus que `ε` est grand).

## Le calcul du cercle (`cercle.py`, l'outil indépendant de V-P3-RAYON-02)

Python 3.13, sans dépendance ; il imprime les lignes, le sommet, le rayon et `x0` aux profondeurs 1, 2 et 4.

```python
"""Outil du juge (P3) : la parabole des moindres carrés ajustée à un cercle de rayon R échantillonné aux centres des lignes d'un quart
(y_i = (i + 1/2) dx), avec la règle des lignes de la doc (consécutives depuis l'axe, profondeur x0 - x_i <= h), en décimal à 60 chiffres.
Indépendant du Rust : sert de valeur attendue (juste) au test rayon_sur_un_cercle."""
from decimal import Decimal as D, getcontext
getcontext().prec = 60
def mesure(R, dx, h):
    R, dx, h = D(R), D(dx), D(h)
    xs = []
    i = 0
    while True:
        y = (D(i) + D("0.5")) * dx
        if y >= R: break
        xs.append((y, (R * R - y * y).sqrt()))
        i += 1
    x0 = xs[0][1]
    pts = []
    for y, x in xs:
        if x0 - x <= h: pts.append((y, x))
        else: break
    m = D(len(pts))
    my = sum(y * y for y, _ in pts) / m
    mx = sum(x for _, x in pts) / m
    sxy = sum((y * y - my) * (x - mx) for y, x in pts)
    syy = sum((y * y - my) ** 2 for y, _ in pts)
    b = sxy / syy
    return len(pts), mx - b * my, -1 / (2 * b), x0
for h in ["1", "2", "4"]:
    m, s, r, x0 = mesure(8, "0.5", h)
    print(f"R=8 dx=0.5 h={h} : lignes={m} sommet={s:.17f} rayon={r:.17f} x0={x0:.17f}")
```

# Étape P2 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape P2 (la dendrite 2D du champ de phase de Karma et Rappel, en `f64`, déterministe
et symétrique sous les huit symétries du carré) attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque
identifiant (`V-P2-…`) est cité dans la doc des tests (`tests/etape_p2_*.rs`). **Aucune valeur « juste » de cette fiche ne vient du code
qu'elle juge.** Les tests `régression` (un résultat de notre code, figé) sont dits tels : le repère de la diagonale, la position de la
pointe en temps court, la zone sûre.

**La vitesse de pointe de la théorie n'est PAS une valeur de cette fiche** : ce schéma ne l'atteint pas (voir `V-P2-REPERE-VT` : un
repère mesuré, aucun test, aucune tolérance).

**Outils indépendants** (Python 3.13, dans les notes de la préparation ; aucun n'est dans le dépôt) : `p2_ref.py` et
`p2_cas_empreinte.py` (l'écrivain : le schéma d'après la doc de `src/dendrite.rs`, en flottants Python sans numpy) ; `juge_ref.py` (le
juge : un troisième code, écrit d'après la seule doc sans lire les deux autres, une case à la fois) ; `fractions.Fraction` (le pas isotrope
à la main) ; du côté de P1 : `juge_onde.py` (la vitesse exacte du front poussé). Le code de référence de l'étape est un second prototype
Rust écrit d'après la seule doc : il donne les bits du premier sur 30 000 cas tirés (valides et refusés), hors un point où le premier
s'écartait de la doc (la borne de pas de la graine ronde : V-P2-REF-01, (e)).

## Ce qui a été lu, et ce qui ne l'a pas été

- **Lu** (l'écrivain sur la version HTML, le juge sur le PDF d'arXiv, v1 du 16 octobre 1998, et sur la version HTML) : Y.-T. Kim,
  N. Provatas, N. Goldenfeld, J. Dantzig, « Universal Dynamics of Phase-Field Models for Dendritic Growth », Phys. Rev. E 59, R2546 (1999),
  arXiv cond-mat/9810189.
  - Page 2, équations 4 à 6 : `∂u/∂t = D ∇²u + ½ ∂h(φ)/∂t` ; `τ(n) ∂φ/∂t = ∇·(W(n)² ∇φ) − ∂F/∂φ + ∂x(|∇φ|² W ∂W/∂φ,x) + ∂y(|∇φ|² W ∂W/∂φ,y)` ;
    `F = f(φ) + λ u g(φ)`, `f = φ⁴/4 − φ²/2` ; `W(n) = W0 A(n)`, `τ(n) = τ0 A(n)²`,
    `A(n) = (1 − 3ε)[1 + (4ε/(1 − 3ε)) (φ,x⁴ + φ,y⁴)/|∇φ|⁴]` ; `d(n) = d0 [A + ∂θ²A] = d0 (1 − 15 ε cos 4θ)` ; `λ = W0 a1/d0`,
    `τ0 = W0³ a1 a2/(d0 D) + W0² β0/d0`.
  - Page 2, table I, modèle 2 (« a form used by Karma and Rappel ») : `∂g/∂φ = (1 − φ²)²`, `h = φ`, `a1 = 5/(4√2)`, `a2 = 47/75`.
  - Page 3, table II (les quatre lignes, colonnes `Δ`, `L`, `Ro`, `Δx`, `Δt`, `D`, `do`, `Ṽ`, `IPe`) : `0,45 ; 1000 ; 17 ; 0,39 ; 0,010 ;
    3 ; 0,5 ; 0,00545 ; 0,011` — `0,55 ; 800 ; 15 ; 0,39 ; 0,016 ; 2 ; 0,5 ; 0,0170 ; 0,034` — `0,65 ; 800 ; 15 ; 0,39 ; 0,016 ; 1 ; 0,5 ;
    0,0469 ; 0,094` — `0,65 ; 800 ; 15 ; 0,39 ; 0,004 ; 2 ; 1,5 ; 0,0469 ; 0,031`. Le texte de la même page : `Ṽ = V d0/D` est la vitesse
    de pointe sans dimension **prédite par la théorie de la solvabilité** (ce n'est pas une vitesse simulée), `Δx` est le pas minimal
    d'un maillage adaptatif par éléments finis, et `ε = 0,05`, `β0 = 0`, `W0 = 1` pour toutes les simulations.
  - Page 3, légende de la figure 2 (`Δ = 0,65`, `d0 = 0,5`, `IPe = 0,094`) : « When IPe is too large the different models do not exhibit
    universal behavior. » La phrase porte sur **l'accord entre les cinq modèles**, pas sur la valeur de la théorie. Le corps du texte
    (même page) attribue ces écarts à des corrections d'ordre supérieur en `IPe` et y voit le signe que les équations ne travaillent pas
    dans la limite de l'interface nette (« sharp interface limit ») ; la figure 3 (`d0 = 1,5`, `IPe = 0,031`) retrouve l'accord entre modèles. Les courbes des
    figures sont des images : la valeur de plateau du modèle 2 à `d0 = 0,5` n'y est pas lisible.
- **Non ouvert** : A. Karma et W.-J. Rappel, Phys. Rev. E 57, 4323 (1998) (payant, pas sur arXiv), d'où viennent le modèle et, selon Kim
  et coll., la théorie à laquelle ils se comparent ; aucune page n'en est citée, aucune valeur n'en est tirée.
- **Lu par l'écrivain, non relu par le juge** : N. Provatas, N. Goldenfeld, J. Dantzig, Phys. Rev. Lett. 80, 3308 (1998) (ne donne pas le
  chiffre).

## V-P2-A1, V-P2-A2 : `a1 = 5√2/8`, `a2 = 47/75`

Source : Kim et coll., table I, modèle 2 (lu) ; la fiche de P1 (V-P1-A1 : calcul à la main ; V-P1-A2 : recalculé par `juge_k.py`).
`A1 = 0,883 883 476 483 184 4`, `A2 = 47/75`. Tolérance : aucune.

## V-P2-H-01 : le facteur `H(s, t)`

Source : le calcul à la main des termes croisés à partir de l'équation 6 (`∂a/∂φx = 16 ε φx φy² (φx² − φy²)/|∇φ|⁶`, refait par le juge),
avec `ε = 1/16` (dyadique : exact) : `H(1,0) = 289/256`, `H(1,1) = 225/256`, `H(0,1) = 17/256`, `H(3,1) = 1085/1024` ; garde `1e-24`,
stricte. Tolérance : aucune (`1e-15` pour `ε = 0,05`).

## V-P2-MAIN-01 : un pas isotrope sur 3 × 3 cases

Source : `fractions.Fraction`, rationnels exacts (`epsilon = 0`, `dt = 1/16`), recoupé bit pour bit par `p2_ref.py`. Les 18 valeurs sont
dans le test. Tolérance : aucune.

## V-P2-02 : la vitesse exacte du front droit d'axe (le front poussé de P1)

Source : V-P1-ONDE de la fiche de P1 (le tir dans le plan de phase de `juge_onde.py`, à tout ordre en `λΔ`) : `c(0,1) = 0,112 975 044 459`,
`c(1) = 1,020 609 388 452` ; `v = c W0 / (τ0 (1 + ε))` car, quand `φy = 0`, les termes croisés s'annulent (`∂a/∂φx = 0`) et le modèle se
réduit à P1 avec `W = W0 (1 + ε)`, `τ = τ0 (1 + ε)²`. Pour la diagonale, `1 − ε` (les termes croisés s'annulent aussi quand `φx = φy`).

## V-P2-AXE-01 : le front droit d'axe, tolérances

`W0 = 1,25`, `τ0 = 2`, `D = 1`, `ε = 0,05`, boîte 24, front en 6, `t_a = 10`, `dt` = borne. Écart relatif `v/v_exacte − 1` observé (les deux
prototypes, mêmes bits) / tolérance :

| `(λ ; λΔ)` | `dx = 1` | `dx = 0,5` | `dx = 0,25` |
|---|---|---|---|
| (2 ; 0,1) | −4,5004e-2 / 5,1e-2 | −7,5759e-3 / 9,5e-3 | −1,8709e-3 / 2,35e-3 |
| (1 ; 1) | −4,5922e-2 / 5,2e-2 | −1,2046e-2 / 1,51e-2 | −2,9812e-3 / 3,75e-3 |

Tolérances : 1,13 à 1,26 fois l'observé. Critère qui ne dépend pas de la mesure : le rapport des écarts de `0,5` à `0,25` est entre 3,7 et
4,4 (observé : 4,05 et 4,04 ; ordre 2 : 4) ; de `1` à `0,5` : 5,9 et 3,8 (hors de l'asymptote, non exigé). Toutes les lignes égales au bit
près. Boîte doublée (48) à `dx = 0,5`, hors des tests : le même écart à 1e-7 près. Un `dx = 0,125` coûte 14 à 20 s : non retenu (mesuré par l'écrivain
hors des tests : −4,93e-4 et −7,04e-4).

## V-P2-AXE-02 : le repère de la diagonale (`régression`, pas une valeur exacte)

Boîte 60 (l'écrivain a mesuré +16 % dans une boîte de 24 et +0,4 % dans 40 : les bords comptent), `dx = 0,5`, `t_a = 10` : −2,5165e-4 pour
`(2 ; 0,1)` et −3,5832e-3 pour `(1 ; 1)` ; bornes des deux côtés, de 0,8 à 1,25 fois l'observé : `[−3,2e-4 ; −2,0e-4]` et
`[−4,5e-3 ; −2,85e-3]`. Boîte 80 : −2,5509e-4 et −3,5832e-3. À `dx = 0,25` : +3,2122e-4 et −4,7343e-4 (l'écrivain, hors des tests). C'est
un repère de la grille carrée.

## V-P2-POINTE-01 : la position de la pointe à la main

Calcul à la main de l'interpolation de la doc (`0,75 + 0,5·(0,5/0,75)`, etc.) ; l'inégalité stricte à gauche ; seule la ligne `i = 0` ;
l'ordre des opérations de la doc sur 300 lignes tirées (la formule de la doc écrite dans les tests, `position_ligne`). Tolérance : aucune.

## V-P2-CONV-01 : la position de la pointe en temps court (`régression`, description du calcul)

Réglages de la table II (`Δ = 0,65`, `d0 = 0,5`, `D = 1`, `W0 = 1`, `ε = 0,05`), graine ronde de rayon 15, `u` joué, quart de 44,8,
`t = 50`, `dt = t/⌈t/dt_borne⌉`. Observé (les deux prototypes) : `x` = 25,501745 ; 25,773431 ; 25,830097 pour `dx` = 0,8 ; 0,4 ; 0,2
(tolérance 5e-5), rapport des écarts 4,79 (plage 4,0 à 5,6 : ordre 2 donne 4 à la limite, ordre 1 donne 2) ; domaine doublé : +2,4e-6 et
+2,9e-6 (tolérance 1e-4 ; le plus petit écart de grille est 5,7e-2) ; `dt` moitié : +5,13e-3 et +1,70e-3 (bornes : ± 20 %), `dt` quart :
+7,72e-3 et +2,55e-3 (ordre 1 en temps, hors des tests). Hors des tests, à `t = 30`, quart de 35,2, quatre pas de grille : `x` =
22,428971 ; 22,627547 ; 22,665234 ; 22,674873 pour `dx` = 0,8 ; 0,4 ; 0,2 ; 0,1, rapports des écarts 5,27 puis 3,91 (ordre 1,97).
**Ce n'est pas la vitesse publiée**, et ces positions ne sont pas une valeur de la théorie.

## V-P2-REPERE-VT : `Ṽ = V d0/D` à `Δ = 0,65` (REPÈRE MESURÉ, pas une valeur gelée, aucun test)

Publié : **0,0469** (théorie de la solvabilité), Kim et coll., table II, ligne `Δ = 0,65`, `d0 = 0,5`, `D = 1` (voir plus haut). Mesuré
par ce schéma (grille uniforme, quart, rayon 15, `u` joué, `dt` = la borne, `V` = l'avance de la pointe sur 100 de temps) :

| `dx` | quart, temps | `Ṽ` | écart au publié | qui |
|---|---|---|---|---|
| 0,8 | `L = 200` et `L = 400`, `t` de 1200 à 1500 | 0,04078 ; 0,04080 ; 0,04072 (`L = 400` : les mêmes à 3e-6 près) : **0,0408** | −13,0 % | le juge (31 s et 154 s) et l'écrivain (les mêmes chiffres) |
| 0,4 | `L = 200`, `t` de 1000 à 1400 | 0,04397 ; 0,04396 ; 0,04392 ; 0,04388 : **0,0440** | −6,2 % | le juge (environ 8 min) ; l'écrivain : 0,0440 à `t = 1100` |
| extrapolation d'ordre 2 des deux | | ≈ 0,0450 | ≈ −4 % | un calcul, pas une mesure |

`Ṽ` bouge donc encore de 8 % quand la grille passe de 0,8 à 0,4. `Δ = 0,55` (publié 0,0170) : 0,0165 à `t = 3000`, `dx = 0,8`, encore en
baisse (l'écrivain ; non établi, non refait par le juge). `Δ = 0,45` (publié 0,00545) : non mesuré (`L = 1000`, `D = 3` : au moins 1250
cases de côté à `dx = 0,8`). `d0 = 1,5` (la seconde ligne à `Δ = 0,65`) : mesuré hors des tests le 15/11, au pas 0,8 seulement :
0,0450 de `t = 5500` à 6000 (−4 %), en régime non établi, un repère (`MODELE.md`, « Étape P2 »).

## V-P2-NAIF-01 à 04, V-P2-SYM-01 à 03, V-P2-QUART-01, V-P2-FAUTE-01/02

Sources : un simulateur naïf écrit dans le test d'après la doc (une case à la fois), des calculs entiers (les images de symétrie, les
cases du quart), `A1` et `A2`. Tolérance : aucune (égalité au bit près). La graine à huit symétries : `h(max(|a|,|b|), min(|a|,|b|), sel)`,
mantisses pleines. V-P2-NAIF-04 (la garde au seuil exact) : `1e-12 · 1e-12` est le flottant `1e-24` (vérifié dans le test). Hors des
tests (le juge) : les huit symétries sur 6000 couples (réglages, taille de 3 à 33, 2890 paires et 3110 impaires, mode, nombre de pas ;
2,8 millions de pas) : 0 couple asymétrique ; le quart contre l'entier sur 2000 cas (615 000 pas) : 0 écart.

## V-P2-REF-01 à 05, V-P2-STAB-01, V-P2-MARGE-01

Sources : calcul à la main de la borne (`0,5·(2/10)`, `0,5·(1/8,25)`, `0,5·(450/4232)`, `0,5·(2/(10+(1,54·1)·2))`) ; les ordres d'erreur
sont ceux de la doc. **La zone sûre est une MESURE de notre schéma** (`régression`). Dans le test : 80 réglages × 6 familles, 1500 pas.
Hors des tests (le juge, la référence sans le refus du pas) : `W0` de 0,3 à 3 ; `τ0` de 0,1 à 10 ; `λ` de 0,05 à 20 ; `D` de 0,05 à 10
(tous tirés sur une échelle logarithmique) ; `ε` de 0 à `1/15` exclu (dont 0, 0,05 et le dernier flottant avant `1/15`) ; `dx` de 0,15 à
2,5 ; `U` de 0 à 3 ; huit familles d'états sur un quart de 16 × 16 (front rond et `u` en rampe ; front rond et `u = 0` ; `φ` tiré et `u`
dans `[−U, 0]` ; front rond et `u = −U` gelé ; `φ` tiré et `u` des deux signes, joué puis gelé ; damier `φ = ±1`, `u = ±U` ; front droit) ;
3000 pas :

| `dt / dt_borne` | états | `Divergence` | dépassement de `max(1, \|φ\| de départ)` |
|---|---|---|---|
| 1 (deux séries de 3200 réglages) | 51 200 | 0 | 0, exactement |
| 1,1 | 25 600 | 0 | 0 |
| 1,25 | 25 600 | 8 (0,03 %) | 0 chez les autres |
| 1,5 | 25 600 | 273 (1,1 %) | 2,4e-5 |
| 2 | 25 600 | 1602 (6,3 %) | 9,5e-4 |
| 3 | 25 600 | 16 376 (64 %) | — |

Le `0,5` de la borne n'est donc pas un facteur 2 de réserve : la réserve mesurée est entre 10 % et 25 %. En dendrite pleine (`Δ = 0,65`,
`dt` = la borne) : `|φ| − 1 = 0` exactement à chaque relevé (tous les 100 de temps) à `dx` = 0,8 jusqu'à `t = 1500` et à `dx` = 0,4 jusqu'à
`t = 1400` (le juge) ; l'écrivain a relevé un dépassement inférieur à `5e-13`. `MARGE = 0,001` : au moins `10^9` fois ce dépassement.

## V-P2-EMP-01 à 03 : l'écriture canonique et les empreintes

Écriture canonique : à la main (octets) dans le test. Empreintes S1 (4), S2, S3 : calculées par `p2_cas_empreinte.py` (l'écrivain) et
recalculées à l'identique par `juge_ref.py` (le juge, sans lire le premier). Empreintes J1 à J6, les trois bornes de pas en décimal
(`0,012618586934709504` ; `0,009284637797648739` ; `0,06809049201035892`) et les bits de la pointe de J6 (`0x401bd6f4e255571c`,
soit 6,9599185337627) : `juge_ref.py`. Aucune ne vient du Rust.

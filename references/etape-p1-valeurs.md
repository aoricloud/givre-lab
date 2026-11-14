# Étape P1 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape P1 (le front droit du champ de phase de Karma et Rappel, en 1D, en `f64` et en
virgule fixe, et l'essai 2D de la règle des nombres) attend, d'où elle vient, avec quelle tolérance et quel critère de réussite. Chaque
identifiant (`V-P1-…`) est cité dans la doc des tests qui l'emploient (`tests/etape_p1_*.rs`). Elle est séparée des résultats figés de notre
propre code (`references/*.empreinte`) : **aucune valeur « juste » de cette fiche ne vient du code qu'elle juge**. Les tolérances, elles, ont
été MESURÉES (le critère était écrit avant le calcul : « 1,15 à 1,25 fois la valeur observée » ; tenu partout sauf une ligne, à 1,14 :
le cas 1 à `dx = 0,5`, voir V-P1-01), et c'est dit ; chaque cas porte en plus un critère qui ne dépend pas de la mesure (un ordre de
convergence).

**Outils indépendants** (Python 3.13, dans les notes de la préparation ; le code Rust n'est jamais l'outil). De l'écrivain,
`essai-p1-outils/` : `p1_k.py` (les `k` de Neumann par bissection, en `float` avec `math.erfc` et en `decimal` à 60 chiffres ; `a1` par
Simpson) ; `p1_ref.py` (le schéma 1D d'après la doc, en flottants Python et en entiers, l'écriture canonique, le SHA-256). Du juge,
`essai-p1-juge-outils/` : `juge_k.py` (`a1`, `a2` par leurs intégrales ; les `k` par une autre écriture de la fonction ; les résidus de la
solution de Neumann) ; `juge_onde.py` (la vitesse exacte de l'onde progressive du front poussé, par un tir dans le plan de phase) ;
`juge_ref.py` (le schéma 1D où chaque opération `f64` est faite en rationnels exacts puis arrondie une seule fois : la définition de
l'IEEE 754).

## Ce qui a été lu, et ce qui ne l'a pas été

- **Lu** (écrivain, relu par le juge) : A. Karma et W.-J. Rappel, « Phase-Field Model of Dendritic Sidebranching with Thermal Noise »,
  Phys. Rev. E 60, 3614 (1999), arXiv cond-mat/9902017 : équations 41 et 42 (le modèle sans dimension,
  `∂φ/∂t = ∇²φ + φ - φ³ - Λ u (1-φ²)² [+ bruit]` et `∂u/∂t = D ∇²u + ½ ∂p(φ)/∂t [+ bruit]`), le profil `φ0(z) = -tanh(z / (√2 W))`,
  `J = 16/15`. Sa référence 21 est l'article de 1998.
- **Lu** (juge) : Y.-T. Kim, N. Provatas, N. Goldenfeld et J. Dantzig, « Universal Dynamics of Phase-Field Models for Dendritic Growth »,
  arXiv cond-mat/9810189 : équations 4 et 5 (`∂u/∂t = D ∇²u + ½ ∂h(φ)/∂t` ; `τ ∂φ/∂t = ∇·(W² ∇φ) - ∂F(φ, λu)/∂φ + …`, `φ = +1` dans le
  solide, `-1` dans le liquide) ; « λ = W a1 / d0 » et « τ = W³ a1 a2 / (d0 D) + W² β / d0 », attribués à Karma et Rappel ; **table 1,
  modèle 2** (`∂g/∂φ = (1-φ²)²`, `h = φ` : le nôtre) : `a1 = 5/(4√2)`, `a2 = 47/75`.
- **Lu** (juge) : A. Karma, « Phase-Field Formulation for Quantitative Modeling of Alloy Solidification », arXiv cond-mat/0103289 : pour
  le modèle symétrique, la condition de Stefan `V = -D (∂n u|⁺ - ∂n u|⁻)`, la condition de Gibbs-Thomson `u = -d0 κ - β V` (éq. 5), et
  **l'équation 6 : `d0 = a1 W / λ` et `β = a1 [τ / (W λ) - a2 W / D]`**, « identiques à celles de Karma et Rappel » (sa référence 3 :
  Phys. Rev. E 53, R3017, 1996 et Phys. Rev. E 57, 4323, 1998) ; la cinétique s'annule pour `λ = D τ / (a2 W²)` ; `a1 = I / J`,
  `a2 = (K + J F) / (2 I)` avec `F = √2 ln 2` et `K` (éq. 12) ; `a1 = 0,8839` et `a2 = 0,6267` pour `∂g/∂φ = (1-φ²)²`.
- **La formule de `β`** : le brief de l'étape écrivait `a2 λ W / D` ; c'est `a2 W / D` (un `λ` en trop dans le brief : les deux sources
  ci-dessus concordent, et la relation de `τ` de Kim et coll. redonne `β = a1 (τ / (λ W) - a2 W / D)` par `d0 = a1 W / λ`).
- **Non ouvert** : l'article de 1998 lui-même (Phys. Rev. E 57, 4323 : payant, pas sur arXiv) : aucune page, aucune équation n'est citée
  de lui ; ce qui précède vient de trois articles qui le reprennent, dont un de son premier auteur. **Non ouvert** : Carslaw et Jaeger,
  *Conduction of Heat in Solids*, 2e éd. (1959), chap. XI, et aucune source ouverte donnant la forme « liquide surfondu » de la solution
  de Neumann n'a été trouvée : elle est dérivée à la main (V-P1-K) et vérifiée par ses résidus.

## V-P1-A1 : `a1 = 5√2/8`

Source : Kim et coll., table 1 (`5/(4√2)`) ; Karma 2001 (`0,8839`, `a1 = I/J`) ; et le calcul à la main : avec `W = 1`,
`φ0 = -tanh(x/√2)`, `t = tanh(x/√2)`, `φ0' = -(1 - t²)/√2`, `dx = √2 dt / (1 - t²)` : `I = ∫ φ0'² dx = (√2/2) ∫₋₁¹ (1 - t²) dt = 2√2/3` ;
`J = ∫ (1 - φ0²)² |φ0'| dx = ∫₋₁¹ (1 - t²)² dt = 16/15` ; `a1 = I / J = 5√2/8 = 0,883 883 476 483 184 4`. Contrôles numériques (Simpson) :
`p1_k.py` et `juge_k.py` donnent `I = 0,942 809 041 582`, `J = 1,066 666 666 667`, `I/J = 0,883 883 476 483`. Tolérance : aucune (`A1` dans
`etape_p1_commun`).

## V-P1-A2 : `a2 = 47/75`

Source : Kim et coll., table 1, modèle 2 : `47/75` (= 0,626 666…) ; Karma 2001 : `0,6267`. Recalculé par `juge_k.py` avec la formule de
Karma 2001 : `K = ∫ φ0' g'(φ0) ∫₀^η φ0 = 0,136 045 312 416`, `F = √2 ln 2 = 0,980 258 143 469`,
`a2 = (K + J F) / (2 I) = 0,626 666 666 667`. Il ne sert qu'à CHOISIR, dans le cas 3, le `λ` qui annule la cinétique (`A2 = 47.0 / 75.0`
dans `etape_p1_commun`). Tolérance : aucune.

## V-P1-01 : le front au repos

Source : le profil exact `φ0(x) = -tanh(x / (W√2))` (Karma et Rappel 1999), calculé dans le test avec `tanh` de la bibliothèque standard
(permise dans un test). Exact pour ce modèle : c'est la solution stationnaire de `W² φ'' + φ - φ³ = 0` (`u = 0`). Réglage : `W = 1,25`,
`tau = 1,6`, `lambda = 1`, `D = 1`, domaine de largeur 40 centré sur le front (`n = 40/dx`), départ en marche, `u = 0` gelé,
`dt = dt_max / 2`, temps `60 tau`. Tolérance : **mesurée** : `dx` = 1 ; 0,5 ; 0,25 ; 0,125 → observé 1,668e-2 ; 4,375e-3 ; 1,1193e-3 ;
2,815e-4 (`f64`) et 2,8153e-4 (`Fixe`) → tolérance 1,95e-2 ; 5,0e-3 ; 1,3e-3 ; 3,3e-4 (rapports 1,17 ; 1,14 ; 1,16 ; 1,17). Critère qui ne
dépend pas de la mesure : l'erreur du laplacien à 3 points est d'ordre 2, le rapport des écarts de deux pas successifs est entre 3,4 et
4,6 (observé 3,8 ; 3,9 ; 4,0). Critère : écart max ≤ tolérance, en `f64` et en `Fixe` ; position du front = `n dx / 2` à 1e-12 (`f64`) et
1e-8 (`Fixe` : son arrondi n'est pas symétrique ; observé 2,5e-9).

## V-P1-ONDE : la vitesse exacte du front poussé

Source : `juge_onde.py`. Avec `u = -Δ` gelé, le modèle est `τ φ_t = W² φ_xx + φ - φ³ + a (1 - φ²)²`, `a = λΔ`. Son onde progressive
`φ(x - vt)` vérifie, en unités `W = τ = 1` et dans le plan de phase `p(φ) = dφ/dz`, `p dp/dφ + c p + φ - φ³ + a (1 - φ²)² = 0`, de `φ = +1` à
`φ = -1` ; `c = v τ / W` est la valeur pour laquelle l'orbite partie de `+1` rejoint celle qui arrive à `-1` (RK4 en `φ`, 40 000 pas,
départ sur les variétés linéaires corrigées à l'ordre 2 ; contrôle `a = 0` : `p(0) = -1/√2` à 2e-13 ; 20 000 pas donnent les 12 mêmes
chiffres). Ni grille d'espace ni pas de temps : la valeur est exacte à tout ordre en `a`.

| `a = λΔ` | `c = v τ / W` | `c / c1 - 1` (`c1 = a / a1`, le premier ordre) |
|---|---|---|
| 0,01 | 0,011 313 545 590 | -1,44e-5 |
| 0,1 | 0,112 975 044 459 | -1,432e-3 |
| 0,3 | 0,335 213 712 822 | -1,2367e-2 |
| 1 | 1,020 609 388 452 | -9,790e-2 |

Le premier ordre de l'article (`v = λ Δ W / (a1 τ)` : `β = a1 τ / (λ W)` quand `D` est infini, `V = Δ / β`) est donc retrouvé à
`-0,144 a²` près aux petits `a` (`a` = 0,001 : -1,44e-7), et cesse de suffire vers `a = 0,3` (1,2 %). Tolérance : aucune (`C_EXACT`).

## V-P1-02 : le front poussé à température fixée

Source : V-P1-ONDE. Réglage : `W = 1,25`, `tau = 2`, `D = 1`, front à 10 au départ (profil du repos), grille de largeur 60,
`dt = dt_max / 2`, vitesse mesurée par la position de la doc entre `t = 20` et `t = 20 + max(20, 1,5 / v)` (au moins 1,5 de course : sur
moins d'une case, la mesure ne verrait que l'erreur de l'interpolation du zéro ; c'était le cas du jeu de l'écrivain à `λΔ = 0,01`, dont
l'écart changeait de signe avec `dx`). La vitesse ne dépend que de `λΔ` : le test l'exerce avec `λ = 2`, `Δ = 0,05`. Tolérance :
**mesurée** (`TOL_CAS2`), l'écart à la vitesse exacte : `dx = 0,5` : -8,171e-3 ; -8,237e-3 ; -8,846e-3 ; -1,8561e-2 (`λΔ` = 0,01 ; 0,1 ;
0,3 ; 1) → 9,9e-3 ; 9,9e-3 ; 1,07e-2 ; 2,23e-2 — `dx = 0,25` : -2,0195e-3 ; -2,0435e-3 ; -2,2322e-3 ; -4,7174e-3 → 2,5e-3 ; 2,5e-3 ; 2,7e-3 ;
5,7e-3 — `dx = 0,125` : -5,005e-4 ; -5,101e-4 ; -5,819e-4 ; -1,1587e-3 → 6,1e-4 ; 6,2e-4 ; 7,0e-4 ; 1,4e-3 (rapports de 1,20 à 1,24). `Fixe`
suit `f64` à 3e-8 près. Critère qui ne dépend pas de la mesure : l'écart est une erreur de grille d'ordre 2, le rapport de deux pas
successifs est entre 3,4 et 4,6 (observé : de 3,8 à 4,1). Contre le premier ordre seul, l'écart serait de 10 % à `λΔ = 1` : la tolérance
de l'écrivain (13 %) laissait passer une faute de cette taille ; la référence exacte la ramène à 0,14 %. Une part de l'écart vient de
`dt` (à `dt` quatre fois plus petit, celui de `λΔ = 1`, `dx = 0,125` passe de -1,16e-3 à -7,7e-4) : mesuré, non gelé.

## V-P1-K : les `k` de Neumann

Source : le problème de Stefan à une phase du modèle, dérivé à la main : solide `u = 0` pour `x < X(t)`, liquide `∂u/∂t = D ∂²u/∂x²`,
`u(∞) = -Δ`, `u(X) = 0` (cinétique nulle), chaleur latente 1 (le `½ ∂φ/∂t` intègre à `V` à travers l'interface, `φ` allant de -1 à +1 :
c'est la condition de Stefan `V = -D (∂n u|⁺ - ∂n u|⁻)` de Karma 2001, le solide restant à `u = 0`). La solution est
`u = -Δ (1 - erfc(x / (2√(Dt))) / erfc(k))`, `X = 2 k √(D t)` ; le bilan `V = -D ∂u/∂x(X⁺)` avec `V = k √(D/t)` donne
`Δ = √π k e^{k²} erfc(k)`. Vérifié par ses résidus (`juge_k.py`, `Δ = 0,5`, `D = 1,5`, `t = 30`) : `u(X) = 0`, résidu de l'équation de la
chaleur 2e-11, `V = -D ∂u/∂x(X⁺) = 0,096 766 199 4` des deux côtés. (Carslaw et Jaeger non ouvert : la formule est celle que le brief leur
attribue.) Valeurs : `p1_k.py` (bissection en `float` et en `decimal` à 60 chiffres) et `juge_k.py` (une autre écriture,
`Δ(k) = 2k ∫₀^∞ e^{-s² - 2ks} ds`, Simpson) :

| `Δ` | `k` (`p1_k.py`) | `k` (`juge_k.py`) |
|---|---|---|
| 0,25 | 0,168 999 042 462 002 | 0,168 999 042 462 0 |
| 0,5 | 0,432 751 599 366 308 | 0,432 751 599 366 3 |
| 0,75 | 0,971 850 282 583 413 | 0,971 850 282 583 4 |

(le brief annonçait `k ≈ 0,43` pour `Δ = 0,5` : confirmé.) Tolérance : aucune sur le `k` (les 12 chiffres de `K_NEUMANN` sont exacts).

## V-P1-03 : le front de Neumann

Source : V-P1-K. Ce cas n'est exact que dans la limite de l'interface mince : le modèle a une interface de largeur `W`, la solution de
Neumann une interface nette. Réglage : `D = 1,5`, `W` = 1 ou 0,5, `tau = 2 W²`, donc `lambda = D tau / (a2 W²) = 2 D / a2 = 4,787…` (la
cinétique `β` est nulle : V-P1-A2), solide contre un mur sans flux, départ à `x0 = 10` sur le profil exact de `u` au temps
`t0 = (x0 / 2k)² / D` (le départ singulier `X(0) = 0` est évité : le temps est décalé de `t0`, `X(t)² = 4 k² D (t + t0)`), on avance de 400
en temps ; `dt = dt_max / 2`. Comparaison : `X_mesuré / (2 k √(D (t0 + 400))) - 1`. Tolérance : **mesurée** (`TOL_CAS3`), `f64` / `Fixe` :
`Δ = 0,25` : -3,802e-4 / -3,818e-4 (`dx` 0,5), -1,874e-4 / -1,932e-4 (`dx` 0,25) → 4,6e-4 ; 2,4e-4 — `Δ = 0,5` : -4,389e-4 / -4,401e-4,
+2,941e-4 / +2,893e-4 → 5,3e-4 ; 3,6e-4, et `W = 0,5`, `dx = 0,25` : -3,584e-4 / -3,622e-4 → 4,4e-4 — `Δ = 0,75` : +8,917e-3 / +8,916e-3,
+1,1903e-2 / +1,1899e-2 → 1,07e-2 ; 1,43e-2, et `W = 0,5`, `dx = 0,25` : +1,3640e-3 / +1,3611e-3 → 1,65e-3 (rapports de 1,20 à 1,24).

**D'où vient l'écart, et ce que le cas compare** (mesures du juge). Il ne vient ni de `dx` (il ne décroît pas avec lui), ni de `dt` (un
`dt` huit fois plus petit : +9,3e-3 pour +8,9e-3), ni du bord (domaine de `8 √(D t) + 20`). C'est le reste de l'analyse d'interface
mince : la cinétique n'est annulée qu'au premier ordre, et le terme suivant grandit avec `λ` et avec la vitesse du front rapportée à
l'interface (`τ V / W`, soit 0,57 au départ pour `Δ = 0,75`, `W = 1`). À `Δ = 0,75`, `dx = W/2` : `λ = 1,2` (`τ = 0,5`, `W = 1`) : -0,8 % ;
`λ = 4,79` : +0,9 % ; `λ = 19` (`τ = 8`, `W = 1`, ou `τ = 2`, `W = 0,5`) : +20 % et +5,9 %. L'écrivain gardait `τ = 2` en divisant `W` par 2 :
`λ` était multiplié par 4 et l'écart grandissait (+6 %, +8 % à `dx = 0,125`), d'où sa question. À `λ` fixé (`τ ∝ W²`, le réglage retenu),
l'écart diminue avec `W` comme il doit : +2,8 % (`W = 2`), +0,9 % (`W = 1`), +0,14 % (`W = 0,5`). Il décroît aussi avec le temps (`V`
diminue) : +1,6 % après 100, +0,9 % après 400, +0,3 % après 1600 ; et avec un départ plus lointain (`x0 = 40` : +0,03 %). Critère qui ne
dépend pas de la mesure : à `Δ = 0,75`, l'écart à `W = 0,5` est moins de la moitié de celui à `W = 1` (observé : 6,5 fois moins).

## V-P1-ARITH : flottant contre virgule fixe

Source : le même schéma exécuté dans les deux arithmétiques (aucune valeur publiée). Tolérances mesurées : au repos `|φ_f64 - φ_Fixe|`
≤ 2e-8 (observé 2,6e-9 à `dx` = 0,5, 8,3e-9 à 0,25) ; front poussé (`dx` = 0,25, `λΔ` = 0,1) : écarts relatifs à `v` qui diffèrent de moins
de 1e-7 (observé 5e-9) ; Neumann (`Δ = 0,5`, `W = 1`, `dx = 0,25`) : de moins de 1,2e-5 (observé 4,8e-6). Critère : ces trois bornes, et
l'écart au repos n'est pas nul (les deux arithmétiques ne donnent pas les mêmes bits).

## V-P1-NAIF-xx : le naïf, et les pas à la main

Source : la doc de `src/phase.rs` (le schéma à la lettre) ; le naïf est écrit dans `etape_p1_commun`, sans emprunter de fonction à la
bibliothèque. Le juge a écrit un second prototype d'après la seule doc : il donne les mêmes bits que ce naïf. `-01` : bits égaux en
`f64`, 24 graines ; `-02` : champs bruts égaux en `Fixe` ; `-03` : un pas à la main (valeurs binaires exactes, écrites dans le test) ;
`-04` : `avancer` se compose ; `-05` : la position du front à la main (neuf cas) ; `-06` : la position au bit près contre la formule de la
doc, 300 états tirés ; `-07` : un pas à la main sur un demi-cas de la virgule fixe (`φ = 3/2048` : champ brut 6 291 456 → 9 437 177 ;
`u` : 1 572 861 depuis `+φ`, -1 572 860 depuis `-φ`), recalculé en entiers par `juge_ref.py`. Tolérance : aucune (égalité exacte).

## V-P1-SYM-xx : la symétrie

Source : la règle des nombres (`AGENTS.md`, règle 2) ; les symétries sont des transformations d'indices écrites dans le test. `-01` :
miroir 1D pair et impair, `f64`, au bit près, 9 instants jusqu'à 2000 pas ; `-02` : `Fixe` pair exact, impair non (arrondi demi-cas vers +∞
de `Mul`) ; `-03` : le naïf sans paires perd la symétrie 1D dès les 3 premiers pas ; `-04` : `Diffusion2d` garde les 8 symétries du carré,
`n` = 3, 8, 16, 17, 33, six graines, 300 pas ; `-05` : sans paires (3 écritures), les symétries sont perdues dès le pas 1 :
`((e+o)+n)+s` perd 6 des 7 (pas le miroir est-ouest), `(e+n)+(o+s)` en perd 4, `((e+n)+o)+s` les 7 ; `-06` : `Diffusion2d` = le naïf par
paires, et un pas à la main (`[[1,0],[0,0]] -> [[1/2,1/4],[1/4,0]]`) ; `-07` : `n` impair : le miroir pair reste exact au bit près, le
miroir impair en valeur seulement (la case du milieu vaut `+0.0`, son image `-0.0`). Tolérance : aucune (bits) ; la somme est conservée à
1e-9 (observé ≈ 1e-14).

## V-P1-REF-xx : les refus

Source : la doc de `src/phase.rs` (les erreurs et leur ordre) et des calculs à la main. `-01` : `dt_max` à la main (`1/3`, `4/3`, `1/4`) ;
`-02` : les réglages refusés ; `-03` : les états de départ refusés ; `-04` : `Diffusion2d::new` ; `-05` : le débordement ; `-06` : la borne
`2^20` incluse et l'ordre des constantes ; `-07` : les réglages démesurés (constantes NaN, `dt_max` NaN ou infini) ; `-08` : un côté
démesuré de `Diffusion2d` ; `-09` : le débordement de `u` seul, `±2^31` en `Fixe` ; `-10` : le balayage de 600 réglages sans panique ;
`-11` : `dt_max` au bit près, 300 réglages. Tolérance : aucune.

## V-P1-STAB-xx : ce que `dt_max` garantit

Source : l'analyse linéaire (écrite dans la doc de `src/phase.rs`, « La stabilité ») et les mesures du juge (3000 réglages tirés par
essai, 8000 pas). `-01` : à `dt = dt_max` avec `u` joué (`W = 1,25`, `tau = 2`, `lambda = 1`, `D = 1`, `dx = 0,5`), le facteur du mode en
dents de scie de `u` est `1 - 4 c_u - ½ r λ (1 - φ²)² < -1` dans l'interface : `Err(Debordement)` (observé : au pas 147 depuis un profil en
`tanh`) ; à `0,9 dt_max`, sous la borne couplée (`0,9846 dt_max`), 20 000 pas dans `[-1, 1]`. `-02` (le seul test « régression » de l'étape : une mesure de notre schéma) : à `dt = dt_sur / 2`, 80 réglages
tirés, aucun ne déborde (la mesure large : 3000 réglages, `λU` jusqu'à 80, aucun ; à `dt = dt_max / 2`, 220 sur 3000 débordent, tous avec
`λU ≥ 2` ; à `dt = dt_max`, 844 sur 3000, dont 65 sur 1261 avec `λU < 0,5`). Tolérance : `1e-9` sur `|φ| ≤ 1`.

## V-P1-EMP-xx : les empreintes

Source : `-01` l'écriture canonique à la main ; `-02` les 8 empreintes du scénario (24 cases, 0 et 500 pas, `u` joué ou gelé, `f64` et
`Fixe`) et `-03` les 2 du front poussé (320 pas) : `p1_ref.py` ; `-04` les 4 du scénario du juge (31 cases, 1 et 257 pas) : `juge_ref.py`.
`juge_ref.py` (rationnels exacts, un seul arrondi par opération) redonne les 10 empreintes de `p1_ref.py` (flottants natifs de Python),
qui sont celles des deux prototypes Rust : ce ne sont pas des régressions déguisées, et ni Rust ni CPython ne contractent un produit et
une somme. Elles sont calculées avec des `f64` en `+ - * /` et des entiers seulement, donc identiques sous Linux et sous Windows (les
profils de départ sont écrits sans `tanh`). Tolérance : aucune.

## V-P1-FAUTE-xx : les fautes

Source : dix variantes du naïf (signe de `λ`, `½` oublié, `(1-φ²)` simple, mise à jour en place, bord qui fuit, `u` joué quand il est
gelé, `W` au lieu de `W²`, `dt` au lieu de `dt/τ`, somme sans paires, `dφ` du pas d'avant). `-01` : sortent de la tolérance dans les cas où
elles se voient (table mesurée, rejouée par le juge sur les cas amendés : inchangée) ; `-02` : changent les bits, la bibliothèque donnant
ceux du naïf sans faute. Deux fautes ne se voient par aucun des trois cas (la somme sans paires ; le `dφ` du pas d'avant, une erreur
d'ordre 1 en `dt` : -3,6e-4 pour -4,4e-4 au cas 3) : ce sont le naïf bit à bit, les empreintes et la symétrie qui les condamnent.
Tolérance : celles des cas.

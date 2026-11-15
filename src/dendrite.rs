//! La dendrite 2D du champ de phase (étape P2) : le modèle de Karma et Rappel avec l'anisotropie d'ordre 4, en différences
//! finies explicites sur une grille carrée, en `f64`, déterministe au bit près et symétrique sous les huit symétries du carré.
//!
//! # Le modèle
//!
//! Sources : A. Karma et W.-J. Rappel, Phys. Rev. E 57, 4323 (1998) (non ouvert : payant, pas sur arXiv), dans la forme que
//! reprennent Y.-T. Kim, N. Provatas, N. Goldenfeld et J. Dantzig, « Universal Dynamics of Phase-Field Models for Dendritic
//! Growth », Phys. Rev. E 59, R2546 (1999), arXiv cond-mat/9810189 (lu, PDF et version HTML : équations 4, 5 et 6, table I modèle 2, table II).
//! `φ = +1` dans le solide, `-1` dans le liquide, `u < 0` dans le liquide surfondu :
//!
//! ```text
//! τ(n) ∂φ/∂t = ∇·(W(n)² ∇φ) + ∂x(|∇φ|² W(n) ∂W(n)/∂(∂xφ)) + ∂y(|∇φ|² W(n) ∂W(n)/∂(∂yφ)) + φ - φ³ - λ u (1 - φ²)²
//! ∂u/∂t    = D ∇²u + ½ ∂φ/∂t
//! W(n) = W0 a(n),   τ(n) = τ0 a(n)²,   a(n) = (1 - 3ε) + 4ε (φx⁴ + φy⁴) / |∇φ|⁴     (= 1 + ε cos 4θ)
//! ```
//!
//! (`n = ∇φ/|∇φ|` ; `θ` l'angle de `n` avec l'axe `x` ; `ε` l'anisotropie, `0,05` chez Kim et coll. ; `a = 1 + ε` sur les
//! axes, `1 - ε` sur les diagonales). Les deux termes croisés, calculés à la main, valent
//! `∂a/∂φx = 16 ε φx φy² (φx² - φy²) / |∇φ|⁶` et `∂a/∂φy = 16 ε φy φx² (φy² - φx²) / |∇φ|⁶`, donc les flux de `φ` sont
//! `Fx = W0² φx a (a + 16 ε φy² (φx² - φy²) / |∇φ|⁴)` et `Fy = W0² φy a (a + 16 ε φx² (φy² - φx²) / |∇φ|⁴)` : le même
//! facteur `H(propre², autre²)` multiplie le gradient de la direction du flux (voir [`facteur_h`]). **Sur un front droit
//! d'axe (`φy = 0`), `∂a/∂φx = 0`, les termes croisés s'annulent**, `a = 1 + ε` partout où `φx ≠ 0` : le modèle se réduit
//! au modèle 1D de P1 avec `W = W0 (1 + ε)` et `τ = τ0 (1 + ε)²` (test `front_droit_d_axe_a_la_vitesse_exacte_de_p1`).
//!
//! Les paramètres se rattachent au problème à frontière libre (Kim et coll., après l'équation 6 ; Karma 2001, arXiv
//! cond-mat/0103289, équation 6) par la longueur capillaire `d0 = a1 W0 / λ` et la cinétique `β = a1 (τ0/(λ W0) - a2 W0/D)` ;
//! `β = 0` pour `λ = D τ0 / (a2 W0²)` ; `a1 = 5√2/8` et `a2 = 47/75` (modèle 2 de la table I). [`Reglages::depuis_capillarite`]
//! écrit ce rattachement. La grandeur publiée est `Ṽ = V d0 / D` (la vitesse de pointe en régime établi).
//!
//! # Le schéma, à la lettre
//!
//! **Cette section est la spécification** : une implémentation qui la suit donne EXACTEMENT les mêmes bits.
//!
//! Une grille `n × n` (`n >= 3`), au pas `dx` ; la case `(i, j)` (`i` la ligne, `j` la colonne ; stockage ligne par ligne,
//! `phi[i * n + j]`) est lue comme le point `(x, y)` avec `x` croissant avec `j` et `y` croissant avec `i`. Ses voisines : `x+` =
//! `(i, j+1)`, `x-` = `(i, j-1)`, `y+` = `(i+1, j)`, `y-` = `(i-1, j)`, et les diagonales. **Hors de la grille, la voisine est la
//! case du bord par miroir** : `φ[-1][j] = φ[0][j]`, `φ[n][j] = φ[n-1][j]`, `φ[i][-1] = φ[i][0]`, `φ[i][n] = φ[i][n-1]`, et
//! `φ[-1][-1] = φ[0][0]` (de même aux autres coins), de même pour `u` (bord sans flux, qui respecte les huit symétries du
//! carré). Un pas lit **seulement l'ancien état** et écrit le nouveau. **Le pas est le même pour les deux domaines** : `Domaine` ne
//! change que la graine ronde, la position de la pointe et un octet de l'écriture canonique. Les coins de l'anneau miroir n'entrent dans
//! aucun résultat tant que rien ne déborde (ils ne servent qu'au gradient transverse d'une face de bord, dont le gradient normal est
//! nul, donc le flux aussi) : la règle des coins n'est là que pour que la description soit complète.
//!
//! Constantes, calculées **une fois** en `f64`, dans cet ordre exact d'opérations (une grandeur n'est jamais recalculée sous
//! une autre écriture : c'est ce qui garde les rotations) :
//!
//! ```text
//! ig  = 1 / dx            ig2 = 0.5 * ig        ig4 = 0.25 * ig
//! w2  = w0 * w0           c1  = 1 - 3 * eps     c2  = 4 * eps        k16 = 16 * eps
//! r0  = dt / tau0         c_u = (d * dt) / (dx * dx)
//! ```
//!
//! **Le facteur `H`.** Pour deux réels `s >= 0` (le carré de la composante du gradient dans la direction du flux) et `t >= 0`
//! (le carré de l'autre composante) : `r = s + t` ; si `r < GARDE` (`1e-24`), `H(s, t) = 1` (la garde de `|∇φ| = 0`, voir
//! plus bas) ; sinon, avec `rr = r * r` :
//!
//! ```text
//! a = c1 + (c2 * ((s * s) + (t * t))) / rr
//! b = (k16 * (t * (s - t))) / rr
//! H(s, t) = a * (a + b)
//! ```
//!
//! **Les flux** (de `φ`, aux faces des cases). La face `x` entre `(i, j)` et `(i, j+1)` (`j` de `-1` à `n-1` : les faces des
//! bords sont celles de la case et de sa voisine miroir) :
//!
//! ```text
//! gn = (phi[i][j+1] - phi[i][j]) * ig
//! gt = (( phi[i+1][j] + phi[i+1][j+1] ) - ( phi[i-1][j] + phi[i-1][j+1] )) * ig4       (le gradient en y à la face)
//! Fx[i][j] = (w2 * H(gn * gn, gt * gt)) * gn
//! ```
//!
//! La face `y` entre `(i, j)` et `(i+1, j)` (`i` de `-1` à `n-1`) :
//!
//! ```text
//! gn = (phi[i+1][j] - phi[i][j]) * ig
//! gt = (( phi[i][j+1] + phi[i+1][j+1] ) - ( phi[i][j-1] + phi[i+1][j-1] )) * ig4       (le gradient en x à la face)
//! Fy[i][j] = (w2 * H(gn * gn, gt * gt)) * gn
//! ```
//!
//! **Pour chaque case `(i, j)`**, `ph = phi[i][j]` (l'ancien) :
//!
//! ```text
//! dxx   = Fx[i][j] - Fx[i][j-1]
//! dyy   = Fy[i][j] - Fy[i-1][j]
//! div   = (dxx + dyy) * ig
//! gxc   = (phi[i][j+1] - phi[i][j-1]) * ig2            gyc = (phi[i+1][j] - phi[i-1][j]) * ig2     (gradient au centre)
//! p = gxc * gxc;   q = gyc * gyc;   rs = p + q
//! ac    = 1                                                   si rs < GARDE
//!       = c1 + (c2 * ((p * p) + (q * q))) / (rs * rs)          sinon                                  (a(n) au centre)
//! f     = ph - (ph * ph) * ph
//! qq    = 1 - ph * ph
//! g     = (lambda * u[i][j]) * (qq * qq)
//! rate  = div + (f - g)
//! dphi  = (r0 * rate) / (ac * ac)                              (le `dt / τ(n)`)
//! phi'  = ph + dphi
//! ```
//!
//! et, **si `u` est joué** ([`ModeU::Joue`]) :
//!
//! ```text
//! lap_u = ((u[i][j+1] + u[i][j-1]) + (u[i+1][j] + u[i-1][j])) - 4 * u[i][j]
//! u'    = u[i][j] + (c_u * lap_u + 0.5 * dphi)                 (la chaleur latente : le `½ ∂φ/∂t`, avec le dphi de CE pas)
//! ```
//!
//! si `u` est gelé ([`ModeU::Gele`]) : `u' = u` (rien n'est calculé, les bits ne changent pas). Chaque `*`, `+`, `-`, `/` est
//! une opération `f64` (arrondi au plus proche de l'IEEE 754) **dans l'ordre des parenthèses ci-dessus**, sans `mul_add`, sans
//! factorisation (`ph * (1 - ph * ph)` n'est pas `ph - (ph * ph) * ph`). `4 * x`, `0.5 * x`, `0.25 * x` sont exacts (hors débordement
//! et nombres sous-normaux). La bibliothèque donne les bits d'un naïf écrit dans les tests d'après cette section (`dendrite_egale_au_naif_bit_pour_bit`)
//! et ceux de deux codes Python écrits d'après elle (`empreinte_des_scenarios`, `empreinte_des_scenarios_du_juge`).
//!
//! **Pourquoi les huit symétries tiennent au bit près.** Le flux est écrit en `H(propre², autre²) * gradient propre` avec la même
//! fonction pour `x` et pour `y` (la transposition échange `Fx` et `Fy`, bit pour bit) ; `H` ne dépend que des carrés (les miroirs
//! changent le signe d'un gradient, jamais son carré), et `a` est une somme `(s*s) + (t*t)` de deux termes qu'un échange
//! intervertit (`a + b` et `b + a` sont le même flottant) ; chaque différence antisymétrique `x - y` a pour image `(-y) - (-x)`,
//! exactement `-(x - y)` ; chaque somme que la symétrie échange est une somme de **deux** paires `(a + b) + (c + d)` ; les constantes
//! (`ig`, `w2`…) sont communes aux deux directions. Trois tests montrent la symétrie au bit près, et qu'elle se perd sans ces écritures
//! (`AGENTS.md`, règle 2 ; `tests/etape_p2_symetrie.rs` : `huit_symetries_exactes_depuis_un_etat_tire`,
//! `huit_symetries_exactes_sur_la_graine_ronde`, `sans_paires_perd_les_symetries`).
//!
//! **La garde.** Là où `|∇φ|² < 1e-24` (loin du front : `φ` plat), la direction `n` n'est pas définie : `H = 1` et `ac = 1`
//! (l'isotrope). Là, un flux vaut au plus `w2 · 1e-12` en valeur absolue ; l'écart au modèle que la garde introduit n'est mesuré par
//! aucun test. La garde est faite sur la somme `p + q` (symétrique), jamais sur un `if gn == 0` ni sur une seule composante. **Le seuil
//! est strict** : à `p + q = 1e-24` exactement, c'est le calcul normal (`facteur_h_a_la_main`, `la_garde_au_seuil_exact`).
//!
//! # Le domaine entier et le quart
//!
//! - [`Domaine::Entier`] : `n × n` cases, la graine ronde au centre. **Le centre est au milieu de la grille** : entre quatre cases
//!   si `n` est pair, sur une case si `n` est impair.
//! - [`Domaine::Quart`] : `n × n` cases qui sont **le quart supérieur droit d'un domaine entier de côté `2 n`** : la case
//!   `(0, 0)` est celle qui touche le centre ; les bords `x- ` (`j = -1`) et `y-` (`i = -1`) sont les axes de symétrie : la
//!   voisine miroir y est exactement l'image que le domaine entier donne. **Le quart calculé seul est, au bit près, le quart du
//!   domaine entier de côté `2 n`** (cases `(n + i, n + j)` du domaine entier), tant que l'état de départ du domaine entier est
//!   symétrique : voir `Dendrite::graine_ronde`, qui donne les mêmes bits de départ dans les deux domaines, et le test
//!   `le_quart_est_le_quart_du_domaine_entier`.
//!
//! # La graine ronde
//!
//! [`Dendrite::graine_ronde`] : le centre du point `(x, y)` de la case `(i, j)` est, pour `Entier`, `x = (j - (n-1)/2) * dx`
//! et `y = (i - (n-1)/2) * dx` (`j` et `n - 1` convertis en `f64` : `(n - 1) as f64 / 2.0`, exact ; la soustraction est exacte) ; pour
//! `Quart`, `x = (j + 0.5) * dx`, `y = (i + 0.5) * dx` (le même `|x|` que la case `(n + i, n + j)` de l'entier de côté `2 n`). Puis
//! `ray = sqrt(x * x + y * y)`, `s = (ray - rayon) / (w0 * SQRT_2)` avec `SQRT_2 = std::f64::consts::SQRT_2` (`1.4142135623730951` :
//! écrire la constante de la bibliothèque, clippy refuse le littéral), `φ = 0 - clamp(s, -1, 1)` (une rampe, de pente celle de
//! `-tanh`, que les premiers pas relaxent), et `u = 0 - (delta * ((1 - φ) * 0.5))` (`0` dans le solide, `-delta` dans le
//! liquide). Aucune autre fonction que `sqrt` et `clamp`. La graine construite n'est pas revérifiée (`ValeurDeDepart` n'est pas une
//! erreur de `graine_ronde`). **Les écritures `0 - x` (et non `-x`) donnent `+0.0` quand `x` vaut `+0.0`
//! (`0.0 - 0.0 = +0.0`, tandis que `-(0.0) = -0.0`) : le solide a `u = +0.0` et le centre de la rampe `φ = +0.0`, les mêmes bits
//! en Rust et en Python (le signe d'un zéro est dans l'écriture canonique).**
//!
//! # La stabilité : refuser avant tout calcul
//!
//! **`dt_borne`.** Avec `U = max |u|` de l'état de départ (le plus grand `|u|`, `u.iter().fold(0.0, |m, v| m.max(v.abs()))` ; `0` pour un
//! `u` nul ; pour `graine_ronde`, `U = delta`, même si la graine ne contient aucune case à `u = -delta`), dans cet ordre exact d'opérations :
//!
//! ```text
//! dx2     = dx * dx
//! s_max   = (w0 * w0) * ((1 - eps) * (1 + 15 * eps))       la raideur maximale du terme anisotrope (sur l'axe θ où elle est la
//!                                                          plus grande : W² + (W²)''/2 pour a = 1 + ε cos 4θ)
//! tau_min = tau0 * ((1 - eps) * (1 - eps))
//! dt_phi  = (2 * tau_min) / ((((8 * s_max) / dx2) + 2) + ((1.54 * lambda) * U))
//! dt_u    = 1 / (((4 * d) / dx2) + (lambda / (4 * tau_min)))                       (seulement si u est joué)
//! dt_borne = 0.5 * min(dt_phi, dt_u)           (`dt_phi` seul si u est gelé)
//! ```
//!
//! (les mêmes bornes linéaires que `dt_sur` de P1 : diffusion de `φ` avec l'anisotropie, relaxation près de `φ = ±1`
//! (`∂/∂φ (φ - φ³) = -2`), couplage `λ U` (`1,54` majore `8/(3√3) ≈ 1,5396`, le maximum de `|4φ(1 - φ²)|`), diffusion de `u` en 2D ;
//! `0,5` : la marge.) [`Reglages::verifier`] refuse `dt > dt_borne` (`dt = dt_borne` est accepté).
//!
//! **Ce que la borne promet, à l'égalité mesurée seulement.** À `dt = dt_borne` exactement (le plus grand pas accepté), aucune
//! erreur `Divergence` n'a été vue, et `|φ|` n'a pas dépassé `max(1, |φ| de départ)` de plus de `1e-12` aux relevés (tous les 100 pas), sur les réglages tirés du test
//! `la_zone_sure_sans_erreur_ni_derive` (`tests/etape_p2_refus.rs` : la plage tirée y est écrite). **C'est une mesure, pas une
//! démonstration** ; `MODELE.md` donne les mesures faites hors des tests (une plage plus large, la dendrite pleine). Au-delà de la borne
//! la doc ne promet rien, et **le `0,5` n'est pas un facteur 2 de réserve** : hors des tests, sur 25 600 états tirés, aucune divergence
//! à `1,1 dt_borne`, 8 à `1,25 dt_borne`, 273 à `1,5 dt_borne`.
//!
//! # La position de la pointe
//!
//! [`Dendrite::position_pointe`] (domaine `Quart` seulement, `None` sinon) : sur la ligne `i = 0` (l'axe), `j` est le plus
//! petit indice (de `0` à `n - 2`) avec `φ[0][j] > 0` (strict) et `φ[0][j+1] <= 0` (large) ; la position est
//! `((j as f64 + 0.5) * dx) + dx * (φ[0][j] / (φ[0][j] - φ[0][j+1]))` (interpolation linéaire du zéro de `φ`, dans cet ordre) ;
//! `None` s'il n'y a pas de `j`.
//!
//! # L'écriture canonique et l'empreinte
//!
//! Les octets, dans l'ordre : l'étiquette ASCII `P2DEN\n` (6 octets) ; `n` en `u64` petit-boutiste ; le nombre de pas faits en
//! `u64` petit-boutiste ; un octet de domaine (`0` : entier, `1` : quart) ; un octet de mode (`0` : `u` gelé, `1` : joué) ; puis les
//! `n²` valeurs de `φ` (ligne par ligne), puis les `n²` valeurs de `u`, chacune `to_bits()` sur 8 octets petit-boutistes.
//! L'empreinte est `empreinte::sha256_hex` de ces octets.
//!
//! # Les erreurs, et jamais de panique
//!
//! **Un réglage est valide** si `w0`, `tau0`, `lambda`, `d`, `dx`, `dt` sont finis et `> 0`, et si `epsilon` est fini, `>= 0` et
//! `< 1.0 / 15.0` ; le premier réglage invalide, dans l'ordre `w0`, `tau0`, `lambda`, `d`, `epsilon`, `dx`, `dt`, est rendu par
//! [`ErreurDendrite::ParametreInvalide`]. **`Reglages::verifier(u_max, mode)`** : un réglage invalide, puis `PasInstable` si
//! `dt > dt_borne(u_max, mode)` (`u_max` lui-même n'est pas vérifié).
//!
//! **`Dendrite::new`**, dans cet ordre (la première règle fausse est celle qu'on rend) : un réglage invalide (les sept, `dt` compris) ;
//! `GrilleTropPetite(n)` si `n < 3` ; `GrilleTropGrande(n)` si `n > N_MAX` (`N_MAX` est permis) ; `LongueursDifferentes` si `phi` ou `u`
//! n'a pas `n * n` valeurs ; `ValeurDeDepart` si une valeur de `phi` ou de `u` n'est pas finie (`u` aussi quand il est gelé), ou si un
//! `|φ|` dépasse `1 + MARGE` (`MARGE = 0.001`, incluse : `|φ| = 1 + MARGE` est accepté) ; `PasInstable` si `dt > dt_borne` (avec le `U`
//! de l'état de départ). **`Dendrite::graine_ronde`**, dans cet ordre : un réglage invalide (les sept, `dt` compris), `GrilleTropPetite(n)`,
//! `GrilleTropGrande(n)`, `ParametreInvalide("delta")` (`delta` fini et `> 0`), `ParametreInvalide("rayon")` (`rayon` fini et `> 0`),
//! puis `PasInstable` (avec `U = delta`). (Un `dt` non valide (`<= 0`, NaN, infini) est donc un `ParametreInvalide("dt")` au rang des
//! réglages ; un `dt` valide mais trop grand est un `PasInstable`, le dernier de la liste.) Aucun de ces refus ne fait paniquer, même
//! pour des réglages démesurés (`1e200`, `1e-200` : `reglages_demesures_sans_panique`) : le refus du pas se lit `!(dt <= dt_borne)`
//! (une borne NaN refuse donc tout `dt`), `min` est `f64::min` (qui rend l'autre opérande quand l'un est NaN : un `dt_phi` NaN est
//! ignoré quand `u` est joué).
//!
//! **`avancer`** rend `Divergence` si un `φ` neuf n'est pas fini ou dépasse `1 + MARGE` en valeur absolue (le refus se lit
//! `!(|φ| <= 1 + MARGE)` : un NaN est refusé, `|φ| = 1 + MARGE` exactement ne l'est pas), ou si, `u` étant joué, un `u` neuf n'est pas
//! fini (`u` gelé n'est pas regardé) : l'état est celui d'avant le pas fautif (le pas n'est pas compté ; les pas faits avant dans le
//! même appel restent faits, avec leur état). `avancer(0)` ne fait rien. Sur les réglages tirés de « La stabilité », aucune divergence
//! n'arrive à `dt = dt_borne` ; **ce n'est pas vrai de tout état accepté** : des réglages ou des valeurs démesurés mais finis peuvent
//! être acceptés puis rendre `Divergence` au premier pas (`reglages_demesures_sans_panique` : `w0 = dx = 1e200` avec `u` joué,
//! `u = 1e308` joué, `dx = 1e-150`).

use crate::empreinte::sha256_hex;

/// `a1 = 5√2/8` (modèle 2 de Kim et coll., table I : `5/(4√2)`).
pub const A1: f64 = 0.883_883_476_483_184_4;
/// `a2 = 47/75` (modèle 2 de Kim et coll., table I).
pub const A2: f64 = 47.0 / 75.0;
/// Côté maximal d'une grille (`n <= N_MAX` : `n * n` ne déborde pas un `usize`, même sur 32 bits, et un côté démesuré est refusé avant
/// toute allocation). À `N_MAX`, un champ pèse 2 Gio : `graine_ronde` peut encore manquer de mémoire, ce qu'aucun test ne regarde.
pub const N_MAX: usize = 16384;
/// Seuil de la garde : `|∇φ|² < GARDE` (voir la doc du module).
pub const GARDE: f64 = 1e-24;
/// Marge de `φ` : `|φ| <= 1 + MARGE` (voir la doc du module). Mesurée : à `dt = dt_borne`, `|φ|` ne dépasse pas
/// `max(1, |φ| de départ)` (le test `la_zone_sure_sans_erreur_ni_derive` tolère `1e-12` aux relevés, tous les 100 pas, sur 480 états,
/// et a vu 0 ; hors des tests : nul sur
/// 51 200 états tirés, 3000 pas, et, en dendrite pleine, `|φ| - 1` inférieur à `5e-13` à `dx` = 0,8 jusqu'à `t = 1500` et à `dx` = 0,4
/// jusqu'à `t = 1100`). `0,001` vaut au moins `10^9` fois ce dernier dépassement. Un pas qui franchit la marge est refusé avec l'état
/// d'avant (`un_pas_qui_franchit_la_marge_est_refuse_avec_l_etat_d_avant`).
pub const MARGE: f64 = 0.001;

/// Pourquoi un réglage ou un état de départ est refusé, ou pourquoi un pas diverge.
#[derive(Clone, Debug, PartialEq)]
pub enum ErreurDendrite {
    /// Moins de 3 cases de côté (`n` donné).
    GrilleTropPetite(usize),
    /// Plus de `N_MAX` cases de côté (`n` donné).
    GrilleTropGrande(usize),
    /// `phi` ou `u` n'a pas `n * n` valeurs.
    LongueursDifferentes,
    /// Un réglage n'est pas valide (`w0`, `tau0`, `lambda`, `d`, `dx`, `dt` : fini et `> 0` ; `epsilon` : fini et dans
    /// `[0, 1/15[`, la raideur anisotrope devant rester positive) ; le nom du réglage.
    ParametreInvalide(&'static str),
    /// `dt > dt_borne` : le pas donné et la borne.
    PasInstable { dt: f64, dt_borne: f64 },
    /// Une valeur de départ n'est pas finie, ou `|φ| > 1 + MARGE`.
    ValeurDeDepart,
    /// Pendant `avancer` : une valeur neuve n'est pas finie, ou `|φ| > 1 + MARGE`. L'état est celui d'avant le pas fautif.
    Divergence,
}

/// Ce que devient `u` pendant un pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeU {
    /// `u` reste ce qu'il est.
    Gele,
    /// `u` suit sa propre équation de diffusion avec la chaleur latente.
    Joue,
}

/// Le domaine calculé (voir la doc du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domaine {
    Entier,
    Quart,
}

/// Les réglages : `w0` (`W0`), `tau0` (`τ0`), `lambda` (`λ`), `d` (`D`), `epsilon` (`ε`), `dx`, `dt`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reglages {
    pub w0: f64,
    pub tau0: f64,
    pub lambda: f64,
    pub d: f64,
    pub epsilon: f64,
    pub dx: f64,
    pub dt: f64,
}

impl Reglages {
    /// Les réglages de cinétique nulle pour une longueur capillaire `d0` : `lambda = (A1 * w0) / d0` et
    /// `tau0 = (lambda * (A2 * (w0 * w0))) / d` (`β = 0`), dans cet ordre d'opérations.
    pub fn depuis_capillarite(w0: f64, d0: f64, d: f64, epsilon: f64, dx: f64, dt: f64) -> Reglages {
        let lambda = (A1 * w0) / d0;
        let tau0 = (lambda * (A2 * (w0 * w0))) / d;
        Reglages { w0, tau0, lambda, d, epsilon, dx, dt }
    }

    /// Le pas maximal accepté (voir « La stabilité ») pour `u_max = max |u|` de l'état de départ.
    pub fn dt_borne(&self, u_max: f64, mode: ModeU) -> f64 {
        let dx2 = self.dx * self.dx;
        let s_max = (self.w0 * self.w0) * ((1.0 - self.epsilon) * (1.0 + 15.0 * self.epsilon));
        let tau_min = self.tau0 * ((1.0 - self.epsilon) * (1.0 - self.epsilon));
        let dt_phi = (2.0 * tau_min) / ((((8.0 * s_max) / dx2) + 2.0) + ((1.54 * self.lambda) * u_max));
        let dt_u = 1.0 / (((4.0 * self.d) / dx2) + (self.lambda / (4.0 * tau_min)));
        match mode {
            ModeU::Gele => 0.5 * dt_phi,
            ModeU::Joue => 0.5 * dt_phi.min(dt_u),
        }
    }

    /// Vérifie les réglages (les erreurs et leur ordre : la doc du module ; `dt_borne` avec `u_max`).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn verifier(&self, u_max: f64, mode: ModeU) -> Result<(), ErreurDendrite> {
        verifier_les_sept(self)?;
        let b = self.dt_borne(u_max, mode);
        if !(self.dt <= b) {
            return Err(ErreurDendrite::PasInstable { dt: self.dt, dt_borne: b });
        }
        Ok(())
    }
}

/// Le facteur `H(s, t)` de la doc du module.
#[inline(always)]
pub fn facteur_h(s: f64, t: f64, c1: f64, c2: f64, k16: f64) -> f64 {
    let r = s + t;
    if r < GARDE {
        return 1.0;
    }
    let rr = r * r;
    let a = c1 + (c2 * ((s * s) + (t * t))) / rr;
    let b = (k16 * (t * (s - t))) / rr;
    a * (a + b)
}

/// La dendrite : l'état `φ`, `u`, le nombre de pas faits, et le schéma de la doc du module.
#[derive(Clone, Debug)]
pub struct Dendrite {
    r: Reglages,
    n: usize,
    domaine: Domaine,
    mode: ModeU,
    pas: u64,
    phi: Vec<f64>,
    u: Vec<f64>,
}

impl Dendrite {
    /// Construit l'état de départ depuis des champs donnés (voir « Les erreurs »). Zéro pas fait.
    pub fn new(
        r: Reglages,
        n: usize,
        domaine: Domaine,
        phi: &[f64],
        u: &[f64],
        mode: ModeU,
    ) -> Result<Dendrite, ErreurDendrite> {
        verifier_les_sept(&r)?;
        if n < 3 {
            return Err(ErreurDendrite::GrilleTropPetite(n));
        }
        if n > N_MAX {
            return Err(ErreurDendrite::GrilleTropGrande(n));
        }
        if phi.len() != n * n || u.len() != n * n {
            return Err(ErreurDendrite::LongueursDifferentes);
        }
        if phi.iter().any(|v| !v.is_finite() || v.abs() > 1.0 + MARGE) || u.iter().any(|v| !v.is_finite()) {
            return Err(ErreurDendrite::ValeurDeDepart);
        }
        let u_max = u.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        r.verifier(u_max, mode)?;
        Ok(Dendrite { r, n, domaine, mode, pas: 0, phi: phi.to_vec(), u: u.to_vec() })
    }

    /// La graine ronde de la doc du module (`delta > 0` fini : `ParametreInvalide("delta")` ; `rayon > 0` fini :
    /// `ParametreInvalide("rayon")`, après les erreurs de réglage et de grille).
    pub fn graine_ronde(
        r: Reglages,
        n: usize,
        domaine: Domaine,
        delta: f64,
        rayon: f64,
        mode: ModeU,
    ) -> Result<Dendrite, ErreurDendrite> {
        verifier_les_sept(&r)?;
        if n < 3 {
            return Err(ErreurDendrite::GrilleTropPetite(n));
        }
        if n > N_MAX {
            return Err(ErreurDendrite::GrilleTropGrande(n));
        }
        if !(delta.is_finite() && delta > 0.0) {
            return Err(ErreurDendrite::ParametreInvalide("delta"));
        }
        if !(rayon.is_finite() && rayon > 0.0) {
            return Err(ErreurDendrite::ParametreInvalide("rayon"));
        }
        r.verifier(delta, mode)?;
        let centre = (n - 1) as f64 / 2.0;
        let mut phi = Vec::with_capacity(n * n);
        let mut u = Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                let (x, y) = match domaine {
                    Domaine::Quart => ((j as f64 + 0.5) * r.dx, (i as f64 + 0.5) * r.dx),
                    Domaine::Entier => ((j as f64 - centre) * r.dx, (i as f64 - centre) * r.dx),
                };
                let ray = (x * x + y * y).sqrt();
                let s = (ray - rayon) / (r.w0 * std::f64::consts::SQRT_2);
                let p = 0.0 - s.clamp(-1.0, 1.0);
                phi.push(p);
                u.push(0.0 - (delta * ((1.0 - p) * 0.5)));
            }
        }
        Ok(Dendrite { r, n, domaine, mode, pas: 0, phi, u })
    }

    /// Le côté `n`.
    pub fn n(&self) -> usize {
        self.n
    }
    /// Les réglages.
    pub fn reglages(&self) -> Reglages {
        self.r
    }
    /// Le domaine.
    pub fn domaine(&self) -> Domaine {
        self.domaine
    }
    /// Le mode de `u`.
    pub fn mode(&self) -> ModeU {
        self.mode
    }
    /// `φ`, ligne par ligne.
    pub fn phi(&self) -> &[f64] {
        &self.phi
    }
    /// `u`, ligne par ligne.
    pub fn u(&self) -> &[f64] {
        &self.u
    }
    /// Le nombre de pas faits.
    pub fn pas_faits(&self) -> u64 {
        self.pas
    }
    /// Le temps écoulé : `pas_faits() as f64 * dt`.
    pub fn temps(&self) -> f64 {
        self.pas as f64 * self.r.dt
    }

    /// Fait `pas` pas de temps (le schéma de la doc du module).
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn avancer(&mut self, pas: u64) -> Result<(), ErreurDendrite> {
        let n = self.n;
        let ig = 1.0 / self.r.dx;
        let ig2 = 0.5 * ig;
        let ig4 = 0.25 * ig;
        let w2 = self.r.w0 * self.r.w0;
        let c1 = 1.0 - 3.0 * self.r.epsilon;
        let c2 = 4.0 * self.r.epsilon;
        let k16 = 16.0 * self.r.epsilon;
        let r0 = self.r.dt / self.r.tau0;
        let c_u = (self.r.d * self.r.dt) / (self.r.dx * self.r.dx);
        let joue = self.mode == ModeU::Joue;
        let nf = n + 1;
        for _ in 0..pas {
            let mut fx = vec![0.0_f64; n * nf];
            for i in 0..n {
                let im = miroir(i as isize - 1, n);
                let ip = miroir(i as isize + 1, n);
                let base = i * n;
                let base_m = im * n;
                let base_p = ip * n;
                let base_f = i * nf;
                for jj in 0..=n {
                    let j0 = miroir(jj as isize - 1, n);
                    let j1 = miroir(jj as isize, n);
                    let gn = (self.phi[base + j1] - self.phi[base + j0]) * ig;
                    let gt = ((self.phi[base_p + j0] + self.phi[base_p + j1])
                        - (self.phi[base_m + j0] + self.phi[base_m + j1]))
                        * ig4;
                    fx[base_f + jj] = (w2 * facteur_h(gn * gn, gt * gt, c1, c2, k16)) * gn;
                }
            }
            let mut fy = vec![0.0_f64; nf * n];
            for ii in 0..=n {
                let i0 = miroir(ii as isize - 1, n);
                let i1 = miroir(ii as isize, n);
                let base0 = i0 * n;
                let base1 = i1 * n;
                let base_f = ii * n;
                for j in 0..n {
                    let jm = miroir(j as isize - 1, n);
                    let jp = miroir(j as isize + 1, n);
                    let gn = (self.phi[base1 + j] - self.phi[base0 + j]) * ig;
                    let gt = ((self.phi[base0 + jp] + self.phi[base1 + jp])
                        - (self.phi[base0 + jm] + self.phi[base1 + jm]))
                        * ig4;
                    fy[base_f + j] = (w2 * facteur_h(gn * gn, gt * gt, c1, c2, k16)) * gn;
                }
            }
            let mut nphi = vec![0.0_f64; n * n];
            let mut nu = if joue { vec![0.0_f64; n * n] } else { Vec::new() };
            for i in 0..n {
                let im = miroir(i as isize - 1, n);
                let ip = miroir(i as isize + 1, n);
                let ligne = i * n;
                let ligne_m = im * n;
                let ligne_p = ip * n;
                for j in 0..n {
                    let jm = miroir(j as isize - 1, n);
                    let jp = miroir(j as isize + 1, n);
                    let k = ligne + j;
                    let ph = self.phi[k];
                    let dxx = fx[i * nf + (j + 1)] - fx[i * nf + j];
                    let dyy = fy[(i + 1) * n + j] - fy[i * n + j];
                    let div = (dxx + dyy) * ig;
                    let gxc = (self.phi[ligne + jp] - self.phi[ligne + jm]) * ig2;
                    let gyc = (self.phi[ligne_p + j] - self.phi[ligne_m + j]) * ig2;
                    let p = gxc * gxc;
                    let q = gyc * gyc;
                    let rs = p + q;
                    let ac = if rs < GARDE { 1.0 } else { c1 + (c2 * ((p * p) + (q * q))) / (rs * rs) };
                    let f = ph - (ph * ph) * ph;
                    let qq = 1.0 - ph * ph;
                    let g = (self.r.lambda * self.u[k]) * (qq * qq);
                    let rate = div + (f - g);
                    let dphi = (r0 * rate) / (ac * ac);
                    nphi[k] = ph + dphi;
                    if joue {
                        let e = self.u[ligne + jp];
                        let o = self.u[ligne + jm];
                        let no = self.u[ligne_m + j];
                        let su = self.u[ligne_p + j];
                        let lap_u = ((e + o) + (no + su)) - 4.0 * self.u[k];
                        nu[k] = self.u[k] + (c_u * lap_u + 0.5 * dphi);
                    }
                }
            }
            if nphi.iter().any(|p| !(p.abs() <= 1.0 + MARGE)) {
                return Err(ErreurDendrite::Divergence);
            }
            if joue && nu.iter().any(|v| !v.is_finite()) {
                return Err(ErreurDendrite::Divergence);
            }
            self.phi = nphi;
            if joue {
                self.u = nu;
            }
            self.pas = self.pas.saturating_add(1);
        }
        Ok(())
    }

    /// La position de la pointe (voir la doc du module).
    pub fn position_pointe(&self) -> Option<f64> {
        if self.domaine != Domaine::Quart {
            return None;
        }
        let dx = self.r.dx;
        let derniere = self.n.saturating_sub(1);
        for j in 0..derniere {
            if self.phi[j] > 0.0 && self.phi[j + 1] <= 0.0 {
                return Some(((j as f64 + 0.5) * dx) + dx * (self.phi[j] / (self.phi[j] - self.phi[j + 1])));
            }
        }
        None
    }

    /// L'écriture canonique (voir la doc du module).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"P2DEN\n");
        o.extend_from_slice(&(self.n as u64).to_le_bytes());
        o.extend_from_slice(&self.pas.to_le_bytes());
        o.push(match self.domaine {
            Domaine::Entier => 0,
            Domaine::Quart => 1,
        });
        o.push(match self.mode {
            ModeU::Gele => 0,
            ModeU::Joue => 1,
        });
        for v in &self.phi {
            o.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        for v in &self.u {
            o.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        o
    }

    /// `sha256_hex(ecriture_canonique())`.
    pub fn empreinte(&self) -> String {
        sha256_hex(&self.ecriture_canonique())
    }
}

/// `v` est un réglage positif : fini et `> 0`.
fn reglage_positif(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

/// `epsilon` est valide : fini, `>= 0` et `< 1/15` (la raideur anisotrope reste positive).
#[allow(clippy::manual_range_contains)]
fn epsilon_valide(e: f64) -> bool {
    e.is_finite() && e >= 0.0 && e < 1.0 / 15.0
}

/// Les sept réglages, dans l'ordre de la doc du module : `w0`, `tau0`, `lambda`, `d`, `epsilon`, `dx`, `dt`.
fn verifier_les_sept(r: &Reglages) -> Result<(), ErreurDendrite> {
    for (nom, valeur) in [("w0", r.w0), ("tau0", r.tau0), ("lambda", r.lambda), ("d", r.d)] {
        if !reglage_positif(valeur) {
            return Err(ErreurDendrite::ParametreInvalide(nom));
        }
    }
    if !epsilon_valide(r.epsilon) {
        return Err(ErreurDendrite::ParametreInvalide("epsilon"));
    }
    for (nom, valeur) in [("dx", r.dx), ("dt", r.dt)] {
        if !reglage_positif(valeur) {
            return Err(ErreurDendrite::ParametreInvalide(nom));
        }
    }
    Ok(())
}

/// L'indice miroir : un indice hors de `[0, n - 1]` ramené sur la case du bord (`n >= 3`).
fn miroir(k: isize, n: usize) -> usize {
    k.clamp(0, n as isize - 1) as usize
}

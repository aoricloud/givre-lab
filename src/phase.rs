//! Le champ de phase des étapes P (étape P1 : le front droit en 1D) : le modèle de Karma et Rappel sans anisotropie, en
//! différences finies explicites, calculé deux fois par le même schéma, en `f64` ([`Front1d`]) et en virgule fixe Q31.32
//! ([`Front1dFixe`]), et la diffusion 2D de l'essai de la règle des nombres ([`Diffusion2d`]).
//!
//! # Le modèle
//!
//! Source : A. Karma et W.-J. Rappel, « Quantitative phase-field modeling of dendritic growth in two and three
//! dimensions », Phys. Rev. E 57, 4323 (1998) (l'article lui-même n'a pas pu être ouvert : il n'est pas sur arXiv), dans la
//! forme sans anisotropie reprise (équations 41 et 42, et le profil au repos) par Karma et Rappel, « Phase-Field Model of Dendritic
//! Sidebranching with Thermal Noise », Phys. Rev. E 60, 3614 (1999), arXiv cond-mat/9902017, et (équations 4 et 5) par Kim,
//! Provatas, Goldenfeld et Dantzig, « Universal Dynamics of Phase-Field Models for Dendritic Growth », arXiv cond-mat/9810189.
//! Chaque case porte une phase `φ` (`+1` solide, `-1` liquide) et une température sans dimension
//! `u` (`u < 0` : liquide surfondu). En unités de longueur `W` et de temps `τ` :
//!
//! ```text
//! τ ∂φ/∂t = W² ∂²φ/∂x² + φ - φ³ - λ u (1 - φ²)²
//! ∂u/∂t   = D ∂²u/∂x² + ½ ∂φ/∂t
//! ```
//!
//! Au repos (`u = 0`) le profil du front est `φ0(x) = -tanh(x / (W√2))` (solide, `φ = +1`, du côté `x < 0`). L'analyse de
//! l'interface mince donne la longueur capillaire `d0 = a1 W / λ` et la cinétique `β = a1 (τ / (λ W) - a2 W / D)` (A. Karma,
//! « Phase-Field Formulation for Quantitative Modeling of Alloy Solidification », arXiv cond-mat/0103289, équation 6, qui
//! les dit identiques à celles de l'article de 1998 ; `β = 0` pour `λ = D τ / (a2 W²)`), avec
//! `a1 = I / J = (2√2/3) / (16/15) = 5√2/8 ≈ 0,8839` (`I = ∫φ0'² dx`, `J = ∫(1 - φ0²)² |φ0'| dx`, recalculés à la main
//! dans `references/etape-p1-valeurs.md`) et `a2 = 47/75 ≈ 0,6267` (cond-mat/9810189, table 1 ; 0,6267 dans
//! cond-mat/0103289 ; recalculé dans la fiche). Ces formules ne sont jamais calculées par ce module : elles servent aux
//! tests.
//!
//! # Le schéma, à la lettre
//!
//! **Cette section est la spécification** : une implémentation qui la suit donne EXACTEMENT les mêmes bits.
//!
//! Une grille de `n >= 3` cases, au pas `dx`, la case `i` ayant son centre en `x_i = (i + 1/2) · dx` (le bord gauche de la
//! case 0 est en `x = 0`). Les bords sont **sans flux** : hors de la grille, la case voisine vaut la case du bord
//! (`φ[-1] = φ[0]`, `φ[n] = φ[n-1]`, de même pour `u`). Un pas de temps lit **seulement l'ancien état** (jamais une
//! mise à jour en place) et écrit le nouveau.
//!
//! Constantes, calculées **une fois** en `f64`, dans cet ordre exact d'opérations :
//!
//! ```text
//! r      = dt / tau
//! c_phi  = (w * w) / (dx * dx)
//! c_u    = (d * dt) / (dx * dx)
//! ```
//!
//! (`Front1dFixe` convertit chacune, ainsi que `lambda`, par `Fixe::depuis_f64` ; `2` est `Fixe::entier(2)`, `1` est
//! `Fixe::UN` et `1/2` est `Fixe::depuis_f64(0.5)`.) Pour chaque case `i`, de `0` à `n - 1`, avec `φ`, `u` l'ancien état :
//!
//! ```text
//! lap_phi = (phi[i-1] + phi[i+1]) - 2 * phi[i]
//! f       = phi[i] - (phi[i] * phi[i]) * phi[i]               (φ - φ³)
//! q       = 1 - phi[i] * phi[i]
//! g       = (lambda * u[i]) * (q * q)                          (λ u (1 - φ²)²)
//! vitesse = c_phi * lap_phi + (f - g)
//! dphi    = r * vitesse                                        (le terme `dt/τ · τ ∂φ/∂t`)
//! phi'[i] = phi[i] + dphi
//! ```
//!
//! et, **si `u` est joué** ([`ModeU::Joue`]) :
//!
//! ```text
//! lap_u   = (u[i-1] + u[i+1]) - 2 * u[i]
//! u'[i]   = u[i] + (c_u * lap_u + 0.5 * dphi)                  (la chaleur latente : le `½ ∂φ/∂t`, avec le dphi de CE pas)
//! ```
//!
//! si `u` est gelé ([`ModeU::Gele`]) : `u'[i] = u[i]` (rien n'est calculé, les bits ne changent pas). Chaque `*`, `+`, `-`
//! est une opération du type du calcul (`f64` : l'arrondi au plus proche de l'IEEE 754 ; `Fixe` : les opérateurs de
//! [`crate::math::Fixe`], produits arrondis au plus proche), **dans l'ordre des parenthèses ci-dessus**, sans `mul_add`.
//! `2 * x` et `0.5 * x` valent `x + x` et la moitié de `x` (exacts en `f64`). En `Fixe`, `2 * x` est `Fixe::entier(2) * x`
//! (exact : le double du champ brut) et `0.5 * x` est `Fixe::depuis_f64(0.5) * x` (la moitié du champ brut arrondie au plus
//! proche, demi-cas vers +∞ : `(brut + 1) >> 1`, le même résultat que `x.div_entier(2)`).
//!
//! **Les paires.** Dans chaque somme que la symétrie miroir échange (`phi[i-1] + phi[i+1]`, `u[i-1] + u[i+1]`), les deux
//! voisins sont les deux opérandes d'un même `+`. C'est ce qui rend la symétrie `φ(x) = -φ(-x)` exacte en `f64`
//! (voir `AGENTS.md`, règle 2, et [`Front1d`]).
//!
//! # La stabilité
//!
//! [`Parametres::verifier`] refuse `dt > dt_max` ([`ErreurPhase::PasInstable`]), avec
//! `dt_max = min( dx² / (2 d),  tau · dx² / (2 w² + dx²) )` : la borne linéaire exacte de chacune des deux équations prise
//! seule (la diffusion de `u` ; la diffusion et la relaxation de `φ` près de `φ = ±1`, où `∂/∂φ(φ - φ³) = -2`).
//! **Cette borne est nécessaire, elle n'est pas suffisante** : le couplage des deux équations et un `λ u` grand la
//! resserrent. Mesuré par le juge de P1 (3000 réglages tirés, 8000 pas, un front et un bruit de `1e-6` ; `U = max |u|`) :
//!
//! - à `dt = dt_max`, 28 % des réglages explosent, et 5 % même pour `λ U < 0,5` : par exemple, quand `u` est joué, la chaleur
//!   latente `½ dphi` pousse le mode en dents de scie de `u` au-delà de `-1` (un cas est gelé dans `tests/etape_p1_refus.rs`) ;
//! - à `dt <= dt_max / 2`, aucun des 1833 réglages avec `λ U < 2` ne sort de `[-1, 1]` ; au-delà de `λ U = 2`, certains
//!   explosent (3 sur 162 entre 2 et 3, 157 sur 367 au-delà de 16) ;
//! - à `dt <= dt_sur / 2`, avec
//!   `dt_sur = min( dt_max,  2 tau / (4 w²/dx² + 2 + 1,54 λ U),  [si u est joué] 1 / (2 d/dx² + λ / (4 tau)) )`
//!   (les mêmes bornes linéaires avec le couplage ; `1,54 = 8/(3√3)` est le maximum de `|4φ(1 - φ²)|`), aucun des 3000
//!   réglages n'explose (`λ U` jusqu'à 80). C'est une mesure, pas une démonstration.
//!
//! Une des causes est dans le modèle lui-même : pour `u < 0` et `a = λ|u|`, le solide `φ = +1` n'est stable que jusqu'à
//! `φ* = (1 + √(1 + 4a²)) / (2a) ≈ 1 + 1/(2a)` ; au-delà, `φ` part à l'infini (le terme `(1 - φ²)²` l'emporte sur le cube),
//! et un dépassement numérique de `1/(2a)` suffit. Dans le régime instable, `avancer` rend `Err(Debordement)` (voir plus
//! bas) ou un état fini mais faux (des dents de scie) : jamais de panique.
//!
//! # La position du front
//!
//! [`Front1d::position_front`] : `i` est le plus petit indice avec `φ[i] > 0` et `φ[i+1] <= 0` ; la position est
//! `((i as f64 + 0.5) * dx) + dx * (φ[i] / (φ[i] - φ[i+1]))` (interpolation linéaire du zéro de `φ`, en `f64`, dans cet ordre
//! d'opérations ; pour `Fixe`, les `φ` sont d'abord convertis par `vers_f64`) ; `None` s'il n'y a pas d'`i` (pas de front solide-à-gauche).
//!
//! # L'écriture canonique et l'empreinte
//!
//! [`Front1d::ecriture_canonique`] : les octets, dans l'ordre : l'étiquette ASCII `P1F64\n` (6 octets) pour `Front1d`,
//! `P1FIX\n` pour `Front1dFixe` ; `n` en `u64` petit-boutiste (8 octets) ; le nombre de pas faits en `u64` petit-boutiste ;
//! un octet de mode (`0` : `u` gelé, `1` : `u` joué) ; puis les `n` valeurs de `φ` (de la case 0 à `n-1`), puis les `n`
//! valeurs de `u`, chacune sur 8 octets petit-boutistes : pour `f64`, `to_bits()` ; pour `Fixe`, le champ brut `i64`
//! (complément à deux). L'empreinte est `empreinte::sha256_hex` de ces octets.
//!
//! # Jamais de panique, et le débordement
//!
//! Aucune entrée permise ne fait paniquer : les constructeurs rendent une [`ErreurPhase`], et `avancer` rend
//! `Err(ErreurPhase::Debordement)` au lieu de paniquer. **Le débordement** : pendant un pas, si une opération de
//! `Front1dFixe` sort de `[-2^31, 2^31[` (le calcul de `Mul` de [`crate::math::Fixe`], `brut = (a·b + 2^31) >> 32` en `i128`,
//! doit tenir dans un `i64` ; de même `+` et `-` sur les champs bruts), ou si, après un pas, une valeur neuve de `Front1d`
//! (de `φ`, ou de `u` quand il est joué) n'est pas finie (NaN ou infinie), `avancer` s'arrête **avant d'avoir changé l'état de ce pas** (le pas fautif n'est pas compté ;
//! `phi()`, `u()`, `pas_faits()` sont ceux d'avant lui) et rend l'erreur. Les pas faits avant le pas fautif restent faits.
//! Dans le régime stable (état de départ dans `[-1, 1]`, `|lambda u|` modéré, `dt <= dt_max`) aucun débordement n'arrive.

#![allow(unused_variables, dead_code)]

use crate::math::Fixe;

/// Pourquoi un réglage ou un état de départ est refusé.
#[derive(Clone, Debug, PartialEq)]
pub enum ErreurPhase {
    /// Moins de 3 cases (`n` donné).
    GrilleTropPetite(usize),
    /// `phi` et `u` n'ont pas la même longueur.
    LongueursDifferentes,
    /// Un réglage (`w`, `tau`, `lambda`, `d`, `dx`, `dt`) n'est pas fini et strictement positif, ou une constante dérivée
    /// dépasse `2^20` ; le nom du réglage.
    ParametreInvalide(&'static str),
    /// `dt > dt_max` : le pas donné et le pas maximal.
    PasInstable { dt: f64, dt_max: f64 },
    /// Une valeur de départ n'est pas finie (ni, pour `Front1dFixe`, de valeur absolue `< 2^31`).
    ValeurDeDepart,
    /// Pendant `avancer` : un débordement de la virgule fixe, ou une valeur non finie en `f64` (voir la doc du module). L'état
    /// est celui d'avant le pas fautif.
    Debordement,
}

/// Ce que devient `u` pendant un pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeU {
    /// `u` reste ce qu'il est (la température est imposée : le cas du front poussé).
    Gele,
    /// `u` suit sa propre équation de diffusion avec la chaleur latente.
    Joue,
}

/// Les réglages du schéma : `w` (la largeur `W` de l'interface), `tau` (`τ`), `lambda` (`λ`), `d` (`D`), `dx` (le pas de
/// grille), `dt` (le pas de temps). Tous finis et strictement positifs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parametres {
    pub w: f64,
    pub tau: f64,
    pub lambda: f64,
    pub d: f64,
    pub dx: f64,
    pub dt: f64,
}

impl Parametres {
    /// Le pas maximal stable : `min( dx² / (2 d),  tau · dx² / (2 w² + dx²) )`, calculé dans cet ordre :
    /// `a = (dx * dx) / (2.0 * d)`, `b = (tau * (dx * dx)) / (2.0 * (w * w) + dx * dx)`, `a.min(b)` (le `min` de `f64` : si
    /// l'un des deux est NaN, l'autre est rendu). Les réglages sont supposés valides.
    pub fn dt_max(&self) -> f64 {
        todo!()
    }

    /// Vérifie les réglages, en trois temps, dans cet ordre :
    ///
    /// 1. chacun de `w`, `tau`, `lambda`, `d`, `dx`, `dt` (dans cet ordre) est fini et `> 0`, sinon `ParametreInvalide(nom)` ;
    /// 2. chacune des constantes `c_phi`, `c_u`, `r` (celles de la doc du module, calculées de la même façon) puis `lambda`,
    ///    dans cet ordre, vérifie `valeur <= 2^20` (= 1 048 576 : `2^20` lui-même est accepté, le flottant suivant est
    ///    refusé), sinon `ParametreInvalide("c_phi")`, `("c_u")`, `("r")` ou `("lambda")`. Une constante NaN est refusée
    ///    (`valeur <= 2^20` est faux) : par exemple `c_phi` quand `w * w` et `dx * dx` débordent tous deux ;
    /// 3. `dt <= dt_max()` est vrai, sinon `PasInstable { dt, dt_max }` (un `dt_max()` NaN est donc refusé aussi).
    ///
    /// Le pas `dt = dt_max()` est accepté : la borne est nécessaire, pas suffisante (voir « La stabilité » dans la doc du
    /// module).
    pub fn verifier(&self) -> Result<(), ErreurPhase> {
        todo!()
    }
}

/// Le front 1D en `f64` : l'état `φ`, `u`, le nombre de pas faits, et le schéma de la doc du module.
///
/// **La symétrie.** Si l'état de départ est pair (`phi[n-1-i] = phi[i]` et `u[n-1-i] = u[i]`, au bit près) ou impair
/// (`phi[n-1-i] = -phi[i]` et `u[n-1-i] = -u[i]`), il le reste, **au bit près**, après chaque pas, que `u` soit gelé ou
/// joué : chaque somme que le miroir échange est écrite par paires, la négation est exacte et l'arrondi au plus proche est
/// symétrique. Une seule réserve, pour le miroir impair quand `n` est impair : la case du milieu vaut `+0.0`, et son image
/// attendue `-(+0.0) = -0.0` a d'autres bits (les deux sont égaux par `==`) ; le miroir pair n'a pas cette réserve.
#[derive(Clone, Debug)]
pub struct Front1d {
    #[allow(dead_code)]
    p: Parametres,
    #[allow(dead_code)]
    phi: Vec<f64>,
    #[allow(dead_code)]
    u: Vec<f64>,
    #[allow(dead_code)]
    pas: u64,
    #[allow(dead_code)]
    mode: ModeU,
}

impl Front1d {
    /// Construit l'état de départ : `phi` et `u` (même longueur, `>= 3`). Erreurs, dans cet ordre :
    /// celles de [`Parametres::verifier`] ; `GrilleTropPetite(n)` si `phi.len() < 3` ; `LongueursDifferentes` ;
    /// `ValeurDeDepart` si une valeur n'est pas finie. Zéro pas fait.
    pub fn new(p: Parametres, phi: &[f64], u: &[f64], mode: ModeU) -> Result<Front1d, ErreurPhase> {
        todo!()
    }

    /// Fait `pas` pas de temps (le schéma de la doc du module) ; `Err(Debordement)` si une valeur n'est plus finie (voir la
    /// doc du module : l'état est celui d'avant le pas fautif).
    pub fn avancer(&mut self, pas: u64) -> Result<(), ErreurPhase> {
        todo!()
    }

    /// Le nombre de pas faits depuis le départ.
    pub fn pas_faits(&self) -> u64 {
        todo!()
    }

    /// Le temps écoulé : `pas_faits() as f64 * dt` (un produit, dans cet ordre).
    pub fn temps(&self) -> f64 {
        todo!()
    }

    /// La phase `φ`, case 0 à `n - 1`.
    pub fn phi(&self) -> &[f64] {
        todo!()
    }

    /// La température `u`, case 0 à `n - 1`.
    pub fn u(&self) -> &[f64] {
        todo!()
    }

    /// La position du front (voir la doc du module), ou `None`.
    pub fn position_front(&self) -> Option<f64> {
        todo!()
    }

    /// L'écriture canonique (voir la doc du module).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        todo!()
    }

    /// `sha256_hex(ecriture_canonique())`.
    pub fn empreinte(&self) -> String {
        todo!()
    }
}

/// Le même front en virgule fixe Q31.32 : les mêmes formules, avec les opérateurs de [`Fixe`] (le produit est arrondi au
/// plus proche, chaque produit intermédiaire l'est : l'ordre des parenthèses de la doc du module compte). Les valeurs de
/// départ sont converties par `Fixe::depuis_f64`.
///
/// **Ce que la virgule fixe ne garde pas.** Le miroir pair reste exact (champs bruts égaux). Le miroir impair ne l'est
/// pas : le produit de `Fixe` arrondit les demi-cas vers +∞, donc `(-x) * y` n'est pas toujours `-(x * y)` ; dès que `u`
/// est joué, `0.5 * dphi` est un demi-cas une fois sur deux et le miroir impair est perdu au premier pas. Et l'arrondi de
/// chaque incrément (au pas de `2^-32`) s'accumule : l'écart à `Front1d` croît comme `1 / dt` (mesuré au repos, `dx = 0,25` :
/// `8e-9` à `dt/tau = 1e-2`, `1e-5` à `dt/tau = 1e-5`).
#[derive(Clone, Debug)]
pub struct Front1dFixe {
    #[allow(dead_code)]
    p: Parametres,
    #[allow(dead_code)]
    phi: Vec<Fixe>,
    #[allow(dead_code)]
    u: Vec<Fixe>,
    #[allow(dead_code)]
    pas: u64,
    #[allow(dead_code)]
    mode: ModeU,
}

impl Front1dFixe {
    /// Comme [`Front1d::new`] ; en plus, `ValeurDeDepart` si une valeur de départ est de valeur absolue `>= 2^31`.
    pub fn new(p: Parametres, phi: &[f64], u: &[f64], mode: ModeU) -> Result<Front1dFixe, ErreurPhase> {
        todo!()
    }

    /// Fait `pas` pas de temps ; `Err(Debordement)` si une opération sort de `[-2^31, 2^31[` (voir la doc du module).
    pub fn avancer(&mut self, pas: u64) -> Result<(), ErreurPhase> {
        todo!()
    }

    /// Le nombre de pas faits.
    pub fn pas_faits(&self) -> u64 {
        todo!()
    }

    /// Le temps écoulé : `pas_faits() as f64 * dt`.
    pub fn temps(&self) -> f64 {
        todo!()
    }

    /// La phase, case 0 à `n - 1`, en virgule fixe.
    pub fn phi(&self) -> &[Fixe] {
        todo!()
    }

    /// La température, case 0 à `n - 1`, en virgule fixe.
    pub fn u(&self) -> &[Fixe] {
        todo!()
    }

    /// La position du front (voir la doc du module).
    pub fn position_front(&self) -> Option<f64> {
        todo!()
    }

    /// L'écriture canonique (voir la doc du module : l'étiquette est `P1FIX\n`).
    pub fn ecriture_canonique(&self) -> Vec<u8> {
        todo!()
    }

    /// `sha256_hex(ecriture_canonique())`.
    pub fn empreinte(&self) -> String {
        todo!()
    }
}

/// La diffusion seule en 2D (`∂c/∂t = ∇²c`, laplacien à cinq points) sur une grille carrée `n × n`, en `f64` : l'essai
/// de la règle des nombres (`AGENTS.md`, règle 2).
///
/// La case `(i, j)` (`i` la ligne, `j` la colonne ; stockage ligne par ligne, `c[i * n + j]`) a quatre voisines : est
/// `(i, j+1)`, ouest `(i, j-1)`, nord `(i-1, j)`, sud `(i+1, j)` ; hors de la grille, la voisine est la case elle-même
/// (bord sans flux : il respecte les huit symétries du carré). Un pas, sur l'ancien état seulement, pour chaque case :
///
/// ```text
/// somme = (est + ouest) + (nord + sud)          (par paires : chaque symétrie du carré échange les deux opérandes d'un `+`)
/// c'    = c + k * (somme - 4 * c)
/// ```
///
/// Les huit symétries du carré (rotations d'un quart de tour, miroirs) laissent alors un état symétrique **identique au bit
/// près** à chaque pas.
#[derive(Clone, Debug)]
pub struct Diffusion2d {
    #[allow(dead_code)]
    n: usize,
    #[allow(dead_code)]
    k: f64,
    #[allow(dead_code)]
    c: Vec<f64>,
}

impl Diffusion2d {
    /// Construit la grille de côté `n` (`>= 2`) avec `depart` (`n * n` valeurs finies, ligne par ligne) et `k` (`D dt / dx²`,
    /// fini, `> 0` et `<= 0,25`). Erreurs : `GrilleTropPetite(n)` si `n < 2` ; `LongueursDifferentes` si
    /// `depart.len() != n * n` (aussi quand `n * n` déborde un `usize` : jamais de panique, `n.checked_mul(n)`) ;
    /// `ParametreInvalide("k")` ; `ValeurDeDepart`. Dans cet ordre.
    pub fn new(n: usize, k: f64, depart: &[f64]) -> Result<Diffusion2d, ErreurPhase> {
        todo!()
    }

    /// Fait un pas (la doc du type).
    pub fn pas(&mut self) {
        todo!()
    }

    /// Les valeurs, ligne par ligne.
    pub fn cellules(&self) -> &[f64] {
        todo!()
    }
}

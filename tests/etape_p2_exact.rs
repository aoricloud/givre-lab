//! Étape P2 : les cas à solution connue de la dendrite 2D : le facteur `H` et un pas isotrope calculés à la main, et le front droit le
//! long d'un axe (qui retrouve, à l'ordre 2 en `dx`, la vitesse EXACTE du front poussé de P1), avec son repère en diagonale. Valeurs
//! attendues : `references/etape-p2-valeurs.md`, identifiants `V-P2-H-xx`, `V-P2-MAIN-xx`, `V-P2-AXE-xx` (la vitesse exacte : V-P2-02).

mod etape_p2_commun;

use etape_p2_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages, facteur_h};

/// juste : (V-P2-H-01) le facteur `H(s, t)` de la doc à la main, avec `epsilon = 1/16` (`c1 = 13/16`, `c2 = 1/4`, `k16 = 1` : tout est
/// dyadique, donc exact) : `H(1, 0) = (1 + eps)² = 289/256` (un gradient le long de l'axe), `H(1, 1) = (1 - eps)² = 225/256` (le long de
/// la diagonale), `H(0, 1) = (1 + eps)(1 - 15 eps) = 17/256` (la raideur MINIMALE d'un flux le long d'un axe quand c'est l'autre composante
/// qui porte le gradient), `H(3, 1) = 1085/1024` ; l'échange des rôles (`s` et `t`) n'est PAS une symétrie de `H`, mais `H(s, t)` ne
/// dépend que de `s` et `t` (donc des carrés des composantes). Avec `epsilon = 0` : `H = 1` partout. La garde : `H(s, t) = 1` si
/// `s + t < 1e-24` (donc `H(0, 0) = 1`, `H(5e-25, 0) = 1`), et le calcul normal à `s + t = 1e-24` (non inférieur : `H(1e-24, 0) = 289/256`)
/// ; `ε = 0,05` : `H(1, 0) = 1,05²` à `1e-15` près, `H(1, 1) = 0,95²`.
///
/// Ne couvre pas : les gradients d'un état réel (voir les tests du naïf et du front d'axe).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn facteur_h_a_la_main() {
    let (c1, c2, k16) = (13.0 / 16.0, 0.25, 1.0);
    assert_eq!(facteur_h(1.0, 0.0, c1, c2, k16), 289.0 / 256.0);
    assert_eq!(facteur_h(1.0, 1.0, c1, c2, k16), 225.0 / 256.0);
    assert_eq!(facteur_h(0.0, 1.0, c1, c2, k16), 17.0 / 256.0);
    assert_eq!(facteur_h(3.0, 1.0, c1, c2, k16), 1085.0 / 1024.0);
    assert_eq!(facteur_h(1e-24, 0.0, c1, c2, k16), 289.0 / 256.0);
    assert_eq!(facteur_h(5e-25, 0.0, c1, c2, k16), 1.0);
    assert_eq!(facteur_h(0.0, 5e-25, c1, c2, k16), 1.0);
    assert_eq!(facteur_h(0.0, 0.0, c1, c2, k16), 1.0);
    for (s, t) in [(1.0, 0.0), (0.0, 1.0), (2.0, 3.0), (1e-30, 0.0)] {
        assert_eq!(facteur_h(s, t, 1.0, 0.0, 0.0), 1.0, "epsilon = 0 : H({s}, {t})");
    }
    let e = 0.05;
    let (c1, c2, k16) = (1.0 - 3.0 * e, 4.0 * e, 16.0 * e);
    assert!((facteur_h(1.0, 0.0, c1, c2, k16) - 1.05 * 1.05).abs() < 1e-15);
    assert!((facteur_h(1.0, 1.0, c1, c2, k16) - 0.95 * 0.95).abs() < 1e-15);
}

/// juste : (V-P2-MAIN-01) un pas isotrope (`epsilon = 0` : `H = 1`, `a = 1`) calculé à la main, en rationnels exacts, sur 3 × 3 cases :
/// `w0 = tau0 = lambda = D = dx = 1`, `dt = 1/16` (la borne de la doc vaut 0,0928 pour `U = 3/4` : accepté), `u` joué,
/// `φ = [[1/2, 1/4, 0], [3/4, 1/2, -1/4], [0, -1/2, -1]]`, `u = [[-1/2, -1/4, 0], [-1/4, -1/2, -1/2], [0, -1/4, -3/4]]`.
/// Pour chaque case, `φ' = φ + (1/16)(Δφ + φ - φ³ - λ u (1 - φ²)²)` (Δφ : la somme des quatre voisines miroir moins `4 φ`) et
/// `u' = u + (Δu + (φ' - φ)/2)/16` : toutes les valeurs attendues sont des rationnels dyadiques, exacts en `f64` (un calcul fait par
/// `fractions.Fraction`, puis recoupé par `p2_ref.py` bit pour bit). Un signe, un `1/2`, un carré, un bord, ou le pas lu en place changent
/// au moins une des 18 valeurs.
///
/// Ne couvre pas : l'anisotropie (voir `facteur_h_a_la_main` et le front d'axe) ; plus d'un pas.
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn un_pas_isotrope_a_la_main() {
    let r = Reglages { w0: 1.0, tau0: 1.0, lambda: 1.0, d: 1.0, epsilon: 0.0, dx: 1.0, dt: 1.0 / 16.0 };
    let phi = [0.5, 0.25, 0.0, 0.75, 0.5, -0.25, 0.0, -0.5, -1.0];
    let u = [-0.5, -0.25, 0.0, -0.25, -0.5, -0.5, 0.0, -0.25, -0.75];
    let phi1: [f64; 9] = [
        277.0 / 512.0,
        4817.0 / 16384.0,
        0.0,
        11393.0 / 16384.0,
        221.0 / 512.0,
        -1815.0 / 8192.0,
        1.0 / 64.0,
        -463.0 / 1024.0,
        -59.0 / 64.0,
    ];
    let u1: [f64; 9] = [
        -459.0 / 1024.0,
        -7983.0 / 32768.0,
        -3.0 / 64.0,
        -9599.0 / 32768.0,
        -499.0 / 1024.0,
        -7703.0 / 16384.0,
        -3.0 / 128.0,
        -527.0 / 2048.0,
        -85.0 / 128.0,
    ];
    assert!(r.dt_borne(0.75, ModeU::Joue) >= r.dt, "la borne de la doc est au moins 1/16 pour U = 3/4");
    for domaine in [Domaine::Entier, Domaine::Quart] {
        let mut d = Dendrite::new(r, 3, domaine, &phi, &u, ModeU::Joue).unwrap();
        d.avancer(1).unwrap();
        // `==` et non les bits : le zéro de la case (0, 2) peut être `+0.0` ou `-0.0` dans une écriture fautive sans autre conséquence
        assert_eq!(d.phi(), &phi1[..], "phi, {domaine:?}");
        assert_eq!(d.u(), &u1[..], "u, {domaine:?}");
        assert_eq!((d.pas_faits(), d.temps()), (1, 1.0 / 16.0));
    }
}

/// Les cas du front droit : `(lambda, lambda Delta)`.
const CAS_AXE: [(f64, f64); 2] = [(2.0, 0.1), (1.0, 1.0)];

/// Largeur de la boîte et temps d'attente du front droit : voir `front_droit`.
const LARGEUR_AXE: f64 = 24.0;
const T_A_AXE: f64 = 10.0;

/// Les tolérances MESURÉES du front droit d'axe (sur le prototype, boîte 24, `t_a = 10`) : `(dx, tolérance sur |v / v_exacte - 1|)` pour
/// chaque cas de `CAS_AXE`. Observé : cas `(2 ; 0,1)` : -4,5004e-2 ; -7,5759e-3 ; -1,8709e-3 pour `dx` = 1 ; 0,5 ; 0,25 ; cas `(1 ; 1)` :
/// -4,5922e-2 ; -1,2046e-2 ; -2,9812e-3. La tolérance est de 1,13 à 1,26 fois l'observé, arrondie vers le haut.
const TOL_AXE: [[(f64, f64); 3]; 2] =
    [[(1.0, 5.1e-2), (0.5, 9.5e-3), (0.25, 2.35e-3)], [(1.0, 5.2e-2), (0.5, 1.51e-2), (0.25, 3.75e-3)]];

/// juste : (V-P2-AXE-01) un front droit le long d'un axe : (a) un état qui ne dépend que de `x` le reste, au bit près, TOUTES les lignes
/// sont égales (domaine entier, `u = -Delta` gelé : les termes croisés sont nuls quand `φy = 0`) ; (b) sa vitesse est celle, EXACTE,
/// du front poussé du modèle 1D de P1 (V-P1-ONDE, calculée à tout ordre en `lambda Delta` par un tir dans le plan de phase : voir la
/// fiche, V-P2-02) avec `W = W0 (1 + eps)` et `tau = tau0 (1 + eps)²` : `v = c W0 / (tau0 (1 + eps))`. `W0 = 1,25`, `tau0 = 2`, `D = 1`,
/// `eps = 0,05`, `(lambda, lambda Delta) = (2 ; 0,1)` et `(1 ; 1)`, `dx` = 1 ; 0,5 ; 0,25 (boîte de 24, front en 6, `dt` la borne de la
/// doc ; la vitesse est mesurée entre `t = 10` et `10 + T`, `T = max(10, 1,5 / v_exacte)`, par l'interpolation de la doc) ; (c) l'écart relatif
/// est sous la tolérance mesurée (`TOL_AXE`) à chaque pas de grille, et il est d'ORDRE 2 : le rapport des écarts de `dx = 0,5` à `0,25` est
/// entre 3,7 et 4,4 (observé : 4,05 et 4,04 ; au pas 1 à 0,5 : 5,9 et 3,8, hors de l'asymptote).
///
/// Ne couvre pas : la valeur publiée de la vitesse de pointe (voir `etape_p2_pointe.rs`) ; un front courbe ; un `dx` plus fin que 0,25.
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn front_droit_d_axe_a_la_vitesse_exacte_de_p1() {
    for (k, &(lambda, ld)) in CAS_AXE.iter().enumerate() {
        let mut ecarts = Vec::new();
        for &(dx, tol) in &TOL_AXE[k] {
            let (ecart, egales) = front_droit(Moteur::Lib, dx, LARGEUR_AXE, T_A_AXE, lambda, ld, 0);
            assert!(egales, "lambda Delta = {ld}, dx = {dx} : toutes les lignes devaient être égales au bit près");
            assert!(
                ecart.abs() <= tol,
                "lambda Delta = {ld}, dx = {dx} : écart {ecart:e} hors de la tolérance {tol:e}"
            );
            ecarts.push(ecart.abs());
        }
        let rapport = ecarts[1] / ecarts[2];
        assert!(
            (3.7..=4.4).contains(&rapport),
            "lambda Delta = {ld} : rapport des écarts de 0,5 à 0,25 = {rapport} (ordre 2 : 4)"
        );
    }
}

/// régression : (V-P2-AXE-02) le REPÈRE de l'anisotropie de la grille : le même front poussé en DIAGONALE (la normale à 45 degrés ;
/// `a = 1 - eps`, `v = c W0 / (tau0 (1 - eps))` pour le modèle continu : les termes croisés sont nuls aussi quand `φx = φy`) sur une
/// boîte de 60 (la boîte de 24 ou de 40 est trop étroite : les bords l'altèrent), `dx = 0,5`, `t_a = 10`, mêmes réglages que le front d'axe :
/// l'écart relatif à la vitesse continue est -2,5165e-4 pour `(lambda, lambda Delta) = (2 ; 0,1)` et -3,5832e-3 pour `(1 ; 1)` ; il doit
/// rester entre 0,8 et 1,25 fois l'observé, signe compris (de -3,2e-4 à -2,0e-4, et de -4,5e-3 à -2,85e-3 : une régression se borne des
/// deux côtés). C'est un repère de NOTRE code, pas une valeur exacte : la grille carrée n'est pas exactement isotrope, et à `dx = 0,25`
/// l'écart vaut +3,2e-4 et -4,7e-4 (mesuré hors des tests) ; dans une boîte de 80 il vaut -2,5509e-4 et -3,5832e-3.
///
/// Ne couvre pas : une valeur exacte ; la convergence (à `dx = 0,25` le test coûterait 6 s de plus pour rien).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn repere_du_front_en_diagonale() {
    for (&(lambda, ld), (bas, haut)) in CAS_AXE.iter().zip([(-3.2e-4, -2.0e-4), (-4.5e-3, -2.85e-3)]) {
        let (ecart, _) = front_droit(Moteur::Lib, 0.5, 60.0, 10.0, lambda, ld, 1);
        assert!(
            (bas..=haut).contains(&ecart),
            "lambda Delta = {ld} : écart {ecart:e} hors du repère [{bas:e} ; {haut:e}]"
        );
    }
}

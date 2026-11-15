//! Étape P2 : la position de la pointe de la dendrite : l'interpolation de la doc à la main, et la CONVERGENCE de la position de la pointe
//! en temps court quand la grille s'affine. **Ce fichier ne reproduit pas la vitesse de pointe de la théorie** (voir `MODELE.md`, « Étape
//! P2 » : la vitesse publiée n'est pas atteinte, et elle bouge encore quand la grille s'affine à `dx = 0,4`) : il décrit ce que le calcul
//! fait. Valeurs attendues : `references/etape-p2-valeurs.md`, identifiants `V-P2-POINTE-xx` et `V-P2-CONV-xx`.

mod etape_p2_commun;

use etape_p2_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};

/// juste : (V-P2-POINTE-01) la position de la pointe de la doc, à la main : sur la ligne `i = 0` d'un quart, le plus petit `j` avec
/// `φ[0][j] > 0` et `φ[0][j+1] <= 0`, position `((j + 0,5) dx) + dx (φ[0][j] / (φ[0][j] - φ[0][j+1]))`. Avec `dx = 0,5` et la ligne
/// `[1 ; 0,5 ; -0,25 ; -1]` : `j = 1`, `0,75 + 0,5 · (0,5 / 0,75)` ; avec `[1 ; 0 ; -1 ; -1]` (un zéro exact : `0 <= 0`, donc le
/// croisement est de `j = 0` à `j = 1`, fraction 1) : `0,25 + 0,5 · 1 = 0,75` ; avec deux croisements (`[1 ; -1 ; 1 ; -1]`) le
/// premier : `0,25 + 0,5 · 0,5 = 0,5` ; sans croisement (`[1 ; 1 ; 1 ; 1]`, `[-1 ; -1 ; -1 ; -1]`) : `None` ; et `None` pour un domaine entier.
/// L'inégalité de gauche est STRICTE : `[0 ; -1 ; -1 ; -1]` et `[0 ; 0 ; -1 ; -1]` n'ont pas de croisement (`None`, et non `0,25` ni NaN),
/// `[1 ; 0 ; 0 ; -1]` croise de `j = 0` à `j = 1` (`0,75`). Seule la ligne `i = 0` compte : avec une ligne 0 sans croisement et les autres
/// lignes qui croisent, `None` ; avec la ligne 0 qui croise et les autres non, la position. Et l'ORDRE des opérations est celui de la doc :
/// sur 300 lignes tirées (`n = 7`, `dx` tiré de 0,1 à 1,3), la position est, bit pour bit, celle de `position_ligne` (la formule de la
/// doc écrite dans les tests), et l'autre écriture `(j + 0,5 + f) · dx` donne d'autres bits pour au moins 30 de ces lignes.
///
/// Ne couvre pas : un état né d'un calcul (voir `dendrite_egale_au_naif_bit_pour_bit`, qui compare la pointe à chaque point).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn position_de_la_pointe_a_la_main() {
    let r = Reglages { w0: 1.0, tau0: 1.0, lambda: 1.0, d: 1.0, epsilon: 0.05, dx: 0.5, dt: 0.01 };
    let depart = |ligne: [f64; 4], domaine: Domaine| {
        let phi: Vec<f64> = (0..4).flat_map(|_| ligne).collect();
        Dendrite::new(r, 4, domaine, &phi, &[0.0; 16], ModeU::Gele).unwrap()
    };
    let q = Domaine::Quart;
    assert_eq!(depart([1.0, 0.5, -0.25, -1.0], q).position_pointe(), Some(0.75 + 0.5 * (0.5 / 0.75)));
    assert_eq!(depart([1.0, 0.0, -1.0, -1.0], q).position_pointe(), Some(0.25 + 0.5 * 1.0));
    assert_eq!(depart([1.0, -1.0, 1.0, -1.0], q).position_pointe(), Some(0.25 + 0.5 * 0.5));
    assert_eq!(depart([1.0, 1.0, 1.0, 1.0], q).position_pointe(), None);
    assert_eq!(depart([-1.0, -1.0, -1.0, -1.0], q).position_pointe(), None);
    assert_eq!(depart([1.0, 0.5, -0.25, -1.0], Domaine::Entier).position_pointe(), None);
    // l'inégalité stricte à gauche
    assert_eq!(depart([0.0, -1.0, -1.0, -1.0], q).position_pointe(), None);
    assert_eq!(depart([0.0, 0.0, -1.0, -1.0], q).position_pointe(), None);
    assert_eq!(depart([1.0, 0.0, 0.0, -1.0], q).position_pointe(), Some(0.25 + 0.5 * 1.0));
    // seule la ligne i = 0 compte
    let lignes = |premiere: [f64; 4], autres: [f64; 4]| {
        let phi: Vec<f64> = premiere.into_iter().chain((0..3).flat_map(|_| autres)).collect();
        Dendrite::new(r, 4, q, &phi, &[0.0; 16], ModeU::Gele).unwrap().position_pointe()
    };
    assert_eq!(lignes([1.0, 1.0, 1.0, 1.0], [1.0, -1.0, -1.0, -1.0]), None);
    assert_eq!(lignes([1.0, -1.0, -1.0, -1.0], [1.0, 1.0, 1.0, 1.0]), Some(0.25 + 0.5 * 0.5));
    // l'ordre des opérations de la doc, bit pour bit
    let mut g = Lcg(77);
    let (mut compares, mut autre_ordre_differe) = (0, 0);
    for _ in 0..300 {
        let n = 7;
        let rr = Reglages { dx: g.entre(0.1, 1.3), dt: 1e-4, ..r };
        let phi: Vec<f64> = (0..n * n).map(|_| g.entre(-1.0, 1.0)).collect();
        let d = Dendrite::new(rr, n, q, &phi, &[0.0; 49], ModeU::Gele).unwrap();
        let attendue = position_ligne(&phi[..n], rr.dx);
        assert_eq!(
            d.position_pointe().map(f64::to_bits),
            attendue.map(f64::to_bits),
            "dx = {}, ligne {:?}",
            rr.dx,
            &phi[..n]
        );
        if let Some(x) = attendue {
            compares += 1;
            let j = (0..n - 1).find(|&j| phi[j] > 0.0 && phi[j + 1] <= 0.0).unwrap();
            let autre = (j as f64 + 0.5 + phi[j] / (phi[j] - phi[j + 1])) * rr.dx;
            autre_ordre_differe += usize::from(autre.to_bits() != x.to_bits());
        }
    }
    assert!(
        compares >= 200 && autre_ordre_differe >= 30,
        "{compares} positions, {autre_ordre_differe} où l'autre ordre diffère"
    );
}

/// La position de la pointe au temps `t = 50` de la graine ronde de la table II de Kim et coll. : `W0 = 1`, `d0 = 0,5`, `D = 1`,
/// `epsilon = 0,05`, `Delta = 0,65`, rayon 15, `u` joué, quart de `n` cases de côté au pas `dx` ; `dt` est `t / ceil(t / dt_borne)` :
/// il est sous la borne de la doc (calculée à la main et recoupée avec la bibliothèque) et `pas · dt = t` : tous les pas de grille sont
/// comparés AU MÊME temps. Rend `(position, temps écoulé)`.
fn pointe(dx: f64, n: usize, t: f64) -> (f64, f64) {
    pointe_au_pas(dx, n, t, 1.0)
}

/// La même, avec un pas de temps `diviseur` fois plus petit : `dt = t / ceil(diviseur · t / dt_borne)`.
fn pointe_au_pas(dx: f64, n: usize, t: f64, diviseur: f64) -> (f64, f64) {
    let delta = 0.65;
    let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, dx, 1.0);
    // le plus grand |u| de la graine est `Delta` (le liquide lointain)
    let borne = pas_de_temps(&r0, delta, ModeU::Joue);
    let pas = (diviseur * t / borne).ceil() as u64;
    let r = Reglages { dt: t / pas as f64, ..r0 };
    let mut d = Dendrite::graine_ronde(r, n, Domaine::Quart, delta, 15.0, ModeU::Joue).unwrap();
    d.avancer(pas).unwrap();
    (d.position_pointe().expect("la pointe est dans le domaine"), d.temps())
}

/// régression : (V-P2-CONV-01) **description du calcul, pas reproduction de la théorie** : la position de la pointe d'une graine ronde de
/// rayon 15 à `t = 50` (table II de Kim et coll., `Delta = 0,65`, `d0 = 0,5`, `epsilon = 0,05` : `Ṽ` publiée = 0,0469, NON atteinte par
/// ce schéma : -13 % à `dx = 0,8`, -6 % à `dx = 0,4`, voir `MODELE.md`) pour `dx` = 0,8 ; 0,4 ; 0,2 (quart de 44,8 de côté, `dt` à la borne
/// du constructeur, au sens de `pointe`) : (a) la position est celle de nos deux prototypes à 5e-5 près (25,501745 ; 25,773431 ; 25,830097) ;
/// (b) le rapport des écarts `(x(0,4) - x(0,8)) / (x(0,2) - x(0,4))` est entre 4,0 et 5,6 (observé : 4,79 ; ordre 2 : 4 à la limite
/// `dx -> 0`, un peu plus ici : l'état de départ est une rampe et le temps est court ; un schéma d'ordre 1 donnerait 2, d'ordre 3
/// environ 8) ; (c) le domaine est assez grand pour que le mur ne compte pas : le quart doublé (`n` doublé) donne la même position à 1e-4
/// près pour `dx` = 0,8 et 0,4 (observé : 2,4e-6 et 2,9e-6 ; l'écart de grille le plus petit est 5,7e-2) ; (d) le temps écoulé est `t = 50` à
/// 1e-9 près ; (e) le pas de temps compte peu devant le pas de grille : avec `dt` moitié, la pointe avance de 5,13e-3 (`dx` = 0,8) et de
/// 1,70e-3 (`dx` = 0,4) (le schéma est d'ordre 1 en temps ; exigé : entre 4,1e-3 et 6,2e-3, entre 1,35e-3 et 2,05e-3, soit ± 20 % de
/// l'observé), soit environ 2 % et 3 % des écarts de grille 0,272 et 0,057. L'ordre 2 est donc celui de l'affinement conjoint
/// (`dx`, `dt` = la borne, qui suit `dx²`). Hors des tests (fiche) : à `t = 30`, avec un quatrième pas de grille (`dx` = 0,1), les rapports
/// des écarts sont 5,27 puis 3,91.
///
/// Ne couvre pas : la vitesse de pointe en régime établi (`t` ≳ 1200), le rayon de la pointe, la forme de la dendrite, le cas `dx` plus fin
/// que 0,2, ni l'accord à la théorie (-13 % à 0,8 ; -6 % à 0,4 ; ≈ -4 % extrapolé : l'écart est dit dans `MODELE.md`, non expliqué).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn la_pointe_converge_a_l_ordre_2_sans_reproduire_la_theorie() {
    let t = 50.0;
    let cas = [(0.8, 56_usize, 25.501_745), (0.4, 112, 25.773_431), (0.2, 224, 25.830_097)];
    let mut x = Vec::new();
    for (dx, n, attendu) in cas {
        let (pos, temps) = pointe(dx, n, t);
        assert!((temps - t).abs() < 1e-9, "dx = {dx} : le temps écoulé est {temps}");
        assert!((pos - attendu).abs() < 5e-5, "dx = {dx} : position {pos}, attendue {attendu} à 5e-5 près");
        x.push(pos);
    }
    let rapport = (x[1] - x[0]) / (x[2] - x[1]);
    assert!((4.0..=5.6).contains(&rapport), "rapport des écarts {rapport} (ordre 2 : 4 à la limite)");
    for (dx, n) in [(0.8, 112_usize), (0.4, 224)] {
        let (double, _) = pointe(dx, n, t);
        let simple = if dx == 0.8 { x[0] } else { x[1] };
        assert!(
            (double - simple).abs() < 1e-4,
            "dx = {dx} : le domaine doublé change la position de {:e}",
            double - simple
        );
    }
    for (dx, n, simple, bas, haut) in [(0.8, 56_usize, x[0], 4.1e-3, 6.2e-3), (0.4, 112, x[1], 1.35e-3, 2.05e-3)] {
        let (demi, temps) = pointe_au_pas(dx, n, t, 2.0);
        assert!((temps - t).abs() < 1e-9);
        let avance = demi - simple;
        assert!((bas..=haut).contains(&avance), "dx = {dx} : avec dt moitié la pointe avance de {avance:e}");
    }
}

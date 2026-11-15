//! Étape P2 : la bibliothèque (`Dendrite`, `Reglages::depuis_capillarite`) donne EXACTEMENT les bits d'un simulateur naïf écrit dans les
//! tests d'après la doc de `src/dendrite.rs` (une case à la fois, une fonction `H` et une garde recalculées partout). Valeurs attendues :
//! `references/etape-p2-valeurs.md`, identifiants `V-P2-NAIF-xx`.

mod etape_p2_commun;

use etape_p2_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};

/// juste : (V-P2-NAIF-01) pour 24 graines (côté `n` de 3 à 17, domaine entier ou quart, `W0` de 0,5 à 2, `tau0` de 0,5 à 3, `lambda` de
/// 0,2 à 1,5, `D` de 0,3 à 2, `epsilon` de 0 à 0,0665, `dx` de 0,3 à 1,2, `dt` de 0,3 à 1 fois la borne de la doc, `φ` tiré dans
/// `[-1 ; 1]` et `u` dans `[-1 ; 0,4]`), les deux modes de `u`, 1 à 300 pas, l'état de `Dendrite` est celui du naïf de `etape_p2_commun`,
/// **bit pour bit** (`φ` et `u`) ; `pas_faits` et `temps` (`pas_faits as f64 * dt`) sont ceux qu'on attend ; et, à chaque point, la
/// position de la pointe est celle de la doc calculée ici sur la ligne `i = 0` du naïf (`position_ligne`), **bit pour bit**, pour un quart,
/// et `None` pour un domaine entier (au moins 20 positions comparées sur l'ensemble). Un signe, un facteur,
/// un carré, un ordre d'opérations, une garde, un bord, une mise à jour en place ou une constante faussés (voir `etape_p2_fautes.rs`)
/// cassent l'égalité.
///
/// Ne couvre pas : la justesse physique (voir `etape_p2_exact.rs`) ; les grandes grilles (voir `etape_p2_pointe.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn dendrite_egale_au_naif_bit_pour_bit() {
    let tailles = [3_usize, 4, 5, 9, 12, 17];
    let mut pointes = 0;
    for graine in 1..=24_u64 {
        let n = tailles[(graine as usize) % tailles.len()];
        let domaine = if graine % 2 == 0 { Domaine::Entier } else { Domaine::Quart };
        let mut g = Lcg(2000 + graine);
        let (w0, tau0, lambda, d, dx) =
            (g.entre(0.5, 2.0), g.entre(0.5, 3.0), g.entre(0.2, 1.5), g.entre(0.3, 2.0), g.entre(0.3, 1.2));
        let epsilon = if graine % 4 == 0 { 0.0 } else { g.entre(0.0, 0.0665) };
        let phi: Vec<f64> = (0..n * n).map(|_| g.entre(-1.0, 1.0)).collect();
        let u: Vec<f64> = (0..n * n).map(|_| g.entre(-1.0, 0.4)).collect();
        let u_max = u.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        let fraction = g.entre(0.3, 1.0);
        for mode in [ModeU::Gele, ModeU::Joue] {
            let mut r = Reglages { w0, tau0, lambda, d, epsilon, dx, dt: 1.0 };
            r.dt = fraction * pas_de_temps(&r, u_max, mode);
            let mut lib = Dendrite::new(r, n, domaine, &phi, &u, mode).unwrap();
            assert_eq!((lib.pas_faits(), lib.temps()), (0, 0.0));
            let mut fait = 0;
            for pas in [1_u64, 2, 7, 60, 300] {
                lib.avancer(pas - fait).unwrap();
                fait = pas;
                let b = naif(&r, n, &phi, &u, mode, pas, None);
                assert_eq!(bits(lib.phi()), bits(&b.phi), "graine {graine} n={n} {mode:?} pas {pas} : phi");
                assert_eq!(bits(lib.u()), bits(&b.u), "graine {graine} n={n} {mode:?} pas {pas} : u");
                assert_eq!((lib.pas_faits(), lib.temps()), (pas, pas as f64 * r.dt));
                let attendue = if domaine == Domaine::Quart { position_ligne(&b.phi[..n], r.dx) } else { None };
                assert_eq!(
                    lib.position_pointe().map(f64::to_bits),
                    attendue.map(f64::to_bits),
                    "graine {graine} n={n} {mode:?} pas {pas} : la pointe"
                );
                pointes += usize::from(attendue.is_some());
            }
        }
    }
    assert!(pointes >= 20, "trop peu de positions de pointe comparées : {pointes}");
}

/// juste : (V-P2-NAIF-02) la graine ronde de la bibliothèque est celle de la doc, recalculée à la main (`graine_ronde_naive` :
/// le centre, `sqrt`, la rampe `0 - clamp(s, -1, 1)`, `u = 0 - (delta · ((1 - φ) / 2))`), **bit pour bit**, pour les deux domaines,
/// `n` = 3, 8, 13, 24, `dx` = 0,4 ; 0,8, rayon 1 ; 3 ; 15, `W0` = 1 ; 1,3, `Delta` = 0,65 ; 0,3 ; et AUCUN zéro négatif (`-0.0`) n'y est
/// écrit : `φ` et `u` valent `+0.0` quand ils sont nuls (l'écriture `0 - x` de la doc donne `+0.0`, `-x` donnerait `-0.0` : le même état
/// n'aurait pas la même empreinte). Le solide a `u = +0.0` et `φ = +1`, le liquide lointain `u = -Delta` et `φ = -1`. Le zéro de la rampe
/// est atteint EXACTEMENT dans deux cas : domaine entier `n = 11`, `dx = 1`, rayon 5 (la case `(9, 8)` est en `(x, y) = (3, 4)`, à distance 5
/// exactement) et quart `n = 6`, `dx = 0,8`, rayon `sqrt(x² + y²)` de la case `(2, 4)` : `φ` y vaut `+0.0` (bits nuls), jamais `-0.0`.
///
/// Ne couvre pas : l'évolution (voir `dendrite_egale_au_naif_bit_pour_bit`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn graine_ronde_egale_a_la_graine_a_la_main() {
    let negatif = (-0.0_f64).to_bits();
    for domaine in [Domaine::Entier, Domaine::Quart] {
        for n in [3_usize, 8, 13, 24] {
            for (dx, rayon, w0, delta) in
                [(0.4, 1.0, 1.0, 0.65), (0.8, 3.0, 1.3, 0.3), (0.8, 15.0, 1.0, 0.65), (0.4, 3.0, 1.0, 0.3)]
            {
                let mut r = Reglages::depuis_capillarite(w0, 0.5, 1.0, 0.05, dx, 1.0);
                r.dt = pas_de_temps(&r, delta, ModeU::Joue);
                let d = Dendrite::graine_ronde(r, n, domaine, delta, rayon, ModeU::Joue).unwrap();
                let (phi, u) = graine_ronde_naive(&r, n, domaine, delta, rayon);
                assert_eq!(bits(d.phi()), bits(&phi), "{domaine:?} n = {n} dx = {dx} rayon = {rayon} : phi");
                assert_eq!(bits(d.u()), bits(&u), "{domaine:?} n = {n} dx = {dx} rayon = {rayon} : u");
                assert!(
                    d.phi().iter().chain(d.u()).all(|x| x.to_bits() != negatif),
                    "un zéro négatif dans l'état de départ"
                );
                assert_eq!((d.n(), d.pas_faits(), d.domaine(), d.mode()), (n, 0, domaine, ModeU::Joue));
            }
        }
    }
    // le zéro exact de la rampe : `+0.0`
    for (domaine, n, dx, case, rayon) in [
        (Domaine::Entier, 11_usize, 1.0, (9_usize, 8_usize), 5.0),
        (Domaine::Quart, 6, 0.8, (2, 4), (((4.5_f64 * 0.8) * (4.5 * 0.8)) + ((2.5 * 0.8) * (2.5 * 0.8))).sqrt()),
    ] {
        let mut r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, dx, 1.0);
        r.dt = pas_de_temps(&r, 0.65, ModeU::Joue);
        let d = Dendrite::graine_ronde(r, n, domaine, 0.65, rayon, ModeU::Joue).unwrap();
        let (phi, u) = graine_ronde_naive(&r, n, domaine, 0.65, rayon);
        assert_eq!((bits(d.phi()), bits(d.u())), (bits(&phi), bits(&u)), "{domaine:?} : la graine au zéro exact");
        assert_eq!(d.phi()[case.0 * n + case.1].to_bits(), 0, "{domaine:?} : le zéro de la rampe doit être +0.0");
        assert_eq!(d.u()[case.0 * n + case.1], -0.325);
    }
    // le coin d'un grand quart : solide au centre (`phi = 1`, `u = +0`), liquide au loin (`phi = -1`, `u = -Delta`)
    let mut r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
    r.dt = pas_de_temps(&r, 0.65, ModeU::Joue);
    let d = Dendrite::graine_ronde(r, 40, Domaine::Quart, 0.65, 15.0, ModeU::Joue).unwrap();
    assert_eq!((d.phi()[0], d.u()[0].to_bits()), (1.0, 0.0_f64.to_bits()));
    assert_eq!((d.phi()[39 * 40 + 39], d.u()[39 * 40 + 39]), (-1.0, -0.65));
}

/// juste : (V-P2-NAIF-03) le rattachement de la doc entre `d0` et les réglages (`Reglages::depuis_capillarite`) : `lambda = (A1 w0) / d0`,
/// `tau0 = (lambda (A2 (w0 w0))) / D` (cinétique nulle), calculés à la main : pour `W0 = 1`, `d0 = 0,5`, `D = 1` : `lambda = 2 A1` (exact :
/// `5 sqrt(2) / 4 = 1,767 766 952 966 369`) et `tau0 = lambda · A2` ; pour `W0 = 1,25`, `d0 = 1,5`, `D = 2` : `lambda = (A1 · 1,25) / 1,5` et
/// `tau0 = (lambda · (A2 · 1,5625)) / 2` ; et, pour 8 réglages tirés, les bits de la bibliothèque sont ceux de `reglages_naifs` (la doc).
/// Les champs `w0`, `d`, `epsilon`, `dx`, `dt` sont ceux donnés. La faute `A1PourA2` (`a1` à la place de `a2` dans `tau0`) donne un
/// `tau0` différent (voir `etape_p2_fautes.rs`).
///
/// Ne couvre pas : la signification physique de `d0` (voir `MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn capillarite_a_la_main() {
    let r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 0.04);
    assert_eq!(r.lambda, 2.0 * A1);
    assert_eq!(r.tau0, (2.0 * A1) * A2);
    assert!((r.lambda - 1.767_766_952_966_369).abs() < 1e-15);
    assert_eq!((r.w0, r.d, r.epsilon, r.dx, r.dt), (1.0, 1.0, 0.05, 0.8, 0.04));
    let r = Reglages::depuis_capillarite(1.25, 1.5, 2.0, 0.03, 0.5, 0.01);
    assert_eq!(r.lambda, (A1 * 1.25) / 1.5);
    assert_eq!(r.tau0, (r.lambda * (A2 * 1.5625)) / 2.0);
    assert_eq!((r.w0, r.d, r.epsilon, r.dx, r.dt), (1.25, 2.0, 0.03, 0.5, 0.01));
    let mut g = Lcg(31);
    for _ in 0..8 {
        let (w0, d0, d, e, dx, dt) = (
            g.entre(0.5, 2.0),
            g.entre(0.3, 3.0),
            g.entre(0.3, 3.0),
            g.entre(0.0, 0.0665),
            g.entre(0.2, 1.5),
            g.entre(0.001, 0.05),
        );
        let a = Reglages::depuis_capillarite(w0, d0, d, e, dx, dt);
        let b = reglages_naifs(w0, d0, d, e, dx, dt, None);
        assert_eq!(
            [a.w0, a.tau0, a.lambda, a.d, a.epsilon, a.dx, a.dt].map(f64::to_bits),
            [b.w0, b.tau0, b.lambda, b.d, b.epsilon, b.dx, b.dt].map(f64::to_bits)
        );
    }
}

/// juste : (V-P2-NAIF-04) la garde de `|∇φ| = 0` est STRICTE (`< 1e-24`), dans un état et pas seulement dans `facteur_h` : deux états de
/// 3 × 3 cases (`dx = 0,5`, donc `ig = 2` et `ig2 = 1`, exacts ; `W0 = 1`, `d0 = 0,5`, `D = 1`, `epsilon = 0,05`, `u = -0,3` gelé, `dt` la
/// borne de la doc) dont les colonnes sont constantes : (A) colonnes `[0 ; 0,5 ; 1e-12]` : au centre de la colonne 1, le gradient au centre
/// vaut `1e-12` exactement, son carré `1e-24` exactement (vérifié ici), donc `rs = 1e-24` : PAS sous le seuil, `ac = c1 + c2` ; (B) colonnes
/// `[0 ; 5e-13 ; 0,7]` : à la face entre les colonnes 0 et 1, `gn = 1e-12`, `s = 1e-24`, `t = 0` : `H = (c1 + c2)²` et non 1. La
/// bibliothèque donne les bits du naïf (qui écrit `<`) après 1 et 5 pas, et le naïf dont le seuil est `1e-12` (pour qui ces gradients sont
/// sous le seuil) donne d'autres bits dès le premier pas : le cas est bien au seuil, une garde écrite `<=` y est vue.
///
/// Ne couvre pas : un gradient entre deux flottants voisins du seuil ; l'effet physique de la garde.
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn la_garde_au_seuil_exact() {
    assert_eq!((1e-12_f64 * 1e-12).to_bits(), GARDE.to_bits(), "le carré de 1e-12 est le seuil, au bit près");
    assert_eq!(((5e-13_f64 - 0.0) * 2.0).to_bits(), 1e-12_f64.to_bits());
    let mut r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.5, 1.0);
    r.dt = pas_de_temps(&r, 0.3, ModeU::Gele);
    for colonnes in [[0.0, 0.5, 1e-12], [0.0, 5e-13, 0.7]] {
        let phi: Vec<f64> = (0..3).flat_map(|_| colonnes).collect();
        let u = [-0.3; 9];
        for domaine in [Domaine::Entier, Domaine::Quart] {
            for pas in [1_u64, 5] {
                let mut lib = Dendrite::new(r, 3, domaine, &phi, &u, ModeU::Gele).unwrap();
                lib.avancer(pas).unwrap();
                let juste = naif(&r, 3, &phi, &u, ModeU::Gele, pas, None);
                assert_eq!(bits(lib.phi()), bits(&juste.phi), "colonnes {colonnes:?} {domaine:?} pas {pas}");
            }
        }
        let juste = naif(&r, 3, &phi, &u, ModeU::Gele, 1, None);
        let seuil_faux = naif(&r, 3, &phi, &u, ModeU::Gele, 1, Some(Faute::GardeSeuilFaux));
        assert_ne!(bits(&juste.phi), bits(&seuil_faux.phi), "colonnes {colonnes:?} : le cas n'est pas au seuil");
    }
}

//! Étape P2 : les entrées refusées (jamais de panique), l'ordre des erreurs, la borne de pas et ce qu'elle garantit (la zone sûre), la marge
//! de `φ`. Valeurs attendues : `references/etape-p2-valeurs.md`, identifiants `V-P2-REF-xx`, `V-P2-STAB-xx` et `V-P2-MARGE-xx`.

mod etape_p2_commun;

use etape_p2_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ErreurDendrite as E, MARGE, ModeU, N_MAX, Reglages};

/// Des réglages valides de base (`dt = 0,01` : sous la borne, 0,056 pour `U = 0,5`).
fn bon() -> Reglages {
    Reglages { w0: 1.0, tau0: 1.0, lambda: 1.0, d: 1.0, epsilon: 0.05, dx: 1.0, dt: 0.01 }
}

const PHI9: [f64; 9] = [1.0, 0.5, 0.0, 0.5, 0.0, -0.5, 0.0, -0.5, -1.0];
const U9: [f64; 9] = [-0.5; 9];
const NOMS: [&str; 7] = ["w0", "tau0", "lambda", "d", "epsilon", "dx", "dt"];

/// Les réglages `bon()` dont le champ `k` (de `NOMS`) vaut `v`.
fn avec(k: usize, v: f64) -> Reglages {
    let mut r = bon();
    match k {
        0 => r.w0 = v,
        1 => r.tau0 = v,
        2 => r.lambda = v,
        3 => r.d = v,
        4 => r.epsilon = v,
        5 => r.dx = v,
        _ => r.dt = v,
    }
    r
}

/// juste : (V-P2-REF-01) la borne de pas de la doc, à la main (`Reglages::dt_borne(u_max, mode)`), dans l'ordre exact d'opérations de la doc :
/// (a) `w0 = tau0 = d = lambda = dx = 1`, `epsilon = 0`, `U = 0` : `dt_phi = 2 / (8 + 2) = 0,2`, `dt_u = 1 / (4 + 1/4) = 1/4,25` : borne
/// `0,5 · 0,2 = 0,1` pour `u` gelé ET joué ; (b) `d = 2` : `dt_u = 1 / (8 + 1/4) = 1/8,25` : joué `0,5 · (1 / 8,25)`, gelé `0,1` ; (c)
/// `epsilon = 1/16` (`s_max = 465/256`, `tau_min = 225/256`, tout est dyadique) : `0,5 · (450 / 4232)` en gelé ; (d) `U = 2`, `lambda = 1` :
/// `0,5 · (2 / (10 + (1,54 · 1) · 2))` (le terme du couplage) ; le pas `dt = dt_borne` est ACCEPTÉ par `verifier` et par `Dendrite::new`, et le
/// flottant suivant est refusé par `PasInstable { dt, dt_borne }` (les deux valeurs rendues) dans `verifier`, `new` et `graine_ronde` ;
/// (e) pour `graine_ronde`, la borne est celle de `U = delta`, MÊME quand la graine ne contient aucune case à `u = -delta` : une graine toute
/// solide (rayon `1e6` : `u = +0` partout) ou sans liquide lointain (`n = 3`, rayon `1e-3`, `dx = 0,1` : `|u| < delta` partout) est refusée
/// au flottant qui suit `dt_borne(delta)`, alors que `Dendrite::new` accepte ce même état et ce même pas (sa borne est celle du `U` réel).
///
/// Ne couvre pas : la stabilité réelle (voir `la_zone_sure_sans_erreur_ni_derive`) ; les réglages démesurés (voir plus bas).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn borne_de_pas_a_la_main() {
    let base = Reglages { epsilon: 0.0, ..bon() };
    assert_eq!(base.dt_borne(0.0, ModeU::Gele), 0.5 * (2.0 / 10.0));
    assert_eq!(base.dt_borne(0.0, ModeU::Joue), 0.5 * (2.0 / 10.0));
    let d2 = Reglages { d: 2.0, ..base };
    assert_eq!(d2.dt_borne(0.0, ModeU::Joue), 0.5 * (1.0 / 8.25));
    assert_eq!(d2.dt_borne(0.0, ModeU::Gele), 0.5 * (2.0 / 10.0));
    let e16 = Reglages { epsilon: 1.0 / 16.0, ..base };
    assert_eq!(e16.dt_borne(0.0, ModeU::Gele), 0.5 * (450.0 / 4232.0));
    assert_eq!(base.dt_borne(2.0, ModeU::Gele), 0.5 * (2.0 / (10.0 + (1.54 * 1.0) * 2.0)));
    // la borne de la bibliothèque est celle de la doc, sur 200 réglages tirés (les deux bornes `dt_phi` et `dt_u` l'emportent chacune)
    let mut g = Lcg(404);
    let (mut par_phi, mut par_u) = (0, 0);
    for _ in 0..200 {
        let r = Reglages {
            w0: g.entre(0.3, 3.0),
            tau0: g.entre(0.3, 3.0),
            lambda: g.entre(0.1, 5.0),
            d: g.entre(0.1, 5.0),
            epsilon: g.entre(0.0, 0.0665),
            dx: g.entre(0.2, 2.0),
            dt: 1.0,
        };
        let u_max = g.entre(0.0, 1.5);
        let attendu = dt_borne_a_la_main(&r, u_max, ModeU::Joue);
        assert_eq!(r.dt_borne(u_max, ModeU::Joue).to_bits(), attendu.to_bits(), "{r:?} U = {u_max}");
        assert_eq!(r.dt_borne(u_max, ModeU::Gele).to_bits(), dt_borne_a_la_main(&r, u_max, ModeU::Gele).to_bits());
        if r.dt_borne(u_max, ModeU::Gele) > attendu {
            par_u += 1;
        } else {
            par_phi += 1;
        }
    }
    assert!(par_u >= 20 && par_phi >= 20, "les deux bornes devaient l'emporter : {par_phi} et {par_u}");
    // l'égalité est acceptée, le flottant suivant refusé, par les trois entrées
    for (r, mode) in [(base, ModeU::Gele), (d2, ModeU::Joue), (e16, ModeU::Gele)] {
        let borne = r.dt_borne(0.5, mode);
        let ok = Reglages { dt: borne, ..r };
        let trop = Reglages { dt: borne.next_up(), ..r };
        let (phi, u) = (PHI9, U9);
        assert_eq!(ok.verifier(0.5, mode), Ok(()));
        assert!(Dendrite::new(ok, 3, Domaine::Quart, &phi, &u, mode).is_ok());
        assert!(Dendrite::graine_ronde(ok, 8, Domaine::Quart, 0.5, 3.0, mode).is_ok());
        let attendu = Err(E::PasInstable { dt: borne.next_up(), dt_borne: borne });
        assert_eq!(trop.verifier(0.5, mode), attendu);
        assert_eq!(Dendrite::new(trop, 3, Domaine::Quart, &phi, &u, mode).err(), attendu.clone().err());
        assert_eq!(Dendrite::graine_ronde(trop, 8, Domaine::Quart, 0.5, 3.0, mode).err(), attendu.clone().err());
    }
    // (e) la graine ronde : `U = delta`, quel que soit le contenu de la graine
    for mode in [ModeU::Gele, ModeU::Joue] {
        for (n, dx, rayon) in [(8_usize, 1.0, 1e6), (3, 0.1, 1e-3)] {
            let r0 = Reglages { dx, lambda: 4.0, ..bon() };
            let borne = r0.dt_borne(0.65, mode);
            let ok =
                Dendrite::graine_ronde(Reglages { dt: borne, ..r0 }, n, Domaine::Quart, 0.65, rayon, mode).unwrap();
            let u_reel = ok.u().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            assert!(u_reel < 0.65, "la graine ne devait pas contenir de case à u = -delta (n = {n} : {u_reel})");
            let trop = Reglages { dt: borne.next_up(), ..r0 };
            assert!(trop.dt <= r0.dt_borne(u_reel, mode), "le pas devait rester sous la borne du U réel");
            assert_eq!(
                Dendrite::graine_ronde(trop, n, Domaine::Quart, 0.65, rayon, mode).err(),
                Some(E::PasInstable { dt: borne.next_up(), dt_borne: borne }),
                "graine_ronde : la borne est celle de delta (n = {n}, {mode:?})"
            );
            assert!(Dendrite::new(trop, n, Domaine::Quart, ok.phi(), ok.u(), mode).is_ok(), "new : la borne du U réel");
        }
    }
}

/// juste : (V-P2-REF-02) chaque réglage hors plage est refusé par `ParametreInvalide(nom)`, dans l'ordre de la doc (`w0`, `tau0`, `lambda`, `d`,
/// `epsilon`, `dx`, `dt`), par `verifier`, `Dendrite::new` et `Dendrite::graine_ronde` : zéro, négatif, NaN, infini ; pour `epsilon`, `1/15`
/// (`1.0 / 15.0` : la raideur anisotrope s'annule) et au-delà sont refusés, `0` et le flottant juste avant `1/15` acceptés ; et, quand DEUX
/// règles sont fausses à la fois (un NaN pour le premier, `-1` pour le suivant), la première de l'ordre est rendue : pour chaque paire de
/// réglages voisins de la liste, ET pour le premier avec le dernier (`w0` NaN et `dt` NaN).
///
/// Ne couvre pas : la comparaison à la borne (voir `borne_de_pas_a_la_main`) ; les états de départ (voir plus bas).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn reglages_refuses_et_leur_ordre() {
    let phi = PHI9;
    let u = U9;
    for (k, nom) in NOMS.iter().enumerate() {
        let mut mauvais = vec![0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
        if k == 4 {
            mauvais.extend([1.0 / 15.0, 0.07, 1.0]);
        }
        for v in mauvais {
            let r = avec(k, v);
            let attendu = Err(E::ParametreInvalide(nom));
            // epsilon = 0 est valide : seul son `0.0` fait exception
            if k == 4 && v == 0.0 {
                assert_eq!(r.verifier(0.5, ModeU::Joue), Ok(()));
                continue;
            }
            assert_eq!(r.verifier(0.5, ModeU::Joue), attendu, "{nom} = {v}");
            assert_eq!(
                Dendrite::new(r, 3, Domaine::Quart, &phi, &u, ModeU::Joue).err(),
                attendu.clone().err(),
                "{nom} = {v} (new)"
            );
            assert_eq!(
                Dendrite::graine_ronde(r, 8, Domaine::Quart, 0.5, 3.0, ModeU::Gele).err(),
                attendu.clone().err(),
                "{nom} = {v} (graine_ronde)"
            );
        }
    }
    // le flottant juste avant 1/15 est accepté (avec un pas assez petit pour sa borne)
    assert_eq!(Reglages { dt: 1e-3, ..avec(4, (1.0_f64 / 15.0).next_down()) }.verifier(0.5, ModeU::Gele), Ok(()));
    // l'ordre : deux règles fausses à la fois (ici un NaN et un zéro) : la première de la liste gagne
    for k in 0..NOMS.len() - 1 {
        let mut r = bon();
        for (idx, v) in [(k, f64::NAN), (k + 1, -1.0)] {
            match idx {
                0 => r.w0 = v,
                1 => r.tau0 = v,
                2 => r.lambda = v,
                3 => r.d = v,
                4 => r.epsilon = v,
                5 => r.dx = v,
                _ => r.dt = v,
            }
        }
        assert_eq!(
            r.verifier(0.5, ModeU::Joue),
            Err(E::ParametreInvalide(NOMS[k])),
            "{} avant {}",
            NOMS[k],
            NOMS[k + 1]
        );
        assert_eq!(
            Dendrite::new(r, 3, Domaine::Quart, &PHI9, &U9, ModeU::Joue).err(),
            Some(E::ParametreInvalide(NOMS[k]))
        );
    }
    let r = Reglages { w0: -1.0, dt: f64::NAN, ..bon() };
    assert_eq!(r.verifier(0.5, ModeU::Gele), Err(E::ParametreInvalide("w0")));
    // un `dt` invalide l'emporte sur un `dt` trop grand (ce dernier est la dernière règle)
    assert_eq!(Reglages { dt: -5.0, ..bon() }.verifier(1e9, ModeU::Joue), Err(E::ParametreInvalide("dt")));
}

/// juste : (V-P2-REF-03) les états de départ refusés par `Dendrite::new`, dans l'ordre de la doc : (1) un réglage invalide ; (2) moins de 3
/// cases (0, 1, 2) : `GrilleTropPetite(n)` ; (3) plus de `N_MAX` cases (`N_MAX + 1`, `1 << 40`, `usize::MAX` : sans allouer, sans panique) :
/// `GrilleTropGrande(n)` ; (4) `phi` ou `u` de longueur autre que `n * n` : `LongueursDifferentes` ; (5) une valeur NaN ou infinie dans `phi`
/// ou dans `u`, ou un `|φ|` au-delà de `1 + MARGE` (`MARGE = 0,001`, incluse : `1 + MARGE` et `-(1 + MARGE)` sont acceptés, le flottant suivant
/// refusé) : `ValeurDeDepart` ; (6) un `dt` plus grand que la borne pour le `U` de l'état de départ : `PasInstable` (une valeur de `u` de `1e6`
/// rend la borne trop petite pour `dt = 0,01`). Pour CHAQUE paire de règles voisines de cette liste, un cas où les deux sont fausses à la fois
/// rend la première. `N_MAX = 16384`, et `N_MAX` lui-même est un côté PERMIS (avec des champs de 9 valeurs : `LongueursDifferentes`, pas
/// `GrilleTropGrande`). Un état accepté rend ce qu'on lui a donné : `reglages()`, `n()`, `domaine()`, `mode()`, `phi()`, `u()`,
/// `pas_faits() = 0`, `temps() = 0`.
///
/// Ne couvre pas : `Dendrite::graine_ronde` (voir le test suivant).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn etats_de_depart_refuses_et_leur_ordre() {
    let r = bon();
    let new =
        |r: Reglages, n: usize, phi: &[f64], u: &[f64]| Dendrite::new(r, n, Domaine::Quart, phi, u, ModeU::Joue).err();
    assert_eq!(MARGE, 0.001);
    assert_eq!(N_MAX, 16384);
    // (2) la grille trop petite
    for n in 0..3_usize {
        let v = vec![0.0; n * n];
        assert_eq!(new(r, n, &v, &v), Some(E::GrilleTropPetite(n)));
    }
    // (3) la grille trop grande, sans allocation
    for n in [N_MAX + 1, 1 << 40, usize::MAX] {
        assert_eq!(new(r, n, &PHI9, &U9), Some(E::GrilleTropGrande(n)), "n = {n}");
    }
    assert_eq!(new(r, N_MAX, &PHI9, &U9), Some(E::LongueursDifferentes), "N_MAX est un côté permis");
    // un état accepté rend ce qu'on lui a donné
    for (domaine, mode) in [(Domaine::Quart, ModeU::Joue), (Domaine::Entier, ModeU::Gele)] {
        let d = Dendrite::new(r, 3, domaine, &PHI9, &U9, mode).unwrap();
        assert_eq!((d.reglages(), d.n(), d.domaine(), d.mode()), (r, 3, domaine, mode));
        assert_eq!((bits(d.phi()), bits(d.u())), (bits(&PHI9), bits(&U9)));
        assert_eq!((d.pas_faits(), d.temps()), (0, 0.0));
    }
    // (4) les longueurs
    assert_eq!(new(r, 3, &[0.0; 8], &U9), Some(E::LongueursDifferentes));
    assert_eq!(new(r, 3, &PHI9, &[0.0; 10]), Some(E::LongueursDifferentes));
    assert_eq!(new(r, 4, &PHI9, &U9), Some(E::LongueursDifferentes));
    // (5) les valeurs
    for mauvais in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut phi = PHI9;
        phi[4] = mauvais;
        assert_eq!(new(r, 3, &phi, &U9), Some(E::ValeurDeDepart), "phi = {mauvais}");
        let mut u = U9;
        u[7] = mauvais;
        assert_eq!(new(r, 3, &PHI9, &u), Some(E::ValeurDeDepart), "u = {mauvais}");
    }
    for (v, accepte) in [
        (1.0 + MARGE, true),
        (-(1.0 + MARGE), true),
        ((1.0 + MARGE).next_up(), false),
        (-(1.0 + MARGE).next_up(), false),
        (220.0, false),
    ] {
        let mut phi = PHI9;
        phi[0] = v;
        assert_eq!(new(r, 3, &phi, &U9).is_none(), accepte, "phi = {v:e}");
    }
    // (6) le pas trop grand pour le U de l'état de départ : le même `dt = 0,01` passe pour `U = 0,5` et pas pour `U = 1e6`
    assert_eq!(new(r, 3, &PHI9, &U9), None);
    let borne = r.dt_borne(1e6, ModeU::Joue);
    assert_eq!(new(r, 3, &PHI9, &[-1e6; 9]), Some(E::PasInstable { dt: 0.01, dt_borne: borne }));
    assert_eq!(new(r, 3, &PHI9, &[1e6; 9]), Some(E::PasInstable { dt: 0.01, dt_borne: borne }));
    // les paires : la première règle de la liste gagne quand deux sont fausses
    let reglage_faux = Reglages { w0: f64::NAN, ..r };
    assert_eq!(new(reglage_faux, 2, &[0.0; 3], &U9), Some(E::ParametreInvalide("w0")), "réglage avant grille");
    assert_eq!(new(r, 2, &[0.0; 3], &U9), Some(E::GrilleTropPetite(2)), "grille trop petite avant longueurs");
    assert_eq!(
        new(r, N_MAX + 1, &[0.0; 3], &U9),
        Some(E::GrilleTropGrande(N_MAX + 1)),
        "grille trop grande avant longueurs"
    );
    assert_eq!(new(r, 3, &[f64::NAN; 8], &U9), Some(E::LongueursDifferentes), "longueurs avant valeurs");
    assert_eq!(new(r, 3, &[220.0; 9], &[f64::NAN; 9]), Some(E::ValeurDeDepart), "valeurs");
    let pas_trop_grand = Reglages { dt: 1.0, ..r };
    assert_eq!(new(pas_trop_grand, 3, &[220.0; 9], &U9), Some(E::ValeurDeDepart), "valeurs avant pas instable");
    assert_eq!(
        new(pas_trop_grand, 3, &PHI9, &[f64::NAN; 9]),
        Some(E::ValeurDeDepart),
        "valeurs avant pas instable (u)"
    );
    assert_eq!(
        new(Reglages { dt: f64::NAN, ..r }, 3, &PHI9, &U9),
        Some(E::ParametreInvalide("dt")),
        "dt invalide avant tout le reste"
    );
    // une grille trop petite ET un pas instable : la grille
    assert_eq!(new(pas_trop_grand, 2, &[0.0; 3], &[0.0; 3]), Some(E::GrilleTropPetite(2)));
}

/// juste : (V-P2-REF-04) `Dendrite::graine_ronde`, dans l'ordre de la doc : (1) un réglage invalide ; (2) `n < 3` : `GrilleTropPetite(n)` ;
/// (3) `n > N_MAX` : `GrilleTropGrande(n)` ; (4) `delta` nul, négatif, NaN ou infini : `ParametreInvalide("delta")` ; (5) `rayon` de même :
/// `ParametreInvalide("rayon")` ; (6) un `dt` plus grand que la borne pour `U = delta` : `PasInstable` (avec cette borne). Pour chaque paire de
/// règles voisines, les deux fausses à la fois rendent la première.
///
/// Ne couvre pas : la graine elle-même (voir `etape_p2_naif.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn graine_ronde_refus_et_ordre() {
    let r = bon();
    let g = |r: Reglages, n: usize, delta: f64, rayon: f64| {
        Dendrite::graine_ronde(r, n, Domaine::Quart, delta, rayon, ModeU::Joue).err()
    };
    assert_eq!(g(r, 8, 0.5, 3.0), None);
    for n in 0..3 {
        assert_eq!(g(r, n, 0.5, 3.0), Some(E::GrilleTropPetite(n)));
    }
    for n in [N_MAX + 1, usize::MAX] {
        assert_eq!(g(r, n, 0.5, 3.0), Some(E::GrilleTropGrande(n)));
    }
    for mauvais in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(g(r, 8, mauvais, 3.0), Some(E::ParametreInvalide("delta")), "delta = {mauvais}");
        assert_eq!(g(r, 8, 0.5, mauvais), Some(E::ParametreInvalide("rayon")), "rayon = {mauvais}");
    }
    let borne = r.dt_borne(0.5, ModeU::Joue);
    let trop = Reglages { dt: borne.next_up(), ..r };
    assert_eq!(g(trop, 8, 0.5, 3.0), Some(E::PasInstable { dt: borne.next_up(), dt_borne: borne }));
    // les paires
    assert_eq!(
        g(Reglages { dx: 0.0, ..r }, 2, f64::NAN, 3.0),
        Some(E::ParametreInvalide("dx")),
        "réglage avant grille"
    );
    assert_eq!(g(r, 2, f64::NAN, 3.0), Some(E::GrilleTropPetite(2)), "grille avant delta");
    assert_eq!(
        g(r, N_MAX + 1, f64::NAN, f64::NAN),
        Some(E::GrilleTropGrande(N_MAX + 1)),
        "grille trop grande avant delta"
    );
    assert_eq!(g(r, 8, f64::NAN, f64::NAN), Some(E::ParametreInvalide("delta")), "delta avant rayon");
    assert_eq!(g(trop, 8, 0.5, 0.0), Some(E::ParametreInvalide("rayon")), "rayon avant pas instable");
    assert_eq!(g(trop, 8, 0.0, 3.0), Some(E::ParametreInvalide("delta")), "delta avant pas instable");
    assert_eq!(g(Reglages { dt: 0.0, ..r }, 8, 0.5, 3.0), Some(E::ParametreInvalide("dt")), "dt invalide");
}

/// juste : (V-P2-REF-05) des réglages démesurés mais finis ne font pas paniquer, et la doc dit ce qu'ils donnent (mesuré sur le prototype) :
/// `w0 = 1e200` (`w0²` déborde : borne `0`) et `dx = 1e-200` (`dx²` vaut 0) : `PasInstable { dt: 0,01, dt_borne: 0 }`, `u` gelé ou joué ;
/// `d = 1e308` : `PasInstable` à `u` joué (borne 0) et accepté à `u` gelé (qui n'a pas de `dt_u`), 3 pas faits ; `w0 = dx = 1e200` :
/// `PasInstable` à `dt_borne` NaN à `u` gelé, accepté à `u` joué mais `Divergence` au premier pas (état inchangé, `pas_faits() = 0`) ;
/// enfin un `u` de `1e308` (fini : accepté au départ) à `u` joué, `lambda = 1e-300` : `Divergence` au premier pas (`2 · 1e308` déborde),
/// l'état est celui d'avant (même empreinte), alors qu'à `u` gelé le pas passe ; `dx = 1e-150` (accepté avec `dt` = sa borne, de l'ordre de
/// `1e-301`) : le carré du carré du gradient déborde, `H` vaut `inf / inf` = NaN, donc `φ` neuf est NaN : `Divergence` au premier pas, `u` gelé
/// COMME joué (un `φ` neuf NaN est refusé par lui-même, sans l'aide de `u` : le refus se lit `!(|φ| <= 1 + MARGE)`).
///
/// Ne couvre pas : tous les débordements possibles (toute autre combinaison : au moins, jamais de panique).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn reglages_demesures_sans_panique() {
    let (phi, u) = ([0.5; 16], [-0.5; 16]);
    let refus_borne_nulle = |r: Reglages, mode| {
        assert_eq!(
            Dendrite::new(r, 4, Domaine::Quart, &phi, &u, mode).err(),
            Some(E::PasInstable { dt: 0.01, dt_borne: 0.0 }),
            "{r:?} {mode:?}"
        );
    };
    for mode in [ModeU::Gele, ModeU::Joue] {
        refus_borne_nulle(Reglages { w0: 1e200, ..bon() }, mode);
        refus_borne_nulle(Reglages { dx: 1e-200, ..bon() }, mode);
    }
    refus_borne_nulle(Reglages { d: 1e308, ..bon() }, ModeU::Joue);
    let mut d = Dendrite::new(Reglages { d: 1e308, ..bon() }, 4, Domaine::Quart, &phi, &u, ModeU::Gele).unwrap();
    d.avancer(3).unwrap();
    assert_eq!(d.pas_faits(), 3);
    let nan = Reglages { w0: 1e200, dx: 1e200, ..bon() };
    match Dendrite::new(nan, 4, Domaine::Quart, &phi, &u, ModeU::Gele).err() {
        Some(E::PasInstable { dt, dt_borne }) => assert!(dt == 0.01 && dt_borne.is_nan()),
        autre => panic!("PasInstable à borne NaN attendu : {autre:?}"),
    }
    let mut d = Dendrite::new(nan, 4, Domaine::Quart, &phi, &u, ModeU::Joue).unwrap();
    let avant = d.empreinte();
    assert_eq!(d.avancer(3), Err(E::Divergence));
    assert_eq!((d.pas_faits(), d.empreinte()), (0, avant));
    let r = Reglages { lambda: 1e-300, dt: 1e-9, ..bon() };
    let mut d = Dendrite::new(r, 4, Domaine::Quart, &phi, &[1e308; 16], ModeU::Joue).unwrap();
    let avant = d.empreinte();
    assert_eq!(d.avancer(1), Err(E::Divergence));
    assert_eq!((d.pas_faits(), d.empreinte()), (0, avant));
    let mut d = Dendrite::new(r, 4, Domaine::Quart, &phi, &[1e308; 16], ModeU::Gele).unwrap();
    assert_eq!(d.avancer(1), Ok(()));
    assert_eq!(d.pas_faits(), 1);
    // dx = 1e-150 : un NaN né dans le flux, u gelé comme joué
    let rampe: Vec<f64> = (0..16).map(|k| 0.9 - 0.1 * (k % 4) as f64 - 0.05 * (k / 4) as f64).collect();
    for mode in [ModeU::Gele, ModeU::Joue] {
        let mut r = Reglages { dx: 1e-150, ..bon() };
        r.dt = r.dt_borne(0.5, mode);
        assert!(r.dt > 0.0 && r.dt < 1e-290, "la borne de dx = 1e-150 est {:e}", r.dt);
        let mut d = Dendrite::new(r, 4, Domaine::Quart, &rampe, &u, mode).unwrap();
        let avant = d.empreinte();
        assert_eq!(d.avancer(2), Err(E::Divergence), "{mode:?}");
        assert_eq!((d.pas_faits(), d.empreinte()), (0, avant));
    }
}

/// Les six familles d'états de départ de la zone sûre (quart de 16 cases de côté, `Domaine::Quart`), tirés avec `g` : (0) un front
/// (rampe circulaire de rayon `4 dx`, bruit `1e-6`) et `u` en rampe de 0 à `-U` ; (1) le même front et `u = 0` joué (la chaleur latente
/// chauffe) ; (2) `φ` tiré dans `[-1, 1]` et `u` dans `[-U, 0]` ; (3) le front, `u = -U` partout, GELÉ ; (4) `φ` tiré dans `[-1, 1]` et `u`
/// tiré dans `[-U, U]` (les deux signes), joué ; (5) un damier `φ = ±1`, `u = ±U` (le mode le plus raide de la grille), joué. Rend
/// `(φ, u, mode)`.
fn etat_tire(g: &mut Lcg, famille: usize, r: &Reglages, uu: f64) -> (Vec<f64>, Vec<f64>, ModeU) {
    let n = 16;
    let (mut phi, mut u) = (Vec::new(), Vec::new());
    for i in 0..n {
        for j in 0..n {
            let (x, y) = ((j as f64 + 0.5) * r.dx, (i as f64 + 0.5) * r.dx);
            let ray = (x * x + y * y).sqrt();
            let p0 = 0.0 - ((ray - 4.0 * r.dx.max(1.0)) / (r.w0 * std::f64::consts::SQRT_2)).clamp(-1.0, 1.0);
            let (a, b) = match famille {
                0 => (p0 + 1e-6 * (g.suivant() - 0.5), -uu * (1.0 - p0) * 0.5 + 1e-6 * (g.suivant() - 0.5)),
                1 => (p0 + 1e-6 * (g.suivant() - 0.5), 1e-6 * (g.suivant() - 0.5)),
                2 => (2.0 * g.suivant() - 1.0, -uu * g.suivant()),
                3 => (p0 + 1e-6 * (g.suivant() - 0.5), -uu),
                4 => (2.0 * g.suivant() - 1.0, uu * (2.0 * g.suivant() - 1.0)),
                _ => {
                    let damier = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                    (damier, uu * damier)
                }
            };
            phi.push(a);
            u.push(b);
        }
    }
    (phi, u, if famille == 3 { ModeU::Gele } else { ModeU::Joue })
}

/// régression : (V-P2-STAB-01) la zone sûre, MESURÉE sur notre schéma (ce n'est pas une démonstration) : sur 80 réglages tirés (`W0` de 0,5 à 2 ;
/// `tau0` de 0,2 à 5 ; `lambda` de 0,1 à 10 (produit de deux tirages : la plupart sous 2,5) ; `D` de 0,1 à 5 ; `epsilon` de 0 à 0,0665 ; `dx` de
/// 0,25 à 1,5 ; `U` de 0,05 à 1,5), six familles d'états de départ (voir `etat_tire`), quart de 16 × 16 cases, `dt` = la borne de la doc : (a) la
/// borne est acceptée par `Dendrite::new` et son flottant suivant refusé (`PasInstable` avec les deux valeurs) ; (b) 1500 pas sans `Err` (aucune
/// divergence, aucune valeur non finie) ; (c) `|φ|` ne dépasse jamais `max(1, |φ| de départ)` de plus de `1e-12` (observé : 0, exactement). Ce
/// qui est promis est à l'ÉGALITÉ `dt = dt_borne`, et seulement sur ces tirages. Hors des tests (fiche, V-P2-STAB-01) : 0 divergence et 0
/// dépassement sur 51 200 états d'une plage plus large à `dt = dt_borne`, 0 sur 25 600 à `1,1 dt_borne`, 8 à `1,25`, 273 à `1,5`.
///
/// Ne couvre pas : un `dt` plus grand que la borne ; une autre famille d'états ; une grille plus grande (voir `etape_p2_pointe.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn la_zone_sure_sans_erreur_ni_derive() {
    let mut g = Lcg(20_251_115);
    for k in 0..80 {
        let w0 = g.entre(0.5, 2.0);
        let tau0 = g.entre(0.2, 5.0);
        let lambda = (g.entre(0.0, 10.0) * g.suivant()).max(0.1);
        let d = g.entre(0.1, 5.0);
        let epsilon = g.entre(0.0, 0.0665);
        let dx = g.entre(0.25, 1.5);
        let uu = g.entre(0.05, 1.5);
        for famille in 0..6 {
            let r0 = Reglages { w0, tau0, lambda, d, epsilon, dx, dt: 1.0 };
            let (phi, u, mode) = etat_tire(&mut g, famille, &r0, uu);
            let u_max = u.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            let borne = r0.dt_borne(u_max, mode);
            assert!(borne > 0.0 && borne.is_finite(), "tirage {k} : la borne est {borne}");
            let r = Reglages { dt: borne, ..r0 };
            let trop = Reglages { dt: borne.next_up(), ..r0 };
            assert_eq!(
                Dendrite::new(trop, 16, Domaine::Quart, &phi, &u, mode).err(),
                Some(E::PasInstable { dt: borne.next_up(), dt_borne: borne }),
                "tirage {k} famille {famille}"
            );
            let mut s = Dendrite::new(r, 16, Domaine::Quart, &phi, &u, mode)
                .unwrap_or_else(|e| panic!("tirage {k} famille {famille} : {e:?} pour {r:?}"));
            let depart = phi.iter().fold(1.0_f64, |m, v| m.max(v.abs()));
            for _ in 0..15 {
                s.avancer(100)
                    .unwrap_or_else(|e| panic!("tirage {k} famille {famille} : {e:?} pour {r:?} U = {u_max}"));
                let m = s.phi().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
                assert!(m <= depart + 1e-12, "tirage {k} famille {famille} : |phi| = {m} > {depart}");
            }
            assert_eq!(s.pas_faits(), 1500);
        }
    }
}

/// juste : (V-P2-MARGE-01) la marge de `φ` (`MARGE = 0,001`) est franchie, et un pas qui la franchit est refusé SANS gâter l'état : une cellule
/// uniforme (`n = 4`, `u = -1` gelé, `lambda = 1000`, `W0 = tau0 = D = dx = 1`, `epsilon = 0,05`, `dt` la borne de la doc) partant de
/// `φ = 1 + MARGE - 2,5e-6` (accepté au départ) est poussée vers le haut (`λ|u|(1-φ²)² > |φ - φ³|` pour `φ` à `1 + MARGE`) : le naïf
/// de `etape_p2_commun` donne le premier pas `k` où `|φ| > 1 + MARGE` ; la bibliothèque : `avancer(1)` pas à pas réussit jusqu'au pas `k - 1`
/// (mêmes bits que le naïf), puis rend `Err(Divergence)` au pas `k`, avec `pas_faits() = k - 1` et l'état (bits, empreinte) d'avant ; et un
/// `avancer(k + 5)` d'un coup rend la même erreur avec `pas_faits() = k - 1` (les pas faits avant restent faits). L'état n'est donc jamais
/// « fini mais faux » au-delà de la marge. Le départ à exactement `1 + MARGE` (pas franchi au départ) est accepté, et franchi au pas 1.
/// La marge vaut des DEUX côtés : les mêmes cas en miroir (`φ` et `u` de signes opposés : `φ = -(1 + MARGE - 2,5e-6)`, `u = +1`) franchissent
/// `-(1 + MARGE)` au même pas et sont refusés de même. Et elle est INCLUSE en marche comme au départ : une cellule uniforme à `φ = 1 + MARGE`
/// exactement, avec le `u` gelé qui annule sa vitesse (`u = (φ - φ³) / (λ (1 - φ²)²)`, calculé ici : le reste d'arrondi déplace `φ` de moins
/// d'un demi-dernier bit), fait 5 pas sans erreur et sans que `φ` change d'un bit.
///
/// Ne couvre pas : la valeur de la marge (voir `etats_de_depart_refuses_et_leur_ordre`, qui la fixe) ; une dérive lente en zone sûre
/// (voir `la_zone_sure_sans_erreur_ni_derive`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn un_pas_qui_franchit_la_marge_est_refuse_avec_l_etat_d_avant() {
    let mut r = Reglages { w0: 1.0, tau0: 1.0, lambda: 1000.0, d: 1.0, epsilon: 0.05, dx: 1.0, dt: 1.0 };
    r.dt = pas_de_temps(&r, 1.0, ModeU::Gele);
    let limite = 1.0 + MARGE;
    for (depart, premier, signe) in [
        (limite - 2.5e-6, None, 1.0),
        (limite, Some(1_u64), 1.0),
        (limite - 2.5e-6, None, -1.0),
        (limite, Some(1), -1.0),
    ] {
        let (phi, u) = (vec![signe * depart; 16], vec![-signe; 16]);
        // le naïf : le premier pas qui franchit
        let mut k = 0;
        let mut avant = naif(&r, 4, &phi, &u, ModeU::Gele, 0, None);
        loop {
            let apres = naif(&r, 4, &phi, &u, ModeU::Gele, k + 1, None);
            if apres.phi.iter().any(|x| x.abs() > limite) {
                break;
            }
            avant = apres;
            k += 1;
            assert!(k < 200, "la cellule ne franchit pas la marge");
        }
        let k = k + 1; // le pas fautif
        if let Some(p) = premier {
            assert_eq!(k, p);
        } else {
            assert!(k >= 2, "le pas fautif devait être au moins le deuxième (k = {k})");
        }
        // pas à pas
        let mut s = Dendrite::new(r, 4, Domaine::Quart, &phi, &u, ModeU::Gele).unwrap();
        for _ in 0..k - 1 {
            s.avancer(1).unwrap();
        }
        assert_eq!(bits(s.phi()), bits(&avant.phi));
        let (emp, pas) = (s.empreinte(), s.pas_faits());
        assert_eq!(s.avancer(1), Err(E::Divergence));
        assert_eq!((s.pas_faits(), s.empreinte()), (pas, emp.clone()), "l'état doit être celui d'avant le pas fautif");
        assert_eq!(pas, k - 1);
        // en un seul appel
        let mut t = Dendrite::new(r, 4, Domaine::Quart, &phi, &u, ModeU::Gele).unwrap();
        assert_eq!(t.avancer(k + 5), Err(E::Divergence));
        assert_eq!((t.pas_faits(), t.empreinte()), (k - 1, emp));
        // avancer(0) ne fait rien, même après une erreur
        assert_eq!(t.avancer(0), Ok(()));
        assert_eq!(t.pas_faits(), k - 1);
    }
    // la marge est incluse en marche : une cellule qui reste à exactement 1 + MARGE n'est pas une divergence
    for signe in [1.0, -1.0] {
        let ph: f64 = signe * limite;
        let qq = 1.0 - ph * ph;
        let u0 = (ph - (ph * ph) * ph) / (r.lambda * (qq * qq));
        assert!((u0.abs() - 0.5).abs() < 1e-3, "le u qui annule la vitesse vaut {u0}");
        let mut s = Dendrite::new(r, 4, Domaine::Quart, &[ph; 16], &[u0; 16], ModeU::Gele).unwrap();
        assert_eq!(s.avancer(5), Ok(()), "phi = {ph} : rester à la marge n'est pas diverger");
        assert_eq!((s.pas_faits(), bits(s.phi())), (5, bits(&[ph; 16])));
    }
}

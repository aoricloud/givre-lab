//! Étape P1 : les entrées refusées (jamais de panique), le pas maximal et ce qu'il garantit. Valeurs attendues :
//! `references/etape-p1-valeurs.md`, identifiants `V-P1-REF-xx` et `V-P1-STAB-xx`.

mod etape_p1_commun;

use etape_p1_commun::*;
use givre_lab::phase::{Diffusion2d, ErreurPhase, Front1d, Front1dFixe, ModeU, Parametres};

fn bon() -> Parametres {
    param(1.0, 1.0, 1.0, 1.0, 1.0, 0.1)
}

/// juste : (V-P1-REF-01) `dt_max = min( dx² / (2 d), tau dx² / (2 w² + dx²) )`, à la main : `w = tau = d = dx = 1` : `min(1/2, 1/3) = 1/3`
/// (`1.0 / 3.0`) ; `w = 1, tau = 4, d = 1/4, dx = 1` : `min(2, 4/3) = 4/3` ; `w = 0,1, tau = 1, d = 2, dx = 1` : `min(1/4, 1/1,02) = 1/4`.
/// Le pas `dt = dt_max` est ACCEPTÉ ; le flottant suivant `dt_max` est refusé par `PasInstable { dt, dt_max }` (les deux valeurs rendues).
///
/// Ne couvre pas : la stabilité réelle (la borne est nécessaire, pas suffisante : voir `le_pas_maximal_n_est_pas_suffisant`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn pas_maximal_a_la_main() {
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 1.0, 0.1).dt_max(), 1.0 / 3.0);
    assert_eq!(param(1.0, 4.0, 1.0, 0.25, 1.0, 0.1).dt_max(), 4.0 / 3.0);
    assert_eq!(param(0.1, 1.0, 1.0, 2.0, 1.0, 0.1).dt_max(), 0.25);
    for base in [bon(), param(1.3, 2.1, 0.7, 0.4, 0.55, 0.01)] {
        let dt_max = base.dt_max();
        let ok = Parametres { dt: dt_max, ..base };
        assert_eq!(ok.verifier(), Ok(()));
        let phi = [1.0, 0.0, -1.0];
        assert!(Front1d::new(ok, &phi, &[0.0; 3], ModeU::Gele).is_ok());
        let trop = Parametres { dt: dt_max.next_up(), ..base };
        assert_eq!(trop.verifier(), Err(ErreurPhase::PasInstable { dt: dt_max.next_up(), dt_max }));
        assert_eq!(
            Front1d::new(trop, &phi, &[0.0; 3], ModeU::Gele).err(),
            Some(ErreurPhase::PasInstable { dt: dt_max.next_up(), dt_max })
        );
        assert_eq!(
            Front1dFixe::new(trop, &phi, &[0.0; 3], ModeU::Gele).err(),
            Some(ErreurPhase::PasInstable { dt: dt_max.next_up(), dt_max })
        );
    }
}

/// juste : (V-P1-REF-02) chaque réglage hors plage est refusé, par `ParametreInvalide(nom)`, dans l'ordre de la doc (`w`, `tau`, `lambda`, `d`,
/// `dx`, `dt`) : zéro, négatif, NaN, infini ; puis les constantes (`c_phi = w²/dx²`, `c_u = d dt/dx²`, `r = dt/tau`, `lambda`) plus grandes
/// que `2^20 = 1 048 576` : `dx = 9e-4` donne `"c_phi"` (`1/dx² = 1,2e6`), `lambda = 2e6` donne `"lambda"`, `dt = 2e6, d = 1e-9` donne `"r"` ;
/// `1e6` (< `2^20`) passe ; et, quand plusieurs réglages sont faux, le PREMIER de la liste est nommé.
///
/// Ne couvre pas : l'ordre entre `c_u` et `r`, la valeur exacte de la borne (voir `bornes_exactes_et_ordre_des_constantes`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn reglages_refuses() {
    let noms = ["w", "tau", "lambda", "d", "dx", "dt"];
    for (idx, nom) in noms.iter().enumerate() {
        for mauvais in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut p = [1.0, 1.0, 1.0, 1.0, 1.0, 0.1];
            p[idx] = mauvais;
            let p = param(p[0], p[1], p[2], p[3], p[4], p[5]);
            assert_eq!(p.verifier(), Err(ErreurPhase::ParametreInvalide(nom)), "{nom} = {mauvais}");
            assert_eq!(
                Front1d::new(p, &[1.0, 0.0, -1.0], &[0.0; 3], ModeU::Gele).err(),
                Some(ErreurPhase::ParametreInvalide(nom))
            );
        }
    }
    assert_eq!(param(f64::NAN, -1.0, 0.0, 1.0, 1.0, 0.1).verifier(), Err(ErreurPhase::ParametreInvalide("w")));
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 9e-4, 1e-9).verifier(), Err(ErreurPhase::ParametreInvalide("c_phi")));
    assert_eq!(param(1.0, 1.0, 2e6, 1.0, 1.0, 0.1).verifier(), Err(ErreurPhase::ParametreInvalide("lambda")));
    assert_eq!(param(1.0, 1.0, 1.0, 1e-9, 1.0, 2e6).verifier(), Err(ErreurPhase::ParametreInvalide("r")));
    // 1e6 < 2^20 passe (et la stabilité est vérifiée après : dt <= dt_max)
    assert_eq!(param(1.0, 1.0, 1e6, 1.0, 1.0, 0.1).verifier(), Ok(()));
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 1.0 / 1000.0, 1e-7).verifier(), Ok(()));
}

/// juste : (V-P1-REF-03) les états de départ refusés, dans l'ordre de la doc (d'abord les réglages) : moins de 3 cases (0, 1, 2) :
/// `GrilleTropPetite(n)` ; `phi` et `u` de longueurs différentes : `LongueursDifferentes` ; une valeur NaN ou infinie dans `phi` ou `u` :
/// `ValeurDeDepart` ; pour `Front1dFixe` en plus, une valeur de `2^31` ou plus (`3e9`, `-3e9`) : `ValeurDeDepart` (acceptée en `f64` : qui ne
/// panique pas non plus) ; `2^31 - 1` n'est pas refusé. Des réglages faux l'emportent sur une grille trop petite, et une grille trop
/// petite sur des longueurs différentes.
///
/// Ne couvre pas : `Diffusion2d` (voir le test suivant).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn etats_de_depart_refuses() {
    let p = bon();
    for n in 0..3_usize {
        let v = vec![0.0; n];
        assert_eq!(Front1d::new(p, &v, &v, ModeU::Gele).err(), Some(ErreurPhase::GrilleTropPetite(n)));
        assert_eq!(Front1dFixe::new(p, &v, &v, ModeU::Joue).err(), Some(ErreurPhase::GrilleTropPetite(n)));
    }
    assert_eq!(Front1d::new(p, &[1.0; 4], &[0.0; 3], ModeU::Gele).err(), Some(ErreurPhase::LongueursDifferentes));
    assert_eq!(Front1dFixe::new(p, &[1.0; 3], &[0.0; 5], ModeU::Gele).err(), Some(ErreurPhase::LongueursDifferentes));
    for mauvais in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            Front1d::new(p, &[1.0, mauvais, -1.0], &[0.0; 3], ModeU::Gele).err(),
            Some(ErreurPhase::ValeurDeDepart)
        );
        assert_eq!(
            Front1d::new(p, &[1.0, 0.0, -1.0], &[0.0, 0.0, mauvais], ModeU::Joue).err(),
            Some(ErreurPhase::ValeurDeDepart)
        );
        assert_eq!(
            Front1dFixe::new(p, &[mauvais, 0.0, -1.0], &[0.0; 3], ModeU::Gele).err(),
            Some(ErreurPhase::ValeurDeDepart)
        );
    }
    assert!(Front1d::new(p, &[3e9, 0.0, -3e9], &[0.0; 3], ModeU::Gele).is_ok());
    assert_eq!(Front1dFixe::new(p, &[3e9, 0.0, -1.0], &[0.0; 3], ModeU::Gele).err(), Some(ErreurPhase::ValeurDeDepart));
    assert_eq!(
        Front1dFixe::new(p, &[1.0, 0.0, -1.0], &[0.0, -3e9, 0.0], ModeU::Gele).err(),
        Some(ErreurPhase::ValeurDeDepart)
    );
    assert!(Front1dFixe::new(p, &[2_147_483_647.0, 0.0, -1.0], &[0.0; 3], ModeU::Gele).is_ok());
    let mauvais = param(0.0, 1.0, 1.0, 1.0, 1.0, 0.1);
    assert_eq!(Front1d::new(mauvais, &[1.0], &[0.0], ModeU::Gele).err(), Some(ErreurPhase::ParametreInvalide("w")));
    // la grille trop petite l'emporte sur les longueurs différentes
    assert_eq!(Front1d::new(p, &[1.0; 2], &[0.0; 5], ModeU::Gele).err(), Some(ErreurPhase::GrilleTropPetite(2)));
    assert_eq!(Front1dFixe::new(p, &[1.0; 2], &[0.0; 5], ModeU::Gele).err(), Some(ErreurPhase::GrilleTropPetite(2)));
}

/// juste : (V-P1-REF-04) `Diffusion2d::new` refuse, dans l'ordre de la doc : `n < 2` : `GrilleTropPetite(n)` ; `depart.len() != n * n` :
/// `LongueursDifferentes` ; `k` nul, négatif, NaN, infini ou plus grand que 0,25 : `ParametreInvalide("k")` (0,25 est accepté, le flottant suivant
/// est refusé) ; une valeur NaN ou infinie : `ValeurDeDepart` ; et, quand plusieurs entrées sont fausses, la première de cet ordre
/// est nommée.
///
/// Ne couvre pas : `n` énorme (voir `diffusion_2d_cote_demesure`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn diffusion_2d_refus() {
    assert_eq!(Diffusion2d::new(0, 0.2, &[]).err(), Some(ErreurPhase::GrilleTropPetite(0)));
    assert_eq!(Diffusion2d::new(1, 0.2, &[0.0]).err(), Some(ErreurPhase::GrilleTropPetite(1)));
    assert_eq!(Diffusion2d::new(3, 0.2, &[0.0; 8]).err(), Some(ErreurPhase::LongueursDifferentes));
    for k in [0.0, -0.1, f64::NAN, f64::INFINITY, 0.25_f64.next_up()] {
        assert_eq!(Diffusion2d::new(2, k, &[0.0; 4]).err(), Some(ErreurPhase::ParametreInvalide("k")), "k = {k}");
    }
    assert!(Diffusion2d::new(2, 0.25, &[0.0; 4]).is_ok());
    assert_eq!(Diffusion2d::new(2, 0.2, &[0.0, f64::NAN, 0.0, 0.0]).err(), Some(ErreurPhase::ValeurDeDepart));
    assert_eq!(Diffusion2d::new(2, 0.2, &[0.0, 0.0, f64::INFINITY, 0.0]).err(), Some(ErreurPhase::ValeurDeDepart));
    // l'ordre, quand plusieurs entrées sont fausses : le côté, puis les longueurs, puis `k`, puis les valeurs
    assert_eq!(Diffusion2d::new(1, f64::NAN, &[]).err(), Some(ErreurPhase::GrilleTropPetite(1)));
    assert_eq!(Diffusion2d::new(3, f64::NAN, &[0.0; 8]).err(), Some(ErreurPhase::LongueursDifferentes));
    assert_eq!(Diffusion2d::new(2, 0.0, &[f64::NAN; 4]).err(), Some(ErreurPhase::ParametreInvalide("k")));
}

/// juste : (V-P1-REF-05) le débordement ne fait pas paniquer : `Front1dFixe` partant de `phi = [1e6, -1e6, 1e6]` (valeurs permises, hors du
/// régime stable : le cube dépasse `2^31`) rend `Err(Debordement)` au premier pas, l'état est inchangé (`pas_faits() = 0`, mêmes champs
/// bruts) et l'empreinte est celle d'avant ; `Front1d` partant de `phi = [1e200, 0, -1e200]` (le cube est infini) rend aussi `Err(Debordement)`
/// et garde l'état de départ ; et, pour les deux, un pas sain avant le pas fautif reste compté (`phi = [1e60, 0, -1e60]` en `f64`, `[40, 0, -40]` en `Fixe` : `pas_faits() = 1` après `avancer(5)`, l'état est celui d'un seul pas).
///
/// Ne couvre pas : l'endroit exact du débordement en régime presque stable.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn debordement_sans_panique() {
    let p = bon();
    let mut f = Front1dFixe::new(p, &[1e6, -1e6, 1e6], &[0.0; 3], ModeU::Joue).unwrap();
    let (avant, e0) = (f.phi().to_vec(), f.empreinte());
    assert_eq!(f.avancer(1), Err(ErreurPhase::Debordement));
    assert_eq!((f.pas_faits(), f.phi().to_vec(), f.empreinte()), (0, avant, e0));
    let mut g = Front1d::new(p, &[1e200, 0.0, -1e200], &[0.0; 3], ModeU::Gele).unwrap();
    assert_eq!(g.avancer(5), Err(ErreurPhase::Debordement));
    assert_eq!(g.pas_faits(), 0);
    assert_eq!(g.phi(), [1e200, 0.0, -1e200]);
    // un pas sain, puis un pas fautif : le pas sain reste fait, l'état est celui d'après lui
    let mut h = Front1d::new(p, &[1e60, 0.0, -1e60], &[0.0; 3], ModeU::Gele).unwrap();
    assert_eq!(h.avancer(5), Err(ErreurPhase::Debordement));
    assert_eq!(h.pas_faits(), 1);
    assert!(h.phi().iter().all(|x| x.is_finite()) && h.phi()[0].abs() > 1e170);
    let mut k = Front1dFixe::new(p, &[40.0, 0.0, -40.0], &[0.0; 3], ModeU::Gele).unwrap();
    assert_eq!(k.avancer(5), Err(ErreurPhase::Debordement));
    assert_eq!(k.pas_faits(), 1);
    let mut mem = Front1dFixe::new(p, &[40.0, 0.0, -40.0], &[0.0; 3], ModeU::Gele).unwrap();
    mem.avancer(1).unwrap();
    assert_eq!(k.empreinte(), mem.empreinte());
}

/// juste : (V-P1-REF-11) `dt_max()` au bit près, dans l'ordre d'opérations de la doc (`a = (dx * dx) / (2.0 * d)`,
/// `b = (tau * (dx * dx)) / (2.0 * (w * w) + dx * dx)`, `a.min(b)`), recalculé ici sur 300 réglages tirés (`w`, `tau`, `d`, `dx` de 0,05
/// à 5) : les deux bornes l'emportent chacune au moins 50 fois. (Les autres tests prennent leur `dt` de `dt_max()` : sans ce test, un
/// `dt_max` arrondi autrement décalerait la bibliothèque et le naïf ensemble, sans se voir.)
///
/// Ne couvre pas : les réglages démesurés (voir `reglages_demesures_sans_panique`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn pas_maximal_bit_pour_bit() {
    let mut g = Lcg(86);
    let (mut par_u, mut par_phi) = (0, 0);
    for _ in 0..300 {
        let (w, tau, d, dx) = (g.entre(0.05, 5.0), g.entre(0.05, 5.0), g.entre(0.05, 5.0), g.entre(0.05, 5.0));
        let a = (dx * dx) / (2.0 * d);
        let b = (tau * (dx * dx)) / (2.0 * (w * w) + dx * dx);
        let attendu = if a < b { a } else { b };
        assert_eq!(
            param(w, tau, 1.0, d, dx, 0.1).dt_max().to_bits(),
            attendu.to_bits(),
            "w={w} tau={tau} d={d} dx={dx}"
        );
        if a < b {
            par_u += 1;
        } else {
            par_phi += 1;
        }
    }
    assert!(par_u >= 50 && par_phi >= 50, "les deux bornes devaient l'emporter : {par_u} et {par_phi}");
}

/// juste : (V-P1-REF-06) la borne des constantes est `2^20`, incluse, et leur ordre est celui de la doc : `lambda = 2^20` passe, le
/// flottant suivant est refusé (`"lambda"`) ; `c_phi = 2^20` exactement (`w = 1024`, `dx = 1`) passe, le flottant suivant de `w` est
/// refusé (`"c_phi"`) ; `c_u` est nommé avant `r` quand les deux dépassent (`d = 1`, `dx = 1`, `tau = 1`, `dt = 2e6` : `"c_u"`), et
/// `c_phi` avant `c_u` (`dx = 9e-4`, `dt = 2e6` : `"c_phi"`).
///
/// Ne couvre pas : `r = 2^20` exactement.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn bornes_exactes_et_ordre_des_constantes() {
    let deux_20 = 1_048_576.0_f64;
    assert_eq!(param(1.0, 1.0, deux_20, 1.0, 1.0, 0.1).verifier(), Ok(()));
    assert_eq!(
        param(1.0, 1.0, deux_20.next_up(), 1.0, 1.0, 0.1).verifier(),
        Err(ErreurPhase::ParametreInvalide("lambda"))
    );
    assert_eq!(param(1024.0, 1.0, 1.0, 1.0, 1.0, 1e-9).verifier(), Ok(()));
    assert_eq!(
        param(1024.0_f64.next_up(), 1.0, 1.0, 1.0, 1.0, 1e-9).verifier(),
        Err(ErreurPhase::ParametreInvalide("c_phi"))
    );
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 1.0, 2e6).verifier(), Err(ErreurPhase::ParametreInvalide("c_u")));
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 9e-4, 2e6).verifier(), Err(ErreurPhase::ParametreInvalide("c_phi")));
}

/// juste : (V-P1-REF-07) des réglages finis mais démesurés ne font pas paniquer, et la doc dit ce qu'ils donnent : `w = dx = 1e200`
/// (les deux carrés débordent : `c_phi` est NaN) : `ParametreInvalide("c_phi")` ; `d = 1e308`, `dx = 1e200`, `dt = 1e10` (`c_phi = 0`,
/// `c_u` NaN) : `ParametreInvalide("c_u")` ; `d = 1e308`, `dx = 1e200`, `dt = 1` (`c_u = 0`, mais `dt_max()` est NaN) :
/// `PasInstable` avec ce `dt` et un `dt_max` NaN ; `dx = 1e200` seul (`c_phi = c_u = 0`, `dt_max() = +∞` : `a.min(b)` rend `a` quand `b`
/// est NaN) : accepté, et les deux fronts se construisent et avancent (il ne reste que la relaxation locale). `dx = 1e-200` (`dx²`
/// nul, `c_phi` infini) : `ParametreInvalide("c_phi")`.
///
/// Ne couvre pas : tous les débordements possibles (voir `balayage_sans_panique`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn reglages_demesures_sans_panique() {
    assert_eq!(param(1e200, 1.0, 1.0, 1.0, 1e200, 1.0).verifier(), Err(ErreurPhase::ParametreInvalide("c_phi")));
    assert_eq!(param(1.0, 1.0, 1.0, 1e308, 1e200, 1e10).verifier(), Err(ErreurPhase::ParametreInvalide("c_u")));
    match param(1.0, 1.0, 1.0, 1e308, 1e200, 1.0).verifier() {
        Err(ErreurPhase::PasInstable { dt, dt_max }) => {
            assert!(dt == 1.0 && dt_max.is_nan(), "dt {dt} dt_max {dt_max}")
        }
        autre => panic!("un dt_max NaN devait être refusé par PasInstable : {autre:?}"),
    }
    let loin = param(1.0, 1.0, 1.0, 1.0, 1e200, 0.5);
    assert_eq!(loin.dt_max(), f64::INFINITY);
    assert_eq!(loin.verifier(), Ok(()));
    let mut a = Front1d::new(loin, &[0.5, 0.25, -0.5], &[0.0; 3], ModeU::Joue).unwrap();
    assert_eq!(a.avancer(3), Ok(()));
    // sans diffusion (c_phi = 0) : un pas de la relaxation locale, phi + r (phi - phi³), à la main
    let mut b = Front1dFixe::new(loin, &[0.5, 0.25, -0.5], &[0.0; 3], ModeU::Gele).unwrap();
    assert_eq!(b.avancer(1), Ok(()));
    let vers: Vec<f64> = b.phi().iter().map(|x| x.vers_f64()).collect();
    assert_eq!(vers, [0.5 + 0.5 * 0.375, 0.25 + 0.5 * (0.25 - 1.0 / 64.0), -0.5 - 0.5 * 0.375]);
    assert_eq!(param(1.0, 1.0, 1.0, 1.0, 1e-200, 1e-300).verifier(), Err(ErreurPhase::ParametreInvalide("c_phi")));
}

/// juste : (V-P1-REF-08) un côté démesuré de `Diffusion2d` ne fait pas paniquer : `n = usize::MAX` et `n = 2^(bits/2)` (dont le carré
/// déborde un `usize` ; en arithmétique qui reboucle il vaudrait 0, la longueur d'un départ vide) rendent `LongueursDifferentes`.
///
/// Ne couvre pas : une grande grille réelle (la mémoire).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn diffusion_2d_cote_demesure() {
    assert_eq!(Diffusion2d::new(usize::MAX, 0.2, &[]).err(), Some(ErreurPhase::LongueursDifferentes));
    assert_eq!(Diffusion2d::new(1_usize << (usize::BITS / 2), 0.2, &[]).err(), Some(ErreurPhase::LongueursDifferentes));
    assert_eq!(Diffusion2d::new(usize::MAX, 0.2, &[0.0; 4]).err(), Some(ErreurPhase::LongueursDifferentes));
}

/// juste : (V-P1-REF-09) deux bords du débordement : (a) en `f64`, `u` seul peut cesser d'être fini (`phi = [1, 1, 1]` ne bouge pas,
/// `u = [1e308, -1e308, 1e308]` joué : le laplacien de `u` est infini) : `Err(Debordement)`, état inchangé, zéro pas ; le même
/// départ avec `u` gelé avance sans erreur (rien n'est calculé sur `u`) ; (b) en `Fixe`, la valeur de départ `2^31` exactement et
/// `-2^31` sont refusées (`ValeurDeDepart` : la doc dit « valeur absolue `>= 2^31` »), dans `phi` comme dans `u`.
///
/// Ne couvre pas : les débordements en cours de calcul (voir `debordement_sans_panique`).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn debordement_de_u_seul_et_bord_de_la_virgule_fixe() {
    let p = bon();
    let (phi, u) = ([1.0, 1.0, 1.0], [1e308, -1e308, 1e308]);
    let mut f = Front1d::new(p, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(f.avancer(2), Err(ErreurPhase::Debordement));
    assert_eq!((f.pas_faits(), f.phi(), f.u()), (0, &phi[..], &u[..]));
    let mut g = Front1d::new(p, &phi, &u, ModeU::Gele).unwrap();
    assert_eq!(g.avancer(2), Ok(()));
    assert_eq!((g.pas_faits(), g.phi(), g.u()), (2, &phi[..], &u[..]));
    let deux_31 = 2_147_483_648.0;
    for mauvais in [deux_31, -deux_31] {
        assert_eq!(
            Front1dFixe::new(p, &[1.0, mauvais, -1.0], &[0.0; 3], ModeU::Gele).err(),
            Some(ErreurPhase::ValeurDeDepart),
            "phi = {mauvais}"
        );
        assert_eq!(
            Front1dFixe::new(p, &[1.0, 0.0, -1.0], &[0.0, 0.0, mauvais], ModeU::Joue).err(),
            Some(ErreurPhase::ValeurDeDepart),
            "u = {mauvais}"
        );
    }
}

/// Un flottant `m · 2^e` tiré : `m` dans `[1, 2[`, `e` entier entre `-e_max` et `e_max` (des opérations exactes seulement, pour que
/// le tirage soit le même sur toutes les machines).
fn tire(g: &mut Lcg, e_max: i32) -> f64 {
    let e = (g.suivant() * (2 * e_max + 1) as f64) as i32 - e_max;
    g.entre(1.0, 2.0) * 2f64.powi(e)
}

/// juste : (V-P1-REF-10) jamais de panique sur toute la plage permise : 600 réglages tirés (un sur trois sur toute l'étendue des
/// `f64`, de `2^-1000` à `2^1000` ; les autres de `2^-13` à `2^13` ; `lambda` de `2^-20` à `2^20` ; `dt` égal à `dt_max()`, ou une
/// fraction de lui), 3 à 32 cases, des départs dans `[-1,3 ; 1,3]`, minuscules (`1e-300`), ou énormes (`30`, `1e5`, jusqu'à
/// `2^31 - 1`), `u` gelé ou joué, 200 pas, en `f64` ET en `Fixe`. Chaque construction rend `Ok` ou une `ErreurPhase` ; chaque
/// `avancer` rend `Ok(())` (et 200 pas sont faits) ou `Err(Debordement)` (moins de 200) ; l'état `f64` reste fini ; `position_front`,
/// `temps`, `empreinte` ne paniquent pas ; `f64` et `Fixe` refusent les mêmes réglages. Les quatre issues se produisent (le balayage
/// n'est pas à vide) : au moins 150 constructions, 150 refus, 50 courses entières et 50 débordements.
///
/// Ne couvre pas : les réglages que le tirage n'atteint pas ; la justesse des états (voir les autres fichiers).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn balayage_sans_panique() {
    let mut g = Lcg(20_261_114);
    let (mut construits, mut refuses, mut entiers, mut debordes) = (0, 0, 0, 0);
    for essai in 0..600_u32 {
        let e_max = if essai % 3 == 0 { 1000 } else { 13 };
        let (w, tau, lambda, d, dx) =
            (tire(&mut g, e_max), tire(&mut g, e_max), tire(&mut g, 20), tire(&mut g, e_max), tire(&mut g, e_max));
        let mut p = param(w, tau, lambda, d, dx, 0.0);
        let dt_max = p.dt_max();
        p.dt = match essai % 5 {
            0 => dt_max,
            1 => dt_max * 2f64.powi(-((g.suivant() * 40.0) as i32)),
            _ => dt_max * g.entre(0.01, 1.0),
        };
        let n = 3 + (g.suivant() * 30.0) as usize;
        let ampleur = [1.3, 1.3, 1.3, 1e-300, 30.0, 1e5, 2_147_483_647.0][(essai % 7) as usize];
        let phi: Vec<f64> = (0..n).map(|_| g.entre(-ampleur, ampleur)).collect();
        let u: Vec<f64> = (0..n).map(|_| g.entre(-ampleur, ampleur)).collect();
        let mode = if essai % 2 == 0 { ModeU::Joue } else { ModeU::Gele };
        let a = Front1d::new(p, &phi, &u, mode);
        let b = Front1dFixe::new(p, &phi, &u, mode);
        assert_eq!(a.as_ref().err(), b.as_ref().err(), "essai {essai} : f64 et Fixe devaient refuser de même ({p:?})");
        let (Ok(mut a), Ok(mut b)) = (a, b) else {
            refuses += 1;
            continue;
        };
        construits += 1;
        for (nom, issue, faits) in [("f64", a.avancer(200), a.pas_faits()), ("Fixe", b.avancer(200), b.pas_faits())] {
            match issue {
                Ok(()) => {
                    assert_eq!(faits, 200, "essai {essai} {nom}");
                    entiers += 1;
                }
                Err(ErreurPhase::Debordement) => {
                    assert!(faits < 200, "essai {essai} {nom}");
                    debordes += 1;
                }
                Err(autre) => panic!("essai {essai} {nom} : avancer a rendu {autre:?}"),
            }
        }
        assert!(a.phi().iter().chain(a.u().iter()).all(|x| x.is_finite()), "essai {essai} : état f64 non fini");
        assert_eq!((a.phi().len(), b.phi().len(), b.u().len()), (n, n, n));
        let _ = (a.position_front(), b.position_front(), a.temps(), b.temps());
        assert_eq!((a.empreinte().len(), b.empreinte().len()), (64, 64));
    }
    assert!(
        construits >= 150 && refuses >= 150 && entiers >= 50 && debordes >= 50,
        "le balayage devait donner les quatre issues : {construits} construits, {refuses} refusés, {entiers} courses entières, {debordes} débordements"
    );
}

/// Le front de Stefan des tests de stabilité : `n` cases, `phi = -clamp((x - x0) / 2, -1, 1)` (linéaire par morceaux : aucune
/// fonction du système), `u = 0` dans le solide (`x < x0`) et `-u_liquide` ailleurs, plus un bruit de `1e-6` sur `phi`
/// (borné à `[-1, 1]`).
fn front_de_stefan(n: usize, dx: f64, x0: f64, u_liquide: f64, graine: u64) -> (Vec<f64>, Vec<f64>) {
    let mut g = Lcg(graine);
    let phi = (0..n)
        .map(|i| (-(((i as f64 + 0.5) * dx - x0) / 2.0).clamp(-1.0, 1.0) + 1e-6 * (g.suivant() - 0.5)).clamp(-1.0, 1.0))
        .collect();
    let u = (0..n).map(|i| if (i as f64 + 0.5) * dx < x0 { 0.0 } else { -u_liquide }).collect();
    (phi, u)
}

/// juste : (V-P1-STAB-01) la borne `dt_max` est NÉCESSAIRE, pas suffisante (la doc le dit) : `W = 1,25`, `tau = 2`, `lambda = 1`,
/// `D = 1`, `dx = 0,5` (donc `dt_max = dx² / (2 D) = 0,125`, la borne de la diffusion de `u`), un front de Stefan (`u = -0,5` dans le
/// liquide), `u` joué. À `dt = dt_max`, accepté par `verifier`, le schéma explose : le facteur du mode en dents de scie de `u` est
/// `1 - 4 c_u - ½ r lambda (1 - phi²)² < -1` dans l'interface (analyse linéaire : `c_u = 1/2`) ; `avancer` rend `Err(Debordement)` avant
/// 20 000 pas, sans panique, l'état rendu est fini. À `dt = 0,9 dt_max` (sous la borne couplée `1 / (2 D/dx² + lambda/(4 tau))
/// = 0,9846 dt_max`), 20 000 pas se font et `phi` reste dans `[-1, 1]` à `1e-9` près. En `f64` et en `Fixe`.
///
/// Ne couvre pas : la valeur exacte du seuil (mesuré depuis un profil en `tanh`, non gelé : rien à 0,98 `dt_max`, des dents de scie à
/// 0,99, un débordement à 0,995).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn le_pas_maximal_n_est_pas_suffisant() {
    let base = param(1.25, 2.0, 1.0, 1.0, 0.5, 0.0);
    assert_eq!(base.dt_max(), 0.125);
    let (phi, u) = front_de_stefan(200, 0.5, 10.0, 0.5, 1);
    let au_bord = Parametres { dt: base.dt_max(), ..base };
    assert_eq!(au_bord.verifier(), Ok(()));
    let mut a = Front1d::new(au_bord, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(a.avancer(20_000), Err(ErreurPhase::Debordement));
    assert!(a.pas_faits() > 0 && a.pas_faits() < 20_000);
    assert!(a.phi().iter().chain(a.u().iter()).all(|x| x.is_finite()));
    let mut b = Front1dFixe::new(au_bord, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(b.avancer(20_000), Err(ErreurPhase::Debordement));
    assert!(b.pas_faits() > 0 && b.pas_faits() < 20_000);
    let dessous = Parametres { dt: 0.9 * base.dt_max(), ..base };
    assert!(dessous.dt < dt_sur(&dessous, 0.5, true));
    let mut c = Front1d::new(dessous, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(c.avancer(20_000), Ok(()));
    assert!(c.phi().iter().all(|x| x.abs() <= 1.0 + 1e-9), "f64 : phi est sorti de [-1, 1]");
    let mut d = Front1dFixe::new(dessous, &phi, &u, ModeU::Joue).unwrap();
    assert_eq!(d.avancer(20_000), Ok(()));
    assert!(d.phi().iter().all(|x| x.vers_f64().abs() <= 1.0 + 1e-9), "Fixe : phi est sorti de [-1, 1]");
}

/// régression : (V-P1-STAB-02) la zone sûre de la doc (une MESURE de notre schéma, pas une valeur exacte) : à `dt = dt_sur / 2` (`dt_sur` : les bornes linéaires avec le couplage, recalculé
/// dans `etape_p1_commun`), 80 réglages tirés (`W` de 0,3 à 3, `tau` de 0,2 à 5, `lambda` de 0,1 à 40, `D` de 0,1 à 5, `dx` de 0,1 à
/// 2, `U = max |u|` de 0 à 1,5 : `lambda U` va jusqu'à 60), un front de Stefan de 80 cases, `u` gelé ou joué, 4000 pas : aucun ne
/// déborde et `phi` reste dans `[-1, 1]` à `1e-9` près ; un réglage sur quatre est rejoué en `Fixe` (aucun débordement). C'est la
/// mesure qui porte la phrase de la doc, pas une démonstration. (`u`, lui, peut dépasser 0 derrière le front : rien n'est affirmé.)
///
/// Ne couvre pas : `dt` entre `dt_sur / 2` et `dt_max` (des réglages y explosent : voir le test précédent et la doc).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn zone_sure_mesuree() {
    let mut g = Lcg(7_654_321);
    let mut plus_grand_lambda_u = 0.0_f64;
    for essai in 0..80_u64 {
        let (w, tau, lambda, d, dx) =
            (g.entre(0.3, 3.0), g.entre(0.2, 5.0), g.entre(0.1, 40.0), g.entre(0.1, 5.0), g.entre(0.1, 2.0));
        let u_max = g.entre(0.0, 1.5);
        let joue = essai % 2 == 0;
        let mut p = param(w, tau, lambda, d, dx, 0.0);
        p.dt = 0.5 * dt_sur(&p, u_max, joue);
        assert_eq!(p.verifier(), Ok(()), "essai {essai}");
        plus_grand_lambda_u = plus_grand_lambda_u.max(lambda * u_max);
        let (phi, u) = front_de_stefan(80, dx, 24.0 * dx, u_max, essai);
        let mode = if joue { ModeU::Joue } else { ModeU::Gele };
        let mut f = Front1d::new(p, &phi, &u, mode).unwrap();
        assert_eq!(f.avancer(4000), Ok(()), "essai {essai} : {p:?} U = {u_max}");
        assert_eq!((f.pas_faits(), f.phi().len()), (4000, 80), "essai {essai}");
        assert!(
            f.phi().iter().zip(&phi).any(|(a, b)| a.to_bits() != b.to_bits()),
            "essai {essai} : l'état n'a pas bougé"
        );
        assert!(f.phi().iter().all(|x| x.abs() <= 1.0 + 1e-9), "essai {essai} : phi est sorti de [-1, 1]");
        if essai % 4 == 0 {
            let mut fx = Front1dFixe::new(p, &phi, &u, mode).unwrap();
            assert_eq!(fx.avancer(4000), Ok(()), "essai {essai} (Fixe)");
            assert_eq!(fx.pas_faits(), 4000, "essai {essai} (Fixe)");
        }
    }
    assert!(plus_grand_lambda_u > 20.0, "le tirage devait atteindre de grands lambda U : {plus_grand_lambda_u}");
}

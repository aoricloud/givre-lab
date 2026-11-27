//! Étape P3 : la vitesse de la pointe entre deux mesures (`givre_lab::pointe::vitesse_de_pointe`) et la constante de sélection
//! `σ* = 2 d0 D / (ρ² V)` (`givre_lab::pointe::constante_de_selection`), à la main. Valeurs attendues : `references/etape-p3-valeurs.md`,
//! identifiants `V-P3-VIT-xx` et `V-P3-SIGMA-xx`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::pointe::{ErreurPointe as E, Pointe, constante_de_selection, vitesse_de_pointe};

fn pointe(temps: f64, position: f64, sommet: f64) -> Pointe {
    Pointe { temps, position, lignes: 5, sommet, rayon: 3.0 }
}

/// juste : (V-P3-VIT-01) la vitesse de la doc à la main : `(apres.position - avant.position) / (apres.temps - avant.temps)`, sur la
/// POSITION (la ligne 0) et non sur le sommet ajusté : `(1 ; 2)` puis `(3 ; 5)` donnent 1,5 (les sommets 7 et 7 donneraient 0) ;
/// `(10 ; 31,5)` puis `(14 ; 30,5)` donnent `-0,25` (une pointe qui recule est rendue telle quelle) ; `(0 ; 0)` puis `(0,5 ; 0,125)` :
/// `0,25`. Les autres champs de `Pointe` (lignes, rayon) ne comptent pas. L'ordre des opérations : sur 200 paires tirées, les bits sont
/// ceux de `vitesse_naive`.
///
/// Ne couvre pas : des mesures nées d'une dendrite (voir `mesure_egale_au_naif_bit_pour_bit`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn vitesse_a_la_main() {
    assert_eq!(vitesse_de_pointe(&pointe(1.0, 2.0, 7.0), &pointe(3.0, 5.0, 7.0)), Ok(1.5));
    assert_eq!(vitesse_de_pointe(&pointe(10.0, 31.5, 1.0), &pointe(14.0, 30.5, 9.0)), Ok(-0.25));
    assert_eq!(vitesse_de_pointe(&pointe(0.0, 0.0, 0.0), &pointe(0.5, 0.125, 0.0)), Ok(0.25));
    let mut g = Lcg(99);
    for _ in 0..200 {
        let a = Pointe { temps: g.entre(0.0, 100.0), position: g.entre(0.0, 80.0), lignes: 7, sommet: 1.0, rayon: 2.0 };
        let b = Pointe { temps: a.temps + g.entre(0.01, 50.0), position: g.entre(0.0, 80.0), ..a };
        assert_eq!(vitesse_de_pointe(&a, &b).map(f64::to_bits), vitesse_naive(&a, &b, None).map(f64::to_bits));
    }
}

/// juste : (V-P3-SIGMA-01) la constante de sélection de la doc à la main : `((2 d0) D) / ((ρ ρ) V)`. `d0 = 0,5`, `D = 1`, `ρ = 4`,
/// `V = 0,125` : exactement `0,5` (avec `ρ` au lieu de `ρ²` : 2) ; `d0 = 0,25`, `D = 2`, `ρ = 0,5`, `V = 4` : exactement `1` ;
/// `d0 = 0,5`, `D = 1`, `ρ = 5,77`, `V = 0,0816` : `0,368 108…` à `1e-12` près (`1 / (33,2929 · 0,0816)`) ; et, sur 200 quadruplets tirés,
/// les bits de `selection_naive`.
///
/// Ne couvre pas : la valeur de `σ*` d'une dendrite (voir `etape_p3_sens.rs`, qui la mesure et ne la compare à aucune valeur publiée).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn selection_a_la_main() {
    assert_eq!(constante_de_selection(0.5, 1.0, 4.0, 0.125), Ok(0.5));
    assert_eq!(constante_de_selection(0.25, 2.0, 0.5, 4.0), Ok(1.0));
    let s = constante_de_selection(0.5, 1.0, 5.77, 0.0816).unwrap();
    assert!((s - 1.0 / (33.2929 * 0.0816)).abs() < 1e-12, "{s}");
    let mut g = Lcg(7);
    for _ in 0..200 {
        let (d0, d, r, v) = (g.entre(0.01, 3.0), g.entre(0.1, 5.0), g.entre(0.5, 60.0), g.entre(0.001, 0.5));
        assert_eq!(
            constante_de_selection(d0, d, r, v).map(f64::to_bits),
            selection_naive(d0, d, r, v, None).map(f64::to_bits)
        );
    }
}

/// juste : (V-P3-VIT-02, V-P3-SIGMA-02) les refus, dans l'ordre de la doc. `vitesse_de_pointe` : (1) un temps non fini (NaN, infini, de
/// l'une ou l'autre mesure) ou `apres.temps <= avant.temps` (égaux, ou dans le mauvais ordre) : `ParametreInvalide("temps")`, même si
/// une position est aussi fausse ; (2) une position non finie : `ParametreInvalide("position")` ; (3) un quotient non fini (positions
/// `-1e308` et `1e308` à `1e-300` de temps d'écart) : `HorsDePortee`. `constante_de_selection` : `ParametreInvalide(nom)` pour le premier
/// de `d0`, `d`, `rayon`, `vitesse` qui est nul, négatif, NaN ou infini (deux faux à la fois : le premier de la liste) ; `HorsDePortee`
/// si le résultat déborde (`d0 = d = 1e300`) ou tombe à zéro (`d0 = d = 1e-300`, `ρ = 1e10`). Aucun ne panique.
///
/// Ne couvre pas : les refus de la mesure (voir `etape_p3_rayon.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn vitesse_et_selection_refus_et_ordre() {
    let ok = pointe(1.0, 2.0, 0.0);
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(vitesse_de_pointe(&pointe(t, 2.0, 0.0), &ok), Err(E::ParametreInvalide("temps")));
        assert_eq!(vitesse_de_pointe(&ok, &pointe(t, 2.0, 0.0)), Err(E::ParametreInvalide("temps")));
        assert_eq!(vitesse_de_pointe(&ok, &pointe(t, f64::NAN, 0.0)), Err(E::ParametreInvalide("temps")));
    }
    assert_eq!(vitesse_de_pointe(&ok, &pointe(1.0, 3.0, 0.0)), Err(E::ParametreInvalide("temps")));
    assert_eq!(vitesse_de_pointe(&ok, &pointe(0.5, 3.0, 0.0)), Err(E::ParametreInvalide("temps")));
    assert_eq!(vitesse_de_pointe(&ok, &pointe(0.5, f64::NAN, 0.0)), Err(E::ParametreInvalide("temps")));
    for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(vitesse_de_pointe(&ok, &pointe(2.0, x, 0.0)), Err(E::ParametreInvalide("position")));
        assert_eq!(vitesse_de_pointe(&pointe(0.0, x, 0.0), &ok), Err(E::ParametreInvalide("position")));
    }
    assert_eq!(vitesse_de_pointe(&pointe(0.0, -1e308, 0.0), &pointe(1e-300, 1e308, 0.0)), Err(E::HorsDePortee));
    let noms = ["d0", "d", "rayon", "vitesse"];
    for (k, nom) in noms.iter().enumerate() {
        for mauvais in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut v = [0.5, 1.0, 4.0, 0.125];
            v[k] = mauvais;
            assert_eq!(
                constante_de_selection(v[0], v[1], v[2], v[3]),
                Err(E::ParametreInvalide(nom)),
                "{nom} = {mauvais}"
            );
            if k + 1 < noms.len() {
                v[k + 1] = -1.0;
                assert_eq!(
                    constante_de_selection(v[0], v[1], v[2], v[3]),
                    Err(E::ParametreInvalide(nom)),
                    "{nom} d'abord"
                );
            }
        }
    }
    assert_eq!(constante_de_selection(1e300, 1e300, 1.0, 1.0), Err(E::HorsDePortee));
    assert_eq!(constante_de_selection(1e-300, 1e-300, 1e10, 1.0), Err(E::HorsDePortee));
}

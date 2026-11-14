//! Étape P1 : des fautes plausibles, qui DOIVENT échouer. Chaque faute est une variante du naïf de `etape_p1_commun` (le signe de `lambda`, le
//! `1/2` oublié, `(1 - φ²)` au lieu de `(1 - φ²)²`, une mise à jour en place, un bord qui fuit, `u` joué dans le cas du front poussé, `W` au lieu de
//! `W²`, `dt` au lieu de `dt/tau`, la somme sans paires, le `dphi` du pas d'avant). Valeurs attendues : `references/etape-p1-valeurs.md`,
//! identifiants `V-P1-FAUTE-xx`.

mod etape_p1_commun;

use etape_p1_commun::*;
use givre_lab::phase::ModeU;

/// Les cas (parmi les trois) où la faute fait dépasser la tolérance de `etape_p1_exact.rs` : mesuré sur le prototype (voir la table de fautes du
/// rapport). `[cas 1, cas 2, cas 3]`.
fn vues_par_les_cas(f: Faute) -> [bool; 3] {
    match f {
        Faute::SigneLambda => [false, true, true],
        Faute::DemiOublie => [false, false, true],
        Faute::UnMoinsPhi2Simple => [false, true, true],
        Faute::EnPlace => [true, true, true],
        Faute::BordFuit => [true, false, true],
        Faute::UJoue => [false, true, false],
        Faute::WSansCarre => [true, true, true],
        Faute::TauOublie => [false, true, true],
        Faute::SansPaires => [false, false, false],
        Faute::DphiEnRetard => [false, false, false],
    }
}

/// Vrai si la mesure sort de la tolérance (ou n'existe pas : NaN : le front a disparu).
fn hors(mesure: f64, tol: f64) -> bool {
    mesure.is_nan() || mesure.abs() > tol
}

/// Les trois mesures d'un moteur : `[cas 1 (dx = 0,5), cas 2 (dx = 0,25, lambda = 2, lambda Delta = 0,1), cas 3 (Delta = 0,5, W = 1, dx = 0,5)]` ou, pour
/// `W` : le cas 3 avec `W = 0,5`, `dx = 0,25` en plus (une faute sur `W` n'y est visible qu'à `W != 1`).
fn hors_tolerance(m: Moteur) -> [bool; 3] {
    let tol1 = TOL_CAS1.iter().find(|t| t.0 == 0.5).unwrap().1;
    let tol2 = TOL_CAS2.iter().find(|t| t.0 == 0.25 && t.1 == 2.0).unwrap().3;
    let (delta, k) = K_NEUMANN[1];
    let tol3 = TOL_CAS3.iter().find(|t| t.0 == 1 && t.1 == 1.0 && t.2 == 0.5).unwrap().3;
    let tol3w = TOL_CAS3.iter().find(|t| t.0 == 1 && t.1 == 0.5).unwrap().3;
    [
        hors(cas1(m, 0.5).0, tol1),
        hors(cas2(m, 0.25, 2.0, 0.1), tol2),
        hors(cas3(m, delta, k, 1.0, 0.5), tol3) || hors(cas3(m, delta, k, 0.5, 0.25), tol3w),
    ]
}

/// juste : (V-P1-FAUTE-01) le naïf SANS faute passe les trois cas (le point de départ : sinon les fautes ci-dessous ne prouveraient rien) ; chacune
/// des dix fautes, appliquée au naïf, sort de la tolérance dans les cas où elle doit se voir (`vues_par_les_cas`, mesuré : par exemple le signe de `lambda`
/// fait disparaître le front, le `1/2` oublié ne se voit qu'au cas 3, `u` joué ne se voit qu'au cas 2, `W` au lieu de `W²` partout). Deux fautes ne se voient
/// par AUCUN des trois cas : la somme sans paires (ce n'est qu'un autre arrondi : c'est la symétrie qui la voit, `etape_p1_symetrie.rs`) et le `dphi` du
/// pas d'avant (une erreur d'ordre 1 en `dt`, sous la tolérance : c'est l'égalité bit à bit au naïf qui la voit, ci-dessous).
///
/// Ne couvre pas : les fautes que personne n'a imaginées ; des fautes combinées.
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn les_fautes_sortent_de_la_tolerance() {
    assert_eq!(
        hors_tolerance(Moteur::Naif64(None)),
        [false, false, false],
        "le naïf sans faute devait passer les trois cas"
    );
    for f in TOUTES_LES_FAUTES {
        let vue = hors_tolerance(Moteur::Naif64(Some(f)));
        let attendue = vues_par_les_cas(f);
        for cas in 0..3 {
            if attendue[cas] {
                assert!(vue[cas], "la faute {f:?} devait sortir de la tolérance au cas {}", cas + 1);
            }
        }
    }
}

/// juste : (V-P1-FAUTE-02) chacune des dix fautes donne des bits DIFFÉRENTS de ceux de `Front1d` (le naïf sans faute étant, lui, identique : vérifié
/// ici aussi, pour qu'une bibliothèque fausse ne passe pas ce test à vide) sur un petit cas (16 cases, `W = 1,3`, `tau = 1,7`, `lambda = 1,1`, `D = 0,9`, `dx = 0,5`, `dt = 0,6 dt_max`, un départ tiré, 25 pas ;
/// `u` gelé pour la faute `UJoue`, joué pour les autres). C'est ce qui fait échouer un code qui aurait cette faute contre le naïf.
///
/// Ne couvre pas : la tolérance (voir le test précédent).
#[test]
#[cfg_attr(not(feature = "etape-p1"), ignore)]
fn les_fautes_changent_les_bits() {
    let mut g = Lcg(77);
    let mut p = param(1.3, 1.7, 1.1, 0.9, 0.5, 0.0);
    p.dt = pas_de_temps(&p, 0.6);
    let phi: Vec<f64> = (0..16).map(|_| g.entre(-1.0, 1.0)).collect();
    let u: Vec<f64> = (0..16).map(|_| g.entre(-0.6, 0.0)).collect();
    for f in TOUTES_LES_FAUTES {
        let mode = if f == Faute::UJoue { ModeU::Gele } else { ModeU::Joue };
        let bon = simuler(Moteur::Lib64, &p, &phi, &u, mode, 25);
        let sans_faute = simuler(Moteur::Naif64(None), &p, &phi, &u, mode, 25);
        let faux = simuler(Moteur::Naif64(Some(f)), &p, &phi, &u, mode, 25);
        let bits = |e: &Etat| -> Vec<u64> { e.phi.iter().chain(e.u.iter()).map(|x| x.to_bits()).collect() };
        assert_eq!(bits(&bon), bits(&sans_faute), "la bibliothèque devait donner les bits du naïf sans faute");
        assert_ne!(bits(&bon), bits(&faux), "la faute {f:?} n'a pas changé les bits");
    }
}

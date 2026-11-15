//! Étape P2 : la règle des nombres d'`AGENTS.md` (des `f64` permis sous une symétrie exacte si chaque somme que la symétrie échange
//! s'écrit par paires, et si un test montre la symétrie au bit près) pour la dendrite 2D : les huit symétries du carré sur le domaine
//! entier, et le quart de domaine qui est, au bit près, le quart du domaine entier. Valeurs attendues :
//! `references/etape-p2-valeurs.md`, identifiants `V-P2-SYM-xx` et `V-P2-QUART-xx`.

mod etape_p2_commun;

use etape_p2_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};

/// Les réglages des essais de symétrie : `W0 = 1,1`, `tau0 = 1,3`, `lambda = 1,2`, `D = 0,8`, `dx = 0,6`, `epsilon` donné, `dt` la
/// borne de la doc pour `U = 1` (les `u` tirés sont dans `]-1, 0]`).
fn reglages(epsilon: f64, mode: ModeU) -> Reglages {
    let mut r = Reglages { w0: 1.1, tau0: 1.3, lambda: 1.2, d: 0.8, epsilon, dx: 0.6, dt: 1.0 };
    r.dt = pas_de_temps(&r, 1.0, mode);
    r
}

/// juste : (V-P2-SYM-01) les huit symétries du carré sont exactes au bit près : depuis un état tiré à huit symétries (des flottants à
/// mantisse pleine : `φ` dans `[-1, 1[`, `u` dans `]-1, 0]`), domaine entier de côté pair ET impair (6, 7, 12, 17), 3 graines,
/// anisotropie `epsilon` = 0,05 (et `epsilon` = 0, et 0,0665 : tout près de la borne `1/15`), `u` joué ou gelé, `φ` et `u` ne cessent d'être
/// symétriques (les sept images non triviales, au bit près) à 1, 2, 3, 5, 10, 50, 200, 1000 et 2000 pas. Chaque somme et chaque produit
/// que l'une des huit symétries échange est écrit par paires dans la doc, les différences antisymétriques ont une image exacte, `H` ne dépend
/// que de carrés : c'est la règle de `AGENTS.md`. (Le champ de `u` est joué avec la chaleur latente : le couplage `φ`-`u` est compris.)
///
/// Ne couvre pas : la graine ronde (voir le test suivant) ; l'état sans symétrie ; un autre `epsilon` ; les réglages au-delà de ceux-ci.
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn huit_symetries_exactes_depuis_un_etat_tire() {
    let points = [1_u64, 2, 3, 5, 10, 50, 200, 1000, 2000];
    for (epsilon, mode) in [(0.05, ModeU::Joue), (0.05, ModeU::Gele), (0.0, ModeU::Joue), (0.0665, ModeU::Joue)] {
        let r = reglages(epsilon, mode);
        for n in [6_usize, 7, 12, 17] {
            for sel in 1..=3_u64 {
                let (phi, u) = depart_symetrique(n, sel);
                assert_eq!(defauts_2d(n, &phi), vec![0; 7], "le départ n'est pas symétrique (n = {n})");
                let mut d = Dendrite::new(r, n, Domaine::Entier, &phi, &u, mode).unwrap();
                let mut fait = 0;
                for pas in points {
                    d.avancer(pas - fait).unwrap();
                    fait = pas;
                    assert_eq!(
                        defauts_2d(n, d.phi()),
                        vec![0; 7],
                        "phi : epsilon {epsilon} {mode:?} n = {n}, graine {sel}, pas {pas}"
                    );
                    assert_eq!(
                        defauts_2d(n, d.u()),
                        vec![0; 7],
                        "u : epsilon {epsilon} {mode:?} n = {n}, graine {sel}, pas {pas}"
                    );
                }
                // l'état a bougé (le test ne serait pas à vide) et le décompte est juste
                assert_ne!(bits(d.phi()), bits(&phi));
                assert_eq!(d.pas_faits(), 2000);
            }
        }
    }
}

/// juste : (V-P2-SYM-02) les huit symétries sont exactes sur la graine ronde au centre du domaine entier (`m` = 10, 13 et 24 cases de
/// côté, `epsilon = 0,05`, `u` joué, `Delta = 0,65`, rayon 3 ; 800 pas) : les trois états de départ et les trois états finaux ont 0
/// case asymétrique pour chacune des sept images, et la graine a grandi (`φ` a changé, le nombre de cases à `φ > 0` a augmenté).
///
/// Ne couvre pas : une graine non ronde (voir le test précédent) ; la croissance réelle d'une dendrite (voir `etape_p2_pointe.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn huit_symetries_exactes_sur_la_graine_ronde() {
    let positifs = |v: &[f64]| v.iter().filter(|x| **x > 0.0).count();
    for m in [10_usize, 13, 24] {
        let mut r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
        r.dt = pas_de_temps(&r, 0.65, ModeU::Joue);
        let mut d = Dendrite::graine_ronde(r, m, Domaine::Entier, 0.65, 3.0, ModeU::Joue).unwrap();
        let (phi0, u0) = (d.phi().to_vec(), d.u().to_vec());
        assert_eq!(defauts_2d(m, &phi0), vec![0; 7], "départ, m = {m}");
        assert_eq!(defauts_2d(m, &u0), vec![0; 7], "départ (u), m = {m}");
        d.avancer(800).unwrap();
        assert_eq!(defauts_2d(m, d.phi()), vec![0; 7], "phi, m = {m}");
        assert_eq!(defauts_2d(m, d.u()), vec![0; 7], "u, m = {m}");
        assert_ne!(bits(d.phi()), bits(&phi0));
        assert!(positifs(d.phi()) > positifs(&phi0), "la graine devait grandir (m = {m})");
    }
}

/// juste : (V-P2-SYM-03) la règle est NÉCESSAIRE : le naïf écrit SANS paires (la somme des quatre voisines de `u` en chaîne, ou le
/// gradient transverse en chaîne), avec une constante calculée sous deux écritures (le flux en `y`), ou en place, perd les huit symétries
/// (`n` = 8 et 9, 4 graines : au moins une case asymétrique au bout de 1, 2, 3, 10 ou 100 pas, pour chacun des huit couples), alors que le
/// même naïf sans faute n'en a aucune aux mêmes points (et ses bits sont ceux de la bibliothèque : voir `etape_p2_naif.rs`). Sans ce test,
/// la symétrie exacte pourrait n'être qu'une propriété de tout schéma.
///
/// Ne couvre pas : la perte à long terme ; les autres fautes (voir `etape_p2_fautes.rs`, qui affirme toute la table).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn sans_paires_perd_les_symetries() {
    assert_eq!(symetrie_perdue(None), vec![false; 8], "le naïf sans faute devait rester symétrique");
    for faute in [Faute::SansPairesU, Faute::SansPairesGt, Faute::ConstanteDeuxFois, Faute::EnPlace] {
        assert_eq!(
            symetrie_perdue(Some(faute)),
            vec![true; 8],
            "la faute {faute:?} devait perdre une symétrie, pour chaque couple"
        );
    }
}

/// Le quart (cases `(n + i, n + j)`) d'un état de côté `2 n`.
fn quart_de(entier: &[f64], n: usize) -> Vec<f64> {
    let mut q = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            q.push(entier[(n + i) * (2 * n) + n + j]);
        }
    }
    q
}

/// juste : (V-P2-QUART-01) le quart de domaine calculé SEUL est, au bit près, le quart supérieur droit du domaine entier de côté `2 n`
/// (cases `(n + i, n + j)`), pour `φ` et pour `u` : (a) sur la graine ronde (`n` = 10, 13 et 24, `dx = 0,8`, rayon 3 ; 1, 7, 100 et 800
/// pas), (b) sur un départ tiré à huit symétries (`n` = 6, 9 et 12, 2 graines, `epsilon = 0,05`, `u` joué ; 1, 7, 100 et 500 pas). Le
/// domaine entier de départ est celui de la doc (`Domaine::Entier` de côté `2 n`, centre entre quatre cases) ; le quart a pour voisines
/// hors grille, sur ses deux bords de symétrie, les cases du bord par miroir, qui sont exactement les images que l'entier calcule.
///
/// Ne couvre pas : un départ sans symétrie (le quart n'a alors aucun sens) ; les grands domaines (voir `etape_p2_pointe.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p2"), ignore)]
fn le_quart_est_le_quart_du_domaine_entier() {
    // (a) la graine ronde
    let mut r = Reglages::depuis_capillarite(1.0, 0.5, 1.0, 0.05, 0.8, 1.0);
    r.dt = pas_de_temps(&r, 0.65, ModeU::Joue);
    for n in [10_usize, 13, 24] {
        let mut quart = Dendrite::graine_ronde(r, n, Domaine::Quart, 0.65, 3.0, ModeU::Joue).unwrap();
        let mut entier = Dendrite::graine_ronde(r, 2 * n, Domaine::Entier, 0.65, 3.0, ModeU::Joue).unwrap();
        assert_eq!(bits(quart.phi()), bits(&quart_de(entier.phi(), n)), "départ (phi), n = {n}");
        assert_eq!(bits(quart.u()), bits(&quart_de(entier.u(), n)), "départ (u), n = {n}");
        let mut fait = 0;
        for pas in [1_u64, 7, 100, 800] {
            quart.avancer(pas - fait).unwrap();
            entier.avancer(pas - fait).unwrap();
            fait = pas;
            assert_eq!(bits(quart.phi()), bits(&quart_de(entier.phi(), n)), "graine ronde : phi, n = {n}, pas {pas}");
            assert_eq!(bits(quart.u()), bits(&quart_de(entier.u(), n)), "graine ronde : u, n = {n}, pas {pas}");
        }
    }
    // (b) un départ tiré à huit symétries
    let r = reglages(0.05, ModeU::Joue);
    for n in [6_usize, 9, 12] {
        for sel in 1..=2_u64 {
            let (phi, u) = depart_symetrique(2 * n, sel);
            let mut entier = Dendrite::new(r, 2 * n, Domaine::Entier, &phi, &u, ModeU::Joue).unwrap();
            let mut quart =
                Dendrite::new(r, n, Domaine::Quart, &quart_de(&phi, n), &quart_de(&u, n), ModeU::Joue).unwrap();
            let mut fait = 0;
            for pas in [1_u64, 7, 100, 500] {
                quart.avancer(pas - fait).unwrap();
                entier.avancer(pas - fait).unwrap();
                fait = pas;
                assert_eq!(
                    bits(quart.phi()),
                    bits(&quart_de(entier.phi(), n)),
                    "tiré : phi, n = {n}, graine {sel}, pas {pas}"
                );
                assert_eq!(
                    bits(quart.u()),
                    bits(&quart_de(entier.u(), n)),
                    "tiré : u, n = {n}, graine {sel}, pas {pas}"
                );
            }
        }
    }
}

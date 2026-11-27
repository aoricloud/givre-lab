//! Étape P3 : sur des états réels de la dendrite de P2 (plusieurs `ε`, `dx`, temps, rayons de graine), le module `pointe` donne
//! EXACTEMENT les bits de la mesure naïve écrite dans les tests d'après sa doc (`etape_p3_commun` : le contour, la règle des lignes, la
//! parabole, la vitesse, `σ*`). Valeurs attendues : `references/etape-p3-valeurs.md`, identifiant `V-P3-NAIF-01`.

mod etape_p3_commun;

use etape_p3_commun::*;
use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::pointe::{ErreurPointe, constante_de_selection, contour, mesurer_pointe, vitesse_de_pointe};

/// Les profondeurs essayées à chaque relevé : de très peu (une ou deux lignes : `TropPeuDeLignes`) à démesurée (toutes les lignes
/// consécutives qui croisent).
const PROFONDEURS: [f64; 6] = [0.05, 0.5, 1.0, 2.0, 3.7, 1e6];

/// juste : (V-P3-NAIF-01) pour 16 graines rondes tirées (`W0` de 0,7 à 1,4, `d0` de 0,3 à 1, `D` de 0,5 à 2, `epsilon` de 0 à 0,0665
/// (0 et 0,0665 compris), `dx` de 0,4 à 1, `Delta` de 0,4 à 0,8, rayon de 3 à 12, quart de 20 à 44 cases, `u` joué, `dt` de 0,5 à 1 fois
/// la borne de P2) et six relevés chacune (après 1, 50, 150, 300, 600 et 1000 pas), le contour est, bit pour bit, celui de
/// `contour_naif` (et sa ligne 0 est `position_pointe`) ; pour chacune des six profondeurs de `PROFONDEURS`, `mesurer_pointe` rend
/// exactement le `Result` de `mesure_naive` (les bits de `temps`, `position`, `sommet`, `rayon`, le nombre de lignes, ou la même
/// erreur) ; entre deux relevés mesurés à la même profondeur, `vitesse_de_pointe` a les bits de `vitesse_naive` ; et, quand la vitesse est
/// positive, `constante_de_selection(d0, D, rayon, vitesse)` a ceux de `selection_naive`. Sur l'ensemble : au moins 300 mesures
/// réussies, au moins 50 `TropPeuDeLignes`, au moins 200 vitesses et 150 `σ*` comparées. Un domaine entier rend `DomaineEntier` des deux
/// côtés.
///
/// Ne couvre pas : la justesse de la dendrite elle-même (les tests de P2) ; le sens de variation (voir `etape_p3_sens.rs`).
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn mesure_egale_au_naif_bit_pour_bit() {
    let (mut reussies, mut trop_peu, mut vitesses, mut sigmas) = (0, 0, 0, 0);
    for graine in 1..=16_u64 {
        let mut g = Lcg(9000 + graine);
        let (w0, d0, d, dx, delta, rayon) = (
            g.entre(0.7, 1.4),
            g.entre(0.3, 1.0),
            g.entre(0.5, 2.0),
            g.entre(0.4, 1.0),
            g.entre(0.4, 0.8),
            g.entre(3.0, 12.0),
        );
        let epsilon = match graine % 8 {
            0 => 0.0,
            1 => 0.0665,
            _ => g.entre(0.0, 0.0665),
        };
        let n = 20 + (g.suivant() * 25.0) as usize;
        let fraction = g.entre(0.5, 1.0);
        let r0 = Reglages::depuis_capillarite(w0, d0, d, epsilon, dx, 1.0);
        let r = Reglages { dt: fraction * r0.dt_borne(delta, ModeU::Joue), ..r0 };
        let mut dendrite = Dendrite::graine_ronde(r, n, Domaine::Quart, delta, rayon, ModeU::Joue).unwrap();
        let mut avant: Vec<Option<givre_lab::pointe::Pointe>> = vec![None; PROFONDEURS.len()];
        let mut fait = 0;
        for pas in [1_u64, 50, 150, 300, 600, 1000] {
            dendrite.avancer(pas - fait).unwrap();
            fait = pas;
            let c = contour(&dendrite).unwrap();
            let naif = contour_naif(dendrite.phi(), n, dx, None);
            assert_eq!(
                c.iter().map(|x| x.map(f64::to_bits)).collect::<Vec<_>>(),
                naif.iter().map(|x| x.map(f64::to_bits)).collect::<Vec<_>>(),
                "graine {graine} pas {pas} : le contour"
            );
            assert_eq!(c[0].map(f64::to_bits), dendrite.position_pointe().map(f64::to_bits));
            for (k, &h) in PROFONDEURS.iter().enumerate() {
                let lib = mesurer_pointe(&dendrite, h);
                let attendu = mesure_naive_de(&dendrite, h, None);
                assert_eq!(bits_mesure(&lib), bits_mesure(&attendu), "graine {graine} pas {pas} profondeur {h}");
                match lib {
                    Ok(p) => {
                        reussies += 1;
                        if let Some(a) = avant[k] {
                            let v = vitesse_de_pointe(&a, &p);
                            assert_eq!(v.clone().map(f64::to_bits), vitesse_naive(&a, &p, None).map(f64::to_bits));
                            vitesses += 1;
                            if let Ok(v) = v
                                && v > 0.0
                            {
                                assert_eq!(
                                    constante_de_selection(d0, d, p.rayon, v).map(f64::to_bits),
                                    selection_naive(d0, d, p.rayon, v, None).map(f64::to_bits),
                                    "graine {graine} pas {pas} profondeur {h} : sigma"
                                );
                                sigmas += 1;
                            }
                        }
                        avant[k] = Some(p);
                    }
                    Err(ErreurPointe::TropPeuDeLignes(_)) => {
                        trop_peu += 1;
                        avant[k] = None;
                    }
                    Err(_) => avant[k] = None,
                }
            }
        }
        // le même état dans un domaine entier
        let entier = Dendrite::new(r, n, Domaine::Entier, dendrite.phi(), dendrite.u(), ModeU::Joue).unwrap();
        assert_eq!(mesurer_pointe(&entier, 2.0), Err(ErreurPointe::DomaineEntier));
        assert_eq!(mesure_naive_de(&entier, 2.0, None), Err(ErreurPointe::DomaineEntier));
    }
    assert!(
        reussies >= 300 && trop_peu >= 50 && vitesses >= 200 && sigmas >= 150,
        "{reussies} mesures, {trop_peu} TropPeuDeLignes, {vitesses} vitesses, {sigmas} sigma"
    );
}

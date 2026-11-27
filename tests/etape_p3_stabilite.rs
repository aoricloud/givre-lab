//! Étape P3 : la dendrite pleine jusqu'au bout de la plage de `ε`, au pas de temps le plus grand qu'accepte la borne de P2, et la
//! mesure de la pointe qui la suit. Valeurs attendues : `references/etape-p3-valeurs.md`, identifiant `V-P3-STAB-01`.

use givre_lab::dendrite::{Dendrite, Domaine, ModeU, Reglages};
use givre_lab::pointe::mesurer_pointe;

/// régression : (V-P3-STAB-01) la graine ronde de Kim et coll. (`W0 = 1`, `d0 = 0,5`, `D = 1`, `Delta = 0,65`, rayon 15, `u` joué),
/// quart de 60 × 60 cases au pas 0,8, `dt` = EXACTEMENT `dt_borne(Delta, Joue)` (le plus grand pas accepté), pour `ε` = 0 ; 0,06 et
/// 0,0665 (le bord de la plage permise) : 3000 pas sans erreur, et, relevé tous les 500 pas : `|φ|` ne dépasse pas `1 + 1e-12`
/// (observé : 1 exactement) ; la pointe se mesure (`mesurer_pointe` à la profondeur 2 rend `Ok`) et sa position avance d'un relevé
/// au suivant. Hors des tests (`MODELE.md`, « La stabilité selon ε ») : aucune divergence à 1 ; 1,1 ; 1,25 et 1,5 fois la borne sur la
/// dendrite pleine (96 calculs de 10 000 pas), et la réserve de la borne ne fond pas quand `ε` monte sur les états tirés.
///
/// Ne couvre pas : un pas au-delà de la borne (refusé par P2) ; un temps long ; une grille plus fine.
#[test]
#[cfg_attr(not(feature = "etape-p3"), ignore)]
fn la_dendrite_pleine_tient_a_la_borne_jusqu_a_epsilon_maximal() {
    for epsilon in [0.0, 0.06, 0.0665] {
        let r0 = Reglages::depuis_capillarite(1.0, 0.5, 1.0, epsilon, 0.8, 1.0);
        let r = Reglages { dt: r0.dt_borne(0.65, ModeU::Joue), ..r0 };
        let mut d = Dendrite::graine_ronde(r, 60, Domaine::Quart, 0.65, 15.0, ModeU::Joue).unwrap();
        let mut position = mesurer_pointe(&d, 2.0).unwrap().position;
        for releve in 1..=6 {
            d.avancer(500).unwrap_or_else(|e| panic!("epsilon {epsilon}, relevé {releve} : {e:?}"));
            let m = d.phi().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            assert!(m <= 1.0 + 1e-12, "epsilon {epsilon}, relevé {releve} : |phi| = {m}");
            let p = mesurer_pointe(&d, 2.0).unwrap_or_else(|e| panic!("epsilon {epsilon}, relevé {releve} : {e:?}"));
            assert!(p.position > position, "epsilon {epsilon}, relevé {releve} : la pointe n'avance pas");
            position = p.position;
        }
        assert_eq!(d.pas_faits(), 3000);
    }
}

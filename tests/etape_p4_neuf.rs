//! Étape P4 : le métal **avant tout démarrage**, dans un processus neuf (ce fichier n'a qu'un test : l'état global de la vitrine n'y a jamais été posé). Valeurs attendues : `references/etape-p4-valeurs.md`, identifiant `V-P4-12`.

use givre_lab::vitrine::*;

/// juste : avant tout démarrage (V-P4-12), dans un processus neuf, `givre_metal_reglage`, `givre_metal_mesure` et `givre_metal_case` rendent `NaN` pour tout champ et toute case essayés (`u32::MAX` compris) : il n'y a pas de métal ;
/// des démarrages refusés (`ε` = 60 001, côtés 127, 257, 0, `u32::MAX`) rendent leur code (1 ou 2) **et ne démarrent rien** : les lecteurs rendent encore `NaN`, le temps et la durée 0, l'empreinte 0, et `givre_avancer` ne joue rien.
///
/// Ne couvre pas : un démarrage refusé après un autre modèle (voir `etape_p4_vitrine.rs`) ; le module (voir `etape_p4_module.rs`, dont le script lit d'abord le module neuf).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn avant_tout_demarrage_il_n_y_a_pas_de_metal() {
    let vide = |contexte: &str| {
        for champ in [0, 1, 7, 8, 10, 11, 14, 15, u32::MAX] {
            assert!(givre_metal_reglage(champ).is_nan(), "{contexte} : givre_metal_reglage({champ})");
        }
        for champ in [0, 1, 2, 3, 4, 9, 10, 11, u32::MAX] {
            assert!(givre_metal_mesure(champ).is_nan(), "{contexte} : givre_metal_mesure({champ})");
        }
        for (champ, i) in [(0, 0), (1, 0), (0, 16_383), (1, 65_535), (2, 0), (u32::MAX, u32::MAX)] {
            assert!(givre_metal_case(champ, i).is_nan(), "{contexte} : givre_metal_case({champ}, {i})");
        }
        assert_eq!((givre_temps(), givre_duree(), givre_empreinte_mot(0)), (0, 0, 0), "{contexte} : rien de démarré");
    };
    vide("module neuf");
    for (e, c, code) in [(60_001, 128, 1), (0, 127, 2), (0, 257, 2), (50_000, 0, 2), (u32::MAX, u32::MAX, 1)] {
        assert_eq!(givre_metal_demarrer(e, c), code, "givre_metal_demarrer({e}, {c})");
    }
    vide("après des démarrages refusés");
    assert_eq!(givre_avancer(100), 0);
    vide("après un givre_avancer sans rien de démarré");
}

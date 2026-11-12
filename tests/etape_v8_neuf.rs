//! Étape V8 : les masses **avant tout démarrage**, en natif. Valeurs attendues : `references/etape-v8-valeurs.md`, identifiant `V-V8-10`.
//!
//! **Ce fichier ne contient qu'UN test, et il doit n'en contenir qu'un** : chaque fichier de `tests/` est un programme à part, donc un processus neuf où l'état global de la vitrine n'a jamais été posé ; avec un seul test, aucun autre test du processus
//! ne peut démarrer un modèle avant lui. C'est ce qui permet d'éprouver en natif ce que les deux fonctions des masses rendent « avant tout démarrage » (0 et `u32::MAX`, jamais une panique), que `etape_v8_vitrine.rs` ne peut pas voir (son état global est
//! déjà posé par un autre test) et que `etape_v8_module.rs` ne voit que sous Node, une fois le module reconstruit. N'ajoutez aucun test ici.
//!
//! Tant qu'une des deux fonctions `givre_neige_masse*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test s'arrête d'un coup (une panique ne traverse pas une `extern "C" fn`) : c'est un échec.

use givre_lab::vitrine::{
    givre_avancer, givre_demarrer, givre_neige_demarrer, givre_neige_masse, givre_neige_masses_preparer,
};

/// juste : avant tout démarrage (V-V8-10), dans un processus neuf (ce fichier n'a qu'un test), `givre_neige_masses_preparer()` rend 0 (pas de neige : pas de case) et `givre_neige_masse(i)` rend `u32::MAX` pour tout `i` essayé (0, 1, 15 150, 30 300,
/// 30 301, 65 535, `2^31`, `u32::MAX − 1`, `u32::MAX`), **avant et après** des préparations, des démarrages refusés (le jeu 19, `u32::MAX` partout, `ρ` hors plage, le modèle 8) et un `givre_avancer` qui ne joue rien ; puis `givre_neige_demarrer(0, …)` démarre la neige :
/// les masses sont **encore** absentes (aucune préparation), la préparation rend 30 301, la masse du germe (case 15 150) est strictement positive, une masse hors du tampon est `u32::MAX` ; puis un `givre_demarrer(0, 1)` (la DLA) efface l'instantané et la préparation rend 0.
/// Une lecture qui panique avant tout démarrage, ou qui rend 0 au lieu de `u32::MAX`, ou une préparation qui rend `u32::MAX` au lieu de 0, se voit.
///
/// Ne couvre pas : les masses d'une neige démarrée (voir `etape_v8_vitrine.rs`) ; le module WebAssembly neuf (voir `etape_v8_module.rs`, la ligne `avant=`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn avant_tout_demarrage_il_n_y_a_pas_de_masse() {
    let indices = [0_u32, 1, 15_150, 30_300, 30_301, 65_535, 1 << 31, u32::MAX - 1, u32::MAX];
    let rien = |contexte: &str| {
        assert_eq!(givre_neige_masses_preparer(), 0, "{contexte} : la préparation");
        for i in indices {
            assert_eq!(givre_neige_masse(i), u32::MAX, "{contexte} : la masse {i}");
        }
    };
    rien("processus neuf");
    rien("une seconde fois");
    assert_eq!(givre_neige_demarrer(19, 400_000, 1_300_000, 3_000, 70_000), 1);
    rien("après un jeu inconnu");
    assert_eq!(givre_neige_demarrer(u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX), 1);
    rien("après des valeurs absurdes");
    assert_eq!(givre_neige_demarrer(0, 1_000_001, 1_300_000, 3_000, 70_000), 2);
    rien("après un réglage hors plage");
    assert_eq!(givre_demarrer(8, 0), 0, "le modèle 8 n'existe pas");
    rien("après un givre_demarrer refusé");
    assert_eq!(givre_avancer(0), 0);
    assert_eq!(givre_avancer(5), 0, "rien à jouer");
    rien("après un givre_avancer qui ne joue rien");

    assert_eq!(givre_neige_demarrer(0, 400_000, 1_300_000, 3_000, 70_000), 0);
    for i in indices {
        assert_eq!(givre_neige_masse(i), u32::MAX, "neige démarrée mais jamais préparée : la masse {i}");
    }
    assert_eq!(givre_neige_masses_preparer(), 30_301, "la neige démarrée a 30 301 cases");
    assert!(givre_neige_masse(15_150) > 0, "le germe a une masse");
    assert_eq!(givre_neige_masse(30_301), u32::MAX);
    assert_eq!(givre_neige_masse(u32::MAX), u32::MAX);

    assert_eq!(givre_demarrer(0, 1), 1);
    rien("après la DLA qui remplace la neige");
}

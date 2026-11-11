//! Étape G3 : la neige **avant tout démarrage**, en natif. Valeurs attendues : `references/etape-g3-valeurs.md`, identifiant `V-G3-13`.
//!
//! **Ce fichier ne contient qu'UN test, et il doit n'en contenir qu'un** : chaque fichier de `tests/` est un programme à part, donc un processus neuf où l'état global de la vitrine n'a jamais été posé ; avec un seul test, aucun
//! autre test du processus ne peut appeler `givre_demarrer` avant lui. C'est ce qui permet d'éprouver en natif ce que les cinq fonctions de la neige rendent « avant tout démarrage » (`u32::MAX`, jamais une panique), que les autres
//! fichiers de G3 ne peuvent pas voir (leur état global est déjà posé par un autre test) et que `etape_g3_module.rs` ne voit que sous Node, une fois le module reconstruit. N'ajoutez aucun test ici.
//!
//! Tant qu'une des cinq fonctions `givre_neige_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test s'arrête d'un coup (une panique ne traverse pas une `extern "C" fn`) : c'est un échec.

use givre_lab::vitrine::{
    givre_avancer, givre_demarrer, givre_duree, givre_neige_demarrer, givre_neige_jeu, givre_neige_jeux,
    givre_neige_mesure, givre_neige_reglage, givre_nette_largeur, givre_nette_preparer, givre_temps,
};

/// juste : avant tout démarrage (V-G3-13), dans un processus neuf (ce fichier n'a qu'un test), `givre_neige_reglage` et `givre_neige_mesure` rendent `u32::MAX` pour tout champ (0 à 9 et 0 à 7, et au-delà, `u32::MAX` compris) : il n'y a
/// pas de neige en cours ; `givre_neige_jeux()` vaut 19 et `givre_neige_jeu(0, 0)` 9 (les jeux nommés ne dépendent d'aucun état) ; un `givre_neige_demarrer` refusé (jeu 19 ; `ρ` hors plage) ne démarre rien (le temps et la durée valent 0, la préparation de l'image nette
/// rend 2 : « rien n'est démarré », les lecteurs de la neige rendent encore `u32::MAX`) ; un `givre_demarrer` refusé (le modèle 8) non plus ; puis `givre_neige_demarrer(0, …)` démarre la neige (0), les lecteurs répondent, et un `givre_demarrer(0, 1)` (la DLA) la remplace :
/// les lecteurs de la neige rendent de nouveau `u32::MAX`. Une lecture qui panique avant tout démarrage, ou qui rend 0 au lieu de `u32::MAX`, se voit.
///
/// Ne couvre pas : les valeurs lues d'une neige démarrée (voir `etape_g3_vitrine.rs`) ; le module WebAssembly neuf (voir `etape_g3_module.rs`, la ligne `avant=`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn avant_tout_demarrage_la_neige_n_existe_pas() {
    let champs = [0_u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 100, u32::MAX];
    let rien = |contexte: &str| {
        for champ in champs {
            assert_eq!(givre_neige_reglage(champ), u32::MAX, "{contexte} : réglage {champ}");
            assert_eq!(givre_neige_mesure(champ), u32::MAX, "{contexte} : mesure {champ}");
        }
        assert_eq!((givre_temps(), givre_duree()), (0, 0), "{contexte} : temps et durée");
        assert_eq!(givre_nette_largeur(), 0, "{contexte} : le tampon de l'image nette");
    };
    rien("processus neuf");
    assert_eq!(givre_neige_jeux(), 19);
    assert_eq!(givre_neige_jeu(0, 0), 9);
    assert_eq!(givre_neige_jeu(18, 0), 13);
    assert_eq!(givre_nette_preparer(62, 2), 2, "rien n'est démarré");

    assert_eq!(givre_neige_demarrer(19, 400_000, 1_300_000, 3_000, 70_000), 1);
    rien("après un jeu inconnu");
    assert_eq!(givre_neige_demarrer(0, 1_000_001, 1_300_000, 3_000, 70_000), 2);
    rien("après un réglage hors plage");
    assert_eq!(givre_neige_demarrer(u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX), 1);
    rien("après des valeurs absurdes");
    assert_eq!(givre_demarrer(8, 0), 0, "le modèle 8 n'existe pas");
    rien("après un givre_demarrer refusé");
    assert_eq!(givre_avancer(5), 0, "rien à jouer");
    assert_eq!(givre_nette_preparer(62, 2), 2, "toujours rien de démarré");

    assert_eq!(givre_neige_demarrer(0, 400_000, 1_300_000, 3_000, 70_000), 0);
    assert_eq!(givre_neige_reglage(7), 0);
    assert_eq!(givre_neige_reglage(0), 400_000);
    assert_eq!(givre_neige_mesure(0), 1, "le germe seul : une case rattachée");
    assert_eq!(givre_duree(), 2000);
    assert_eq!(givre_demarrer(0, 1), 1);
    for champ in champs {
        assert_eq!(givre_neige_reglage(champ), u32::MAX, "la DLA remplace la neige : réglage {champ}");
        assert_eq!(givre_neige_mesure(champ), u32::MAX, "la DLA remplace la neige : mesure {champ}");
    }
}

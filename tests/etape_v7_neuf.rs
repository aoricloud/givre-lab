//! Étape V7 : l'image nette **avant tout démarrage**, en natif. Valeurs attendues : `references/etape-v7-valeurs.md`, identifiant `V-V7-09`.
//!
//! **Ce fichier ne contient qu'UN test, et il doit n'en contenir qu'un** : chaque fichier de `tests/` est un programme à part, donc un processus neuf où l'état global de la vitrine n'a jamais été posé ; avec un seul test, aucun
//! autre test du processus ne peut appeler `givre_demarrer` avant lui. C'est ce qui permet d'éprouver en natif le code 2 (« rien n'est démarré ») de `givre_nette_preparer`, que les autres fichiers de V7 ne peuvent pas voir
//! (leur état global est déjà posé par un autre test) et que `etape_v7_module.rs` ne voit que sous Node, une fois le module reconstruit. N'ajoutez aucun test ici.
//!
//! Tant qu'une des quatre fonctions `givre_nette_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test s'arrête d'un coup (une panique ne traverse pas une `extern "C" fn`) : c'est un échec.

use givre_lab::vitrine::{
    givre_avancer, givre_demarrer, givre_nette_hauteur, givre_nette_largeur, givre_nette_pixel, givre_nette_preparer,
    givre_temps,
};

/// Le tampon est vide : largeur et hauteur 0, tout pixel vaut `u32::MAX`.
fn exiger_le_tampon_vide(contexte: &str) {
    assert_eq!(givre_nette_largeur(), 0, "{contexte} : largeur du tampon");
    assert_eq!(givre_nette_hauteur(), 0, "{contexte} : hauteur du tampon");
    for (x, y) in [(0, 0), (1, 0), (0, 1), (61, 61), (u32::MAX, u32::MAX)] {
        assert_eq!(givre_nette_pixel(x, y), u32::MAX, "{contexte} : le pixel ({x}, {y}) d'un tampon vide");
    }
}

/// juste : avant tout démarrage, l'image nette n'existe pas et `givre_nette_preparer` rend le code 2 (V-V7-09). Dans un processus neuf (ce fichier n'a qu'un test), sans aucun `givre_demarrer` : la largeur et la hauteur du tampon valent 0
/// et tout pixel vaut `u32::MAX` ; `givre_nette_preparer` rend **2** pour des réglages ordinaires `(62, 2)` comme pour des réglages absurdes (`(0, 0)`, `(61, 2)`, `(62, 9)`, `(2048, 4)`, `(u32::MAX, u32::MAX)` : « rien n'est démarré » est
/// vérifié **avant** le modèle et les réglages, il ne rend donc ni 1, ni 3, ni 4, ni 5) et le tampon reste vide ; un `givre_demarrer` refusé (le modèle 9) ne change rien (toujours 2) ; un `givre_avancer` ne joue rien. Puis, après
/// `givre_demarrer(2, 0)`, le tampon est encore vide, `(62, 2)` rend 0 et donne une image de `62 × 62` pixels dont le coin est blanc ; et après `givre_demarrer(0, 1)` (la DLA) le tampon est de nouveau vide et la préparation rend 1 :
/// **un modèle carré démarré après un flocon ne garde pas l'image du flocon**.
///
/// Ne couvre pas : le contenu de l'image (voir `etape_v7_vitrine.rs`) ; le module WebAssembly neuf (voir `etape_v7_module.rs`, la ligne `avant=`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn avant_tout_demarrage_la_preparation_rend_le_code_2() {
    exiger_le_tampon_vide("processus neuf");
    for (cote, sur) in [(62_u32, 2_u32), (0, 0), (61, 2), (62, 9), (2048, 4), (u32::MAX, u32::MAX)] {
        assert_eq!(givre_nette_preparer(cote, sur), 2, "rien n'est démarré, ({cote}, {sur})");
        exiger_le_tampon_vide("après une préparation sans démarrage");
    }
    assert_eq!(givre_demarrer(9, 0), 0, "le modèle 9 n'existe pas");
    assert_eq!(givre_nette_preparer(62, 2), 2, "un démarrage refusé ne démarre rien");
    assert_eq!(givre_avancer(5), 0, "rien à jouer avant tout démarrage");
    assert_eq!(givre_temps(), 0);
    exiger_le_tampon_vide("après un démarrage refusé");

    assert_eq!(givre_demarrer(2, 0), 1);
    exiger_le_tampon_vide("après le premier démarrage, avant toute préparation");
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!((givre_nette_largeur(), givre_nette_hauteur()), (62, 62));
    assert_eq!(givre_nette_pixel(0, 0), 0x00FF_FFFF, "le coin de l'image est blanc");

    // Un modèle carré démarré après un flocon : le tampon du flocon ne survit pas.
    assert_eq!(givre_demarrer(0, 1), 1);
    exiger_le_tampon_vide("la DLA démarrée après un flocon préparé");
    assert_eq!(givre_nette_preparer(62, 2), 1, "la DLA n'est pas sur la grille hexagonale");
    exiger_le_tampon_vide("après le code 1");
}

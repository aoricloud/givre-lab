//! Étape D2 : les repères publiés (leur TRANSCRIPTION, jamais une comparaison) et les REFUS de `givre_lab::courbe`. Valeurs attendues : `references/etape-d2-valeurs.md`,
//! identifiants `V-D2-10` et `V-D2-11`.
//!
//! **La table des fautes visées** (chaque faute doit faire échouer au moins un test de l'étape ; la campagne de fautes jouée est dans `essai-d2-ecrivain.md`, dans les notes de la session) :
//!
//! | faute | test qui la voit |
//! |---|---|
//! | un `D` qui ne baisse pas avec `η` (poids sans `η`, `η` décalé, `η` ignoré) | `etape_d2_ordre::la_dimension_baisse_avec_eta`, `etape_d2_exact::point_de_la_courbe_egal_au_calcul_naif` |
//! | erreur-type avec `n` au lieu de `n - 1` | `etape_d2_exact::moyenne_et_erreur_type_a_la_main_et_en_bits`, `…::point_de_la_courbe_egal_au_calcul_naif` |
//! | somme de la moyenne ou de la variance à l'envers | `etape_d2_exact::moyenne_et_erreur_type_a_la_main_et_en_bits`, `…::point_de_la_courbe_egal_au_calcul_naif` (huit graines inversées) |
//! | un repère transposé (`1,433` et `1,263` échangés) | `reperes_publies_transcrits` |
//! | une fenêtre qui dépend du résultat, du nombre de sites, de leur position | `etape_d2_exact::fenetre_ne_depend_que_de_l_etendue`, `…::fenetre_suit_sa_formule`, `…::dimension_egale_la_correlation_sur_la_fenetre` |
//! | `η = 0` oublié ou mal traité | `etape_d2_exact::courbe_donne_les_points_dans_l_ordre_donne`, `etape_d2_ordre::la_dimension_baisse_avec_eta`, `etape_d2_exemple::csv_a_la_main` (`publie` vide) |
//! | un écart de signe inversé | `etape_d2_exemple::ecart_et_reperes_affiches`, `etape_d2_exemple::csv_a_la_main` |
//! | `courbe` qui ne rend pas l'ordre donné | `etape_d2_exact::courbe_donne_les_points_dans_l_ordre_donne` |
//! | le `ln` du système à la place de `math::logarithme` | `etape_0_regles::sources_sans_fonction_du_systeme` (un test de `main`, qui lit `src/courbe.rs`) et les bits de `dimension_egale_la_correlation_sur_la_fenetre` |
//! | une `taille` qui n'est pas atteinte sans le dire | `l_amas_arrete_a_la_frontiere_est_dit` |
//! | une dimension indéfinie passée sous silence (fenêtre vide) | `une_fenetre_vide_est_dite` |
//! | un point fait d'une seule graine | `un_point_demande_deux_graines` |

use givre_lab::courbe::{DLA_RAPPEL, HASTINGS_2001_TABLE_II, point_de_la_courbe, publie};

/// Le texte du message d'une panique attrapée.
fn texte_de(message: &(dyn std::any::Any + Send)) -> String {
    message
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| message.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// juste : la TRANSCRIPTION des repères publiés (V-D2-11), lue à la source et non d'après le code : Hastings, Phys. Rev. Lett. 87, 175502 (2001), arXiv:cond-mat/0103312v2, tableau II (« For various
/// values of η, calculated dimension based on different parts of data set »), colonne « full set » : `η = 2` : `1.433` ; `η = 3` : `1.263` (lu dans le PDF de l'arXiv le 09/11 ; les autres colonnes et lignes,
/// `1.4256`, `1.435`, `1.4522` et `1.264`, `1.262`, `1.243`, `η = 3,5` à `5`, ne sont pas transcrites). `HASTINGS_2001_TABLE_II` est exactement `[(2.0, 1.433), (3.0, 1.263)]` (à `to_bits` près, dans cet ordre : un
/// repère transposé, `(2.0, 1.263)` et `(3.0, 1.433)`, échoue) ; `DLA_RAPPEL` est `1.71` (Mathiesen, Jensen et Bakke, arXiv:0712.1739, section IV : « the DLA value D=1.71 for η=1 » ; Hastings écrit seulement
/// « roughly 1.7 » : ce n'est PAS une mesure de lui) ; `publie(η)` rend `Some(1.71)`, `Some(1.433)`, `Some(1.263)` pour `η = 1, 2, 3` et `None` pour `η = 0` et de 4 à 8.
///
/// Ne couvre pas : que ces valeurs soient celles de la revue imprimée (seul l'arXiv v2 est lu) ; la comparaison d'une mesure à ces valeurs (jamais testée : un test « proche de 1,433 » se règlerait pour passer).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn reperes_publies_transcrits() {
    assert_eq!(HASTINGS_2001_TABLE_II.len(), 2);
    let [(eta2, d2), (eta3, d3)] = HASTINGS_2001_TABLE_II;
    assert_eq!((eta2.to_bits(), d2.to_bits()), (2.0_f64.to_bits(), 1.433_f64.to_bits()), "η = 2 : {eta2} → {d2}");
    assert_eq!((eta3.to_bits(), d3.to_bits()), (3.0_f64.to_bits(), 1.263_f64.to_bits()), "η = 3 : {eta3} → {d3}");
    assert_eq!(DLA_RAPPEL.to_bits(), 1.71_f64.to_bits());
    assert_eq!(publie(1).map(f64::to_bits), Some(1.71_f64.to_bits()));
    assert_eq!(publie(2).map(f64::to_bits), Some(1.433_f64.to_bits()));
    assert_eq!(publie(3).map(f64::to_bits), Some(1.263_f64.to_bits()));
    for eta in [0, 4, 5, 6, 7, 8] {
        assert_eq!(publie(eta), None, "η = {eta} n'a pas de repère");
    }
}

/// juste : un point de la courbe de moins de deux graines est REFUSÉ (V-D2-10) : `point_de_la_courbe` avec une seule graine panique (message contenant « au moins deux graines ») ; une erreur-type d'une seule
/// graine n'existe pas, et un point qui se donnerait quand même (0, `NaN`) serait un mensonge.
///
/// Ne couvre pas : la liste vide (même refus, vu par `moyenne_et_erreur_type_refuse_aucune_valeur` pour les valeurs).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
#[should_panic(expected = "au moins deux graines")]
fn un_point_demande_deux_graines() {
    let _ = point_de_la_courbe(1, &[7], 40, 16);
}

/// juste : un amas qui touche la frontière avant d'avoir `taille` sites est DIT, pas passé sous silence (V-D2-10) : à `η = 1`, `demi_cote = 3`, `taille = 40` (le domaine n'a que 29 sites) et à `η = 0`,
/// `demi_cote = 5`, `taille = 100` (81 sites au plus), `point_de_la_courbe` panique, le message contient « frontière ». Un point fait de moins de sites que `taille` serait un point d'une autre courbe.
///
/// Ne couvre pas : la graine nommée dans le message ; le cas limite d'un amas qui touche la frontière EXACTEMENT au dernier site (il est alors accepté : `taille` atteinte, voir `Claquage`).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn l_amas_arrete_a_la_frontiere_est_dit() {
    for (eta, taille, demi_cote) in [(1_u32, 40_usize, 3_u32), (0, 100, 5)] {
        let resultat = std::panic::catch_unwind(|| point_de_la_courbe(eta, &[1, 2], taille, demi_cote));
        let message = resultat.expect_err("l'amas ne peut pas atteindre sa taille");
        let texte = texte_de(message.as_ref());
        assert!(texte.contains("frontière"), "η={eta} demi-côté {demi_cote} : message {texte:?}");
    }
}

/// juste : une dimension indéfinie est DITE (V-D2-10) : `point_de_la_courbe` d'amas d'un seul site (`taille = 1`) ou de quelques sites (`η = 0`, `taille = 5`, étendue de 2 à 4 : fenêtre vide) panique, le
/// message contient « fenêtre ». Une moyenne faite des seules graines qui ont une dimension changerait l'effectif sans le dire.
///
/// Ne couvre pas : le cas d'une `C(r)` nulle (un amas de claquage est connexe : `C(1) > 0`, donc jamais nulle).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn une_fenetre_vide_est_dite() {
    for taille in [1_usize, 5] {
        let resultat = std::panic::catch_unwind(|| point_de_la_courbe(0, &[1, 2], taille, 20));
        let message = resultat.expect_err("la fenêtre est vide");
        let texte = texte_de(message.as_ref());
        assert!(texte.contains("fenêtre"), "taille {taille} : message {texte:?}");
    }
}

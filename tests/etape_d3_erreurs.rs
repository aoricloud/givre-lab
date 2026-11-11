//! Étape D3 : les entrées de `givre_lab::courbe` qui rendent une erreur au lieu de paniquer (`ErreurCourbe`, `essayer_demi_cote_de_la_courbe`,
//! `essayer_dimension_d_une_graine`, `essayer_point_de_la_courbe`) et `mesure_petit_amas`. Valeurs attendues : `references/etape-d3-valeurs.md`, identifiants `V-D3-xx`.
//!
//! Les fonctions de D2 ne changent pas (leurs tests, gelés, les gardent : voir `etape_d2_fautes.rs`, qui exige la panique). Ici : chaque erreur est provoquée à la main
//! avec ses champs exacts, **aucun cas absurde ne panique**, les entrées neuves rendent les **mêmes bits** que celles de D2 quand rien n'échoue, et le texte de chaque erreur est fixé.
//!
//! Le calcul de comparaison est celui de D2 (`etape_d2_commun` : dimension et point **naïfs**, écrits d'après la seule doc, qui n'appellent ni `courbe` ni `correlation`).

mod etape_d3_commun;

use etape_d3_commun::d2::{
    amas_naif, carre, correlation_naive, croix, dimension_naive, etendue_doc, fenetre_doc, point_naif, trait_,
};
use givre_lab::claquage::{Claquage, DEMI_COTE_MAX, ETA_MAX, PARTICULES_MAX};
use givre_lab::courbe::{
    ETA_COURBE_MAX, ErreurCourbe, demi_cote_de_la_courbe, dimension_d_un_amas, essayer_demi_cote_de_la_courbe,
    essayer_dimension_d_une_graine, essayer_point_de_la_courbe, mesure_petit_amas, point_de_la_courbe,
};

/// Les bits d'un point de la courbe : `(eta, taille, graines, bits de la moyenne, bits de l'erreur-type)`.
fn bits(p: &givre_lab::courbe::PointDeLaCourbe) -> (u32, usize, usize, u64, u64) {
    (p.eta, p.taille, p.graines, p.moyenne.to_bits(), p.erreur_type.to_bits())
}

/// juste : chaque erreur est provoquée **à la main**, avec ses champs exacts (V-D3-01). Les graines : moins de deux (zéro, une) ; `eta` au-delà de 8 (9, `u32::MAX`) pour les deux
/// premières entrées et au-delà de 3 pour `essayer_demi_cote_de_la_courbe` ; `demi_cote` au-delà de 255 (256, `u32::MAX`) ; `taille` au-delà de 100 000 (100 001, `usize::MAX`) ; un amas
/// qui touche la frontière (`η = 1`, `taille = 40`, `demi_cote = 3` : le domaine n'a que 29 sites ; `η = 0`, `taille = 100`, `demi_cote = 5`) ; une fenêtre vide (`taille` 0, 1, 5). Les
/// **bornes sont incluses** : `eta = 8`, `demi_cote = 255`, `taille = 100 000` sont acceptés (ils ne donnent pas leur erreur de borne : `taille = 100 000` à `demi_cote = 10` donne la
/// frontière, `eta = 8` et `taille = 1` la fenêtre vide).
///
/// Ne couvre pas : l'ordre des vérifications entre elles (voir `l_ordre_des_verifications`) ; les bits des résultats justes (voir les tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn chaque_erreur_est_provoquee_a_la_main() {
    // GrainesInsuffisantes
    assert_eq!(essayer_point_de_la_courbe(1, &[], 40, 20), Err(ErreurCourbe::GrainesInsuffisantes { donnees: 0 }));
    assert_eq!(essayer_point_de_la_courbe(1, &[7], 40, 20), Err(ErreurCourbe::GrainesInsuffisantes { donnees: 1 }));
    // EtaTropGrand : la borne est 8 pour les deux premières entrées, 3 pour la règle du demi-côté
    for eta in [9, 100, u32::MAX] {
        assert_eq!(
            essayer_dimension_d_une_graine(eta, 1, 40, 20),
            Err(ErreurCourbe::EtaTropGrand { eta, maximum: 8 }),
            "η = {eta}"
        );
        assert_eq!(
            essayer_point_de_la_courbe(eta, &[1, 2], 40, 20),
            Err(ErreurCourbe::EtaTropGrand { eta, maximum: 8 }),
            "η = {eta}"
        );
    }
    for eta in [4, 5, 8, 9, u32::MAX] {
        assert_eq!(
            essayer_demi_cote_de_la_courbe(eta, 100),
            Err(ErreurCourbe::EtaTropGrand { eta, maximum: 3 }),
            "η = {eta}"
        );
    }
    assert_eq!(ETA_MAX, 8);
    assert_eq!(ETA_COURBE_MAX, 3);
    // DemiCoteHorsBorne
    for demi_cote in [256, 1000, u32::MAX] {
        assert_eq!(
            essayer_dimension_d_une_graine(0, 1, 40, demi_cote),
            Err(ErreurCourbe::DemiCoteHorsBorne { demi_cote }),
            "demi-côté {demi_cote}"
        );
        assert_eq!(
            essayer_point_de_la_courbe(0, &[1, 2], 40, demi_cote),
            Err(ErreurCourbe::DemiCoteHorsBorne { demi_cote }),
            "demi-côté {demi_cote}"
        );
    }
    assert_eq!(DEMI_COTE_MAX, 255);
    // TailleHorsBorne
    for taille in [100_001, 1_000_000, usize::MAX] {
        assert_eq!(
            essayer_dimension_d_une_graine(0, 1, taille, 20),
            Err(ErreurCourbe::TailleHorsBorne { taille }),
            "taille {taille}"
        );
        assert_eq!(
            essayer_point_de_la_courbe(0, &[1, 2], taille, 20),
            Err(ErreurCourbe::TailleHorsBorne { taille }),
            "taille {taille}"
        );
        assert_eq!(
            essayer_demi_cote_de_la_courbe(0, taille),
            Err(ErreurCourbe::TailleHorsBorne { taille }),
            "taille {taille}"
        );
    }
    assert_eq!(PARTICULES_MAX, 100_000);
    // FrontiereTouchee : la graine de l'erreur est celle de l'appel (la première fautive d'un point)
    assert_eq!(essayer_dimension_d_une_graine(1, 7, 40, 3), Err(ErreurCourbe::FrontiereTouchee { graine: 7 }));
    assert_eq!(essayer_dimension_d_une_graine(0, 9, 100, 5), Err(ErreurCourbe::FrontiereTouchee { graine: 9 }));
    assert_eq!(essayer_point_de_la_courbe(1, &[9, 4], 40, 3), Err(ErreurCourbe::FrontiereTouchee { graine: 9 }));
    assert_eq!(essayer_point_de_la_courbe(0, &[4, 9], 100, 5), Err(ErreurCourbe::FrontiereTouchee { graine: 4 }));
    // FenetreVide : un amas de 0, 1 ou 5 sites (étendue de moins de 8) n'a pas de dimension
    for taille in [0, 1, 5] {
        assert_eq!(
            essayer_dimension_d_une_graine(0, 3, taille, 20),
            Err(ErreurCourbe::FenetreVide { graine: 3 }),
            "taille {taille}"
        );
        assert_eq!(
            essayer_point_de_la_courbe(0, &[3, 4], taille, 20),
            Err(ErreurCourbe::FenetreVide { graine: 3 }),
            "taille {taille}"
        );
    }
    // Les bornes sont incluses : la valeur de la borne n'est PAS l'erreur de borne.
    assert_eq!(essayer_dimension_d_une_graine(8, 1, 1, 10), Err(ErreurCourbe::FenetreVide { graine: 1 }), "η = 8");
    assert_eq!(
        essayer_dimension_d_une_graine(0, 1, 1, 255),
        Err(ErreurCourbe::FenetreVide { graine: 1 }),
        "demi-côté 255"
    );
    assert_eq!(
        essayer_dimension_d_une_graine(0, 1, 100_000, 10),
        Err(ErreurCourbe::FrontiereTouchee { graine: 1 }),
        "taille 100 000"
    );
    assert!(essayer_demi_cote_de_la_courbe(3, 100).is_ok(), "η = 3");
    assert!(essayer_demi_cote_de_la_courbe(0, 100_000).is_ok(), "taille 100 000");
}

/// juste : **l'ordre des vérifications** est celui de la doc (V-D3-02). Pour `essayer_point_de_la_courbe` : les graines d'abord (même quand tout le reste est absurde), puis `eta`, puis
/// `demi_cote`, puis `taille`, puis la croissance des graines dans l'ordre donné. Pour `essayer_dimension_d_une_graine` : `eta`, `demi_cote`, `taille`. Pour
/// `essayer_demi_cote_de_la_courbe` : `eta`, `taille`. Une erreur de paramètre est rendue **avant** de faire croître le moindre amas : tous les cas ci-dessous finissent aussitôt.
///
/// Ne couvre pas : l'ordre entre une frontière touchée et une fenêtre vide d'une même graine (voir `chaque_erreur_est_provoquee_a_la_main` : elles ont des cas distincts).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn l_ordre_des_verifications() {
    // Tout est absurde à la fois : les graines passent avant eta, demi_cote et taille.
    assert_eq!(
        essayer_point_de_la_courbe(u32::MAX, &[7], usize::MAX, u32::MAX),
        Err(ErreurCourbe::GrainesInsuffisantes { donnees: 1 })
    );
    assert_eq!(
        essayer_point_de_la_courbe(u32::MAX, &[], usize::MAX, u32::MAX),
        Err(ErreurCourbe::GrainesInsuffisantes { donnees: 0 })
    );
    // Deux graines : eta, puis demi_cote, puis taille.
    assert_eq!(
        essayer_point_de_la_courbe(u32::MAX, &[1, 2], usize::MAX, u32::MAX),
        Err(ErreurCourbe::EtaTropGrand { eta: u32::MAX, maximum: 8 })
    );
    assert_eq!(
        essayer_point_de_la_courbe(1, &[1, 2], usize::MAX, u32::MAX),
        Err(ErreurCourbe::DemiCoteHorsBorne { demi_cote: u32::MAX })
    );
    assert_eq!(
        essayer_point_de_la_courbe(1, &[1, 2], usize::MAX, 255),
        Err(ErreurCourbe::TailleHorsBorne { taille: usize::MAX })
    );
    // Une graine : eta, puis demi_cote, puis taille.
    assert_eq!(
        essayer_dimension_d_une_graine(u32::MAX, 1, usize::MAX, u32::MAX),
        Err(ErreurCourbe::EtaTropGrand { eta: u32::MAX, maximum: 8 })
    );
    assert_eq!(
        essayer_dimension_d_une_graine(1, 1, usize::MAX, u32::MAX),
        Err(ErreurCourbe::DemiCoteHorsBorne { demi_cote: u32::MAX })
    );
    assert_eq!(
        essayer_dimension_d_une_graine(1, 1, usize::MAX, 255),
        Err(ErreurCourbe::TailleHorsBorne { taille: usize::MAX })
    );
    // La règle du demi-côté : eta, puis taille.
    assert_eq!(essayer_demi_cote_de_la_courbe(4, usize::MAX), Err(ErreurCourbe::EtaTropGrand { eta: 4, maximum: 3 }));
}

/// juste : `essayer_demi_cote_de_la_courbe` rend `demi_cote_de_la_courbe` (V-D3-03) pour `η = 0` à `3` et chaque taille de 0 à 100 000 d'une liste (0, 1, 7, 50, 99, 100, 101, 250, 300,
/// 1000, 20 000, 100 000), et **sa valeur est celle de la règle de la doc écrite à la main** (`min(255, A + B × taille / 100)`, `A = [30, 32, 22, 12]`, `B = [10, 8, 18, 36]`) ; en
/// particulier le plafond 255 est atteint (`η = 3`, 1000 sites) et le produit `B × taille` ne déborde jamais (`taille = 100 000`, `η = 3` : 255). `usize::MAX / 10 + 1`, où D2 déborde, est une erreur
/// de taille.
///
/// Ne couvre pas : la mesure du demi-côté (D2).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_demi_cote_essaye_est_celui_de_la_regle() {
    for eta in 0..=3 {
        for taille in [0_usize, 1, 7, 50, 99, 100, 101, 250, 300, 1000, 20_000, 100_000] {
            let attendu = etape_d3_commun::demi_cote_regle(eta, taille);
            assert_eq!(essayer_demi_cote_de_la_courbe(eta, taille), Ok(attendu), "η = {eta}, taille {taille}");
            assert_eq!(demi_cote_de_la_courbe(eta, taille), attendu, "η = {eta}, taille {taille} (D2)");
        }
    }
    assert_eq!(essayer_demi_cote_de_la_courbe(3, 1000), Ok(255));
    assert_eq!(essayer_demi_cote_de_la_courbe(3, 100_000), Ok(255));
    assert_eq!(essayer_demi_cote_de_la_courbe(0, 100), Ok(40));
    assert_eq!(essayer_demi_cote_de_la_courbe(3, 100), Ok(48));
    assert_eq!(
        essayer_demi_cote_de_la_courbe(0, usize::MAX / 10 + 1),
        Err(ErreurCourbe::TailleHorsBorne { taille: usize::MAX / 10 + 1 })
    );
}

/// juste : le **même point** que D2, à `to_bits` près (V-D3-04). Pour `η = 0` à `3` à 40 sites (graines `[1, 2]`, demi-côté de la règle), un `demi_cote` choisi par l'appelant (`η = 2`, 33), des graines
/// répétées et en désordre (`η = 0` : `[5, 5, 6]`, `[3, 1, 2]`) et huit graines (`η = 0`, 60 sites) : `essayer_point_de_la_courbe` donne `eta`, `taille`, `graines`, les bits de la moyenne et ceux de
/// l'erreur-type de `point_de_la_courbe` ; pour `η = 1` et `η = 3` (et les cas d'`η = 0`, presque gratuits), ET ceux du calcul **naïf** de D2 (`point_naif`, écrit d'après la doc seule).
///
/// Ne couvre pas : les cas d'erreur (voir `chaque_erreur_est_provoquee_a_la_main`) ; la borne de la page, 100 sites (voir `etape_d3_vitrine.rs`, qui compare la courbe de la vitrine à cette entrée).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_point_essaye_egale_le_point_de_d2_a_to_bits_pres() {
    // (eta, graines, taille, demi_cote, comparé aussi au calcul naïf)
    let mut cas: Vec<(u32, Vec<u64>, usize, u32, bool)> = Vec::new();
    for eta in 0..=3 {
        cas.push((eta, vec![1, 2], 40, etape_d3_commun::demi_cote_regle(eta, 40), eta != 2));
    }
    cas.push((2, vec![1, 2], 40, 33, false));
    cas.push((0, vec![5, 5, 6], 60, 40, true));
    cas.push((0, vec![3, 1, 2], 60, 40, true));
    cas.push((0, (1..=8).collect(), 60, 40, true));
    for (eta, graines, taille, demi_cote, avec_le_naif) in cas {
        let essaye = essayer_point_de_la_courbe(eta, &graines, taille, demi_cote)
            .unwrap_or_else(|e| panic!("η = {eta}, graines {graines:?}, taille {taille} : {e}"));
        let de_d2 = point_de_la_courbe(eta, &graines, taille, demi_cote);
        assert_eq!(
            bits(&essaye),
            bits(&de_d2),
            "η = {eta}, graines {graines:?}, taille {taille}, demi-côté {demi_cote}"
        );
        assert_eq!((essaye.eta, essaye.taille, essaye.graines), (eta, taille, graines.len()));
        if avec_le_naif {
            let (moyenne, erreur_type) = point_naif(eta, &graines, taille, demi_cote);
            assert_eq!(essaye.moyenne.to_bits(), moyenne.to_bits(), "moyenne naïve, η = {eta}, graines {graines:?}");
            assert_eq!(
                essaye.erreur_type.to_bits(),
                erreur_type.to_bits(),
                "erreur-type naïve, η = {eta}, graines {graines:?}"
            );
        }
    }
}

/// juste : `essayer_dimension_d_une_graine` donne la dimension d'**un** amas, bit pour bit celle du calcul naïf de D2 sur l'amas que la graine fait croître (V-D3-05) : pour `η = 0` à `3` à 40 sites,
/// graines 1 à 3, et bit pour bit celle de `dimension_d_un_amas` ; une graine déjà vue donne la même dimension deux fois (aucune mémoire d'un appel à l'autre) ; et la dimension dépend de la graine
/// (les trois graines de `η = 1` donnent trois bits différents) et de `η` (les quatre `η` de la graine 1 aussi). Un `η` au-delà de la règle du demi-côté (`η = 5`, graine 1, 40 sites,
/// demi-côté 40) est accepté et donne la dimension du calcul naïf **de cet `η`** (différente de celle de `η = 3` : un `η` plafonné à 3 dans la croissance se verrait).
///
/// Ne couvre pas : les cas d'erreur ; la moyenne et l'erreur-type (voir le test du point) ; `η = 6` à `8`.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_dimension_d_une_graine_est_celle_du_calcul_naif() {
    let mut par_graine = Vec::new();
    let mut par_eta = Vec::new();
    for eta in 0..=3 {
        let demi_cote = etape_d3_commun::demi_cote_regle(eta, 40);
        for graine in 1..=3_u64 {
            let amas = amas_naif(graine, demi_cote, eta, 40).expect("l'amas atteint sa taille");
            let attendue = dimension_naive(&amas).expect("une dimension");
            let obtenue = essayer_dimension_d_une_graine(eta, graine, 40, demi_cote).expect("une dimension");
            assert_eq!(obtenue.to_bits(), attendue.to_bits(), "η = {eta}, graine {graine}");
            assert_eq!(
                Some(obtenue.to_bits()),
                dimension_d_un_amas(&amas).map(f64::to_bits),
                "η = {eta}, graine {graine} (D2)"
            );
            if eta == 1 {
                par_graine.push(obtenue.to_bits());
            }
            if graine == 1 {
                par_eta.push(obtenue.to_bits());
            }
        }
    }
    for liste in [&mut par_graine, &mut par_eta] {
        let avant = liste.len();
        liste.sort();
        liste.dedup();
        assert_eq!(liste.len(), avant, "des dimensions différentes pour des graines (ou des η) différents");
    }
    let une = essayer_dimension_d_une_graine(1, 3, 40, 40).expect("une dimension");
    let deux = essayer_dimension_d_une_graine(1, 3, 40, 40).expect("une dimension");
    assert_eq!(une.to_bits(), deux.to_bits());
    // η = 5 (au-delà de la règle du demi-côté, en deçà de ETA_MAX) : l'amas est bien celui de cet η.
    let amas = amas_naif(1, 40, 5, 40).expect("l'amas de η = 5 atteint 40 sites");
    let attendue = dimension_naive(&amas).expect("une dimension");
    let obtenue = essayer_dimension_d_une_graine(5, 1, 40, 40).expect("une dimension");
    assert_eq!(obtenue.to_bits(), attendue.to_bits(), "η = 5, graine 1");
    let a_eta_3 = essayer_dimension_d_une_graine(3, 1, 40, 40).expect("une dimension");
    assert_ne!(obtenue.to_bits(), a_eta_3.to_bits(), "η = 5 ne donne pas l'amas de η = 3");
}

/// juste : un amas qui atteint **exactement** sa taille en touchant la frontière est accepté, un site de plus est refusé (V-D3-06, la règle de D2 : `Claquage::essayer_ajouter` rend
/// `None` au pas suivant). On cherche, par le code natif de D1, le nombre de sites `n` de l'amas (`η = 0`, `demi_cote = 12`, graine 4) au moment où `touche_la_frontiere()` devient vrai ;
/// `essayer_dimension_d_une_graine(0, 4, n, 12)` rend `Ok` de la dimension naïve de cet amas, et `n + 1` donne `FrontiereTouchee { graine: 4 }`.
///
/// Ne couvre pas : la frontière à `η >= 1` (même code, mais coûteux).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_frontiere_touchee_au_dernier_site_est_acceptee() {
    let mut claquage = Claquage::new(4, 12, 0);
    while !claquage.touche_la_frontiere() {
        claquage.ajouter();
    }
    let n = claquage.sites().len();
    assert!(n > 20, "l'amas de la graine 4 touche la frontière à {n} sites : assez pour avoir une dimension");
    let attendue = dimension_naive(claquage.sites());
    let obtenue = essayer_dimension_d_une_graine(0, 4, n, 12);
    assert_eq!(obtenue.map(f64::to_bits), Ok(attendue.expect("une dimension").to_bits()), "exactement {n} sites");
    assert_eq!(essayer_dimension_d_une_graine(0, 4, n + 1, 12), Err(ErreurCourbe::FrontiereTouchee { graine: 4 }));
    assert_eq!(essayer_point_de_la_courbe(0, &[4, 5], n + 1, 12), Err(ErreurCourbe::FrontiereTouchee { graine: 4 }));
}

/// juste : le texte de `Display` de chaque erreur est celui de la doc, caractère pour caractère (V-D3-07), et `ErreurCourbe` est une `std::error::Error` (rendue en `Box<dyn Error>`).
///
/// Ne couvre pas : la mise en forme de `Debug`.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_texte_de_chaque_erreur() {
    let cas: [(ErreurCourbe, &str); 6] = [
        (ErreurCourbe::GrainesInsuffisantes { donnees: 1 }, "au moins deux graines sont nécessaires, 1 données"),
        (ErreurCourbe::EtaTropGrand { eta: 9, maximum: 8 }, "η = 9 : au-delà du maximum 8"),
        (ErreurCourbe::DemiCoteHorsBorne { demi_cote: 256 }, "demi-côté 256 : au-delà du maximum 255"),
        (ErreurCourbe::TailleHorsBorne { taille: 100_001 }, "taille 100001 : au-delà du maximum 100000"),
        (
            ErreurCourbe::FrontiereTouchee { graine: 7 },
            "la graine 7 : l'amas touche la frontière avant d'atteindre sa taille",
        ),
        (ErreurCourbe::FenetreVide { graine: 3 }, "la graine 3 : aucune dimension (fenêtre vide ou C(r) nul)"),
    ];
    for (erreur, texte) in cas {
        assert_eq!(erreur.to_string(), texte);
        let boite: Box<dyn std::error::Error> = Box::new(erreur.clone());
        assert_eq!(boite.to_string(), texte);
    }
    // Les erreurs réellement rendues disent leurs champs (pas ceux d'un autre cas).
    let rendue = essayer_dimension_d_une_graine(1, 7, 40, 3).expect_err("la frontière");
    assert_eq!(rendue.to_string(), "la graine 7 : l'amas touche la frontière avant d'atteindre sa taille");
    let rendue = essayer_point_de_la_courbe(1, &[7], 40, 20).expect_err("une graine");
    assert_eq!(rendue.to_string(), "au moins deux graines sont nécessaires, 1 données");
}

/// juste : **aucun cas absurde ne panique** (V-D3-08). Une grille de paramètres absurdes (`η` de 0, 9, `u32::MAX` ; `demi_cote` de 0, 1, 255, 256, `u32::MAX` ; `taille` de 0, 1, 5,
/// 100 001, `usize::MAX` ; graines `[]`, `[1]`, `[1, 2]`) est donnée aux trois entrées, chaque appel sous `catch_unwind` : aucun ne panique. Les combinaisons qui ne sont pas
/// des erreurs de paramètre (`η = 0`, `taille` 0, 1 ou 5) finissent aussitôt (fenêtre vide, ou la frontière à `demi_cote` 0 ou 1). Un `demi_cote` de 0 (le germe touche la frontière) et de 1
/// sont des cas limites : `taille = 5` à `demi_cote = 0` donne la frontière, pas une panique.
///
/// Ne couvre pas : un amas de grande taille (le temps) ; les paniques de la bibliothèque qui ne passent pas par ces entrées (`Claquage::relaxer` ne converge pas : jamais atteinte).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn aucun_cas_absurde_ne_panique() {
    let graines_possibles: [&[u64]; 3] = [&[], &[1], &[1, 2]];
    for eta in [0_u32, 9, u32::MAX] {
        for demi_cote in [0_u32, 1, 255, 256, u32::MAX] {
            for taille in [0_usize, 1, 5, 100_001, usize::MAX] {
                let resultat = std::panic::catch_unwind(|| {
                    let _ = essayer_demi_cote_de_la_courbe(eta, taille);
                    let _ = essayer_dimension_d_une_graine(eta, 1, taille, demi_cote);
                    for graines in graines_possibles {
                        let _ = essayer_point_de_la_courbe(eta, graines, taille, demi_cote);
                    }
                });
                assert!(resultat.is_ok(), "panique pour η = {eta}, demi-côté {demi_cote}, taille {taille}");
            }
        }
    }
    // Les cas limites du demi-côté 0 : le germe touche la frontière.
    assert_eq!(essayer_dimension_d_une_graine(0, 1, 5, 0), Err(ErreurCourbe::FrontiereTouchee { graine: 1 }));
    assert_eq!(essayer_dimension_d_une_graine(0, 1, 1, 0), Err(ErreurCourbe::FenetreVide { graine: 1 }));
    // η = 8 : accepté (un champ à φ^8), sur un petit domaine.
    assert_eq!(essayer_dimension_d_une_graine(8, 1, 5, 6), Err(ErreurCourbe::FenetreVide { graine: 1 }));
}

/// Un amas d'un nombre de sites de la forme `forme`, pour `mesure_petit_amas`.
fn formes() -> Vec<(&'static str, Vec<(i32, i32)>)> {
    vec![
        ("trait de 30 sites", trait_(30)),
        ("trait de 61 sites", trait_(61)),
        ("carré 12 x 12", carre(12)),
        ("croix de bras 15", croix(15)),
        ("amas de claquage η = 0, 60 sites", amas_naif(2, 40, 0, 60).expect("amas")),
        ("amas de claquage η = 2, 60 sites", amas_naif(2, 40, 2, 60).expect("amas")),
        ("amas de claquage η = 3, 60 sites", amas_naif(2, 40, 3, 60).expect("amas")),
    ]
}

/// juste : `mesure_petit_amas` est la fenêtre des petits amas, les `C(r)` et la dimension (V-D3-09). Pour des formes à solution connue et des amas de claquage : `rayons` est
/// `fenetre_doc(etendue_doc)` (la formule de la doc, `2..=max(4, e / 6)`), `valeurs[i]` est `C(rayons[i])` du calcul naïf de D2 (paires comptées une à une, dénominateur entier exact), au bit
/// près, et `dimension` est celle du calcul naïf ET de `dimension_d_un_amas`, au bit près. Rend `None` quand il n'y a pas de dimension : liste vide, un site, un trait de 5 sites (fenêtre
/// vide), deux sites à distance 20 (une `C(r)` nulle).
///
/// Ne couvre pas : la valeur physique de la dimension (D2).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_mesure_des_petits_amas_est_le_calcul_naif() {
    for (nom, sites) in formes() {
        let rayons = fenetre_doc(etendue_doc(&sites));
        let mesure = mesure_petit_amas(&sites).unwrap_or_else(|| panic!("{nom} : une mesure"));
        assert_eq!(mesure.rayons, rayons, "{nom} : rayons");
        let attendues = correlation_naive(&sites, &rayons);
        assert_eq!(mesure.valeurs.len(), attendues.len(), "{nom} : nombre de valeurs");
        for (i, (obtenue, attendue)) in mesure.valeurs.iter().zip(&attendues).enumerate() {
            assert_eq!(obtenue.to_bits(), attendue.to_bits(), "{nom} : C(r) du rang {i}");
        }
        assert_eq!(
            mesure.dimension.to_bits(),
            dimension_naive(&sites).expect("une dimension").to_bits(),
            "{nom} : dimension naïve"
        );
        assert_eq!(
            Some(mesure.dimension.to_bits()),
            dimension_d_un_amas(&sites).map(f64::to_bits),
            "{nom} : dimension de D2"
        );
    }
    let sans_mesure: [(&str, Vec<(i32, i32)>); 4] = [
        ("liste vide", Vec::new()),
        ("un site", vec![(0, 0)]),
        ("trait de 5 sites", trait_(5)),
        ("deux sites éloignés", vec![(0, 0), (20, 0)]),
    ];
    for (nom, sites) in sans_mesure {
        assert!(mesure_petit_amas(&sites).is_none(), "{nom}");
        assert!(dimension_d_un_amas(&sites).is_none(), "{nom} (D2)");
    }
}

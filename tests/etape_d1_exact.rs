//! Étape D1 : le claquage diélectrique (`givre_lab::claquage`), tests EXACTS. Valeurs attendues : `references/etape-d1-valeurs.md`,
//! identifiants `V-D1-xx`.
//!
//! Le test le plus fort est `amas_egal_au_simulateur_naif` : un simulateur NAÏF écrit dans `etape_d1_commun/mod.rs` (des boucles
//! simples sur toute la boîte, rien d'optimisé) qui répète mot pour mot la section « La dynamique, à la lettre » de la doc de
//! `src/claquage.rs` : la sur-relaxation rouge-noir, les candidates, les poids `φ^η` par multiplications répétées, le tirage, l'arrêt.
//! La bibliothèque doit donner EXACTEMENT ses sites, ses candidates, ses poids et ses potentiels, au bit près, et l'état de son
//! générateur. Il a un défaut qu'on connaît : il est écrit par la même main que la doc. C'est pourquoi `valeurs_du_code_independant`
//! compare aussi à des valeurs calculées par un TROISIÈME code, en Python (`d1_ref.py`, d'après la doc seule, SHA-256 par `hashlib`).

mod etape_d1_commun;

use etape_d1_commun::{Naif, SEUIL_DOC, etat_du_generateur, hex, omega_doc};
use givre_lab::alea::Alea;
use givre_lab::claquage::{Claquage, DEMI_COTE_MAX, ETA_MAX, PARTICULES_MAX, PASSES_MAX, SEUIL, surrelaxation};
use givre_lab::empreinte::sha256_hex;

/// Compare un `Claquage` à un `Naif` : sites, rayon, arrêt, candidates (sites et BITS des poids), potentiel de chaque site de la boîte
/// (bits), état du générateur.
fn comparer(c: &Claquage, n: &Naif, contexte: &str) {
    assert_eq!(c.sites(), &n.sites[..], "{contexte} : sites");
    assert_eq!(c.rayon(), n.rayon(), "{contexte} : rayon");
    assert_eq!(c.touche_la_frontiere(), n.touche, "{contexte} : touche la frontière");
    assert_eq!(etat_du_generateur(&c.ecriture_canonique()), n.octets_alea(), "{contexte} : état du générateur");
    let attendues = n.candidates();
    let obtenues = c.candidates();
    assert_eq!(obtenues.len(), attendues.len(), "{contexte} : nombre de candidates");
    for (k, (a, b)) in obtenues.iter().zip(&attendues).enumerate() {
        assert_eq!(a.0, b.0, "{contexte} : candidate {k} (ordre canonique)");
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "{contexte} : poids de la candidate {:?} : {} contre {}",
            a.0,
            a.1,
            b.1
        );
    }
    if c.eta() == 0 {
        assert!(c.potentiel(0, 0).is_none() && c.potentiel(1, 0).is_none(), "{contexte} : pas de champ à eta = 0");
        return;
    }
    let d = c.demi_cote() as i32;
    for y in -d - 2..=d + 2 {
        for x in -d - 2..=d + 2 {
            let obtenu = c.potentiel(x, y).expect("un champ à eta >= 1");
            assert_eq!(
                obtenu.to_bits(),
                n.valeur(x, y).to_bits(),
                "{contexte} : potentiel de ({x}, {y}) : {obtenu} contre {}",
                n.valeur(x, y)
            );
        }
    }
}

/// juste : le simulateur naïf écrit dans ce fichier (voir plus haut ; V-D1-01), pour 4 exposants (`η` = 0, 1, 2, 3), 6 graines (1 à 6)
/// et `demi_cote = 14` : à 1, 4, 10, 20 puis 30 sites (ou à l'arrêt, si l'amas touche la frontière avant), le même `Claquage`, grandi
/// pas à pas, a EXACTEMENT les sites, le rayon, l'arrêt, les candidates, les BITS des poids (`f64::to_bits`), les BITS du potentiel de
/// chaque site de la boîte (et du bord au-delà) et l'état du générateur du simulateur naïf. Puis `η` = 4, 5 et 8 sur de petits amas
/// (`demi_cote = 10`, 12 sites) : les poids `φ^η` y sont des produits de plus de deux facteurs, dont l'ordre des multiplications change
/// les bits (un `powi` du système, qui multiplie par carrés, y diffère de multiplications répétées). Enfin des amas jusqu'à l'ARRÊT
/// (jusqu'à ce que `essayer_ajouter` rende `None`), comparés à chaque pas, pour `η` = 0, 1, 2 : `demi_cote = 5` (graines 21 à 25),
/// `demi_cote` 2, 3 et 4 (graines 21 à 60) et 6 (graines 21 à 40) ; ces petits domaines atteignent les bords du disque, où une ligne ou
/// une colonne extrême oubliée (le site `(0, d)` par exemple, seul sur sa ligne) change les candidates (ajouté par le juge : un mutant qui
/// oubliait la ligne `y = d` passait).
///
/// Ne couvre pas : les grands domaines (le simulateur naïf y est lent ; voir `etape_d1_dimensions`), ni l'accord avec un code écrit par
/// une autre main (voir `valeurs_du_code_independant`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn amas_egal_au_simulateur_naif() {
    for eta in 0..=3_u32 {
        for graine in 1..=6_u64 {
            let mut c = Claquage::new(graine, 14, eta);
            let mut naif = Naif::new(graine, 14, eta);
            comparer(&c, &naif, &format!("eta {eta}, graine {graine}, germe seul"));
            for n in [4_usize, 10, 20, 30] {
                let atteint_naif = naif.croitre(n);
                let atteint = (|| {
                    while c.sites().len() < n {
                        c.essayer_ajouter()?;
                    }
                    Some(())
                })()
                .is_some();
                assert_eq!(atteint, atteint_naif, "eta {eta}, graine {graine}, {n} sites : atteint ou arrêté");
                comparer(&c, &naif, &format!("eta {eta}, graine {graine}, {n} sites"));
                if !atteint {
                    break;
                }
            }
        }
    }
    for eta in [4_u32, 5, 8] {
        for graine in [1_u64, 2, 3] {
            let mut c = Claquage::new(graine, 10, eta);
            let mut naif = Naif::new(graine, 10, eta);
            for n in [1_usize, 5, 12] {
                let atteint_naif = naif.croitre(n);
                while c.sites().len() < n && c.essayer_ajouter().is_some() {}
                assert_eq!(c.sites().len() == n, atteint_naif, "eta {eta}, graine {graine}, {n} sites");
                comparer(&c, &naif, &format!("eta {eta}, graine {graine}, {n} sites"));
            }
        }
    }
    for (d, graines) in [(5_u32, 21..=25_u64), (2, 21..=60), (3, 21..=60), (4, 21..=60), (6, 21..=40)] {
        for eta in 0..=2_u32 {
            for graine in graines.clone() {
                arret_contre_le_naif(graine, d, eta);
            }
        }
    }
}

/// Un amas grandi jusqu'à l'ARRÊT, comparé pas à pas au simulateur naïf (voir `amas_egal_au_simulateur_naif`).
fn arret_contre_le_naif(graine: u64, d: u32, eta: u32) {
    let mut c = Claquage::new(graine, d, eta);
    let mut naif = Naif::new(graine, d as i32, eta);
    loop {
        let a = c.essayer_ajouter();
        let b = naif.pas();
        assert_eq!(a, b, "d {d}, eta {eta}, graine {graine} : le site ajouté (ou l'arrêt)");
        comparer(&c, &naif, &format!("d {d}, eta {eta}, graine {graine}, {} sites", naif.sites.len()));
        if a.is_none() {
            break;
        }
    }
    assert!(c.touche_la_frontiere(), "d {d}, eta {eta}, graine {graine} : l'arrêt vient de la frontière");
}

/// juste : comme `amas_egal_au_simulateur_naif`, sur un domaine plus grand et des amas plus longs (V-D1-02) : `demi_cote = 24`, graines 31 à
/// 33, `η` = 1, 2 et 3, 80 sites (ou jusqu'à l'arrêt), comparés au simulateur naïf à la fin (sites, potentiel de toute la boîte, poids,
/// générateur). C'est ici que les erreurs qui ne se voient qu'à longue portée (un voisin diagonal, un côté oublié, une relaxation
/// arrêtée trop tôt : le champ diffère alors de plus en plus) ont le temps de changer un tirage.
///
/// Ne couvre pas : les domaines au-delà de `demi_cote = 24`.
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn amas_egal_au_simulateur_naif_plus_long() {
    for eta in 1..=3_u32 {
        for graine in 31..=33_u64 {
            let mut c = Claquage::new(graine, 24, eta);
            let mut naif = Naif::new(graine, 24, eta);
            let atteint_naif = naif.croitre(80);
            while c.sites().len() < 80 && c.essayer_ajouter().is_some() {}
            assert_eq!(c.sites().len() == 80, atteint_naif, "eta {eta}, graine {graine} : atteint ou arrêté");
            comparer(&c, &naif, &format!("eta {eta}, graine {graine}, {} sites", naif.sites.len()));
        }
    }
}

/// juste : le germe seul (V-D1-03). Les quatre candidates du premier pas sont, dans l'ordre canonique (`y` croissant, puis `x`
/// croissant), `(0,-1) (-1,0) (1,0) (0,1)` ; à `η = 0` leurs poids valent exactement `1.0` et le premier site est la candidate
/// d'indice `⌊4 t⌋`, `t` étant le premier `suivant_f64` de `Alea::new(graine)` (400 graines, sans exception : `4 t` est exact) ; à `η = 1`
/// à `3`, les quatre poids sont ÉGAUX (à `1e-13` près : la symétrie du champ du germe seul, les quatre sommes de voisins ne différant
/// que par l'ordre d'addition ; mesuré : au plus `1,7e-16`) et le premier site est encore la candidate d'indice `⌊4 t⌋` (sauf si `4 t` est à moins de `1e-9` d'un
/// entier, où l'arrondi des poids peut décider). Les quatre candidates sont tirées de façon à peu près équilibrée (mesuré : de 95 à
/// 103 fois sur 400 graines pour chaque `η`, les mêmes effectifs puisque le premier choix est `⌊4 t⌋` ; la formule exacte est le vrai test, la borne 70 à 130 n'est qu'un recoupement).
///
/// Ne couvre pas : les pas suivants (voir `amas_egal_au_simulateur_naif`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn germe_seul_quatre_candidates_symetriques() {
    let attendues = [(0, -1), (-1, 0), (1, 0), (0, 1)];
    for eta in 0..=3_u32 {
        let mut effectifs = [0_usize; 4];
        for graine in 1..=400_u64 {
            let mut c = Claquage::new(graine, 6, eta);
            let candidates = c.candidates();
            assert_eq!(
                candidates.iter().map(|k| k.0).collect::<Vec<_>>(),
                attendues,
                "eta {eta} : les quatre candidates"
            );
            if eta == 0 {
                assert!(candidates.iter().all(|k| k.1.to_bits() == 1.0_f64.to_bits()), "eta 0 : poids exactement 1.0");
            } else {
                let w0 = candidates[0].1;
                assert!(w0 > 0.0 && w0 < 1.0, "le poids d'une candidate du germe seul est dans ]0, 1[ : {w0}");
                for k in &candidates {
                    assert!((k.1 - w0).abs() <= 1e-13, "eta {eta}, graine {graine} : poids {} et {w0}", k.1);
                }
            }
            let t = Alea::new(graine).suivant_f64();
            let indice = (4.0 * t).floor() as usize;
            let proche_d_un_entier = (4.0 * t - (4.0 * t).round()).abs() < 1e-9;
            let site = c.ajouter();
            if eta == 0 || !proche_d_un_entier {
                assert_eq!(site, attendues[indice], "eta {eta}, graine {graine} : t = {t}");
            }
            effectifs[attendues.iter().position(|&a| a == site).expect("une des quatre candidates")] += 1;
        }
        assert!(
            effectifs.iter().all(|&e| (70..=130).contains(&e)),
            "eta {eta} : premiers sites par candidate : {effectifs:?}"
        );
    }
}

/// juste : les ÉGALITÉS du tirage (V-D1-04). `Alea::new(g).suivant_f64()` vaut exactement `0,25`, `0,5` ou `0,75` pour trois graines
/// FABRIQUÉES (en inversant l'amorçage de PCG32, comme à l'étape 1 ; code Python `d1_ref.py fabriquer`) : à `η = 0`, avec les quatre
/// candidates du germe seul, `u = t * W` vaut exactement `1,0`, `2,0` ou `3,0`, et la règle « la première somme cumulée qui dépasse
/// STRICTEMENT `u` » donne les candidates d'indice 1, 2 et 3 (`(-1,0)`, `(1,0)`, `(0,1)`), pas 0, 1 et 2 comme le ferait un `>=`. À `η = 1` les
/// poids du germe seul sont égaux à un arrondi près : l'égalité y est décidée par l'arrondi des sommes, et le résultat attendu est
/// celui du code Python, qui fait les mêmes opérations dans le même ordre (c'est une vérification des bits de la doc, pas une
/// propriété qui survivrait à une autre forme de somme). Les empreintes de 12 sites sont celles du code Python indépendant.
///
/// Ne couvre pas : les égalités à plus de quatre candidates (non fabriquées).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn egalites_du_tirage() {
    // (graine fabriquée, t, indice attendu, premiers sites η = 0, empreinte η = 0 à 12 sites, rayon)
    #[allow(clippy::type_complexity)]
    let cas: [(u64, f64, usize, [(i32, i32); 6], &str, u32); 3] = [
        (
            12_457_697_844_934_659_938,
            0.25,
            1,
            [(0, 0), (-1, 0), (1, 0), (-2, 0), (-1, 1), (-1, -1)],
            "7e942fb53e1d298fb6d97a9d29fea33d8a134808686ddff1003cc2537c1505ec",
            4,
        ),
        (
            8_335_436_391_877_801_038,
            0.5,
            2,
            [(0, 0), (1, 0), (1, -1), (0, -1), (0, -2), (2, 0)],
            "47e75c715b0e0c72a20bd7308efb033a8d070263c235965ba42c153b6eebfb1b",
            3,
        ),
        (
            8_760_127_956_802_293_904,
            0.75,
            3,
            [(0, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (-1, -1)],
            "f8c7d63c0790873eaebbc2c369fbb1b61084556b819bba551ea879b29191156f",
            4,
        ),
    ];
    let candidates = [(0, -1), (-1, 0), (1, 0), (0, 1)];
    for (graine, t, indice, premiers, empreinte, rayon) in cas {
        assert_eq!(Alea::new(graine).suivant_f64(), t, "la graine fabriquée {graine} donne t = {t} exactement");
        let mut c = Claquage::new(graine, 6, 0);
        assert_eq!(
            c.ajouter(),
            candidates[indice],
            "t = {t}, eta 0 : u = {} est une égalité avec une somme cumulée",
            4.0 * t
        );
        c.croitre_jusqu_a(12);
        assert_eq!(&c.sites()[..6], &premiers, "t = {t}, eta 0 : les six premiers sites");
        assert_eq!(c.empreinte(), empreinte, "t = {t}, eta 0 : empreinte à 12 sites");
        assert_eq!(c.rayon(), rayon, "t = {t}, eta 0 : rayon à 12 sites");
    }
    let mut c = Claquage::new(8_335_436_391_877_801_038, 6, 1);
    c.croitre_jusqu_a(12);
    assert_eq!(&c.sites()[..6], &[(0, 0), (1, 0), (1, -1), (0, -1), (0, -2), (0, 1)]);
    assert_eq!(c.empreinte(), "35f2e51e113930a54164e76870fca11e15ea89184de8648183588021cf506351");
}

/// juste : `η = 0` n'a ni champ ni dépendance au potentiel (V-D1-05). Le simulateur naïf de `η = 0` n'a pas de relaxation (voir
/// `amas_egal_au_simulateur_naif`) ; ici : `potentiel` rend `None` pour tout site, sur l'amas, hors du domaine ou libre ; et deux
/// domaines de `demi_cote` 30 et 45 donnent EXACTEMENT les mêmes 150 sites pour 6 graines (un amas d'Eden de 150 sites a un rayon
/// d'une dizaine : il n'approche pas la frontière, et le `demi_cote` ne change rien d'autre à `η = 0`, où `ω` n'est pas employé).
/// À l'inverse, `η = 1` n'est pas indépendant du domaine : sur 6 graines et 120 sites, deux domaines de `demi_cote` 30 et 45 donnent au
/// moins un amas différent (le champ dépend de la frontière ; mesuré : 6 amas différents sur 6, le premier site différent étant le 12e au plus tôt ; un code qui ignorerait la
/// frontière ou le `demi_cote` les verrait identiques).
///
/// Ne couvre pas : la forme de l'amas (voir `etape_d1_dimensions`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn eta_zero_sans_champ_et_independant_du_domaine() {
    for graine in 1..=6_u64 {
        let mut petit = Claquage::new(graine, 30, 0);
        let mut grand = Claquage::new(graine, 45, 0);
        petit.croitre_jusqu_a(150);
        grand.croitre_jusqu_a(150);
        assert_eq!(petit.sites(), grand.sites(), "eta 0, graine {graine} : le demi_cote ne change pas l'amas");
        assert!(petit.rayon() < 25, "un amas d'Eden de 150 sites est loin de la frontière");
        for (x, y) in [(0, 0), (1, 0), (5, 5), (100, 100), (i32::MAX, i32::MIN)] {
            assert_eq!(petit.potentiel(x, y), None, "eta 0 : aucun champ en ({x}, {y})");
        }
    }
    let mut differents = 0;
    for graine in 1..=6_u64 {
        let mut petit = Claquage::new(graine, 30, 1);
        let mut grand = Claquage::new(graine, 45, 1);
        petit.croitre_jusqu_a(120);
        grand.croitre_jusqu_a(120);
        differents += usize::from(petit.sites() != grand.sites());
    }
    assert!(differents >= 1, "eta 1 : deux domaines différents donnent le même amas pour les 6 graines");
}

/// juste : l'arrêt (V-D1-06). `demi_cote = 0` : le germe touche la frontière (ses voisins sont à distance 1 > 0), `essayer_ajouter` rend
/// `None` tout de suite, `ajouter` PANIQUE avec un message contenant « frontière » ; `demi_cote = 1` : le germe ne la touche pas, le
/// premier site ajouté (un des quatre voisins, à distance 1) la touche, et `essayer_ajouter` rend alors `None` : exactement DEUX sites,
/// pour 40 graines et `η` = 0, 1, 2. Pour `demi_cote` 2, 3 et 4 : un amas grandi jusqu'au `None` s'arrête sur le PREMIER site qui a
/// un voisin hors du domaine (`x² + y² > demi_cote²`, en entiers) : le dernier site touche, aucun des précédents ne touche ; tous les
/// sites sont dans le domaine, distincts, et chacun (sauf le germe) a un voisin par un côté parmi les sites précédents. Un `None` ne
/// change rien : l'écriture canonique (sites ET état du générateur) est la même avant et après, répétée.
///
/// Ne couvre pas : l'arrêt faute de candidates ou à `W = 0` (inatteignable en pratique).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn arret_a_la_frontiere() {
    for eta in 0..=2_u32 {
        let mut zero = Claquage::new(3, 0, eta);
        assert!(zero.touche_la_frontiere(), "demi_cote 0 : le germe touche la frontière");
        let avant = zero.ecriture_canonique();
        assert_eq!(zero.essayer_ajouter(), None);
        assert_eq!(zero.ecriture_canonique(), avant);
        let panique =
            std::panic::catch_unwind(move || Claquage::new(3, 0, eta).ajouter()).expect_err("ajouter devait paniquer");
        let message =
            panique.downcast_ref::<String>().cloned().or_else(|| panique.downcast_ref::<&str>().map(|s| s.to_string()));
        assert!(message.is_some_and(|m| m.contains("frontière")), "le message doit contenir « frontière »");

        for graine in 1..=40_u64 {
            let mut un = Claquage::new(graine, 1, eta);
            assert!(!un.touche_la_frontiere(), "demi_cote 1 : le germe ne touche pas la frontière");
            let site = un.essayer_ajouter().expect("un premier site");
            assert_eq!(site.0.abs() + site.1.abs(), 1, "un voisin du germe");
            assert!(un.touche_la_frontiere(), "demi_cote 1 : le premier site touche la frontière");
            let avant = un.ecriture_canonique();
            assert_eq!(un.essayer_ajouter(), None, "eta {eta}, graine {graine}");
            assert_eq!(un.essayer_ajouter(), None);
            assert_eq!(un.sites().len(), 2);
            assert_eq!(un.ecriture_canonique(), avant, "un None ne consomme aucun tirage");
        }
    }
    for d in 2..=4_i32 {
        for eta in 0..=2_u32 {
            for graine in 1..=15_u64 {
                let mut c = Claquage::new(graine, d as u32, eta);
                while c.essayer_ajouter().is_some() {}
                let sites = c.sites().to_vec();
                let touche = |&(x, y): &(i32, i32)| {
                    [(1, 0), (0, 1), (-1, 0), (0, -1)]
                        .iter()
                        .any(|&(dx, dy)| (x + dx) * (x + dx) + (y + dy) * (y + dy) > d * d)
                };
                assert!(
                    touche(sites.last().unwrap()),
                    "d {d}, eta {eta}, graine {graine} : le dernier site touche la frontière"
                );
                assert!(
                    !sites[..sites.len() - 1].iter().any(touche),
                    "d {d}, eta {eta}, graine {graine} : un site avant le dernier touche"
                );
                assert!(c.touche_la_frontiere());
                for (k, &(x, y)) in sites.iter().enumerate() {
                    assert!(x * x + y * y <= d * d, "site dans le domaine");
                    assert!(!sites[..k].contains(&(x, y)), "sites distincts");
                    if k > 0 {
                        assert!(
                            sites[..k].iter().any(|&(a, b)| (a - x).abs() + (b - y).abs() == 1),
                            "d {d}, eta {eta}, graine {graine} : le site {k} {:?} n'a pas de voisin par un côté parmi les précédents",
                            (x, y)
                        );
                    }
                }
                let avant = c.ecriture_canonique();
                assert_eq!(c.essayer_ajouter(), None);
                assert_eq!(c.ecriture_canonique(), avant);
            }
        }
    }
}

/// juste : les bornes (V-D1-07). `ETA_MAX` vaut 8, `DEMI_COTE_MAX` 255, `PARTICULES_MAX` 100 000, `PASSES_MAX` 100 000 (la garde de terminaison de la relaxation), `SEUIL` exactement `2^-20` (les bits de
/// `f64`) ; `Claquage::new` PANIQUE pour `eta = 9` et pour `demi_cote = 256`, et accepte `eta = 8` et `demi_cote = 255` (avec `eta = 0`, donc
/// sans relaxer ; le coût d'un champ à 255 est hors d'un test) ; `surrelaxation(d)` vaut `2 - 6 / (d + 6)` : exactement `1.0` pour 0, `1.5`
/// pour 6, `1.75` pour 18, `1.875` pour 42 et `1.9375` pour 90 (des valeurs exactes en `f64`), et la formule pour 60 et 255.
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn bornes_et_constantes() {
    assert_eq!(ETA_MAX, 8);
    assert_eq!(DEMI_COTE_MAX, 255);
    assert_eq!(PARTICULES_MAX, 100_000);
    assert_eq!(PASSES_MAX, 100_000);
    assert_eq!(SEUIL.to_bits(), (2.0_f64).powi(-20).to_bits());
    assert_eq!(SEUIL.to_bits(), SEUIL_DOC.to_bits());
    for (d, attendu) in [(0_u32, 1.0_f64), (6, 1.5), (18, 1.75), (42, 1.875), (90, 1.9375)] {
        assert_eq!(surrelaxation(d).to_bits(), attendu.to_bits(), "surrelaxation({d})");
    }
    for d in [1_u32, 30, 60, 100, 255] {
        assert_eq!(surrelaxation(d).to_bits(), omega_doc(d as i32).to_bits(), "surrelaxation({d}) est 2 - 6 / (d + 6)");
    }
    assert!(std::panic::catch_unwind(|| Claquage::new(1, 5, ETA_MAX + 1)).is_err());
    assert!(std::panic::catch_unwind(|| Claquage::new(1, DEMI_COTE_MAX + 1, 0)).is_err());
    assert!(std::panic::catch_unwind(|| Claquage::new(1, DEMI_COTE_MAX + 1, 1)).is_err());
    assert_eq!(Claquage::new(1, 5, ETA_MAX).eta(), 8);
    let grand = Claquage::new(1, DEMI_COTE_MAX, 0);
    assert_eq!(grand.demi_cote(), 255);
    assert_eq!(grand.sites(), [(0, 0)]);
    assert_eq!(grand.rayon(), 0);
}

/// juste : `croitre_jusqu_a(n)` ne fait rien quand l'amas a déjà `n` sites ou plus (l'écriture canonique ne change pas) ; elle donne le
/// même amas, les mêmes potentiels et le même générateur par morceaux que d'un seul coup : `croitre_jusqu_a(10)` puis `(25)` égale `(25)` ;
/// `contient` est vrai exactement sur les sites et faux, sans paniquer, très loin ; `ajouter()` rend le site ajouté, le dernier de `sites()`
/// ; deux amas de même graine ont les mêmes sites, deux graines différentes des empreintes différentes (V-D1-08).
///
/// Ne couvre pas : que deux graines donnent des amas « assez différents » (une propriété statistique).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn croissance_par_morceaux_et_proprietes() {
    for eta in [0_u32, 1, 3] {
        let mut a = Claquage::new(9, 16, eta);
        let germe = a.ecriture_canonique();
        a.croitre_jusqu_a(0);
        a.croitre_jusqu_a(1);
        assert_eq!(a.ecriture_canonique(), germe, "croitre_jusqu_a(0 ou 1) ne fait rien sur le germe");
        a.croitre_jusqu_a(10);
        let a_10 = a.ecriture_canonique();
        a.croitre_jusqu_a(10);
        a.croitre_jusqu_a(4);
        assert_eq!(a.ecriture_canonique(), a_10, "croitre_jusqu_a(n) ne fait rien si l'amas a déjà n sites ou plus");
        a.croitre_jusqu_a(25);
        let mut b = Claquage::new(9, 16, eta);
        b.croitre_jusqu_a(25);
        assert_eq!(a.sites(), b.sites());
        assert_eq!(a.ecriture_canonique(), b.ecriture_canonique());
        assert_eq!(a.empreinte(), b.empreinte());
        for y in -17..=17 {
            for x in -17..=17 {
                assert_eq!(
                    a.potentiel(x, y).map(f64::to_bits),
                    b.potentiel(x, y).map(f64::to_bits),
                    "potentiel de ({x}, {y})"
                );
                assert_eq!(a.contient(x, y), a.sites().contains(&(x, y)), "contient({x}, {y})");
            }
        }
        assert!(!a.contient(i32::MAX, i32::MIN) && !a.contient(i32::MIN, 0) && !a.contient(0, i32::MAX));
        let mut c = Claquage::new(9, 16, eta);
        for k in 1..25 {
            let site = c.ajouter();
            assert_eq!(c.sites().last(), Some(&site), "ajouter() rend le dernier site");
            assert_eq!(c.sites().len(), k + 1);
        }
        assert_eq!(c.sites(), a.sites());
        let mut vus = std::collections::BTreeSet::new();
        for graine in 1..=6_u64 {
            let mut e = Claquage::new(graine, 16, eta);
            e.croitre_jusqu_a(20);
            assert!(vus.insert(e.empreinte()), "la graine {graine} donne une empreinte déjà vue");
        }
    }
}

/// juste : l'écriture canonique, octet par octet (V-D1-09). Le germe seul, `demi_cote = 5`, `η = 1`, graine 7 : `GIVREDB1` (`47 49 56
/// 52 45 44 42 31`), `05 00 00 00`, `01 00 00 00` (`η`), `01 00 00 00` (un site), le germe `00 00 00 00 00 00 00 00`, puis l'état de
/// `Alea::new(7)` (`d1a0bdb386a42205 6d00000000000000`, celui de l'étape 0 et de l'étape 1), soit 44 octets ; avec `η = 0`, le même avec
/// `00 00 00 00` à la place de `η`. Puis les quatre premiers sites de la graine 1, `demi_cote = 9`, `η = 1` : `(0,0) (1,0) (0,1) (2,0)`
/// (`x` puis `y`, chacun `i32` petit-boutiste), l'état du générateur après le quatrième site `3dce35a13968f346 6d00000000000000`, 68
/// octets. Les empreintes (SHA-256) sont celles de Python `hashlib` sur ces octets. Une écriture grand-boutiste, `x` et `y` échangés, `η` ou
/// `demi_cote` oublié ou un état du générateur oublié échouent ici. L'empreinte est le SHA-256 de l'écriture.
///
/// Ne couvre pas : les amas de plus de quatre sites (voir `valeurs_du_code_independant`).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn ecriture_canonique_a_la_main() {
    let germe_1 = Claquage::new(7, 5, 1);
    let attendu = "47495652454442310500000001000000010000000000000000000000d1a0bdb386a422056d00000000000000";
    assert_eq!(hex(&germe_1.ecriture_canonique()), attendu);
    assert_eq!(germe_1.ecriture_canonique().len(), 44);
    assert_eq!(germe_1.empreinte(), "7720013b7a1839e97a25a7d9180cee0dda8b0ae42848921f8db06bf75a17a82b");
    let germe_0 = Claquage::new(7, 5, 0);
    let attendu = "47495652454442310500000000000000010000000000000000000000d1a0bdb386a422056d00000000000000";
    assert_eq!(hex(&germe_0.ecriture_canonique()), attendu);
    assert_eq!(germe_0.empreinte(), "30576b4a851d03092631f680522ab9611e22c714eb3f495ab022e84e0cf8727b");

    let mut quatre = Claquage::new(1, 9, 1);
    quatre.croitre_jusqu_a(4);
    assert_eq!(quatre.sites(), [(0, 0), (1, 0), (0, 1), (2, 0)]);
    let attendu = concat!(
        "4749565245444231", // GIVREDB1
        "09000000",         // demi_cote 9
        "01000000",         // eta 1
        "04000000",         // quatre sites
        "0000000000000000", // (0, 0)
        "0100000000000000", // (1, 0)
        "0000000001000000", // (0, 1)
        "0200000000000000", // (2, 0)
        "3dce35a13968f346", // l'état du générateur après le quatrième site
        "6d00000000000000", // l'incrément (54 << 1) | 1 = 109
    );
    assert_eq!(hex(&quatre.ecriture_canonique()), attendu);
    assert_eq!(quatre.ecriture_canonique().len(), 36 + 8 * 4);
    assert_eq!(quatre.empreinte(), "8d350a34c6513ab06ad8c81848b50c1a79aa3d53b4d5ad7cbcb2dae8f7f055e6");
    assert_eq!(sha256_hex(&quatre.ecriture_canonique()), quatre.empreinte());
}

/// juste : des valeurs calculées par le code Python indépendant (`d1_ref.py`, écrit d'après la doc seule, sans lire le code Rust ; V-D1-10) :
/// pour neuf (graine, `demi_cote`, `η`, `n`), `η` de 0 à 8, les douze premiers sites, le rayon et l'empreinte (SHA-256 de l'écriture canonique
/// par `hashlib`). C'est le recoupement du simulateur naïf (écrit ici, par la main qui a écrit la doc) par une autre implémentation de la
/// même doc, en Python : même sur-relaxation, mêmes bits.
///
/// Ne couvre pas : les très grands amas (le Python est lent).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn valeurs_du_code_independant() {
    #[allow(clippy::type_complexity)]
    let cas: [(u64, u32, u32, usize, [(i32, i32); 12], u32, &str); 9] = [
        (
            1,
            16,
            1,
            40,
            [(0, 0), (1, 0), (0, 1), (2, 0), (-1, 1), (1, -1), (3, 0), (-2, 1), (-2, 2), (-2, 3), (-1, 0), (4, 0)],
            8,
            "bf5a6d997d89e54fd83d031a65ea6491cce877e8e7844bea03ba24b67b9161f4",
        ),
        (
            2,
            16,
            2,
            40,
            [
                (0, 0),
                (-1, 0),
                (-1, -1),
                (-2, 0),
                (-2, -1),
                (-3, 0),
                (-2, -2),
                (-1, 1),
                (-2, -3),
                (-2, -4),
                (-2, -5),
                (-2, -6),
            ],
            12,
            "13ee509ce8d93734b1884967c4738758d3a31d63f17a97bd914feb366da582d7",
        ),
        (
            3,
            18,
            3,
            40,
            [(0, 0), (0, 1), (0, -1), (-1, 0), (1, -1), (1, -2), (0, 2), (0, 3), (1, 3), (0, -2), (1, 4), (1, 5)],
            13,
            "67f2379ec4643a52f847bd5030406ab7c370e28e8109b19a2cdbfa3e5afcd27b",
        ),
        (
            4,
            16,
            0,
            60,
            [(0, 0), (0, -1), (-1, 0), (-2, 0), (0, 1), (-1, 1), (-3, 0), (-2, 1), (-2, 2), (-3, 1), (0, -2), (-1, -1)],
            10,
            "acba3e552e063e6e2fc96dd97e6cfb541ade2ada22cb4bb66a7dc850fc374328",
        ),
        (
            5,
            16,
            4,
            30,
            [(0, 0), (1, 0), (0, 1), (-1, 1), (-1, 2), (1, 1), (-2, 2), (-3, 2), (-2, 3), (-4, 2), (1, 2), (-2, 4)],
            11,
            "08aaf543e22c992e9b2bb6d71fe826d6ffe68f37883561cc5acc3a0bdd2f50f1",
        ),
        (
            6,
            24,
            8,
            25,
            [(0, 0), (1, 0), (2, 0), (3, 0), (-1, 0), (4, 0), (-2, 0), (5, 0), (6, 0), (-2, -1), (7, 0), (8, 0)],
            17,
            "f61337d038b9a1b54de44fabe24e0c6ea821698e836050a581dd5699ca102481",
        ),
        (
            7,
            20,
            1,
            12,
            [(0, 0), (1, 0), (0, -1), (0, 1), (1, 1), (-1, 1), (0, -2), (-1, 2), (2, 0), (1, -2), (-1, -1), (1, 2)],
            3,
            "a27a53864e2db6c56850f317b7928ece69130821690fb9150ef51c46f7331ddc",
        ),
        (
            8,
            20,
            2,
            12,
            [(0, 0), (1, 0), (1, 1), (1, -1), (2, 1), (2, 2), (1, -2), (2, -1), (3, 1), (3, -1), (-1, 0), (2, 3)],
            4,
            "bef1cf6650dde03ded04dadb9336d893110ad642ee61cccc1c7e65b974852dca",
        ),
        (
            9,
            20,
            0,
            12,
            [(0, 0), (-1, 0), (-1, -1), (-1, 1), (0, 1), (1, 1), (0, -1), (1, 0), (2, 0), (0, 2), (-2, 0), (2, 1)],
            3,
            "ba6b8b5b2bb665fd025ade4941552fdc86290a8f114f398fa769fa49a2bda8ec",
        ),
    ];
    for (graine, d, eta, n, premiers, rayon, empreinte) in cas {
        let mut c = Claquage::new(graine, d, eta);
        c.croitre_jusqu_a(n);
        assert_eq!(&c.sites()[..12], &premiers, "graine {graine}, d {d}, eta {eta} : douze premiers sites");
        assert_eq!(c.rayon(), rayon, "graine {graine}, d {d}, eta {eta} : rayon");
        assert_eq!(c.empreinte(), empreinte, "graine {graine}, d {d}, eta {eta}, {n} sites : empreinte");
    }
}

/// juste : le total `W` est sommé DANS L'ORDRE CANONIQUE, de gauche à droite (V-D1-32 ; ajouté par le juge). Au germe seul et à `η >= 1`, les quatre
/// poids ne sont égaux qu'à un arrondi près (l'ordre d'addition des voisins) ; pour les `(demi_cote, η)` ci-dessous, la somme faite à l'envers
/// (`w_3 + w_2 + w_1 + w_0`) diffère de `W` d'un ulp, et le premier tirage EXACT `t = 0,25`, `0,5` ou `0,75` (les graines fabriquées de
/// `egalites_du_tirage`) tombe alors d'un côté ou de l'autre d'une somme cumulée : le premier site change (mesuré par le juge : la somme à
/// l'envers choisit l'autre voisin dans les douze cas, ou ne trouve aucune candidate). Les sites attendus sont ceux du code Python
/// indépendant (`d1_ref.py premiers <graine> <d> <eta> 3`) et du simulateur naïf. C'est une vérification des bits de la doc (comme le cas
/// `η = 1` de `egalites_du_tirage`), pas une propriété qui survivrait à un autre champ.
///
/// Ne couvre pas : un ordre de somme faux qui ne changerait `W` d'aucun ulp sur ces douze états (une somme par paires, par exemple, est vue ici
/// seulement si elle change un de ces totaux).
#[test]
#[cfg_attr(not(feature = "etape-d1"), ignore)]
fn somme_des_poids_dans_l_ordre_canonique() {
    const T_QUART: u64 = 12_457_697_844_934_659_938;
    const T_DEMI: u64 = 8_335_436_391_877_801_038;
    const T_TROIS_QUARTS: u64 = 8_760_127_956_802_293_904;
    #[allow(clippy::type_complexity)]
    let cas: [(u64, u32, u32, [(i32, i32); 2]); 12] = [
        (T_DEMI, 4, 1, [(-1, 0), (0, -1)]),
        (T_DEMI, 4, 2, [(1, 0), (1, -1)]),
        (T_DEMI, 12, 1, [(1, 0), (1, -1)]),
        (T_DEMI, 5, 1, [(1, 0), (1, -1)]),
        (T_TROIS_QUARTS, 4, 2, [(0, 1), (-1, 1)]),
        (T_TROIS_QUARTS, 11, 1, [(0, 1), (-1, 1)]),
        (T_TROIS_QUARTS, 12, 1, [(0, 1), (-1, 1)]),
        (T_TROIS_QUARTS, 3, 3, [(0, 1), (-1, 1)]),
        (T_QUART, 4, 1, [(0, -1), (-1, 0)]),
        (T_QUART, 11, 1, [(-1, 0), (1, 0)]),
        (T_QUART, 2, 3, [(0, -1), (-1, 0)]),
        (T_QUART, 8, 2, [(0, -1), (1, 0)]),
    ];
    for (graine, d, eta, attendus) in cas {
        let mut c = Claquage::new(graine, d, eta);
        let mut naif = Naif::new(graine, d as i32, eta);
        c.croitre_jusqu_a(3);
        assert!(naif.croitre(3));
        assert_eq!(&c.sites()[1..], &attendus, "graine {graine}, d {d}, eta {eta} : les deux premiers sites");
        assert_eq!(c.sites(), &naif.sites[..], "graine {graine}, d {d}, eta {eta} : comme le simulateur naïf");
    }
}

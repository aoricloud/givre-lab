//! Étape V7 : la vitrine côté natif, l'image nette des modèles sur la grille hexagonale (les flocons de Reiter, modèles 1 à 3) par les quatre fonctions `givre_nette_*`. Valeurs attendues :
//! `references/etape-v7-valeurs.md`, identifiants `V-V7-xx`.
//!
//! Comme `etape_d3_vitrine.rs`, on appelle les fonctions `givre_*` comme des fonctions Rust et on les compare à ce que la bibliothèque donne **seule** : un `Flocon` construit en face, son champ
//! **recalculé à la main** d'après la doc (`etape_v7_commun::champ_de_la_doc`, jamais lu dans `vitrine`) et `rendu_hex::rendre` appliqué à ce champ. Le module WebAssembly lui-même est éprouvé dans
//! `etape_v7_module.rs` (sous Node), la page dans `etape_v7_page.rs`, un calcul exact à la main et une empreinte de référence dans `etape_v7_rendu.rs`.
//!
//! L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle : chaque test prend le verrou `VERROU`, propre à ce fichier, et le garde jusqu'à la fin. Aucun test de ce
//! fichier n'a de tolérance : tout est une égalité exacte (entiers, octets).
//!
//! Tant qu'une des quatre fonctions `givre_nette_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test **s'arrête d'un coup** au lieu de marquer un test en échec : une panique ne
//! peut pas traverser une `extern "C" fn` (le message est « panicked at src/vitrine.rs:<ligne> : not yet implemented », puis « panic in a function that cannot unwind » et l'abandon du processus :
//! `SIGABRT` sous Linux, `0xc0000409` sous Windows). C'est un échec, et la ligne citée dit quelle fonction reste à écrire ; les autres fichiers de test continuent avec `--no-fail-fast`.

mod etape_v7_commun;

use std::sync::{Mutex, MutexGuard};

use etape_v7_commun::{
    COTE_MAX, GRILLE_MAX, HORS, MODELES_NETS, SUR_MAX, champ_du_modele, d3, e5, image_attendue, image_du_champ,
    pixel_de,
};
use givre_lab::empreinte::sha256_hex;
use givre_lab::flocon::Parametres;
use givre_lab::vitrine::{
    NETTE_COTE_MAX, NETTE_GRILLE_MAX, NETTE_SUR_MAX, givre_avancer, givre_compacite, givre_demarrer, givre_duree,
    givre_empreinte_mot, givre_hauteur, givre_largeur, givre_mesurer, givre_nette_hauteur, givre_nette_largeur,
    givre_nette_pixel, givre_nette_preparer, givre_pixel, givre_rayon, givre_sites, givre_temps,
};

static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou de la vitrine ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les octets R, V, B du tampon de l'image nette, lus par `givre_nette_pixel` : `(largeur, hauteur, octets)`. Aucun pixel du tampon n'est `u32::MAX`.
fn lire_le_tampon(contexte: &str) -> (u32, u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_nette_largeur(), givre_nette_hauteur());
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_nette_pixel(x, y);
            assert_ne!(couleur, HORS, "{contexte} : le pixel ({x}, {y}) du tampon est hors image");
            assert!(
                couleur <= 0x00FF_FFFF,
                "{contexte} : le pixel ({x}, {y}) n'est pas empaqueté en 0x00RRGGBB : {couleur:#x}"
            );
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, hauteur, rvb)
}

/// Le tampon est vide : largeur et hauteur 0, et tout pixel vaut `u32::MAX` (V-V7-06).
fn exiger_le_tampon_vide(contexte: &str) {
    assert_eq!(givre_nette_largeur(), 0, "{contexte} : largeur du tampon");
    assert_eq!(givre_nette_hauteur(), 0, "{contexte} : hauteur du tampon");
    for (x, y) in [(0, 0), (1, 0), (0, 1), (10, 10), (61, 61), (u32::MAX, u32::MAX)] {
        assert_eq!(givre_nette_pixel(x, y), HORS, "{contexte} : le pixel ({x}, {y}) d'un tampon vide");
    }
}

/// Exige que le tampon soit l'image nette de `cote` pixels : `attendu` (les octets de `rendre`), dimensions comprises, et que les bords soient hors image.
fn exiger_le_tampon(attendu: &[u8], cote: u32, contexte: &str) {
    let (largeur, hauteur, rvb) = lire_le_tampon(contexte);
    assert_eq!((largeur, hauteur), (cote, cote), "{contexte} : dimensions");
    assert!(rvb == attendu, "{contexte} : l'image n'est pas celle de rendu_hex::rendre appliqué au champ de la doc");
    assert_eq!(givre_nette_pixel(cote, 0), HORS, "{contexte} : x = largeur est hors image");
    assert_eq!(givre_nette_pixel(0, cote), HORS, "{contexte} : y = hauteur est hors image");
    assert_eq!(givre_nette_pixel(cote, cote), HORS, "{contexte} : (largeur, hauteur) est hors image");
    assert_eq!(
        givre_nette_pixel(cote - 1, cote - 1),
        pixel_de(attendu, cote, cote - 1, cote - 1),
        "{contexte} : le dernier pixel"
    );
}

/// Les huit mots de l'empreinte de la vitrine, chacun sur 8 chiffres hexadécimaux, dans l'ordre.
fn empreinte_de_la_vitrine() -> String {
    (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect()
}

/// Les octets R, V, B de l'image d'un pixel par case (`givre_pixel`), lus un par un.
fn image_d_un_pixel_par_case() -> (u32, u32, Vec<u8>) {
    let (largeur, hauteur) = (givre_largeur(), givre_hauteur());
    let mut rvb = Vec::with_capacity(3 * largeur as usize * hauteur as usize);
    for y in 0..hauteur {
        for x in 0..largeur {
            let couleur = givre_pixel(x, y);
            rvb.extend_from_slice(&[(couleur >> 16) as u8, (couleur >> 8) as u8, couleur as u8]);
        }
    }
    (largeur, hauteur, rvb)
}

/// Exige que l'état de la vitrine soit celui du code natif pour le modèle 0 à 3 (`e5::natif`) : temps, durée, dimensions, **chaque pixel de l'image d'un pixel par case**, sites, rayon, compacité (bits),
/// les huit mots de l'empreinte et l'empreinte entière. C'est ce qu'une préparation d'image nette ne doit jamais changer.
fn exiger_l_etat_natif(modele: u32, graine: u32, pas: u32, contexte: &str) {
    let natif = e5::natif(modele, graine, pas);
    assert_eq!(givre_temps(), natif.temps, "{contexte} : temps");
    assert_eq!(givre_duree(), natif.duree, "{contexte} : durée");
    assert_eq!(givre_sites(), natif.sites, "{contexte} : sites");
    assert_eq!(givre_rayon(), natif.rayon, "{contexte} : rayon");
    match natif.compacite {
        None => assert!(givre_compacite().is_nan(), "{contexte} : compacité NaN"),
        Some(attendue) => assert_eq!(givre_compacite().to_bits(), attendue.to_bits(), "{contexte} : compacité"),
    }
    for (i, mot) in natif.mots().iter().enumerate() {
        assert_eq!(givre_empreinte_mot(i as u32), *mot, "{contexte} : mot {i} de l'empreinte");
    }
    assert_eq!(empreinte_de_la_vitrine(), natif.empreinte, "{contexte} : empreinte");
    let (largeur, hauteur, rvb) = image_d_un_pixel_par_case();
    assert_eq!(
        (largeur, hauteur),
        (natif.largeur, natif.hauteur),
        "{contexte} : dimensions de l'image d'un pixel par case"
    );
    assert!(
        rvb == natif.rvb,
        "{contexte} : l'image d'un pixel par case n'est plus celle de l'aperçu de la bibliothèque"
    );
}

/// Le code que la doc donne pour `(cote, sur)` quand le modèle en cours est (`hexagonal`) ou non sur la grille hexagonale, un état étant démarré : les vérifications dans l'ordre 1, 3, 4, 5 (V-V7-04).
fn code_de_la_doc(hexagonal: bool, cote: u32, sur: u32) -> u32 {
    if !hexagonal {
        1
    } else if !(2..=2048).contains(&cote) || !cote.is_multiple_of(2) {
        3
    } else if !(1..=4).contains(&sur) {
        4
    } else if u64::from(cote) * u64::from(sur) > 4096 {
        5
    } else {
        0
    }
}

/// juste : l'image nette de chaque flocon (modèles 1 à 3) est **exactement** celle de `rendu_hex::rendre` appliquée au champ de la doc (V-V7-02), pour plusieurs états et plusieurs réglages.
/// Les états : le temps 0 (le germe seul), après 1, 37 et 400 pas pour la plaque ; 0, 5, 60 et 120 pour l'étoile ; 0, 150, 800 et 10 000 (qui dépasse la durée : 800 sont joués) pour la dendrite. Les
/// réglages : `(62, 2)`, `(96, 3)`, `(128, 1)` et `(2, 1)` (l'image de deux pixels). Pour chaque cas, `givre_nette_preparer` rend 0, la largeur et la hauteur valent `cote`, **chaque octet** du tampon est celui de
/// `rendre(R, champ, { cote, rayon_vue: R, sur, plein: 4 E })` (un champ lu dans un autre ordre de cases qui n'est pas une symétrie de l'hexagone, une valeur de `sur` ignorée, un pixel `(x, y)` lu en `(y, x)`,
/// un champ qui n'est pas celui de la doc, ou l'image d'un pixel par case rendue à la place se voient), et les pixels hors du tampon valent `u32::MAX`. Le tampon est relu une seconde fois : même image (une lecture ne recalcule rien).
///
/// Ne couvre pas : les grandes images (voir `l_image_d_une_page_est_celle_de_la_doc`) ; un champ lu dans un autre ordre qui est une symétrie de l'hexagone (la symétrie d'ordre 6 et le miroir du flocon sont exacts : l'image est la même, cette faute-là ne
/// change rien) ; une largeur et une hauteur échangées (l'image est carrée).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn l_image_nette_est_celle_de_rendre_pour_le_champ_de_la_doc() {
    let _verrou = verrou();
    for (indice, modele) in MODELES_NETS.iter().enumerate() {
        let pas_a_jouer: [u32; 4] = match indice {
            0 => [0, 1, 37, 400],
            1 => [0, 5, 60, 120],
            _ => [0, 150, 800, 10_000],
        };
        for pas in pas_a_jouer {
            assert_eq!(givre_demarrer(modele.numero, 1), 1, "modèle {}", modele.numero);
            assert_eq!(givre_duree(), modele.duree);
            givre_avancer(pas);
            let champ = champ_du_modele(indice, pas);
            for (cote, sur) in [(62_u32, 2_u32), (96, 3), (128, 1), (2, 1)] {
                let contexte = format!("modèle {}, {pas} pas, côté {cote}, sur {sur}", modele.numero);
                let attendue = image_du_champ(indice, &champ, cote, sur);
                assert_eq!(givre_nette_preparer(cote, sur), 0, "{contexte}");
                exiger_le_tampon(&attendue, cote, &contexte);
                // Une lecture ne recalcule rien : le tampon est relu, identique.
                exiger_le_tampon(&attendue, cote, &format!("{contexte}, relu"));
            }
        }
    }
}

/// juste : les images de la page (V-V7-02), `(384, 2)` pour l'animation et `(1024, 3)` pour la fin du scénario (les constantes de `docs/app.js`), sont celles de la doc : le modèle 2 jusqu'au bout et le
/// modèle 3 jusqu'au bout. L'image de la fin a `1024 × 1024` pixels et `3072²` échantillons (9,4 millions) : chaque octet du tampon est celui de `rendre` appliqué au champ de la doc.
///
/// Ne couvre pas : les autres états et réglages (voir `l_image_nette_est_celle_de_rendre_pour_le_champ_de_la_doc`) ; le temps que cela prend (voir le rapport de l'étape).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn l_image_d_une_page_est_celle_de_la_doc() {
    let _verrou = verrou();
    assert_eq!(givre_demarrer(2, 0), 1);
    givre_avancer(120);
    assert_eq!(givre_nette_preparer(384, 2), 0);
    exiger_le_tampon(&image_attendue(1, 120, 384, 2), 384, "modèle 2, animation (384, 2)");
    assert_eq!(givre_demarrer(3, 0), 1);
    givre_avancer(800);
    assert_eq!(givre_nette_preparer(1024, 3), 0);
    exiger_le_tampon(&image_attendue(2, 800, 1024, 3), 1024, "modèle 3, fin (1024, 3)");
}

/// juste : les bornes de la doc (V-V7-03), exactes. Les constantes publiques valent 2048, 4 et 4096 ; `(2048, 2)` et `(1024, 4)` (le côté de la grille d'échantillons est 4096, **inclus**) rendent 0 et l'image de
/// `rendre` ; `(2, 1)` rend 0 ; refusés : `cote` impair (1, 3, 2047, 61) ou inférieur à 2 (0), ou supérieur à 2048 (2049, 2050, 4096, `u32::MAX`) : le code 3 ; `sur` 0, 5, 16 ou `u32::MAX` : le code 4 ; `(2048, 3)`,
/// `(1366, 3)` (4098 > 4096), `(2048, 4)` et `(1026, 4)` : le code 5. L'ordre des vérifications est celui de la doc : le côté avant le suréchantillonnage (`(2050, 5)` : 3, `(0, 0)` : 3), le suréchantillonnage avant
/// le nombre d'échantillons (`(2048, 5)` : 4, `(1000, 5)` : 4 et non 5).
///
/// Ne couvre pas : les images de taille ordinaire (voir `l_image_nette_est_celle_de_rendre_pour_le_champ_de_la_doc`) ; les codes 1, 2 et 6 (voir `les_modeles_carres_n_ont_pas_d_image_nette` et, sur un module neuf, `etape_v7_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn les_bornes_et_les_codes_d_erreur() {
    let _verrou = verrou();
    assert_eq!((NETTE_COTE_MAX, NETTE_SUR_MAX, NETTE_GRILLE_MAX), (COTE_MAX, SUR_MAX, GRILLE_MAX));
    assert_eq!((NETTE_COTE_MAX, NETTE_SUR_MAX, NETTE_GRILLE_MAX), (2048, 4, 4096));
    assert_eq!(givre_demarrer(2, 0), 1);
    givre_avancer(10);

    // Les bornes incluses : chacune rend 0 et l'image de `rendre`.
    for (cote, sur) in [(2_u32, 1_u32), (2048, 2), (1024, 4)] {
        assert_eq!(givre_nette_preparer(cote, sur), 0, "({cote}, {sur}) est permis");
        exiger_le_tampon(&image_attendue(1, 10, cote, sur), cote, &format!("({cote}, {sur})"));
    }

    // Les refus, avec le code de la doc ; chacun efface le tampon d'une préparation réussie.
    let refus: [(u32, u32, u32); 25] = [
        (0, 2, 3),
        (1, 2, 3),
        (3, 2, 3),
        (61, 2, 3),
        (2047, 2, 3),
        (2049, 2, 3),
        (2050, 2, 3),
        (4096, 1, 3),
        (u32::MAX, 2, 3),
        (62, 0, 4),
        (62, 5, 4),
        (62, 16, 4),
        (62, u32::MAX, 4),
        (2048, 3, 5),
        (1366, 3, 5),
        (2048, 4, 5),
        (1026, 4, 5),
        (2050, 5, 3),
        (0, 0, 3),
        (3, 0, 3),
        (2048, 5, 4),
        (1000, 5, 4),
        (2048, u32::MAX, 4),
        (u32::MAX, u32::MAX, 3),
        (1, u32::MAX, 3),
    ];
    for (cote, sur, code) in refus {
        assert_eq!(givre_nette_preparer(62, 2), 0);
        assert_eq!(givre_nette_largeur(), 62);
        assert_eq!(givre_nette_preparer(cote, sur), code, "({cote}, {sur})");
        assert_eq!(code, code_de_la_doc(true, cote, sur), "({cote}, {sur}) : le tableau du test et la règle de la doc");
        exiger_le_tampon_vide(&format!("après le refus ({cote}, {sur})"));
    }
}

/// juste : jamais de panique pour un `cote` ou un `sur` absurde, quel que soit le modèle (V-V7-04). Pour chacun des huit modèles et chaque couple de `cote` dans `0, 1, 2, 3, 62, 2047, 2049, 4096, 65536, u32::MAX` et de `sur` dans
/// `0, 1, 2, 4, 5, 16, u32::MAX`, `givre_nette_preparer` rend le code de la doc (1 pour un modèle sur une grille carrée, sinon 0, 3, 4 ou 5 d'après les bornes) et **jamais** 6 ; un pixel absurde (`(u32::MAX, u32::MAX)`) rend `u32::MAX`.
/// Une panique dans une fonction `extern "C"` abat tout le binaire de test, ce qui est un échec.
///
/// Ne couvre pas : le contenu des images (voir les autres tests) ; l'absence de tout démarrage (module neuf : voir `etape_v7_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn jamais_de_panique_pour_un_cote_ou_un_sur_absurde() {
    let _verrou = verrou();
    for modele in 0..=7_u32 {
        assert_eq!(givre_demarrer(modele, 3), 1, "modèle {modele}");
        givre_avancer(5);
        let hexagonal = (1..=3).contains(&modele);
        for cote in [0_u32, 1, 2, 3, 62, 2047, 2049, 4096, 65_536, u32::MAX] {
            for sur in [0_u32, 1, 2, 4, 5, 16, u32::MAX] {
                let code = givre_nette_preparer(cote, sur);
                assert_ne!(code, 6, "modèle {modele}, ({cote}, {sur}) : le rendu ne refuse jamais avec ces bornes");
                assert_eq!(code, code_de_la_doc(hexagonal, cote, sur), "modèle {modele}, ({cote}, {sur})");
                assert_eq!(givre_nette_pixel(u32::MAX, u32::MAX), HORS);
                if code != 0 {
                    assert_eq!(
                        givre_nette_largeur(),
                        0,
                        "modèle {modele}, ({cote}, {sur}) : un refus laisse un tampon vide"
                    );
                }
            }
        }
    }
}

/// juste : une préparation ne change **rien** à l'état (V-V7-05). Pour chaque flocon, après 37 pas (puis 300, puis jusqu'au bout), l'état de la vitrine est celui du code natif (`e5::natif` : temps, durée, sites, rayon, compacité, les
/// huit mots de l'empreinte et l'empreinte, et **chaque pixel de l'image d'un pixel par case**) avant et après des préparations (une réussie, deux refusées, une réussie de plus petit côté) ; et la suite du scénario, jouée après elles,
/// donne l'état natif de la durée entière (la dynamique ne sait rien des images). Une image d'un pixel par case écrasée par l'image nette, une empreinte ou un temps qui bouge se voient. Et dans l'autre sens : **lire l'état n'efface pas le tampon**
/// (après toutes ces lectures et un `givre_mesurer`, qui rend 0 pour un flocon, le tampon est encore l'image `(64, 2)` de cet état : seuls `givre_demarrer`, un `givre_avancer` qui joue et un refus l'effacent).
///
/// Ne couvre pas : le contenu de l'image nette (voir `l_image_nette_est_celle_de_rendre_pour_le_champ_de_la_doc`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn une_preparation_ne_change_rien_a_l_etat() {
    let _verrou = verrou();
    for modele in &MODELES_NETS {
        let numero = modele.numero;
        assert_eq!(givre_demarrer(numero, 9), 1);
        let mut joue = 0_u32;
        for jusqu_a in [37_u32, 100, modele.duree] {
            givre_avancer(jusqu_a - joue);
            joue = jusqu_a;
            exiger_l_etat_natif(numero, 9, joue, &format!("modèle {numero}, {joue} pas, avant la préparation"));
            assert_eq!(givre_nette_preparer(96, 3), 0);
            assert_eq!(givre_nette_preparer(97, 3), 3);
            assert_eq!(givre_nette_preparer(96, 9), 4);
            assert_eq!(givre_nette_preparer(64, 2), 0);
            assert_eq!(givre_nette_largeur(), 64);
            exiger_l_etat_natif(numero, 9, joue, &format!("modèle {numero}, {joue} pas, après la préparation"));
            // Lire l'état (et demander une mesure : 0 pour un flocon) n'efface pas le tampon : seuls `givre_demarrer`, `givre_avancer` qui joue et un refus l'effacent.
            assert_eq!(givre_mesurer(), 0, "un flocon n'a pas de mesure");
            exiger_le_tampon(
                &image_attendue(numero as usize - 1, joue, 64, 2),
                64,
                &format!("modèle {numero}, {joue} pas : le tampon après les lectures de l'état"),
            );
        }
        assert_eq!(givre_demarrer(numero, 9), 1);
        givre_avancer(20);
        assert_eq!(givre_nette_preparer(62, 2), 0);
        givre_avancer(modele.duree);
        exiger_l_etat_natif(
            numero,
            9,
            modele.duree,
            &format!("modèle {numero}, jusqu'au bout après une préparation au pas 20"),
        );
    }
}

/// juste : le tampon appartient à l'état qu'il montre (V-V7-06). Après `givre_demarrer` et des pas, **avant toute préparation**, le tampon est vide ; après `(62, 2)` il a `62 × 62` pixels ; `givre_avancer(0)` le laisse ;
/// `givre_avancer(1)` l'efface (et joue une unité) ; un `givre_demarrer` refusé (le modèle 9) le laisse ; un `givre_demarrer` qui réussit (même modèle) l'efface ; une préparation de `(64, 1)` remplace celle de `(62, 2)`
/// (64 pixels de côté) ; **`(64, 3)` remplace `(64, 1)`** (le même côté, un autre suréchantillonnage : l'image est recalculée, à 60 pas où les deux diffèrent) ; un refus (côté impair) l'efface ; **un scénario fini** (le modèle 2 joué jusqu'à 120) garde son tampon quand `givre_avancer(1)` et `givre_avancer(100)` ne jouent rien (rendent 0) ; et deux préparations de
/// suite du même état donnent le même tampon. Une image d'avant un `givre_avancer` gardée à tort (la faute typique : un tampon jamais effacé), ou effacée à tort quand rien n'a changé, se voit.
///
/// Ne couvre pas : l'absence de tout démarrage (voir `etape_v7_module.rs`, sur un module neuf).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_tampon_appartient_a_l_etat_qu_il_montre() {
    let _verrou = verrou();
    assert_eq!(givre_demarrer(2, 0), 1);
    exiger_le_tampon_vide("après givre_demarrer, avant toute préparation");
    assert_eq!(givre_avancer(10), 10);
    exiger_le_tampon_vide("après des pas, avant toute préparation");

    assert_eq!(givre_nette_preparer(62, 2), 0);
    exiger_le_tampon(&image_attendue(1, 10, 62, 2), 62, "(62, 2) au pas 10");
    assert_eq!(givre_avancer(0), 0);
    exiger_le_tampon(&image_attendue(1, 10, 62, 2), 62, "après givre_avancer(0)");
    assert_eq!(givre_avancer(1), 1);
    exiger_le_tampon_vide("après givre_avancer(1), qui joue une unité");

    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(givre_demarrer(9, 0), 0, "le modèle 9 n'existe pas");
    exiger_le_tampon(&image_attendue(1, 11, 62, 2), 62, "après un givre_demarrer refusé : l'état n'a pas changé");
    assert_eq!(givre_demarrer(2, 0), 1);
    exiger_le_tampon_vide("après un givre_demarrer qui réussit");

    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(givre_nette_preparer(64, 1), 0);
    exiger_le_tampon(&image_attendue(1, 0, 64, 1), 64, "(64, 1) remplace (62, 2)");
    // Le même côté, un autre suréchantillonnage : l'image est recalculée (un tampon « déjà de la bonne taille » rendu tel quel est une faute).
    assert_eq!(givre_avancer(60), 60);
    assert_eq!(givre_nette_preparer(64, 1), 0);
    assert_eq!(givre_nette_preparer(64, 3), 0);
    let (fine, grossiere) = (image_attendue(1, 60, 64, 3), image_attendue(1, 60, 64, 1));
    assert!(fine != grossiere, "les images (64, 3) et (64, 1) de l'étoile à 60 pas diffèrent");
    exiger_le_tampon(&fine, 64, "(64, 3) remplace (64, 1) : même côté, autre suréchantillonnage");
    assert_eq!(givre_demarrer(2, 0), 1);
    assert_eq!(givre_nette_preparer(63, 1), 3);
    exiger_le_tampon_vide("après un refus");

    // Un scénario fini : `givre_avancer` ne joue plus rien, le tampon reste.
    assert_eq!(givre_avancer(1000), 120);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(givre_avancer(1), 0);
    exiger_le_tampon(&image_attendue(1, 120, 62, 2), 62, "scénario fini, après givre_avancer(1)");
    assert_eq!(givre_avancer(100), 0);
    exiger_le_tampon(&image_attendue(1, 120, 62, 2), 62, "scénario fini, après givre_avancer(100)");
    assert_eq!(givre_nette_preparer(62, 2), 0);
    exiger_le_tampon(&image_attendue(1, 120, 62, 2), 62, "deux préparations de suite");
}

/// juste : un modèle sur une grille carrée n'a pas d'image nette (V-V7-07). Pour la DLA (modèle 0) et le claquage (modèles 4 à 7), `givre_nette_preparer` rend 1 — pour des réglages ordinaires `(62, 2)` comme pour des
/// réglages absurdes (`(0, 0)`, `(u32::MAX, u32::MAX)` : le modèle est vérifié avant les réglages) —, le tampon reste vide, et **l'image d'un pixel par case reste celle de la bibliothèque** (la DLA après 60 particules : chaque
/// pixel de `amas_colore`, l'état et l'empreinte ; chaque claquage après 30 sites : chaque pixel). La DLA est démarrée **après un flocon dont l'image nette est prête** : son tampon est vide dès le démarrage (jamais l'image d'un autre modèle).
/// Un modèle carré qui rend une image, qui garde celle du flocon d'avant, ou qui change ce qu'il montre, se voit.
///
/// Ne couvre pas : le claquage jusqu'au bout (voir `etape_d3_vitrine.rs`) ; la DLA à d'autres graines.
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn les_modeles_carres_n_ont_pas_d_image_nette() {
    let _verrou = verrou();
    // Un flocon préparé d'abord : la DLA démarrée ensuite ne garde pas son image.
    assert_eq!(givre_demarrer(2, 0), 1);
    assert_eq!(givre_nette_preparer(62, 2), 0);
    assert_eq!(givre_nette_largeur(), 62);
    assert_eq!(givre_demarrer(0, 4), 1);
    exiger_le_tampon_vide("la DLA démarrée après un flocon préparé");
    givre_avancer(60);
    for (cote, sur) in [(62_u32, 2_u32), (0, 0), (u32::MAX, u32::MAX), (3, 9)] {
        assert_eq!(givre_nette_preparer(cote, sur), 1, "DLA, ({cote}, {sur})");
        exiger_le_tampon_vide("DLA");
    }
    let natif = e5::natif(0, 4, 60);
    assert_eq!(givre_temps(), natif.temps);
    assert_eq!(empreinte_de_la_vitrine(), natif.empreinte);
    let (largeur, hauteur, rvb) = image_d_un_pixel_par_case();
    assert_eq!((largeur, hauteur), (natif.largeur, natif.hauteur));
    assert!(rvb == natif.rvb, "l'image d'un pixel par case de la DLA a changé");

    for modele in 4..=7_u32 {
        assert_eq!(givre_demarrer(modele, 5), 1);
        givre_avancer(30);
        for (cote, sur) in [(62_u32, 2_u32), (0, 0), (u32::MAX, u32::MAX)] {
            assert_eq!(givre_nette_preparer(cote, sur), 1, "modèle {modele}, ({cote}, {sur})");
            exiger_le_tampon_vide(&format!("modèle {modele}"));
        }
        let natif = d3::natif_claquage(modele - 4, 5, 30);
        assert_eq!(givre_temps(), natif.temps);
        assert_eq!(empreinte_de_la_vitrine(), natif.empreinte);
        let (largeur, hauteur, rvb) = image_d_un_pixel_par_case();
        assert_eq!((largeur, hauteur), (natif.largeur, natif.hauteur));
        assert!(rvb == natif.rvb, "l'image d'un pixel par case du modèle {modele} a changé");
    }
}

/// juste : les trois modèles de flocon ont trois images nettes différentes, et le dernier démarré est celui qui est dessiné (V-V7-08). Pour les modèles 1, 2 et 3 au temps 0 et à la fin, `(96, 3)` donne l'image de `rendre`
/// pour le champ de **ce** modèle (le rayon de l'hexagone 30, 40 ou 60, le `plein` `4 E` de ce modèle) ; les trois empreintes SHA-256 des images finales sont deux à deux différentes. Un modèle dessiné avec le rayon, le
/// `E` ou le champ d'un autre se voit. Les valeurs de `E` de la doc sont `γ.brut × durée` (`214 748 365 × 400`, `429 496 730 × 120`, `4 294 967 × 800`), recalculées ici d'après `Parametres::lire`.
///
/// Ne couvre pas : des graines (les flocons n'ont pas de hasard : la graine est ignorée).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn chaque_modele_est_dessine_avec_son_champ() {
    let _verrou = verrou();
    let mut finales = Vec::new();
    for (indice, modele) in MODELES_NETS.iter().enumerate() {
        let parametres = Parametres::lire(modele.alpha, modele.beta, modele.gamma).expect("réglages valides");
        assert_eq!(
            parametres.gamma.0 * i64::from(modele.duree),
            modele.e,
            "E du modèle {} d'après la doc",
            modele.numero
        );
        for pas in [0_u32, modele.duree] {
            assert_eq!(givre_demarrer(modele.numero, 123), 1);
            givre_avancer(pas);
            assert_eq!(givre_nette_preparer(96, 3), 0);
            exiger_le_tampon(&image_attendue(indice, pas, 96, 3), 96, &format!("modèle {}, {pas} pas", modele.numero));
        }
        let (_, _, rvb) = lire_le_tampon("fin");
        finales.push(sha256_hex(&rvb));
    }
    finales.sort();
    finales.dedup();
    assert_eq!(finales.len(), 3, "les trois images finales sont différentes");
}

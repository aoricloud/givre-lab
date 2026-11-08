//! Tests d'acceptation de l'étape 5 (a) : la vitrine est la simulation, côté natif.
//!
//! Les fonctions `givre_*` de `vitrine` sont celles que la page appelle à travers le module WebAssembly ; ici on les appelle
//! comme des fonctions Rust et on les compare, à chaque fois, à un état **natif** construit en face par la bibliothèque
//! (`dla::Amas`, `flocon::Flocon`, avec les réglages de la table de la doc de `vitrine`, voir `etape_5_commun`). Le module
//! WebAssembly lui-même est éprouvé dans `etape_5_artefact.rs`, la page dans `etape_5_page.rs`.
//!
//! L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle : chaque test prend d'abord le
//! verrou `VERROU`, propre à ce fichier, et le garde jusqu'à la fin.
//!
//! Ce que valent les fonctions avant tout `givre_demarrer` n'est pas testé ici (l'état global est déjà posé par les autres
//! tests du processus) : `etat-wasm.mjs` l'éprouve sur un module neuf (`etape_5_artefact.rs`).
//!
//! Valeurs attendues et sources : `references/etape-5-valeurs.md`, identifiants `V-VIT-xx` cités dans les tests. Aucun test de
//! ce fichier n'a de tolérance : tout est une égalité exacte (entiers, octets, mots d'empreinte, bits des `f64`). Ils
//! n'utilisent, parmi les fonctions de `f64`, que `to_bits`, `is_nan` et `is_finite`.

mod etape_5_commun;

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use etape_5_commun::{Natif, TABLE, natif, racine, reference};
use givre_lab::dla::{demi_cote_de_la_boite, particules};
use givre_lab::scenario::{Forme, lire_scenario};
use givre_lab::vitrine::{
    givre_avancer, givre_compacite, givre_demarrer, givre_duree, givre_empreinte_mot, givre_hauteur, givre_largeur,
    givre_pixel, givre_rayon, givre_sites, givre_temps,
};

/// Sérialise les tests de ce fichier : l'état de la vitrine est unique.
static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les huit mots de l'empreinte de la vitrine, chacun sur 8 chiffres hexadécimaux, dans l'ordre.
fn empreinte_de_la_vitrine() -> String {
    (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect()
}

/// Vérifie que la vitrine, dans son état courant, est identique à `attendu` sur tout ce qu'elle expose : le temps, la durée,
/// les dimensions de l'image, **chaque pixel** de l'image, les compteurs, la compacité (bit à bit, `NaN` pour la DLA), chacun
/// des huit mots de l'empreinte et l'empreinte entière.
fn verifier_identique(attendu: &Natif, contexte: &str) {
    assert_eq!(givre_temps(), attendu.temps, "{contexte} : temps");
    assert_eq!(givre_duree(), attendu.duree, "{contexte} : durée");
    assert_eq!(givre_largeur(), attendu.largeur, "{contexte} : largeur");
    assert_eq!(givre_hauteur(), attendu.hauteur, "{contexte} : hauteur");
    assert_eq!(givre_sites(), attendu.sites, "{contexte} : sites");
    assert_eq!(givre_rayon(), attendu.rayon, "{contexte} : rayon");
    match attendu.compacite {
        None => assert!(givre_compacite().is_nan(), "{contexte} : la compacité de la DLA est NaN"),
        Some(compacite) => {
            let obtenue = givre_compacite();
            assert_eq!(
                obtenue.to_bits(),
                compacite.to_bits(),
                "{contexte} : compacité {obtenue} au lieu de {compacite}"
            );
        }
    }
    for (i, mot) in attendu.mots().iter().enumerate() {
        assert_eq!(givre_empreinte_mot(i as u32), *mot, "{contexte} : mot {i} de l'empreinte");
    }
    assert_eq!(empreinte_de_la_vitrine(), attendu.empreinte, "{contexte} : empreinte");
    for y in 0..attendu.hauteur {
        for x in 0..attendu.largeur {
            assert_eq!(givre_pixel(x, y), attendu.pixel(x, y), "{contexte} : pixel ({x}, {y})");
        }
    }
}

/// Démarre `modele` avec `graine`, vérifie qu'il est au temps 0 comme l'état natif, l'avance de `pas` unités d'un seul appel
/// (que la durée permet : `givre_avancer` doit rendre `pas`) et vérifie qu'il redonne l'état natif.
fn verifier_modele(modele: u32, graine: u32, pas: u32) {
    assert_eq!(givre_demarrer(modele, graine), 1, "modèle {modele} : givre_demarrer");
    verifier_identique(&natif(modele, graine, 0), &format!("modèle {modele}, graine {graine}, temps 0"));
    assert_eq!(givre_avancer(pas), pas, "modèle {modele} : givre_avancer({pas})");
    verifier_identique(&natif(modele, graine, pas), &format!("modèle {modele}, graine {graine}, {pas} pas"));
}

/// Le nombre de pixels de chaque couleur de l'image courante.
fn couleurs_de_l_image() -> BTreeMap<u32, usize> {
    let mut nombres = BTreeMap::new();
    for y in 0..givre_hauteur() {
        for x in 0..givre_largeur() {
            *nombres.entry(givre_pixel(x, y)).or_insert(0) += 1;
        }
    }
    nombres
}

/// juste : la DLA de graine 1 jouée jusqu'au bout (2999 particules, 3000 sites : `scenarios/dla.txt`) est l'amas natif
/// `Amas::new(1, 200)` à 3000 sites, sur tout ce que la vitrine expose (V-VIT-02) : temps, durée, image pixel par pixel,
/// compteurs, mots de l'empreinte ; et son empreinte est celle du fichier `references/dla-graine-1.empreinte` (la référence
/// du dépôt, figée par l'étape 1 : une régression de notre code, ici recoupée avec le calcul natif). Le rayon est au moins 27
/// (calcul exact : les 3000 sites sont distincts et tiennent dans le disque de rayon `rayon`, qui a au plus `(2 rayon + 1)²`
/// sites entiers, donc `(2 rayon + 1)² >= 3000`).
///
/// Ne couvre pas : une graine autre que 1 à 3000 sites (voir `dla_autres_graines`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn dla_graine_1_jusqu_au_bout() {
    let _verrou = verrou();
    verifier_modele(0, 1, 2999);
    assert_eq!(givre_temps(), 2999);
    assert_eq!(givre_duree(), 2999);
    assert_eq!(givre_sites(), 3000);
    assert!(givre_rayon() >= 27, "rayon {}", givre_rayon());
    assert_eq!(empreinte_de_la_vitrine(), reference("dla-graine-1.empreinte"));
}

/// juste : d'autres graines (2, 0, une graine de 32 bits qui ne tient ni sur 16 bits ni sur 31, et la plus grande) à 400
/// particules sont, chacune, l'amas natif de cette graine, et leurs empreintes sont toutes différentes : la graine de 32 bits
/// est lue en entier (ni tronquée, ni lue comme un entier signé, ni ignorée).
///
/// Ne couvre pas : les durées longues (voir `dla_graine_1_jusqu_au_bout`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn dla_autres_graines() {
    let _verrou = verrou();
    let mut empreintes = Vec::new();
    for graine in [2, 0, 0xDEAD_BEEF, u32::MAX, 0x0001_0001, 0x8000_0000] {
        verifier_modele(0, graine, 400);
        empreintes.push(empreinte_de_la_vitrine());
    }
    let mut distinctes = empreintes.clone();
    distinctes.sort();
    distinctes.dedup();
    assert_eq!(distinctes.len(), empreintes.len(), "deux graines ont donné la même empreinte");
}

/// juste : la DLA à la main (V-VIT-04 et V-VIT-05). Au temps 0, le germe est seul : `m = 0`, `cote = 1`, `echelle = 512`,
/// donc une image de 512 × 512 pixels **tous** blancs (`0xFFFFFF`, `PALETTE[0]`), un site, un rayon 0. Après une particule
/// (quelle que soit la graine : elle colle à un des quatre voisins du germe), `m = 1`, `cote = 3`, `echelle = 170`, donc une
/// image de 510 × 510 pixels (260 100) avec exactement trois couleurs : 28 900 pixels blancs `0xFFFFFF` (le germe, rang 0),
/// 28 900 pixels roses `0xFF5AA0` (la particule, rang 1 sur 2 : `PALETTE[4]`) et 202 300 pixels de fond `0x0D1117`. Deux
/// sites, rayon 1 (`ceil(sqrt(1))`). Les canaux sont tous différents pour le rose et le fond : un ordre des octets inversé
/// (BGR) ou un octet de poids fort non nul est vu.
///
/// Ne couvre pas : la position de la particule (la graine la fixe : voir `dla_autres_graines`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn dla_a_la_main_germe_puis_une_particule() {
    let _verrou = verrou();
    for graine in [1, 2, 3, 0xDEAD_BEEF] {
        assert_eq!(givre_demarrer(0, graine), 1);
        assert_eq!((givre_largeur(), givre_hauteur()), (512, 512));
        assert_eq!((givre_temps(), givre_sites(), givre_rayon()), (0, 1, 0));
        assert_eq!(couleurs_de_l_image(), BTreeMap::from([(0x00FF_FFFF, 512 * 512)]), "graine {graine}, temps 0");

        assert_eq!(givre_avancer(1), 1);
        assert_eq!((givre_largeur(), givre_hauteur()), (510, 510));
        assert_eq!((givre_temps(), givre_sites(), givre_rayon()), (1, 2, 1));
        let attendu = BTreeMap::from([(0x00FF_FFFF, 28_900), (0x00FF_5AA0, 28_900), (0x000D_1117, 202_300)]);
        assert_eq!(couleurs_de_l_image(), attendu, "graine {graine}, une particule");
    }
}

/// juste : `givre_avancer` appelé par morceaux de tailles quelconques (dont 0) suit pas à pas l'état natif : après chaque
/// morceau, le temps, l'image entière, les compteurs et l'empreinte sont ceux du natif au même temps, jamais une valeur
/// gardée d'avant (une image ou une empreinte périmées se voient). `givre_avancer` rend à chaque fois le nombre d'unités
/// demandé. Pour la DLA (graine 3) et pour l'étoile ; les morceaux font 80 puis 3 unités au total.
///
/// Ne couvre pas : les morceaux qui dépassent la durée (voir `avancer_s_arrete_a_la_duree`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn avancer_par_morceaux() {
    let _verrou = verrou();
    for modele in [0, 2] {
        assert_eq!(givre_demarrer(modele, 3), 1);
        let mut cumul = 0;
        for morceau in [1, 0, 6, 13, 0, 60, 1, 0, 2] {
            assert_eq!(givre_avancer(morceau), morceau);
            cumul += morceau;
            verifier_identique(&natif(modele, 3, cumul), &format!("modèle {modele}, après {cumul} unités"));
        }
        assert_eq!(cumul, 83);
    }
}

/// juste : les trois flocons (plaque, étoile, dendrite : `reiter-*.txt`) joués jusqu'au bout (400, 120 et 800 pas) sont les
/// `Flocon::new(rayon, Parametres::lire(..))` natifs au même temps (V-VIT-02, V-VIT-03) : temps, durée, image pixel par pixel
/// (côté `2R + 1` : 61, 81, 121), cases gelées, rayon du cristal, compacité bit à bit, empreinte ; d'abord au temps 0, puis à
/// mi-chemin, puis au bout. Aucun compteur n'est éprouvé à vide : chaque flocon a au moins 100 cases gelées à la fin. La plaque
/// a pour empreinte finale le fichier `references/reiter-plaque.empreinte` (régression de notre code recoupée avec le natif) et
/// une compacité de exactement 1 (un hexagone plein : calcul fermé, voir `etape_3_exact`).
///
/// Ne couvre pas : les mesures des flocons autres que celles que la vitrine expose (pointes, défauts de symétrie).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn flocons_jusqu_au_bout() {
    let _verrou = verrou();
    for modele in 1..=3_u32 {
        let duree = TABLE[modele as usize].duree;
        assert_eq!(givre_demarrer(modele, 1), 1);
        verifier_identique(&natif(modele, 1, 0), &format!("modèle {modele}, temps 0"));
        assert_eq!(givre_avancer(duree / 2), duree / 2);
        verifier_identique(&natif(modele, 1, duree / 2), &format!("modèle {modele}, mi-chemin"));
        assert_eq!(givre_avancer(duree - duree / 2), duree - duree / 2);
        verifier_identique(&natif(modele, 1, duree), &format!("modèle {modele}, au bout"));
        assert_eq!(givre_temps(), duree);
        assert!(givre_sites() >= 100, "modèle {modele} : {} cases gelées", givre_sites());
        let cote = 2 * TABLE[modele as usize].rayon + 1;
        assert_eq!((givre_largeur(), givre_hauteur()), (cote, cote));
    }

    assert_eq!(givre_demarrer(1, 1), 1);
    assert_eq!(givre_avancer(400), 400);
    assert_eq!(empreinte_de_la_vitrine(), reference("reiter-plaque.empreinte"));
    assert_eq!(givre_compacite().to_bits(), 1.0_f64.to_bits());
}

/// juste : un flocon n'a pas de hasard, la vitrine ignore la graine pour les modèles 1 à 3 : la graine 1, la graine
/// `u32::MAX` et la graine 0 donnent, à 50 pas, la même empreinte, celle du flocon natif.
///
/// Ne couvre pas : la graine de la DLA (voir `dla_autres_graines`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn les_flocons_ignorent_la_graine() {
    let _verrou = verrou();
    for modele in 1..=3_u32 {
        for graine in [1, u32::MAX, 0] {
            verifier_modele(modele, graine, 50);
            assert_eq!(empreinte_de_la_vitrine(), natif(modele, 1, 50).empreinte);
        }
    }
}

/// juste : `givre_demarrer` remplace l'état entier. Après des pas, rappelé avec le même modèle et la même graine, il repart du
/// temps 0 (image, compteurs et empreinte de l'état initial natif) et, rejoué, redonne l'état natif. Passer d'un modèle à un
/// autre change tout : la durée, les dimensions de l'image (512, 61, 81, 121 puis de nouveau 512 au départ de la DLA), le
/// compteur de sites, l'empreinte ; rien de l'ancien modèle ne reste (ni image gardée, ni temps). Rendre à la DLA une autre graine
/// donne l'amas de cette graine, pas l'ancien.
///
/// Ne couvre pas : l'état avant tout démarrage (voir l'en-tête).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn demarrer_repart_de_zero_et_change_de_modele() {
    let _verrou = verrou();
    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_avancer(50), 50);
    assert_eq!(givre_demarrer(0, 1), 1);
    verifier_identique(&natif(0, 1, 0), "DLA redémarrée");
    assert_eq!(givre_avancer(50), 50);
    verifier_identique(&natif(0, 1, 50), "DLA redémarrée puis rejouée");

    assert_eq!(givre_demarrer(0, 2), 1);
    verifier_identique(&natif(0, 2, 0), "DLA, autre graine");
    assert_eq!(givre_avancer(50), 50);
    verifier_identique(&natif(0, 2, 50), "DLA, autre graine, rejouée");

    for (modele, cote) in [(1, 61), (2, 81), (3, 121), (2, 81), (0, 512), (3, 121), (1, 61)] {
        assert_eq!(givre_demarrer(modele, 9), 1);
        assert_eq!((givre_largeur(), givre_hauteur()), (cote, cote), "modèle {modele}");
        verifier_identique(&natif(modele, 9, 0), &format!("modèle {modele} au départ"));
        assert_eq!(givre_avancer(30), 30);
        verifier_identique(&natif(modele, 9, 30), &format!("modèle {modele} après 30 unités"));
    }
}

/// juste : `givre_avancer` ne joue jamais au-delà de la durée du scénario (2999, 400, 120, 800) et rend le nombre d'unités
/// qu'il a vraiment jouées. À trois unités de la fin, `givre_avancer(10)` en joue 3 ; ensuite il en joue 0, quel que soit `n`
/// (jusqu'à `u32::MAX`), et le temps, l'image et l'empreinte ne bougent plus : ce sont ceux de l'état natif à la durée. Un seul
/// appel de `u32::MAX` depuis le départ joue la durée entière, ni plus ni moins. `givre_avancer(0)` ne change rien et rend 0.
///
/// Ne couvre pas : une particule de DLA qui collerait hors du réseau (cela n'arrive pas pour 3000 sites).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn avancer_s_arrete_a_la_duree() {
    let _verrou = verrou();
    for modele in 0..=3_u32 {
        let duree = TABLE[modele as usize].duree;
        assert_eq!(givre_demarrer(modele, 4), 1);
        assert_eq!(givre_avancer(duree - 3), duree - 3, "modèle {modele}");
        assert_eq!(givre_avancer(0), 0);
        assert_eq!(givre_temps(), duree - 3);
        assert_eq!(givre_avancer(10), 3, "modèle {modele} : à trois unités de la fin");
        assert_eq!(givre_temps(), duree);
        let fin = empreinte_de_la_vitrine();
        for n in [1, 10, u32::MAX] {
            assert_eq!(givre_avancer(n), 0, "modèle {modele} : au-delà de la durée");
        }
        assert_eq!(givre_temps(), duree);
        assert_eq!(empreinte_de_la_vitrine(), fin);
        verifier_identique(&natif(modele, 4, duree), &format!("modèle {modele}, à la durée"));

        assert_eq!(givre_demarrer(modele, 4), 1);
        assert_eq!(givre_avancer(u32::MAX), duree, "modèle {modele} : un appel de u32::MAX");
        assert_eq!(empreinte_de_la_vitrine(), fin, "modèle {modele}");
    }
}

/// juste : un modèle qui n'existe pas (4, 100, `u32::MAX`) est refusé : `givre_demarrer` rend 0 et ne change **rien** (le
/// temps, la durée, l'image et l'empreinte de la simulation en cours restent ceux d'avant, et celle-ci continue d'avancer
/// comme le natif) ; un modèle qui existe rend 1.
///
/// Ne couvre pas : le refus quand rien n'a jamais été démarré (voir l'en-tête).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn modele_inconnu_ne_change_rien() {
    let _verrou = verrou();
    assert_eq!(givre_demarrer(2, 5), 1);
    assert_eq!(givre_avancer(20), 20);
    for inconnu in [4, 5, 100, u32::MAX] {
        assert_eq!(givre_demarrer(inconnu, 5), 0, "modèle {inconnu}");
        verifier_identique(&natif(2, 5, 20), &format!("après givre_demarrer({inconnu}, 5)"));
    }
    assert_eq!(givre_avancer(10), 10);
    verifier_identique(&natif(2, 5, 30), "la simulation continue");
}

/// juste : hors de portée, pour chaque modèle, au départ et à la fin. `givre_pixel` rend `u32::MAX` pour `x == largeur`, pour
/// `y == hauteur`, pour les deux ensemble et très au-delà (`u32::MAX`) ; les quatre coins de l'image, eux, sont dans l'image
/// (ce sont des couleurs : au plus `0xFFFFFF`). `givre_empreinte_mot` rend 0 pour 8, 9, 64 et `u32::MAX`, et les huit premiers
/// mots ne sont pas tous nuls. La compacité de la DLA est `NaN`, celle d'un flocon est finie et dans `[0, 1]`.
///
/// Ne couvre pas : une image dont la largeur n'est pas la hauteur (aucune ne l'est : un accesseur qui échangerait largeur et
/// hauteur donnerait les mêmes résultats).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn hors_de_portee() {
    let _verrou = verrou();
    for modele in 0..=3_u32 {
        assert_eq!(givre_demarrer(modele, 1), 1);
        for pas in [0, 60] {
            assert_eq!(givre_avancer(pas), pas);
            let (largeur, hauteur) = (givre_largeur(), givre_hauteur());
            assert!(largeur > 0 && hauteur > 0);
            for (x, y) in [(0, 0), (largeur - 1, 0), (0, hauteur - 1), (largeur - 1, hauteur - 1)] {
                assert!(givre_pixel(x, y) <= 0x00FF_FFFF, "modèle {modele} : coin ({x}, {y})");
            }
            for (x, y) in [
                (largeur, 0),
                (0, hauteur),
                (largeur, hauteur),
                (largeur, hauteur - 1),
                (largeur - 1, hauteur),
                (largeur + 1, 0),
                (u32::MAX, 0),
                (0, u32::MAX),
                (u32::MAX, u32::MAX),
            ] {
                assert_eq!(givre_pixel(x, y), u32::MAX, "modèle {modele} : pixel ({x}, {y}) hors de l'image");
            }
            assert!((0..8).any(|i| givre_empreinte_mot(i) != 0));
            for i in [8, 9, 64, u32::MAX] {
                assert_eq!(givre_empreinte_mot(i), 0, "mot {i}");
            }
            let compacite = givre_compacite();
            if modele == 0 {
                assert!(compacite.is_nan());
            } else {
                assert!(compacite.is_finite() && (0.0..=1.0).contains(&compacite), "compacité {compacite}");
            }
        }
    }
}

/// juste : les pixels des flocons au temps 0, calculés à la main d'après la doc de `Flocon::apercu` (V-VIT-06, outil :
/// `valeurs_etape5.py`) : la case centrale (gelée) est le pixel `(R, R)` de couleur `(235, 245, 255)` = `0xEBF5FF` ; les autres
/// cases valent `β` et ont la couleur `(0, niveau / 2, niveau)` avec `niveau = (brut * 255) >> 32` : la plaque (`β = 0`) `0x000000`,
/// l'étoile (`β = 0.45`, `brut = 1932735283`, `niveau = 114`) `0x003972`, la dendrite (`β = 0.4`, `brut = 1717986918`,
/// `niveau = 101` : `0.4 * 255 = 102` mais le `brut` est arrondi un peu en dessous) `0x003265` ; les pixels qui ne sont
/// l'image d'aucune case (les coins) sont noirs. Le nombre de pixels de chaque couleur est exact : l'hexagone de rayon `R` a
/// `3R(R + 1) + 1` cases (2791, 4921, 10981), le reste du carré `(2R + 1)²` est noir. Le pixel `(0, R)` est la case `(-R, 0)`
/// de la couronne, de couleur `β` ; le pixel `(0, 0)` n'est pas une case. Les canaux sont tous différents (`0, 57, 114`) : un
/// ordre RVB inversé est vu.
///
/// Ne couvre pas : les couleurs d'un flocon qui a grandi (voir `flocons_jusqu_au_bout`, comparé au natif).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn pixels_des_flocons_au_temps_0() {
    let _verrou = verrou();
    // (modèle, rayon, couleur de β, nombre de pixels de la couleur de β)
    let cas = [(1, 30, 0x0000_0000, 0), (2, 40, 0x0000_3972, 4920), (3, 60, 0x0000_3265, 10_980)];
    for (modele, rayon, fond, cases_de_fond) in cas {
        assert_eq!(givre_demarrer(modele, 1), 1);
        assert_eq!(givre_pixel(rayon, rayon), 0x00EB_F5FF, "modèle {modele} : le germe");
        assert_eq!(givre_pixel(0, rayon), fond, "modèle {modele} : la case (-R, 0)");
        assert_eq!(givre_pixel(0, 0), 0, "modèle {modele} : le coin");
        let cote = (2 * rayon + 1) as usize;
        let cases = (3 * rayon * (rayon + 1) + 1) as usize;
        let mut attendu = BTreeMap::new();
        attendu.insert(0x00EB_F5FF, 1);
        *attendu.entry(0).or_insert(0) += cote * cote - cases;
        if fond == 0 {
            *attendu.entry(0).or_insert(0) += cases - 1;
        } else {
            attendu.insert(fond, cases_de_fond);
            assert_eq!(cases_de_fond, cases - 1);
        }
        assert_eq!(couleurs_de_l_image(), attendu, "modèle {modele} au temps 0");
        assert_eq!((givre_sites(), givre_rayon()), (1, 0));
        assert_eq!(givre_compacite().to_bits(), 1.0_f64.to_bits());
    }
}

/// régression : les scénarios d'exemple du dépôt disent ce que dit la table de la doc de `vitrine`, que la vitrine embarque
/// ou redit : `scenarios/dla.txt` est une `boite 401 401` (donc `demi_cote` 200, la valeur de `Amas::new`) et `dla 3000`
/// (donc la durée 2999) ; chaque `scenarios/reiter-*.txt` est un `hexagone` de rayon `R` et un `reiter` dont les quatre
/// arguments, **tels qu'écrits**, sont ceux de la table (`1`, `0.45`, `0.1`, `120` pour l'étoile…), et dont le dernier est la
/// durée. Si un scénario change sans que la doc, la vitrine et ce test changent ensemble, la page ne montrerait plus ce que
/// disent les scénarios.
///
/// Ne couvre pas : que la vitrine reste d'accord avec ces fichiers dans l'avenir sans que ce test soit relu (c'est une
/// régression de forme : elle compare deux textes).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn les_scenarios_disent_ce_que_dit_la_doc() {
    let lire = |nom: &str| {
        let chemin = racine().join("scenarios").join(nom);
        let texte =
            std::fs::read_to_string(&chemin).unwrap_or_else(|erreur| panic!("{} se lit : {erreur}", chemin.display()));
        lire_scenario(&texte).unwrap_or_else(|erreur| panic!("{nom} se lit comme un scénario : {erreur:?}"))
    };

    let dla = lire(TABLE[0].scenario);
    assert_eq!(dla.forme, Forme::Boite { largeur: 401, hauteur: 401 });
    assert_eq!(demi_cote_de_la_boite(401, 401), 200);
    assert_eq!(particules(&dla), Ok(Some(3000)));
    assert_eq!(TABLE[0].duree, 3000 - 1);

    for modele in &TABLE[1..] {
        let scenario = lire(modele.scenario);
        assert_eq!(scenario.forme, Forme::Hexagone { rayon: modele.rayon }, "{}", modele.scenario);
        assert_eq!(scenario.directives.len(), 1, "{}", modele.scenario);
        let directive = &scenario.directives[0];
        assert_eq!(directive.nom, "reiter", "{}", modele.scenario);
        let duree = modele.duree.to_string();
        assert_eq!(directive.args, [modele.alpha, modele.beta, modele.gamma, duree.as_str()], "{}", modele.scenario);
    }
}

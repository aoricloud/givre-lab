//! Étape 3c : la table β-γ de Reiter (région de rayon 100) avec le BORD CIRCULAIRE de l'auteur. Valeurs attendues : `references/etape-3-valeurs.md`,
//! identifiants `V-BORD-xx` (et `V-PUB-xx` de l'étape 3 pour la source de la table).
//!
//! **La source.** C. A. Reiter, « A local cellular model for snow crystal growth », *Chaos, Solitons & Fractals* 23 (2005) 1111-1119, page de matériaux
//! auxiliaires `webbox.lafayette.edu/~reiterc/mvp/sfn/nn/index.html` (« Beta-Gamma Table 100 : A β-γ Table of Radius 100 », 240 vignettes
//! `th_nn_h<β>_x<γ>_<n>.png`, `o` pour la virgule), noms relus par le juge de l'étape 3 le 8/11/2026 ; les 240 triplets `(β, γ, n)` de ce fichier
//! sont tirés de la liste des noms par `faire_table.py` (un petit script de l'outillage, qui n'invente rien : il découpe les noms).
//!
//! **Ce que le test calcule, d'après le script J de l'auteur** (`.../sfn/sfn.html`, `sfn.ijs`, fonction `mk_dla`, lu par le juge) : à partir du germe, on fait
//! un pas tant qu'aucune glace n'a atteint le cercle intérieur `icir` (`distance2 >= 7203 = 3 · 98² / 4`) et que `k < 10000` (le plafond est un paramètre du script ; les numéros 9999 de la table disent 10 000) ; le numéro `n` de la vignette
//! est `k - 1` où `k` est le nombre de pas faits. Le bord fixé à `β` est `ocir` : `distance2 >= 7500` (`Bord::Circulaire`, `R = 100`). Le test fait donc
//! exactement la boucle de l'auteur : `k = 0`, puis tant que `distance2_cristal() < 7203` et `k < 10000` : `pas()`, `k += 1` ; `n = k - 1`.
//!
//! **Les cases plafonnées** (21 cases, `n = 9999`) : la boucle s'arrête à `k = 10000` sans que `distance2_cristal() >= 7203` ait été vrai avant (elle peut
//! l'être exactement au 10000e pas : `n` vaut 9999 dans les deux cas, la table ne distingue pas). Le test vérifie donc seulement que **aucun arrêt n'a lieu
//! avant le 10000e pas**, c'est-à-dire que `distance2_cristal() < 7203` après chacun des 9999 premiers pas.
//!
//! **Pourquoi ce test.** Avec le bord hexagonal (la couronne d'un hexagone de rayon 100), seules 181 cases sur 240 sont retrouvées (160 non plafonnées et les 21
//! plafonnées) : les 59 « dendrites lentes » (`β <= 0.5` et `γ <= 0.05`) ont 0,1 à 4,9 % de pas en plus que la table. Avec le bord circulaire, les 240 cases
//! sont retrouvées (mesures : `references/etape-3-valeurs.md`, V-BORD-27 et V-BORD-28). Le juge de l'étape 3 l'avait établi avec un outil à part
//! (`table_juge.rs`, 240 sur 240) ; ces tests le font entrer dans la bibliothèque.

mod etape_3_commun;

use etape_3_commun::parametres;
use givre_lab::flocon::{Bord, Flocon};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// L'arrêt de l'auteur, `icir` de `sfn.ijs` pour une région de rayon 100 : `distance2 >= 7203` (`4 · distance2 >= 3 · 98²`).
const ARRET: u64 = 7203;

/// Le plafond de pas de la table : un paramètre de `mk_dla` (2000 dans l'appel d'exemple du script), 10 000 pour la table, dont les numéros plafonnés
/// valent 9999.
const PLAFOND: u64 = 10_000;

/// Les 240 cases de la table publiée : `(β, γ, n)` où `n` est le numéro lu dans le nom de l'image `th_nn_h<β>_x<γ>_<n>.png`.
const TABLE: [(&str, &str, u64); 240] = [
    ("0", "0", 9999),
    ("0", "0.0001", 9999),
    ("0", "0.001", 9999),
    ("0", "0.002", 9999),
    ("0", "0.0035", 9999),
    ("0", "0.005", 9999),
    ("0", "0.01", 8499),
    ("0", "0.02", 4249),
    ("0", "0.05", 1699),
    ("0", "0.1", 849),
    ("0", "0.5", 169),
    ("0", "1", 84),
    ("0.05", "0", 9999),
    ("0.05", "0.0001", 9999),
    ("0.05", "0.001", 9999),
    ("0.05", "0.002", 9999),
    ("0.05", "0.0035", 9999),
    ("0.05", "0.005", 9999),
    ("0.05", "0.01", 6914),
    ("0.05", "0.02", 3682),
    ("0.05", "0.05", 1528),
    ("0.05", "0.1", 847),
    ("0.05", "0.5", 169),
    ("0.05", "1", 84),
    ("0.1", "0", 9999),
    ("0.1", "0.0001", 9999),
    ("0.1", "0.001", 9999),
    ("0.1", "0.002", 9999),
    ("0.1", "0.0035", 9999),
    ("0.1", "0.005", 9253),
    ("0.1", "0.01", 5526),
    ("0.1", "0.02", 3098),
    ("0.1", "0.05", 1356),
    ("0.1", "0.1", 763),
    ("0.1", "0.5", 169),
    ("0.1", "1", 84),
    ("0.15", "0", 9999),
    ("0.15", "0.0001", 9999),
    ("0.15", "0.001", 9999),
    ("0.15", "0.002", 9999),
    ("0.15", "0.0035", 8085),
    ("0.15", "0.005", 6733),
    ("0.15", "0.01", 4338),
    ("0.15", "0.02", 2590),
    ("0.15", "0.05", 1186),
    ("0.15", "0.1", 678),
    ("0.15", "0.5", 169),
    ("0.15", "1", 84),
    ("0.2", "0", 8994),
    ("0.2", "0.0001", 8832),
    ("0.2", "0.001", 7656),
    ("0.2", "0.002", 6700),
    ("0.2", "0.0035", 5654),
    ("0.2", "0.005", 4878),
    ("0.2", "0.01", 3375),
    ("0.2", "0.02", 2162),
    ("0.2", "0.05", 1093),
    ("0.2", "0.1", 593),
    ("0.2", "0.5", 169),
    ("0.2", "1", 84),
    ("0.25", "0", 5679),
    ("0.25", "0.0001", 5613),
    ("0.25", "0.001", 5028),
    ("0.25", "0.002", 4538),
    ("0.25", "0.0035", 4000),
    ("0.25", "0.005", 3564),
    ("0.25", "0.01", 2638),
    ("0.25", "0.02", 1759),
    ("0.25", "0.05", 929),
    ("0.25", "0.1", 508),
    ("0.25", "0.5", 169),
    ("0.25", "1", 84),
    ("0.3", "0", 3755),
    ("0.3", "0.0001", 3741),
    ("0.3", "0.001", 3450),
    ("0.3", "0.002", 3167),
    ("0.3", "0.0035", 2857),
    ("0.3", "0.005", 2622),
    ("0.3", "0.01", 2062),
    ("0.3", "0.02", 1427),
    ("0.3", "0.05", 762),
    ("0.3", "0.1", 508),
    ("0.3", "0.5", 169),
    ("0.3", "1", 84),
    ("0.35", "0", 2601),
    ("0.35", "0.0001", 2593),
    ("0.35", "0.001", 2446),
    ("0.35", "0.002", 2292),
    ("0.35", "0.0035", 2068),
    ("0.35", "0.005", 1909),
    ("0.35", "0.01", 1586),
    ("0.35", "0.02", 1175),
    ("0.35", "0.05", 676),
    ("0.35", "0.1", 423),
    ("0.35", "0.5", 169),
    ("0.35", "1", 84),
    ("0.4", "0", 1818),
    ("0.4", "0.0001", 1817),
    ("0.4", "0.001", 1737),
    ("0.4", "0.002", 1658),
    ("0.4", "0.0035", 1572),
    ("0.4", "0.005", 1489),
    ("0.4", "0.01", 1251),
    ("0.4", "0.02", 927),
    ("0.4", "0.05", 591),
    ("0.4", "0.1", 421),
    ("0.4", "0.5", 168),
    ("0.4", "1", 84),
    ("0.45", "0", 1328),
    ("0.45", "0.0001", 1328),
    ("0.45", "0.001", 1252),
    ("0.45", "0.002", 1249),
    ("0.45", "0.0035", 1168),
    ("0.45", "0.005", 1088),
    ("0.45", "0.01", 926),
    ("0.45", "0.02", 757),
    ("0.45", "0.05", 506),
    ("0.45", "0.1", 338),
    ("0.45", "0.5", 84),
    ("0.45", "1", 84),
    ("0.5", "0", 999),
    ("0.5", "0.0001", 998),
    ("0.5", "0.001", 922),
    ("0.5", "0.002", 920),
    ("0.5", "0.0035", 917),
    ("0.5", "0.005", 839),
    ("0.5", "0.01", 757),
    ("0.5", "0.02", 591),
    ("0.5", "0.05", 423),
    ("0.5", "0.1", 254),
    ("0.5", "0.5", 84),
    ("0.5", "1", 84),
    ("0.55", "0", 751),
    ("0.55", "0.0001", 751),
    ("0.55", "0.001", 672),
    ("0.55", "0.002", 672),
    ("0.55", "0.0035", 672),
    ("0.55", "0.005", 671),
    ("0.55", "0.01", 590),
    ("0.55", "0.02", 506),
    ("0.55", "0.05", 338),
    ("0.55", "0.1", 253),
    ("0.55", "0.5", 84),
    ("0.55", "1", 84),
    ("0.6", "0", 586),
    ("0.6", "0.0001", 585),
    ("0.6", "0.001", 506),
    ("0.6", "0.002", 505),
    ("0.6", "0.0035", 505),
    ("0.6", "0.005", 505),
    ("0.6", "0.01", 421),
    ("0.6", "0.02", 421),
    ("0.6", "0.05", 337),
    ("0.6", "0.1", 253),
    ("0.6", "0.5", 84),
    ("0.6", "1", 84),
    ("0.65", "0", 421),
    ("0.65", "0.0001", 421),
    ("0.65", "0.001", 421),
    ("0.65", "0.002", 421),
    ("0.65", "0.0035", 420),
    ("0.65", "0.005", 420),
    ("0.65", "0.01", 338),
    ("0.65", "0.02", 338),
    ("0.65", "0.05", 253),
    ("0.65", "0.1", 169),
    ("0.65", "0.5", 84),
    ("0.65", "1", 84),
    ("0.7", "0", 336),
    ("0.7", "0.0001", 336),
    ("0.7", "0.001", 336),
    ("0.7", "0.002", 335),
    ("0.7", "0.0035", 335),
    ("0.7", "0.005", 334),
    ("0.7", "0.01", 253),
    ("0.7", "0.02", 253),
    ("0.7", "0.05", 169),
    ("0.7", "0.1", 169),
    ("0.7", "0.5", 84),
    ("0.7", "1", 84),
    ("0.75", "0", 253),
    ("0.75", "0.0001", 253),
    ("0.75", "0.001", 253),
    ("0.75", "0.002", 253),
    ("0.75", "0.0035", 253),
    ("0.75", "0.005", 253),
    ("0.75", "0.01", 253),
    ("0.75", "0.02", 169),
    ("0.75", "0.05", 169),
    ("0.75", "0.1", 168),
    ("0.75", "0.5", 84),
    ("0.75", "1", 84),
    ("0.8", "0", 168),
    ("0.8", "0.0001", 168),
    ("0.8", "0.001", 168),
    ("0.8", "0.002", 168),
    ("0.8", "0.0035", 168),
    ("0.8", "0.005", 168),
    ("0.8", "0.01", 168),
    ("0.8", "0.02", 168),
    ("0.8", "0.05", 168),
    ("0.8", "0.1", 84),
    ("0.8", "0.5", 84),
    ("0.8", "1", 84),
    ("0.85", "0", 168),
    ("0.85", "0.0001", 168),
    ("0.85", "0.001", 168),
    ("0.85", "0.002", 168),
    ("0.85", "0.0035", 168),
    ("0.85", "0.005", 168),
    ("0.85", "0.01", 84),
    ("0.85", "0.02", 84),
    ("0.85", "0.05", 84),
    ("0.85", "0.1", 84),
    ("0.85", "0.5", 84),
    ("0.85", "1", 84),
    ("0.9", "0", 84),
    ("0.9", "0.0001", 84),
    ("0.9", "0.001", 84),
    ("0.9", "0.002", 84),
    ("0.9", "0.0035", 84),
    ("0.9", "0.005", 84),
    ("0.9", "0.01", 84),
    ("0.9", "0.02", 84),
    ("0.9", "0.05", 84),
    ("0.9", "0.1", 84),
    ("0.9", "0.5", 84),
    ("0.9", "1", 84),
    ("0.95", "0", 84),
    ("0.95", "0.0001", 84),
    ("0.95", "0.001", 84),
    ("0.95", "0.002", 84),
    ("0.95", "0.0035", 84),
    ("0.95", "0.005", 84),
    ("0.95", "0.01", 84),
    ("0.95", "0.02", 84),
    ("0.95", "0.05", 84),
    ("0.95", "0.1", 84),
    ("0.95", "0.5", 84),
    ("0.95", "1", 84),
];

/// Le résultat du calcul pour une case : `n = k - 1` et si l'arrêt a eu lieu avant le 10000e pas (`arret_avant_plafond`, faux pour une case plafonnée).
struct Resultat {
    n: u64,
    arret_avant_plafond: bool,
}

/// La boucle de l'auteur (`mk_dla`) sur une région de rayon 100 avec le bord `bord`, `α = 1` : `k` pas tant que `distance2_cristal() < ARRET` et `k <
/// PLAFOND`, puis `n = k - 1`. Les pas sont comptés ici ; `temps` doit valoir ce compte.
fn calculer(beta: &str, gamma: &str, bord: Bord) -> Resultat {
    let mut f = Flocon::avec_bord(100, parametres("1", beta, gamma), bord);
    let mut k: u64 = 0;
    while f.distance2_cristal() < ARRET && k < PLAFOND {
        f.pas();
        k += 1;
    }
    assert_eq!(f.temps(), k, "temps compte les pas");
    Resultat { n: k - 1, arret_avant_plafond: k < PLAFOND }
}

/// Fait calculer `cases` en parallèle (au plus 8 fils, le plus long d'abord) et rend les écarts : `(β, γ, publié, calculé)` pour chaque case dont le
/// calcul diffère du numéro publié. Une case plafonnée (`n = 9999`) doit l'être aussi dans le calcul.
fn ecarts(cases: &[(&'static str, &'static str, u64)], bord: Bord) -> Vec<(&'static str, &'static str, u64, u64)> {
    let mut ordre: Vec<(&'static str, &'static str, u64)> = cases.to_vec();
    ordre.sort_by_key(|&(_, _, n)| std::cmp::Reverse(n));
    let ecarts = Mutex::new(Vec::new());
    let suivant = AtomicUsize::new(0);
    let fils = std::thread::available_parallelism().map_or(4, |n| n.get()).min(8);
    std::thread::scope(|s| {
        for _ in 0..fils {
            s.spawn(|| {
                loop {
                    let i = suivant.fetch_add(1, Ordering::SeqCst);
                    let Some(&(beta, gamma, publie)) = ordre.get(i) else { break };
                    let resultat = calculer(beta, gamma, bord);
                    if resultat.n != publie {
                        ecarts.lock().unwrap().push((beta, gamma, publie, resultat.n));
                    }
                    if publie == PLAFOND - 1 {
                        assert!(
                            !resultat.arret_avant_plafond,
                            "β = {beta}, γ = {gamma} : plafonnée (9999), aucun arrêt avant le 10000e pas"
                        );
                    }
                }
            });
        }
    });
    let mut ecarts = ecarts.into_inner().unwrap();
    ecarts.sort_by(|a, b| a.3.cmp(&b.3).then(a.0.cmp(b.0)).then(a.1.cmp(b.1)));
    ecarts
}

/// juste : vingt cases de la table publiée de rayon 100, avec le bord circulaire de l'auteur (V-BORD-27) : le nombre `n` calculé par la boucle de
/// l'auteur (voir l'en-tête) est exactement le numéro publié, `α = 1`. Le sous-ensemble contient **8 cases que le bord hexagonal rate** (mesuré :
/// `(β, γ) = (0.2, 0.001)` publié 7656 contre 7972 avec le bord hexagonal, soit 4,1 % de plus ; `(0.1, 0.02)` 3098 contre 3112 ; `(0.3, 0.02)` 1427 contre 1430 ;
/// `(0.2, 0.05)` 1093 contre 1094 ; `(0.45, 0.005)` 1088 contre 1089 ; `(0.5, 0)` 999 contre 1000 ; `(0.5, 0.002)` 920 contre 921 ; `(0.5, 0.0035)` 917 contre
/// 918 : les plus lentes et les plus rapides des 59 cases fausses, un écart de 4 % et des écarts d'un seul pas) et **12 cases que le bord hexagonal
/// donne déjà exactes** (des plaques `β = 0`, des étoiles, des dendrites rapides, le haut de la table `β >= 0.55` ou `γ >= 0.1`, dont des égalités à un pas près :
/// `(0.55, 0.002)` 672, `(0.7, 0.002)` 335, `(0.85, 0.005)` 168, `(0.9, 0)` 84) que le bord circulaire doit garder. Un bord ignoré (hexagonal), un cercle de
/// rayon `R` au lieu de `R sin 60°`, un `>` à la place de `>=`, un arrêt au rayon hexagonal 85 ou un `distance2_cristal` faux font échouer ce test
/// (les huit premières cases le font échouer chacune). Durée mesurée : 9 secondes sur ce PC (8 fils) avec le prototype (un pas d'un
/// hexagone de rayon 100 coûte environ 1,2 ms) ; le plus long est (0.2, 0.001) avec 7657 pas. Un code trois fois plus lent resterait vers 30 secondes sur 8 fils
/// (le temps du plus long est une borne : 7657 pas).
///
/// Ne couvre pas : les 232 autres cases (voir `table_publiee_complete`, ignoré par défaut), les cases plafonnées (idem), les régions de rayon 200 et 400.
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
fn table_publiee_sous_ensemble_a_bord_circulaire() {
    const SOUS_ENSEMBLE: [(&str, &str, u64); 20] = [
        ("0.2", "0.001", 7656),
        ("0.1", "0.02", 3098),
        ("0.3", "0.02", 1427),
        ("0.2", "0.05", 1093),
        ("0.45", "0.005", 1088),
        ("0.5", "0", 999),
        ("0.5", "0.002", 920),
        ("0.5", "0.0035", 917),
        ("0", "0.1", 849),
        ("0", "1", 84),
        ("0.2", "0.1", 593),
        ("0.3", "0.05", 762),
        ("0.4", "0.5", 168),
        ("0.45", "0.1", 338),
        ("0.5", "0.1", 254),
        ("0.55", "0.002", 672),
        ("0.6", "0.01", 421),
        ("0.7", "0.002", 335),
        ("0.85", "0.005", 168),
        ("0.9", "0", 84),
    ];
    for cas in SOUS_ENSEMBLE {
        assert!(TABLE.contains(&cas), "{cas:?} est dans la table de l'auteur");
    }
    let ecarts = ecarts(&SOUS_ENSEMBLE, Bord::Circulaire);
    assert!(ecarts.is_empty(), "cases (β, γ, publié, calculé) qui diffèrent : {ecarts:?}");
}

/// juste : les 240 cases de la table publiée de rayon 100 avec le bord circulaire (V-BORD-28) : pour chacune, `n` calculé par la boucle de l'auteur est le
/// numéro publié (219 cases non plafonnées : exactement ; 21 plafonnées à 9999 : aucun arrêt avant le 10000e pas, voir l'en-tête). Le bord hexagonal n'en
/// retrouve que 181 sur 240. Ce test est long (467 245 pas de simulation en tout, un hexagone de 30 301 cases : 99 secondes mesurées sur 8 fils sur ce PC) et
/// reste donc ignoré par défaut :
///
/// `cargo test --features etape-3-bord --test etape_3c_table -- --ignored`
///
/// Ne couvre pas : les tables de rayon 200 et 400 de l'auteur, ni la lecture du numéro (« pas moins un »), qui vient du code de l'auteur, non de la table.
#[test]
#[cfg_attr(not(feature = "etape-3-bord"), ignore)]
#[cfg_attr(feature = "etape-3-bord", ignore = "long : toute la table de 240 cases (voir la doc)")]
fn table_publiee_complete() {
    let ecarts = ecarts(&TABLE, Bord::Circulaire);
    assert!(ecarts.is_empty(), "{} cases sur 240 diffèrent (β, γ, publié, calculé) : {ecarts:?}", ecarts.len());
}

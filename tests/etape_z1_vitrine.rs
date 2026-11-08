//! Étape Z1 : les quatre fonctions de mesure de la vitrine (`givre_mesurer`, `givre_correlation_rayon`, `givre_correlation_valeur`,
//! `givre_dimension`), côté natif. Valeurs attendues : `references/etape-z1-valeurs.md`, identifiants `V-Z1-xx`.
//!
//! Comme `etape_5_vitrine.rs`, on appelle les fonctions `givre_*` comme des fonctions Rust et on les compare à l'état **natif** construit en
//! face (`Amas::new(graine, 200)`, `croitre_jusqu_a`, puis `correlation::mesure`, que les tests de `etape_z1_exact` et `etape_z1_formes`
//! éprouvent). Le module WebAssembly lui-même est éprouvé dans `etape_z1_page.rs` (sous Node).
//!
//! L'état de la vitrine est global au processus et les tests d'un fichier tournent en parallèle : chaque test prend d'abord le verrou
//! `VERROU`, propre à ce fichier, et le garde jusqu'à la fin. Aucun test de ce fichier n'a de tolérance : tout est une égalité exacte
//! (entiers, bits des `f64`).
//!
//! Tant qu'une des quatre fonctions de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test **s'arrête d'un coup** au lieu de
//! marquer un test en échec : une panique ne peut pas traverser une `extern "C" fn` (le message est « panicked at src/vitrine.rs:<ligne> : not
//! yet implemented », puis « panic in a function that cannot unwind » et l'abandon du processus : `SIGABRT` sous Linux, `0xc0000409` sous
//! Windows). C'est un échec, et la ligne citée dit quelle fonction reste à écrire ; les autres fichiers de test continuent avec `--no-fail-fast`.

use std::sync::{Mutex, MutexGuard};

use givre_lab::correlation::{Mesure, etendue, mesure};
use givre_lab::dla::Amas;
use givre_lab::vitrine::{
    MESURE_SITES_MAX, givre_avancer, givre_correlation_rayon, givre_correlation_valeur, givre_demarrer,
    givre_dimension, givre_mesurer, givre_sites,
};

static VERROU: Mutex<()> = Mutex::new(());

/// Prend le verrou ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou() -> MutexGuard<'static, ()> {
    VERROU.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// La mesure native de l'amas de la vitrine de graine `graine` après `pas` particules (`Amas::new(graine, 200)`, `1 + pas` sites).
fn mesure_native(graine: u32, pas: u32) -> Option<Mesure> {
    let mut amas = Amas::new(u64::from(graine), 200);
    amas.croitre_jusqu_a(1 + pas as usize);
    mesure(amas.sites())
}

/// Vérifie que les quatre fonctions disent exactement `attendue` (ou « pas de mesure » si `None`), après un `givre_mesurer` qui a rendu
/// `points`.
fn verifier_la_mesure_gardee(attendue: &Option<Mesure>, points: u32, contexte: &str) {
    match attendue {
        Some(m) => {
            assert_eq!(points as usize, m.rayons.len(), "{contexte} : nombre de points");
            for (i, (rayon, valeur)) in m.rayons.iter().zip(&m.valeurs).enumerate() {
                assert_eq!(givre_correlation_rayon(i as u32), *rayon, "{contexte} : rayon {i}");
                assert_eq!(givre_correlation_valeur(i as u32).to_bits(), valeur.to_bits(), "{contexte} : valeur {i}");
            }
            assert_eq!(givre_dimension().to_bits(), m.dimension.to_bits(), "{contexte} : dimension");
            for hors in [points, points + 1, 1000, u32::MAX] {
                assert_eq!(givre_correlation_rayon(hors), 0, "{contexte} : rayon hors de la courbe ({hors})");
                assert!(givre_correlation_valeur(hors).is_nan(), "{contexte} : valeur hors de la courbe ({hors})");
            }
        }
        None => {
            assert_eq!(points, 0, "{contexte} : pas de mesure");
            assert_eq!(givre_correlation_rayon(0), 0, "{contexte} : rayon 0 sans mesure");
            assert!(givre_correlation_valeur(0).is_nan(), "{contexte} : valeur 0 sans mesure");
            assert!(givre_dimension().is_nan(), "{contexte} : dimension sans mesure");
        }
    }
}

/// juste : la vitrine mesure l'amas du moment comme la bibliothèque (V-Z1-23). Pour la DLA de graine 1 jouée jusqu'au bout (2999 particules, 3000
/// sites), la graine `0xDEAD_BEEF` à 1500, la graine 7 à 800 et `u32::MAX` à 2000 : `givre_mesurer()` rend le nombre de rayons de la fenêtre de
/// `correlation::mesure`, `givre_correlation_rayon(i)` le `i`-ième rayon (4, 5, 6, …), `givre_correlation_valeur(i)` son `C(r)` et
/// `givre_dimension()` la dimension, tous égaux AU BIT PRÈS à ceux de `mesure` ; un second `givre_mesurer()` sur le même état rend le même nombre
/// de points ; hors de la courbe (`i` égal au nombre de points, plus un, 1000, `u32::MAX`) le rayon vaut 0 et la valeur est `NaN`.
///
/// Ne couvre pas : les modèles qui ne sont pas la DLA, ni l'état avant tout démarrage (voir les tests suivants et `correlation-wasm.mjs`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_vitrine_mesure_comme_la_bibliotheque() {
    let _verrou = verrou();
    for (graine, pas) in [(1_u32, 2999_u32), (0xDEAD_BEEF, 1500), (7, 800), (u32::MAX, 2000)] {
        assert_eq!(givre_demarrer(0, graine), 1);
        assert_eq!(givre_avancer(pas), pas);
        let attendue = mesure_native(graine, pas);
        assert!(attendue.is_some(), "graine {graine}, {pas} particules : la bibliothèque a une mesure");
        let contexte = format!("graine {graine}, {pas} particules");
        let points = givre_mesurer();
        assert!(points > 0, "{contexte}");
        verifier_la_mesure_gardee(&attendue, points, &contexte);
        assert_eq!(givre_mesurer(), points, "{contexte} : un second appel");
        verifier_la_mesure_gardee(&attendue, points, &format!("{contexte} (second appel)"));
    }
}

/// juste : « pas de mesure » (V-Z1-24). La plaque, l'étoile et la dendrite, au temps 0 et au bout du scénario : `givre_mesurer()` rend 0, le rayon
/// 0 vaut 0, la valeur 0 et la dimension sont `NaN`. La DLA au temps 0 (un site) : 0. Et la frontière de l'amas trop petit : pour la graine 1, soit
/// `n0` le plus petit nombre de sites dont l'étendue vaut au moins 72 (la plus petite étendue qui a une fenêtre : `etendue / 8 >= 9`, trouvé
/// dans le test en faisant croître l'amas natif site par site) : avec `n0 - 1` sites la vitrine rend 0, avec `n0` sites elle rend le nombre de
/// points de la bibliothèque (6 : les rayons 4 à 9) ; les amas de 2, 21 et `n0 - 1` sites n'ont pas de mesure.
///
/// Ne couvre pas : le plafond de taille (voir `le_plafond_de_la_mesure`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn pas_de_mesure() {
    let _verrou = verrou();
    for (modele, duree) in [(1_u32, 400_u32), (2, 120), (3, 800)] {
        for pas in [0, duree] {
            assert_eq!(givre_demarrer(modele, 1), 1);
            assert_eq!(givre_avancer(pas), pas);
            let points = givre_mesurer();
            verifier_la_mesure_gardee(&None, points, &format!("modèle {modele}, {pas} pas"));
        }
    }
    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_sites(), 1);
    verifier_la_mesure_gardee(&None, givre_mesurer(), "DLA au temps 0");

    let mut amas = Amas::new(1, 200);
    let mut n0 = 0;
    while n0 == 0 {
        amas.ajouter();
        if etendue(amas.sites()) >= 72 {
            n0 = amas.sites().len() as u32;
        }
    }
    assert!(n0 > 100, "l'amas de la graine 1 atteint l'étendue 72 à {n0} sites");
    for pas in [1, 20, n0 - 2] {
        assert_eq!(givre_demarrer(0, 1), 1);
        assert_eq!(givre_avancer(pas), pas);
        verifier_la_mesure_gardee(&None, givre_mesurer(), &format!("{} sites, sous l'étendue 72", pas + 1));
    }
    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_avancer(n0 - 1), n0 - 1);
    assert_eq!(givre_sites(), n0);
    let attendue = mesure_native(1, n0 - 1);
    assert_eq!(attendue.as_ref().map(|m| m.rayons.len()), Some(6));
    let points = givre_mesurer();
    verifier_la_mesure_gardee(&attendue, points, &format!("{n0} sites, juste à l'étendue 72"));
}

/// juste : la mesure gardée appartient à l'amas qu'elle a mesuré (V-Z1-25). Après `givre_mesurer` sur la graine 1 à 1500 particules : `givre_avancer(0)`
/// (qui ne joue rien) ne l'efface pas ; `givre_avancer(1)` l'efface (rayon 0 vaut 0, valeur et dimension `NaN`) ; un nouveau `givre_mesurer` mesure
/// l'amas de 1502 sites, bit pour bit comme la bibliothèque. Au bout du scénario (2999 particules), `givre_avancer(5)` ne joue rien (rend 0) et
/// ne l'efface pas. `givre_demarrer` (même modèle, même graine) l'efface, de même que `givre_demarrer` d'un autre modèle.
///
/// Ne couvre pas : l'état avant tout démarrage (voir `correlation-wasm.mjs`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_mesure_suit_l_etat() {
    let _verrou = verrou();
    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_avancer(1500), 1500);
    let points = givre_mesurer();
    let avant = mesure_native(1, 1500);
    assert!(avant.is_some() && points > 0, "la graine 1 a une mesure à 1500 particules");
    verifier_la_mesure_gardee(&avant, points, "après 1500 particules");

    assert_eq!(givre_avancer(0), 0);
    verifier_la_mesure_gardee(&avant, points, "après givre_avancer(0)");

    assert_eq!(givre_avancer(1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_avancer(1)");
    let points = givre_mesurer();
    verifier_la_mesure_gardee(&mesure_native(1, 1501), points, "1501 particules");
    assert!(points > 0);

    assert_eq!(givre_avancer(1498), 1498);
    let points = givre_mesurer();
    let fin = mesure_native(1, 2999);
    verifier_la_mesure_gardee(&fin, points, "2999 particules");
    assert_eq!(givre_avancer(5), 0, "le scénario est fini");
    verifier_la_mesure_gardee(&fin, points, "après un givre_avancer qui ne joue rien");

    assert_eq!(givre_demarrer(0, 1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_demarrer(0, 1)");
    assert_eq!(givre_avancer(1500), 1500);
    assert!(givre_mesurer() > 0);
    assert_eq!(givre_demarrer(1, 1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_demarrer d'un autre modèle");
}

/// régression : (la valeur de la doc) le plafond `MESURE_SITES_MAX` vaut 10 000 (V-Z1-26) ; il est au moins la taille de la DLA de la vitrine
/// (3000 sites : la page peut toujours se mesurer).
///
/// Ne couvre pas : un amas au-delà du plafond dans la page (la DLA de la vitrine s'arrête à 3000 sites : aucun modèle n'atteint le plafond),
/// ni le temps du module WebAssembly (non mesuré).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn le_plafond_de_la_mesure() {
    assert_eq!(MESURE_SITES_MAX, 10_000);
}

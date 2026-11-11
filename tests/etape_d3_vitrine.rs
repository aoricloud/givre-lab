//! Étape D3 : la vitrine côté natif, les modèles 4 à 7 (le claquage `η = 0` à `3`), la mesure de leur amas, et la courbe `D(η)` à la demande (huit fonctions
//! `givre_courbe_*`). Valeurs attendues : `references/etape-d3-valeurs.md`, identifiants `V-D3-xx`.
//!
//! Comme `etape_5_vitrine.rs` et `etape_z1_vitrine.rs`, on appelle les fonctions `givre_*` comme des fonctions Rust et on les compare à l'état **natif** construit en face
//! (`Claquage::new(graine, demi-côté de la table, η)`, `croitre_jusqu_a`, puis les calculs naïfs de l'étape D2, `essayer_point_de_la_courbe`…). Le module WebAssembly lui-même est
//! éprouvé dans `etape_d3_module.rs` (sous Node), la page dans `etape_d3_page.rs`.
//!
//! L'état de la vitrine est global au processus, celui de la courbe aussi, et les tests d'un fichier tournent en parallèle : chaque test prend le verrou du modèle
//! (`VERROU_MODELE`) ou de la courbe (`VERROU_COURBE`), ou les deux (toujours dans cet ordre), propres à ce fichier, et le garde jusqu'à la fin. Aucun test de ce fichier n'a de tolérance :
//! tout est une égalité exacte (entiers, bits des `f64`).
//!
//! Tant qu'une des huit fonctions `givre_courbe_*` de `src/vitrine.rs` a encore son corps `todo!()`, ce binaire de test **s'arrête d'un coup** au lieu de marquer un test en échec : une
//! panique ne peut pas traverser une `extern "C" fn` (le message est « panicked at src/vitrine.rs:<ligne> : not yet implemented », puis « panic in a function that cannot unwind » et
//! l'abandon du processus : `SIGABRT` sous Linux, `0xc0000409` sous Windows). C'est un échec, et la ligne citée dit quelle fonction reste à écrire ; les autres fichiers de test continuent
//! avec `--no-fail-fast`.

mod etape_d3_commun;

use std::sync::{Mutex, MutexGuard};

use etape_d3_commun::d2::{etendue_doc, point_naif, statistiques_naives};
use etape_d3_commun::{TABLE_CLAQUAGE, mesure_naive, natif_claquage, natif_depuis, repere_ecrit_a_la_main};
use givre_lab::claquage::Claquage;
use givre_lab::courbe::{demi_cote_de_la_courbe, essayer_dimension_d_une_graine, essayer_point_de_la_courbe, publie};
use givre_lab::vitrine::{
    CLAQUAGE_DEMI_COTE, CLAQUAGE_SITES, COURBE_GRAINES_MAX, COURBE_GRAINES_MIN, COURBE_TAILLE_MAX, givre_avancer,
    givre_compacite, givre_correlation_rayon, givre_correlation_valeur, givre_courbe_code, givre_courbe_demarrer,
    givre_courbe_erreur_type, givre_courbe_fait, givre_courbe_moyenne, givre_courbe_pas, givre_courbe_repere,
    givre_courbe_total, givre_demarrer, givre_dimension, givre_duree, givre_empreinte_mot, givre_hauteur,
    givre_largeur, givre_mesurer, givre_pixel, givre_rayon, givre_sites, givre_temps,
};

/// Les dimensions de `d2_ref.py amas <η> 1 <demi-côté> <sites>` (V-D3-35), arrondies au `f64` le plus proche.
const PYTHON_ETA_0: f64 = 1.8928113267749596;
const PYTHON_ETA_1: f64 = 1.7449358834101933;
const PYTHON_ETA_2: f64 = 1.5960740458888258;
const PYTHON_ETA_3: f64 = 1.3830393278712447;

static VERROU_MODELE: Mutex<()> = Mutex::new(());
static VERROU_COURBE: Mutex<()> = Mutex::new(());

/// Prend le verrou du modèle ; un test qui a paniqué en le tenant ne bloque pas les suivants.
fn verrou_modele() -> MutexGuard<'static, ()> {
    VERROU_MODELE.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Prend le verrou de la courbe.
fn verrou_courbe() -> MutexGuard<'static, ()> {
    VERROU_COURBE.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// Les huit mots de l'empreinte de la vitrine, chacun sur 8 chiffres hexadécimaux, dans l'ordre.
fn empreinte_de_la_vitrine() -> String {
    (0..8).map(|i| format!("{:08x}", givre_empreinte_mot(i))).collect()
}

/// Vérifie que la vitrine, dans son état courant, est identique à `attendu` sur tout ce qu'elle expose : temps, durée, dimensions de l'image, **chaque pixel**, sites, rayon, compacité
/// (`NaN`), chacun des huit mots de l'empreinte et l'empreinte entière.
fn verifier_identique(attendu: &etape_d3_commun::NatifClaquage, contexte: &str) {
    verifier_l_etat(attendu, contexte, true);
}

/// Comme `verifier_identique`, mais les pixels ne sont lus que si `avec_les_pixels` (lire une image de 512 × 512 pixels coûte 262 144 appels).
fn verifier_l_etat(attendu: &etape_d3_commun::NatifClaquage, contexte: &str, avec_les_pixels: bool) {
    assert_eq!(givre_temps(), attendu.temps, "{contexte} : temps");
    assert_eq!(givre_duree(), attendu.duree, "{contexte} : durée");
    assert_eq!(givre_largeur(), attendu.largeur, "{contexte} : largeur");
    assert_eq!(givre_hauteur(), attendu.hauteur, "{contexte} : hauteur");
    assert_eq!(givre_sites(), attendu.sites, "{contexte} : sites");
    assert_eq!(givre_rayon(), attendu.rayon, "{contexte} : rayon");
    assert!(givre_compacite().is_nan(), "{contexte} : la compacité du claquage est NaN");
    for (i, mot) in attendu.mots().iter().enumerate() {
        assert_eq!(givre_empreinte_mot(i as u32), *mot, "{contexte} : mot {i} de l'empreinte");
    }
    assert_eq!(empreinte_de_la_vitrine(), attendu.empreinte, "{contexte} : empreinte");
    if !avec_les_pixels {
        return;
    }
    for y in 0..attendu.hauteur {
        for x in 0..attendu.largeur {
            assert_eq!(givre_pixel(x, y), attendu.pixel(x, y), "{contexte} : pixel ({x}, {y})");
        }
    }
}

/// juste : la table des quatre modèles de claquage (V-D3-10). Pour `η = 0` à `3`, `givre_demarrer(4 + η, graine)` rend 1 et la vitrine est au temps 0 comme `Claquage::new(graine,
/// demi-côté de la table, η)` : la durée est 399, 249, 149, 99 (les sites visés 400, 250, 150, 100, moins le germe), 1 site, rayon 0, compacité `NaN`, une image de 512 × 512 pixels (un
/// site : `amas_colore`), chaque pixel et l'empreinte égaux à ceux du natif, pour les graines 1, `0xDEAD_BEEF` et `u32::MAX` (la graine est un `u32` entier, sans troncature ni signe : deux graines
/// donnent deux empreintes différentes). Les constantes publiques de la vitrine disent la même chose que la doc.
///
/// Ne couvre pas : l'état après des pas (voir `l_etat_suit_le_natif_par_morceaux`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_quatre_modeles_de_claquage_au_depart() {
    let _verrou = verrou_modele();
    for reglage in &TABLE_CLAQUAGE {
        let mut empreintes = Vec::new();
        for graine in [1_u32, 0xDEAD_BEEF, u32::MAX] {
            assert_eq!(givre_demarrer(reglage.numero, graine), 1, "modèle {}", reglage.numero);
            assert_eq!(givre_duree(), reglage.duree, "modèle {} : durée", reglage.numero);
            assert_eq!(givre_temps(), 0);
            assert_eq!(givre_sites(), 1);
            assert_eq!(givre_rayon(), 0);
            assert_eq!((givre_largeur(), givre_hauteur()), (512, 512));
            verifier_identique(
                &natif_claquage(reglage.eta, graine, 0),
                &format!("modèle {}, graine {graine}", reglage.numero),
            );
            empreintes.push(empreinte_de_la_vitrine());
        }
        empreintes.sort();
        empreintes.dedup();
        assert_eq!(empreintes.len(), 3, "modèle {} : la graine compte", reglage.numero);
    }
    // Les constantes de la vitrine sont celles de la table de la doc (régression de constantes).
    assert_eq!(CLAQUAGE_SITES, [400, 250, 150, 100]);
    assert_eq!(CLAQUAGE_DEMI_COTE, [70, 52, 49, 48]);
    for eta in 0..4_u32 {
        assert_eq!(
            CLAQUAGE_DEMI_COTE[eta as usize],
            demi_cote_de_la_courbe(eta, CLAQUAGE_SITES[eta as usize] as usize),
            "le demi-côté de η = {eta} est celui de la règle de D2"
        );
    }
}

/// juste : l'état des modèles 4 à 7 suit le natif, par morceaux (V-D3-11). Pour chaque `η`, la graine 7 jouée par morceaux de `1, 0, 6, 13, 0, 25` unités (45 sites en tout ; un morceau de 0
/// ne joue rien et rend 0) : après chaque morceau, `givre_avancer` rend le nombre d'unités demandé et le temps, la durée, l'image (chaque pixel), les sites, le rayon, la compacité, les
/// huit mots de l'empreinte et l'empreinte entière sont ceux de `Claquage::new(7, demi-côté, η)` joué de la même façon (les pixels : après 1, 20 et 45 unités seulement). Un appel de `n` unités puis de `m` donne l'état d'un appel de
/// `n + m` (la dynamique ne dépend pas du découpage : l'état final de ces morceaux est celui d'un seul appel de 45).
///
/// Ne couvre pas : le bout du scénario (voir `le_claquage_s_arrete_a_sa_duree`) ; les graines autres que 7 (voir la table au départ).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn l_etat_suit_le_natif_par_morceaux() {
    let _verrou = verrou_modele();
    for reglage in &TABLE_CLAQUAGE {
        assert_eq!(givre_demarrer(reglage.numero, 7), 1);
        let mut natif = Claquage::new(7, reglage.demi_cote, reglage.eta);
        let mut cumul = 0_u32;
        for morceau in [1_u32, 0, 6, 13, 0, 25] {
            let joue = givre_avancer(morceau);
            assert_eq!(joue, morceau, "modèle {}, morceau {morceau}", reglage.numero);
            cumul += morceau;
            natif.croitre_jusqu_a(1 + cumul as usize);
            // Les pixels sont lus après le premier morceau, après le quatrième (13 unités) et à la fin ; l'empreinte, qui dit tout l'état, après chacun.
            verifier_l_etat(
                &natif_depuis(&natif, reglage.eta),
                &format!("modèle {}, après {cumul} unités", reglage.numero),
                matches!(cumul, 1 | 20 | 45),
            );
        }
        let par_morceaux = empreinte_de_la_vitrine();
        assert_eq!(givre_demarrer(reglage.numero, 7), 1);
        assert_eq!(givre_avancer(45), 45);
        assert_eq!(empreinte_de_la_vitrine(), par_morceaux, "modèle {} : 45 unités d'un coup", reglage.numero);
    }
}

/// juste : le claquage s'arrête à sa durée (V-D3-12) : pour `η = 0` (400 sites) et `η = 3` (100 sites), `givre_avancer(u32::MAX)` rend la durée (399 puis 99), le temps est la durée, les sites
/// sont les sites visés, l'état entier est celui du natif joué jusqu'au bout, un `givre_avancer(1)` de plus rend 0 et ne change rien (l'empreinte est la même), et un appel de `n` unités à
/// 5 unités de la fin ne joue que 5 unités ; pour `η = 0` un seul appel de `u32::MAX` redonne la même empreinte. Un modèle dont la taille ne serait pas bornée à la table fait échouer la durée, les sites ou l'empreinte.
///
/// Ne couvre pas : `η = 1` et `η = 2` jusqu'au bout (2 à 5 s chacun en natif : voir `MODELE.md`, « Le coût ») ; leur durée est lue à la table au départ.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_claquage_s_arrete_a_sa_duree() {
    let _verrou = verrou_modele();
    for eta in [0_u32, 3] {
        let reglage = &TABLE_CLAQUAGE[eta as usize];
        assert_eq!(givre_demarrer(reglage.numero, 3), 1);
        assert_eq!(givre_avancer(reglage.duree - 5), reglage.duree - 5);
        assert_eq!(givre_avancer(u32::MAX), 5, "η = {eta} : les cinq dernières unités");
        assert_eq!(givre_temps(), reglage.duree);
        assert_eq!(givre_sites(), reglage.sites);
        verifier_identique(&natif_claquage(eta, 3, reglage.duree), &format!("η = {eta}, à la durée"));
        let fin = empreinte_de_la_vitrine();
        for n in [1, 10, u32::MAX] {
            assert_eq!(givre_avancer(n), 0, "η = {eta} : au-delà de la durée");
        }
        assert_eq!(givre_temps(), reglage.duree);
        assert_eq!(empreinte_de_la_vitrine(), fin);
        if eta == 0 {
            assert_eq!(givre_demarrer(reglage.numero, 3), 1);
            assert_eq!(givre_avancer(u32::MAX), reglage.duree, "η = {eta} : un appel de u32::MAX");
            assert_eq!(empreinte_de_la_vitrine(), fin);
        }
    }
}

/// juste : les modèles 4 à 7 ne laissent rien derrière eux, et les modèles 0 à 3 sont inchangés (V-D3-13). Un claquage joué, puis `givre_demarrer(0, 1)` et 2999 particules : l'empreinte est
/// celle de `references/dla-graine-1.empreinte` ; la plaque jouée jusqu'au bout après un claquage : celle de `references/reiter-plaque.empreinte` ; et un claquage démarré après la DLA a
/// l'état du natif au temps 0 (ni sites, ni image, ni empreinte de la DLA). Un modèle refusé (8) laisse le claquage en cours intact.
///
/// Ne couvre pas : les autres graines des modèles 0 à 3 (voir `etape_5_vitrine.rs`, qui reste vert).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn un_claquage_ne_laisse_rien_derriere_lui() {
    let _verrou = verrou_modele();
    assert_eq!(givre_demarrer(6, 2), 1);
    assert_eq!(givre_avancer(20), 20);
    assert_eq!(givre_demarrer(8, 2), 0, "le modèle 8 n'existe pas");
    verifier_identique(&natif_claquage(2, 2, 20), "le modèle 8 refusé ne change rien");

    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_avancer(2999), 2999);
    assert_eq!(empreinte_de_la_vitrine(), etape_d3_commun::e5::reference("dla-graine-1.empreinte"));

    assert_eq!(givre_demarrer(7, 1), 1);
    assert_eq!(givre_avancer(10), 10);
    assert_eq!(givre_demarrer(1, 1), 1);
    assert_eq!(givre_avancer(400), 400);
    assert_eq!(empreinte_de_la_vitrine(), etape_d3_commun::e5::reference("reiter-plaque.empreinte"));

    assert_eq!(givre_demarrer(5, 1), 1);
    verifier_identique(&natif_claquage(1, 1, 0), "un claquage après un flocon");
}

/// Vérifie que les accesseurs de la mesure disent exactement `attendue` (ou « pas de mesure » si `None`) après un `givre_mesurer` qui a rendu `points`.
fn verifier_la_mesure_gardee(attendue: &Option<(Vec<u32>, Vec<f64>, f64)>, points: u32, contexte: &str) {
    match attendue {
        Some((rayons, valeurs, dimension)) => {
            assert_eq!(points as usize, rayons.len(), "{contexte} : nombre de points");
            for (i, (rayon, valeur)) in rayons.iter().zip(valeurs).enumerate() {
                assert_eq!(givre_correlation_rayon(i as u32), *rayon, "{contexte} : rayon {i}");
                assert_eq!(givre_correlation_valeur(i as u32).to_bits(), valeur.to_bits(), "{contexte} : valeur {i}");
            }
            assert_eq!(givre_dimension().to_bits(), dimension.to_bits(), "{contexte} : dimension");
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

/// juste : la vitrine mesure l'amas de claquage comme la bibliothèque, par la fenêtre des petits amas (V-D3-14). Pour `η = 0` (graine 1, 400 sites), `η = 1` (graine 2, 60 sites), `η = 2`
/// (graine 3, 60 sites) et `η = 3` (graine 4, 99 unités, 100 sites) : `givre_mesurer()` rend le nombre de rayons de la fenêtre `2..=max(4, étendue / 6)` du calcul naïf de D2, `givre_correlation_rayon(i)` le
/// `i`-ième rayon (2, 3, 4, …), `givre_correlation_valeur(i)` son `C(r)` et `givre_dimension()` la dimension, tous égaux AU BIT PRÈS au calcul naïf (paires comptées une à une, dénominateur entier,
/// pente des moindres carrés dans l'ordre de la doc) ; un second `givre_mesurer()` sur le même état rend le même nombre ; hors de la courbe le rayon vaut 0 et la valeur `NaN`. La fenêtre
/// n'est pas celle de la DLA (qui commence à 4).
///
/// Ne couvre pas : « pas de mesure » (voir `pas_de_mesure_du_claquage`) ; la valeur physique de la dimension.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_vitrine_mesure_le_claquage_comme_la_bibliotheque() {
    let _verrou = verrou_modele();
    for (eta, graine, pas) in [(0_u32, 1_u32, 399_u32), (1, 2, 59), (2, 3, 59), (3, 4, 99)] {
        let reglage = &TABLE_CLAQUAGE[eta as usize];
        assert_eq!(givre_demarrer(reglage.numero, graine), 1);
        assert_eq!(givre_avancer(pas), pas);
        let natif = natif_claquage(eta, graine, pas);
        let attendue = mesure_naive(&natif.amas);
        let contexte = format!("η = {eta}, graine {graine}, {} sites", natif.sites);
        let (rayons, _, _) = attendue.as_ref().expect("la bibliothèque a une mesure");
        assert_eq!(rayons.first(), Some(&2), "{contexte} : la fenêtre commence à 2");
        let points = givre_mesurer();
        assert!(points > 0, "{contexte}");
        verifier_la_mesure_gardee(&attendue, points, &contexte);
        assert_eq!(givre_mesurer(), points, "{contexte} : un second appel");
        verifier_la_mesure_gardee(&attendue, points, &format!("{contexte} (second appel)"));
    }
}

/// juste : la dimension des quatre modèles, jouée jusqu'au bout avec la graine 1 puis mesurée, est celle d'un OUTIL INDÉPENDANT (V-D3-35). `d2_ref.py amas <η> 1 <demi-côté> <sites>` (Python 3.13 : le claquage de
/// `d1_ref.py`, écrit d'après la doc de `src/claquage.rs` seule, et la fenêtre, la corrélation en fractions exactes et la dimension en `decimal` à 60 chiffres de D2) donne `D` = 1,89281132677495956025
/// (modèle 4 : 400 sites, demi-côté 70, étendue 28), 1,74493588341019327296 (modèle 5 : 250 sites, 52, étendue 32), 1,59607404588882589411 (modèle 6 : 150 sites, 49, étendue 33) et 1,38303932787124472016 (modèle 7 : 100 sites, 48,
/// étendue 29). `givre_dimension()` s'en écarte de moins de `1e-12` (la précision de `math::logarithme` est `5e-16` en relatif ; la pente de 3 à 5 points ne l'amplifie pas), et `givre_sites()` est le
/// nombre de sites visés. C'est la preuve que la chaîne entière (le claquage de D1, la fenêtre de D2, la mesure de la vitrine) donne le nombre d'un autre programme.
///
/// Ne couvre pas : d'autres graines que la 1 (voir les tests au bit contre le calcul naïf) ; le module WebAssembly.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_dimension_des_quatre_modeles_est_celle_de_python() {
    let _verrou = verrou_modele();
    let python: [f64; 4] = [PYTHON_ETA_0, PYTHON_ETA_1, PYTHON_ETA_2, PYTHON_ETA_3];
    for reglage in &TABLE_CLAQUAGE {
        assert_eq!(givre_demarrer(reglage.numero, 1), 1);
        assert_eq!(givre_avancer(u32::MAX), reglage.duree);
        assert_eq!(givre_sites(), reglage.sites);
        assert!(givre_mesurer() > 0, "modèle {} : une mesure", reglage.numero);
        let obtenue = givre_dimension();
        let attendue = python[reglage.eta as usize];
        assert!(
            (obtenue - attendue).abs() <= 1e-12,
            "modèle {} : {obtenue} contre {attendue} (Python)",
            reglage.numero
        );
    }
}

/// juste : « pas de mesure » pour le claquage (V-D3-15). Au temps 0 (un site) : `givre_mesurer()` rend 0, le rayon 0 vaut 0, la valeur 0 et la dimension sont `NaN`. La frontière de l'amas trop
/// petit : pour `η = 0` (graine 1) et `η = 2` (graine 1), soit `n0` le plus petit nombre de sites dont l'étendue vaut au moins 8 (la plus petite étendue qui a une fenêtre : `e / 6 <= 4 <= e / 2`),
/// trouvé dans le test en faisant croître l'amas natif site par site : avec `n0 - 1` sites la vitrine rend 0, avec `n0` sites elle rend le nombre de points de la bibliothèque (3 : les rayons
/// 2 à 4). Les flocons restent sans mesure (voir `etape_z1_vitrine.rs`).
///
/// Ne couvre pas : le plafond de taille (10 000 sites : aucun modèle du claquage ne l'atteint).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn pas_de_mesure_du_claquage() {
    let _verrou = verrou_modele();
    for eta in [0_u32, 2] {
        let reglage = &TABLE_CLAQUAGE[eta as usize];
        assert_eq!(givre_demarrer(reglage.numero, 1), 1);
        assert_eq!(givre_sites(), 1);
        verifier_la_mesure_gardee(&None, givre_mesurer(), &format!("η = {eta}, au temps 0"));

        let mut claquage = Claquage::new(1, reglage.demi_cote, eta);
        let mut n0 = 0;
        while n0 == 0 {
            claquage.ajouter();
            if etendue_doc(claquage.sites()) >= 8 {
                n0 = claquage.sites().len() as u32;
            }
        }
        assert!((9..60).contains(&n0), "η = {eta} : l'étendue 8 est atteinte à {n0} sites");
        for pas in [1, n0 - 3] {
            assert_eq!(givre_demarrer(reglage.numero, 1), 1);
            assert_eq!(givre_avancer(pas), pas);
            verifier_la_mesure_gardee(
                &None,
                givre_mesurer(),
                &format!("η = {eta}, {} sites, sous l'étendue 8", pas + 1),
            );
        }
        assert_eq!(givre_demarrer(reglage.numero, 1), 1);
        assert_eq!(givre_avancer(n0 - 2), n0 - 2);
        assert_eq!(givre_sites(), n0 - 1);
        verifier_la_mesure_gardee(
            &None,
            givre_mesurer(),
            &format!("η = {eta}, {} sites, juste sous l'étendue 8", n0 - 1),
        );
        assert_eq!(givre_demarrer(reglage.numero, 1), 1);
        assert_eq!(givre_avancer(n0 - 1), n0 - 1);
        assert_eq!(givre_sites(), n0);
        let attendue = mesure_naive(claquage.sites());
        assert_eq!(attendue.as_ref().map(|m| m.0.len()), Some(3), "η = {eta} : la fenêtre 2 à 4");
        let points = givre_mesurer();
        verifier_la_mesure_gardee(&attendue, points, &format!("η = {eta}, {n0} sites, juste à l'étendue 8"));
    }
}

/// juste : la mesure gardée appartient à l'amas de claquage qu'elle a mesuré (V-D3-16), comme pour la DLA. Après `givre_mesurer` sur `η = 0` (graine 1) à 200 unités : `givre_avancer(0)`
/// ne l'efface pas ; `givre_avancer(1)` l'efface ; un nouveau `givre_mesurer` mesure l'amas de 202 sites, bit pour bit comme le calcul naïf. Au bout du scénario (399), `givre_avancer(5)` ne joue
/// rien et ne l'efface pas. `givre_demarrer` (même modèle, même graine) l'efface, de même que `givre_demarrer` d'un autre modèle.
///
/// Ne couvre pas : l'état avant tout démarrage (voir `etape_z1_vitrine.rs` et les scripts de Node).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_mesure_du_claquage_suit_l_amas() {
    let _verrou = verrou_modele();
    assert_eq!(givre_demarrer(4, 1), 1);
    assert_eq!(givre_avancer(200), 200);
    let avant = mesure_naive(&natif_claquage(0, 1, 200).amas);
    let points = givre_mesurer();
    assert!(avant.is_some() && points > 0);
    verifier_la_mesure_gardee(&avant, points, "après 200 unités");

    assert_eq!(givre_avancer(0), 0);
    verifier_la_mesure_gardee(&avant, points, "après givre_avancer(0)");

    assert_eq!(givre_avancer(1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_avancer(1)");
    let points = givre_mesurer();
    verifier_la_mesure_gardee(&mesure_naive(&natif_claquage(0, 1, 201).amas), points, "201 unités");
    assert!(points > 0);

    assert_eq!(givre_avancer(198), 198);
    let points = givre_mesurer();
    let fin = mesure_naive(&natif_claquage(0, 1, 399).amas);
    verifier_la_mesure_gardee(&fin, points, "399 unités");
    assert_eq!(givre_avancer(5), 0, "le scénario est fini");
    verifier_la_mesure_gardee(&fin, points, "après un givre_avancer qui ne joue rien");

    assert_eq!(givre_demarrer(4, 1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_demarrer(4, 1)");
    assert_eq!(givre_avancer(200), 200);
    assert!(givre_mesurer() > 0);
    assert_eq!(givre_demarrer(0, 1), 1);
    verifier_la_mesure_gardee(&None, 0, "après givre_demarrer d'un autre modèle");
}

/// Joue le travail de la courbe jusqu'au bout, un amas par `givre_courbe_pas`, et rend les `fait` lus après chaque pas.
fn jouer_la_courbe() -> Vec<u32> {
    let mut faits = Vec::new();
    while givre_courbe_fait() < givre_courbe_total() {
        assert_eq!(givre_courbe_pas(), 0, "un pas du travail");
        faits.push(givre_courbe_fait());
    }
    faits
}

/// juste : la courbe, un amas par appel, rend les points de la bibliothèque bit pour bit (V-D3-17). `givre_courbe_demarrer(40, 3)` rend 0 : `fait` vaut 0 et `total` 12 (4 `η` fois 3 graines) ;
/// chaque `givre_courbe_pas` rend 0 et fait **un** amas (`fait` croît de 1) ; la moyenne et l'erreur-type d'un `η` sont `NaN` tant que ses 3 amas ne sont pas faits (`η` de 0 à 3 dans cet ordre :
/// après le 3e, le 6e, le 9e et le 12e pas), puis, bit pour bit, ceux de `essayer_point_de_la_courbe(η, [1, 2, 3], 40, demi-côté de la règle)` (et, pour `η = 1` et `η = 3`, du calcul naïf de D2,
/// `point_naif`) ; un pas de plus sur un travail fini rend 0 et ne change rien ; les moyennes et erreurs-types de `η` de 4 à `u32::MAX` sont `NaN` ; `givre_courbe_code()` rend 0 tout du long. Un second travail à la **borne** (100 sites, 2 graines) joue ses quatre premiers
/// pas : `η = 0` et `η = 1` sont finis et égaux, au bit, à la bibliothèque, `η = 2` et `η = 3` pas encore.
///
/// Ne couvre pas : les codes d'erreur (voir `les_codes_d_erreur_de_la_courbe`) ; plus de 100 sites ; `η = 2` et `η = 3` à 100 sites par le travail (un amas de `η = 3` à 100 sites coûte 0,5 à 1,5 s : voir
/// `la_dimension_des_quatre_modeles_est_celle_de_python`, qui joue le modèle 7 à 100 sites, et `les_graines_et_les_eta_dans_l_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_courbe_egale_la_bibliotheque_bit_a_bit() {
    let _verrou = verrou_courbe();
    // (taille, graines, nombre de pas joués)
    for (taille, graines, pas_joues) in [(40_u32, 3_u32, 12_u32), (100, 2, 4)] {
        assert_eq!(givre_courbe_demarrer(taille, graines), 0);
        assert_eq!((givre_courbe_fait(), givre_courbe_total(), givre_courbe_code()), (0, 4 * graines, 0));
        for eta in 0..4 {
            assert!(givre_courbe_moyenne(eta).is_nan() && givre_courbe_erreur_type(eta).is_nan(), "η = {eta}, avant");
        }
        let liste: Vec<u64> = (1..=u64::from(graines)).collect();
        for pas in 1..=pas_joues {
            assert_eq!(givre_courbe_pas(), 0, "pas {pas}");
            assert_eq!(givre_courbe_fait(), pas, "pas {pas} : un amas de plus");
            assert_eq!(givre_courbe_code(), 0);
            for eta in 0..4 {
                let fini = pas >= (eta + 1) * graines;
                assert_eq!(!givre_courbe_moyenne(eta).is_nan(), fini, "pas {pas}, η = {eta} : moyenne");
                assert_eq!(!givre_courbe_erreur_type(eta).is_nan(), fini, "pas {pas}, η = {eta} : erreur-type");
            }
        }
        for eta in 0..4_u32 {
            if pas_joues < (eta + 1) * graines {
                continue;
            }
            let demi_cote = etape_d3_commun::demi_cote_regle(eta, taille as usize);
            let point = essayer_point_de_la_courbe(eta, &liste, taille as usize, demi_cote).expect("un point");
            assert_eq!(
                givre_courbe_moyenne(eta).to_bits(),
                point.moyenne.to_bits(),
                "taille {taille}, η = {eta} : moyenne"
            );
            assert_eq!(
                givre_courbe_erreur_type(eta).to_bits(),
                point.erreur_type.to_bits(),
                "taille {taille}, η = {eta} : erreur-type"
            );
            if taille == 40 && matches!(eta, 1 | 3) {
                let (moyenne, erreur_type) = point_naif(eta, &liste, taille as usize, demi_cote);
                assert_eq!(givre_courbe_moyenne(eta).to_bits(), moyenne.to_bits(), "η = {eta} : moyenne naïve");
                assert_eq!(
                    givre_courbe_erreur_type(eta).to_bits(),
                    erreur_type.to_bits(),
                    "η = {eta} : erreur-type naïve"
                );
            }
        }
        if pas_joues == 4 * graines {
            let moyenne_zero = givre_courbe_moyenne(0).to_bits();
            assert_eq!(givre_courbe_pas(), 0, "un pas de plus sur un travail fini");
            assert_eq!((givre_courbe_fait(), givre_courbe_total()), (4 * graines, 4 * graines));
            assert_eq!(givre_courbe_moyenne(0).to_bits(), moyenne_zero);
            // Hors de `η` de 0 à 3 : `NaN`, pas la valeur d'un autre `η` (un indice pris modulo 4, par exemple).
            for hors in [4_u32, 5, 7, 8, 100, u32::MAX] {
                assert!(givre_courbe_moyenne(hors).is_nan(), "η = {hors} : moyenne");
                assert!(givre_courbe_erreur_type(hors).is_nan(), "η = {hors} : erreur-type");
            }
        }
    }
}

/// juste : les graines d'un `η` sont 1, 2, … dans l'ordre, et les `η` se font dans l'ordre 0, 1, 2, 3 (V-D3-18). Avec 2 graines à 40 sites : `η = 0` est fini après exactement 2 pas (`η = 1` pas encore),
/// et sa moyenne est celle des graines `[1, 2]` du calcul naïf, qui n'est ni celle de `[0, 1]` ni celle de `[2, 3]` (le test vérifie que ces bits diffèrent avant de s'en servir) ; à la fin les
/// quatre points sont ceux des graines `[1, 2]`. Avec 8 graines à 36 sites : le travail donne, bit pour bit, les statistiques de D2 des huit dimensions de `essayer_dimension_d_une_graine` (graines 1 à 8), dont la moyenne faite dans l'ordre
/// inverse (`[8, …, 1]`) a d'autres bits pour au moins un `η` (la somme de gauche à droite : le test le vérifie avant de s'en servir) : l'ordre des graines est celui de la doc. Un amas à la fois :
/// le total joué pour un point est exactement `graines` pas.
///
/// Ne couvre pas : plus de 8 graines (la borne) ; les tailles autres que 36 et 40 (voir `la_courbe_egale_la_bibliotheque_bit_a_bit`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_graines_et_les_eta_dans_l_ordre() {
    let _verrou = verrou_courbe();
    assert_eq!(givre_courbe_demarrer(40, 2), 0);
    for pas in 1..=2 {
        assert_eq!(givre_courbe_pas(), 0);
        assert_eq!(givre_courbe_fait(), pas);
    }
    // η = 0 est fini après exactement 2 pas, η = 1 pas encore.
    assert!(!givre_courbe_moyenne(0).is_nan());
    assert!(givre_courbe_moyenne(1).is_nan());
    let attendu = point_naif(0, &[1, 2], 40, etape_d3_commun::demi_cote_regle(0, 40));
    assert_eq!(givre_courbe_moyenne(0).to_bits(), attendu.0.to_bits());
    for autres in [vec![0_u64, 1], vec![2, 3]] {
        let autre = point_naif(0, &autres, 40, etape_d3_commun::demi_cote_regle(0, 40));
        assert_ne!(autre.0.to_bits(), attendu.0.to_bits(), "les graines {autres:?} donnent un autre point que [1, 2]");
    }
    jouer_la_courbe();
    for eta in 0..4_u32 {
        let point = point_naif(eta, &[1, 2], 40, etape_d3_commun::demi_cote_regle(eta, 40));
        assert_eq!(givre_courbe_moyenne(eta).to_bits(), point.0.to_bits(), "η = {eta}");
        assert_eq!(givre_courbe_erreur_type(eta).to_bits(), point.1.to_bits(), "η = {eta}");
    }

    // Huit graines : l'ordre de la somme se voit.
    assert_eq!(givre_courbe_demarrer(36, 8), 0);
    jouer_la_courbe();
    let mut un_ordre_se_voit = false;
    for eta in 0..4_u32 {
        let demi_cote = etape_d3_commun::demi_cote_regle(eta, 36);
        // Les huit dimensions, une fois ; les statistiques de D2 dans l'ordre des graines (celles du travail) et dans l'ordre inverse.
        let mut dimensions: Vec<f64> = (1..=8_u64)
            .map(|graine| essayer_dimension_d_une_graine(eta, graine, 36, demi_cote).expect("une dimension"))
            .collect();
        let (moyenne, erreur_type) = statistiques_naives(&dimensions);
        assert_eq!(givre_courbe_moyenne(eta).to_bits(), moyenne.to_bits(), "η = {eta} : moyenne des 8 graines");
        assert_eq!(
            givre_courbe_erreur_type(eta).to_bits(),
            erreur_type.to_bits(),
            "η = {eta} : erreur-type des 8 graines"
        );
        dimensions.reverse();
        let inverse = statistiques_naives(&dimensions);
        un_ordre_se_voit |= inverse.0.to_bits() != moyenne.to_bits() || inverse.1.to_bits() != erreur_type.to_bits();
    }
    assert!(
        un_ordre_se_voit,
        "l'ordre inverse des graines donne d'autres bits pour au moins un η (sinon ce test serait sans pouvoir)"
    );
}

/// juste : les codes d'erreur de la courbe (V-D3-19), table de la doc. `givre_courbe_demarrer(taille, graines)` : `taille` hors de 1 à 100 → 1 (0, 101, `u32::MAX`), `graines` hors de 2 à 8 → 2 (0, 1, 9,
/// `u32::MAX`), la taille avant les graines ((0, 0) et (101, 1) → 1) ; chaque refus laisse **aucun travail** (`fait` 0, `total` 0, moyenne `NaN`, `givre_courbe_code` égal au code rendu) et un
/// `givre_courbe_pas` rend alors 5 (code 5, sans travail) ; les bornes sont incluses ((100, 8) → 0 avec `total` 32 ; (1, 2) → 0 avec `total` 8). Un amas trop petit : (5, 3) accepté, le premier
/// `givre_courbe_pas` rend 4 (fenêtre vide), `fait` reste 0, `givre_courbe_code` vaut 4, et tout `givre_courbe_pas` de plus rend 4 sans rien faire (le travail est arrêté) ; (1, 2) de même.
///
/// Ne couvre pas : les codes 3 et 6 (une frontière touchée, une erreur de paramètre : inatteignables avec les bornes de la vitrine, voir `MODELE.md`) ; la bibliothèque (voir `etape_d3_erreurs.rs`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_codes_d_erreur_de_la_courbe() {
    let _verrou = verrou_courbe();
    assert_eq!((COURBE_TAILLE_MAX, COURBE_GRAINES_MIN, COURBE_GRAINES_MAX), (100, 2, 8));
    let refus: [(u32, u32, u32); 11] = [
        (0, 4, 1),
        (101, 4, 1),
        (1000, 4, 1),
        (u32::MAX, 4, 1),
        (40, 0, 2),
        (40, 1, 2),
        (40, 9, 2),
        (40, u32::MAX, 2),
        (0, 0, 1),
        (101, 1, 1),
        (u32::MAX, u32::MAX, 1),
    ];
    for (taille, graines, code) in refus {
        // Un travail en cours est effacé même par un refus.
        assert_eq!(givre_courbe_demarrer(10, 2), 0);
        assert_eq!(givre_courbe_demarrer(taille, graines), code, "demarrer({taille}, {graines})");
        assert_eq!(
            (givre_courbe_fait(), givre_courbe_total()),
            (0, 0),
            "demarrer({taille}, {graines}) : aucun travail"
        );
        assert_eq!(givre_courbe_code(), code, "demarrer({taille}, {graines}) : le dernier code");
        assert!(givre_courbe_moyenne(0).is_nan());
        assert_eq!(givre_courbe_pas(), 5, "un pas sans travail");
        assert_eq!(givre_courbe_code(), 5);
        assert_eq!(givre_courbe_fait(), 0);
    }
    assert_eq!(givre_courbe_demarrer(100, 8), 0);
    assert_eq!((givre_courbe_fait(), givre_courbe_total(), givre_courbe_code()), (0, 32, 0));
    for (taille, graines) in [(5_u32, 3_u32), (1, 2)] {
        assert_eq!(givre_courbe_demarrer(taille, graines), 0);
        assert_eq!((givre_courbe_fait(), givre_courbe_total()), (0, 4 * graines));
        for _ in 0..3 {
            assert_eq!(givre_courbe_pas(), 4, "taille {taille} : une fenêtre vide");
            assert_eq!(givre_courbe_code(), 4);
            assert_eq!(givre_courbe_fait(), 0, "taille {taille} : le travail est arrêté");
            assert!(givre_courbe_moyenne(0).is_nan());
        }
    }
}

/// juste : un nouveau `givre_courbe_demarrer` efface l'ancien travail (V-D3-20). Après 5 pas sur 12 (η = 0 fini), `givre_courbe_demarrer(40, 3)` remet `fait` à 0, `total` à 12, le code à 0 et la
/// moyenne de η = 0 à `NaN` ; le travail refait donne les mêmes bits qu'avant ; un démarrage avec d'autres paramètres (30, 2) change `total` (8). Un travail arrêté par une erreur est effacé
/// de même.
///
/// Ne couvre pas : l'indépendance du modèle (voir `le_modele_et_la_courbe_sont_independants`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn un_nouveau_demarrage_efface_le_travail() {
    let _verrou = verrou_courbe();
    assert_eq!(givre_courbe_demarrer(40, 3), 0);
    for _ in 0..5 {
        assert_eq!(givre_courbe_pas(), 0);
    }
    assert_eq!(givre_courbe_fait(), 5);
    let premiere_moyenne = givre_courbe_moyenne(0).to_bits();
    assert!(!givre_courbe_moyenne(0).is_nan());
    assert_eq!(givre_courbe_demarrer(40, 3), 0);
    assert_eq!((givre_courbe_fait(), givre_courbe_total(), givre_courbe_code()), (0, 12, 0));
    assert!(givre_courbe_moyenne(0).is_nan() && givre_courbe_erreur_type(0).is_nan());
    jouer_la_courbe();
    assert_eq!(givre_courbe_moyenne(0).to_bits(), premiere_moyenne, "le même travail refait donne les mêmes bits");
    assert_eq!(givre_courbe_demarrer(30, 2), 0);
    assert_eq!((givre_courbe_fait(), givre_courbe_total()), (0, 8));
    assert_eq!(givre_courbe_demarrer(5, 2), 0);
    assert_eq!(givre_courbe_pas(), 4);
    assert_eq!(givre_courbe_demarrer(40, 2), 0);
    assert_eq!((givre_courbe_fait(), givre_courbe_total(), givre_courbe_code()), (0, 8, 0));
    assert_eq!(givre_courbe_pas(), 0, "le travail arrêté a été effacé");
}

/// juste : le modèle en cours et la courbe sont indépendants (V-D3-21). Un travail à 5 pas sur 12, puis `givre_demarrer(0, 1)` et `givre_avancer(10)` : `fait` vaut toujours 5 et les 7 pas qui restent
/// donnent les points de la bibliothèque ; un `givre_courbe_pas` ne change ni le temps, ni l'empreinte, ni l'image du modèle (la DLA à 10 unités, puis le claquage `η = 3` à 20).
///
/// Ne couvre pas : deux fils d'exécution (il n'y en a pas dans le module).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_modele_et_la_courbe_sont_independants() {
    let _verrou_m = verrou_modele();
    let _verrou_c = verrou_courbe();
    assert_eq!(givre_courbe_demarrer(40, 3), 0);
    for _ in 0..5 {
        assert_eq!(givre_courbe_pas(), 0);
    }
    assert_eq!(givre_demarrer(0, 1), 1);
    assert_eq!(givre_avancer(10), 10);
    assert_eq!(givre_courbe_fait(), 5, "givre_demarrer ne touche pas à la courbe");
    let empreinte = empreinte_de_la_vitrine();
    let (temps, sites) = (givre_temps(), givre_sites());
    assert_eq!(givre_courbe_pas(), 0);
    assert_eq!(
        (givre_temps(), givre_sites(), empreinte_de_la_vitrine()),
        (temps, sites, empreinte),
        "un pas de la courbe ne touche pas au modèle"
    );
    assert_eq!(givre_demarrer(7, 2), 1);
    assert_eq!(givre_avancer(20), 20);
    jouer_la_courbe();
    verifier_identique(&natif_claquage(3, 2, 20), "le claquage joué pendant la courbe");
    for eta in 0..4_u32 {
        let point = essayer_point_de_la_courbe(eta, &[1, 2, 3], 40, etape_d3_commun::demi_cote_regle(eta, 40))
            .expect("un point");
        assert_eq!(givre_courbe_moyenne(eta).to_bits(), point.moyenne.to_bits(), "η = {eta}");
        assert_eq!(givre_courbe_erreur_type(eta).to_bits(), point.erreur_type.to_bits(), "η = {eta}");
    }
}

/// juste : les repères viennent de la bibliothèque (V-D3-22). `givre_courbe_repere(η)` rend, au bit près, `courbe::publie(η)` ET les valeurs transcrites à la main (1,71, 1,433, 1,263) pour
/// `η = 1, 2, 3`, et `NaN` pour `η = 0`, 4, 5, 100 et `u32::MAX` ; il ne dépend d'aucun travail (le même avant, pendant et après un travail refusé). Et le texte de `src/vitrine.rs` (hors commentaires)
/// n'écrit aucune de ces valeurs : ni `1.71`, ni `1.433`, ni `1.263` (forme : un repère écrit en dur dans le module se verrait aussi à un changement de `publie`, que ce test ne simule pas).
///
/// Ne couvre pas : la page (voir `etape_d3_page.rs`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_reperes_viennent_de_la_bibliotheque() {
    let _verrou = verrou_courbe();
    let verifier = |contexte: &str| {
        for eta in [1_u32, 2, 3] {
            let rendu = givre_courbe_repere(eta);
            assert_eq!(Some(rendu.to_bits()), publie(eta).map(f64::to_bits), "{contexte} : η = {eta} (publie)");
            assert_eq!(
                Some(rendu.to_bits()),
                repere_ecrit_a_la_main(eta).map(f64::to_bits),
                "{contexte} : η = {eta} (transcrit à la main)"
            );
        }
        for eta in [0_u32, 4, 5, 100, u32::MAX] {
            assert!(givre_courbe_repere(eta).is_nan(), "{contexte} : η = {eta} n'a pas de repère");
        }
    };
    verifier("sans travail");
    assert_eq!(givre_courbe_demarrer(0, 4), 1);
    verifier("après un démarrage refusé");
    assert_eq!(givre_courbe_demarrer(40, 2), 0);
    verifier("avec un travail");
    // La forme du texte : aucune valeur publiée écrite dans le module.
    let source = std::fs::read_to_string(etape_d3_commun::racine().join("src").join("vitrine.rs"))
        .expect("src/vitrine.rs se lit");
    for ligne in source.lines() {
        let code = ligne.split("//").next().unwrap_or("");
        for interdit in ["1.71", "1.433", "1.263", "1,71", "1,433", "1,263"] {
            assert!(!code.contains(interdit), "src/vitrine.rs écrit une valeur publiée « {interdit} » : {ligne}");
        }
    }
    assert!(source.contains("publie("), "le repère de la courbe vient de crate::courbe::publie");
}

/// juste : aucun paramètre absurde ne fait paniquer une fonction de la courbe ou du claquage (V-D3-23) — une panique abattrait ce binaire de test, donc l'échec est l'abandon du processus. Les huit
/// fonctions de la courbe, sans travail, avec un travail et avec un travail refusé, pour des `η` de 0 à 3, 4, 100 et `u32::MAX` ; `givre_correlation_*` et `givre_dimension` sans mesure ; `givre_demarrer`
/// de 8, 9, 100 et `u32::MAX`, de graine `u32::MAX` ; `givre_avancer(u32::MAX)` sur un claquage et sans rien démarrer en route ; des dizaines de `givre_courbe_pas` de trop.
///
/// Ne couvre pas : les valeurs rendues (voir les tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn aucun_parametre_absurde_ne_fait_paniquer() {
    let _verrou_m = verrou_modele();
    let _verrou_c = verrou_courbe();
    for _ in 0..2 {
        for (taille, graines) in [(0_u32, 0_u32), (40, 2), (u32::MAX, u32::MAX), (5, 2)] {
            let _ = givre_courbe_demarrer(taille, graines);
            for _ in 0..12 {
                let _ = givre_courbe_pas();
            }
            for eta in [0_u32, 1, 2, 3, 4, 100, u32::MAX] {
                let _ = givre_courbe_moyenne(eta);
                let _ = givre_courbe_erreur_type(eta);
                let _ = givre_courbe_repere(eta);
            }
            let _ = (givre_courbe_fait(), givre_courbe_total(), givre_courbe_code());
        }
    }
    for modele in [8_u32, 9, 100, u32::MAX] {
        assert_eq!(givre_demarrer(modele, u32::MAX), 0);
    }
    assert_eq!(givre_demarrer(4, u32::MAX), 1);
    let _ = givre_mesurer();
    let _ = (givre_correlation_rayon(u32::MAX), givre_correlation_valeur(u32::MAX), givre_dimension());
    assert_eq!(givre_avancer(u32::MAX), 399);
    let _ = givre_dimension();
}

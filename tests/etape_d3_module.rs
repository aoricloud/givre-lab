//! Étape D3 : le module commité `docs/givre.wasm`, lu par Node, donne ce que donne le code natif pour le claquage et la courbe `D(η)`. Valeurs attendues : `references/etape-d3-valeurs.md`,
//! identifiants `V-D3-xx`.
//!
//! Comme à l'étape Z1, **ces tests lisent `docs/givre.wasm` : ils ne passent qu'une fois le module reconstruit** avec les vingt-trois fonctions (la commande est dans la doc de
//! `src/vitrine.rs`, après l'activation de `etape-d3` dans `Cargo.toml` : un module reconstruit avant l'activation n'est pas celui que la reconstruction propre redonne). Ils sont sautés quand Node est
//! absent hors de la CI (`node_est_la`) ; la CI de Linux et de Windows a Node, et un test sauté y est un échec (`CI` définie).
//!
//! Deux scripts, dans `tests/etape_d3/` : `modeles-wasm.mjs` fait démarrer, avancer et mesurer un modèle de claquage et le fait lire par `lecture.js` (`lireEmpreinte`, `lireImage`,
//! `lireCorrelation`) ; `courbe-wasm.mjs` fait calculer la courbe un amas à la fois et la fait lire par `lireCourbe`. Chacun écrit une ligne `clé=valeur` par grandeur (leur en-tête dit
//! lesquelles) ; ces tests comparent la sortie, **entière**, à celle que le code natif donne (`etape_d3_commun::attendu_du_script_des_modeles`, `attendu_du_script_de_la_courbe`).
//!
//! Étiquettes : les deux tests de lecture sont **justes** (comparés au code natif et au calcul naïf de D2) ; les deux autres sont de la **forme**.

mod etape_d3_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_d3_commun::{
    attendu_du_script_de_la_courbe, attendu_du_script_des_modeles, chemin_de_lecture, chemin_du_module, e5,
    lancer_le_script, racine,
};

/// juste : le module commité, lu par `lecture.js` sous Node, donne l'état et la mesure du code natif pour les modèles de claquage (V-D3-24). Pour chaque cas, le script démarre le modèle, joue
/// `pas` unités d'un seul appel, appelle `givre_mesurer` une fois et écrit ce qu'il lit ; ce test compare cette sortie, ligne par ligne et en entier, à celle du code natif
/// (`Claquage::new(graine, demi-côté de la table, η)` puis le calcul **naïf** de la mesure) : l'empreinte (celle de la page), le SHA-256 de l'image (chaque pixel), les dimensions, le temps, la
/// durée, les sites, le rayon, la compacité (`NaN`), le nombre de points, les rayons, chaque `C(r)` ET la dimension au bit près, le contrat « avant tout démarrage » lu sur un module neuf,
/// que `lireCorrelation` redit exactement les accesseurs, et ce que devient la mesure après un `givre_avancer(1)` de plus (effacée, sauf quand le scénario est fini). Les cas : `η = 0` jusqu'au bout
/// (399), `η = 1` à 60 (graine 2), `η = 2` à 45 (grande graine), `η = 3` jusqu'au bout (graine `u32::MAX`), `η = 3` à 0 pas (un site : pas de mesure), `η = 0` avec un nombre de pas qui dépasse
/// la durée (1000). **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
///
/// Ne couvre pas : la vitesse du module (voir le rapport de l'étape) ; ce qu'`app.js` fait de ce que rend `lecture.js` ; la DLA et les flocons (voir les tests des étapes 5 et Z1).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn node_lit_le_claquage_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let cas: [(u32, u32, u32); 6] =
        [(4, 1, 399), (5, 2, 60), (6, 0xDEAD_BEEF, 45), (7, u32::MAX, 99), (7, 3, 0), (4, 5, 1000)];
    for (modele, graine, pas) in cas {
        let arguments = [module.as_str(), lecture.as_str(), &modele.to_string(), &graine.to_string(), &pas.to_string()];
        let sortie = lancer_le_script("modeles-wasm.mjs", &arguments);
        let erreur = String::from_utf8_lossy(&sortie.stderr);
        assert!(
            sortie.status.success(),
            "modeles-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}",
            sortie.status
        );
        assert_eq!(
            String::from_utf8_lossy(&sortie.stdout),
            attendu_du_script_des_modeles(modele, graine, pas),
            "modèle {modele}, graine {graine}, {pas} pas"
        );
    }
}

/// juste : le module commité calcule la courbe `D(η)` un amas par appel et `lecture.js` (`lireCourbe`) la lit comme le code natif la donne (V-D3-25). Pour `taille = 40`, `graines = 3` (12 amas),
/// `taille = 36`, `graines = 8` (32 amas, le maximum) et `taille = 5`, `graines = 2` (un amas trop petit : le premier pas rend le code 4) : le script écrit les codes de refus de
/// `givre_courbe_demarrer` (taille hors de 1 à 100, graines hors de 2 à 8), l'état sans travail (`fait`, `total`, `code`, puis un pas qui rend 5), le prochain amas avant chaque pas (`η:graine`, dans
/// l'ordre `η` de 0 à 3 et graine de 1 à `graines` : le texte d'avancement de la page « η = 2, graine 3 sur 6 » en dépend), le code et le `fait` après chaque pas, la fin, les quatre points (moyenne,
/// erreur-type et repère, au bit près : ceux de `essayer_point_de_la_courbe`, avec le repère de `publie`), que `lireCourbe` redit exactement les accesseurs, les valeurs hors de portée (`NaN` pour `eta = 4` et
/// `u32::MAX`), un pas de plus et un nouveau démarrage qui efface. Ce test compare la sortie, ligne par ligne et en entier, à celle de la bibliothèque. **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
///
/// Ne couvre pas : le temps de chaque pas dans le module (un amas de 100 sites : voir le rapport de l'étape) ; la page.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn node_calcule_la_courbe_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    for (taille, graines) in [(40_u32, 3_u32), (36, 8), (5, 2)] {
        let arguments = [module.as_str(), lecture.as_str(), &taille.to_string(), &graines.to_string()];
        let sortie = lancer_le_script("courbe-wasm.mjs", &arguments);
        let erreur = String::from_utf8_lossy(&sortie.stderr);
        assert!(
            sortie.status.success(),
            "courbe-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}",
            sortie.status
        );
        assert_eq!(
            String::from_utf8_lossy(&sortie.stdout),
            attendu_du_script_de_la_courbe(taille, graines),
            "taille {taille}, {graines} graines"
        );
    }
}

/// forme : les deux scripts refusent des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard : fichier absent, fichier qui n'est
/// pas un module, `lecture.js` absent, trop ou trop peu d'arguments, modèle au-delà de 7 (`modeles-wasm.mjs`), nombres qui ne sont pas des entiers de 32 bits non signés (V-D3-26).
///
/// Ne couvre pas : les arguments justes (voir les deux tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_scripts_refusent_des_arguments_faux() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let (module, lecture) = (module.as_str(), lecture.as_str());
    let absent = racine().join("docs").join("n-existe-pas.wasm");
    let absent = absent.to_str().expect("chemin en UTF-8");
    let pas_un_module = racine().join("Cargo.toml");
    let pas_un_module = pas_un_module.to_str().expect("chemin en UTF-8");
    let modeles: [&[&str]; 14] = [
        &[absent, lecture, "4", "1", "80"],
        &[pas_un_module, lecture, "4", "1", "80"],
        &[module, "absent.js", "4", "1", "80"],
        &[],
        &[module, lecture],
        &[module, lecture, "4", "1"],
        &[module, lecture, "4", "1", "80", "en trop"],
        &[module, lecture, "8", "1", "80"],
        &[module, lecture, "abc", "1", "80"],
        &[module, lecture, "4", "-1", "80"],
        &[module, lecture, "4", "4294967296", "80"],
        &[module, lecture, "4", "1", "abc"],
        &[module, lecture, "4", "1", "-1"],
        &[module, lecture, "4", "1", ""],
    ];
    let courbes: [&[&str]; 10] = [
        &[absent, lecture, "40", "3"],
        &[pas_un_module, lecture, "40", "3"],
        &[module, "absent.js", "40", "3"],
        &[],
        &[module, lecture, "40"],
        &[module, lecture, "40", "3", "en trop"],
        &[module, lecture, "abc", "3"],
        &[module, lecture, "-1", "3"],
        &[module, lecture, "40", "4294967296"],
        &[module, lecture, "40", ""],
    ];
    for (script, cas) in [("modeles-wasm.mjs", &modeles[..]), ("courbe-wasm.mjs", &courbes[..])] {
        for arguments in cas {
            let sortie = lancer_le_script(script, arguments);
            assert_eq!(sortie.status.code(), Some(1), "{script}, arguments {arguments:?}");
            assert!(sortie.stdout.is_empty(), "{script}, arguments {arguments:?} : la sortie standard n'est pas vide");
            assert!(!sortie.stderr.is_empty(), "{script}, arguments {arguments:?} : aucun message d'erreur");
        }
    }
}

/// forme : les scripts de Node sont du JavaScript valide (modules) : `node --check`, le texte leur étant passé sur l'entrée standard. Le contrôle est sauté, comme les autres, quand Node est
/// absent hors de la CI.
///
/// Ne couvre pas : l'exécution des scripts (voir `node_lit_le_claquage_comme_le_natif` et `node_calcule_la_courbe_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_scripts_sont_du_javascript_valide() {
    if !e5::node_est_la() {
        return;
    }
    for nom in ["modeles-wasm.mjs", "courbe-wasm.mjs", "lecture-pure.mjs"] {
        let script =
            std::fs::read_to_string(racine().join("tests").join("etape_d3").join(nom)).expect("le script se lit");
        let mut node = Command::new("node")
            .args(["--input-type=module", "--check"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node se lance");
        node.stdin.take().expect("entrée standard de Node").write_all(script.as_bytes()).expect("écriture du script");
        let sortie = node.wait_with_output().expect("Node se termine");
        assert!(sortie.status.success(), "{nom} n'est pas valide : {}", String::from_utf8_lossy(&sortie.stderr));
    }
}

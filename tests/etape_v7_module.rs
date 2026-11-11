//! Étape V7 : le module commité `docs/givre.wasm`, lu par Node, donne l'image nette que donne le code natif. Valeurs attendues : `references/etape-v7-valeurs.md`, identifiants `V-V7-xx`.
//!
//! Comme aux étapes Z1 et D3, **ces tests lisent `docs/givre.wasm` : ils ne passent qu'une fois le module reconstruit** avec les vingt-sept fonctions (la commande est dans la doc de `src/vitrine.rs`, après l'activation de
//! `etape-v7` dans `Cargo.toml` : un module reconstruit avant l'activation n'est pas celui que la reconstruction propre redonne). Ils sont sautés quand Node est absent hors de la CI (`node_est_la`) ; la CI de Linux et de
//! Windows a Node, et un test sauté y est un échec (`CI` définie).
//!
//! Un script, dans `tests/etape_v7/` : `nette-wasm.mjs` fait démarrer, avancer et préparer l'image nette d'un modèle et la fait lire par `lecture.js` (`lireImageNette`, `lireImage`, `lireEmpreinte`) ; il écrit une ligne
//! `clé=valeur` par grandeur (son en-tête dit lesquelles) et ce test compare la sortie, **entière**, à celle que le code natif donne (`etape_v7_commun::attendu_du_script_nette` : l'état natif, et l'image de
//! `rendu_hex::rendre` appliqué au champ de la doc).
//!
//! Étiquettes : les deux tests de lecture sont **justes** (comparés au code natif et à `rendre`) ; les deux autres sont de la **forme**.

mod etape_v7_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_v7_commun::{attendu_du_script_nette, chemin_de_lecture, chemin_du_module, e5, lancer_le_script, racine};

/// juste : le module commité, lu par Node, donne l'image nette du code natif (V-V7-20). Pour chaque cas, le script démarre le modèle, joue `pas` unités d'un seul appel, prépare l'image (`lireImageNette` de `lecture.js`) et écrit ce qu'il lit ; ce
/// test compare cette sortie, ligne par ligne et en entier, à celle du code natif : **sur un module neuf** la préparation rend 2 et le tampon est vide ; après le démarrage, le tampon est vide avant toute préparation ; le code, les dimensions, le
/// SHA-256 de ce que `lireImageNette` rend (RGBA) et celui de ce que `givre_nette_pixel` lit pixel par pixel (R, V, B : chaque octet de l'image de `rendre` pour le champ de la doc) ; les cinq pixels hors image valent `u32::MAX` ; l'état
/// (temps, durée, sites, rayon, empreinte, image d'un pixel par case) est le même **avant et après** la préparation ; onze refus ont le code de la doc ; un refus efface le tampon ; un `givre_avancer(1)` l'efface (sauf scénario fini : il le laisse) ; un
/// `givre_demarrer` l'efface. Les cas : la plaque jusqu'au bout (`62`, `sur = 2`), la plaque au temps 0 (le germe), l'étoile à 37 pas (`96`, 3), l'étoile jusqu'au bout (`128`, 1), la dendrite à 500 pas avec la plus grande graine (`100`, 2), la dendrite
/// avec un nombre de pas qui dépasse la durée (`64`, 4), l'étoile aux réglages de l'animation de la page (`384`, 2), et trois modèles sur une grille carrée (la DLA à 150 particules, le claquage `η = 0` à 40 sites et `η = 2` à 30 : le code 1, rien dans le tampon,
/// l'image d'un pixel par case inchangée). **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
///
/// Ne couvre pas : la vitesse du module (voir le rapport de l'étape) ; ce qu'`app.js` fait de ce que rend `lecture.js` ; les grandes images (voir `node_lit_l_image_de_la_fin`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn node_lit_l_image_nette_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let cas: [(u32, u32, u32, u32, u32); 10] = [
        (1, 1, 400, 62, 2),
        (1, 7, 0, 62, 2),
        (2, 0, 37, 96, 3),
        (2, 5, 120, 128, 1),
        (3, u32::MAX, 500, 100, 2),
        (3, 1, 10_000, 64, 4),
        (2, 3, 60, 384, 2),
        (0, 1, 150, 64, 2),
        (4, 3, 40, 64, 2),
        (6, 2, 30, 62, 2),
    ];
    for (modele, graine, pas, cote, sur) in cas {
        let arguments = [
            module.as_str(),
            lecture.as_str(),
            &modele.to_string(),
            &graine.to_string(),
            &pas.to_string(),
            &cote.to_string(),
            &sur.to_string(),
        ];
        let sortie = lancer_le_script("nette-wasm.mjs", &arguments);
        let erreur = String::from_utf8_lossy(&sortie.stderr);
        assert!(
            sortie.status.success(),
            "nette-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}",
            sortie.status
        );
        assert_eq!(
            String::from_utf8_lossy(&sortie.stdout),
            attendu_du_script_nette(modele, graine, pas, cote, sur),
            "modèle {modele}, graine {graine}, {pas} pas, côté {cote}, sur {sur}"
        );
    }
}

/// juste : l'image de la fin de la page, lue par Node (V-V7-21). La dendrite jusqu'au bout (800 pas) à `1024` pixels et `sur = 3` (les constantes de `docs/app.js`), 9,4 millions d'échantillons : tout ce que dit `node_lit_l_image_nette_comme_le_natif`, pour la
/// plus grande image de la page (le script lit un million de pixels deux fois : par `lecture.js` et pixel par pixel). Le temps que cela prend sous Node est mesuré par le pilote (`temps-wasm.mjs`) : ce test n'a aucune minuterie.
///
/// Ne couvre pas : le temps ; les autres modèles à cette taille (voir `etape_v7_vitrine.rs`, en natif). **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn node_lit_l_image_de_la_fin() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let arguments = [module.as_str(), lecture.as_str(), "3", "0", "800", "1024", "3"];
    let sortie = lancer_le_script("nette-wasm.mjs", &arguments);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "nette-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu_du_script_nette(3, 0, 800, 1024, 3));
}

/// forme : le script refuse des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard (V-V7-22) : fichier absent, fichier qui n'est pas un module, `lecture.js` absent, trop ou trop peu
/// d'arguments, modèle qui n'est pas un entier de 0 à 7, graine, nombre de pas, côté ou suréchantillonnage qui n'est pas un entier de 32 bits non signé.
///
/// Ne couvre pas : les arguments justes (voir les tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_script_refuse_des_arguments_faux() {
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
    let cas: [&[&str]; 18] = [
        &[absent, lecture, "2", "1", "80", "62", "2"],
        &[pas_un_module, lecture, "2", "1", "80", "62", "2"],
        &[module, "absent.js", "2", "1", "80", "62", "2"],
        &[],
        &[module, lecture],
        &[module, lecture, "2", "1", "80", "62"],
        &[module, lecture, "2", "1", "80", "62", "2", "en trop"],
        &[module, lecture, "8", "1", "80", "62", "2"],
        &[module, lecture, "abc", "1", "80", "62", "2"],
        &[module, lecture, "2", "-1", "80", "62", "2"],
        &[module, lecture, "2", "4294967296", "80", "62", "2"],
        &[module, lecture, "2", "1", "abc", "62", "2"],
        &[module, lecture, "2", "1", "-1", "62", "2"],
        &[module, lecture, "2", "1", "80", "abc", "2"],
        &[module, lecture, "2", "1", "80", "-2", "2"],
        &[module, lecture, "2", "1", "80", "62", "abc"],
        &[module, lecture, "2", "1", "80", "62", "4294967296"],
        &[module, lecture, "2", "1", "80", "", "2"],
    ];
    for arguments in cas {
        let sortie = lancer_le_script("nette-wasm.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "nette-wasm.mjs, arguments {arguments:?}");
        assert!(
            sortie.stdout.is_empty(),
            "nette-wasm.mjs, arguments {arguments:?} : la sortie standard n'est pas vide"
        );
        assert!(!sortie.stderr.is_empty(), "nette-wasm.mjs, arguments {arguments:?} : aucun message d'erreur");
    }
}

/// forme : les scripts de Node sont du JavaScript valide (modules) : `node --check`, le texte leur étant passé sur l'entrée standard (V-V7-23). Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution des scripts (voir `node_lit_l_image_nette_comme_le_natif`, et `etape_v7_page.rs` pour `lecture-pure.mjs`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn les_scripts_sont_du_javascript_valide() {
    if !e5::node_est_la() {
        return;
    }
    for nom in ["nette-wasm.mjs", "lecture-pure.mjs"] {
        let script =
            std::fs::read_to_string(racine().join("tests").join("etape_v7").join(nom)).expect("le script se lit");
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

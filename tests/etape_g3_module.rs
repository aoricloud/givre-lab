//! Étape G3 : le module commité `docs/givre.wasm`, lu par Node, joue la neige que joue le code natif. Valeurs attendues : `references/etape-g3-valeurs.md`, identifiants `V-G3-xx`.
//!
//! Comme aux étapes Z1, D3 et V7, **ces tests lisent `docs/givre.wasm` : ils ne passent qu'une fois le module reconstruit** avec les trente-deux fonctions (la commande est dans la doc de `src/vitrine.rs`, après l'activation de
//! `etape-g3` dans `Cargo.toml` : un module reconstruit avant l'activation n'est pas celui que la reconstruction propre redonne). Ils sont sautés quand Node est absent hors de la CI (`node_est_la`) ; la CI de Linux et de
//! Windows a Node, et un test sauté y est un échec (`CI` définie).
//!
//! Un script, dans `tests/etape_g3/` : `neige-wasm.mjs` démarre une neige, la fait avancer, lit ses réglages, ses mesures et son image nette par `lecture.js` et par le module, et écrit une ligne `clé=valeur` par grandeur (son
//! en-tête dit lesquelles) ; ce test compare la sortie, **entière**, à celle que le code natif donne (`etape_g3_commun::attendu_du_script_neige` : la neige de la bibliothèque, ses mesures recoupées par le calcul naïf de G2, l'image de
//! `rendu_hex::rendre` appliqué au champ de la doc).
//!
//! Étiquettes : les trois tests de lecture sont **justes** (comparés au code natif, et pour le dernier au tableau de G2) ; les deux autres sont de la **forme**.

mod etape_g3_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_g3_commun::{
    JEUX, attendu_du_script_neige, chemin_de_lecture, chemin_du_module, e5, lancer_le_script, ligne_g2, quatre_du_jeu,
    racine,
};

/// Lance `neige-wasm.mjs` pour ce cas et rend sa sortie standard (le test échoue si le script échoue).
fn jouer(jeu: usize, quatre: [u32; 4], pas: u32, cote: u32, sur: u32) -> String {
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let arguments = [
        module.as_str(),
        lecture.as_str(),
        &jeu.to_string(),
        &quatre[0].to_string(),
        &quatre[1].to_string(),
        &quatre[2].to_string(),
        &quatre[3].to_string(),
        &pas.to_string(),
        &cote.to_string(),
        &sur.to_string(),
    ];
    let sortie = lancer_le_script("neige-wasm.mjs", &arguments);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "neige-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
    String::from_utf8_lossy(&sortie.stdout).into_owned()
}

/// juste : le module commité, lu par Node, joue la neige du code natif (V-G3-30). Pour chaque cas, le script lit d'abord le module **neuf** (les lecteurs de la neige rendent `u32::MAX`, la préparation rend 2, les dix-neuf jeux nommés se lisent sans rien démarrer), fait huit
/// démarrages refusés (codes 1, 1, 2, 3, 3, 4, 5, 2) qui ne démarrent rien, puis démarre la neige du jeu avec ses quatre réglages, la joue de `pas` pas d'un seul appel, lit l'état (temps, durée, sites, rayon, empreinte, image d'un pixel par case), les réglages (`lireReglagesNeige`), les mesures (`lireMesuresNeige` : les
/// six mesures, l'étiquette, « au mur ») et l'image nette (`lireImageNette` de `lecture.js`, puis le tampon pixel par pixel) ; ce test compare cette sortie, **ligne par ligne et en entier**, à celle du code natif : les dix-neuf jeux nommés (la table écrite à la main), l'état, les mesures, le SHA-256 de ce que `lireImageNette`
/// rend et celui de ce que `givre_nette_pixel` lit (chaque octet de l'image de `rendre` pour le champ de la doc, la vue de rayon 69 et le `plein` 1,5), l'état **avant et après** la préparation, un `givre_avancer(1)` qui efface le tampon (sauf neige arrêtée : il le laisse), un démarrage qui l'efface, un `givre_demarrer(1, 0)` après
/// lequel il n'y a plus de neige en cours, et `givre_demarrer(8, 0)` qui rend 0. Les cas : le jeu 5 (figure 9) à 120 pas en `(96, 3)`, le germe seul du jeu 8 à 0 pas, le milieu de la figure 13 (jeu 18) à 150 pas en `(128, 1)`, le jeu 11 à 200 pas en `(64, 4)`, des **réglages libres** (le jeu 3 avec `ρ = 0,7`, `β = 2`, `κ = 0,005`, `μ = 0,03`) à 100 pas
/// aux réglages de l'animation de la page `(384, 2)`, et une neige rapide qui **s'arrête au mur** (le jeu 1 avec `ρ = 0,956862`…) demandée à 1000 pas. **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
///
/// Ne couvre pas : la vitesse du module (voir le rapport de l'étape) ; ce qu'`app.js` fait de ce que rend `lecture.js` ; la neige jusqu'au bout (voir `node_joue_un_jeu_jusqu_au_bout`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn node_lit_la_neige_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let cas: [(usize, [u32; 4], u32, u32, u32); 6] = [
        (5, quatre_du_jeu(5), 120, 96, 3),
        (8, quatre_du_jeu(8), 0, 62, 2),
        (18, quatre_du_jeu(18), 150, 128, 1),
        (11, quatre_du_jeu(11), 200, 64, 4),
        (3, [700_000, 2_000_000, 5_000, 30_000], 100, 384, 2),
        (1, [956_862, 1_122_299, 48_396, 36_667], 1000, 62, 2),
    ];
    for (jeu, quatre, pas, cote, sur) in cas {
        assert_eq!(
            jouer(jeu, quatre, pas, cote, sur),
            attendu_du_script_neige(jeu, quatre, pas, cote, sur),
            "jeu {jeu}, réglages {quatre:?}, {pas} pas, côté {cote}, sur {sur}"
        );
    }
}

/// juste : un jeu joué jusqu'au bout sous Node est le cristal de G2 (V-G3-31). Le jeu 8 (figure 10, `β = 2,4`, 1300 pas) demandé à 100 000 pas : le script rend 1300 pas joués, l'état final est celui du code natif (même sortie entière que ci-dessus) **et** son
/// empreinte et ses mesures sont celles de la ligne du jeu dans `apercus/etape-g2-mesures.csv` (le tableau de G2, calculé sous Linux et sous Windows) : **le même cristal, à l'octet, dans le navigateur (le module sous Node), sous Linux et sous Windows**. Le temps que cela prend est mesuré
/// par le pilote : ce test n'a aucune minuterie.
///
/// Ne couvre pas : les autres jeux jusqu'au bout (leurs débuts sont dans `node_lit_la_neige_comme_le_natif`) ; le temps. **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn node_joue_un_jeu_jusqu_au_bout() {
    if !e5::node_est_la() {
        return;
    }
    let jeu = 8;
    let quatre = quatre_du_jeu(jeu);
    let sortie = jouer(jeu, quatre, 100_000, 62, 2);
    assert_eq!(sortie, attendu_du_script_neige(jeu, quatre, 100_000, 62, 2));
    let g2 = ligne_g2(jeu);
    let ligne = |cle: &str| -> String {
        sortie
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{cle}=")))
            .unwrap_or_else(|| panic!("pas de ligne « {cle} »"))
            .to_string()
    };
    assert_eq!(ligne("joue"), JEUX[jeu].pas.to_string(), "les pas joués");
    let etat = ligne("etat");
    let champs: Vec<&str> = etat.split(',').collect();
    assert_eq!(champs[4], g2.empreinte, "l'empreinte du cristal joué sous Node est celle du tableau de G2");
    let mesure: Vec<u32> = ligne("mesure").split(',').map(|v| v.parse().expect("un entier")).collect();
    assert_eq!(mesure[..6], g2.champs, "les six mesures du tableau de G2");
    assert_eq!(mesure[6], g2.forme, "l'étiquette du tableau de G2");
    assert_eq!(mesure[7], 0, "ce cristal ne touche pas le mur");
}

/// forme : le script refuse des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard (V-G3-32) : fichier absent, fichier qui n'est pas un module, `lecture.js` absent, trop ou trop peu d'arguments, et chacun des huit nombres (jeu, `ρ`,
/// `β`, `κ`, `μ`, pas, côté, suréchantillonnage) qui n'est pas un entier de 32 bits non signé (un texte, un signe, `4294967296`, la chaîne vide).
///
/// Ne couvre pas : les arguments justes (voir les tests précédents) ; un réglage hors plage (le script le rend dans les lignes `refus` et lève si le démarrage du cas est refusé).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
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
    let bons = ["5", "500000", "1300000", "3000", "70000", "100", "62", "2"];
    let avec = |position: usize, valeur: &'static str| -> Vec<&str> {
        let mut v: Vec<&str> = vec![module, lecture];
        v.extend(bons.iter().enumerate().map(|(i, b)| if i == position { valeur } else { *b }));
        v
    };
    let mut cas: Vec<Vec<&str>> = vec![
        [vec![absent, lecture], bons.to_vec()].concat(),
        [vec![pas_un_module, lecture], bons.to_vec()].concat(),
        [vec![module, "absent.js"], bons.to_vec()].concat(),
        vec![],
        vec![module, lecture],
        [vec![module, lecture], bons[..7].to_vec()].concat(),
        [vec![module, lecture], bons.to_vec(), vec!["en trop"]].concat(),
    ];
    for position in 0..8 {
        for mauvaise in ["abc", "-1", "4294967296", "", "1.5"] {
            cas.push(avec(position, mauvaise));
        }
    }
    for arguments in cas {
        let sortie = lancer_le_script("neige-wasm.mjs", &arguments);
        assert_eq!(sortie.status.code(), Some(1), "neige-wasm.mjs, arguments {arguments:?}");
        assert!(
            sortie.stdout.is_empty(),
            "neige-wasm.mjs, arguments {arguments:?} : la sortie standard n'est pas vide"
        );
        assert!(!sortie.stderr.is_empty(), "neige-wasm.mjs, arguments {arguments:?} : aucun message d'erreur");
    }
}

/// forme : les scripts de Node de la neige sont du JavaScript valide (modules) : `node --check`, le texte leur étant passé sur l'entrée standard (V-G3-33). Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution des scripts (voir `node_lit_la_neige_comme_le_natif`, et `etape_g3_page.rs` pour `neige-pure.mjs`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_scripts_de_la_neige_sont_du_javascript_valide() {
    if !e5::node_est_la() {
        return;
    }
    for nom in ["neige-wasm.mjs", "neige-pure.mjs"] {
        let script =
            std::fs::read_to_string(racine().join("tests").join("etape_g3").join(nom)).expect("le script se lit");
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

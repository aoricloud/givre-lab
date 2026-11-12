//! Étape V8 : le module commité `docs/givre.wasm`, lu par Node, donne les masses de la neige du code natif, et redessine l'image nette à partir d'elles. Valeurs attendues : `references/etape-v8-valeurs.md`, identifiants `V-V8-xx`.
//!
//! Comme aux étapes Z1, D3, V7 et G3, **ces tests lisent `docs/givre.wasm` : ils ne passent qu'une fois le module reconstruit** avec les trente-quatre fonctions (la commande est dans la doc de `src/vitrine.rs`, après l'activation de `etape-v8` dans
//! `Cargo.toml` : un module reconstruit avant l'activation n'est pas celui que la reconstruction propre redonne). Ils sont sautés quand Node est absent hors de la CI (`node_est_la`) ; la CI de Linux et de Windows a Node, et un test sauté y est un échec (`CI` définie).
//!
//! Deux scripts, dans `tests/etape_v8/` : `masses-wasm.mjs` démarre une neige, la joue, lit les masses une à une, vérifie les règles de l'instantané et redessine l'image nette à partir des masses par `reference-masses.mjs` (le rastériseur de la doc en JavaScript,
//! `BigInt`) ; ce test compare la sortie, **entière**, à celle du code natif.
//!
//! Étiquettes : les deux tests de lecture sont **justes** (comparés au code natif et au rendu de la bibliothèque) ; les autres sont de la **forme**.

mod etape_v8_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_v8_commun::g3::{JEUX, avancer_la_neige, image_nette_attendue, parametres_du_jeu, quatre_du_jeu};
use etape_v8_commun::{CASES, chemin_du_module, e5, lancer_le_script, masses_de, racine, somme_des_masses};
use givre_lab::empreinte::sha256_hex;
use givre_lab::neige::Neige;

/// `sha256,somme,non_nulles,max` des masses : ce que le script écrit pour un instantané.
fn resume(masses: &[u32]) -> String {
    let somme: u64 = masses.iter().map(|&m| u64::from(m)).sum();
    let non_nulles = masses.iter().filter(|&&m| m > 0).count();
    let max = masses.iter().max().copied().unwrap_or(0);
    format!("{},{somme},{non_nulles},{max}", somme_des_masses(masses))
}

/// Ce que `masses-wasm.mjs` doit écrire pour ce cas : la neige de la bibliothèque, jouée `pas` pas puis 37 de plus (au plus la durée du jeu, jusqu'au mur), ses masses (`valeurs_du_cristal >> 21`), et l'image de `rendre` appliqué au champ de la doc (l'état final).
fn attendu_du_script(jeu: usize, quatre: [u32; 4], pas: u32, cote: u32, sur: u32) -> String {
    let duree = JEUX[jeu].pas;
    let mut neige =
        Neige::essayer_new(JEUX[jeu].rayon, parametres_du_jeu(jeu, quatre[0], quatre[1], quatre[2], quatre[3]))
            .expect("une neige valide");
    let joue = avancer_la_neige(&mut neige, pas, duree);
    let premieres = masses_de(&neige);
    let de_plus = avancer_la_neige(&mut neige, joue + 37, duree);
    let secondes = masses_de(&neige);
    assert_eq!(secondes.len(), CASES);
    let image = sha256_hex(&image_nette_attendue(&neige, cote, sur));
    let absent = u32::MAX;
    [
        format!("avant=0,{absent},{absent},{absent},{absent}"),
        "demarre=0".to_string(),
        format!("non_preparee={absent},{absent}"),
        format!("joue={joue}"),
        format!("nombre={CASES}"),
        format!("masses={}", resume(&premieres)),
        format!("hors={absent},{absent},{absent},{absent},{absent},{absent}"),
        format!("joue_de_plus={de_plus}"),
        format!("gele={}", resume(&premieres)),
        format!("nombre_2={CASES}"),
        format!("masses_2={}", resume(&secondes)),
        "refus_garde=1,1".to_string(),
        format!("temps={},{duree}", joue + de_plus),
        "nette=0".to_string(),
        format!("pixels={image}"),
        format!("reference={image}"),
        "ecarts=0".to_string(),
        format!("redemarre=0,{absent},{absent}"),
        format!("apres_redemarre={CASES}"),
        format!("autre=1,{absent},{absent},0,{absent}"),
        "huit=0".to_string(),
    ]
    .join("\n")
        + "\n"
}

/// Lance `masses-wasm.mjs` pour ce cas et rend sa sortie standard (le test échoue si le script échoue).
fn jouer(jeu: usize, quatre: [u32; 4], pas: u32, cote: u32, sur: u32) -> String {
    let module = chemin_du_module();
    let arguments = [
        module.as_str(),
        &jeu.to_string(),
        &quatre[0].to_string(),
        &quatre[1].to_string(),
        &quatre[2].to_string(),
        &quatre[3].to_string(),
        &pas.to_string(),
        &cote.to_string(),
        &sur.to_string(),
    ];
    let sortie = lancer_le_script("masses-wasm.mjs", &arguments);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "masses-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
    String::from_utf8_lossy(&sortie.stdout).into_owned()
}

/// forme : le module exporte les deux fonctions des masses avec les bonnes signatures (V-V8-20) : `givre_neige_masses_preparer () -> i32` et `givre_neige_masse (i32) -> i32` ; avec `etape-v8` il exporte **exactement** les trente-quatre fonctions de la vitrine
/// (les trente-deux d'avant et ces deux-là) plus `memory`, sans importation ; sans elle, les trente-deux ou les trente-quatre. Les deux ne prennent et ne rendent que des `i32` (un `u32` de la doc).
///
/// Ne couvre pas : ce que les fonctions calculent (voir `node_lit_les_masses_comme_le_natif`) ; les autres tests de forme du module sont ceux de `etape_5_artefact.rs`.
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn le_module_exporte_les_deux_fonctions_des_masses() {
    let module = e5::analyser(&e5::lire_module());
    assert_eq!(module.importations, 0);
    let noms: std::collections::BTreeSet<&str> =
        module.exportations.iter().filter(|e| e.genre == 0).map(|e| e.nom.as_str()).collect();
    let toutes: Vec<&(&str, &[u8], &[u8])> = e5::FONCTIONS
        .iter()
        .chain(e5::FONCTIONS_Z1.iter())
        .chain(e5::FONCTIONS_D3.iter())
        .chain(e5::FONCTIONS_V7.iter())
        .chain(e5::FONCTIONS_G3.iter())
        .chain(e5::FONCTIONS_V8.iter())
        .collect();
    assert_eq!(toutes.len(), 34, "la liste écrite dans etape_5_commun a trente-quatre fonctions");
    assert_eq!(noms.len(), 34, "le module exporte {} fonctions : {noms:?}", noms.len());
    for (nom, parametres, resultats) in toutes {
        let exportation = module
            .exportations
            .iter()
            .find(|e| e.genre == 0 && e.nom == *nom)
            .unwrap_or_else(|| panic!("la fonction {nom} n'est pas exportée"));
        let type_ = *module.fonctions.get(exportation.indice as usize).expect("l'indice de la fonction");
        let (obtenus, rendus) = module.types.get(type_ as usize).expect("le type");
        assert_eq!(obtenus.as_slice(), *parametres, "{nom} : paramètres");
        assert_eq!(rendus.as_slice(), *resultats, "{nom} : résultats");
    }
}

/// juste : le module commité, lu par Node, donne les masses du code natif (V-V8-21). Pour chaque cas, le script lit le module **neuf** (0 case, rien à lire), démarre la neige, lit les masses **sans préparer** (rien), joue `pas` pas, prépare, lit les 30 301 masses une à une
/// (SHA-256 de leurs octets, somme, nombre de masses non nulles, plus grande) et les bornes de `i` (six `u32::MAX`), joue 37 pas de plus **sans préparer** (l'instantané est le même), prépare encore (l'instantané est celui du temps joué), vérifie qu'un démarrage refusé ne
/// change rien et que le temps n'a avancé que de ce qui s'est joué, **redessine l'image nette à partir des seules masses par le rastériseur de la doc en JavaScript (`BigInt`)** et la compare, octet pour octet, à `givre_nette_pixel` ; enfin démarrer efface l'instantané, la préparation
/// le refait, puis `givre_demarrer(1, 0)` (un autre modèle) **efface à lui seul l'instantané** (deux masses lues avant toute préparation : `u32::MAX`) et la préparation rend 0 case, et `givre_demarrer(8, 0)` rend 0. Ce test compare cette sortie, **ligne par ligne et en entier**, à celle du code natif : les masses de la bibliothèque (`valeurs_du_cristal >> 21`), l'image de `rendu_hex::rendre` pour le champ de la doc.
/// Les cas : le jeu 5 à 120 pas en `(96, 3)`, le germe seul du jeu 8 à 0 pas en `(62, 2)`, le jeu 18 à 150 pas en `(128, 1)`, le jeu 11 à 200 pas en `(64, 4)`, des **réglages libres** (le jeu 3 avec `ρ = 0,7`, `β = 2`, `κ = 0,005`, `μ = 0,03`) à 100 pas en `(200, 1)`, une neige rapide qui
/// **s'arrête au mur** (le jeu 1 avec `ρ = 0,956862`) demandée à 1000 pas en `(62, 2)`, et le jeu 8 **jusqu'au bout** (100 000 pas demandés : 1300 joués) en `(62, 2)`. **Ne passe qu'une fois `docs/givre.wasm` reconstruit.**
///
/// Ne couvre pas : la vitesse du module (le script écrit son temps sur la sortie d'erreur : une information, jamais une assertion ; voir le rapport de l'étape) ; la page (voir `etape_5_page.rs`) ; une faute que le natif et Node auraient en commun.
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn node_lit_les_masses_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let cas: [(usize, [u32; 4], u32, u32, u32); 7] = [
        (5, quatre_du_jeu(5), 120, 96, 3),
        (8, quatre_du_jeu(8), 0, 62, 2),
        (18, quatre_du_jeu(18), 150, 128, 1),
        (11, quatre_du_jeu(11), 200, 64, 4),
        (3, [700_000, 2_000_000, 5_000, 30_000], 100, 200, 1),
        (1, [956_862, 1_122_299, 48_396, 36_667], 1000, 62, 2),
        (8, quatre_du_jeu(8), 100_000, 62, 2),
    ];
    for (jeu, quatre, pas, cote, sur) in cas {
        assert_eq!(
            jouer(jeu, quatre, pas, cote, sur),
            attendu_du_script(jeu, quatre, pas, cote, sur),
            "jeu {jeu}, réglages {quatre:?}, {pas} pas, côté {cote}, sur {sur}"
        );
    }
}

/// forme : le script refuse des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard (V-V8-22) : fichier absent, fichier qui n'est pas un module, trop ou trop peu d'arguments, et chacun des huit nombres (jeu, `ρ`, `β`, `κ`, `μ`,
/// pas, côté, suréchantillonnage) qui n'est pas un entier de 32 bits non signé (un texte, un signe, `4294967296`, la chaîne vide) ; un démarrage refusé (le jeu 19) fait échouer le script au lieu de lire un module neuf.
///
/// Ne couvre pas : les arguments justes (voir `node_lit_les_masses_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn le_script_refuse_des_arguments_faux() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let module = module.as_str();
    let absent = racine().join("docs").join("n-existe-pas.wasm");
    let absent = absent.to_str().expect("chemin en UTF-8");
    let pas_un_module = racine().join("Cargo.toml");
    let pas_un_module = pas_un_module.to_str().expect("chemin en UTF-8");
    let bons = ["5", "400000", "1300000", "3000", "70000", "10", "62", "2"];
    let mut cas: Vec<Vec<&str>> = vec![
        vec![absent, "5", "400000", "1300000", "3000", "70000", "10", "62", "2"],
        vec![pas_un_module, "5", "400000", "1300000", "3000", "70000", "10", "62", "2"],
        vec![],
        vec![module],
        vec![module, "5", "400000", "1300000", "3000", "70000", "10", "62"],
        vec![module, "5", "400000", "1300000", "3000", "70000", "10", "62", "2", "en trop"],
        // Un jeu qui n'existe pas : le démarrage est refusé, le script échoue.
        vec![module, "19", "400000", "1300000", "3000", "70000", "10", "62", "2"],
    ];
    for position in 0..8 {
        for mauvais in ["abc", "-1", "4294967296", "", "1.5", "+7"] {
            let mut arguments = vec![module];
            arguments.extend(bons);
            arguments[1 + position] = mauvais;
            cas.push(arguments);
        }
    }
    for arguments in &cas {
        let sortie = lancer_le_script("masses-wasm.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "masses-wasm.mjs, arguments {arguments:?}");
        assert!(
            sortie.stdout.is_empty(),
            "masses-wasm.mjs, arguments {arguments:?} : la sortie standard n'est pas vide"
        );
        assert!(!sortie.stderr.is_empty(), "masses-wasm.mjs, arguments {arguments:?} : aucun message d'erreur");
    }
}

/// forme : les scripts de Node de l'étape sont du JavaScript valide (modules) : `node --check`, le texte leur étant passé sur l'entrée standard (V-V8-23). Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution des scripts (voir `node_lit_les_masses_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-v8"), ignore)]
fn les_scripts_des_masses_sont_du_javascript_valide() {
    if !e5::node_est_la() {
        return;
    }
    for nom in ["masses-wasm.mjs", "reference-masses.mjs"] {
        let script = std::fs::read_to_string(racine().join("tests").join("etape_v8").join(nom))
            .unwrap_or_else(|e| panic!("{nom} se lit : {e}"));
        assert!(!script.trim().is_empty(), "{nom} est vide");
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

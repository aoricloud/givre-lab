//! Étape P4 : le module commité `docs/givre.wasm` joue le métal du code natif, **au bit près** (« même empreinte partout »). Valeurs attendues : `references/etape-p4-valeurs.md`, identifiants `V-P4-xx`.
//!
//! Le module est lu sous Node (`tests/etape_p4/metal-wasm.mjs`), par `docs/lecture.js` comme la page le lit, et comparé au métal natif (`etape_p4_commun::MetalNatif` : la bibliothèque, jamais `vitrine`). La CI joue ces tests sous Linux
//! et sous Windows : le même module y rend les mêmes bits que le code natif de chacun ; `etape_p4_vitrine.rs` fige en plus des empreintes écrites (`les_empreintes_de_reference`) : natif Linux = natif Windows = module.
//! Étiquettes : « juste » = comparé au code natif et à la doc ; la forme du module est de la forme.

mod etape_p4_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_p4_commun::{
    ABSENT, MetalNatif, PAS, chemin_de_lecture, chemin_du_module, e5, hache_des_champs, hexa, image_du_metal,
    lancer_le_script, racine,
};
use givre_lab::empreinte::sha256_hex;

/// Ce que `metal-wasm.mjs` doit écrire pour ces arguments : le métal natif joué morceau par morceau, lu comme le script le lit (voir son en-tête).
fn attendu_du_script(epsilon: u32, cote: u32, morceaux: &[u32], taille: usize) -> String {
    let nan = hexa(f64::NAN);
    let mut lignes = vec![
        format!("avant={nan},{nan},{nan},0,0,null,null,null"),
        "refus=1,2,2,1,2,1".to_string(),
        format!("apres_refus={nan},{nan},{nan},0,0"),
        "demarre=0".to_string(),
    ];
    let mut metal = MetalNatif::new(epsilon, cote);
    lignes.push(format!("reglages={}", metal.reglages().iter().map(|x| hexa(*x)).collect::<Vec<_>>().join(",")));
    for &morceau in morceaux {
        let joue = metal.avancer(morceau);
        let (sites, rayon) = metal.sites_et_rayon();
        lignes.push(format!(
            "morceau={joue};{};{PAS};{};{};{};{sites},{rayon};0,0,{ABSENT},{nan},1,{ABSENT},{ABSENT}",
            metal.dendrite.pas_faits(),
            metal.dendrite.empreinte(),
            metal.mesures().iter().map(|x| hexa(*x)).collect::<Vec<_>>().join(","),
            hache_des_champs(metal.dendrite.phi(), metal.dendrite.u()),
        ));
    }
    let n = cote as usize * cote as usize;
    let (phi, u) = (metal.dendrite.phi(), metal.dendrite.u());
    lignes.push(format!("cases={},{},{nan},{nan},{nan},{nan},{nan},{nan}", hexa(phi[n - 1]), hexa(u[n - 1])));
    let image = image_du_metal(cote as usize, phi, u, 0.65, taille);
    lignes.push(format!("image={taille}x{taille},Uint8ClampedArray,{}", sha256_hex(&image)));
    lignes.push(format!("autre=1,{nan},{nan},{nan},null,null,null"));
    lignes.join("\n") + "\n"
}

/// Lance le script et exige qu'il réussisse ; rend sa sortie standard.
fn jouer(epsilon: u32, cote: u32, morceaux: &[u32], taille: usize) -> String {
    let liste: Vec<String> = morceaux.iter().map(u32::to_string).collect();
    let arguments = [
        chemin_du_module(),
        chemin_de_lecture(),
        epsilon.to_string(),
        cote.to_string(),
        liste.join(","),
        taille.to_string(),
    ];
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let sortie = lancer_le_script("metal-wasm.mjs", &arguments);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "metal-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
    String::from_utf8_lossy(&sortie.stdout).into_owned()
}

/// forme : le module exporte les quatre fonctions du métal avec leurs signatures (V-P4-20) : `givre_metal_demarrer (i32, i32) -> i32`, `givre_metal_reglage (i32) -> f64`, `givre_metal_mesure (i32) -> f64`, `givre_metal_case (i32, i32) -> f64` ;
/// avec `etape-p4` il exporte **exactement** les trente-huit fonctions de la vitrine (les trente-quatre d'avant et ces quatre) plus `memory`, sans importation.
///
/// Ne couvre pas : ce que les fonctions calculent (voir `node_joue_le_metal_comme_le_natif`) ; les autres tests de forme du module sont ceux de `etape_5_artefact.rs`.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn le_module_exporte_les_quatre_fonctions_du_metal() {
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
        .chain(e5::FONCTIONS_P4.iter())
        .collect();
    assert_eq!(toutes.len(), 38, "la liste écrite dans etape_5_commun a trente-huit fonctions");
    assert_eq!(noms.len(), 38, "le module exporte {} fonctions : {noms:?}", noms.len());
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
    let p4: Vec<(&str, Vec<u8>, Vec<u8>)> =
        e5::FONCTIONS_P4.iter().map(|(n, p, r)| (*n, p.to_vec(), r.to_vec())).collect();
    assert_eq!(
        p4,
        [
            ("givre_metal_demarrer", vec![e5::I32, e5::I32], vec![e5::I32]),
            ("givre_metal_reglage", vec![e5::I32], vec![e5::F64]),
            ("givre_metal_mesure", vec![e5::I32], vec![e5::F64]),
            ("givre_metal_case", vec![e5::I32, e5::I32], vec![e5::F64]),
        ],
        "les quatre signatures de la doc"
    );
}

/// juste : **le module commité, sous Node, joue le métal du code natif au bit près** (V-P4-21), lu par `docs/lecture.js` comme la page le lit. Pour chaque cas, le script lit le module **neuf** (les lecteurs du métal rendent `NaN`,
/// `lecture.js` rend `null`), essaie six démarrages refusés (rien ne démarre), démarre, lit les quinze réglages (bits), puis **après chaque morceau** : ce que rend `givre_avancer`, le temps, la durée, **l'empreinte** (celle de
/// `Dendrite::empreinte()` native), les onze mesures (bits : la pointe, les relevés, la vitesse, `σ*`), **les champs `φ` et `u` entiers** (leurs bits), les sites et le rayon, et ce que rendent les fonctions d'avant pour le
/// métal ; à la fin les cases hors de portée, **l'image de `imageDuMetal`** (le SHA-256 de ses octets contre l'image de la doc réécrite en Rust : `etape_p4_commun::image_du_metal`), et un `givre_demarrer(1, 0)` qui remplace
/// le métal. Les cas : quatre `ε` (0 ; 0,02 ; 0,05 ; 0,06, et un `ε` hors de la grille de la page, 0,037 001), les côtés 128, 131 (impair), 200 et 256, des morceaux de 0, 1, 499, 500, 501 pas (les relevés tombent sur des
/// nombres de pas), jusqu'à 2 100 pas, et une taille d'image impaire.
///
/// Ne couvre pas : les longues croissances (jusqu'au mur : voir `etape_p4_vitrine.rs`, natif) ; une faute que le module et le natif feraient ensemble (la dendrite elle-même est éprouvée par P2) ; la page dans un navigateur.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn node_joue_le_metal_comme_le_natif() {
    if !e5::node_est_la() {
        return;
    }
    let cas: [(u32, u32, &[u32], usize); 5] = [
        (50_000, 128, &[0, 1, 498, 1, 500, 600], 96),
        (0, 131, &[499, 1, 1, 1000], 63),
        (60_000, 256, &[700, 800], 128),
        (20_000, 200, &[2_100], 100),
        (37_001, 128, &[1, 1_500], 32),
    ];
    for (epsilon, cote, morceaux, taille) in cas {
        assert_eq!(
            jouer(epsilon, cote, morceaux, taille),
            attendu_du_script(epsilon, cote, morceaux, taille),
            "ε {epsilon} ppm, côté {cote}, morceaux {morceaux:?}"
        );
    }
}

/// forme : le script refuse des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard (V-P4-22) : fichier absent, fichier qui n'est pas un module, `lecture.js` absent,
/// trop ou trop peu d'arguments, un nombre qui n'est pas un entier de 32 bits non signé, une liste de morceaux avec un élément vide, et un démarrage que le module refuse (le côté 127).
///
/// Ne couvre pas : les arguments justes (voir `node_joue_le_metal_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn le_script_refuse_des_arguments_faux() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let lecture = chemin_de_lecture();
    let absent = racine().join("docs").join("n-existe-pas.wasm");
    let absent = absent.to_str().expect("chemin en UTF-8");
    let pas_un_module = racine().join("Cargo.toml");
    let pas_un_module = pas_un_module.to_str().expect("chemin en UTF-8");
    let (m, l) = (module.as_str(), lecture.as_str());
    let cas: [&[&str]; 12] = [
        &[absent, l, "50000", "128", "1", "8"],
        &[pas_un_module, l, "50000", "128", "1", "8"],
        &[m, "absent.js", "50000", "128", "1", "8"],
        &[],
        &[m, l, "50000", "128", "1"],
        &[m, l, "50000", "128", "1", "8", "en trop"],
        &[m, l, "-1", "128", "1", "8"],
        &[m, l, "50000", "abc", "1", "8"],
        &[m, l, "50000", "128", "1,,2", "8"],
        &[m, l, "50000", "128", "1", "4294967296"],
        &[m, l, "50000", "127", "1", "8"],
        &[m, l, "60001", "128", "1", "8"],
    ];
    for arguments in cas {
        let sortie = lancer_le_script("metal-wasm.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "arguments {arguments:?}");
        assert!(sortie.stdout.is_empty(), "arguments {arguments:?} : la sortie standard n'est pas vide");
        assert!(!sortie.stderr.is_empty(), "arguments {arguments:?} : aucun message d'erreur");
    }
}

/// forme : les scripts de Node de l'étape sont du JavaScript valide (modules) : `node --check`, le texte leur étant passé sur l'entrée standard (V-P4-23). Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : ce que font les scripts (voir les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_scripts_du_metal_sont_du_javascript_valide() {
    if !e5::node_est_la() {
        return;
    }
    for script in ["metal-wasm.mjs", "metal-pur.mjs", "metal-absurde.mjs"] {
        let texte = std::fs::read_to_string(racine().join("tests").join("etape_p4").join(script))
            .unwrap_or_else(|erreur| panic!("{script} se lit : {erreur}"));
        let mut node = Command::new("node")
            .args(["--input-type=module", "--check"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node se lance");
        node.stdin.take().expect("entrée standard de Node").write_all(texte.as_bytes()).expect("écriture du script");
        let sortie = node.wait_with_output().expect("Node se termine");
        assert!(sortie.status.success(), "{script} n'est pas valide : {}", String::from_utf8_lossy(&sortie.stderr));
    }
}

/// juste : **jamais de panique dans le module, sous Node** (V-P4-24). `tests/etape_p4/metal-absurde.mjs` fait, sur le module commité, des centaines d'appels tirés (graine écrite dans le script) des quatre fonctions du métal et des
/// fonctions d'avant, avec des arguments absurdes (0, 1, 127, 128, 256, 257, 60 000, 60 001, `u32::MAX`, et des nombres tirés) ; une panique dans le module est une instruction `unreachable` que Node rattrape : le script la compterait. Il
/// écrit le nombre d'appels, le nombre d'exceptions (0), et ce que rendent à la fin les lecteurs du métal. Les valeurs sont comparées au métal natif rejoué par la même suite d'appels (`attendu_de_l_absurde`).
///
/// Ne couvre pas : la mémoire au plus grand côté sous Node (mesurée par le juge : `MODELE.md`, étape P4) ; un appel qui épuiserait la mémoire.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn node_ne_panique_jamais_sur_des_appels_absurdes() {
    if !e5::node_est_la() {
        return;
    }
    let module = chemin_du_module();
    let sortie = lancer_le_script("metal-absurde.mjs", &[module.as_str()]);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "metal-absurde.mjs : {:?}, sortie d'erreur : {erreur}", sortie.status);
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), etape_p4_commun::attendu_de_l_absurde());
}

//! Tests d'acceptation de l'étape 5 (b) : le module commité `docs/givre.wasm`.
//!
//! La CI n'a pas la cible WebAssembly : le module est construit à part puis commité (la commande est dans la doc de
//! `src/vitrine.rs`). La preuve qu'il est bon est qu'il tourne, sous Node, et donne le même résultat que le code natif.
//! Si le code change sans que le module soit reconstruit, les tests de Node échouent.
//!
//! - la forme du module, lue par un petit lecteur du format binaire WebAssembly (`etape_5_commun`) : aucune importation,
//!   exactement les onze fonctions de la vitrine plus `memory`, de bonnes signatures, moins de 1 Mio ;
//! - le résultat du module, lu par Node : `tests/etape_5/empreinte-wasm.mjs` pour l'empreinte, `tests/etape_5/etat-wasm.mjs`
//!   pour ce que la page lit, par les accesseurs, et comparés au code natif (`etape_5_commun::natif`, qui n'appelle jamais
//!   `vitrine`) ;
//! - le refus des arguments faux par les deux scripts.
//!
//! Étiquettes : « juste » = comparé au code natif ; la forme du module est de la forme, pas une preuve de comportement.
//! Ces tests ne touchent pas à l'état global de la vitrine (ils ne comparent qu'au code natif) : ils n'ont pas besoin de
//! verrou. Ils n'utilisent aucune fonction de `f64` hors `to_le_bytes`.

mod etape_5_commun;

use std::collections::BTreeSet;

use etape_5_commun::{
    FONCTIONS, analyser, chemin_module, lancer_le_script, lire_module, natif, node_est_la, racine, reference,
};
use givre_lab::empreinte::sha256_hex;

/// Le plafond du module : 1 Mio.
const TAILLE_MAX_MODULE: u64 = 1024 * 1024;

/// Les scripts : celui qui lit l'empreinte du module, et celui qui lit son état par les accesseurs.
const SCRIPT_EMPREINTE: &str = "empreinte-wasm.mjs";
const SCRIPT_ETAT: &str = "etat-wasm.mjs";

/// forme : le fichier est un module WebAssembly en version 1, non vide, et il pèse moins de 1 Mio.
///
/// Ne couvre pas : ce que le module calcule (voir les tests de Node).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn le_module_est_du_webassembly_et_leger() {
    let octets = lire_module();
    analyser(&octets);
    assert!((octets.len() as u64) < TAILLE_MAX_MODULE, "{} octets", octets.len());
}

/// forme : le module n'importe rien : ni fonction, ni mémoire, ni table, ni globale. Il est donc instanciable avec des
/// importations vides, et la page n'a rien à lui fournir.
///
/// Ne couvre pas : ce que le module calcule.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn le_module_n_importe_rien() {
    let module = analyser(&lire_module());
    assert_eq!(module.importations, 0, "le module a {} importation(s)", module.importations);
}

/// forme : le module exporte exactement les fonctions `givre_*` de la liste, ni plus ni moins ; la mémoire `memory` est
/// tolérée, rien d'autre (aucune autre exportation, de quelque genre que ce soit), et aucun nom n'est exporté deux fois.
///
/// Ne couvre pas : ce que les fonctions calculent.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn le_module_exporte_exactement_les_fonctions_de_la_vitrine() {
    let module = analyser(&lire_module());

    let fonctions: BTreeSet<&str> =
        module.exportations.iter().filter(|e| e.genre == 0).map(|e| e.nom.as_str()).collect();
    let attendues: BTreeSet<&str> = FONCTIONS.iter().map(|(nom, _, _)| *nom).collect();
    assert_eq!(fonctions, attendues, "les fonctions exportées ne sont pas celles de la liste");

    let autres: Vec<(&str, u8)> =
        module.exportations.iter().filter(|e| e.genre != 0).map(|e| (e.nom.as_str(), e.genre)).collect();
    for (nom, genre) in &autres {
        assert!((*nom, *genre) == ("memory", 2), "exportation inattendue : « {nom} » (genre {genre})");
    }
    assert_eq!(module.exportations.len(), fonctions.len() + autres.len());
    assert_eq!(
        module.exportations.iter().map(|e| e.nom.as_str()).collect::<BTreeSet<_>>().len(),
        module.exportations.len(),
        "un nom est exporté deux fois"
    );
}

/// forme : chaque fonction exportée a la bonne signature : le nombre et le type des paramètres, et du résultat (`i32` pour un
/// `u32`, `f64` pour un `f64`).
///
/// Ne couvre pas : ce que les fonctions calculent.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn chaque_fonction_a_la_bonne_signature() {
    let module = analyser(&lire_module());
    assert_eq!(module.importations, 0, "les indices de fonction supposent qu'il n'y a aucune importation");

    for (nom, parametres, resultats) in FONCTIONS {
        let exportation = module
            .exportations
            .iter()
            .find(|e| e.genre == 0 && e.nom == nom)
            .unwrap_or_else(|| panic!("la fonction {nom} n'est pas exportée"));
        let indice_de_type = *module
            .fonctions
            .get(exportation.indice as usize)
            .unwrap_or_else(|| panic!("{nom} : indice de fonction {} hors de la section", exportation.indice));
        let (obtenus, rendus) = module
            .types
            .get(indice_de_type as usize)
            .unwrap_or_else(|| panic!("{nom} : type {indice_de_type} absent de la section des types"));
        assert_eq!(obtenus.as_slice(), parametres, "{nom} : types des paramètres");
        assert_eq!(rendus.as_slice(), resultats, "{nom} : types des résultats");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

/// Lance un script et exige qu'il réussisse ; rend sa sortie standard.
fn sortie_du_script(script: &str, arguments: &[&str]) -> String {
    let sortie = lancer_le_script(script, arguments);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "{script} {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
    String::from_utf8_lossy(&sortie.stdout).into_owned()
}

/// juste : le module, chargé par Node (`WebAssembly.instantiate`, importations vides), donne la même empreinte que le code
/// natif, et la donne **après chaque morceau** : DLA de graine 1 jusqu'au bout (2999), graine 2 à 300, une grande graine par
/// morceaux inégaux (`1,0,6,13,0,60`, dont deux de 0 : une empreinte gardée d'avant se voit), la plus grande graine, 0 pas
/// (l'état initial) ; la plaque, l'étoile (graine 0) et la dendrite (grande graine : ignorée) jusqu'au bout, la dendrite par
/// morceaux dont le dernier dépasse la durée. La sortie du script est exactement une ligne de 64 caractères par morceau.
///
/// Ne couvre pas : les accesseurs (voir `node_lit_l_etat_comme_la_page`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn node_donne_l_empreinte_du_code_natif() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");

    let cas: [(u32, u32, &[u32]); 8] = [
        (0, 1, &[2999]),
        (0, 2, &[300]),
        (0, 0xDEAD_BEEF, &[1, 0, 6, 13, 0, 60]),
        (0, u32::MAX, &[5]),
        (0, 1, &[0]),
        (1, 7, &[400]),
        (2, 0, &[120]),
        (3, u32::MAX, &[1, 0, 700, 98, 1, 50]),
    ];
    for (modele, graine, morceaux) in cas {
        let liste: Vec<String> = morceaux.iter().map(u32::to_string).collect();
        let sortie =
            sortie_du_script(SCRIPT_EMPREINTE, &[module, &modele.to_string(), &graine.to_string(), &liste.join(",")]);
        let mut attendu = String::new();
        let mut cumul = 0_u32;
        for morceau in morceaux {
            cumul = cumul.saturating_add(*morceau);
            attendu.push_str(&natif(modele, graine, cumul).empreinte);
            attendu.push('\n');
        }
        assert_eq!(sortie, attendu, "modèle {modele}, graine {graine}, morceaux {morceaux:?}");
    }
}

/// juste : pour la DLA de graine 1 jusqu'au bout et la plaque jusqu'au bout, la sortie du script est le contenu du fichier de
/// référence du dépôt (`references/dla-graine-1.empreinte`, `references/reiter-plaque.empreinte` : des régressions de notre
/// code, recoupées avec le calcul natif des autres tests).
///
/// Ne couvre pas : les autres cas (voir `node_donne_l_empreinte_du_code_natif`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn node_redonne_les_references_du_depot() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");
    for (modele, graine, pas, fichier) in
        [("0", "1", "2999", "dla-graine-1.empreinte"), ("1", "1", "400", "reiter-plaque.empreinte")]
    {
        let sortie = sortie_du_script(SCRIPT_EMPREINTE, &[module, modele, graine, pas]);
        assert_eq!(sortie.trim_end(), reference(fichier), "modèle {modele}");
    }
}

/// Les octets en hexadécimal, dans l'ordre.
fn hexadecimal(octets: &[u8]) -> String {
    octets.iter().map(|octet| format!("{octet:02x}")).collect()
}

/// Ce que `etat-wasm.mjs` doit écrire : l'état du code natif, lu comme le script le lit (voir son en-tête).
fn etat_attendu(modele: u32, graine: u32, pas: u32) -> String {
    let etat = natif(modele, graine, pas);
    // L'image du temps 0, que le script lit avant le premier pas.
    let depart = natif(modele, graine, 0);
    let compacite = match etat.compacite {
        None => "NaN".to_string(),
        Some(valeur) => hexadecimal(&valeur.to_le_bytes()),
    };
    format!(
        "avant=1\ndemarre=1\ndepart={}x{}\nhache_depart={}\njoue={}\ntemps={}\nduree={}\nlargeur={}\nhauteur={}\nsites={}\n\
         rayon={}\ncompacite={}\nhache_image={}\ncouleurs=1\nbords=1\n",
        depart.largeur,
        depart.hauteur,
        sha256_hex(&depart.rvb),
        etat.temps,
        etat.temps,
        etat.duree,
        etat.largeur,
        etat.hauteur,
        etat.sites,
        etat.rayon,
        compacite,
        sha256_hex(&etat.rvb),
    )
}

/// juste : le module, lu comme la page le lit, par ses accesseurs et non par l'empreinte : le temps, la durée, les dimensions
/// de l'image, **chaque pixel** (par le SHA-256 des octets R, V, B), les compteurs, la compacité (bit à bit, `NaN` pour la
/// DLA), ce que rendent `givre_demarrer` et `givre_avancer`, les valeurs hors de portée, le contrat « avant tout démarrage »
/// (lu sur un module neuf) et l'image du temps 0 lue **avant** le premier pas (une image gardée d'avant `givre_avancer` ne se voit
/// que si elle a été lue avant), sont ceux du code natif. L'empreinte seule ne dit rien des accesseurs : un module construit avant
/// la correction d'un accesseur a la même empreinte et dessinerait n'importe quoi. Le script `etat-wasm.mjs` écrit une ligne
/// `clé=valeur` par grandeur ; ce test compare sa sortie, entière, à celle que le code natif donne. Les cas : la DLA à 0, 1, 77
/// et 2999 particules (grandes graines comprises), les trois flocons au temps 0 et au bout, la dendrite avec un nombre de pas
/// (10 000) qui dépasse la durée (800 : `givre_avancer` en joue 800).
///
/// Ne couvre pas : l'empreinte (voir `node_donne_l_empreinte_du_code_natif`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn node_lit_l_etat_comme_la_page() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");

    let cas: [(u32, u32, u32); 11] = [
        (0, 1, 2999),
        (0, 2, 0),
        (0, 0xDEAD_BEEF, 1),
        (0, 3, 77),
        (0, u32::MAX, 500),
        (1, 1, 400),
        (1, 9, 0),
        (2, 5, 120),
        (2, 0, 37),
        (3, u32::MAX, 500),
        (3, 1, 10_000),
    ];
    for (modele, graine, pas) in cas {
        let sortie =
            sortie_du_script(SCRIPT_ETAT, &[module, &modele.to_string(), &graine.to_string(), &pas.to_string()]);
        assert_eq!(sortie, etat_attendu(modele, graine, pas), "modèle {modele}, graine {graine}, {pas} pas");
    }
}

/// forme : les deux scripts refusent des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans
/// rien écrire sur la sortie standard : fichier absent, fichier qui n'est pas un module, trop ou trop peu d'arguments, modèle
/// qui n'est pas un entier de 0 à 3, graine ou nombre de pas qui n'est pas un entier de 32 bits non signé, liste de pas avec
/// un élément vide (pour les deux scripts : `etat-wasm.mjs` ne prend qu'un entier, `empreinte-wasm.mjs` une liste).
///
/// Ne couvre pas : les arguments justes (voir les autres tests de Node).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn les_scripts_refusent_des_arguments_faux() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");
    let absent = racine().join("docs").join("n-existe-pas.wasm");
    let absent = absent.to_str().expect("chemin en UTF-8");
    let pas_un_module = racine().join("Cargo.toml");
    let pas_un_module = pas_un_module.to_str().expect("chemin en UTF-8");

    let cas: [&[&str]; 19] = [
        &[absent, "0", "1", "80"],
        &[pas_un_module, "0", "1", "80"],
        &[],
        &[module],
        &[module, "0", "1"],
        &[module, "0", "1", "80", "en trop"],
        &[module, "4", "1", "80"],
        &[module, "abc", "1", "80"],
        &[module, "-1", "1", "80"],
        &[module, "", "1", "80"],
        &[module, "0", "abc", "80"],
        &[module, "0", "-1", "80"],
        &[module, "0", "4294967296", "80"],
        &[module, "0", "1", "abc"],
        &[module, "0", "1", "-1"],
        &[module, "0", "1", "4294967296"],
        &[module, "0", "1", ""],
        &[module, "0", "1", "1,,2"],
        &[module, "0", "1", "80,"],
    ];
    for script in [SCRIPT_EMPREINTE, SCRIPT_ETAT] {
        for arguments in cas {
            let sortie = lancer_le_script(script, arguments);
            assert_eq!(sortie.status.code(), Some(1), "{script}, arguments {arguments:?}");
            assert!(sortie.stdout.is_empty(), "{script}, arguments {arguments:?} : la sortie standard n'est pas vide");
            assert!(!sortie.stderr.is_empty(), "{script}, arguments {arguments:?} : aucun message d'erreur");
        }
    }
}

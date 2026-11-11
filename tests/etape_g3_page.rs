//! Étape G3 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) montre la neige de Gravner et Griffeath : un panneau (un menu de jeux nommés lus dans le module, quatre curseurs), ses mesures et son étiquette. Valeurs attendues :
//! `references/etape-g3-valeurs.md`, identifiants `V-G3-xx`. Comme aux étapes 5, Z1, D3 et V7, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_js_lit_la_neige_sur_un_faux_module` appelle les fonctions de `lecture.js` de la neige sur un faux module qui **change d'état** et compare à un calcul fait à la main ; il ne lit pas `docs/givre.wasm`.
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : le prototype de l'écrivain fait foi, ses fichiers se recopient à l'octet)
//!
//! - `index.html` : en gardant tout ce que les étapes 5, Z1, D3 et V7 demandent, une **sixième option** `neige` (« Neige de Gravner ») du choix du modèle ; un panneau `<section id="commandes-neige">` **caché au départ** qui contient un `<select id="jeu-neige">` **vide**
//!   (ses options sont écrites par `app.js` d'après le module : la page n'écrit ni le nombre ni les réglages des jeux), quatre curseurs `<input type="range">` `neige-rho`, `neige-beta`, `neige-kappa`, `neige-mu` (des entiers **en millionièmes**, de `min`, `max` et `step`
//!   entiers, dans la plage du module et tels que chaque jeu nommé soit atteignable) avec leurs `<output>` `valeur-neige-…`, une phrase `source-neige` (lu ou déduit) et une légende `legende-neige` qui dit que le mur est à nous, que ce sont des stades précoces et que ce ne serait
//!   plus la neige de l'article au mur ; un bloc `<section id="mesures-neige-bloc">` **caché au départ** avec `reglages-neige`, `mesures-neige` et `forme-neige`.
//! - `lecture.js` exporte de plus (sans DOM) `lireJeuxNeige(m)`, `lireReglagesNeige(m)`, `lireMesuresNeige(m)`, `formaterReglage(x)`, `texteDuJeuNeige(jeu)`, `texteSourceNeige(jeu, modifie)`, `texteDeLaFormeNeige(forme)` (qui écrit « notre classement, pas celui de l'article »),
//!   `texteMesuresNeige(mesures)`, `texteFinNeige(mesures)`, `texteRefusNeige(code)` et `blocsNeige(neige)`.
//! - `app.js` : **l'état suit le modèle démarré, pas les curseurs** : `relancer` (la fonction qui appelle `givre_demarrer(`) appelle `demarrerLaNeige()` pour la neige et ne pose `neigeEnCours` qu'**après** avoir vérifié que le code est 0 ; `demarrerLaNeige` est la seule fonction qui
//!   donne les curseurs à `givre_neige_demarrer` (cinq arguments : le jeu du menu, puis `ρ β κ μ`, aucun nombre écrit) ; ce que la page écrit de la neige démarrée (`afficherLaNeige`) vient du module (`lireReglagesNeige`, `lireMesuresNeige`) et ne lit aucun curseur ; le menu des jeux est rempli
//!   d'après `lireJeuxNeige(m)` et les curseurs posés d'après les réglages du jeu lus dans le module ; le choix d'un jeu et le `change` des curseurs relancent, le `input` des curseurs n'écrit que les valeurs affichées ; les panneaux de la neige sont montrés d'après `blocsNeige(neigeEnCours)` ; la neige est
//!   jouée par lots de `PAS_PAR_APPEL_NEIGE` pas en s'arrêtant quand `BUDGET_IMAGE_MS` sont écoulées (une image de l'animation ne fige pas la page) ; l'empreinte de la neige (huit SHA-256 de tout l'état) n'est calculée qu'à l'arrêt ; **le `dessiner` de V7 est gardé tel quel** (voir `etape_v7_page.rs`).
//! - **Aucun réglage d'un jeu nommé n'est écrit dans les fichiers de la page** (ils viennent du module), aucune valeur publiée de résultat, aucune adresse.
//!
//! Étiquettes : `lecture_js_lit_la_neige_sur_un_faux_module` est **juste** (comparé à un calcul à la main) ; `les_curseurs_atteignent_chaque_jeu_nomme` compare la forme de la page à la table des jeux de la source (**juste** pour la table, **forme** pour la page) ; tous les autres sont de la
//! **forme** (régression de texte), jamais une preuve du comportement de la page.

mod etape_g3_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_g3_commun::v7::{
    a_l_attribut, arites_des_appels, attribut, balise_d_id, balises, code_de_app_js, code_de_lecture_js,
    constante_entiere, e5, fermeture, fonction_qui_contient, nom_de_balise, texte_de, texte_visible, variable_de,
};
use etape_g3_commun::{JEUX, chemin_de_lecture, lancer_le_script, racine};

/// Le corps (de l'accolade ouvrante à la fermante) de la fonction `function nom(` déclarée dans `code`.
fn corps_de(code: &str, nom: &str) -> String {
    let debut =
        code.find(&format!("function {nom}(")).unwrap_or_else(|| panic!("app.js ne déclare pas `function {nom}(`"));
    let ouvrante = debut + code[debut..].find(") {").unwrap_or_else(|| panic!("le corps de {nom}")) + 2;
    code[ouvrante..=fermeture(code, ouvrante)].to_string()
}

/// Les noms des quatre curseurs de la neige dans `app.js` (les variables qui gardent leurs éléments).
fn curseurs_de_la_neige(app: &str) -> Vec<String> {
    ["neige-rho", "neige-beta", "neige-kappa", "neige-mu"]
        .iter()
        .map(|id| {
            variable_de(app, id).unwrap_or_else(|| panic!("app.js ne garde pas l'élément « {id} » dans une variable"))
        })
        .collect()
}

/// forme : `index.html` a le panneau de la neige (V-G3-20). Le choix du modèle a une sixième option de `value` `neige` ; `<section id="commandes-neige">` est **caché au départ** ; il contient `<select id="jeu-neige">` **sans aucune `<option>`** (la page n'écrit pas la liste
/// des jeux : `app.js` la tire du module), quatre `<input type="range">` `neige-rho`, `neige-beta`, `neige-kappa` et `neige-mu` chacun avec `min`, `max` et `step` **entiers** et un `<output>` (`valeur-neige-rho`, …) ; les phrases `source-neige` et `legende-neige` ; `<section id="mesures-neige-bloc">` est caché
/// au départ et contient `reglages-neige`, `mesures-neige` et `forme-neige` ; **aucun curseur n'a de valeur initiale** (`value` : une valeur écrite ici serait un réglage de la page).
///
/// Ne couvre pas : que le panneau soit montré ou caché au bon moment (le DOM n'est pas éprouvé : voir `app_js_demarre_la_neige_d_apres_le_module`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn la_page_a_le_panneau_de_la_neige() {
    let html = e5::lire_page("index.html");
    for (id, sorte) in [
        ("commandes-neige", "section"),
        ("jeu-neige", "select"),
        ("neige-rho", "input"),
        ("neige-beta", "input"),
        ("neige-kappa", "input"),
        ("neige-mu", "input"),
        ("valeur-neige-rho", "output"),
        ("valeur-neige-beta", "output"),
        ("valeur-neige-kappa", "output"),
        ("valeur-neige-mu", "output"),
        ("source-neige", "p"),
        ("legende-neige", "p"),
        ("mesures-neige-bloc", "section"),
        ("reglages-neige", "p"),
        ("mesures-neige", "p"),
        ("forme-neige", "p"),
    ] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    for id in ["commandes-neige", "mesures-neige-bloc"] {
        let balise = balise_d_id(&html, id).expect("le bloc");
        assert!(a_l_attribut(balise, "hidden"), "« {id} » n'est pas caché au départ : <{balise}>");
    }
    for (id, sortie) in [
        ("neige-rho", "valeur-neige-rho"),
        ("neige-beta", "valeur-neige-beta"),
        ("neige-kappa", "valeur-neige-kappa"),
        ("neige-mu", "valeur-neige-mu"),
    ] {
        let curseur = balise_d_id(&html, id).expect("le curseur");
        assert_eq!(attribut(curseur, "type").as_deref(), Some("range"), "« {id} » n'est pas de type range");
        for nom in ["min", "max", "step"] {
            let valeur = attribut(curseur, nom).unwrap_or_else(|| panic!("« {id} » n'a pas de {nom}"));
            assert!(valeur.parse::<u32>().is_ok(), "{nom} de « {id} » n'est pas un entier : {valeur}");
        }
        assert!(
            attribut(curseur, "value").is_none(),
            "« {id} » a une valeur initiale écrite dans la page : <{curseur}>"
        );
        let sortie_balise = balise_d_id(&html, sortie).expect("la sortie");
        assert_eq!(attribut(sortie_balise, "for").as_deref(), Some(id), "la sortie « {sortie} » est liée à « {id} »");
    }
    // Le menu des jeux est vide dans la page : aucune <option> entre <select id="jeu-neige"> et son </select>.
    let mut dans_le_menu = false;
    for morceau in html.split('<').skip(1) {
        let balise = morceau.split('>').next().unwrap_or("");
        if nom_de_balise(balise) == "select" && attribut(balise, "id").as_deref() == Some("jeu-neige") {
            dans_le_menu = true;
        } else if balise.starts_with("/select") {
            dans_le_menu = false;
        } else if dans_le_menu {
            assert_ne!(
                nom_de_balise(balise),
                "option",
                "le menu des jeux a une option écrite dans la page : <{balise}> (elles viennent du module)"
            );
        }
    }
    // La sixième option du choix du modèle.
    let mut options: Vec<String> = Vec::new();
    let mut dans_le_choix = false;
    for morceau in html.split('<').skip(1) {
        let (balise, _) = morceau.split_once('>').unwrap_or((morceau, ""));
        if nom_de_balise(balise) == "select" && attribut(balise, "id").as_deref() == Some("modele") {
            dans_le_choix = true;
        } else if balise.starts_with("/select") {
            dans_le_choix = false;
        } else if dans_le_choix && nom_de_balise(balise) == "option" {
            options.push(attribut(balise, "value").unwrap_or_default());
        }
    }
    assert_eq!(
        options.last().map(String::as_str),
        Some("neige"),
        "la dernière option du choix du modèle : {options:?}"
    );
    assert_eq!(options.len(), 6);
    // Le `legende-nette` de V7 dit encore « hexagone » et ne ment plus sur la neige : il parle de la masse cristalline.
    let legende = texte_de(&html, "legende-nette").expect("legende-nette").to_lowercase();
    assert!(
        legende.contains("hexagone") && legende.contains("neige"),
        "la légende de l'image nette dit : « {legende} »"
    );
}

/// juste + forme : les curseurs de la page atteignent chacun des dix-neuf jeux nommés (V-G3-21). Pour chaque jeu de la table **écrite à la main d'après la source** (`etape_g3_commun::JEUX`), `ρ`, `β`, `κ` et `μ` en millionièmes sont dans `[min, max]` du curseur correspondant de
/// `index.html` et sur sa grille (`(valeur − min)` multiple de `step`) : choisir un jeu nommé pose les curseurs sur ses réglages sans que le navigateur les arrondisse ; et chaque curseur reste dans la plage du module (`ρ`, `κ`, `μ` de 0 à 1 000 000 ; `β` de 1 000 000 à 8 000 000), avec un `step` d'au moins 1.
/// Un curseur trop court, ou trop gros pour un réglage fin (`κ = 0,0001`, `β = 1,09`), se voit.
///
/// Ne couvre pas : que le navigateur affiche bien les curseurs (seul un navigateur le montre).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_curseurs_atteignent_chaque_jeu_nomme() {
    let html = e5::lire_page("index.html");
    let lire = |id: &str, nom: &str| -> u32 {
        attribut(balise_d_id(&html, id).expect("le curseur"), nom)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| panic!("« {id} » : {nom}"))
    };
    // (id, indice du réglage dans `JeuAttendu::reglages`, plage du module).
    for (id, indice, bas, haut) in [
        ("neige-rho", 0_usize, 0_u32, 1_000_000_u32),
        ("neige-beta", 1, 1_000_000, 8_000_000),
        ("neige-kappa", 4, 0, 1_000_000),
        ("neige-mu", 5, 0, 1_000_000),
    ] {
        let (min, max, pas) = (lire(id, "min"), lire(id, "max"), lire(id, "step"));
        assert!(pas >= 1, "« {id} » : step {pas}");
        assert!(
            bas <= min && min < max && max <= haut,
            "« {id} » : [{min}, {max}] n'est pas dans la plage du module [{bas}, {haut}]"
        );
        for (jeu, attendu) in JEUX.iter().enumerate() {
            let v = attendu.reglages[indice];
            assert!((min..=max).contains(&v), "jeu {jeu} : {id} = {v} hors du curseur [{min}, {max}]");
            assert_eq!(
                (v - min) % pas,
                0,
                "jeu {jeu} : {id} = {v} n'est pas sur la grille du curseur (min {min}, step {pas})"
            );
        }
    }
}

/// forme : **aucun réglage d'un jeu nommé n'est écrit dans la page** (V-G3-22). Ni `index.html`, ni `app.js`, ni `lecture.js` (commentaires compris) ne contient un des décimaux des réglages des jeux de la table (`0.0745`, `0.00005`, `0.00001`, `0.025`, `0.004`, `0.015`, `1.09`, `0.003` et `0.0001`, écrits avec un point
/// ou une virgule) : ils viennent du module. `app.js` appelle `givre_neige_demarrer(` **une seule fois**, avec **cinq arguments dont aucun ne commence par un chiffre** (le jeu, `ρ`, `β`, `κ`, `μ` sont des expressions, jamais des nombres écrits) ; `app.js` demande les jeux au module (`lireJeuxNeige(m)`) ; ni `app.js` ni
/// `lecture.js` ne contiennent les valeurs publiées de D de D3 (`1,71`, `1,433`, `1,263`) ni d'adresse (`http://`, `https://`) : aucune valeur publiée de résultat, aucune adresse.
///
/// Ne couvre pas : un réglage écrit sous une autre forme (une fraction, un produit) ; les valeurs affichées à l'écran (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn aucun_reglage_n_est_ecrit_dans_la_page() {
    for nom in ["index.html", "app.js", "lecture.js"] {
        let texte = e5::lire_page(nom);
        for decimal in ["0.0745", "0.00005", "0.00001", "0.025", "0.004", "0.015", "1.09", "0.003", "0.0001"] {
            for ecriture in [decimal.to_string(), decimal.replace('.', ",")] {
                assert!(
                    !texte.contains(&ecriture),
                    "docs/{nom} écrit le réglage « {ecriture} » d'un jeu nommé : il vient du module"
                );
            }
        }
        for interdit in ["1,71", "1.71", "1,433", "1.433", "1,263", "1.263", "http://", "https://"] {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} »");
        }
    }
    let app = code_de_app_js();
    let appels = app.matches("givre_neige_demarrer(").count();
    assert_eq!(appels, 1, "app.js appelle givre_neige_demarrer {appels} fois : une seule, dans demarrerLaNeige");
    let debut = app.find("givre_neige_demarrer(").expect("l'appel") + "givre_neige_demarrer(".len();
    let mut profondeur = 0_i32;
    let mut arguments: Vec<String> = vec![String::new()];
    for c in app[debut..].chars() {
        match c {
            '(' | '[' | '{' => {
                profondeur += 1;
                arguments.last_mut().expect("un argument").push(c);
            }
            ')' | ']' | '}' => {
                if profondeur == 0 {
                    break;
                }
                profondeur -= 1;
                arguments.last_mut().expect("un argument").push(c);
            }
            ',' if profondeur == 0 => arguments.push(String::new()),
            _ => arguments.last_mut().expect("un argument").push(c),
        }
    }
    let arguments: Vec<String> =
        arguments.into_iter().map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect();
    assert_eq!(
        arguments.len(),
        5,
        "givre_neige_demarrer est appelée avec {arguments:?} : cinq arguments (jeu, ρ, β, κ, μ)"
    );
    for argument in &arguments {
        assert!(!argument.starts_with(|c: char| c.is_ascii_digit()), "un réglage écrit dans l'appel : « {argument} »");
    }
    assert!(
        arites_des_appels(&app, "lireJeuxNeige") == [1],
        "app.js ne lit pas les jeux nommés dans le module (lireJeuxNeige(m))"
    );
}

/// forme : l'état suit le modèle démarré, pas les curseurs (V-G3-23). Dans `app.js` : `relancer` (celle qui appelle `givre_demarrer(`) appelle `demarrerLaNeige()` et ne pose `neigeEnCours = neige` **qu'après** le test du code de retour (la fonction se termine par `return` quand le
/// code n'est pas 0 : avant l'affectation) ; `demarrerLaNeige` est la seule fonction qui lit `.value` des **quatre curseurs pour démarrer** et `afficherLesCurseursNeige` la seule autre (elle écrit les valeurs affichées à côté des curseurs et dit s'ils sont encore ceux du jeu) ; **`afficherLaNeige`, qui écrit
/// ce que la page dit de la neige démarrée, ne lit aucun curseur ni le menu** (elle lit `lireReglagesNeige(m)` et `lireMesuresNeige(m)`) ; `montrerLesReperes` écrit `.hidden` du panneau et du bloc des mesures d'après `blocsNeige(neigeEnCours)` (la neige **en cours**, pas le choix) sans négation ; `afficherMesures`
/// appelle `afficherLaNeige` quand `neigeEnCours` ; le menu est rempli par `remplirLeMenuNeige` d'après `lireJeuxNeige(m)` (`document.createElement("option")`) ; les écouteurs : le menu `change` → `choisirUnJeuNeige` (qui pose les curseurs puis relance), chaque curseur `input` → `afficherLesCurseursNeige` et `change` → `relancer`
/// (jamais `input` → `relancer` : un démarrage à chaque cran du curseur ferait recommencer à chaque pixel de glisse).
///
/// Ne couvre pas : que les écouteurs soient installés au bon moment ni que le DOM réagisse.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn app_js_demarre_la_neige_d_apres_le_module() {
    let app = code_de_app_js();
    let curseurs = curseurs_de_la_neige(&app);
    let menu = variable_de(&app, "jeu-neige").expect("app.js garde le menu des jeux dans une variable");
    let appel = app.find("givre_demarrer(").expect("app.js démarre un modèle");
    let (relancer, corps_de_relancer) =
        fonction_qui_contient(&app, appel).expect("l'appel est dans une `function nom(…)`");
    assert!(
        corps_de_relancer.contains("demarrerLaNeige()"),
        "« {relancer} » ne démarre pas la neige par demarrerLaNeige()"
    );
    let affectation =
        corps_de_relancer.find("neigeEnCours = neige;").expect("« relancer » pose `neigeEnCours = neige;`");
    let refus =
        corps_de_relancer.find("texteRefusNeige(").expect("« relancer » dit le refus du module par texteRefusNeige(…)");
    let apres_le_refus = corps_de_relancer[refus + "texteRefusNeige(code);".len()..].trim_start();
    assert!(
        corps_de_relancer[refus..].starts_with("texteRefusNeige(code);") && apres_le_refus.starts_with("return;"),
        "le refus de la neige ne s'arrête pas par `return;` juste après `texteRefusNeige(code);`"
    );
    let retour = corps_de_relancer.len() - apres_le_refus.len();
    assert!(retour < affectation, "`neigeEnCours` est posé avant le test du code de retour du démarrage");
    assert_eq!(
        app.matches("neigeEnCours = neige;").count(),
        1,
        "`neigeEnCours` n'est posé qu'à un seul endroit (après un démarrage réussi)"
    );
    assert_eq!(app.matches("neigeEnCours = ").count(), 2, "`neigeEnCours` : sa déclaration et une seule affectation");

    // Qui lit les curseurs.
    for curseur in &curseurs {
        let lecture = format!("{curseur}.value");
        for (position, _) in app.match_indices(&lecture) {
            let (nom, _) = fonction_qui_contient(&app, position)
                .unwrap_or_else(|| panic!("`{lecture}` est lu hors d'une fonction"));
            assert!(
                ["demarrerLaNeige", "afficherLesCurseursNeige", "poserLesCurseursNeige"].contains(&nom.as_str()),
                "`{lecture}` est lu dans « {nom} » : seuls demarrerLaNeige (le démarrage), afficherLesCurseursNeige (les valeurs affichées) et poserLesCurseursNeige (qui les pose) y touchent"
            );
        }
    }
    let demarrage = corps_de(&app, "demarrerLaNeige");
    for curseur in &curseurs {
        assert!(
            demarrage.contains(&format!("Number({curseur}.value)")),
            "demarrerLaNeige ne lit pas `Number({curseur}.value)`"
        );
    }
    assert!(demarrage.contains(&format!("Number({menu}.value)")), "demarrerLaNeige ne lit pas le jeu du menu");
    // L'ordre des arguments de `givre_neige_demarrer` : le jeu, puis `ρ β κ μ` (celui du module).
    let positions: Vec<usize> = std::iter::once(&menu)
        .chain(curseurs.iter())
        .map(|nom| demarrage.find(&format!("Number({nom}.value)")).expect("la lecture est là"))
        .collect();
    assert!(
        positions.windows(2).all(|deux| deux[0] < deux[1]),
        "demarrerLaNeige ne donne pas au module le jeu, ρ, β, κ, μ dans cet ordre : {positions:?}"
    );
    // Les curseurs sont posés, et dits « déplacés » ou non, d'après le jeu **du menu** (pas le premier, pas un nombre écrit).
    for nom in ["poserLesCurseursNeige", "afficherLesCurseursNeige"] {
        assert!(
            corps_de(&app, nom).contains(&format!("jeuxNeige[Number({menu}.value)]")),
            "{nom} ne prend pas le jeu du menu : `jeuxNeige[Number({menu}.value)]`"
        );
    }
    let affichage = corps_de(&app, "afficherLaNeige");
    for interdit in curseurs.iter().chain(std::iter::once(&menu)) {
        assert!(
            !affichage.contains(interdit.as_str()),
            "afficherLaNeige lit « {interdit} » : ce que la page dit de la neige vient du module, pas des curseurs"
        );
    }
    assert!(
        affichage.contains("lireReglagesNeige(m)") && affichage.contains("lireMesuresNeige(m)"),
        "afficherLaNeige ne lit pas le module"
    );
    assert!(
        corps_de(&app, "afficherMesures").contains("if (neigeEnCours) {\n    afficherLaNeige();"),
        "afficherMesures n'appelle pas afficherLaNeige pour la neige"
    );

    // Ce que la page montre.
    let (panneau, mesures) = (
        variable_de(&app, "commandes-neige").expect("le panneau"),
        variable_de(&app, "mesures-neige-bloc").expect("le bloc des mesures"),
    );
    let montrer = corps_de(&app, "montrerLesReperes");
    assert!(
        montrer.contains("const cachesNeige = blocsNeige(neigeEnCours);"),
        "montrerLesReperes ne décide pas d'après `blocsNeige(neigeEnCours)`"
    );
    assert!(
        montrer.contains(&format!("{panneau}.hidden = cachesNeige.commandes;")),
        "le panneau n'est pas écrit `{panneau}.hidden = cachesNeige.commandes;`"
    );
    assert!(
        montrer.contains(&format!("{mesures}.hidden = cachesNeige.mesures;")),
        "le bloc des mesures n'est pas écrit `{mesures}.hidden = cachesNeige.mesures;`"
    );
    assert_eq!(arites_des_appels(&app, "blocsNeige"), [1], "blocsNeige est appelée une seule fois, avec un argument");

    // Le menu est rempli d'après le module.
    let remplissage = corps_de(&app, "remplirLeMenuNeige");
    assert!(
        remplissage.contains("lireJeuxNeige(m)") && remplissage.contains("document.createElement(\"option\")"),
        "remplirLeMenuNeige n'écrit pas les options d'après lireJeuxNeige(m)"
    );
    assert!(remplissage.contains("texteDuJeuNeige("), "les options n'ont pas le texte de texteDuJeuNeige(…)");

    // Les écouteurs.
    assert!(
        app.contains(&format!("{menu}.addEventListener(\"change\", choisirUnJeuNeige);")),
        "le menu des jeux n'est pas relié à choisirUnJeuNeige par `change`"
    );
    let choisir = corps_de(&app, "choisirUnJeuNeige");
    assert!(
        choisir.contains("poserLesCurseursNeige();") && choisir.contains(&format!("{relancer}();")),
        "choisirUnJeuNeige ne pose pas les curseurs puis ne relance pas"
    );
    assert!(
        app.contains("curseur.addEventListener(\"input\", afficherLesCurseursNeige);"),
        "les curseurs de la neige n'écrivent pas leur valeur à `input`"
    );
    assert!(
        app.contains(&format!("curseur.addEventListener(\"change\", {relancer});")),
        "les curseurs de la neige ne relancent pas à `change`"
    );
    assert!(
        !app.contains(&format!("curseur.addEventListener(\"input\", {relancer})")),
        "un curseur relance à chaque cran (`input`)"
    );
    assert!(
        app.contains("for (const curseur of [")
            && curseurs.iter().all(|c| app[app.find("for (const curseur of [").unwrap_or(0)..].contains(c.as_str())),
        "la boucle des écouteurs ne couvre pas les quatre curseurs"
    );
}

/// forme : la neige se joue sans figer la page et l'empreinte ne la ralentit pas (V-G3-24). `app.js` déclare `const PAS_PAR_APPEL_NEIGE = <entier de 1 à 20>`, employée ; `avancerLaNeige(vitesse)` boucle (`while`) sur des lots `givre_avancer(Math.min(PAS_PAR_APPEL_NEIGE, vitesse - joue))`, lit `performance.now()` et
/// s'arrête quand `BUDGET_IMAGE_MS` sont écoulées (la même constante que le claquage : D3 la borne à 100), s'arrête aussi quand `givre_avancer` rend 0 ; `avancer` joue la neige par `avancerLaNeige(vitesse)` **avant** les autres modèles quand `neigeEnCours` ; l'empreinte de la neige (huit SHA-256 de tout l'état, de l'ordre de
/// 25 ms en natif) n'est pas lue pendant l'animation : `afficherEmpreinte(complete)` écrit `lireEmpreinte(m)` si `!neigeEnCours || complete` et un texte sinon ; `image` l'appelle `afficherEmpreinte(joue === 0)` (calculée à l'arrêt) et `relancer` `afficherEmpreinte(false)`.
///
/// Ne couvre pas : le temps réel d'une image (mesuré sous Node par le pilote) ni que la boucle s'arrête vraiment.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn app_js_joue_la_neige_sans_figer_la_page() {
    let app = code_de_app_js();
    let lot = constante_entiere(&app, "PAS_PAR_APPEL_NEIGE").expect("const PAS_PAR_APPEL_NEIGE = <entier>");
    assert!((1..=20).contains(&lot), "PAS_PAR_APPEL_NEIGE vaut {lot}");
    assert!(app.matches("PAS_PAR_APPEL_NEIGE").count() >= 2, "PAS_PAR_APPEL_NEIGE n'est pas employée");
    let boucle = corps_de(&app, "avancerLaNeige");
    for attendu in [
        "while (",
        "performance.now()",
        "BUDGET_IMAGE_MS",
        "m.givre_avancer(Math.min(PAS_PAR_APPEL_NEIGE, vitesse - joue))",
        "if (lot === 0) {",
    ] {
        assert!(boucle.contains(attendu), "avancerLaNeige n'écrit pas « {attendu} »");
    }
    let avancer = corps_de(&app, "avancer");
    let neige = avancer
        .find("if (neigeEnCours) {\n    return avancerLaNeige(vitesse);")
        .expect("`avancer` joue la neige par avancerLaNeige(vitesse)");
    let autres = avancer.find("m.givre_avancer(vitesse)").expect("les autres modèles par lots de `vitesse`");
    assert!(neige < autres, "avancer joue la neige après les autres modèles");
    let empreinte = corps_de(&app, "afficherEmpreinte");
    assert!(
        empreinte.contains("!neigeEnCours || complete ? lireEmpreinte(m) :"),
        "afficherEmpreinte ne saute pas le calcul de l'empreinte de la neige pendant l'animation"
    );
    let appel = app.find("requestAnimationFrame(").expect("app.js anime");
    let (_, image) = fonction_qui_contient(&app, appel).expect("la fonction d'animation");
    assert!(
        image.contains("afficherEmpreinte(joue === 0);"),
        "l'image de l'animation n'écrit pas `afficherEmpreinte(joue === 0);`"
    );
    let demarre = app.find("givre_demarrer(").expect("app.js démarre un modèle");
    let (_, relance) = fonction_qui_contient(&app, demarre).expect("la fonction de démarrage");
    assert!(relance.contains("afficherEmpreinte(false);"), "le démarrage n'écrit pas `afficherEmpreinte(false);`");
    assert_eq!(
        arites_des_appels(&app, "afficherEmpreinte"),
        [1, 1, 1],
        "afficherEmpreinte : sa déclaration et deux appels, chacun avec un argument"
    );
    assert!(image.contains("texteDeFin()"), "la fin du scénario n'est pas dite par texteDeFin()");
}

/// forme : ce que la page dit de la neige ne prétend pas plus que le modèle (V-G3-25). `legende-neige` dit que le mur est à nous (« le mur est à nous »), que ce ne serait plus la neige de l'article au mur (« plus la neige de l'article »), que ce sont des stades précoces
/// (« stades précoces ») ; `source-neige` est vide dans la page (`app.js` l'écrit d'après le jeu : lu ou déduit) ; le texte visible de la page ne dit ni « ressemble », ni « conforme à l'article », ni « identique à l'article », ni « fidèle », ni « réaliste » ; **l'étiquette est dite « notre classement, pas celui de l'article »** par
/// `lecture.js` (`texteDeLaFormeNeige` : voir `lecture_js_lit_la_neige_sur_un_faux_module`, qui l'écrit en entier) et `app.js` l'écrit par cette fonction.
///
/// Ne couvre pas : ce que la page écrit pendant l'animation (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn la_page_ne_dit_pas_plus_que_le_modele() {
    let html = e5::lire_page("index.html");
    let legende = texte_de(&html, "legende-neige").expect("legende-neige").to_lowercase();
    for mot in ["le mur est à nous", "plus la neige de l'article", "stades précoces"] {
        assert!(legende.contains(mot), "la légende de la neige ne dit pas « {mot} » : « {legende} »");
    }
    assert_eq!(
        texte_de(&html, "source-neige").as_deref(),
        Some(""),
        "source-neige n'est pas vide dans la page : elle est écrite d'après le jeu"
    );
    let visible = texte_visible(&html).to_lowercase();
    for interdit in
        ["ressemble", "conforme à l'article", "identique à l'article", "fidèle", "réaliste", "prouve que la neige"]
    {
        assert!(!visible.contains(interdit), "la page dit « {interdit} »");
    }
    let app = code_de_app_js();
    assert!(
        app.contains("texteDeLaFormeNeige(mesures.forme)"),
        "app.js n'écrit pas l'étiquette par texteDeLaFormeNeige(mesures.forme)"
    );
    assert!(
        code_de_lecture_js().contains("notre classement, pas celui de l'article"),
        "lecture.js ne dit pas « notre classement, pas celui de l'article »"
    );
}

/// forme : `app.js` s'appuie sur `lecture.js` pour ce qui se teste sous Node (V-G3-26). `lecture.js` déclare `export function …(` pour les onze fonctions de la neige (`lireJeuxNeige`, `lireReglagesNeige`, `lireMesuresNeige`, `formaterReglage`, `texteDuJeuNeige`, `texteSourceNeige`, `texteDeLaFormeNeige`,
/// `texteMesuresNeige`, `texteFinNeige`, `texteRefusNeige`, `blocsNeige`) et n'emploie ni `document` ni `window` ; `app.js` les importe de `"./lecture.js"` et chacune est appelée avec le bon nombre d'arguments (1 partout, 2 pour `texteSourceNeige`) ; **le module n'est lu que par `lecture.js`** pour les réglages, les mesures et les
/// jeux : `app.js` n'appelle ni `givre_neige_jeu`, ni `givre_neige_reglage`, ni `givre_neige_mesure`, ni `givre_neige_jeux` (il passe par `lireJeuxNeige`, `lireReglagesNeige` et `lireMesuresNeige`) et `lecture.js` n'appelle pas `givre_neige_demarrer`.
///
/// Ne couvre pas : ce que ces fonctions rendent (voir `lecture_js_lit_la_neige_sur_un_faux_module` et `etape_g3_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn app_js_s_appuie_sur_lecture_js_pour_la_neige() {
    let lecture = code_de_lecture_js();
    let app = code_de_app_js();
    let attendues: [(&str, usize); 11] = [
        ("lireJeuxNeige", 1),
        ("lireReglagesNeige", 1),
        ("lireMesuresNeige", 1),
        ("formaterReglage", 1),
        ("texteDuJeuNeige", 1),
        ("texteSourceNeige", 2),
        ("texteDeLaFormeNeige", 1),
        ("texteMesuresNeige", 1),
        ("texteFinNeige", 1),
        ("texteRefusNeige", 1),
        ("blocsNeige", 1),
    ];
    for (nom, arite) in attendues {
        assert!(
            lecture.contains(&format!("export function {nom}(")),
            "lecture.js ne déclare pas `export function {nom}(`"
        );
        let arites = arites_des_appels(&app, nom);
        assert!(!arites.is_empty(), "app.js n'appelle jamais {nom}(…)");
        assert!(
            arites.iter().all(|a| *a == arite),
            "app.js appelle {nom} avec {arites:?} argument(s) au lieu de {arite}"
        );
        let importe = ["\"./lecture.js\"", "'./lecture.js'"].iter().any(|chemin| {
            app.match_indices(chemin).any(|(debut, _)| {
                let avant = &app[..debut];
                avant.trim_end().ends_with("from") && avant[avant.rfind("import").unwrap_or(0)..].contains(nom)
            })
        });
        assert!(importe, "app.js n'importe pas {nom} de \"./lecture.js\"");
    }
    for interdit in ["document", "window"] {
        assert!(
            !lecture.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).any(|mot| mot == interdit),
            "lecture.js emploie `{interdit}`"
        );
    }
    for lecteur in ["givre_neige_jeu", "givre_neige_reglage", "givre_neige_mesure", "givre_neige_jeux"] {
        assert!(
            arites_des_appels(&app, lecteur).is_empty(),
            "app.js appelle {lecteur}(…) : la lecture du module passe par lecture.js"
        );
        assert!(!arites_des_appels(&lecture, lecteur).is_empty(), "lecture.js n'appelle jamais {lecteur}(…)");
    }
    assert!(
        arites_des_appels(&lecture, "givre_neige_demarrer").is_empty(),
        "lecture.js démarre la neige : seul app.js démarre un modèle"
    );
    // Rien d'autre que la neige n'a changé dans le texte de `dessiner` de V7 (le test de V7 le garde mot pour mot ; on vérifie ici qu'il est encore là).
    let dessiner = corps_de(&app, "dessiner");
    assert!(dessiner.contains("imagesCachees(nette.code)"), "dessiner n'est plus celui de V7");
    let _ = (balises(&e5::lire_page("index.html")), racine());
}

/// juste : les fonctions de `lecture.js` de la neige, sur un **faux module qui change d'état**, disent ce que la doc dit (V-G3-27). Un calcul **à la main** : le faux module (`tests/etape_g3/neige-pure.mjs`) a trois jeux dont les champs sont des formules, et se souvient du dernier démarrage et du temps joué.
/// `lireJeuxNeige` rend les trois jeux avec leurs sept réglages (le champ `4 + k` d'un jeu est le réglage `k`), `lireReglagesNeige` rend `null` avant tout démarrage, **les réglages du dernier démarrage** après un démarrage, ceux du second (pas ceux du premier) après un second, puis `null` quand le module n'a
/// plus de neige ; `lireMesuresNeige` rend `null` avant tout démarrage puis des mesures qui **changent avec le temps joué** (au mur à 53 pas) et reviennent à celles du temps 0 après un nouveau démarrage ; `formaterReglage` écrit les millionièmes en décimal avec une virgule et au moins deux décimales (`0,40`, `0,000001`, `1,09`,
/// `0,0745` : des entiers, aucun arrondi) ; les textes du menu, de la source (lue, déduite, curseurs déplacés), de l'étiquette (« Forme (notre classement, pas celui de l'article) : … », `inconnue` pour un numéro inconnu), des mesures, de la fin (mur ou durée), des refus et `blocsNeige` sont ceux écrits ici. Un réglage lu dans le mauvais ordre,
/// un état gardé d'avant le démarrage suivant, un `>>> 0` oublié (le faux rend `-1` pour `u32::MAX`), ou une étiquette qui n'est pas dite « notre classement » se voient.
///
/// Ne couvre pas : le vrai module (voir `etape_g3_module.rs`) ; ce que `app.js` fait de ces textes (les écrire dans le DOM).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn lecture_js_lit_la_neige_sur_un_faux_module() {
    if !e5::node_est_la() {
        return;
    }
    let lecture = chemin_de_lecture();
    let sortie = lancer_le_script("neige-pure.mjs", &[lecture.as_str()]);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "neige-pure.mjs : {:?}, sortie d'erreur : {erreur}", sortie.status);

    // Les trois jeux du faux module : le jeu `j` a la figure 9, 10 ou 13, le rang 2, 6 ou 1, 6, 6 ou 1 cristaux, le réglage balayé 0, 1 ou 7, `ρ β α θ κ μ γ = 100 + j … 700 + j`, le rayon 100, `1000 + j` pas, déduit `j % 2`.
    let jeux: Vec<String> = [(9, 2, 6, 0), (10, 6, 6, 1), (13, 1, 1, 7)]
        .iter()
        .enumerate()
        .map(|(j, (figure, rang, cristaux, balaye))| {
            let r: Vec<String> = (1..=7).map(|k| (100 * k + j).to_string()).collect();
            format!("{j}:{figure}.{rang}.{cristaux}.{balaye}:{}:100:{}:{}", r.join(","), 1000 + j, j % 2)
        })
        .collect();
    // Les réglages d'un démarrage `(jeu, ρ, β, κ, μ)` : `jeu:100:1000+jeu:ρ,β,α,θ,κ,μ,γ` avec `α = 11 (jeu + 1)`, `θ = 22 (jeu + 1)`, `γ = 33 (jeu + 1)`.
    let reglage = |jeu: u32, rho: u32, beta: u32, kappa: u32, mu: u32| {
        format!(
            "{jeu}:100:{}:{rho},{beta},{},{},{kappa},{mu},{}",
            1000 + jeu,
            11 * (jeu + 1),
            22 * (jeu + 1),
            33 * (jeu + 1)
        )
    };
    // Les mesures au temps `t` : `10 t + 1, t + 2, 500 + t, 7, t mod 5, 1000 − t, t mod 5, (au mur dès t >= 50)`.
    let mesure = |t: u32| {
        format!("{},{},{},7,{},{},{},{}", 10 * t + 1, t + 2, 500 + t, t % 5, 1000 - t, t % 5, u32::from(t >= 50))
    };
    let attendu = [
        format!("jeux={}", jeux.join("|")),
        format!("reglages=null|{}|{}|null", reglage(1, 11, 12, 13, 14), reglage(2, 21, 22, 23, 24)),
        format!("mesures=null|{}|{}|{}|{}", mesure(0), mesure(3), mesure(53), mesure(0)),
        "formater=0,00;0,000001;0,000005;0,00005;0,25;0,40;1,09;1,50;8,00;0,123456;0,999999;1,00;0,0745;0,0001;0,001;2,00".to_string(),
        "jeux_texte=Figure 9 : cristal 2 sur 6 (ρ varie)|Figure 10 : cristal 6 sur 6 (β varie)|Figure 12 : cristal 3 sur 6 (μ varie)|Figure 13 : le cristal du milieu".to_string(),
        "sources=Réglages lus dans l'article (ses figures, son texte et son annexe se recoupent).|Réglages déduits de l'annexe de l'article : la colonne du réglage qui varie y manque.|\
Curseurs déplacés : ρ, β, κ et μ ne sont plus ceux de la figure ; α, θ, γ, la taille du domaine et le nombre de pas restent ceux du jeu.|\
Curseurs déplacés : ρ, β, κ et μ ne sont plus ceux de la figure ; α, θ, γ, la taille du domaine et le nombre de pas restent ceux du jeu."
            .to_string(),
        "formes=Forme (notre classement, pas celui de l'article) : trop petit pour être rangé.|Forme (notre classement, pas celui de l'article) : plaque.|\
Forme (notre classement, pas celui de l'article) : plaque à secteurs.|Forme (notre classement, pas celui de l'article) : dendrite.|\
Forme (notre classement, pas celui de l'article) : fougère.|Forme (notre classement, pas celui de l'article) : inconnue."
            .to_string(),
        "textes=1687 cases · rayon 26 · compacité 0,800 · pointes 18 · aiguilles 0 · hexagonalité 0,500|1 cases · rayon 0 · compacité 1,000 · pointes 1 · aiguilles 1 · hexagonalité 1,000|\
4639 cases · rayon 59 · compacité 0,005 · pointes 120 · aiguilles 12 · hexagonalité 0,045"
            .to_string(),
        "fins=Arrêté près du mur du domaine : au-delà, ce ne serait plus la neige de l'article (le mur est à nous, et pour certains réglages le cristal le ressent déjà). Recommencez ou choisissez un autre modèle.|\
Scénario terminé (tous les pas du jeu sont joués) : recommencez ou choisissez un autre modèle."
            .to_string(),
        "refus=La neige ne démarre pas : jeu inconnu (code 1).|La neige ne démarre pas : ρ hors plage (code 2).|La neige ne démarre pas : β hors plage (code 3).|\
La neige ne démarre pas : κ hors plage (code 4).|La neige ne démarre pas : μ hors plage (code 5).|La neige ne démarre pas : refus du module (code 6).|\
La neige ne démarre pas : refus du module (code 9)."
            .to_string(),
        "blocs=false,false|true,true".to_string(),
    ]
    .join("\n")
        + "\n";
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu);

    for arguments in [&[][..], &["a", "b"], &["n-existe-pas.js"]] {
        let sortie = lancer_le_script("neige-pure.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "neige-pure.mjs {arguments:?}");
        assert!(sortie.stdout.is_empty(), "neige-pure.mjs {arguments:?} : la sortie standard n'est pas vide");
    }
    let script = std::fs::read_to_string(racine().join("tests").join("etape_g3").join("neige-pure.mjs"))
        .expect("le script se lit");
    let mut node = Command::new("node")
        .args(["--input-type=module", "--check"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node se lance");
    node.stdin.take().expect("entrée standard de Node").write_all(script.as_bytes()).expect("écriture du script");
    let sortie = node.wait_with_output().expect("Node se termine");
    assert!(sortie.status.success(), "neige-pure.mjs n'est pas valide : {}", String::from_utf8_lossy(&sortie.stderr));
}

/// Un texte sans ses blancs de mise en page : chaque suite d'espaces, de tabulations et de sauts de ligne devient une espace.
fn aux_espaces_pres(texte: &str) -> String {
    texte.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// forme : les fonctions de `app.js` qui portent la neige sont celles du prototype, **mot pour mot aux espaces près** (V-G3-28, ajouté par le juge). Les tests de forme ci-dessus exigent des morceaux de ces fonctions ; onze fautes de page
/// plausibles les passaient tous (les panneaux montrés **avant** que `neigeEnCours` soit posé : le panneau du modèle d'avant reste à l'écran ; le texte de fin ou le texte d'état qui suivent le choix et non le modèle démarré ; une graine invalide qui
/// empêche la neige de démarrer ; `κ` et `μ` échangés dans la phrase « Démarrée avec » ou dans les curseurs posés ; la valeur affichée de `ρ` qui est celle de `β` ; la valeur d'une option du menu qui est le rang et non l'indice du jeu ; un jeu choisi
/// qui démarre avec les curseurs d'avant ; le menu jamais rempli ; « curseurs déplacés » à l'envers). La page se recopie à l'octet (son `sha256` est dans le plan) : ce test dit laquelle des quinze fonctions a bougé. Il exige aussi que le chargement
/// remplisse le menu (`remplirLeMenuNeige();`) **avant** le premier `relancer();`, une fois chacun.
///
/// Ne couvre pas : ce que le navigateur fait de ce texte (le DOM n'est pas éprouvé : `MODELE.md`, « Ce que cette étape n'établit pas ») ; le reste de `app.js` (les tests des étapes 5, Z1, D3 et V7).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn app_js_ecrit_la_neige_mot_pour_mot() {
    let app = code_de_app_js();
    let attendus: [(&str, &str); 15] = [
        (
            "afficherEmpreinte",
            r##"{
  affichageEmpreinte.textContent = !neigeEnCours || complete ? lireEmpreinte(m) : "calculée à l'arrêt";
}"##,
        ),
        (
            "afficherMesures",
            r##"{
  const sites = m.givre_sites() >>> 0;
  const rayon = m.givre_rayon() >>> 0;
  const compacite = m.givre_compacite();
  const complement = Number.isNaN(compacite) ? "" : ` · compacité ${compacite.toFixed(4)}`;
  affichageMesures.textContent = `${sites} sites · rayon ${rayon}${complement}`;
  if (neigeEnCours) {
    afficherLaNeige();
  }
}"##,
        ),
        (
            "afficherLaNeige",
            r##"{
  const reglages = lireReglagesNeige(m);
  const mesures = lireMesuresNeige(m);
  if (reglages === null || mesures === null) {
    return;
  }
  const r = reglages.reglages;
  affichageReglagesNeige.textContent = `Démarrée avec ρ ${formaterReglage(r.rho)} · β ${formaterReglage(r.beta)} · κ ${formaterReglage(r.kappa)} · μ ${formaterReglage(r.mu)} (et α ${formaterReglage(r.alpha)} · θ ${formaterReglage(r.theta)} · γ ${formaterReglage(r.gamma)} du jeu), ${reglages.pas} pas au plus.`;
  affichageMesuresNeige.textContent = texteMesuresNeige(mesures);
  affichageFormeNeige.textContent = texteDeLaFormeNeige(mesures.forme);
}"##,
        ),
        (
            "afficherLesCurseursNeige",
            r##"{
  valeurNeigeRho.textContent = formaterReglage(Number(curseurNeigeRho.value));
  valeurNeigeBeta.textContent = formaterReglage(Number(curseurNeigeBeta.value));
  valeurNeigeKappa.textContent = formaterReglage(Number(curseurNeigeKappa.value));
  valeurNeigeMu.textContent = formaterReglage(Number(curseurNeigeMu.value));
  const jeu = jeuxNeige[Number(menuJeuNeige.value)];
  if (jeu !== undefined) {
    const modifie =
      curseurNeigeRho.value !== String(jeu.reglages.rho) ||
      curseurNeigeBeta.value !== String(jeu.reglages.beta) ||
      curseurNeigeKappa.value !== String(jeu.reglages.kappa) ||
      curseurNeigeMu.value !== String(jeu.reglages.mu);
    affichageSourceNeige.textContent = texteSourceNeige(jeu, modifie);
  }
}"##,
        ),
        (
            "remplirLeMenuNeige",
            r##"{
  jeuxNeige = lireJeuxNeige(m);
  for (const jeu of jeuxNeige) {
    const option = document.createElement("option");
    option.value = String(jeu.indice);
    option.textContent = texteDuJeuNeige(jeu);
    menuJeuNeige.append(option);
  }
  poserLesCurseursNeige();
}"##,
        ),
        (
            "poserLesCurseursNeige",
            r##"{
  const jeu = jeuxNeige[Number(menuJeuNeige.value)];
  if (jeu === undefined) {
    return;
  }
  curseurNeigeRho.value = String(jeu.reglages.rho);
  curseurNeigeBeta.value = String(jeu.reglages.beta);
  curseurNeigeKappa.value = String(jeu.reglages.kappa);
  curseurNeigeMu.value = String(jeu.reglages.mu);
  afficherLesCurseursNeige();
}"##,
        ),
        (
            "demarrerLaNeige",
            r##"{
  return (
    m.givre_neige_demarrer(
      Number(menuJeuNeige.value),
      Number(curseurNeigeRho.value),
      Number(curseurNeigeBeta.value),
      Number(curseurNeigeKappa.value),
      Number(curseurNeigeMu.value),
    ) >>> 0
  );
}"##,
        ),
        (
            "texteDeFin",
            r##"{
  const mesures = neigeEnCours ? lireMesuresNeige(m) : null;
  return mesures === null ? "Scénario terminé : recommencez ou choisissez un autre modèle." : texteFinNeige(mesures);
}"##,
        ),
        (
            "montrerLesReperes",
            r##"{
  const caches = blocsCaches(choixModele.value);
  blocZinc.hidden = caches.zinc;
  blocRepereEta.hidden = caches.repereEta;
  commandeEta.hidden = caches.commandeEta;
  const cachesNeige = blocsNeige(neigeEnCours);
  commandesNeige.hidden = cachesNeige.commandes;
  blocMesuresNeige.hidden = cachesNeige.mesures;
}"##,
        ),
        (
            "avancer",
            r##"{
  const vitesse = Number(curseurVitesse.value);
  if (neigeEnCours) {
    return avancerLaNeige(vitesse);
  }
  if (!claquageEnCours) {
    return m.givre_avancer(vitesse) >>> 0;
  }
  const debut = performance.now();
  let joue = 0;
  while (joue < vitesse) {
    const unite = m.givre_avancer(1) >>> 0;
    if (unite === 0) {
      break;
    }
    joue += unite;
    if (performance.now() - debut >= BUDGET_IMAGE_MS) {
      break;
    }
  }
  return joue;
}"##,
        ),
        (
            "avancerLaNeige",
            r##"{
  const debut = performance.now();
  let joue = 0;
  while (joue < vitesse) {
    const lot = m.givre_avancer(Math.min(PAS_PAR_APPEL_NEIGE, vitesse - joue)) >>> 0;
    if (lot === 0) {
      break;
    }
    joue += lot;
    if (performance.now() - debut >= BUDGET_IMAGE_MS) {
      break;
    }
  }
  return joue;
}"##,
        ),
        (
            "image",
            r##"{
  if (!enCours) {
    return;
  }
  const joue = avancer();
  dessiner(joue === 0);
  afficherTemps();
  if (joue === 0 || maintenant - derniereEmpreinte >= INTERVALLE_EMPREINTE_MS) {
    afficherEmpreinte(joue === 0);
    afficherMesures();
    derniereEmpreinte = maintenant;
  }
  if (joue === 0) {
    enCours = false;
    affichageEtat.textContent = texteDeFin();
    return;
  }
  requestAnimationFrame(image);
}"##,
        ),
        (
            "texteDEtat",
            r##"{
  if (neigeEnCours) {
    return "Neige de Gravner et Griffeath (pas de hasard : la graine ne sert pas).";
  }
  if (claquageEnCours) {
    return `Claquage diélectrique, η = ${etaDuModele}, graine ${graine}.`;
  }
  return modele === 0 ? `DLA, graine ${graine}.` : "Flocon de Reiter (la graine ne sert qu'à la DLA).";
}"##,
        ),
        (
            "relancer",
            r##"{
  if (m === null) {
    return;
  }
  const neige = neigeChoisie();
  const graine = lireGraine(champGraine.value);
  if (!neige && graine === null) {
    affichageEtat.textContent = "Graine invalide : un entier de 0 à 4294967295.";
    return;
  }
  const modele = neige ? -1 : numeroDuModele(choixModele.value, etaChoisi());
  if (neige) {
    const code = demarrerLaNeige();
    if (code !== 0) {
      affichageEtat.textContent = texteRefusNeige(code);
      return;
    }
  } else if ((m.givre_demarrer(modele, graine) >>> 0) !== 1) {
    affichageEtat.textContent = "Modèle inconnu.";
    return;
  }
  claquageEnCours = claquageChoisi();
  etaDuModele = etaChoisi();
  neigeEnCours = neige;
  montrerLesReperes();
  dessiner(false);
  afficherTemps();
  afficherEmpreinte(false);
  afficherMesures();
  effacerMesure();
  derniereEmpreinte = performance.now();
  affichageEtat.textContent = texteDEtat(modele, graine);
  if (!enCours) {
    enCours = true;
    requestAnimationFrame(image);
  }
}"##,
        ),
        (
            "choisirUnJeuNeige",
            r##"{
  poserLesCurseursNeige();
  relancer();
}"##,
        ),
    ];
    for (nom, attendu) in attendus {
        assert_eq!(
            aux_espaces_pres(&corps_de(&app, nom)),
            aux_espaces_pres(attendu),
            "le corps de « {nom} » dans app.js n'est pas celui du prototype (aux espaces près)"
        );
    }
    let chargement = corps_de(&app, "charger");
    assert_eq!(chargement.matches("remplirLeMenuNeige();").count(), 1, "charger remplit le menu des jeux une fois");
    assert_eq!(
        chargement.matches("relancer();").count(),
        2,
        "charger relance à la touche Entrée de la graine et à la fin"
    );
    let menu = chargement.find("remplirLeMenuNeige();").expect("le menu");
    let depart = chargement.rfind("relancer();").expect("le premier démarrage");
    assert!(menu < depart, "le menu des jeux est rempli après le premier démarrage");
    assert_eq!(
        aux_espaces_pres(&chargement[depart..]),
        "relancer(); }",
        "le chargement finit par le premier démarrage : `relancer();`"
    );
    assert_eq!(app.matches("remplirLeMenuNeige()").count(), 2, "remplirLeMenuNeige : sa déclaration et un seul appel");
}

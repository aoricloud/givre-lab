//! Étape G3 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) montre la neige de Gravner et Griffeath : un panneau (un menu de jeux nommés lus dans le module, quatre curseurs), ses mesures et son étiquette. Valeurs attendues :
//! `references/etape-g3-valeurs.md`, identifiants `V-G3-xx`. Comme aux étapes 5, Z1, D3 et V7, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_js_lit_la_neige_sur_un_faux_module` appelle les fonctions de `lecture.js` de la neige sur un faux module qui **change d'état** et compare à un calcul fait à la main ; il ne lit pas `docs/givre.wasm`.
//!
//! **Étape V8 : la page est tous les fichiers de son dossier** (voir `etape_5_page.rs`) : ces tests lisent `index.html`, ses scripts en ligne et **tous** les fichiers JavaScript ; **le « mot pour mot » de quinze fonctions d'`app.js` (V-G3-28) est retiré** : la page de la session de site calcule dans des workers, avec un tiroir de réglages ; ce que les tests de
//! forme exigeaient de l'écriture d'`app.js` (l'état suit le modèle démarré, pas les curseurs ; pas de résidu d'un modèle à l'autre ; un refus du module est affiché) ne se voit plus ici : `MODELE.md`, étape V8, « Ce que cette étape n'établit plus ».
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : celle de G3 ; V8 en a retiré ce qui n'était que l'écriture d'`app.js`)
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
    arites_des_appels, attribut, balise_d_id, balises, code_de_app_js, code_de_lecture_js, constante_entiere, e5,
    nom_de_balise, texte_de,
};
use etape_g3_commun::{JEUX, chemin_de_lecture, lancer_le_script, racine};

/// forme : `index.html` a le panneau de la neige (V-G3-20). Le choix du modèle a une sixième option de `value` `neige` ; l'élément `commandes-neige` (**de n'importe quelle sorte, à n'importe quelle place** : V8) existe ; il y a un `<select id="jeu-neige">` **sans aucune `<option>`** (la page n'écrit pas la liste
/// des jeux : `app.js` la tire du module), quatre `<input type="range">` `neige-rho`, `neige-beta`, `neige-kappa` et `neige-mu` chacun avec `min`, `max` et `step` **entiers** et un `<output>` (`valeur-neige-rho`, …) ; les éléments `source-neige` et `legende-neige` ; `mesures-neige-bloc`, `reglages-neige`, `mesures-neige` et `forme-neige` existent ; **aucun curseur n'a de valeur initiale** (`value` : une valeur écrite ici serait un réglage de la page). **V8 ne demande plus** que le panneau et le bloc des mesures soient cachés au départ (la page rhabillée ouvre sur la neige), ni la sorte des éléments, ni le texte de `legende-nette`.
///
/// Ne couvre pas : que le panneau soit montré ou caché au bon moment (le DOM n'est pas éprouvé : voir `MODELE.md`, étape V8).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn la_page_a_le_panneau_de_la_neige() {
    let html = e5::lire_page("index.html");
    for (id, sorte) in [
        ("jeu-neige", "select"),
        ("neige-rho", "input"),
        ("neige-beta", "input"),
        ("neige-kappa", "input"),
        ("neige-mu", "input"),
        ("valeur-neige-rho", "output"),
        ("valeur-neige-beta", "output"),
        ("valeur-neige-kappa", "output"),
        ("valeur-neige-mu", "output"),
    ] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    // V8 : ces éléments existent, **de n'importe quelle sorte et à n'importe quelle place** (un tiroir), visibles ou cachés au départ (la page rhabillée ouvre sur la neige).
    for id in [
        "commandes-neige",
        "source-neige",
        "legende-neige",
        "mesures-neige-bloc",
        "reglages-neige",
        "mesures-neige",
        "forme-neige",
    ] {
        assert!(balise_d_id(&html, id).is_some(), "pas d'élément d'id « {id} »");
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

/// Le nombre écrit `texte` (`0.0745`, `0,0745`, `.4`, `1.09`, `400000`) en millionièmes ; `None` si ce n'est pas un nombre décimal d'au plus six chiffres après la virgule (un exposant, une fraction, un mot).
fn en_millionniemes(texte: &str) -> Option<u64> {
    let texte = texte.trim().replace(',', ".");
    let (entier, fraction) = texte.split_once('.').unwrap_or((texte.as_str(), ""));
    if fraction.len() > 6
        || !entier.chars().all(|c| c.is_ascii_digit())
        || !fraction.chars().all(|c| c.is_ascii_digit())
        || (entier.is_empty() && fraction.is_empty())
    {
        return None;
    }
    let mut valeur = entier.parse::<u64>().unwrap_or(0) * 1_000_000;
    valeur += format!("{fraction:0<6}").parse::<u64>().ok()?;
    Some(valeur)
}

/// Vrai si `texte` contient le nombre `ecriture` comme **un nombre entier de lui-même** : ni précédé d'un chiffre, d'un `.` ou d'une `,`, ni suivi d'un chiffre (`0.003` n'est pas dans `0.0035` ni dans `10.003`), **ni suivi d'une lettre ou de `%`** : `-0.025em`, `0.4s`, `0.5px` sont des longueurs et des durées de la feuille de style, pas des réglages
/// (le guide des vitrines demande une approche de `−0,025 em` pour le titre : ce n'est pas le `κ` de la figure 12).
fn contient_le_nombre(texte: &str, ecriture: &str) -> bool {
    texte.match_indices(ecriture).any(|(debut, _)| {
        let avant = texte[..debut].chars().next_back();
        let apres = texte[debut + ecriture.len()..].chars().next();
        !avant.is_some_and(|c| c.is_ascii_digit() || c == '.' || c == ',')
            && !apres.is_some_and(|c| c.is_ascii_alphanumeric() || c == '%')
    })
}

/// Les sept réglages des dix-neuf jeux (`etape_g3_commun::JEUX`, en millionièmes), sans doublon.
fn tous_les_reglages_des_jeux() -> std::collections::BTreeSet<u64> {
    JEUX.iter().flat_map(|jeu| jeu.reglages.iter().map(|&r| u64::from(r))).collect()
}

/// forme : **aucun réglage d'un jeu nommé n'est écrit dans la page** (V-G3-22, étendu par V8 : tous les fichiers et tous les jeux). Deux détecteurs, sur **tous les fichiers de texte du dossier de la page** (HTML, scripts en ligne, CSS, JavaScript, SVG ; les `data:` rendus vides) : (1) **aucun** des décimaux des réglages qui ne se confondent avec aucune autre
/// constante d'une page (`0.0745`, `0.00005`, `0.00001`, `0.025`, `0.004`, `0.015`, `1.09`, `0.003`, `0.0001` **et `0.005`**, le `θ` de la figure 12 que la relecture de G3 a trouvé oublié), écrits avec un point ou une virgule, commentaires compris ; (2) **dans le code JavaScript** (commentaires exclus), un nom de réglage (`rho`, `beta`, `kappa`, `mu`, `theta`, `ρ`, `β`, `κ`,
/// `μ`, `θ`, en majuscules ou en minuscules, dans un nom composé : `neigeRho`, `MU`) **suivi de `:` ou de `=` et d'un nombre qui vaut un réglage de l'un des dix-neuf jeux** (sur les sept réglages, décimal ou en millionièmes : `mu: 0.07`, `const beta = 1.9`, `kappa = 1000`) est refusé : y compris la valeur balayée d'une figure (`ρ = 0,42`, `μ = 0,09`) et les réglages de la figure 12 (`1,4`, `0,005`, `0,001`).
/// `givre_neige_demarrer(` est appelée **avec cinq arguments dont les quatre derniers (`ρ`, `β`, `κ`, `μ`) ne commencent pas par un chiffre** (le jeu, lui, peut être écrit : une planche de six cristaux démarre les jeux 6 à 11) ; la page demande les jeux au module (`lireJeuxNeige(` est appelée quelque part). Aucune valeur publiée de résultat (`1,71`, `1,433`, `1,263`).
/// V8 ne demande plus « un seul appel de `givre_neige_demarrer` » : un worker par planche peut démarrer sa neige.
///
/// Ne couvre pas : un réglage écrit sous une autre forme (une fraction, un produit, `4e-1`) ; un réglage commun avec une constante de dessin (`0.5`, `0.05`, `0.1` : seul un nom de réglage devant un `:` ou un `=` le voit) ; les valeurs affichées à l'écran (le DOM n'est pas éprouvé).
/// Fiche : V-V8-36 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn aucun_reglage_n_est_ecrit_dans_la_page() {
    let fichiers = e5::textes_des_fichiers(&["html", "htm", "js", "mjs", "css", "svg"]);
    assert!(fichiers.iter().any(|(nom, _)| nom == "index.html"), "index.html n'est pas lu");
    for (nom, texte) in &fichiers {
        let texte = e5::sans_donnees(texte);
        for decimal in ["0.0745", "0.00005", "0.00001", "0.025", "0.004", "0.015", "1.09", "0.003", "0.0001", "0.005"] {
            for ecriture in [decimal.to_string(), decimal.replace('.', ",")] {
                assert!(
                    !contient_le_nombre(&texte, &ecriture),
                    "docs/{nom} écrit le réglage « {ecriture} » d'un jeu nommé : il vient du module"
                );
            }
        }
        for interdit in ["1,71", "1.71", "1,433", "1.433", "1,263", "1.263"] {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} »");
        }
    }
    // (2) Un nom de réglage suivi d'un nombre qui vaut un réglage d'un jeu, dans le code.
    let valeurs = tous_les_reglages_des_jeux();
    for (nom, code) in e5::code_js_de_la_page() {
        let minuscules = code.to_lowercase();
        for reglage in ["rho", "beta", "kappa", "mu", "theta", "ρ", "β", "κ", "μ", "θ"] {
            for (position, _) in minuscules.match_indices(reglage) {
                let apres = minuscules[position + reglage.len()..].trim_start_matches(['"', '\'', '`', ' ', '\t']);
                let Some(suite) = apres.strip_prefix([':', '=']).filter(|s| !s.starts_with('=')) else { continue };
                let nombre: String =
                    suite.trim_start().chars().take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
                // La ponctuation qui suit le nombre dans une phrase n'en fait pas partie (« vapeur ρ = 0,8, seuil β » ; « ρ = 0,8. »).
                let nombre = nombre.trim_end_matches(['.', ',']);
                // Un nombre à virgule est un décimal ; un entier est lu en millionièmes (`mu: 70000`).
                let valeur =
                    if nombre.contains(['.', ',']) { en_millionniemes(nombre) } else { nombre.parse::<u64>().ok() };
                if let Some(valeur) = valeur {
                    assert!(
                        !valeurs.contains(&valeur),
                        "{nom} écrit un réglage d'un jeu nommé : « {reglage} … {nombre} » : il vient du module"
                    );
                }
            }
        }
    }
    // L'appel de `givre_neige_demarrer` : jamais un réglage écrit.
    let code = code_de_app_js();
    for (position, _) in code.match_indices("givre_neige_demarrer(") {
        if code[..position].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let debut = position + "givre_neige_demarrer(".len();
        let mut profondeur = 0_i32;
        let mut arguments: Vec<String> = vec![String::new()];
        for c in code[debut..].chars() {
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
        for argument in &arguments[1..] {
            assert!(
                !argument.starts_with(|c: char| c.is_ascii_digit()),
                "un réglage écrit dans l'appel : « {argument} »"
            );
        }
    }
    assert!(
        !arites_des_appels(&code, "lireJeuxNeige").is_empty(),
        "la page ne lit pas les jeux nommés dans le module (lireJeuxNeige(m))"
    );
}

/// forme : les bornes de la boucle de la neige (V-G3-24). **Si** une constante `PAS_PAR_APPEL_NEIGE` est déclarée dans le code de la page (`const PAS_PAR_APPEL_NEIGE = <entier>`), elle vaut de 1 à 20 ; chaque appel de `givre_avancer(` a un seul argument. **V8 ne lit plus** `avancerLaNeige`, `avancer` ni `afficherEmpreinte` (le corps de fonctions d'`app.js` : des lots
/// de `PAS_PAR_APPEL_NEIGE` pas sous un budget de temps, `!neigeEnCours || complete ? lireEmpreinte(m) :`, `afficherEmpreinte(joue === 0)`) : la page de la session de site joue la neige dans un worker, hors du fil de la page. Que la page ne se fige pas et que l'empreinte (huit SHA-256 de tout l'état) ne soit calculée qu'à l'arrêt ne se voit plus ici : voir `MODELE.md`, étape V8.
///
/// Ne couvre pas : le temps réel d'une image (mesuré sous Node par le pilote) ni que la boucle s'arrête vraiment ; que l'empreinte soit lue rarement.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn les_bornes_de_la_boucle_de_la_neige() {
    let app = code_de_app_js();
    if let Some(lot) = constante_entiere(&app, "PAS_PAR_APPEL_NEIGE") {
        assert!((1..=20).contains(&lot), "PAS_PAR_APPEL_NEIGE vaut {lot}");
    }
    for arite in arites_des_appels(&app, "givre_avancer") {
        assert_eq!(arite, 1, "givre_avancer est appelée avec {arite} arguments");
    }
}

/// forme : ce que la page dit de la neige ne prétend pas plus que le modèle (V-G3-25). `legende-neige` dit que le mur est à nous (« le mur est à nous »), que ce ne serait plus la neige de l'article au mur (« plus la neige de l'article »), que ce sont des stades précoces (« stades précoces ») ; `source-neige` est vide dans la page (une phrase écrite
/// d'avance dirait « lu » ou « déduit » sans savoir le jeu : un script l'écrit d'après le jeu) ; **tout ce que la page écrit** (le texte visible de chaque fichier HTML, les chaînes de caractères de tous les fichiers JavaScript, le texte des SVG) ne dit ni « ressemble », ni « conforme à l'article », ni « identique à l'article », ni « fidèle », ni « réaliste », ni « prouve que la neige » ;
/// **l'étiquette est dite « notre classement, pas celui de l'article »** par `lecture.js` (`texteDeLaFormeNeige` : voir `lecture_js_lit_la_neige_sur_un_faux_module`, qui l'écrit en entier) et **un fichier de la page autre que `lecture.js` l'appelle** (`texteDeLaFormeNeige(`, un seul argument). **V8 : la légende peut être écrite par un script** : si `legende-neige` est vide dans `index.html`, chacune des
/// trois phrases doit être une chaîne de caractères de l'un des fichiers JavaScript ; si elle a un texte, c'est lui qui est jugé.
///
/// Ne couvre pas : ce que la page écrit pendant l'animation (le DOM n'est pas éprouvé) ; que le script écrive la légende dans le bon élément.
/// Fiche : V-V8-34 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn la_page_ne_dit_pas_plus_que_le_modele() {
    let html = e5::lire_page("index.html");
    assert!(balise_d_id(&html, "legende-neige").is_some(), "pas d'élément d'id « legende-neige »");
    let statique = texte_de(&html, "legende-neige").unwrap_or_default().to_lowercase();
    let chaines: Vec<String> = e5::chaines_du_code(&e5::tout_le_code_js()).iter().map(|c| c.to_lowercase()).collect();
    for mot in ["le mur est à nous", "plus la neige de l'article", "stades précoces"] {
        let dit = if statique.is_empty() { chaines.iter().any(|c| c.contains(mot)) } else { statique.contains(mot) };
        assert!(dit, "la légende de la neige ne dit pas « {mot} » : « {statique} »");
    }
    assert_eq!(
        texte_de(&html, "source-neige").as_deref(),
        Some(""),
        "source-neige n'est pas vide dans la page : elle est écrite d'après le jeu"
    );
    let ecrit = e5::textes_ecrits_par_la_page().to_lowercase();
    for interdit in
        ["ressemble", "conforme à l'article", "identique à l'article", "fidèle", "réaliste", "prouve que la neige"]
    {
        assert!(!ecrit.contains(interdit), "la page dit « {interdit} »");
    }
    let hors_lecture = e5::code_js_hors_lecture();
    let arites = arites_des_appels(&hors_lecture, "texteDeLaFormeNeige");
    assert!(
        !arites.is_empty() && arites.iter().all(|a| *a == 1),
        "aucun fichier de la page, hors lecture.js, n'écrit l'étiquette par texteDeLaFormeNeige(forme) : {arites:?}"
    );
    assert!(
        code_de_lecture_js().contains("notre classement, pas celui de l'article"),
        "lecture.js ne dit pas « notre classement, pas celui de l'article »"
    );
}

/// forme : ce que la page lit de la neige passe par `lecture.js`, que Node éprouve (V-G3-26). `lecture.js` déclare `export function …(` pour les onze fonctions de la neige (`lireJeuxNeige`, `lireReglagesNeige`, `lireMesuresNeige`, `formaterReglage`, `texteDuJeuNeige`, `texteSourceNeige`, `texteDeLaFormeNeige`, `texteMesuresNeige`, `texteFinNeige`, `texteRefusNeige`, `blocsNeige`) et n'emploie ni `document` ni `window` ;
/// **chaque appel** d'une de ces fonctions dans la page (tous les fichiers) a le bon nombre d'arguments (1 partout, 2 pour `texteSourceNeige`) ; **un fichier de la page autre que `lecture.js`** appelle `lireJeuxNeige(` (le menu vient du module), `texteRefusNeige(` (**un refus du module est affiché**) et `texteDeLaFormeNeige(` (l'étiquette) ; **le module n'est lu que par `lecture.js`** pour les réglages, les mesures et
/// les jeux : aucun fichier de la page **hors `lecture.js`** n'appelle `givre_neige_jeu`, `givre_neige_reglage`, `givre_neige_mesure` ni `givre_neige_jeux` ; `lecture.js` n'appelle pas `givre_neige_demarrer` (seule la page démarre un modèle). **V8 ne lit plus** : que `app.js` importe ces onze fonctions et les appelle toutes, ni le corps de `dessiner` de V7.
///
/// Ne couvre pas : ce que ces fonctions rendent (voir `lecture_js_lit_la_neige_sur_un_faux_module` et `etape_g3_module.rs`) ; que le refus soit affiché au bon endroit.
#[test]
#[cfg_attr(not(feature = "etape-g3"), ignore)]
fn lecture_js_declare_ce_que_la_page_lit_de_la_neige() {
    let lecture = code_de_lecture_js();
    let app = code_de_app_js();
    let hors_lecture = e5::code_js_hors_lecture();
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
        assert!(
            arites.iter().all(|a| *a == arite),
            "la page appelle {nom} avec {arites:?} argument(s) au lieu de {arite}"
        );
    }
    for nom in ["lireJeuxNeige", "texteRefusNeige", "texteDeLaFormeNeige"] {
        assert!(
            !arites_des_appels(&hors_lecture, nom).is_empty(),
            "aucun fichier de la page, hors lecture.js, n'appelle {nom}(…)"
        );
    }
    for interdit in ["document", "window"] {
        assert!(
            !lecture.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).any(|mot| mot == interdit),
            "lecture.js emploie `{interdit}`"
        );
    }
    for lecteur in ["givre_neige_jeu", "givre_neige_reglage", "givre_neige_mesure", "givre_neige_jeux"] {
        assert!(
            arites_des_appels(&hors_lecture, lecteur).is_empty(),
            "un fichier de la page, hors lecture.js, appelle {lecteur}(…) : la lecture du module passe par lecture.js"
        );
        assert!(!arites_des_appels(&lecture, lecteur).is_empty(), "lecture.js n'appelle jamais {lecteur}(…)");
    }
    assert!(
        arites_des_appels(&lecture, "givre_neige_demarrer").is_empty(),
        "lecture.js démarre la neige : seule la page démarre un modèle"
    );
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

//! Étape P4 : la page montre le métal (la dendrite à quatre bras du champ de phase) : une septième option du choix du modèle, un panneau (un curseur `ε`, un choix du côté), un canvas lisse, ses mesures et ses textes. Valeurs
//! attendues : `references/etape-p4-valeurs.md`, identifiants `V-P4-xx`. Comme aux étapes 5 à V8, presque tous les tests sont des **tests de forme** : ils lisent **tous les fichiers** du dossier de la page (`docs/`, ou `GIVRE_DOCS` pour les
//! pages d'essai de `tests/etape_p4/pages/`) ; un seul fait tourner du code de la page sans le module : `lecture_js_lit_le_metal_sur_un_faux_module` (sous Node) ; la lecture du **vrai** module et l'image tirée de ses champs sont dans
//! `etape_p4_module.rs`.
//!
//! # Ce que fait la page (la consigne des mains, que ces tests ne lisent que par la forme et par `lecture.js`)
//!
//! - `index.html` : en gardant tout ce que les étapes d'avant demandent, une **septième option** `metal` (« Métal (dendrite) ») du choix du modèle, après `neige` ; un panneau `commandes-metal` (n'importe quel élément) avec un curseur
//!   `<input id="metal-epsilon" type="range">` (en **millionièmes**, `min`, `max`, `step` et `value` entiers, dans la plage du module et sur la grille que le juge a jouée), son `<output id="valeur-metal-epsilon" for="metal-epsilon">`, un
//!   `<select id="metal-cote">` (les côtés du quart que le juge a joués) ; un `<canvas id="vitrine-metal">` (jamais pixélisé : seule la règle de `#vitrine` peut l'être, voir `etape_v7_page.rs`) ; une légende `legende-metal` qui dit
//!   ce que le modèle est et n'est pas ; un bloc `mesures-metal-bloc` avec `reglages-metal` et `mesures-metal`.
//! - `lecture.js` exporte de plus (sans DOM) `lireReglagesMetal(m)`, `lireMesuresMetal(m)`, `lireChampsMetal(m)`, `imageDuMetal(champs, taille)`, `formaterMetal(x, decimales)`, `texteReglagesMetal(r)`, `texteMesuresMetal(m)`,
//!   `texteFinMetal(m)`, `texteRefusMetal(code)` et `blocsMetal(metal)`.
//! - `app.js` (ou tout autre fichier de la page) : **l'état suit le modèle démarré, pas les commandes** (le métal ne tourne que si `givre_metal_demarrer` a rendu 0 ; ce que la page écrit du métal vient du module, par
//!   `lireReglagesMetal` et `lireMesuresMetal`) ; un refus est affiché (`texteRefusMetal`) ; le métal est joué par lots sous un budget de temps (une image ne fige pas la page), dessiné par `imageDuMetal` sur son canvas, les deux
//!   autres canvas cachés ; **pas de résidu** d'un modèle à l'autre. Ces règles d'`app.js` ne sont lues ici que par leur forme (V8 : la page peut être réécrite) : voir « Ce que cette étape n'établit pas » dans `MODELE.md`.
//!
//! Étiquettes : `lecture_js_lit_le_metal_sur_un_faux_module` est **juste** (comparé à un calcul fait à la main et à l'image de la doc réécrite en Rust) ; `les_reglages_offerts_ont_ete_joues` compare la forme de la page à la table des
//! scénarios joués par le juge (**juste** pour la table, **forme** pour la page) ; tous les autres sont de la **forme**, jamais une preuve du comportement de la page.

mod etape_p4_commun;

use std::collections::BTreeSet;

use etape_p4_commun::v7::{
    arites_des_appels, attribut, balise_d_id, balises, code_de_app_js, code_de_lecture_js, constante_entiere,
    nom_de_balise, texte_de,
};
use etape_p4_commun::{COTE_MAX, COTE_MIN, EPSILON_MAX_PPM, chemin_de_lecture, e5, image_du_metal, lancer_le_script};

/// Les identifiants du métal dans `index.html`.
const IDENTIFIANTS_DU_METAL: [&str; 9] = [
    "commandes-metal",
    "metal-epsilon",
    "valeur-metal-epsilon",
    "metal-cote",
    "vitrine-metal",
    "legende-metal",
    "mesures-metal-bloc",
    "reglages-metal",
    "mesures-metal",
];

/// Les `ε` (en millionièmes) et les côtés que le juge a joués jusqu'au bout aux conditions de la page (V-P4-31 : `MODELE.md`, étape P4) : chacun s'arrête au mur, sans divergence.
const EPSILONS_JOUES: [u32; 13] =
    [0, 5000, 10_000, 15_000, 20_000, 25_000, 30_000, 35_000, 40_000, 45_000, 50_000, 55_000, 60_000];
const COTES_JOUES: [u32; 3] = [128, 192, 256];

/// Les `<option>` d'un `<select>` d'`id` donné : `(value, texte)`.
fn options_de(html: &str, id: &str) -> Vec<(String, String)> {
    let mut options = Vec::new();
    let mut dedans = false;
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>').unwrap_or((morceau, ""));
        if nom_de_balise(balise) == "select" && attribut(balise, "id").as_deref() == Some(id) {
            dedans = true;
        } else if balise.starts_with("/select") {
            dedans = false;
        } else if dedans && nom_de_balise(balise) == "option" {
            options.push((attribut(balise, "value").unwrap_or_default(), texte.trim().to_lowercase()));
        }
    }
    options
}

/// forme : `index.html` a le panneau du métal (V-P4-30). La **dernière** option du choix du modèle a la valeur `metal` et son texte dit « métal » ; les neuf identifiants du métal sont chacun **une seule fois** dans `index.html` et dans
/// aucun autre fichier HTML ; `metal-epsilon` est un `<input type="range">` dont `min`, `max`, `step` et `value` sont des entiers ; `valeur-metal-epsilon` est un `<output>` lié à `metal-epsilon` ; `metal-cote` est un `<select>` d'au moins
/// une option ; `vitrine-metal` est un `<canvas>`. Les autres éléments existent, de n'importe quelle sorte et à n'importe quelle place (un tiroir).
///
/// Ne couvre pas : que le panneau soit montré ou caché au bon moment (le DOM n'est pas éprouvé) ; la place des éléments.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn la_page_a_le_panneau_du_metal() {
    let html = e5::lire_page("index.html");
    let modeles = options_de(&html, "modele");
    let (valeur, texte) = modeles.last().expect("des options du choix du modèle");
    assert_eq!(valeur, "metal", "la dernière option du choix du modèle : {modeles:?}");
    assert!(texte.contains("métal"), "le texte de l'option du métal : « {texte} »");
    let tous: Vec<String> = balises(&html).iter().filter_map(|b| attribut(b, "id")).collect();
    for id in IDENTIFIANTS_DU_METAL {
        let fois = tous.iter().filter(|autre| autre.as_str() == id).count();
        assert_eq!(fois, 1, "l'identifiant « {id} » est écrit {fois} fois dans index.html (une seule, exactement)");
    }
    for (nom, autre) in e5::textes_des_fichiers(&["html", "htm"]) {
        if nom == "index.html" {
            continue;
        }
        for balise in balises(&autre) {
            let id = attribut(balise, "id").unwrap_or_default();
            assert!(!IDENTIFIANTS_DU_METAL.contains(&id.as_str()), "{nom} reprend l'identifiant « {id} »");
        }
    }
    for (id, sorte) in [
        ("metal-epsilon", "input"),
        ("valeur-metal-epsilon", "output"),
        ("metal-cote", "select"),
        ("vitrine-metal", "canvas"),
    ] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    let curseur = balise_d_id(&html, "metal-epsilon").expect("le curseur");
    assert_eq!(attribut(curseur, "type").as_deref(), Some("range"), "le curseur ε n'est pas de type range");
    for nom in ["min", "max", "step", "value"] {
        let valeur = attribut(curseur, nom).unwrap_or_else(|| panic!("le curseur ε n'a pas de {nom}"));
        assert!(valeur.parse::<u32>().is_ok(), "{nom} du curseur ε n'est pas un entier : {valeur}");
    }
    let sortie = balise_d_id(&html, "valeur-metal-epsilon").expect("la sortie");
    assert_eq!(attribut(sortie, "for").as_deref(), Some("metal-epsilon"), "la sortie du curseur ε lui est liée");
    assert!(!options_de(&html, "metal-cote").is_empty(), "le choix du côté n'a aucune option");
}

/// juste + forme : **la page n'offre que ce que le juge a joué** (V-P4-31). Le curseur `metal-epsilon` reste dans la plage du module (de 0 à 60 000 millionièmes) et **chaque valeur qu'il peut prendre** (`min`, `min + step`, …,
/// jusqu'à `max`) est un `ε` joué jusqu'au bout aux conditions de la page (0 à 60 000 par 5 000) ; sa `value` initiale est sur sa grille ; **chaque option** de `metal-cote` est un côté joué (128, 192 ou 256), dans la plage du module
/// (128 à 256). Un curseur trop long (0,07), trop fin (un pas de 1 000 : des `ε` non joués) ou un côté non joué se voient. La table des scénarios joués est écrite ici d'après la fiche.
///
/// Ne couvre pas : que le navigateur respecte la grille du curseur ; les scénarios eux-mêmes (mesurés par le juge, hors des tests : 39 scénarios, tous arrêtés au mur, sans divergence).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_reglages_offerts_ont_ete_joues() {
    let html = e5::lire_page("index.html");
    let curseur = balise_d_id(&html, "metal-epsilon").expect("le curseur ε");
    let lire = |nom: &str| -> u32 {
        attribut(curseur, nom).and_then(|v| v.parse().ok()).unwrap_or_else(|| panic!("le curseur ε : {nom}"))
    };
    let (min, max, pas, valeur) = (lire("min"), lire("max"), lire("step"), lire("value"));
    assert!(pas >= 1 && min < max && max <= EPSILON_MAX_PPM, "le curseur ε : [{min}, {max}] par {pas}");
    let mut e = min;
    while e <= max {
        assert!(
            EPSILONS_JOUES.contains(&e),
            "le curseur ε peut prendre {e} millionièmes : un ε que le juge n'a pas joué"
        );
        e += pas;
    }
    assert!(
        valeur >= min && valeur <= max && (valeur - min) % pas == 0,
        "la valeur initiale {valeur} hors de la grille"
    );
    let cotes = options_de(&html, "metal-cote");
    for (valeur, texte) in &cotes {
        let cote: u32 =
            valeur.parse().unwrap_or_else(|_| panic!("l'option « {texte} » n'a pas un côté entier : {valeur}"));
        assert!((COTE_MIN..=COTE_MAX).contains(&cote), "le côté {cote} est hors de la plage du module");
        assert!(COTES_JOUES.contains(&cote), "le côté {cote} n'a pas été joué par le juge");
    }
}

/// forme : **ce que la page dit du métal ne prétend pas plus que le modèle** (V-P4-32). `legende-metal` dit (en entier, sans tenir compte de la casse) : « notre modèle », « pas une reproduction », la vitesse de notre régime établi « 0,0408 », la vitesse publiée « 0,0469 » avec
/// « Kim » et « n'est pas atteinte », sa vitesse affichée « 0,0938 », « le mur est à nous », « avant le bord », « le régime n'est pas établi », « notre mesure » et « profondeur » ; elle ne dit ni « Lydite » ni un prix (« prix », « € », « $ »). **V8 : la légende peut être
/// écrite par un script** : si `legende-metal` est vide dans `index.html`, chaque phrase doit être une chaîne de l'un des fichiers JavaScript. **Tout ce que la page écrit** ne dit ni « ressemble », ni « conforme à », ni « identique à »,
/// ni « fidèle », ni « réaliste », ni « reproduit », ni « prouve que le métal », ni « reproduction exacte ».
///
/// Ne couvre pas : que la légende soit montrée avec le métal (le DOM n'est pas éprouvé) ; la justesse de chaque phrase (le juge l'a rapprochée de ses tests et mesures : `MODELE.md`, étape P4).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn la_page_dit_ce_que_le_metal_est_et_n_est_pas() {
    let html = e5::lire_page("index.html");
    assert!(balise_d_id(&html, "legende-metal").is_some(), "pas d'élément d'id « legende-metal »");
    let statique = texte_de(&html, "legende-metal").unwrap_or_default().to_lowercase();
    let chaines: Vec<String> = e5::chaines_du_code(&e5::tout_le_code_js()).iter().map(|c| c.to_lowercase()).collect();
    for phrase in [
        "notre modèle",
        "pas une reproduction",
        "0,0408",
        "0,0469",
        "0,0938",
        "kim",
        "n'est pas atteinte",
        "le mur est à nous",
        "avant le bord",
        "le régime n'est pas établi",
        "notre mesure",
        "profondeur",
    ] {
        let dite =
            if statique.is_empty() { chaines.iter().any(|c| c.contains(phrase)) } else { statique.contains(phrase) };
        assert!(dite, "la légende du métal ne dit pas « {phrase} » : « {statique} »");
    }
    let legende = if statique.is_empty() {
        chaines.iter().filter(|c| c.contains("notre modèle")).cloned().collect::<Vec<_>>().join("\n")
    } else {
        statique
    };
    for interdit in ["lydite", "prix", "€", "$"] {
        assert!(!legende.contains(interdit), "la légende du métal dit « {interdit} »");
    }
    let ecrit = e5::textes_ecrits_par_la_page().to_lowercase();
    for interdit in [
        "ressemble",
        "conforme à",
        "identique à",
        "fidèle",
        "réaliste",
        "reproduit",
        "prouve que le métal",
        "reproduction exacte",
    ] {
        assert!(!ecrit.contains(interdit), "la page dit « {interdit} »");
    }
}

/// forme : **ce que la page lit du métal passe par `lecture.js`, que Node éprouve** (V-P4-33). `lecture.js` déclare `export function …(` pour ses dix fonctions du métal et n'emploie ni `document` ni `window` ; chaque appel de
/// l'une d'elles, dans tous les fichiers de la page, a le bon nombre d'arguments (deux pour `imageDuMetal` et `formaterMetal`, un sinon) ; **un fichier hors de `lecture.js`** appelle `lireChampsMetal(`, `imageDuMetal(` (le dessin),
/// `lireReglagesMetal(` et `lireMesuresMetal(` (ce que la page écrit vient du module), `texteRefusMetal(` (**un refus du module est affiché**) et `blocsMetal(` ; **aucun fichier hors de `lecture.js`** n'appelle `givre_metal_reglage`,
/// `givre_metal_mesure` ni `givre_metal_case`, que `lecture.js` appelle ; `lecture.js` n'appelle pas `givre_metal_demarrer` (seule la page démarre un modèle) ; et **chaque appel de `givre_metal_demarrer(`** a deux arguments dont aucun ne
/// commence par un chiffre (l'anisotropie et le côté viennent des commandes, jamais d'un nombre écrit dans l'appel).
///
/// Ne couvre pas : ce que ces fonctions rendent (voir `lecture_js_lit_le_metal_sur_un_faux_module` et `etape_p4_module.rs`) ; que l'état de la page suive vraiment le modèle démarré (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn lecture_js_declare_ce_que_la_page_lit_du_metal() {
    let lecture = code_de_lecture_js();
    let tout = code_de_app_js();
    let hors_lecture = e5::code_js_hors_lecture();
    let attendues: [(&str, usize); 10] = [
        ("lireReglagesMetal", 1),
        ("lireMesuresMetal", 1),
        ("lireChampsMetal", 1),
        ("imageDuMetal", 2),
        ("formaterMetal", 2),
        ("texteReglagesMetal", 1),
        ("texteMesuresMetal", 1),
        ("texteFinMetal", 1),
        ("texteRefusMetal", 1),
        ("blocsMetal", 1),
    ];
    for (nom, arite) in attendues {
        assert!(
            lecture.contains(&format!("export function {nom}(")),
            "lecture.js ne déclare pas `export function {nom}(`"
        );
        let arites = arites_des_appels(&tout, nom);
        assert!(
            arites.iter().all(|a| *a == arite),
            "la page appelle {nom} avec {arites:?} argument(s) au lieu de {arite}"
        );
    }
    for nom in
        ["lireChampsMetal", "imageDuMetal", "lireReglagesMetal", "lireMesuresMetal", "texteRefusMetal", "blocsMetal"]
    {
        assert!(
            !arites_des_appels(&hors_lecture, nom).is_empty(),
            "aucun fichier de la page, hors lecture.js, n'appelle {nom}(…)"
        );
    }
    let mots: BTreeSet<&str> = lecture.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).collect();
    for interdit in ["document", "window"] {
        assert!(!mots.contains(interdit), "lecture.js emploie `{interdit}`");
    }
    for lecteur in ["givre_metal_reglage", "givre_metal_mesure", "givre_metal_case"] {
        assert!(
            arites_des_appels(&hors_lecture, lecteur).is_empty(),
            "un fichier de la page, hors lecture.js, appelle {lecteur}(…) : la lecture du module passe par lecture.js"
        );
        assert!(!arites_des_appels(&lecture, lecteur).is_empty(), "lecture.js n'appelle jamais {lecteur}(…)");
    }
    assert!(arites_des_appels(&lecture, "givre_metal_demarrer").is_empty(), "lecture.js démarre le métal");
    let mut appels = 0;
    for (position, _) in tout.match_indices("givre_metal_demarrer(") {
        if tout[..position].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        appels += 1;
        let reste = &tout[position + "givre_metal_demarrer(".len()..];
        let (mut profondeur, mut arguments, mut courant) = (0_i32, Vec::<String>::new(), String::new());
        for c in reste.chars() {
            match c {
                '(' | '[' | '{' => profondeur += 1,
                ')' | ']' | '}' if profondeur == 0 => break,
                ')' | ']' | '}' => profondeur -= 1,
                ',' if profondeur == 0 => {
                    arguments.push(std::mem::take(&mut courant).trim().to_string());
                    continue;
                }
                _ => {}
            }
            courant.push(c);
        }
        arguments.push(courant.trim().to_string());
        arguments.retain(|a| !a.is_empty());
        assert_eq!(arguments.len(), 2, "givre_metal_demarrer est appelée avec {arguments:?}");
        for argument in &arguments {
            assert!(
                !argument.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '.'),
                "un réglage écrit dans l'appel de givre_metal_demarrer : « {argument} »"
            );
        }
    }
    assert!(appels >= 1, "la page n'appelle jamais givre_metal_demarrer");
}

/// forme : les bornes de la boucle et de l'image du métal (V-P4-34). **Si** une constante `PAS_PAR_APPEL_METAL` est déclarée dans le code de la page (`const PAS_PAR_APPEL_METAL = <entier>`), elle vaut de 1 à 20 (au plus 36 ms
/// de calcul par appel au côté 256 sous Node) ; **si** `METAL_IMAGE_ANIMATION` ou `METAL_IMAGE_FIN` sont déclarées, elles valent de 2 à 1024 ; **chaque appel** de `imageDuMetal(` dont la taille est un entier écrit la respecte.
///
/// Ne couvre pas : le temps réel d'une image dans un navigateur (mesuré sous Node par le juge : `MODELE.md`, étape P4) ; une taille calculée.
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn les_bornes_de_la_boucle_et_de_l_image_du_metal() {
    let tout = code_de_app_js();
    if let Some(lot) = constante_entiere(&tout, "PAS_PAR_APPEL_METAL") {
        assert!((1..=20).contains(&lot), "PAS_PAR_APPEL_METAL vaut {lot}");
    }
    for nom in ["METAL_IMAGE_ANIMATION", "METAL_IMAGE_FIN"] {
        if let Some(taille) = constante_entiere(&tout, nom) {
            assert!((2..=1024).contains(&taille), "{nom} vaut {taille}");
        }
    }
    for (position, _) in tout.match_indices("imageDuMetal(") {
        let reste = &tout[position + "imageDuMetal(".len()..];
        let fin = reste.find(')').unwrap_or(reste.len());
        if let Some((_, taille)) = reste[..fin].rsplit_once(',')
            && let Ok(taille) = taille.trim().parse::<u64>()
        {
            assert!((2..=1024).contains(&taille), "imageDuMetal(…, {taille})");
        }
    }
}

/// juste : les fonctions de `lecture.js` du métal, sur un **faux module qui change d'état**, disent ce que la doc dit (V-P4-35). Le faux module (`tests/etape_p4/metal-pur.mjs`) a des réglages, des mesures et des champs donnés par des
/// formules (voir son en-tête) et se souvient du démarrage et du temps joué. `lireReglagesMetal`, `lireMesuresMetal` et `lireChampsMetal` rendent `null` sans métal (avant tout démarrage, après des refus, après que le métal est
/// effacé) ; avec un métal, les quinze réglages et les onze mesures **dans l'ordre du module** (un champ lu à la place d'un autre se voit), `auMur` et `diverge` en booléens, la vitesse `NaN` avant trois pas ; les champs de `cote²` valeurs
/// dans deux `Float64Array`, qui **changent avec le temps** ; **l'image** de `imageDuMetal` est, octet pour octet, celle de la doc réécrite en Rust (`etape_p4_commun::image_du_metal`) pour 5 × 5 et 4 × 4 pixels, et vaut **à la main** pour
/// un champ d'une seule case : le solide `(236, 242, 248)`, le liquide froid `(11, 17, 24)`, le liquide chaud `(255, 178, 122)`, et deux mélanges `(142, 154, 178)` et `(206, 210, 222)` ; les textes des réglages, des mesures, des fins
/// (au mur, à la durée, divergé), des refus, `blocsMetal` et `formaterMetal` sont ceux écrits ici.
///
/// Ne couvre pas : le vrai module (voir `etape_p4_module.rs`) ; ce que la page fait de ces textes et de cette image (les écrire dans le DOM, la poser sur le canvas).
#[test]
#[cfg_attr(not(feature = "etape-p4"), ignore)]
fn lecture_js_lit_le_metal_sur_un_faux_module() {
    if !e5::node_est_la() {
        return;
    }
    let lecture = chemin_de_lecture();
    let sortie = lancer_le_script("metal-pur.mjs", &[lecture.as_str()]);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "metal-pur.mjs : {:?}, sortie d'erreur : {erreur}", sortie.status);

    // Le faux module : réglage k = 10 k + 0,5 (sauf 0 : 0,05 ; 1 : 0,65 ; 10 : le côté 3 ; 11 : 2,5) ; mesure k = 100 k + t + 0,25 (sauf 2, 3, 9, 10).
    let reglages = "0.05,0.65,20.5,30.5,40.5,50.5,60.5,70.5,80.5,90.5,3,2.5,120.5,130.5,140.5";
    let mesures = |t: u32| {
        let m = |k: u32| format!("{}", f64::from(100 * k + t) + 0.25);
        let (v, s) = if t < 3 { ("NaN", "NaN") } else { ("0.125", "0.5") };
        format!("{},{},false,false,{},{},{},{},{},{v},{s}", m(0), m(1), m(4), m(5), m(6), m(7), m(8))
    };
    let champs_de = |t: u32| -> (Vec<f64>, Vec<f64>) {
        let phi = (0..9).map(|i: u32| f64::from((37 * i + t) % 17) / 8.0 - 1.0).collect();
        let u = (0..9).map(|i: u32| f64::from((11 * i + t) % 13) / 20.0 - 0.65).collect();
        (phi, u)
    };
    let champs = |t: u32| {
        let (phi, u) = champs_de(t);
        let ecrire = |v: &[f64]| v.iter().map(|x| format!("{x}")).collect::<Vec<_>>().join(",");
        format!("3;0.65;{};{};Float64Array.Float64Array", ecrire(&phi), ecrire(&u))
    };
    let hexa = |octets: &[u8]| octets.iter().map(|o| format!("{o:02x}")).collect::<String>();
    let image = |t: u32, taille: usize| {
        let (phi, u) = champs_de(t);
        hexa(&image_du_metal(3, &phi, &u, 0.65, taille))
    };
    let attendu = [
        "avant=null,null,null".to_string(),
        "refus=1,2,null,null,null".to_string(),
        format!("reglages={reglages}"),
        format!("mesures={}", mesures(0)),
        format!("champs={}", champs(0)),
        format!("image={},5x5,Uint8ClampedArray", image(0, 5)),
        format!("apres={}|{}|{}", mesures(3), champs(3), image(3, 4)),
        "uniformes=236.242.248.255,11.17.24.255,255.178.122.255,142.154.178.255,206.210.222.255".to_string(),
        "textes=Démarré avec ε 0,050 · surfusion Δ 0,65 · longueur capillaire d0 20,5 · diffusivité 31 · graine de rayon 91 · pas de grille 70,5 · pas de temps 80,5000 · quart de 3 cases ; arrêt quand la pointe atteint 2,5 (le bord est à 211,5).\
|Pointe à 3,25 au temps 103,3 · au dernier relevé (temps 403,3) : vitesse 0,1250 · rayon 803,25 (parabole sur 603 lignes) · σ* 0,500"
            .to_string(),
        "fins=Arrêté avant le bord : la pointe a atteint la limite du mur (le mur est à nous ; plus près, le bord la freinerait). Recommencez ou choisissez un autre modèle.\
|Scénario terminé (tous les pas sont joués) : recommencez ou choisissez un autre modèle.\
|Arrêté : un pas a divergé (le module garde l'état d'avant). Recommencez ou choisissez un autre modèle."
            .to_string(),
        "au_mur=true".to_string(),
        "refus_texte=Le métal ne démarre pas : ε hors plage (code 1).|Le métal ne démarre pas : côté hors plage (code 2).\
|Le métal ne démarre pas : graine refusée (code 3).|Le métal ne démarre pas : refus du module (code 9)."
            .to_string(),
        "blocs=false,false,false|true,true,true".to_string(),
        "formats=–;0,0813;112,0;-0,500;6".to_string(),
        "oublie=null,null,null".to_string(),
    ]
    .join("\n")
        + "\n";
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu);

    for arguments in [&[][..], &["a", "b"], &["n-existe-pas.js"]] {
        let sortie = lancer_le_script("metal-pur.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "metal-pur.mjs {arguments:?}");
        assert!(sortie.stdout.is_empty(), "metal-pur.mjs {arguments:?} : la sortie standard n'est pas vide");
    }
}

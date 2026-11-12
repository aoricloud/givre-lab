//! Tests d'acceptation de l'étape 5 (c) : la page `docs/index.html` + `docs/app.js` + `docs/lecture.js`. Presque tous sont des
//! **tests de forme** : ils lisent les fichiers comme du texte (et Node vérifie que `app.js` et `lecture.js` sont du
//! JavaScript valide). Un seul fait tourner du code de la page : `lecture_js_lit_le_module_comme_le_natif` fait lire le
//! module commité par `lecture.js` (sous Node, sans navigateur) et compare l'empreinte, l'image et la graine qu'il lit au
//! code natif. Le reste de la page (le DOM, le canvas, l'animation) ne tourne que dans un navigateur : il n'est éprouvé que
//! par sa forme et par un coup d'œil humain ; voir « Ce que cela n'établit pas » dans `MODELE.md`.
//!
//! Ce test-là lit `docs/givre.wasm` : il ne passe qu'une fois le module construit et commité.
//!
//! **Étape V8 : la page est tous les fichiers de son dossier.** Ces tests ne lisent plus trois fichiers à la virgule près : ils lisent **tout** ce que le dossier de la page contient (`docs/`, ou le dossier que désigne la variable d'environnement `GIVRE_DOCS`, que
//! seuls les tests de `tests/etape_v8/` posent) : `index.html` et ses scripts en ligne, les feuilles de style, **tous** les fichiers JavaScript (`app.js`, `lecture.js`, un ou plusieurs workers), les polices et les images. La page peut donc être réécrite (un tiroir de réglages,
//! des polices en `data:`, des liens `https://`, un SVG, des workers) tant qu'elle tient ses promesses : les identifiants, les phrases d'honnêteté, `lecture.js` pur et éprouvé sous Node, le canvas de la neige non pixélisé, les exportations du module, aucune ressource chargée du
//! dehors, un poids borné. Ce que ces tests ne prouvent plus (parce que seul le texte d'un fichier précis le montrait) est dit dans `MODELE.md`, étape V8, « Ce que les tests de la page figent, et ce qu'ils ne figent plus ».
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : celle de l'étape 5 ; les étapes suivantes la complètent)
//!
//! Une page web **sans bibliothèque ni adresse extérieure** (aucun `http://`, `https://`, aucune police ni script distant), qui
//! charge `givre.wasm` (relatif à la page, qui est servie depuis `docs/`) par `WebAssembly.instantiate`, avec des importations
//! vides, et dessine sur un `<canvas>` :
//!
//! - `index.html` : `<meta name="viewport" content="width=device-width, initial-scale=1">` (lisible sur un téléphone : la
//!   largeur du canvas est fluide, `max-width: 100%`), un `<title>`, un `<script type="module" src="app.js">`, et ces éléments,
//!   chacun avec **cet `id`** : le `<canvas id="vitrine">` ; le `<select id="modele">` de quatre `<option>` dont les `value`
//!   sont `0`, `1`, `2`, `3` et dont les textes disent DLA, plaque, étoile, dendrite (dans cet ordre) ; un `<input id="graine">`
//!   (la graine, utile à la DLA seulement) ; un `<button id="relancer">` ; un `<input id="vitesse" type="range">` (le
//!   curseur de vitesse : unités de temps jouées par image) ; un élément `id="temps"` (le temps affiché) et un élément
//!   `id="empreinte"` (l'empreinte SHA-256 affichée) ;
//! - `lecture.js` : un module ES **sans DOM** (il n'emploie ni `document`, ni `window`, ni le canvas : Node le charge seul), qui
//!   exporte exactement ces trois fonctions, écrites `export function <nom>(…)`, où `m` est `instance.exports` du module :
//!   - `lireEmpreinte(m)` rend une chaîne de 64 chiffres hexadécimaux minuscules : pour `i` de 0 à 7 dans l'ordre,
//!     `m.givre_empreinte_mot(i) >>> 0` écrit en hexadécimal sur 8 chiffres (zéros à gauche), mis bout à bout ; c'est
//!     l'empreinte que la page affiche ;
//!   - `lireImage(m)` rend `{ largeur, hauteur, rgba }` : `largeur` est `m.givre_largeur() >>> 0`, `hauteur`
//!     `m.givre_hauteur() >>> 0`, et `rgba` un **`Uint8ClampedArray`** de `4 × largeur × hauteur` octets où le pixel `(x, y)`
//!     (`x` vers la droite, `y` vers le bas) occupe les quatre octets à partir de `4 × (y × largeur + x)` : R, V, B lus dans
//!     `m.givre_pixel(x, y) >>> 0` (`0x00RRGGBB` : R les bits 16 à 23, V 8 à 15, B 0 à 7), puis 255 (opaque). C'est le
//!     tableau qu'attend `new ImageData(rgba, largeur, hauteur)` : la page le dessine tel quel ;
//!   - `lireGraine(texte)` lit la graine écrite dans le champ `graine` : le texte sans les espaces de début et de fin
//!     (`trim()`) ; s'il n'est fait que de chiffres décimaux ASCII (au moins un : zéros de tête permis, ni signe, ni point, ni
//!     exposant, ni `0x`) et vaut au plus 4294967295, rend ce nombre ; sinon `null`.
//! - `app.js` : un fichier de modules ES (`node --check`), qui importe ces trois fonctions de `"./lecture.js"` (un `import { … }
//!   from "./lecture.js"`) et les appelle, chacune avec un argument ; qui demande chacun des éléments de la page par
//!   `document.getElementById("…")` (un texte littéral) ; anime par `requestAnimationFrame` et dessine par le contexte 2D
//!   (`getContext("2d")`). À eux deux, `app.js` et `lecture.js` appellent **chacune des onze exportations** du module par son
//!   nom entier (`….givre_pixel(x, y)`) avec le bon nombre d'arguments. La page dessine l'image de `lireImage` telle quelle
//!   (les couleurs de l'aperçu de la bibliothèque) ; le bouton recommence le modèle choisi (`givre_demarrer(modele, graine)`,
//!   la graine lue par `lireGraine`, une graine `null` n'étant pas démarrée mais dite à l'écran ; la graine ne sert qu'à la
//!   DLA) ; le curseur règle le nombre d'unités passées à `givre_avancer` à chaque image ; la page affiche le temps
//!   (`givre_temps` sur `givre_duree`) et l'empreinte de `lireEmpreinte`, **lue au plus toutes les 400 ms** (huit appels de
//!   `givre_empreinte_mot` par lecture, chacun hache tout l'état) : une constante `INTERVALLE_EMPREINTE_MS` d'au moins 400,
//!   employée, dans `app.js`. Les mots de 32 bits du module sont lus signés par JavaScript : `>>> 0` les relit non signés.
//!   Quand `givre_avancer` rend 0 (le scénario est fini), la page cesse d'avancer. Un module qui ne charge pas se dit à
//!   l'écran (un message), sans exception non rattrapée.
//!
//! Ce que les tests de forme lisent, et rien d'autre : le texte d'`app.js` et de `lecture.js` **sans leurs commentaires** (`//`
//! et `/* */`, hors des guillemets ; un nom qui ne figure que dans un commentaire n'est pas employé ; pas de littéral
//! d'expression régulière contenant un guillemet ou `//`) et `index.html` tel quel.
//!
//! Étiquettes : `lecture_js_lit_le_module_comme_le_natif` est **juste** (comparé au code natif et à un calcul exact) ; tous les
//! autres sont de la **forme** (régression de texte), jamais une preuve du comportement de la page.

mod etape_5_commun;

use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use etape_5_commun::{
    CODE_MAX, D3_ACTIVE, FONCTIONS, FONCTIONS_D3, FONCTIONS_G3, FONCTIONS_V7, FONCTIONS_V8, FONCTIONS_Z1, G3_ACTIVE,
    IDENTIFIANTS_DE_LA_PAGE, IMAGE_MAX, IMAGES_MAX, V7_ACTIVE, Z1_ACTIVE, adresse_externe, chaines_du_code,
    chemin_de_lecture, chemin_module, code_js_de_la_page, est_une_image, fichiers_de_la_page, lancer_le_script,
    lire_page, natif, node_est_la, racine, ressources_externes, sans_commentaires, scripts_en_ligne,
    textes_des_fichiers, tout_le_code_js,
};
use givre_lab::empreinte::sha256_hex;

/// Les éléments que la page de l'étape 5 doit avoir, chacun par son `id` (la liste de toute la page est `IDENTIFIANTS_DE_LA_PAGE`).
const IDENTIFIANTS: [&str; 7] = ["vitrine", "modele", "graine", "relancer", "vitesse", "temps", "empreinte"];

/// Tout le code JavaScript de la page (chaque fichier `.js` ou `.mjs` du dossier et chaque script en ligne), sans ses commentaires : voir `etape_5_commun::tout_le_code_js`.
fn lire_le_code_de_la_page() -> String {
    tout_le_code_js()
}

/// Le code de `lecture.js` (le fichier que Node charge seul), sans ses commentaires.
fn lire_le_code_de_lecture_js() -> String {
    sans_commentaires(&lire_page("lecture.js"))
}

/// Les fonctions que `lecture.js` exporte et qu'`app.js` appelle, chacune avec un argument.
const FONCTIONS_DE_LECTURE: [&str; 3] = ["lireEmpreinte", "lireImage", "lireGraine"];

/// Les noms `givre_…` que le texte emploie : chaque suite de lettres, de chiffres et de `_` qui commence par `givre_`.
fn noms_employes(texte: &str) -> BTreeSet<String> {
    let mut noms = BTreeSet::new();
    let mut reste = texte;
    while let Some(debut) = reste.find("givre_") {
        let a_partir_du_nom = &reste[debut..];
        let fin =
            a_partir_du_nom.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(a_partir_du_nom.len());
        noms.insert(a_partir_du_nom[..fin].to_string());
        reste = &a_partir_du_nom[fin..];
    }
    noms
}

/// Pour chaque appel `nom(…)` du texte, le nombre d'arguments : les virgules de premier niveau plus un (une virgule finale
/// ne compte pas), ou 0 si les parenthèses sont vides. Un `nom` précédé d'une lettre, d'un chiffre ou d'un `_` est un autre
/// nom (`mon_givre_pixel(`) et n'est pas compté.
fn arites_des_appels(code: &str, nom: &str) -> Vec<usize> {
    let motif = format!("{nom}(");
    let mut arites = Vec::new();
    let mut depuis = 0;
    while let Some(position) = code[depuis..].find(&motif) {
        let debut = depuis + position;
        depuis = debut + motif.len();
        if code[..debut].chars().next_back().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let (mut profondeur, mut virgules, mut vide, mut derniere_virgule) = (0_usize, 0_usize, true, false);
        for c in code[depuis..].chars() {
            match c {
                '(' | '[' | '{' => {
                    profondeur += 1;
                    vide = false;
                    derniere_virgule = false;
                }
                ')' | ']' | '}' => {
                    if profondeur == 0 {
                        break;
                    }
                    profondeur -= 1;
                    derniere_virgule = false;
                }
                ',' if profondeur == 0 => {
                    virgules += 1;
                    derniere_virgule = true;
                }
                c if c.is_whitespace() => {}
                _ => {
                    vide = false;
                    derniere_virgule = false;
                }
            }
        }
        arites.push(if vide { 0 } else { virgules + 1 - usize::from(derniere_virgule) });
    }
    arites
}

/// Les balises du HTML, chacune réduite à son contenu entre `<` et `>` (le nom, puis les attributs).
fn balises(html: &str) -> Vec<&str> {
    html.split('<').skip(1).filter_map(|morceau| morceau.split('>').next()).collect()
}

/// La valeur de l'attribut `nom` d'une balise (écrit entre guillemets ou entre apostrophes, précédé d'une espace).
fn attribut(balise: &str, nom: &str) -> Option<String> {
    etape_5_commun::attribut(balise, nom)
}

/// Le nom d'une balise, en minuscules (`canvas` pour `<canvas id="x">`).
fn nom_de_balise(balise: &str) -> String {
    balise.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// La première balise du HTML qui a cet `id`.
fn balise_d_id<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    balises(html).into_iter().find(|balise| attribut(balise, "id").as_deref() == Some(id))
}

/// forme : `docs/index.html` contient un `<canvas` d'`id` `vitrine`, une balise `<meta name="viewport"` dont le contenu dit `width=device-width` (lisible sur un téléphone), un `<title>`, et **au moins un `<script type="module"`** : en ligne (non vide), ou avec un `src` **relatif** qui désigne un fichier du
/// dossier de la page (`app.js`, ou un autre : un script dont le fichier manque est une page cassée).
///
/// Ne couvre pas : ce que la page montre ; que le script se charge ; la lisibilité réelle sur un téléphone.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_a_un_canvas_un_viewport_et_charge_un_script() {
    let html = lire_page("index.html");
    let minuscules = html.to_lowercase();
    let canvas = balise_d_id(&html, "vitrine").expect("pas d'élément d'id « vitrine »");
    assert_eq!(nom_de_balise(canvas), "canvas", "l'élément d'id « vitrine » n'est pas un <canvas>");
    assert!(minuscules.contains("<title>"), "pas de <title>");

    let viewport = balises(&html)
        .into_iter()
        .find(|balise| nom_de_balise(balise) == "meta" && attribut(balise, "name").as_deref() == Some("viewport"))
        .expect("pas de <meta name=\"viewport\">");
    let contenu = attribut(viewport, "content").unwrap_or_default().replace(' ', "");
    assert!(contenu.contains("width=device-width"), "le viewport ne dit pas width=device-width : {viewport}");

    let modules: Vec<(String, String)> = scripts_en_ligne(&html)
        .into_iter()
        .filter(|(ouvrante, _)| attribut(ouvrante, "type").as_deref() == Some("module"))
        .collect();
    assert!(!modules.is_empty(), "pas de <script type=\"module\"> dans index.html");
    for (ouvrante, corps) in &modules {
        match attribut(ouvrante, "src") {
            Some(src) => {
                assert!(!adresse_externe(&src), "le script de type module charge une adresse externe : {src}");
                let fichier = src.split(['?', '#']).next().unwrap_or("").trim_start_matches("./");
                assert!(
                    etape_5_commun::dossier_des_pages().join(fichier).is_file(),
                    "le script de type module « {src} » n'est pas un fichier du dossier de la page"
                );
            }
            None => assert!(!corps.trim().is_empty(), "un <script type=\"module\"> en ligne est vide"),
        }
    }
}

/// forme : `index.html` a les commandes et les affichages demandés, chacun avec son `id` et sa sorte d'élément : un
/// `<select id="modele">` de quatre `<option>` de `value` `0`, `1`, `2`, `3` (dans cet ordre) dont les textes disent DLA,
/// plaque, étoile et dendrite ; un `<input id="graine">` ; un `<button id="relancer">` ; un `<input id="vitesse"
/// type="range">` ; des éléments `id="temps"` et `id="empreinte"`.
///
/// Ne couvre pas : que ces éléments soient visibles, lisibles ou reliés à quoi que ce soit (voir `app_js_demande_…`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_a_ses_commandes_et_ses_affichages() {
    let html = lire_page("index.html");
    for (id, sorte) in [("modele", "select"), ("graine", "input"), ("relancer", "button"), ("vitesse", "input")] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    let vitesse = balise_d_id(&html, "vitesse").expect("vitesse");
    assert_eq!(attribut(vitesse, "type").as_deref(), Some("range"), "le curseur de vitesse n'est pas de type range");
    for id in ["temps", "empreinte"] {
        assert!(balise_d_id(&html, id).is_some(), "pas d'élément d'id « {id} »");
    }

    // Les options du <select id="modele"> : chaque morceau coupé sur `<` est « balise>texte ».
    let mut options: Vec<(String, String)> = Vec::new();
    let mut dans_le_select = false;
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>').unwrap_or((morceau, ""));
        let nom = nom_de_balise(balise);
        if nom == "select" && attribut(balise, "id").as_deref() == Some("modele") {
            dans_le_select = true;
        } else if nom == "/select" {
            dans_le_select = false;
        } else if dans_le_select && nom == "option" {
            options.push((attribut(balise, "value").unwrap_or_default(), texte.trim().to_lowercase()));
        }
    }
    let valeurs: Vec<&str> = options.iter().map(|(valeur, _)| valeur.as_str()).collect();
    // Étape D3 : une cinquième option, le claquage diélectrique (`value` « claquage » : le curseur η donne le numéro du modèle). Avec `etape-d3`, les
    // cinq seulement ; sans elle, les quatre d'avant ou les cinq.
    // Étape G3 : une sixième option, la neige (`value` « neige » : le menu des jeux nommés et les curseurs donnent son démarrage). Avec `etape-g3`, les six seulement ; sans elle, ce que `etape-d3` dit ou les six.
    if G3_ACTIVE {
        assert_eq!(valeurs, ["0", "1", "2", "3", "claquage", "neige"], "les <option> du choix du modèle");
    } else if D3_ACTIVE {
        assert!(
            valeurs == ["0", "1", "2", "3", "claquage"] || valeurs == ["0", "1", "2", "3", "claquage", "neige"],
            "les <option> du choix du modèle : {valeurs:?}"
        );
    } else {
        assert!(
            valeurs == ["0", "1", "2", "3"]
                || valeurs == ["0", "1", "2", "3", "claquage"]
                || valeurs == ["0", "1", "2", "3", "claquage", "neige"],
            "les <option> du choix du modèle : {valeurs:?}"
        );
    }
    for ((_, texte), mot) in options.iter().zip(["dla", "plaque", "toile", "dendrite", "claquage"]) {
        assert!(texte.contains(mot), "le texte de l'option « {texte} » ne dit pas « {mot} »");
    }
}

/// forme : le code de la page (un fichier `.js` ou un script en ligne : `app.js`, un worker, n'importe lequel) désigne le module par un **texte littéral qui finit par `givre.wasm`**, relatif à la page (ni `http`, ni `//`, ni `/` en tête : la page est servie depuis `docs/`, jamais `/docs/givre.wasm`), et
/// l'instancie par `WebAssembly.instantiate` (`instantiateStreaming` compris) ou `new WebAssembly.Instance`.
///
/// Ne couvre pas : que le chargement réussisse dans un navigateur ; quel fichier charge le module (un worker le charge souvent à la place de la page).
/// Fiche : V-V8-39 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_charge_le_module() {
    let code = lire_le_code_de_la_page();
    let relatif = chaines_du_code(&code)
        .into_iter()
        .any(|chaine| chaine.ends_with("givre.wasm") && !chaine.starts_with('/') && !chaine.contains("://"));
    assert!(relatif, "le code de la page ne désigne pas le module par un texte relatif qui finit par givre.wasm");
    assert!(
        code.contains("WebAssembly.instantiate") || code.contains("new WebAssembly.Instance"),
        "le code de la page n'instancie pas le module"
    );
}

/// forme : le code de la page (tous les fichiers) anime par `requestAnimationFrame` et dessine sur un canvas par un contexte (`getContext(`, de n'importe quelle sorte : `"2d"`, `"bitmaprenderer"`…).
///
/// Ne couvre pas : ce qui est dessiné ; que ce soit le fil de la page ou un worker qui dessine.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_anime_et_dessine_sur_un_canvas() {
    let code = lire_le_code_de_la_page();
    assert!(code.contains("requestAnimationFrame"), "le code de la page n'emploie pas requestAnimationFrame");
    assert!(code.contains("getContext("), "le code de la page n'emploie aucun contexte de canvas (getContext)");
}

/// forme : **tous les fichiers JavaScript de la page** (`app.js`, `lecture.js`, un ou plusieurs workers, les scripts en ligne de `index.html` : leurs codes mis bout à bout) n'emploient **que** des exportations du module (chaque nom `givre_…` qu'ils emploient est l'une des trente-quatre fonctions de la
/// vitrine : aucune inconnue) et **toutes** celles d'avant l'étape V8 (les trente-deux) ; **les deux de V8 (`givre_neige_masses_preparer`, `givre_neige_masse`) peuvent être employées ou non**, dans les deux régimes ; chaque fonction employée est **appelée** (`nom(…)`) et **tous** ses appels ont le bon
/// nombre d'arguments (`givre_pixel(x)` sans `y`, ou `givre_demarrer(modele)` sans graine, sont vus). Les noms s'écrivent en entier dans le script, jamais composés. (Ce test relit la liste des fonctions dans `etape_5_commun::FONCTIONS` et ses suites, que `etape_5_artefact.rs` compare aux exportations du module.)
///
/// Ne couvre pas : que les arguments soient les bons (un `x` pour un `y` : ceux de `lecture.js` sont éprouvés par `lecture_js_lit_le_module_comme_le_natif`) ; que l'appel soit atteint ; quel fichier emploie quoi (un worker peut tout employer).
/// Fiche : V-V8-33 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_emploie_les_exportations_du_module_et_rien_d_autre() {
    let code = lire_le_code_de_la_page();
    let onze: BTreeSet<String> = FONCTIONS.iter().map(|(nom, _, _)| nom.to_string()).collect();
    // Étape Z1 : avec `etape-z1` la page emploie les quinze fonctions ; sans elle, les onze ou les quinze.
    let quinze: BTreeSet<String> =
        onze.iter().cloned().chain(FONCTIONS_Z1.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape D3 : avec `etape-d3` la page emploie les vingt-trois fonctions ; sans elle, les onze, les quinze ou les vingt-trois.
    let vingt_trois: BTreeSet<String> =
        quinze.iter().cloned().chain(FONCTIONS_D3.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape V7 : avec `etape-v7` la page emploie les vingt-sept fonctions ; sans elle, ce que les fonctionnalités d'avant disent ou les vingt-sept.
    let vingt_sept: BTreeSet<String> =
        vingt_trois.iter().cloned().chain(FONCTIONS_V7.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape G3 : avec `etape-g3` la page emploie les trente-deux fonctions ; sans elle, ce que les fonctionnalités d'avant disent ou les trente-deux.
    let trente_deux: BTreeSet<String> =
        vingt_sept.iter().cloned().chain(FONCTIONS_G3.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape V8 : les deux fonctions des masses sont **permises** (la page de la session de site les emploie dans un worker) mais jamais exigées : ni l'une ni l'autre régime ne les demande.
    let masses: BTreeSet<String> = FONCTIONS_V8.iter().map(|(nom, _, _)| nom.to_string()).collect();
    let permises: BTreeSet<String> = trente_deux.union(&masses).cloned().collect();
    let employes = noms_employes(&code);
    let employes_d_avant: BTreeSet<String> = employes.difference(&masses).cloned().collect();
    let exportees = if G3_ACTIVE || employes_d_avant == trente_deux {
        trente_deux
    } else if V7_ACTIVE || employes_d_avant == vingt_sept {
        vingt_sept
    } else if D3_ACTIVE || employes_d_avant == vingt_trois {
        vingt_trois
    } else if Z1_ACTIVE || employes_d_avant == quinze {
        quinze
    } else {
        onze
    };

    let inconnus: Vec<&String> = employes.difference(&permises).collect();
    assert!(inconnus.is_empty(), "la page emploie des noms que le module n'exporte pas : {inconnus:?}");
    let oublies: Vec<&String> = exportees.difference(&employes).collect();
    assert!(oublies.is_empty(), "la page n'emploie pas ces exportations du module : {oublies:?}");

    for (nom, parametres, _) in FONCTIONS
        .iter()
        .chain(FONCTIONS_Z1.iter())
        .chain(FONCTIONS_D3.iter())
        .chain(FONCTIONS_V7.iter())
        .chain(FONCTIONS_G3.iter())
        .chain(FONCTIONS_V8.iter())
        .filter(|(nom, _, _)| employes.contains(*nom))
    {
        let arites = arites_des_appels(&code, nom);
        assert!(!arites.is_empty(), "la page n'appelle jamais {nom}(…)");
        for arite in arites {
            assert_eq!(
                arite,
                parametres.len(),
                "la page appelle {nom} avec {arite} argument(s) au lieu de {}",
                parametres.len()
            );
        }
    }
}

/// forme : **les identifiants** (V8). Les quarante-huit `id` que les tests de la page exigent (`IDENTIFIANTS_DE_LA_PAGE`) sont **chacun une seule fois dans `index.html`** (aucun doublon, aucun absent : à n'importe quelle place, dans n'importe quel élément, dans un tiroir, un `<aside>`, un `<dialog>`
/// ou un élément `hidden`) et **aucun autre fichier HTML du dossier** n'en reprend un ; les sept de l'étape 5 (`IDENTIFIANTS`) y sont ; et **tout identifiant que le code de la page demande par un texte littéral** (`getElementById("…")`, `querySelector("#…")`, `$("#…")`) **existe** dans `index.html`
/// (un script ne demande pas un élément absent). Aucun test n'exige la **place** d'un élément.
///
/// Ne couvre pas : que le script demande chaque élément (un script peut les garder autrement, ou n'en lire qu'une partie) ; un identifiant demandé par un texte composé.
/// Fiche : V-V8-30 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn les_identifiants_sont_dans_index_html_une_seule_fois() {
    let html = lire_page("index.html");
    let tous: Vec<String> = balises(&html).iter().filter_map(|balise| attribut(balise, "id")).collect();
    for id in IDENTIFIANTS_DE_LA_PAGE {
        let fois = tous.iter().filter(|autre| autre.as_str() == id).count();
        assert_eq!(fois, 1, "l'identifiant « {id} » est écrit {fois} fois dans index.html (une seule, exactement)");
    }
    for id in IDENTIFIANTS {
        assert!(IDENTIFIANTS_DE_LA_PAGE.contains(&id), "{id} manque dans la liste de la page");
    }
    for (nom, autre) in textes_des_fichiers(&["html", "htm"]) {
        if nom == "index.html" {
            continue;
        }
        for balise in balises(&autre) {
            if let Some(id) = attribut(balise, "id") {
                assert!(
                    !IDENTIFIANTS_DE_LA_PAGE.contains(&id.as_str()),
                    "{nom} reprend l'identifiant « {id} » : ils sont tous dans index.html"
                );
            }
        }
    }
    // Les identifiants demandés par un texte littéral existent.
    let code = lire_le_code_de_la_page();
    let mut demandes: BTreeSet<String> = BTreeSet::new();
    for debut in ["getElementById(", "querySelector(", "$("] {
        for (position, _) in code.match_indices(debut) {
            let apres = code[position + debut.len()..].trim_start();
            let Some(guillemet) = apres.chars().next().filter(|c| matches!(c, '"' | '\'' | '`')) else { continue };
            let Some(fin) = apres[1..].find(guillemet) else { continue };
            let texte = &apres[1..1 + fin];
            let id = if debut == "getElementById(" { Some(texte) } else { texte.strip_prefix('#') };
            if let Some(id) =
                id.filter(|id| !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
            {
                demandes.insert(id.to_string());
            }
        }
    }
    for id in &demandes {
        assert!(
            tous.iter().any(|autre| autre == id),
            "le code de la page demande l'élément « {id} » : index.html n'en a pas"
        );
    }
}

/// forme : `lecture.js` relit les mots du module non signés (`>>> 0` : JavaScript lit un `i32`) ; **si** une constante `INTERVALLE_EMPREINTE_MS` est déclarée dans le code de la page (`const INTERVALLE_EMPREINTE_MS = <nombre>`), elle vaut au moins 400 et est employée ailleurs que dans sa déclaration (huit appels de `givre_empreinte_mot`
/// hachent huit fois tout l'état) ; et le code de la page, **hors `lecture.js`**, lit l'empreinte (`lireEmpreinte(`).
///
/// Ne couvre pas : que la lecture respecte vraiment un intervalle (le nom de la constante n'est plus exigé : une page qui lit l'empreinte autrement, par exemple à l'arrêt seulement, n'est pas vue ici : voir `MODELE.md`, étape V8).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn lecture_js_lit_l_empreinte_non_signee_et_l_intervalle_est_borne() {
    let lecture = lire_le_code_de_lecture_js();
    assert!(
        lecture.contains(">>> 0") || lecture.contains(">>>0"),
        "lecture.js ne relit pas les mots de 32 bits non signés (>>> 0)"
    );
    let code = lire_le_code_de_la_page();
    assert!(
        code_js_de_la_page().iter().any(|(nom, c)| !nom.ends_with("lecture.js") && c.contains("lireEmpreinte(")),
        "la page ne lit l'empreinte nulle part hors de lecture.js"
    );

    let nom = "INTERVALLE_EMPREINTE_MS";
    // La déclaration : `const INTERVALLE_EMPREINTE_MS = <nombre>` (la première occurrence n'est pas forcément elle).
    let declaration = code.match_indices(nom).find_map(|(debut, _)| {
        let apres = code[debut + nom.len()..].trim_start();
        let valeur = apres.strip_prefix('=').filter(|reste| !reste.starts_with('='))?;
        code[..debut].trim_end().ends_with("const").then_some(valeur.trim_start())
    });
    if let Some(valeur) = declaration {
        let chiffres: String = valeur.chars().take_while(char::is_ascii_digit).collect();
        let intervalle: u64 = chiffres.parse().expect("INTERVALLE_EMPREINTE_MS = <un entier>");
        assert!(intervalle >= 400, "INTERVALLE_EMPREINTE_MS vaut {intervalle} : il faut au moins 400");
        assert!(code.matches(nom).count() >= 2, "INTERVALLE_EMPREINTE_MS n'est pas employée");
    }
}

/// forme : **chaque fichier `.js` ou `.mjs` de la page** (`app.js`, `lecture.js`, un worker…) est du JavaScript valide (modules) : `node --check`, le texte lui étant passé sur l'entrée standard. Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution des scripts (celle de `lecture.js` : voir `lecture_js_lit_le_module_comme_le_natif`) ; les scripts en ligne.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn les_scripts_de_la_page_sont_du_javascript_valide() {
    let scripts = textes_des_fichiers(&["js", "mjs"]);
    assert!(scripts.iter().any(|(nom, _)| nom == "lecture.js"), "lecture.js n'est pas dans la page");
    if !node_est_la() {
        return;
    }
    for (nom, script) in scripts {
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

/// forme : `lecture.js` déclare chacune des trois fonctions `lireEmpreinte`, `lireImage` et `lireGraine` par `export function <nom>(` et n'emploie ni `document`, ni `window` (Node le charge sans navigateur) ; **un fichier de la page autre que `lecture.js`** (`app.js`, un worker…) importe de
/// `lecture.js` (`import { … } from "…/lecture.js"`, un chemin relatif) ; chaque appel de ces trois fonctions, où qu'il soit dans la page, a un seul argument.
///
/// Ne couvre pas : ce que la page fait de ce qu'elles rendent (l'afficher, le dessiner) ; que chacune soit appelée (une page qui ne lit plus l'image d'un pixel par case n'appelle plus `lireImage`).
/// Fiche : V-V8-37 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_emploie_lecture_js() {
    let importe = code_js_de_la_page().iter().any(|(nom, code)| {
        !nom.ends_with("lecture.js")
            && ["lecture.js\"", "lecture.js'", "lecture.js`"].iter().any(|fin| {
                code.match_indices(fin).any(|(debut, _)| {
                    let avant = &code[..debut];
                    avant.rfind(['"', '\'', '`']).is_some_and(|guillemet| {
                        let chemin = &avant[guillemet + 1..];
                        let devant = code[..guillemet].trim_end();
                        !adresse_externe(chemin) && (devant.ends_with("from") || devant.ends_with("import("))
                    })
                })
            })
    });
    assert!(
        importe,
        "aucun fichier de la page, hors lecture.js, n'importe de lecture.js (import {{ … }} from \"./lecture.js\")"
    );

    let code = lire_le_code_de_la_page();
    let lecture = lire_le_code_de_lecture_js();
    for nom in FONCTIONS_DE_LECTURE {
        let arites = arites_des_appels(&code, nom);
        assert!(
            arites.iter().all(|arite| *arite == 1),
            "la page appelle {nom} avec {arites:?} argument(s) au lieu de 1"
        );
        assert!(
            lecture.contains(&format!("export function {nom}(")),
            "lecture.js ne déclare pas `export function {nom}(`"
        );
    }
    for interdit in ["document", "window"] {
        assert!(
            !noms_du_code(&lecture).contains(interdit),
            "lecture.js emploie `{interdit}` : il doit rester sans DOM"
        );
    }
}

/// Les mots (suites de lettres, de chiffres, de `_` et de `$`) du code, hors des chaînes de caractères.
fn noms_du_code(code: &str) -> BTreeSet<String> {
    let mut noms = BTreeSet::new();
    let mut mot = String::new();
    let mut guillemet: Option<char> = None;
    let mut echappe = false;
    for c in code.chars().chain(std::iter::once(' ')) {
        if let Some(g) = guillemet {
            if echappe {
                echappe = false;
            } else if c == '\\' {
                echappe = true;
            } else if c == g {
                guillemet = None;
            }
            continue;
        }
        if c.is_alphanumeric() || c == '_' || c == '$' {
            mot.push(c);
            continue;
        }
        if !mot.is_empty() {
            noms.insert(std::mem::take(&mut mot));
        }
        if c == '"' || c == '\'' || c == '`' {
            guillemet = Some(c);
        }
    }
    noms
}

/// Les textes que `lecture-wasm.mjs` donne à `lireGraine`, dans son ordre, et ce que la doc de la page en attend (calcul exact
/// d'après la règle de `lireGraine` : `trim()`, chiffres décimaux ASCII seulement, au plus 4294967295).
const GRAINES_ATTENDUES: [(&str, &str); 14] = [
    ("0", "0"),
    ("1", "1"),
    (" 42 ", "42"),
    ("007", "7"),
    ("4294967295", "4294967295"),
    ("4294967296", "null"),
    ("-1", "null"),
    ("", "null"),
    (" ", "null"),
    ("abc", "null"),
    ("1.5", "null"),
    ("1e3", "null"),
    ("0x10", "null"),
    ("+7", "null"),
];

/// juste : `lecture.js`, la partie de la page qui lit le module, chargé par Node avec le module commité
/// (`tests/etape_5/lecture-wasm.mjs`), lit **ce que le code natif donne** : l'empreinte affichée par la page (`lireEmpreinte`)
/// est celle du natif, l'image dessinée (`lireImage` : ses dimensions et le SHA-256 de ses octets R, V, B, 255, ligne par
/// ligne) est l'aperçu de la bibliothèque, et `rgba` est un `Uint8ClampedArray` ; `lireGraine` rend, pour quatorze textes, ce
/// que dit sa règle (`GRAINES_ATTENDUES`, calcul exact). Les cas : la DLA de graine 1 jusqu'au bout (2999 : l'empreinte est
/// alors celle de `references/dla-graine-1.empreinte`), une grande graine à 77 particules, l'étoile à 37 pas, la dendrite au
/// bout, la plaque au temps 0. Une empreinte affichée à l'envers, des mots lus signés, une image transposée ou des canaux
/// permutés se voient ici. Les arguments faux du script le font sortir en code 1, sortie standard vide.
///
/// Ne couvre pas : ce qu'`app.js` fait de ce que rend `lecture.js` (l'afficher dans `#empreinte`, le dessiner sur le canvas,
/// passer la graine lue à `givre_demarrer`) : seul un navigateur le montre.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn lecture_js_lit_le_module_comme_le_natif() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");
    let lecture = chemin_de_lecture();
    let lecture = lecture.to_str().expect("chemin en UTF-8");
    let graines: Vec<&str> = GRAINES_ATTENDUES.iter().map(|(_, attendu)| *attendu).collect();

    let cas: [(u32, u32, u32); 5] = [(0, 1, 2999), (0, 0xDEAD_BEEF, 77), (2, 0, 37), (3, 1, 800), (1, 1, 0)];
    for (modele, graine, pas) in cas {
        let arguments = [module, lecture, &modele.to_string(), &graine.to_string(), &pas.to_string()];
        let sortie = lancer_le_script("lecture-wasm.mjs", &arguments);
        let erreur = String::from_utf8_lossy(&sortie.stderr);
        assert!(
            sortie.status.success(),
            "lecture-wasm.mjs {arguments:?} : {:?}, sortie d'erreur : {erreur}",
            sortie.status
        );

        let etat = natif(modele, graine, pas);
        let mut rgba = Vec::with_capacity(etat.rvb.len() / 3 * 4);
        for pixel in etat.rvb.chunks(3) {
            rgba.extend_from_slice(pixel);
            rgba.push(255);
        }
        let attendu = format!(
            "empreinte={}\nlargeur={}\nhauteur={}\nrgba={}\nsorte=Uint8ClampedArray\ngraines={}\n",
            etat.empreinte,
            etat.largeur,
            etat.hauteur,
            sha256_hex(&rgba),
            graines.join(";"),
        );
        assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu, "modèle {modele}, graine {graine}, {pas} pas");
    }
    let reference = std::fs::read_to_string(racine().join("references").join("dla-graine-1.empreinte"))
        .expect("references/dla-graine-1.empreinte se lit");
    assert_eq!(natif(0, 1, 2999).empreinte, reference.trim());

    for arguments in
        [&[module, lecture, "0", "1"][..], &[module, lecture, "4", "1", "0"], &[module, "absent.js", "0", "1", "0"]]
    {
        let sortie = lancer_le_script("lecture-wasm.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "lecture-wasm.mjs {arguments:?}");
        assert!(sortie.stdout.is_empty(), "lecture-wasm.mjs {arguments:?} : la sortie standard n'est pas vide");
    }
}

/// forme : **la page ne charge rien du dehors** (V8). Ni un `src` (script, image, iframe, source…), ni le `href` d'un `<link>` (feuille de style, police, icône, préchargement, manifeste) ou d'un élément SVG autre qu'un lien, ni un `url(…)` ou un `@import` d'un CSS (une feuille, un `<style>`, un attribut `style`), ni une adresse
/// que le JavaScript charge (`fetch(`, `new Worker(`, `importScripts(`, `import(`, `import … from`, `.src =`, `EventSource`, `WebSocket`, `sendBeacon`, le second argument de `.open(`) ne désigne une adresse **externe** (un protocole `http`, `https`, `ws`, `wss`, `ftp`, `file`, ou `//…`), dans **aucun fichier** du dossier de la page
/// (HTML, scripts en ligne, CSS, JavaScript, SVG ; un contenu `data:` n'est pas lu comme un texte). **Passent** : une adresse relative, `data:`, `blob:`, un **lien** `<a href="https://…">`, l'attribut `xmlns="http://www.w3.org/2000/svg"` d'un SVG en ligne, une balise `<meta>` (`og:image`) et un `<link rel="canonical">`.
/// Les noms des hébergeurs (`unpkg`, `jsdelivr`, `cdnjs`, `googleapis`, `gstatic`) ne sont plus cherchés comme du texte : une adresse qui les contiendrait est externe et se voit par son protocole.
///
/// Ne couvre pas : une ressource chargée par une adresse composée (`"https:" + "//…"`) ; une adresse écrite dans une chaîne que le script passe plus tard à `fetch` ; ce que le navigateur charge sans qu'une adresse soit écrite.
/// Fiche : V-V8-31 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_n_a_aucune_ressource_externe() {
    let externes = ressources_externes();
    assert!(externes.is_empty(), "la page charge des ressources externes :\n{}", externes.join("\n"));
    // Le détecteur voit ce qu'il doit voir (un compte vide n'est pas un compte vert) : une adresse externe, et seulement elle.
    assert!(
        adresse_externe("https://exemple.org/x.js")
            && adresse_externe("//cdn.exemple.org/x.js")
            && adresse_externe("HTTP://exemple.org")
    );
    assert!(
        !adresse_externe("./calcul.js")
            && !adresse_externe("polices/inter.woff2")
            && !adresse_externe("data:font/woff2;base64,AAAA")
            && !adresse_externe("blob:abc")
            && !adresse_externe("/docs/x")
    );
}

/// forme : **la page tient dans son budget** (V8) : tout le **code** du dossier de la page (HTML, CSS, JavaScript, SVG, polices, données : tout fichier qui n'est ni une image ni le module, les polices en `data:` comprises puisqu'elles sont dans le CSS) pèse au plus `CODE_MAX` octets (384 Kio) ; **une image**
/// (`png`, `jpg`, `jpeg`, `webp`, `avif`, `gif`, `ico`) au plus `IMAGE_MAX` (300 000 octets) et **toutes les images ensemble** au plus `IMAGES_MAX` (600 000) ; chaque nombre est un seul `const` de `etape_5_commun`. Le module a son plafond dans `etape_5_artefact.rs` (1 Mio). Un dossier vide est un échec.
///
/// Ne couvre pas : le poids transféré (compression) ; le poids d'une police ou d'une image hors du dossier de la page.
/// Fiche : V-V8-32 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_ne_depasse_pas_son_budget() {
    let fichiers = fichiers_de_la_page();
    let code: u64 = fichiers.iter().filter(|f| !est_une_image(&f.nom)).map(|f| f.taille).sum();
    assert!(code > 0, "la page n'a aucun fichier de code");
    assert!(code <= CODE_MAX, "le code de la page pèse {code} octets : {CODE_MAX} au plus");
    let images: Vec<&etape_5_commun::FichierDePage> = fichiers.iter().filter(|f| est_une_image(&f.nom)).collect();
    for image in &images {
        assert!(image.taille <= IMAGE_MAX, "l'image {} pèse {} octets : {IMAGE_MAX} au plus", image.nom, image.taille);
    }
    let total: u64 = images.iter().map(|f| f.taille).sum();
    assert!(total <= IMAGES_MAX, "les images de la page pèsent {total} octets : {IMAGES_MAX} au plus");
}

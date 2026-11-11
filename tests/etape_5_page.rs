//! Tests d'acceptation de l'étape 5 (c) : la page `docs/index.html` + `docs/app.js` + `docs/lecture.js`. Presque tous sont des
//! **tests de forme** : ils lisent les fichiers comme du texte (et Node vérifie que `app.js` et `lecture.js` sont du
//! JavaScript valide). Un seul fait tourner du code de la page : `lecture_js_lit_le_module_comme_le_natif` fait lire le
//! module commité par `lecture.js` (sous Node, sans navigateur) et compare l'empreinte, l'image et la graine qu'il lit au
//! code natif. Le reste de la page (le DOM, le canvas, l'animation) ne tourne que dans un navigateur : il n'est éprouvé que
//! par sa forme et par un coup d'œil humain ; voir « Ce que cela n'établit pas » dans `MODELE.md`.
//!
//! Ce test-là lit `docs/givre.wasm` : il ne passe qu'une fois le module construit et commité.
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent)
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
use std::path::Path;
use std::process::{Command, Stdio};

use etape_5_commun::{
    D3_ACTIVE, FONCTIONS, FONCTIONS_D3, FONCTIONS_V7, FONCTIONS_Z1, V7_ACTIVE, Z1_ACTIVE, chemin_module,
    lancer_le_script, lire_page, natif, node_est_la, racine,
};
use givre_lab::empreinte::sha256_hex;

/// Le plafond de la page, tous fichiers de `docs/` confondus sauf le module : 200 Ko.
const TAILLE_MAX_PAGE: u64 = 200 * 1024;

/// Les éléments que la page doit avoir, chacun par son `id` ; `app.js` les demande tous.
const IDENTIFIANTS: [&str; 7] = ["vitrine", "modele", "graine", "relancer", "vitesse", "temps", "empreinte"];

/// Le code de `app.js` sans ses commentaires : sans les blocs `/* … */` et sans les `//` jusqu'à la fin de la ligne, partout
/// où ils ne sont pas dans une chaîne de caractères (`"…"`, `'…'` ou `` `…` ``, avec leurs échappements). Les chaînes sont
/// gardées telles quelles.
fn sans_commentaires(script: &str) -> String {
    let c: Vec<char> = script.chars().collect();
    let mut sortie = String::new();
    let mut i = 0;
    while i < c.len() {
        match c[i] {
            '"' | '\'' | '`' => {
                let guillemet = c[i];
                sortie.push(guillemet);
                i += 1;
                while i < c.len() && c[i] != guillemet {
                    if c[i] == '\\' && i + 1 < c.len() {
                        sortie.push(c[i]);
                        i += 1;
                    }
                    sortie.push(c[i]);
                    i += 1;
                }
                if i < c.len() {
                    sortie.push(guillemet);
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'/') => {
                while i < c.len() && c[i] != '\n' {
                    i += 1;
                }
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                sortie.push(' ');
            }
            autre => {
                sortie.push(autre);
                i += 1;
            }
        }
    }
    sortie
}

/// Le code de `docs/app.js`, sans ses commentaires.
fn lire_le_code_de_app_js() -> String {
    sans_commentaires(&lire_page("app.js"))
}

/// Le code de `docs/lecture.js`, sans ses commentaires.
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

/// Les identifiants que le script demande par `getElementById("…")` (ou avec des apostrophes).
fn identifiants_demandes(script: &str) -> Vec<String> {
    let mut identifiants = Vec::new();
    let mut reste = script;
    while let Some(debut) = reste.find("getElementById(") {
        let apres = reste[debut + "getElementById(".len()..].trim_start();
        let Some(guillemet) = apres.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            panic!("getElementById doit recevoir un texte littéral : {}", &apres[..apres.len().min(30)]);
        };
        let nom = &apres[1..];
        let fin = nom.find(guillemet).expect("guillemet fermant de getElementById");
        identifiants.push(nom[..fin].to_string());
        reste = &nom[fin..];
    }
    identifiants
}

/// Les balises du HTML, chacune réduite à son contenu entre `<` et `>` (le nom, puis les attributs).
fn balises(html: &str) -> Vec<&str> {
    html.split('<').skip(1).filter_map(|morceau| morceau.split('>').next()).collect()
}

/// La valeur de l'attribut `nom` d'une balise (écrit entre guillemets ou entre apostrophes, précédé d'une espace).
fn attribut(balise: &str, nom: &str) -> Option<String> {
    let motif = format!("{nom}=");
    for (position, _) in balise.match_indices(&motif) {
        if !balise[..position].ends_with(char::is_whitespace) {
            continue;
        }
        let apres = &balise[position + motif.len()..];
        let guillemet = apres.chars().next().filter(|c| *c == '"' || *c == '\'')?;
        let valeur = &apres[1..];
        return Some(valeur[..valeur.find(guillemet)?].to_string());
    }
    None
}

/// Le nom d'une balise, en minuscules (`canvas` pour `<canvas id="x">`).
fn nom_de_balise(balise: &str) -> String {
    balise.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// La première balise du HTML qui a cet `id`.
fn balise_d_id<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    balises(html).into_iter().find(|balise| attribut(balise, "id").as_deref() == Some(id))
}

/// forme : `docs/index.html` contient un `<canvas` d'`id` `vitrine`, une balise `<meta name="viewport"` dont le contenu dit
/// `width=device-width` (lisible sur un téléphone), un `<title>`, et un `<script type="module"` qui charge `app.js`.
///
/// Ne couvre pas : ce que la page montre ; que le script se charge ; la lisibilité réelle sur un téléphone.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_a_un_canvas_un_viewport_et_charge_app_js() {
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

    let debut = minuscules.find("<script type=\"module\"").expect("pas de <script type=\"module\" dans index.html");
    let fin = minuscules[debut..].find('>').expect("balise <script> non fermée");
    let balise = &minuscules[debut..debut + fin];
    assert!(
        balise.contains("src=") && balise.contains("app.js"),
        "le script de type module ne charge pas app.js : {balise}"
    );
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
    if D3_ACTIVE {
        assert_eq!(valeurs, ["0", "1", "2", "3", "claquage"], "les <option> du choix du modèle");
    } else {
        assert!(
            valeurs == ["0", "1", "2", "3"] || valeurs == ["0", "1", "2", "3", "claquage"],
            "les <option> du choix du modèle : {valeurs:?}"
        );
    }
    for ((_, texte), mot) in options.iter().zip(["dla", "plaque", "toile", "dendrite", "claquage"]) {
        assert!(texte.contains(mot), "le texte de l'option « {texte} » ne dit pas « {mot} »");
    }
}

/// forme : `docs/app.js` désigne le module par le texte littéral `"givre.wasm"` (ou `"./givre.wasm"`), c'est-à-dire par un
/// chemin relatif à la page, qui est servie depuis `docs/` (jamais `/docs/givre.wasm`), et l'instancie par
/// `WebAssembly.instantiate` (ou `new WebAssembly.Instance`).
///
/// Ne couvre pas : que le chargement réussisse dans un navigateur.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_charge_le_module() {
    let code = lire_le_code_de_app_js();
    let litteral =
        |forme: &str| ["\"", "'", "`"].iter().any(|guillemet| code.contains(&format!("{guillemet}{forme}{guillemet}")));
    assert!(
        litteral("givre.wasm") || litteral("./givre.wasm"),
        "app.js ne désigne pas le module par le texte \"givre.wasm\""
    );
    assert!(
        code.contains("WebAssembly.instantiate") || code.contains("new WebAssembly.Instance"),
        "app.js n'instancie pas le module"
    );
}

/// forme : `app.js` avance la simulation par `requestAnimationFrame` et dessine sur le canvas par son contexte 2D
/// (`getContext("2d")`).
///
/// Ne couvre pas : ce qui est dessiné.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_anime_et_dessine_sur_le_canvas() {
    let code = lire_le_code_de_app_js();
    assert!(code.contains("requestAnimationFrame"), "app.js n'emploie pas requestAnimationFrame");
    assert!(
        code.contains("getContext(\"2d\")") || code.contains("getContext('2d')"),
        "app.js n'emploie pas le contexte 2D du canvas"
    );
}

/// forme : chaque nom `givre_…` qu'emploient `app.js` et `lecture.js` (leurs deux codes mis bout à bout) est l'une des onze
/// fonctions de la vitrine, et chacune est employée ; chaque fonction est **appelée** (`nom(…)`) au moins une fois et **tous**
/// ses appels ont le bon nombre d'arguments (`givre_pixel(x)` sans `y`, ou `givre_demarrer(modele)` sans graine, sont vus).
/// Les noms s'écrivent en entier dans le script, jamais composés. (Ce test relit la liste des fonctions dans
/// `etape_5_commun::FONCTIONS`, que `etape_5_artefact.rs` compare aux exportations du module.)
///
/// Ne couvre pas : que les arguments soient les bons (un `x` pour un `y` dans `app.js` : ceux de `lecture.js` sont éprouvés
/// par `lecture_js_lit_le_module_comme_le_natif`) ; que l'appel soit atteint.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_emploie_exactement_les_exportations_du_module() {
    let code = format!("{}\n{}", lire_le_code_de_app_js(), lire_le_code_de_lecture_js());
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
    let employes = noms_employes(&code);
    let exportees = if V7_ACTIVE || employes == vingt_sept {
        vingt_sept
    } else if D3_ACTIVE || employes == vingt_trois {
        vingt_trois
    } else if Z1_ACTIVE || employes == quinze {
        quinze
    } else {
        onze
    };

    let inconnus: Vec<&String> = employes.difference(&exportees).collect();
    assert!(inconnus.is_empty(), "app.js emploie des noms que le module n'exporte pas : {inconnus:?}");
    let oublies: Vec<&String> = exportees.difference(&employes).collect();
    assert!(oublies.is_empty(), "app.js n'emploie pas ces exportations du module : {oublies:?}");

    for (nom, parametres, _) in FONCTIONS
        .iter()
        .chain(FONCTIONS_Z1.iter())
        .chain(FONCTIONS_D3.iter())
        .chain(FONCTIONS_V7.iter())
        .filter(|(nom, _, _)| exportees.contains(*nom))
    {
        let arites = arites_des_appels(&code, nom);
        assert!(!arites.is_empty(), "app.js n'appelle jamais {nom}(…)");
        for arite in arites {
            assert_eq!(
                arite,
                parametres.len(),
                "app.js appelle {nom} avec {arite} argument(s) au lieu de {}",
                parametres.len()
            );
        }
    }
}

/// forme : `app.js` demande chaque élément de la liste (`vitrine`, `modele`, `graine`, `relancer`, `vitesse`, `temps`,
/// `empreinte`) par `document.getElementById("…")`, et tous les identifiants qu'il demande existent dans la page ; le canvas
/// de la page est celui qu'il demande (`vitrine`).
///
/// Ne couvre pas : l'usage qu'`app.js` fait de ces éléments.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_ne_demande_que_des_elements_de_la_page_et_tous_ceux_de_la_liste() {
    let html = lire_page("index.html");
    let demandes: BTreeSet<String> = identifiants_demandes(&lire_le_code_de_app_js()).into_iter().collect();
    assert!(!demandes.is_empty(), "app.js ne demande aucun élément par getElementById");

    let existants: BTreeSet<String> = balises(&html).iter().filter_map(|balise| attribut(balise, "id")).collect();
    for identifiant in &demandes {
        assert!(existants.contains(identifiant), "index.html n'a pas d'élément d'id « {identifiant} »");
    }
    for identifiant in IDENTIFIANTS {
        assert!(demandes.contains(identifiant), "app.js ne demande pas l'élément d'id « {identifiant} »");
        assert!(existants.contains(identifiant), "index.html n'a pas d'élément d'id « {identifiant} »");
    }
}

/// forme : `lecture.js` relit les mots du module non signés (`>>> 0` : JavaScript lit un `i32`), et `app.js` lit l'empreinte
/// au plus toutes les 400 ms : une constante `INTERVALLE_EMPREINTE_MS` d'au moins 400, écrite `INTERVALLE_EMPREINTE_MS =
/// <nombre>` (avec `const`, un entier en chiffres décimaux), et employée ailleurs que dans sa déclaration (huit appels de
/// `givre_empreinte_mot` hachent huit fois tout l'état).
///
/// Ne couvre pas : que la lecture respecte vraiment cet intervalle.
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_lit_l_empreinte_non_signee_et_rarement() {
    let lecture = lire_le_code_de_lecture_js();
    assert!(
        lecture.contains(">>> 0") || lecture.contains(">>>0"),
        "lecture.js ne relit pas les mots de 32 bits non signés (>>> 0)"
    );
    let code = lire_le_code_de_app_js();

    let nom = "INTERVALLE_EMPREINTE_MS";
    // La déclaration : `const INTERVALLE_EMPREINTE_MS = <nombre>` (la première occurrence n'est pas forcément elle).
    let declaration = code.match_indices(nom).find_map(|(debut, _)| {
        let apres = code[debut + nom.len()..].trim_start();
        let valeur = apres.strip_prefix('=').filter(|reste| !reste.starts_with('='))?;
        code[..debut].trim_end().ends_with("const").then_some(valeur.trim_start())
    });
    let valeur = declaration.expect("app.js n'a pas de `const INTERVALLE_EMPREINTE_MS = <nombre>`");
    let chiffres: String = valeur.chars().take_while(char::is_ascii_digit).collect();
    let intervalle: u64 = chiffres.parse().expect("INTERVALLE_EMPREINTE_MS = <un entier>");
    assert!(intervalle >= 400, "INTERVALLE_EMPREINTE_MS vaut {intervalle} : il faut au moins 400");
    assert!(code.matches(nom).count() >= 2, "INTERVALLE_EMPREINTE_MS n'est pas employée");
}

/// forme : `app.js` et `lecture.js` sont du JavaScript valide (modules) : `node --check`, le texte lui étant passé sur l'entrée
/// standard. Le contrôle est sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution de `app.js` (celle de `lecture.js` : voir `lecture_js_lit_le_module_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_est_du_javascript_valide() {
    if !node_est_la() {
        return;
    }
    for nom in ["app.js", "lecture.js"] {
        let script = lire_page(nom);
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

/// forme : `app.js` importe de `"./lecture.js"` (un `import { … } from "./lecture.js"`, chemin relatif à la page) les trois
/// fonctions `lireEmpreinte`, `lireImage` et `lireGraine`, et appelle chacune, chaque fois avec un argument ; `lecture.js`
/// les déclare chacune par `export function <nom>(` et n'emploie ni `document`, ni `window` (Node le charge sans navigateur).
///
/// Ne couvre pas : ce qu'`app.js` fait de ce qu'elles rendent (l'afficher, le dessiner) ; voir « Ce que cela n'établit pas ».
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn app_js_emploie_lecture_js() {
    let code = lire_le_code_de_app_js();
    let importe = ["\"./lecture.js\"", "'./lecture.js'"]
        .iter()
        .any(|nom| code.match_indices(nom).any(|(debut, _)| code[..debut].trim_end().ends_with("from")));
    assert!(importe, "app.js n'importe rien de \"./lecture.js\" (import {{ … }} from \"./lecture.js\")");
    assert!(code.contains("import"), "app.js n'a pas d'import");

    let lecture = lire_le_code_de_lecture_js();
    for nom in FONCTIONS_DE_LECTURE {
        let arites = arites_des_appels(&code, nom);
        assert!(!arites.is_empty(), "app.js n'appelle jamais {nom}(…)");
        assert!(
            arites.iter().all(|arite| *arite == 1),
            "app.js appelle {nom} avec {arites:?} argument(s) au lieu de 1"
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
    let lecture = racine().join("docs").join("lecture.js");
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

/// forme : ni `index.html`, ni `app.js`, ni `lecture.js` ne contiennent `http://`, `https://` ni `//cdn` : aucune ressource externe, ni script,
/// ni police, ni image. Sont aussi refusés les adresses sans protocole (`"//…"`, `(//…)`, `=//…`) et les noms des hébergeurs
/// de bibliothèques et de polices les plus courants. (Le texte entier est lu, commentaires compris.)
///
/// Ne couvre pas : une ressource externe écrite autrement (un nom d'hôte sans protocole ni barres).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_n_a_aucune_ressource_externe() {
    let interdits = [
        "http://",
        "https://",
        "//cdn",
        "\"//",
        "'//",
        "`//",
        "(//",
        "=//",
        "unpkg",
        "jsdelivr",
        "cdnjs",
        "googleapis",
        "gstatic",
    ];
    for nom in ["index.html", "app.js", "lecture.js"] {
        let texte = lire_page(nom).to_lowercase();
        for interdit in interdits {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} » : une ressource externe ?");
        }
    }
}

/// Ajoute à `total` la taille des fichiers de `dossier`, sous-dossiers compris, sauf le module.
fn taille_sans_le_module(dossier: &Path, total: &mut u64) {
    let entrees = std::fs::read_dir(dossier).unwrap_or_else(|erreur| panic!("{} se lit : {erreur}", dossier.display()));
    for entree in entrees {
        let chemin = entree.expect("entrée de dossier").path();
        if chemin.is_dir() {
            taille_sans_le_module(&chemin, total);
        } else if chemin != chemin_module() {
            *total += std::fs::metadata(&chemin).expect("métadonnées").len();
        }
    }
}

/// forme : la page ne dépasse pas 200 Ko en tout : tous les fichiers de `docs/`, hors le module.
///
/// Ne couvre pas : le poids du module (voir `etape_5_artefact.rs`).
#[test]
#[cfg_attr(not(feature = "etape-5"), ignore)]
fn la_page_ne_depasse_pas_200_ko() {
    let mut total = 0;
    taille_sans_le_module(&racine().join("docs"), &mut total);
    assert!(total > 0, "docs/ est vide");
    assert!(total <= TAILLE_MAX_PAGE, "la page pèse {total} octets");
}

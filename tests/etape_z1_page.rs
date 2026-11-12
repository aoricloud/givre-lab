//! Étape Z1 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) et le module `docs/givre.wasm` qui montrent la mesure de la
//! corrélation. Valeurs attendues : `references/etape-z1-valeurs.md`, identifiants `V-Z1-xx`. Comme à l'étape 5, presque tous les tests sont
//! des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node : il fait lire par
//! `lecture.js` (`lireCorrelation`) la courbe que le module commité donne, et la compare au code natif. **Ce test lit `docs/givre.wasm` : il ne
//! passe qu'une fois le module reconstruit** avec les quatre fonctions de mesure (la commande est dans la doc de `src/vitrine.rs`).
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent)
//!
//! - `index.html` ajoute, en gardant tout ce que l'étape 5 demande : un `<button id="mesurer">` (texte « Mesurer D » : la mesure se fait à la demande,
//!   jamais à chaque image) ; un second `<canvas id="courbe">` (la courbe `C(r)` en échelle log-log et la droite ajustée) ; un élément
//!   `id="resultat"` (la ligne « mesuré par le banc : D = … à N sites ») ; un élément `id="barre-zinc"` (la barre du repère) avec un
//!   repère `id="marque-d"` (la valeur mesurée sur la barre) ; un élément `id="repere-zinc"` dont le texte dit « zinc, Matsushita et coll. 1984 :
//!   1,66 ± 0,03 » (les mots `zinc`, `Matsushita`, `1984`, `1,66` et `0,03`) ; un élément `id="compatible"` dont le texte est
//!   « compatible, ce qui ne prouve pas que le zinc suit la DLA » ; un élément `id="hors-barre"` pour la phrase du cas contraire (« hors de la
//!   barre du zinc… », texte libre) ; **ces deux phrases portent l'attribut `hidden` dans `index.html`** (aucune n'est visible avant une mesure) et
//!   `app.js`, après une mesure, montre `compatible` si et seulement si `|D - 1,66| <= 0,03` (D la dimension mesurée, bornes comprises) et
//!   `hors-barre` sinon, et les cache toutes deux quand il n'y a pas de mesure ; un élément
//!   `id="taille-zinc"` dont le texte est « taille d'amas du zinc non relevée » (l'étude n'a pas pu lire la taille des amas de zinc :
//!   `references/etape-z1-valeurs.md`, V-Z1-31).
//! - **Aucune valeur de D n'est écrite dans la page** : `index.html` n'a aucun « D = » suivi d'un chiffre, et dans `app.js` le texte « mesuré par le
//!   banc : D = » est suivi d'une valeur calculée (`${…}` d'un gabarit), jamais d'un nombre ; ni `index.html`, ni `app.js`, ni `lecture.js` ne contient
//!   « 1,71 » ni « 1.71 » (le repère publié de la DLA n'est pas une mesure du banc). Le repère du zinc, lui, est un texte et une constante.
//! - `lecture.js` exporte de plus `export function lireCorrelation(m)` (sans DOM), qui rend `{ rayons, valeurs, dimension }` : les rayons de la
//!   mesure gardée par le module (`m.givre_correlation_rayon(i) >>> 0` pour `i` = 0, 1, … tant que le rayon n'est pas 0), les `C(r)` de
//!   `m.givre_correlation_valeur(i)` (les `f64` du module, au bit près) et `m.givre_dimension()` ; sans mesure : `rayons` et `valeurs` vides et
//!   dimension `NaN`. Elle ne lance aucune mesure. `app.js` l'importe de `"./lecture.js"` et l'appelle avec un argument.
//! - `app.js` appelle `givre_mesurer()` **une seule fois**, dans une fonction déclarée (`function nom(`) qui est le gestionnaire du clic du bouton
//!   `mesurer` (`addEventListener("click", nom)`) et qui n'appelle ni `requestAnimationFrame` ni `givre_avancer` : la mesure, en `O(n²)`, ne se
//!   fait jamais à chaque image. À eux deux, `app.js` et `lecture.js` appellent chacune des **quinze** exportations du module (les onze de
//!   l'étape 5 et les quatre de l'étape Z1) avec le bon nombre d'arguments.
//!
//! Étiquettes : `lecture_js_lit_la_courbe_comme_le_natif` est **juste** (comparé au code natif) ; tous les autres sont de la **forme**
//! (régression de texte), jamais une preuve du comportement de la page.

mod etape_5_commun;

use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use etape_5_commun::{
    D3_ACTIVE, FONCTIONS, FONCTIONS_D3, FONCTIONS_G3, FONCTIONS_V7, FONCTIONS_V8, FONCTIONS_Z1, G3_ACTIVE, TABLE,
    V7_ACTIVE, Z1_ACTIVE, chaines_du_code, chemin_de_lecture, chemin_module, code_js_de_la_page, lire_page,
    node_est_la, racine, textes_des_fichiers, textes_ecrits_par_la_page, tout_le_code_js,
};
use givre_lab::correlation::mesure;
use givre_lab::dla::Amas;

// ---------------------------------------------------------------------------------------------------------------
// Lecture du texte de la page.
// ---------------------------------------------------------------------------------------------------------------

/// Le code d'un script sans ses commentaires `//` et `/* */` (hors des chaînes `"…"`, `'…'` et `` `…` ``, gardées telles quelles).
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

/// Tout le code JavaScript de la page (tous les fichiers `.js`, `.mjs` et les scripts en ligne), sans ses commentaires (V8).
fn code_de_app_js() -> String {
    tout_le_code_js()
}

fn code_de_lecture_js() -> String {
    sans_commentaires(&lire_page("lecture.js"))
}

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

/// Pour chaque appel `nom(…)` du texte, le nombre d'arguments (les virgules de premier niveau plus un, une virgule finale ne comptant pas,
/// ou 0 si les parenthèses sont vides). Un `nom` précédé d'une lettre, d'un chiffre ou d'un `_` est un autre nom et n'est pas compté.
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

/// Les balises du HTML, chacune réduite à son contenu entre `<` et `>`.
fn balises(html: &str) -> Vec<&str> {
    html.split('<').skip(1).filter_map(|morceau| morceau.split('>').next()).collect()
}

/// La valeur de l'attribut `nom` d'une balise (entre guillemets ou apostrophes, précédé d'une espace).
fn attribut(balise: &str, nom: &str) -> Option<String> {
    etape_5_commun::attribut(balise, nom)
}

fn nom_de_balise(balise: &str) -> String {
    balise.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// La première balise du HTML qui a cet `id`.
fn balise_d_id<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    balises(html).into_iter().find(|balise| attribut(balise, "id").as_deref() == Some(id))
}

/// Le texte qui suit la balise d'`id` donné, jusqu'au `<` suivant, sans espaces de début et de fin.
fn texte_de(html: &str, id: &str) -> Option<String> {
    etape_5_commun::texte_de_l_element(html, id)
}

// ---------------------------------------------------------------------------------------------------------------
// Les tests de forme.
// ---------------------------------------------------------------------------------------------------------------

/// forme : `index.html` a les éléments de la mesure, chacun avec son `id` et sa sorte : `<button id="mesurer">` (texte « Mesurer D »), `<canvas
/// id="courbe">` en plus du `<canvas id="vitrine">`, et des éléments d'`id` `resultat`, `barre-zinc`, `marque-d`, `repere-zinc`, `compatible`,
/// `hors-barre` et `taille-zinc` (V-Z1-27).
///
/// Ne couvre pas : que ces éléments soient visibles, lisibles ou reliés à quoi que ce soit.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_a_les_elements_de_la_mesure() {
    let html = lire_page("index.html");
    for (id, sorte) in [("mesurer", "button"), ("courbe", "canvas"), ("vitrine", "canvas")] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    for id in ["resultat", "barre-zinc", "marque-d", "repere-zinc", "compatible", "hors-barre", "taille-zinc"] {
        assert!(balise_d_id(&html, id).is_some(), "pas d'élément d'id « {id} »");
    }
    let bouton = texte_de(&html, "mesurer").expect("le bouton");
    assert!(bouton.contains("Mesurer D"), "le texte du bouton est « {bouton} »");
}

/// forme : les phrases (V-Z1-28). `repere-zinc` dit `zinc`, `Matsushita`, `1984`, `1,66` et `0,03` ; `compatible` est exactement « compatible, ce qui ne prouve pas que le zinc suit la DLA » ; `taille-zinc` est exactement « taille d'amas du zinc non relevée » (la taille d'amas des feuilles de
/// zinc n'a pas pu être lue, et la page ne l'invente pas) ; et le mot « zinc » ne figure dans la page qu'avec ces mots de repère, jamais comme un test (aucune phrase « dans la barre » n'est écrite à la place de `compatible`). **V8 : la phrase peut être écrite par un script** : si l'élément est vide dans
/// `index.html`, la phrase exacte (ou, pour `repere-zinc`, une même chaîne qui dit les cinq mots) doit être une chaîne de caractères de l'un des fichiers JavaScript de la page ; si l'élément a un texte, c'est lui qui est jugé (il est exactement la phrase).
///
/// Ne couvre pas : l'affichage de l'une ou l'autre phrase selon la mesure (le DOM n'est pas éprouvé) ; que le script écrive la phrase dans le bon élément.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_dit_le_repere_du_zinc_comme_un_repere() {
    let html = lire_page("index.html");
    let chaines = chaines_du_code(&code_de_app_js());
    let repere = texte_de(&html, "repere-zinc").expect("repere-zinc");
    let mots = ["zinc", "Matsushita", "1984", "1,66", "0,03"];
    if repere.is_empty() {
        assert!(
            chaines.iter().any(|chaine| mots.iter().all(|mot| chaine.contains(mot))),
            "repere-zinc est vide dans index.html et aucune chaîne d'un script ne dit zinc, Matsushita, 1984, 1,66 et 0,03"
        );
    } else {
        for mot in mots {
            assert!(repere.contains(mot), "le repère du zinc « {repere} » ne dit pas « {mot} »");
        }
    }
    for (id, phrase) in [
        ("compatible", "compatible, ce qui ne prouve pas que le zinc suit la DLA"),
        ("taille-zinc", "taille d'amas du zinc non relevée"),
    ] {
        let texte = texte_de(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        if texte.is_empty() {
            assert!(
                chaines.iter().any(|chaine| chaine.contains(phrase)),
                "« {id} » est vide dans index.html et aucun script n'écrit « {phrase} »"
            );
        } else {
            assert_eq!(texte, phrase, "le texte de « {id} »");
        }
    }
}

/// forme : aucune valeur de D n'est écrite dans la page (V-Z1-29). **Tout ce que la page écrit à l'écran** (le texte visible de chaque fichier HTML, les chaînes de tous les fichiers JavaScript, le texte des SVG) n'a aucun « D = » ni « D= » suivi d'un chiffre ; **aucun fichier de texte du dossier de la page**
/// (HTML, scripts en ligne, CSS, JavaScript, SVG, commentaires compris) ne contient « 1,71 » ni « 1.71 » (le repère publié de la DLA n'est pas une mesure du banc). V8 ne lit plus le gabarit `mesuré par le banc : D = ${…}` d'`app.js` (le texte d'un fichier précis) : qu'une valeur écrite soit calculée se voit ici par l'absence de tout chiffre.
///
/// Ne couvre pas : le texte réellement affiché (le DOM n'est pas éprouvé) ni qu'un nombre mesuré ne soit pas copié dans une constante autrement écrite.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_ne_fige_aucune_valeur_de_d() {
    let ecrit = textes_ecrits_par_la_page();
    assert!(!ecrit.trim().is_empty(), "la page n'écrit aucun texte");
    for motif in ["D =", "D="] {
        for (position, _) in ecrit.match_indices(motif) {
            let suite = ecrit[position + motif.len()..].trim_start();
            assert!(
                !suite.starts_with(|c: char| c.is_ascii_digit()),
                "la page écrit une valeur de D : « {} »",
                &ecrit[position..(position + 20).min(ecrit.len())]
            );
        }
    }
    let fichiers = textes_des_fichiers(&["html", "htm", "js", "mjs", "css", "svg"]);
    assert!(fichiers.iter().any(|(nom, _)| nom == "index.html"), "index.html n'est pas lu");
    for (nom, texte) in fichiers {
        let texte = etape_5_commun::sans_donnees(&texte);
        for interdit in ["1,71", "1.71"] {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} »");
        }
    }
}

/// Vrai si la balise porte l'attribut `nom` (booléen comme `hidden`, ou avec une valeur).
fn a_l_attribut(balise: &str, nom: &str) -> bool {
    balise
        .split_whitespace()
        .skip(1)
        .any(|mot| mot == nom || mot.starts_with(&format!("{nom}=")) || mot == format!("{nom}/"))
}

/// forme : les deux phrases du repère ne sont pas affichées d'office (V-Z1-39). Dans `index.html`, l'élément `compatible` et l'élément `hors-barre` portent l'attribut `hidden` (aucune des deux n'est visible avant une mesure). Une page qui laisserait « compatible » toujours visible, quel que soit le D mesuré, échoue ici.
/// **V8 : le script n'est plus lu** (`app.js` demandait les deux éléments par `getElementById` et écrivait leur `.hidden` : une écriture, pas une promesse) : « compatible si et seulement si `|D − 1,66| <= 0,03` » ne se prouve que sous un navigateur, voir `MODELE.md`, étape V8.
///
/// Ne couvre pas : la condition elle-même (`|D - 1,66| <= 0,03`) ni l'affichage réel : le DOM n'est pas éprouvé, seul un navigateur le montre.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn les_phrases_du_repere_attendent_la_mesure() {
    let html = lire_page("index.html");
    for id in ["compatible", "hors-barre"] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert!(a_l_attribut(balise, "hidden"), "l'élément « {id} » n'a pas l'attribut hidden : <{balise}>");
    }
}

/// forme : la page appelle la mesure (V-Z1-30). Le code de la page (tous les fichiers JavaScript : `app.js`, un worker, un script en ligne) appelle `givre_mesurer(` au moins une fois, et **toujours sans argument**.
/// **V8 ne lit plus dans quelle fonction** est l'appel : le test d'avant refusait une fonction déclarée qui appelle à la fois `givre_mesurer` et `givre_avancer` ou `requestAnimationFrame` ; or le gestionnaire de messages d'un worker (`function surMessage(e) { switch (e.data.type) { case "avancer": … case "mesurer": … } }`)
/// est exactement cela, et la même chose écrite dans une fonction fléchée passait : le test figeait une écriture sans rien prouver. Que la mesure (en `O(n²)`) ne soit lancée qu'à la demande, jamais à chaque image, ne se voit que sous un navigateur (voir `MODELE.md`, étape V8, « Ce que cette étape n'établit plus »).
///
/// Ne couvre pas : que la mesure soit lancée par le bouton, ni qu'elle ne le soit pas à chaque image.
/// Fiche : V-V8-41 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_appelle_la_mesure_sans_argument() {
    let code = code_de_app_js();
    let arites = arites_des_appels(&code, "givre_mesurer");
    assert!(!arites.is_empty(), "la page n'appelle jamais givre_mesurer()");
    assert!(arites.iter().all(|a| *a == 0), "givre_mesurer est appelée avec des arguments : {arites:?}");
    // Le détecteur voit ce qu'il doit voir (un compte vide n'est pas un compte vert).
    assert_eq!(arites_des_appels("m.givre_mesurer(); x.givre_mesurer(1)", "givre_mesurer"), [0, 1]);
}

/// forme : `lecture.js` déclare `export function lireCorrelation(` et n'emploie ni `document` ni `window` ; **un fichier de la page autre que `lecture.js`** importe `lireCorrelation` (`import { … } from "…/lecture.js"`) et chaque appel de `lireCorrelation(` de la page a un seul argument ; tous les fichiers JavaScript de
/// la page, mis bout à bout, emploient les noms `givre_…` du module avec le bon nombre d'arguments : **jamais un nom inconnu**, et toutes les exportations d'avant l'étape V8 quand la fonctionnalité de leur étape est active (**les deux de V8 sont permises, jamais exigées**) ; **sans** `etape-z1`, ils peuvent encore n'employer que les onze (la page d'avant
/// l'étape), de façon que l'ordre des tâches de la vague du code ne compte pas (V-Z1-32).
///
/// Ne couvre pas : ce que la page fait de la courbe (la dessiner, écrire le résultat) : seul un navigateur le montre ; quel fichier emploie quoi.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_emploie_les_quinze_exportations_et_lire_correlation() {
    let lecture = code_de_lecture_js();
    assert!(
        lecture.contains("export function lireCorrelation("),
        "lecture.js ne déclare pas `export function lireCorrelation(`"
    );
    for interdit in ["document", "window"] {
        assert!(
            !lecture.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).any(|mot| mot == interdit),
            "lecture.js emploie `{interdit}`"
        );
    }
    let code_autre: String = code_js_de_la_page()
        .into_iter()
        .filter(|(nom, _)| nom != "lecture.js" && !nom.ends_with("/lecture.js"))
        .map(|(_, code)| code)
        .collect::<Vec<_>>()
        .join("\n");
    // V8 (juge) : l'import peut être nommé (`import { lireCorrelation } from "./lecture.js"`), d'espace de noms (`import * as lecture from "./lecture.js"`) ou dynamique (`await import("./lecture.js")`) : un fichier hors `lecture.js` importe de `lecture.js`, et `lireCorrelation(` y est appelée.
    let importe = ["lecture.js\"", "lecture.js'", "lecture.js`"].iter().any(|fin| {
        code_autre.match_indices(fin).any(|(debut, _)| {
            let avant = &code_autre[..debut];
            avant.rfind(['"', '\'', '`']).is_some_and(|guillemet| {
                let chemin = &avant[guillemet + 1..];
                let devant = code_autre[..guillemet].trim_end();
                !etape_5_commun::adresse_externe(chemin) && (devant.ends_with("from") || devant.ends_with("import("))
            })
        })
    });
    assert!(importe, "aucun fichier de la page, hors lecture.js, n'importe de lecture.js");
    assert!(
        !arites_des_appels(&code_autre, "lireCorrelation").is_empty(),
        "aucun fichier de la page, hors lecture.js, n'appelle lireCorrelation(…)"
    );
    let code = code_de_app_js();
    let arites = arites_des_appels(&code, "lireCorrelation");
    assert!(arites.iter().all(|a| *a == 1), "la page appelle lireCorrelation avec {arites:?} argument(s) au lieu de 1");

    let base: BTreeSet<String> = FONCTIONS.iter().map(|(nom, _, _)| nom.to_string()).collect();
    let etendu: BTreeSet<String> =
        base.iter().cloned().chain(FONCTIONS_Z1.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape D3 : la page emploie en plus les huit fonctions de la courbe. Avec `etape-d3` les vingt-trois seulement ; sans elle, ce que `etape-z1` dit
    // (les quinze, ou les onze sans `etape-z1`) ou les vingt-trois.
    let etendu_d3: BTreeSet<String> =
        etendu.iter().cloned().chain(FONCTIONS_D3.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape V7 : la page emploie en plus les quatre fonctions de l'image nette. Avec `etape-v7` les vingt-sept seulement ; sans elle, ce que les fonctionnalités d'avant disent ou les vingt-sept.
    let etendu_v7: BTreeSet<String> =
        etendu_d3.iter().cloned().chain(FONCTIONS_V7.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape G3 : la page emploie en plus les cinq fonctions de la neige. Avec `etape-g3` les trente-deux seulement ; sans elle, ce que les fonctionnalités d'avant disent ou les trente-deux.
    let etendu_g3: BTreeSet<String> =
        etendu_v7.iter().cloned().chain(FONCTIONS_G3.iter().map(|(nom, _, _)| nom.to_string())).collect();
    // Étape V8 : les deux fonctions des masses sont permises (jamais exigées) : on les retire de ce que la page emploie avant de comparer.
    let masses: BTreeSet<String> = FONCTIONS_V8.iter().map(|(nom, _, _)| nom.to_string()).collect();
    let employes_avec_v8 = noms_employes(&format!("{code}\n{lecture}"));
    let permis: BTreeSet<String> = etendu_g3.union(&masses).cloned().collect();
    let inconnus: Vec<&String> = employes_avec_v8.difference(&permis).collect();
    assert!(inconnus.is_empty(), "la page emploie des noms que le module n'exporte pas : {inconnus:?}");
    let employes: BTreeSet<String> = employes_avec_v8.difference(&masses).cloned().collect();
    if G3_ACTIVE {
        assert_eq!(employes, etendu_g3, "la page n'emploie pas exactement les trente-deux exportations d'avant V8");
    } else if V7_ACTIVE {
        assert!(
            employes == etendu_v7 || employes == etendu_g3,
            "la page n'emploie ni les vingt-sept ni les trente-deux exportations : {employes:?}"
        );
    } else if D3_ACTIVE {
        assert!(
            employes == etendu_d3 || employes == etendu_v7 || employes == etendu_g3,
            "la page n'emploie ni les vingt-trois ni les vingt-sept ni les trente-deux exportations : {employes:?}"
        );
    } else if Z1_ACTIVE {
        assert!(
            employes == etendu || employes == etendu_d3 || employes == etendu_v7 || employes == etendu_g3,
            "la page n'emploie ni les quinze ni les vingt-trois ni les vingt-sept ni les trente-deux exportations : {employes:?}"
        );
    } else {
        assert!(
            employes == base
                || employes == etendu
                || employes == etendu_d3
                || employes == etendu_v7
                || employes == etendu_g3,
            "la page n'emploie ni les onze ni les quinze ni les vingt-trois ni les vingt-sept ni les trente-deux exportations : {employes:?}"
        );
    }
    let code = format!("{code}\n{lecture}");
    for (nom, parametres, _) in FONCTIONS
        .iter()
        .chain(FONCTIONS_Z1.iter())
        .chain(FONCTIONS_D3.iter())
        .chain(FONCTIONS_V7.iter())
        .chain(FONCTIONS_G3.iter())
        .chain(FONCTIONS_V8.iter())
    {
        for arite in arites_des_appels(&code, nom) {
            assert_eq!(
                arite,
                parametres.len(),
                "la page appelle {nom} avec {arite} argument(s) au lieu de {}",
                parametres.len()
            );
        }
    }
    for (nom, _, _) in FONCTIONS_Z1 {
        assert!(!arites_des_appels(&code, nom).is_empty() || !Z1_ACTIVE, "la page n'appelle jamais {nom}(…)");
    }
    for (nom, _, _) in FONCTIONS_D3 {
        assert!(!arites_des_appels(&code, nom).is_empty() || !D3_ACTIVE, "la page n'appelle jamais {nom}(…)");
    }
    for (nom, _, _) in FONCTIONS_V7 {
        assert!(!arites_des_appels(&code, nom).is_empty() || !V7_ACTIVE, "la page n'appelle jamais {nom}(…)");
    }
    for (nom, _, _) in FONCTIONS_G3 {
        assert!(!arites_des_appels(&code, nom).is_empty() || !G3_ACTIVE, "la page n'appelle jamais {nom}(…)");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

const SCRIPT: &str = "correlation-wasm.mjs";

/// Lance `tests/etape_z1/correlation-wasm.mjs` avec ces arguments.
fn lancer_le_script(arguments: &[&str]) -> std::process::Output {
    Command::new("node")
        .arg(racine().join("tests").join("etape_z1").join(SCRIPT))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

/// Les 8 octets d'un `f64`, petit-boutiste, en hexadécimal, ou `NaN`.
fn hexa(x: f64) -> String {
    if x.is_nan() { "NaN".to_string() } else { x.to_le_bytes().iter().map(|o| format!("{o:02x}")).collect() }
}

/// Ce que `correlation-wasm.mjs` doit écrire : l'état et la mesure du code natif, lus comme le script les lit (voir son en-tête).
fn attendu_du_script(modele: u32, graine: u32, pas: u32) -> String {
    let duree = TABLE[modele as usize].duree;
    let joue = pas.min(duree);
    let (sites, mesure_native) = if modele == 0 {
        let mut amas = Amas::new(u64::from(graine), 200);
        amas.croitre_jusqu_a(1 + joue as usize);
        (amas.sites().len() as u32, mesure(amas.sites()))
    } else {
        (etape_5_commun::natif(modele, graine, joue).sites, None)
    };
    let (points, rayons, valeurs, dimension) = match &mesure_native {
        Some(m) => (
            m.rayons.len(),
            m.rayons.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
            m.valeurs.iter().map(|v| hexa(*v)).collect::<Vec<_>>().join(","),
            hexa(m.dimension),
        ),
        None => (0, String::new(), String::new(), "NaN".to_string()),
    };
    // Un `givre_avancer(1)` de plus : il joue une unité (et efface la mesure) sauf si le scénario est fini (il ne joue rien, la mesure reste).
    let apres = if joue < duree {
        "1,0,NaN".to_string()
    } else {
        match &mesure_native {
            Some(m) => format!("0,{},{}", m.rayons[0], hexa(m.dimension)),
            None => "0,0,NaN".to_string(),
        }
    };
    format!(
        "avant=1\njoue={joue}\npoints={points}\nsites={sites}\nrayons={rayons}\nvaleurs={valeurs}\ndimension={dimension}\nfidele=1\n\
         hors=0,0,0;NaN,NaN,NaN\napres={apres}\n"
    )
}

/// juste : le module commité, lu par `lecture.js` (`lireCorrelation`) sous Node, donne la courbe du code natif (V-Z1-33). Pour chaque cas, le script
/// démarre le modèle, joue `pas` unités, appelle `givre_mesurer` une fois et écrit ce que `lireCorrelation` lit ; ce test compare cette sortie, ligne
/// par ligne et en entier, à celle du code natif (`Amas::new(graine, 200)` puis `correlation::mesure`) : le nombre de points, les rayons, chaque
/// `C(r)` ET la dimension au bit près (8 octets), le contrat « avant tout démarrage » (lu sur un module neuf : `givre_mesurer` rend 0 et le reste vaut 0
/// ou `NaN`), que `lireCorrelation` redit exactement les accesseurs, les valeurs hors de la courbe (rayon 0, valeur `NaN`) et ce que devient la mesure
/// après un `givre_avancer(1)` de plus (effacée, sauf quand le scénario est fini). Les cas : la DLA de graine 1 jusqu'au bout (2999), une grande graine à
/// 1500, la graine 7 à 800, `u32::MAX` à 2000, la graine 2 à 2998 (un `givre_avancer` de plus joue la dernière particule), une DLA trop petite
/// (0 pas, un site ; 200 pas, environ 200 sites), la plaque, l'étoile et la dendrite au bout (pas de mesure : seul le modèle 0 se mesure), et la
/// dendrite avec un nombre de pas qui dépasse la durée. **Ne passe qu'une fois `docs/givre.wasm` reconstruit** avec les quatre fonctions.
///
/// Ne couvre pas : ce qu'`app.js` fait de ce que rend `lireCorrelation` (la dessiner, l'écrire) : seul un navigateur le montre ; la vitesse du
/// module WebAssembly.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn lecture_js_lit_la_courbe_comme_le_natif() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");
    let lecture = chemin_de_lecture();
    let lecture = lecture.to_str().expect("chemin en UTF-8");
    let cas: [(u32, u32, u32); 11] = [
        (0, 1, 2999),
        (0, 0xDEAD_BEEF, 1500),
        (0, 7, 800),
        (0, u32::MAX, 2000),
        (0, 2, 2998),
        (0, 4, 0),
        (0, 5, 200),
        (1, 1, 400),
        (2, 0, 120),
        (3, u32::MAX, 800),
        (3, 1, 10_000),
    ];
    for (modele, graine, pas) in cas {
        let arguments = [module, lecture, &modele.to_string(), &graine.to_string(), &pas.to_string()];
        let sortie = lancer_le_script(&arguments);
        let erreur = String::from_utf8_lossy(&sortie.stderr);
        assert!(sortie.status.success(), "{SCRIPT} {arguments:?} : {:?}, sortie d'erreur : {erreur}", sortie.status);
        assert_eq!(
            String::from_utf8_lossy(&sortie.stdout),
            attendu_du_script(modele, graine, pas),
            "modèle {modele}, graine {graine}, {pas} pas"
        );
    }
}

/// forme : le script refuse des arguments faux avec le code de sortie 1 et un message sur la sortie d'erreur, sans rien écrire sur la sortie standard :
/// fichier absent, fichier qui n'est pas un module, `lecture.js` absent, trop ou trop peu d'arguments, modèle qui n'est pas un entier de 0 à 3, graine ou
/// nombre de pas qui n'est pas un entier de 32 bits non signé (V-Z1-34).
///
/// Ne couvre pas : les arguments justes (voir `lecture_js_lit_la_courbe_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn le_script_refuse_des_arguments_faux() {
    if !node_est_la() {
        return;
    }
    let module = chemin_module();
    let module = module.to_str().expect("chemin en UTF-8");
    let lecture = chemin_de_lecture();
    let lecture = lecture.to_str().expect("chemin en UTF-8");
    let absent = racine().join("docs").join("n-existe-pas.wasm");
    let absent = absent.to_str().expect("chemin en UTF-8");
    let pas_un_module = racine().join("Cargo.toml");
    let pas_un_module = pas_un_module.to_str().expect("chemin en UTF-8");
    let cas: [&[&str]; 14] = [
        &[absent, lecture, "0", "1", "80"],
        &[pas_un_module, lecture, "0", "1", "80"],
        &[module, "absent.js", "0", "1", "80"],
        &[],
        &[module, lecture],
        &[module, lecture, "0", "1"],
        &[module, lecture, "0", "1", "80", "en trop"],
        &[module, lecture, "4", "1", "80"],
        &[module, lecture, "abc", "1", "80"],
        &[module, lecture, "0", "-1", "80"],
        &[module, lecture, "0", "4294967296", "80"],
        &[module, lecture, "0", "1", "abc"],
        &[module, lecture, "0", "1", "-1"],
        &[module, lecture, "0", "1", ""],
    ];
    for arguments in cas {
        let sortie = lancer_le_script(arguments);
        assert_eq!(sortie.status.code(), Some(1), "{SCRIPT}, arguments {arguments:?}");
        assert!(sortie.stdout.is_empty(), "{SCRIPT}, arguments {arguments:?} : la sortie standard n'est pas vide");
        assert!(!sortie.stderr.is_empty(), "{SCRIPT}, arguments {arguments:?} : aucun message d'erreur");
    }
}

/// forme : le script de Node est du JavaScript valide (modules) : `node --check`, le texte lui étant passé sur l'entrée standard. Le contrôle est
/// sauté, comme les autres, quand Node est absent hors de la CI.
///
/// Ne couvre pas : l'exécution du script (voir `lecture_js_lit_la_courbe_comme_le_natif`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn le_script_est_du_javascript_valide() {
    if !node_est_la() {
        return;
    }
    let script =
        std::fs::read_to_string(racine().join("tests").join("etape_z1").join(SCRIPT)).expect("le script se lit");
    let mut node = Command::new("node")
        .args(["--input-type=module", "--check"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node se lance");
    node.stdin.take().expect("entrée standard de Node").write_all(script.as_bytes()).expect("écriture du script");
    let sortie = node.wait_with_output().expect("Node se termine");
    assert!(sortie.status.success(), "{SCRIPT} n'est pas valide : {}", String::from_utf8_lossy(&sortie.stderr));
}

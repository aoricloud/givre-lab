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

use etape_5_commun::{FONCTIONS, FONCTIONS_Z1, TABLE, Z1_ACTIVE, chemin_module, lire_page, node_est_la, racine};
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

fn code_de_app_js() -> String {
    sans_commentaires(&lire_page("app.js"))
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

fn nom_de_balise(balise: &str) -> String {
    balise.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// La première balise du HTML qui a cet `id`.
fn balise_d_id<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    balises(html).into_iter().find(|balise| attribut(balise, "id").as_deref() == Some(id))
}

/// Le texte qui suit la balise d'`id` donné, jusqu'au `<` suivant, sans espaces de début et de fin.
fn texte_de(html: &str, id: &str) -> Option<String> {
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>')?;
        if attribut(balise, "id").as_deref() == Some(id) {
            return Some(texte.trim().to_string());
        }
    }
    None
}

/// Le texte visible de la page : tout ce qui n'est pas dans une balise, ni dans `<style>` ni dans `<script>`.
fn texte_visible(html: &str) -> String {
    let mut sortie = String::new();
    let mut ignore = false;
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>').unwrap_or((morceau, ""));
        let ferme = balise.starts_with('/');
        match nom_de_balise(balise.trim_start_matches('/')).as_str() {
            "style" | "script" => ignore = !ferme,
            _ => {}
        }
        if !ignore {
            sortie.push_str(texte);
            sortie.push(' ');
        }
    }
    sortie
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

/// forme : les phrases (V-Z1-28). `repere-zinc` dit `zinc`, `Matsushita`, `1984`, `1,66` et `0,03` ; `compatible` est exactement « compatible, ce
/// qui ne prouve pas que le zinc suit la DLA » ; `taille-zinc` est exactement « taille d'amas du zinc non relevée » (la taille d'amas des feuilles de
/// zinc n'a pas pu être lue, et la page ne l'invente pas) ; et le mot « zinc » ne figure dans la page qu'avec ces mots de repère, jamais comme un test
/// (aucune phrase « dans la barre » n'est écrite à la place de `compatible`).
///
/// Ne couvre pas : l'affichage de l'une ou l'autre phrase selon la mesure (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_dit_le_repere_du_zinc_comme_un_repere() {
    let html = lire_page("index.html");
    let repere = texte_de(&html, "repere-zinc").expect("repere-zinc");
    for mot in ["zinc", "Matsushita", "1984", "1,66", "0,03"] {
        assert!(repere.contains(mot), "le repère du zinc « {repere} » ne dit pas « {mot} »");
    }
    assert_eq!(
        texte_de(&html, "compatible").expect("compatible"),
        "compatible, ce qui ne prouve pas que le zinc suit la DLA"
    );
    assert_eq!(texte_de(&html, "taille-zinc").expect("taille-zinc"), "taille d'amas du zinc non relevée");
}

/// forme : aucune valeur de D n'est écrite dans la page (V-Z1-29). `index.html` (son texte visible) n'a aucun « D = » ni « D= » suivi d'un chiffre ;
/// `app.js` contient « mesuré par le banc : D = » suivi de `${` (une valeur calculée) et plus loin « sites » ; ni `index.html`, ni `app.js`, ni
/// `lecture.js` ne contient « 1,71 » ni « 1.71 ».
///
/// Ne couvre pas : le texte réellement affiché (le DOM n'est pas éprouvé) ni qu'un nombre mesuré ne soit pas copié dans une constante autrement écrite.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn la_page_ne_fige_aucune_valeur_de_d() {
    let visible = texte_visible(&lire_page("index.html"));
    for motif in ["D =", "D="] {
        for (position, _) in visible.match_indices(motif) {
            let suite = visible[position + motif.len()..].trim_start();
            assert!(
                !suite.starts_with(|c: char| c.is_ascii_digit()),
                "index.html écrit une valeur de D : « {} »",
                &visible[position..(position + 20).min(visible.len())]
            );
        }
    }
    let code = code_de_app_js();
    let debut = code.find("mesuré par le banc : D = ").expect("app.js ne contient pas « mesuré par le banc : D = »");
    let suite = &code[debut + "mesuré par le banc : D = ".len()..];
    assert!(
        suite.starts_with("${"),
        "après « mesuré par le banc : D = », app.js n'écrit pas une valeur calculée : {}",
        &suite[..suite.len().min(20)]
    );
    assert!(suite[..suite.len().min(200)].contains(" sites"), "app.js ne dit pas « à N sites »");
    for nom in ["index.html", "app.js", "lecture.js"] {
        let texte = lire_page(nom);
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

/// forme : les deux phrases du repère ne sont pas affichées d'office (V-Z1-39). Dans `index.html`, l'élément `compatible` et l'élément `hors-barre`
/// portent l'attribut `hidden` (aucune des deux n'est visible avant une mesure), et `app.js` demande les deux éléments par
/// `getElementById("compatible")` et `getElementById("hors-barre")` (guillemets doubles ou simples) et écrit leur `.hidden`. Une page qui
/// laisserait « compatible » toujours visible, quel que soit le D mesuré, échoue ici.
///
/// Ne couvre pas : la condition elle-même (`|D - 1,66| <= 0,03`, voir l'en-tête) ni l'affichage réel : le DOM n'est pas éprouvé, seul un navigateur le
/// montre.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn les_phrases_du_repere_attendent_la_mesure() {
    let html = lire_page("index.html");
    for id in ["compatible", "hors-barre"] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert!(a_l_attribut(balise, "hidden"), "l'élément « {id} » n'a pas l'attribut hidden : <{balise}>");
    }
    let code = code_de_app_js();
    for id in ["compatible", "hors-barre"] {
        assert!(
            code.contains(&format!("getElementById(\"{id}\")")) || code.contains(&format!("getElementById('{id}')")),
            "app.js ne demande pas l'élément « {id} »"
        );
    }
    assert!(code.contains(".hidden"), "app.js n'écrit jamais `.hidden`");
}

/// L'indice du caractère `}` qui ferme l'accolade ouvrante à `ouvrante` dans `code` (les chaînes de caractères sont sautées).
fn fermeture(code: &str, ouvrante: usize) -> usize {
    let c: Vec<(usize, char)> = code.char_indices().collect();
    let depart = c.iter().position(|&(i, _)| i == ouvrante).expect("accolade ouvrante");
    let (mut profondeur, mut guillemet, mut echappe) = (0_i32, None, false);
    for &(i, ch) in &c[depart..] {
        if let Some(g) = guillemet {
            if echappe {
                echappe = false;
            } else if ch == '\\' {
                echappe = true;
            } else if ch == g {
                guillemet = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => guillemet = Some(ch),
            '{' => profondeur += 1,
            '}' => {
                profondeur -= 1;
                if profondeur == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    panic!("accolade non fermée");
}

/// La fonction déclarée (`function nom(…) { … }`) qui contient la position `position` de `code` : son nom et son corps.
fn fonction_qui_contient(code: &str, position: usize) -> Option<(String, String)> {
    let mut candidate = None;
    for (debut, _) in code.match_indices("function ") {
        if debut > position {
            break;
        }
        let nom: String =
            code[debut + 9..].chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
        let ouvrante = debut + code[debut..].find('{')?;
        let fin = fermeture(code, ouvrante);
        if fin > position {
            candidate = Some((nom, code[ouvrante..=fin].to_string()));
        }
    }
    candidate
}

/// forme : la mesure se fait à la demande (V-Z1-30). `app.js` appelle `givre_mesurer(` une seule fois ; l'appel est dans une fonction déclarée
/// `function nom(…) { … }` qui n'appelle ni `requestAnimationFrame` ni `givre_avancer` (donc pas dans l'image de l'animation) ; `nom` est le
/// gestionnaire du clic du bouton `mesurer` : `app.js` demande l'élément `getElementById("mesurer")` et contient `addEventListener("click", nom)`
/// (guillemets doubles ou simples).
///
/// Ne couvre pas : que le gestionnaire soit bien relié au bon bouton (le DOM n'est pas éprouvé), ce que la fonction fait du résultat.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn app_js_mesure_a_la_demande() {
    let code = code_de_app_js();
    let appels: Vec<usize> = code.match_indices("givre_mesurer(").map(|(i, _)| i).collect();
    assert_eq!(
        appels.len(),
        1,
        "app.js doit appeler givre_mesurer() une seule fois, il l'appelle {} fois",
        appels.len()
    );
    let (nom, corps) = fonction_qui_contient(&code, appels[0])
        .expect("l'appel de givre_mesurer est dans une `function nom(…)` déclarée");
    for interdit in ["requestAnimationFrame", "givre_avancer"] {
        assert!(!corps.contains(interdit), "la fonction « {nom} » qui mesure appelle {interdit}");
    }
    assert!(
        code.contains("getElementById(\"mesurer\")") || code.contains("getElementById('mesurer')"),
        "app.js ne demande pas le bouton « mesurer »"
    );
    let gestionnaire = ["\"", "'"].iter().any(|g| code.contains(&format!("addEventListener({g}click{g}, {nom})")));
    assert!(gestionnaire, "« {nom} » n'est pas installée par addEventListener(\"click\", {nom})");
}

/// forme : `lecture.js` déclare `export function lireCorrelation(` et n'emploie ni `document` ni `window` ; `app.js` l'importe de `"./lecture.js"`
/// et l'appelle avec un argument ; à eux deux `app.js` et `lecture.js` emploient exactement les quinze noms `givre_…` du module (les onze de l'étape 5
/// et les quatre de l'étape Z1) quand la fonctionnalité `etape-z1` est active ; **sans elle**, ils peuvent encore n'employer que les onze (la page
/// d'avant l'étape), de façon que l'ordre des tâches de la vague du code ne compte pas. Chaque nom employé est appelé avec le bon nombre
/// d'arguments (V-Z1-32).
///
/// Ne couvre pas : ce que `app.js` fait de la courbe (la dessiner, écrire le résultat) : seul un navigateur le montre.
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
    let app = code_de_app_js();
    let importe = ["\"./lecture.js\"", "'./lecture.js'"]
        .iter()
        .any(|nom| app.match_indices(nom).any(|(debut, _)| app[..debut].trim_end().ends_with("from")));
    assert!(importe, "app.js n'importe rien de \"./lecture.js\"");
    let arites = arites_des_appels(&app, "lireCorrelation");
    assert!(
        !arites.is_empty() && arites.iter().all(|a| *a == 1),
        "app.js appelle lireCorrelation avec {arites:?} argument(s) au lieu de 1"
    );

    let code = format!("{app}\n{lecture}");
    let base: BTreeSet<String> = FONCTIONS.iter().map(|(nom, _, _)| nom.to_string()).collect();
    let etendu: BTreeSet<String> =
        base.iter().cloned().chain(FONCTIONS_Z1.iter().map(|(nom, _, _)| nom.to_string())).collect();
    let employes = noms_employes(&code);
    if Z1_ACTIVE {
        assert_eq!(employes, etendu, "la page n'emploie pas exactement les quinze exportations");
    } else {
        assert!(
            employes == base || employes == etendu,
            "la page n'emploie ni les onze ni les quinze exportations : {employes:?}"
        );
    }
    for (nom, parametres, _) in FONCTIONS.iter().chain(FONCTIONS_Z1.iter()) {
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
    let lecture = racine().join("docs").join("lecture.js");
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
    let lecture = racine().join("docs").join("lecture.js");
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

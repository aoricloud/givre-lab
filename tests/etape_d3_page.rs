//! Étape D3 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) du claquage diélectrique et de la courbe `D(η)`. Valeurs attendues : `references/etape-d3-valeurs.md`,
//! identifiants `V-D3-xx`. Comme aux étapes 5 et Z1, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_pure_dit_ce_que_la_doc_dit` appelle les fonctions de `lecture.js` qui n'ont pas besoin du module (le numéro de modèle, les textes d'avancement et de repère) et compare à un calcul fait à
//! la main ; il ne lit pas `docs/givre.wasm` et passe dès que `lecture.js` est écrit. La lecture du module par `lecture.js` (`lireCourbe`) est éprouvée dans `etape_d3_module.rs`.
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : le prototype de l'écrivain fait foi, ses fichiers se recopient à l'octet)
//!
//! - `index.html` ajoute, en gardant tout ce que les étapes 5 et Z1 demandent : une cinquième `<option value="claquage">` « Claquage diélectrique » dans `#modele` ; un curseur `<input id="eta"
//!   type="range" min="0" max="3" step="1">` (l'exposant `η`, un entier de 0 à 3) avec son `<output id="valeur-eta">`, dans un conteneur `id="commande-eta"` **caché au départ** (`hidden`) ; un
//!   bloc `id="bloc-zinc"` qui contient `barre-zinc`, `marque-d`, `repere-zinc`, `compatible`, `hors-barre` et `taille-zinc` (les phrases du zinc n'ont de sens que pour la DLA) ; un bloc
//!   `id="bloc-repere-eta"` caché au départ, avec `repere-eta` (le repère publié de `η`), `ecart-eta` (caché) et `pas-un-accord` (caché, dit « ce n'est pas un accord ») ; une section « Courbe D(η) » avec
//!   un `<button id="calculer-courbe">`, un `id="avancement-courbe"`, un `<canvas id="courbe-eta">`, un `id="legende-courbe"` (les points bleus du banc, les carrés rouges des repères publiés : le rappel de
//!   la DLA et Hastings 2001), un `id="limite-courbe"` (la limite dite en clair : des amas petits, peu de graines, une méthode et une fenêtre qui diffèrent de celles de Hastings) et un
//!   `id="pas-un-accord-courbe"` (« Ce n'est pas un accord »).
//! - `lecture.js` exporte de plus (sans DOM) `lireCourbe(m)`, `numeroDuModele(choix, eta)`, `blocsCaches(choix)`, `texteAvancement(courbe)`, `texteRepereEta(eta, repere, dimension)` et `formater(valeur)`.
//! - `app.js` : demande tous ces éléments par `getElementById("…")` ; le choix « claquage » montre `commande-eta` et `bloc-repere-eta`, cache `bloc-zinc` (écrit leur `.hidden`) ; le numéro de modèle est
//!   `numeroDuModele(choix, η)` (4 + η pour le claquage) ; les modèles 4 à 7 se jouent **unité par unité** (`givre_avancer(1)`) en s'arrêtant quand une courte durée est écoulée
//!   (`BUDGET_IMAGE_MS`, `performance.now()`) ; « Mesurer D » écrit le repère de `η` lu dans le module (`givre_courbe_repere`), l'écart et la phrase « ce n'est pas un accord » ; le bouton
//!   `calculer-courbe` démarre `givre_courbe_demarrer(COURBE_TAILLE, COURBE_GRAINES)` (au plus 100 sites, de 2 à 8 graines) et fait **un** `givre_courbe_pas()` par tour, avec un `setTimeout` entre deux
//!   tours (une fonction déclarée, jamais une boucle) ; le texte d'avancement vient de `texteAvancement`.
//! - **Aucune valeur publiée ni aucune valeur de `D` n'est écrite dans les fichiers de la page** : les repères viennent du module.
//!
//! Étiquettes : `lecture_pure_dit_ce_que_la_doc_dit` est **juste** (comparé à un calcul à la main) ; tous les autres sont de la **forme** (régression de texte), jamais une preuve du comportement de la page.

mod etape_d3_commun;

use etape_d3_commun::{e5, lancer_le_script, racine};

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
    sans_commentaires(&e5::lire_page("app.js"))
}

fn code_de_lecture_js() -> String {
    sans_commentaires(&e5::lire_page("lecture.js"))
}

/// Pour chaque appel `nom(…)` du texte, le nombre d'arguments (les virgules de premier niveau plus un, une virgule finale ne comptant pas, ou 0 si les parenthèses sont vides). Un `nom`
/// précédé d'une lettre, d'un chiffre ou d'un `_` est un autre nom et n'est pas compté.
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

/// Vrai si la balise porte l'attribut `nom` (booléen comme `hidden`, ou avec une valeur).
fn a_l_attribut(balise: &str, nom: &str) -> bool {
    balise
        .split_whitespace()
        .skip(1)
        .any(|mot| mot == nom || mot.starts_with(&format!("{nom}=")) || mot == format!("{nom}/"))
}

/// Les `id` des balises contenues dans le `<div>` d'`id` `conteneur` (à n'importe quelle profondeur), dans l'ordre du document.
fn ids_dans_le_div(html: &str, conteneur: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut profondeur = 0_i32;
    for balise in balises(html) {
        let ferme = balise.starts_with('/');
        let nom = nom_de_balise(balise.trim_start_matches('/'));
        if profondeur == 0 {
            if !ferme && nom == "div" && attribut(balise, "id").as_deref() == Some(conteneur) {
                profondeur = 1;
            }
            continue;
        }
        if nom == "div" {
            profondeur += if ferme { -1 } else { 1 };
        }
        if profondeur > 0
            && let Some(id) = attribut(balise, "id")
        {
            ids.push(id);
        }
    }
    ids
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

/// Le nom de la variable à laquelle `app.js` affecte `document.getElementById("id")` (`const nom = document.getElementById("id")`).
fn variable_de(code: &str, id: &str) -> Option<String> {
    let appel = format!("getElementById(\"{id}\")");
    let position = code.find(&appel)?;
    let avant = code[..position].trim_end().strip_suffix("document.")?.trim_end().strip_suffix('=')?.trim_end();
    let nom: String = avant.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
    Some(nom.chars().rev().collect())
}

/// La valeur entière de `const NOM = <chiffres>` dans le code.
fn constante_entiere(code: &str, nom: &str) -> Option<u64> {
    for (debut, _) in code.match_indices(nom) {
        let apres = code[debut + nom.len()..].trim_start();
        let Some(valeur) = apres.strip_prefix('=').filter(|reste| !reste.starts_with('=')) else { continue };
        if !code[..debut].trim_end().ends_with("const") {
            continue;
        }
        let chiffres: String = valeur.trim_start().chars().take_while(char::is_ascii_digit).collect();
        if let Ok(n) = chiffres.parse() {
            return Some(n);
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------------------------
// Les tests de forme.
// ---------------------------------------------------------------------------------------------------------------

/// forme : `index.html` a les éléments du claquage et de la courbe, chacun avec son `id` et sa sorte (V-D3-27) : `<input id="eta" type="range" min="0" max="3" step="1">` (un curseur entier de 0 à 3) et
/// son `<output id="valeur-eta">` ; le conteneur `commande-eta` est **caché au départ** ; la cinquième option du choix du modèle a la valeur `claquage` et son texte dit « claquage » et
/// « diélectrique » ; `<button id="calculer-courbe">`, `<canvas id="courbe-eta">`, et des éléments `avancement-courbe`, `legende-courbe`, `limite-courbe`, `pas-un-accord-courbe`, `bloc-zinc`, `bloc-repere-eta`,
/// `repere-eta`, `ecart-eta`, `pas-un-accord`.
///
/// Ne couvre pas : que ces éléments soient visibles, lisibles ou reliés à quoi que ce soit (voir les tests suivants) ; l'ordre des quatre premières options (voir `etape_5_page.rs`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_a_les_elements_du_claquage_et_de_la_courbe() {
    let html = e5::lire_page("index.html");
    for (id, sorte) in [
        ("eta", "input"),
        ("valeur-eta", "output"),
        ("calculer-courbe", "button"),
        ("courbe-eta", "canvas"),
        ("modele", "select"),
    ] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    for id in [
        "commande-eta",
        "avancement-courbe",
        "legende-courbe",
        "limite-courbe",
        "pas-un-accord-courbe",
        "bloc-zinc",
        "bloc-repere-eta",
        "repere-eta",
        "ecart-eta",
        "pas-un-accord",
    ] {
        assert!(balise_d_id(&html, id).is_some(), "pas d'élément d'id « {id} »");
    }
    let eta = balise_d_id(&html, "eta").expect("eta");
    assert_eq!(attribut(eta, "type").as_deref(), Some("range"), "le curseur η n'est pas de type range");
    assert_eq!(attribut(eta, "min").as_deref(), Some("0"));
    assert_eq!(attribut(eta, "max").as_deref(), Some("3"));
    assert_eq!(attribut(eta, "step").as_deref(), Some("1"), "η est un entier");
    let valeur_initiale = attribut(eta, "value").expect("une valeur initiale");
    assert!(matches!(valeur_initiale.as_str(), "0" | "1" | "2" | "3"), "la valeur initiale de η : {valeur_initiale}");
    assert_eq!(
        texte_de(&html, "valeur-eta").as_deref(),
        Some(valeur_initiale.as_str()),
        "la sortie du curseur dit la valeur initiale"
    );
    let commande = balise_d_id(&html, "commande-eta").expect("commande-eta");
    assert!(a_l_attribut(commande, "hidden"), "commande-eta n'est pas cachée au départ : <{commande}>");

    // La dernière option du choix du modèle.
    let mut options: Vec<(String, String)> = Vec::new();
    let mut dans_le_select = false;
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>').unwrap_or((morceau, ""));
        let nom = nom_de_balise(balise);
        if nom == "select" && attribut(balise, "id").as_deref() == Some("modele") {
            dans_le_select = true;
        } else if balise.starts_with("/select") {
            dans_le_select = false;
        } else if dans_le_select && nom == "option" {
            options.push((attribut(balise, "value").unwrap_or_default(), texte.trim().to_lowercase()));
        }
    }
    let (valeur, texte) = options.last().expect("des options");
    assert_eq!(valeur, "claquage", "la dernière option du choix du modèle");
    assert!(texte.contains("claquage") && texte.contains("diélectrique"), "le texte de l'option : « {texte} »");
    assert_eq!(options.len(), 5);
    let bouton = texte_de(&html, "calculer-courbe").expect("le bouton");
    assert!(bouton.contains("courbe"), "le texte du bouton est « {bouton} »");
}

/// forme : les phrases (V-D3-28). `pas-un-accord` et `pas-un-accord-courbe` disent « ce n'est pas un accord » (sans tenir compte de la casse) ; `limite-courbe` dit la limite en clair (les mots `amas`,
/// `graines`, `méthode` ou `corrélation`, `fenêtre`, `Hastings`) ; `legende-courbe` nomme les deux sortes de repère (`Hastings 2001` et `rappel de la DLA`) ; `bloc-repere-eta`, `ecart-eta` et
/// `pas-un-accord` sont **cachés au départ** (rien n'est dit avant une mesure) ; `repere-eta` est vide au départ (le repère vient du module). Le mot « accord » ne figure jamais sans négation dans la page
/// (« en accord », « d'accord », « un accord » hors de « pas un accord » : refusés).
///
/// Ne couvre pas : l'affichage de l'une ou l'autre phrase selon la mesure (le DOM n'est pas éprouvé) ; la justesse de la limite dite.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_dit_pas_un_accord_et_la_limite() {
    let html = e5::lire_page("index.html");
    for id in ["pas-un-accord", "pas-un-accord-courbe"] {
        let texte = texte_de(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »")).to_lowercase();
        assert!(texte.contains("ce n'est pas un accord"), "« {id} » dit : « {texte} »");
    }
    let limite = texte_de(&html, "limite-courbe").expect("limite-courbe").to_lowercase();
    for mot in ["amas", "graines", "fenêtre", "hastings"] {
        assert!(limite.contains(mot), "la limite dite « {limite} » ne parle pas de « {mot} »");
    }
    assert!(
        limite.contains("méthode") || limite.contains("corrélation"),
        "la limite ne dit pas la méthode : « {limite} »"
    );
    let legende = texte_de(&html, "legende-courbe").expect("legende-courbe");
    for mot in ["Hastings 2001", "rappel de la DLA"] {
        assert!(legende.contains(mot), "la légende « {legende} » ne dit pas « {mot} »");
    }
    for id in ["bloc-repere-eta", "ecart-eta", "pas-un-accord"] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert!(a_l_attribut(balise, "hidden"), "l'élément « {id} » n'est pas caché au départ : <{balise}>");
    }
    assert_eq!(
        texte_de(&html, "repere-eta").as_deref(),
        Some(""),
        "le repère de η est écrit par la page, depuis le module"
    );
    // « accord » toujours nié : chaque occurrence est précédée de « pas un ».
    let visible = texte_visible(&html).to_lowercase();
    for (position, _) in visible.match_indices("accord") {
        assert!(
            visible[..position].trim_end().ends_with("pas un"),
            "« accord » sans « pas un » : …{}",
            &visible[position.saturating_sub(30)..(position + 10).min(visible.len())]
        );
    }
}

/// forme : les phrases du zinc ne sont que pour la DLA (V-D3-29). Dans `index.html`, le conteneur `bloc-zinc` contient `barre-zinc`, `marque-d`, `repere-zinc`, `compatible`, `hors-barre` et `taille-zinc`, et
/// ne contient ni `bloc-repere-eta`, ni `repere-eta`, ni `pas-un-accord` ; le conteneur `bloc-repere-eta` contient `repere-eta`, `ecart-eta` et `pas-un-accord`, et rien du zinc. `app.js` demande
/// `bloc-zinc`, `bloc-repere-eta` et `commande-eta` et écrit leur `.hidden` d'après `blocsCaches(choix)` de `lecture.js` (une seule fois, sans négation).
///
/// Ne couvre pas : la condition qui cache ou montre chaque bloc (le DOM n'est pas éprouvé : seul un navigateur le montre).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_phrases_du_zinc_ne_sont_que_pour_la_dla() {
    let html = e5::lire_page("index.html");
    let zinc = ids_dans_le_div(&html, "bloc-zinc");
    for id in ["barre-zinc", "marque-d", "repere-zinc", "compatible", "hors-barre", "taille-zinc"] {
        assert!(zinc.iter().any(|dedans| dedans == id), "« {id} » n'est pas dans bloc-zinc : {zinc:?}");
    }
    for id in ["bloc-repere-eta", "repere-eta", "ecart-eta", "pas-un-accord"] {
        assert!(!zinc.iter().any(|dedans| dedans == id), "« {id} » est dans bloc-zinc");
    }
    let repere = ids_dans_le_div(&html, "bloc-repere-eta");
    for id in ["repere-eta", "ecart-eta", "pas-un-accord"] {
        assert!(repere.iter().any(|dedans| dedans == id), "« {id} » n'est pas dans bloc-repere-eta : {repere:?}");
    }
    for id in ["repere-zinc", "compatible", "hors-barre", "taille-zinc", "barre-zinc"] {
        assert!(!repere.iter().any(|dedans| dedans == id), "« {id} » est dans bloc-repere-eta");
    }
    let code = code_de_app_js();
    for id in ["bloc-zinc", "bloc-repere-eta", "commande-eta"] {
        let variable = variable_de(&code, id)
            .unwrap_or_else(|| panic!("app.js ne garde pas l'élément « {id} » dans une variable"));
        assert!(
            code.contains(&format!("{variable}.hidden =")),
            "app.js n'écrit jamais `{variable}.hidden =` (l'élément « {id} »)"
        );
        // La décision (cacher ou montrer) vient de `blocsCaches` de lecture.js, que Node éprouve : app.js l'écrit telle quelle, sans négation.
        assert!(
            code.contains(&format!("{variable}.hidden = caches.")),
            "app.js n'écrit pas `{variable}.hidden = caches.…` (la décision de blocsCaches, sans négation)"
        );
    }
    let arites = arites_des_appels(&code, "blocsCaches");
    assert!(
        arites == [1],
        "app.js appelle blocsCaches avec {arites:?} argument(s) au lieu d'un seul appel à 1 argument"
    );
    assert!(
        code_de_lecture_js().contains("export function blocsCaches("),
        "lecture.js ne déclare pas `export function blocsCaches(`"
    );
}

/// forme : le calcul de la courbe se fait un amas à la fois, à la demande (V-D3-30). `app.js` appelle `givre_courbe_pas(` **une seule fois**, dans une fonction déclarée `function nom(…) { … }` qui
/// appelle `setTimeout(` (la pause entre deux pas) et ne contient aucune boucle (`for`, `while`) ; `givre_courbe_demarrer(` est appelé une seule fois, avec deux arguments `COURBE_TAILLE` et
/// `COURBE_GRAINES`, dans la fonction installée comme gestionnaire du clic du bouton `calculer-courbe` (`addEventListener("click", nom)`) ; cette fonction n'appelle pas `givre_courbe_pas` (le
/// premier pas vient après la pause) ; ni l'une ni l'autre n'est appelée directement ailleurs (leur nom suivi d'une parenthèse n'apparaît que dans leur déclaration) ; `COURBE_TAILLE` est une constante entière de 1 à 100 et `COURBE_GRAINES` de 2 à 8 (les bornes du module) ; ni `requestAnimationFrame`, ni `givre_avancer`
/// n'est dans ces fonctions (le calcul de la courbe ne touche pas à l'animation) ; **chacune des deux appelle `texteAvancement(`** (le texte d'avancement vient de `lecture.js`, que Node
/// éprouve, et non d'un texte écrit dans `app.js`).
///
/// Ne couvre pas : que le gestionnaire soit relié au bon bouton (le DOM n'est pas éprouvé) ; ce que la page fait du résultat.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn app_js_calcule_la_courbe_un_amas_a_la_fois() {
    let code = code_de_app_js();
    let pas: Vec<usize> = code.match_indices("givre_courbe_pas(").map(|(i, _)| i).collect();
    assert_eq!(pas.len(), 1, "app.js doit appeler givre_courbe_pas() une seule fois, il l'appelle {} fois", pas.len());
    let (nom_du_pas, corps_du_pas) =
        fonction_qui_contient(&code, pas[0]).expect("l'appel de givre_courbe_pas est dans une `function nom(…)`");
    assert!(
        corps_du_pas.contains("setTimeout("),
        "la fonction « {nom_du_pas} » qui fait un pas n'a pas de setTimeout (la page ne respirerait pas)"
    );
    for boucle in ["for (", "for(", "while (", "while("] {
        assert!(
            !corps_du_pas.contains(boucle),
            "la fonction « {nom_du_pas} » contient une boucle « {boucle} » : un seul amas par tour"
        );
    }
    for interdit in ["requestAnimationFrame", "givre_avancer"] {
        assert!(!corps_du_pas.contains(interdit), "la fonction « {nom_du_pas} » appelle {interdit}");
    }

    let demarrages: Vec<usize> = code.match_indices("givre_courbe_demarrer(").map(|(i, _)| i).collect();
    assert_eq!(demarrages.len(), 1, "app.js doit appeler givre_courbe_demarrer() une seule fois");
    let (nom_du_demarrage, corps_du_demarrage) = fonction_qui_contient(&code, demarrages[0])
        .expect("l'appel de givre_courbe_demarrer est dans une `function nom(…)`");
    assert!(
        !corps_du_demarrage.contains("givre_courbe_pas("),
        "« {nom_du_demarrage} » fait un pas : le premier pas vient après la pause"
    );
    assert!(corps_du_demarrage.contains("setTimeout("), "« {nom_du_demarrage} » ne programme pas le premier pas");
    assert!(
        corps_du_demarrage.contains("givre_courbe_demarrer(COURBE_TAILLE, COURBE_GRAINES)"),
        "« {nom_du_demarrage} » ne démarre pas avec COURBE_TAILLE et COURBE_GRAINES"
    );
    for interdit in ["requestAnimationFrame", "givre_avancer"] {
        assert!(!corps_du_demarrage.contains(interdit), "la fonction « {nom_du_demarrage} » appelle {interdit}");
    }
    for (nom, corps) in [(&nom_du_pas, &corps_du_pas), (&nom_du_demarrage, &corps_du_demarrage)] {
        assert!(
            corps.contains("texteAvancement("),
            "« {nom} » n'écrit pas l'avancement par texteAvancement de lecture.js"
        );
    }
    // Ni le pas ni le démarrage ne sont appelés ailleurs que par la pause ou le clic : leur nom suivi d'une parenthèse n'apparaît que dans leur déclaration.
    for nom in [&nom_du_pas, &nom_du_demarrage] {
        assert_eq!(
            code.matches(&format!("{nom}(")).count(),
            1,
            "« {nom} » est appelée directement (hors du clic et de la pause)"
        );
    }
    let bouton =
        variable_de(&code, "calculer-courbe").expect("app.js garde le bouton calculer-courbe dans une variable");
    assert!(
        code.contains(&format!("{bouton}.addEventListener(\"click\", {nom_du_demarrage})")),
        "« {nom_du_demarrage} » n'est pas installée par {bouton}.addEventListener(\"click\", {nom_du_demarrage})"
    );

    let taille = constante_entiere(&code, "COURBE_TAILLE").expect("const COURBE_TAILLE = <entier>");
    let graines = constante_entiere(&code, "COURBE_GRAINES").expect("const COURBE_GRAINES = <entier>");
    assert!((1..=100).contains(&taille), "COURBE_TAILLE vaut {taille} : le module accepte de 1 à 100 sites");
    assert!((2..=8).contains(&graines), "COURBE_GRAINES vaut {graines} : le module accepte de 2 à 8 graines");
}

/// forme : le curseur `η` pilote le modèle (V-D3-37). `app.js` donne à `givre_demarrer` un numéro qui vient de `numeroDuModele(choix, η)` (deuxième argument : **pas un nombre écrit**, le curseur `η` y est lu par
/// `Number(<curseur>.value)` quelque part dans `app.js`) ; la fonction qui appelle `givre_demarrer` (`relancer`) est installée sur le **`change`** du curseur `η` (son déplacement relance) **et** sur le `change` du choix
/// du modèle et sur le clic du bouton `relancer` ; le `input` du curseur ne relance pas (il n'écrit que la valeur affichée : un `relancer` à chaque cran du curseur ferait recommencer un calcul coûteux à chaque pixel de glisse).
///
/// Ne couvre pas : que les écouteurs soient installés au bon moment (après le chargement du module) ni que le DOM réagisse (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn app_js_relie_le_curseur_eta_au_modele() {
    let code = code_de_app_js();
    let appel = code.find("givre_demarrer(").expect("app.js ne démarre jamais un modèle");
    let (relancer, _) = fonction_qui_contient(&code, appel).expect("l'appel est dans une `function nom(…)`");
    let curseur = variable_de(&code, "eta").expect("app.js garde le curseur eta dans une variable");
    let choix = variable_de(&code, "modele").expect("app.js garde le choix du modèle dans une variable");
    let bouton = variable_de(&code, "relancer").expect("app.js garde le bouton relancer dans une variable");
    for (element, evenement) in [(&curseur, "change"), (&choix, "change"), (&bouton, "click")] {
        assert!(
            code.contains(&format!("{element}.addEventListener(\"{evenement}\", {relancer})")),
            "« {relancer} » n'est pas installée par {element}.addEventListener(\"{evenement}\", {relancer})"
        );
    }
    assert!(
        !code.contains(&format!("{curseur}.addEventListener(\"input\", {relancer})")),
        "le curseur η relance à chaque cran (`input`) : seul `change` doit relancer"
    );
    assert!(
        code.contains(&format!("Number({curseur}.value)")),
        "app.js ne lit pas η dans le curseur (`Number({curseur}.value)`)"
    );
    // Le deuxième argument de `numeroDuModele` n'est pas un nombre écrit.
    let debut = code.find("numeroDuModele(").expect("app.js appelle numeroDuModele") + "numeroDuModele(".len();
    let fin = code[debut..].find(')').expect("parenthèse fermante");
    let arguments = &code[debut..debut + fin];
    let (_, second) = arguments.split_once(',').expect("deux arguments");
    assert!(
        !second.trim().starts_with(|c: char| c.is_ascii_digit()),
        "app.js donne un nombre écrit à numeroDuModele comme η : {second}"
    );
}

/// forme : les modèles du claquage se jouent unité par unité, sans bloquer la page (V-D3-31). `app.js` appelle `givre_avancer(1)` (une unité) dans une boucle (`while` ou `for`) d'une fonction qui lit
/// `performance.now()` et s'arrête quand `BUDGET_IMAGE_MS` sont écoulées ; `BUDGET_IMAGE_MS` est une constante entière d'au plus 100 (une image de l'animation ne dure pas plus d'un dixième de seconde à
/// elle seule) et elle est employée ailleurs que dans sa déclaration ; l'autre appel de `givre_avancer` (les modèles 0 à 3) reste à un argument.
///
/// Ne couvre pas : que la boucle s'arrête vraiment (le DOM et l'horloge ne sont pas éprouvés) ; le temps d'une unité dans le module (voir le rapport de l'étape).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn app_js_joue_le_claquage_unite_par_unite() {
    let code = code_de_app_js();
    let unite = code.find("givre_avancer(1)").expect("app.js ne joue jamais givre_avancer(1)");
    let (nom, corps) = fonction_qui_contient(&code, unite).expect("l'appel est dans une `function nom(…)`");
    assert!(corps.contains("while (") || corps.contains("for ("), "« {nom} » ne boucle pas sur les unités");
    assert!(corps.contains("performance.now()"), "« {nom} » ne lit pas l'horloge");
    assert!(corps.contains("BUDGET_IMAGE_MS"), "« {nom} » ne s'arrête pas à BUDGET_IMAGE_MS");
    let budget = constante_entiere(&code, "BUDGET_IMAGE_MS").expect("const BUDGET_IMAGE_MS = <entier>");
    assert!((1..=100).contains(&budget), "BUDGET_IMAGE_MS vaut {budget}");
    assert!(code.matches("BUDGET_IMAGE_MS").count() >= 2, "BUDGET_IMAGE_MS n'est pas employée");
    for arite in arites_des_appels(&code, "givre_avancer") {
        assert_eq!(arite, 1, "givre_avancer est appelée avec {arite} arguments");
    }
}

/// forme : `app.js` s'appuie sur `lecture.js` pour ce qui se teste sous Node (V-D3-32). `lecture.js` déclare `export function lireCourbe(`, `numeroDuModele(`, `texteAvancement(`, `texteRepereEta(` et `formater(`
/// et n'emploie ni `document` ni `window` ; `app.js` les importe de `"./lecture.js"` et les appelle avec le bon nombre d'arguments (`lireCourbe` 1, `numeroDuModele` 2, `texteAvancement` 1,
/// `texteRepereEta` 3, `formater` 1) ; le numéro de modèle que `app.js` donne à `givre_demarrer` vient de `numeroDuModele` (pas d'un calcul à part dans `app.js` : il n'écrit pas `MODELE_CLAQUAGE`).
///
/// Ne couvre pas : ce que ces fonctions rendent (voir `lecture_pure_dit_ce_que_la_doc_dit` et `etape_d3_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn app_js_s_appuie_sur_lecture_js() {
    let lecture = code_de_lecture_js();
    let app = code_de_app_js();
    let attendues: [(&str, usize); 5] =
        [("lireCourbe", 1), ("numeroDuModele", 2), ("texteAvancement", 1), ("texteRepereEta", 3), ("formater", 1)];
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
                // L'instruction `import { … } from "./lecture.js"` : du dernier `import` avant le chemin jusqu'à lui.
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
    assert!(!app.contains("MODELE_CLAQUAGE"), "app.js calcule le numéro du claquage lui-même");
    // « Mesurer D » écrit le repère de η : la fonction qui appelle `givre_mesurer` appelle celle qui appelle `texteRepereEta`.
    let repere = app.find("texteRepereEta(").expect("app.js appelle texteRepereEta");
    let (ecrit_le_repere, _) =
        fonction_qui_contient(&app, repere).expect("texteRepereEta est appelée dans une `function nom(…)`");
    let mesure = app.find("givre_mesurer(").expect("app.js appelle givre_mesurer");
    let (nom_de_la_mesure, corps_de_la_mesure) =
        fonction_qui_contient(&app, mesure).expect("givre_mesurer est appelée dans une `function nom(…)`");
    assert!(
        corps_de_la_mesure.contains(&format!("{ecrit_le_repere}(")),
        "« {nom_de_la_mesure} » (le gestionnaire de « Mesurer D ») n'appelle pas « {ecrit_le_repere} », qui écrit le repère de η"
    );
}

/// forme : aucune valeur publiée ni aucune valeur de `D` n'est écrite dans la page (V-D3-33). Ni `index.html`, ni `app.js`, ni `lecture.js` ne contient `1,71`, `1.71`, `1,433`, `1.433`, `1,263` ni
/// `1.263` (les repères viennent du module : `givre_courbe_repere`) ; le texte visible d'`index.html` n'a aucun « D = » ni « D= » suivi d'un chiffre ; `app.js` appelle `givre_courbe_repere(` pour le repère
/// de `η` et `lecture.js` pour la courbe ; aucun des trois fichiers ne contient `http://` ni `https://`.
///
/// Ne couvre pas : un nombre mesuré copié dans une constante écrite autrement ; le texte réellement affiché.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_ne_fige_aucune_valeur_publiee() {
    for nom in ["index.html", "app.js", "lecture.js"] {
        let texte = e5::lire_page(nom);
        for interdit in ["1,71", "1.71", "1,433", "1.433", "1,263", "1.263", "http://", "https://"] {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} »");
        }
    }
    let visible = texte_visible(&e5::lire_page("index.html"));
    for motif in ["D =", "D="] {
        for (position, _) in visible.match_indices(motif) {
            let suite = visible[position + motif.len()..].trim_start();
            assert!(!suite.starts_with(|c: char| c.is_ascii_digit()), "index.html écrit une valeur de D");
        }
    }
    assert!(
        !arites_des_appels(&code_de_app_js(), "givre_courbe_repere").is_empty(),
        "app.js ne lit pas le repère de η dans le module"
    );
    assert!(
        !arites_des_appels(&code_de_lecture_js(), "givre_courbe_repere").is_empty(),
        "lecture.js ne lit pas les repères de la courbe dans le module"
    );
}

/// juste : les fonctions de `lecture.js` qui ne lisent pas le module disent ce que la doc dit, sous Node (V-D3-34). Un calcul **à la main** : `numeroDuModele` rend 0, 1, 2, 3 pour les choix « 0 » à « 3 » (quel
/// que soit `η`) et 4, 5, 6, 7 pour « claquage » et `η` de 0 à 3 ; `blocsCaches` cache le zinc pour « claquage » seulement (le repère de `η` et le curseur pour tout autre choix) ; `formater` écrit trois décimales avec une virgule (`1,234`, `1,235`, `0,000`, `2,000`) ; `texteAvancement` écrit « η = 2, graine 3 sur 6 » pour 14
/// amas faits de 24 avec 6 graines (le prochain amas est le 15e : `η = 14 / 6 = 2`, graine `14 % 6 + 1 = 3`), « Aucun calcul pour l'instant. » sans travail, « Calcul terminé : 24 amas. » à la fin, et
/// « Calcul arrêté : <ce que veut dire le code> (code N). » pour une erreur (« erreur » pour un code inconnu) ; `texteRepereEta` écrit « pas de repère publié pour η = 0 » sans écart pour un repère
/// `NaN`, sinon « repère publié (rappel de la DLA) : 1,500 » pour `η = 1` et « repère publié (Hastings 2001) : 1,250 » pour `η = 2` et `3`, et « écart du banc au repère : +0,100 » ou « −0,125 » (le signe moins
/// est U+2212) : la dimension mesurée moins le repère, signée. Ce test ne lit pas le module.
///
/// Ne couvre pas : ce que la page fait de ces textes (les écrire dans le DOM) ; les valeurs lues dans le module (voir `etape_d3_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn lecture_pure_dit_ce_que_la_doc_dit() {
    if !e5::node_est_la() {
        return;
    }
    let lecture = racine().join("docs").join("lecture.js");
    let lecture = lecture.to_str().expect("chemin en UTF-8");
    let sortie = lancer_le_script("lecture-pure.mjs", &[lecture]);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "lecture-pure.mjs : {:?}, sortie d'erreur : {erreur}", sortie.status);
    let avancements = [
        "Aucun calcul pour l'instant.",
        "η = 0, graine 1 sur 6",
        "η = 2, graine 3 sur 6",
        "η = 3, graine 6 sur 6",
        "Calcul terminé : 24 amas.",
        "Calcul arrêté : un amas n'a pas de dimension (fenêtre vide) (code 4).",
        "Calcul arrêté : un amas a touché la frontière avant sa taille (code 3).",
        "Calcul arrêté : erreur (code 9).",
    ];
    let reperes = [
        "pas de repère publié pour η = 0 / null",
        "repère publié (rappel de la DLA) : 1,500 / écart du banc au repère : +0,100",
        "repère publié (Hastings 2001) : 1,250 / écart du banc au repère : \u{2212}0,125",
        "repère publié (Hastings 2001) : 1,500 / écart du banc au repère : +0,000",
        "repère publié (rappel de la DLA) : 2,000 / écart du banc au repère : \u{2212}0,500",
        "repère publié (Hastings 2001) : 1,500 / écart du banc au repère : +0,000",
    ];
    let attendu = format!(
        "numeros=0,1,2,3,4,5,6,7\nformater=1,234;1,235;0,000;0,000;2,000;2,000\navancement={}\nreperes={}\n\
         blocs=0:false,true,true|1:false,true,true|2:false,true,true|3:false,true,true|claquage:true,false,false\n",
        avancements.join("|"),
        reperes.join("|")
    );
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu);

    for arguments in [&[][..], &["a", "b"], &["n-existe-pas.js"]] {
        let sortie = lancer_le_script("lecture-pure.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "lecture-pure.mjs {arguments:?}");
        assert!(sortie.stdout.is_empty(), "lecture-pure.mjs {arguments:?} : la sortie standard n'est pas vide");
    }
}

//! Étape D3 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) du claquage diélectrique et de la courbe `D(η)`. Valeurs attendues : `references/etape-d3-valeurs.md`,
//! identifiants `V-D3-xx`. Comme aux étapes 5 et Z1, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_pure_dit_ce_que_la_doc_dit` appelle les fonctions de `lecture.js` qui n'ont pas besoin du module (le numéro de modèle, les textes d'avancement et de repère) et compare à un calcul fait à
//! la main ; il ne lit pas `docs/givre.wasm` et passe dès que `lecture.js` est écrit. La lecture du module par `lecture.js` (`lireCourbe`) est éprouvée dans `etape_d3_module.rs`.
//!
//! **Étape V8 : la page est tous les fichiers de son dossier** (voir `etape_5_page.rs`) : ces tests lisent `index.html`, ses scripts en ligne, les feuilles de style et **tous** les fichiers JavaScript ; ils ne figent plus l'écriture d'`app.js` (qui peut être réécrit, ou calculer dans un worker).
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : celle de D3 ; V8 en a retiré ce qui n'était que l'écriture d'`app.js`)
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

use etape_d3_commun::{e5, lancer_le_script};

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
    e5::tout_le_code_js()
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
    e5::attribut(balise, nom)
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
    e5::texte_de_l_element(html, id)
}

/// Vrai si la balise porte l'attribut `nom` (booléen comme `hidden`, ou avec une valeur).
fn a_l_attribut(balise: &str, nom: &str) -> bool {
    balise
        .split_whitespace()
        .skip(1)
        .any(|mot| mot == nom || mot.starts_with(&format!("{nom}=")) || mot == format!("{nom}/"))
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

/// Les `id` des balises contenues dans l'élément d'`id` `conteneur` (de n'importe quelle sorte : `div`, `section`, `details`, `aside`…), à n'importe quelle profondeur, dans l'ordre du document (V8 : le test ne dépend plus de la sorte du conteneur).
fn ids_dans_l_element(html: &str, conteneur: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let (mut profondeur, mut sorte) = (0_i32, String::new());
    for balise in balises(html) {
        let ferme = balise.starts_with('/');
        let nom = nom_de_balise(balise.trim_start_matches('/'));
        if profondeur == 0 {
            if !ferme && attribut(balise, "id").as_deref() == Some(conteneur) {
                profondeur = 1;
                sorte = nom;
            }
            continue;
        }
        if nom == sorte {
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

/// Vrai si la phrase est **écrite par la page** (V8) : dans le texte de l'élément d'`id` (s'il en a un dans `index.html`), sinon dans une chaîne de caractères de l'un des fichiers JavaScript (une légende peut être écrite par un script).
fn phrase_dite(html: &str, id: &str, contient: &[&str]) -> bool {
    let statique = texte_de(html, id).unwrap_or_default().to_lowercase();
    if !statique.is_empty() {
        return contient.iter().all(|mot| statique.contains(&mot.to_lowercase()));
    }
    e5::chaines_du_code(&e5::tout_le_code_js())
        .iter()
        .any(|chaine| contient.iter().all(|mot| chaine.to_lowercase().contains(&mot.to_lowercase())))
}

// ---------------------------------------------------------------------------------------------------------------
// Les tests de forme.
// ---------------------------------------------------------------------------------------------------------------

/// forme : `index.html` a les éléments du claquage et de la courbe, chacun avec son `id` et sa sorte (V-D3-27) : `<input id="eta" type="range" min="0" max="3" step="1">` (un curseur entier de 0 à 3) et
/// son `<output id="valeur-eta">` ; (V8 : le conteneur `commande-eta` n'est plus exigé caché au départ : c'était l'état d'ouverture de l'ancienne page) ; la cinquième option du choix du modèle a la valeur `claquage` et son texte dit « claquage » et
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
    // Étape G3 : une sixième option, « neige », peut suivre le claquage (avec `etape-g3` elle y est : voir `etape_g3_page.rs`) ; le claquage est alors l'avant-dernière.
    let avec_neige = options.last().is_some_and(|(valeur, _)| valeur == "neige");
    assert!(avec_neige || !e5::G3_ACTIVE, "avec etape-g3, la dernière option du choix du modèle est « neige »");
    let (valeur, texte) = if avec_neige {
        options.get(options.len() - 2).expect("des options")
    } else {
        options.last().expect("des options")
    };
    assert_eq!(valeur, "claquage", "la dernière option du choix du modèle (avant « neige » s'il y en a une)");
    assert!(texte.contains("claquage") && texte.contains("diélectrique"), "le texte de l'option : « {texte} »");
    assert_eq!(options.len(), if avec_neige { 6 } else { 5 });
    let bouton = texte_de(&html, "calculer-courbe").expect("le bouton");
    assert!(bouton.contains("courbe"), "le texte du bouton est « {bouton} »");
}

/// forme : les phrases (V-D3-28). `pas-un-accord` et `pas-un-accord-courbe` disent « ce n'est pas un accord » (sans tenir compte de la casse) ; `limite-courbe` dit la limite en clair (les mots `amas`, `graines`, `méthode` ou `corrélation`, `fenêtre`, `Hastings`) ; `legende-courbe` nomme les deux sortes de repère (`Hastings 2001` et `rappel de la DLA`) ;
/// `bloc-repere-eta`, `ecart-eta` et `pas-un-accord` sont **cachés au départ** (rien n'est dit avant une mesure) ; `repere-eta` est vide au départ (le repère vient du module). Le mot « accord » ne figure jamais sans négation dans **tout ce que la page écrit** (le texte visible de chaque fichier HTML, les chaînes de caractères de tous les fichiers JavaScript, le
/// texte des SVG : « en accord », « d'accord », « un accord » hors de « pas un accord » : refusés). **V8 : une phrase peut être écrite par un script** : si l'élément est vide dans `index.html`, la phrase doit être une chaîne de caractères de l'un des fichiers JavaScript de la page ; si l'élément a un texte, c'est lui qui est jugé.
///
/// Ne couvre pas : l'affichage de l'une ou l'autre phrase selon la mesure (le DOM n'est pas éprouvé) ; la justesse de la limite dite ; que le script écrive la phrase dans le bon élément.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_dit_pas_un_accord_et_la_limite() {
    let html = e5::lire_page("index.html");
    for id in ["pas-un-accord", "pas-un-accord-courbe"] {
        assert!(balise_d_id(&html, id).is_some(), "pas d'élément d'id « {id} »");
        assert!(phrase_dite(&html, id, &["ce n'est pas un accord"]), "« {id} » ne dit pas « ce n'est pas un accord »");
    }
    assert!(balise_d_id(&html, "limite-courbe").is_some(), "pas d'élément d'id « limite-courbe »");
    for mot in ["amas", "graines", "fenêtre", "hastings"] {
        assert!(phrase_dite(&html, "limite-courbe", &[mot]), "la limite dite ne parle pas de « {mot} »");
    }
    assert!(
        phrase_dite(&html, "limite-courbe", &["méthode"]) || phrase_dite(&html, "limite-courbe", &["corrélation"]),
        "la limite ne dit pas la méthode"
    );
    assert!(balise_d_id(&html, "legende-courbe").is_some(), "pas d'élément d'id « legende-courbe »");
    for mot in ["Hastings 2001", "rappel de la DLA"] {
        assert!(phrase_dite(&html, "legende-courbe", &[mot]), "la légende de la courbe ne dit pas « {mot} »");
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
    let ecrit = e5::textes_en_prose_de_la_page().to_lowercase();
    for (position, _) in ecrit.match_indices("accord") {
        assert!(
            ecrit[..position].trim_end().ends_with("pas un"),
            "« accord » sans « pas un » : …{}",
            &ecrit[position.saturating_sub(30)..(position + 10).min(ecrit.len())]
        );
    }
}

/// forme : les phrases du zinc ne sont que pour la DLA (V-D3-29). Dans `index.html`, le conteneur `bloc-zinc` (de n'importe quelle sorte d'élément) contient `barre-zinc`, `marque-d`, `repere-zinc`, `compatible`, `hors-barre` et `taille-zinc`, et ne contient ni `bloc-repere-eta`, ni `repere-eta`, ni `pas-un-accord` ; le conteneur
/// `bloc-repere-eta` contient `repere-eta`, `ecart-eta` et `pas-un-accord`, et rien du zinc. **V8 ne lit plus `app.js`** (il demandait `bloc-zinc`, `bloc-repere-eta` et `commande-eta` et écrivait leur `.hidden` d'après `blocsCaches(choix)`, « sans négation » : une écriture) : que le zinc soit caché pour un autre modèle que la DLA se voit sous un navigateur
/// (voir `MODELE.md`, étape V8) ; `blocsCaches` reste éprouvé sous Node (`lecture_pure_dit_ce_que_la_doc_dit`).
///
/// Ne couvre pas : la condition qui cache ou montre chaque bloc (le DOM n'est pas éprouvé : seul un navigateur le montre).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn les_phrases_du_zinc_ne_sont_que_pour_la_dla() {
    let html = e5::lire_page("index.html");
    let zinc = ids_dans_l_element(&html, "bloc-zinc");
    for id in ["barre-zinc", "marque-d", "repere-zinc", "compatible", "hors-barre", "taille-zinc"] {
        assert!(zinc.iter().any(|dedans| dedans == id), "« {id} » n'est pas dans bloc-zinc : {zinc:?}");
    }
    for id in ["bloc-repere-eta", "repere-eta", "ecart-eta", "pas-un-accord"] {
        assert!(!zinc.iter().any(|dedans| dedans == id), "« {id} » est dans bloc-zinc");
    }
    let repere = ids_dans_l_element(&html, "bloc-repere-eta");
    for id in ["repere-eta", "ecart-eta", "pas-un-accord"] {
        assert!(repere.iter().any(|dedans| dedans == id), "« {id} » n'est pas dans bloc-repere-eta : {repere:?}");
    }
    for id in ["repere-zinc", "compatible", "hors-barre", "taille-zinc", "barre-zinc"] {
        assert!(!repere.iter().any(|dedans| dedans == id), "« {id} » est dans bloc-repere-eta");
    }
}

/// forme : la page demande au module des amas **dans ses bornes** (V-D3-30). **Si** une constante `COURBE_TAILLE` ou `COURBE_GRAINES` est déclarée dans le code de la page (`const COURBE_TAILLE = <entier>`), elle est dans les bornes du module (1 à 100 sites, 2 à 8 graines) ; **si** `givre_courbe_demarrer(` est appelée avec deux entiers écrits (`givre_courbe_demarrer(60, 4)`),
/// ils y sont aussi (le module refuse sinon par un code d'erreur). **V8 ne lit plus** « un seul `givre_courbe_pas(` dans une fonction qui appelle `setTimeout(` et ne boucle pas, gestionnaire du clic, jamais appelée directement » : c'est l'écriture d'`app.js` ; la page de la session de site calcule hors du fil de la page, où une boucle est permise : qu'un
/// amas se fasse à la fois et que la page respire ne se voit plus ici (voir `MODELE.md`, étape V8).
///
/// Ne couvre pas : que la page calcule un amas à la fois ; que le calcul soit lancé par le bouton `calculer-courbe`.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_demande_des_amas_dans_les_bornes_du_module() {
    let code = code_de_app_js();
    if let Some(taille) = constante_entiere(&code, "COURBE_TAILLE") {
        assert!((1..=100).contains(&taille), "COURBE_TAILLE vaut {taille} : le module accepte de 1 à 100 sites");
    }
    if let Some(graines) = constante_entiere(&code, "COURBE_GRAINES") {
        assert!((2..=8).contains(&graines), "COURBE_GRAINES vaut {graines} : le module accepte de 2 à 8 graines");
    }
    for (position, _) in code.match_indices("givre_courbe_demarrer(") {
        let reste = &code[position + "givre_courbe_demarrer(".len()..];
        let arguments: Vec<&str> = reste[..reste.find(')').unwrap_or(reste.len())].split(',').map(str::trim).collect();
        if let [taille, graines] = arguments.as_slice()
            && let (Ok(taille), Ok(graines)) = (taille.parse::<u32>(), graines.parse::<u32>())
        {
            assert!((1..=100).contains(&taille), "givre_courbe_demarrer({taille}, {graines}) : la taille");
            assert!((2..=8).contains(&graines), "givre_courbe_demarrer({taille}, {graines}) : les graines");
        }
    }
}

/// forme : le numéro de modèle de η vient de `lecture.js` (V-D3-37). **Chaque appel** de `numeroDuModele(` du code de la page a deux arguments, et le second (η) **n'est pas un nombre écrit** (il est lu dans le curseur). **V8 ne lit plus** « la fonction qui appelle `givre_demarrer` est installée sur le `change` du curseur η, du choix du modèle et sur le clic du bouton ;
/// le `input` du curseur ne relance pas ; `Number(<curseur>.value)` » : l'écriture d'`app.js` (un tiroir, un worker, un autre gestionnaire : voir `MODELE.md`, étape V8).
///
/// Ne couvre pas : que le curseur relance le modèle ; que ce soit à `change` plutôt qu'à `input` ; que η soit lu dans le curseur (un `Number(…)` n'est plus exigé).
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_numero_du_modele_vient_de_lecture_js() {
    let code = code_de_app_js();
    let arites = arites_des_appels(&code, "numeroDuModele");
    assert!(arites.iter().all(|a| *a == 2), "la page appelle numeroDuModele avec {arites:?} argument(s) au lieu de 2");
    for (position, _) in code.match_indices("numeroDuModele(") {
        let apres = &code[position + "numeroDuModele(".len()..];
        let fin = apres.find(')').expect("parenthèse fermante");
        if let Some((_, second)) = apres[..fin].split_once(',') {
            assert!(
                !second.trim().starts_with(|c: char| c.is_ascii_digit()),
                "la page donne un nombre écrit à numeroDuModele comme η : {second}"
            );
        }
    }
}

/// forme : le budget d'une image reste raisonnable (V-D3-31). **Si** une constante `BUDGET_IMAGE_MS` est déclarée dans le code de la page (`const BUDGET_IMAGE_MS = <entier>`), elle vaut de 1 à 100 (une image de l'animation ne dure pas plus d'un dixième de seconde à elle seule) ; chaque appel de `givre_avancer(` a un seul argument. **V8 ne lit plus** « `givre_avancer(1)` dans une boucle
/// d'une fonction qui lit `performance.now()` et s'arrête à `BUDGET_IMAGE_MS` » : c'est l'écriture d'`app.js` sur le fil de la page ; un worker joue le claquage sans bloquer la page (voir `MODELE.md`, étape V8).
///
/// Ne couvre pas : que le claquage soit joué unité par unité ; que la page ne se fige pas.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn le_budget_d_une_image_est_borne() {
    let code = code_de_app_js();
    if let Some(budget) = constante_entiere(&code, "BUDGET_IMAGE_MS") {
        assert!((1..=100).contains(&budget), "BUDGET_IMAGE_MS vaut {budget}");
    }
    for arite in arites_des_appels(&code, "givre_avancer") {
        assert_eq!(arite, 1, "givre_avancer est appelée avec {arite} arguments");
    }
}

/// forme : ce que la page lit passe par `lecture.js`, que Node éprouve (V-D3-32). `lecture.js` déclare `export function lireCourbe(`, `numeroDuModele(`, `texteAvancement(`, `texteRepereEta(` et `formater(` et n'emploie ni `document` ni `window` ; **chaque appel** de ces cinq fonctions dans la page (tous les fichiers JavaScript) a le bon nombre d'arguments
/// (`lireCourbe` 1, `numeroDuModele` 2, `texteAvancement` 1, `texteRepereEta` 3, `formater` 1) ; aucun fichier de la page **hors `lecture.js`** n'écrit `MODELE_CLAQUAGE` (le numéro du claquage vient de `numeroDuModele`, pas d'un calcul à part). **V8 ne lit plus** : que `app.js` importe ces cinq fonctions et les appelle toutes, ni
/// que « Mesurer D » écrive le repère de η par `texteRepereEta` (l'écriture d'`app.js` : une page qui calcule dans un worker lit le module ailleurs ; voir `MODELE.md`, étape V8).
///
/// Ne couvre pas : ce que ces fonctions rendent (voir `lecture_pure_dit_ce_que_la_doc_dit` et `etape_d3_module.rs`) ; que la page les emploie.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn lecture_js_declare_ce_que_la_page_lit() {
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
        assert!(
            arites.iter().all(|a| *a == arite),
            "la page appelle {nom} avec {arites:?} argument(s) au lieu de {arite}"
        );
    }
    for interdit in ["document", "window"] {
        assert!(
            !lecture.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).any(|mot| mot == interdit),
            "lecture.js emploie `{interdit}`"
        );
    }
    assert!(!e5::code_js_hors_lecture().contains("MODELE_CLAQUAGE"), "la page calcule le numéro du claquage elle-même");
}

/// forme : aucune valeur publiée ni aucune valeur de `D` n'est écrite dans la page (V-D3-33). **Aucun fichier de texte du dossier de la page** (HTML, scripts en ligne, CSS, JavaScript, SVG, commentaires compris) ne contient `1,71`, `1.71`, `1,433`, `1.433`, `1,263` ni `1.263` (les repères viennent du module : `givre_courbe_repere`) ; **tout ce que la page écrit**
/// (le texte visible de chaque fichier HTML, les chaînes de tous les fichiers JavaScript, le texte des SVG) n'a aucun « D = » ni « D= » suivi d'un chiffre ; `lecture.js` appelle `givre_courbe_repere(` pour les repères de la courbe, et la page (un fichier de JavaScript, `lecture.js` compris) lit le repère de `η` dans le module. **V8** : les adresses `http://` et `https://` ne sont
/// plus refusées ici (un lien `<a href>` et le `xmlns` d'un SVG passent) : `la_page_n_a_aucune_ressource_externe`, dans `etape_5_page.rs`, refuse celles qui chargent quelque chose.
///
/// Ne couvre pas : un nombre mesuré copié dans une constante écrite autrement ; le texte réellement affiché.
#[test]
#[cfg_attr(not(feature = "etape-d3"), ignore)]
fn la_page_ne_fige_aucune_valeur_publiee() {
    let fichiers = e5::textes_des_fichiers(&["html", "htm", "js", "mjs", "css", "svg"]);
    assert!(fichiers.iter().any(|(nom, _)| nom == "index.html"), "index.html n'est pas lu");
    for (nom, texte) in fichiers {
        let texte = e5::sans_donnees(&texte);
        for interdit in ["1,71", "1.71", "1,433", "1.433", "1,263", "1.263"] {
            assert!(!texte.contains(interdit), "docs/{nom} contient « {interdit} »");
        }
    }
    let ecrit = e5::textes_ecrits_par_la_page();
    for motif in ["D =", "D="] {
        for (position, _) in ecrit.match_indices(motif) {
            let suite = ecrit[position + motif.len()..].trim_start();
            assert!(!suite.starts_with(|c: char| c.is_ascii_digit()), "la page écrit une valeur de D");
        }
    }
    assert!(
        !arites_des_appels(&code_de_app_js(), "givre_courbe_repere").is_empty(),
        "la page ne lit pas le repère de η dans le module"
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
    let lecture = e5::chemin_de_lecture();
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

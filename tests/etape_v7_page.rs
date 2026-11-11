//! Étape V7 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) montre l'image nette des flocons de Reiter, sans `image-rendering: pixelated`. Valeurs attendues : `references/etape-v7-valeurs.md`, identifiants `V-V7-xx`.
//! Comme aux étapes 5, Z1 et D3, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_js_lit_l_image_nette_sur_un_faux_module` appelle `lireImageNette` et `imagesCachees` de `lecture.js` sur un faux module qui change d'état à chaque préparation et compare à un calcul fait à la main ; il ne lit pas
//! `docs/givre.wasm` et passe dès que `lecture.js` est écrit. La lecture du vrai module est éprouvée dans `etape_v7_module.rs`.
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : le prototype de l'écrivain fait foi, ses fichiers se recopient à l'octet)
//!
//! - `index.html` : en gardant tout ce que les étapes 5, Z1 et D3 demandent, **deux canvas** : le `<canvas id="vitrine">` d'avant (un pixel par case : la DLA et le claquage, sur une grille carrée) et un `<canvas id="vitrine-nette">` (les modèles
//!   sur la grille hexagonale, un hexagone par case), **caché au départ** (`hidden`), avec une légende `<p id="legende-nette">` (cachée au départ, elle dit « hexagone »). **`image-rendering: pixelated` n'est écrit que pour `#vitrine`**
//!   (la règle `#vitrine { image-rendering: pixelated; }`) : aucune règle qui l'applique à tous les `canvas`, à une classe ou à `#vitrine-nette` (le canvas des hexagones est déjà lissé par le module : les agrandir en gros carrés
//!   est ce que la page ne fait plus). Aucun attribut `style` ne règle `image-rendering` ; ni `app.js` ni `lecture.js` n'écrivent `image-rendering`, `imageRendering`, `pixelated` ni `imageSmoothingEnabled`, et ils ne changent ni l'`id`, ni la classe,
//!   ni le style écrit d'un élément (`cssText`, `setProperty`, `setAttribute`, `className`, `classList`, `.id =`). Aucune règle ne lisse `#vitrine`, `image-rendering` ne porte jamais `!important`, et la page n'a pas d'autre feuille de style (`<link>`, `@import`).
//! - `lecture.js` exporte de plus (sans DOM) `lireImageNette(m, cote, sur)` (qui demande au module de préparer l'image, `givre_nette_preparer(cote, sur)`, puis la lit et rend `{ code, largeur, hauteur, rgba }`) et `imagesCachees(code)` (qui rend
//!   `{ carree, nette }` : vrai quand le canvas est caché ; l'image nette remplace l'image d'un pixel par case seulement pour le code 0).
//! - `app.js` : demande `vitrine-nette` et `legende-nette` par `getElementById("…")` ; `dessiner(fin)` demande l'image nette (`lireImageNette(m, côté, sur)`, côté et `sur` d'après `fin`), écrit `.hidden` des deux canvas et de la légende d'après
//!   `imagesCachees(code)` (sans négation), pose l'image nette sur le canvas des hexagones (`putImageData`) quand le code est 0, sinon l'image d'un pixel par case (`lireImage(m)`) sur l'autre ; quatre constantes entières `NETTE_COTE_ANIMATION`,
//!   `NETTE_SUR_ANIMATION`, `NETTE_COTE_FIN` et `NETTE_SUR_FIN` (dans les bornes du module : côté pair de 2 à 2048, `sur` de 1 à 4, côté × `sur` au plus 4096 ; l'image de l'animation, légère, au moins 256 pixels de côté à 2 × 2 échantillons et au plus 1024 échantillons de côté ; celle de la fin plus fine)
//!   ; l'animation dessine à la qualité de l'animation (`dessiner(false)` au démarrage) et la dernière image, quand le scénario s'arrête, plus fine (`dessiner(joue === 0)`).
//!
//! Étiquettes : `lecture_js_lit_l_image_nette_sur_un_faux_module` est **juste** (comparé à un calcul à la main) ; tous les autres sont de la **forme** (régression de texte), jamais une preuve du comportement de la page.

mod etape_v7_commun;

use std::io::Write;
use std::process::{Command, Stdio};

use etape_v7_commun::{
    COTE_MAX, GRILLE_MAX, SUR_MAX, a_l_attribut, arites_des_appels, attendu_de_la_lecture_pure, attribut, balise_d_id,
    balises, chemin_de_lecture, code_de_app_js, code_de_lecture_js, constante_entiere, css_de_la_page, e5, fermeture,
    fonction_qui_contient, lancer_le_script, nom_de_balise, regles_css, texte_de, variable_de,
};

/// Les valeurs de `image-rendering` qui ne lissent pas : un agrandissement en gros carrés (ou presque).
const SANS_LISSAGE: [&str; 5] =
    ["pixelated", "crisp-edges", "-moz-crisp-edges", "optimize-contrast", "-webkit-optimize-contrast"];

/// forme : `index.html` a les deux canvas et la légende de l'image nette (V-V7-30). `<canvas id="vitrine">` (un pixel par case) reste et **n'est pas caché** au départ ; `<canvas id="vitrine-nette">` existe, **caché au départ** (`hidden`, pour que la
/// page n'en montre un qu'une fois le modèle démarré), avec une largeur et une hauteur numériques ; `<p id="legende-nette">` est caché au départ et dit « hexagone » ; chacun des deux canvas est dans le même cadre (leurs balises se suivent).
///
/// Ne couvre pas : que le canvas soit montré ou caché au bon moment (le DOM n'est pas éprouvé : voir `app_js_dessine_l_image_nette_sur_son_canvas`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn la_page_a_les_deux_canvas_et_la_legende() {
    let html = e5::lire_page("index.html");
    for (id, sorte) in [("vitrine", "canvas"), ("vitrine-nette", "canvas"), ("legende-nette", "p")] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), sorte, "l'élément d'id « {id} » n'est pas un <{sorte}>");
    }
    let carre = balise_d_id(&html, "vitrine").expect("vitrine");
    assert!(!a_l_attribut(carre, "hidden"), "le canvas d'un pixel par case est caché au départ : <{carre}>");
    let nette = balise_d_id(&html, "vitrine-nette").expect("vitrine-nette");
    assert!(a_l_attribut(nette, "hidden"), "le canvas de l'image nette n'est pas caché au départ : <{nette}>");
    for nom in ["width", "height"] {
        let valeur = attribut(nette, nom).unwrap_or_else(|| panic!("le canvas de l'image nette n'a pas de {nom}"));
        assert!(valeur.parse::<u32>().is_ok_and(|n| n > 0), "{nom} du canvas de l'image nette : {valeur}");
    }
    let legende = balise_d_id(&html, "legende-nette").expect("legende-nette");
    assert!(a_l_attribut(legende, "hidden"), "la légende de l'image nette n'est pas cachée au départ : <{legende}>");
    let texte = texte_de(&html, "legende-nette").expect("le texte de la légende").to_lowercase();
    assert!(texte.contains("hexagone"), "la légende dit : « {texte} »");
    // Les deux canvas se suivent dans le même cadre : aucune autre balise `canvas` entre eux.
    let canvas: Vec<&str> = balises(&html).into_iter().filter(|b| nom_de_balise(b) == "canvas").collect();
    let position = |id: &str| canvas.iter().position(|b| attribut(b, "id").as_deref() == Some(id)).expect("canvas");
    assert_eq!(position("vitrine-nette"), position("vitrine") + 1, "les deux canvas de l'image se suivent");
}

/// forme : **`image-rendering: pixelated` n'est pas sur le canvas des modèles hexagonaux** (V-V7-31). Les règles CSS de `index.html` sont lues (les `<style>`, sans commentaires, les blocs `@media`, `@supports`, `@layer` compris) : chaque règle qui
/// donne à `image-rendering` une valeur qui ne lisse pas (`pixelated`, `crisp-edges`, `-moz-crisp-edges`, `optimize-contrast`, `-webkit-optimize-contrast`) a pour sélecteurs **exactement** `#vitrine` ou `canvas#vitrine` et rien d'autre : un sélecteur général
/// (`canvas`, `*`, `.cadre canvas`, une classe, `#vitrine-nette`) est refusé, même s'il ne vise « que » le bon canvas aujourd'hui. Et il **en reste une** : `#vitrine` (le canvas d'un pixel par case, la DLA et le claquage) garde `pixelated`. Aucun attribut `style` de
/// la page ne règle `image-rendering`, et ni `app.js` ni `lecture.js` n'écrivent (hors de leurs commentaires) `image-rendering`, `imageRendering`, `pixelated` ni `imageSmoothingEnabled` (un style posé par le script ferait la même faute que la feuille de style).
/// De plus (ajouts du juge) : une règle qui donne à `image-rendering` une valeur **qui lisse** (`auto`…) ne nomme pas `#vitrine` (écrite plus loin, elle lisserait le canvas d'un pixel par case) ; `image-rendering` ne porte jamais `!important` ; la page n'a ni
/// `<link rel="stylesheet">` ni `@import` (tout le CSS est dans les `<style>` lus ici) ; et les scripts n'écrivent ni `cssText`, ni `setProperty`, ni `setAttribute`, ni `className`, ni `classList`, ni `.id =` (un script qui change l'`id`, la classe ou le style écrit
/// d'un élément déplacerait une règle sans que la feuille de style le montre).
///
/// Ne couvre pas : ce que le navigateur fait vraiment de ces règles (seul un navigateur le montre) ; une valeur de `image-rendering` écrite autrement que dans `<style>` (un fichier `.css` : la page n'en a pas, et une ressource de plus serait refusée ailleurs).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_canvas_des_modeles_hexagonaux_n_est_pas_pixelise() {
    let html = e5::lire_page("index.html");
    let regles = regles_css(&css_de_la_page(&html));
    assert!(!regles.is_empty(), "aucune règle CSS lue dans index.html");
    let mut carre_garde_pixelated = false;
    for regle in &regles {
        let sans_lissage = regle.declarations.iter().any(|(propriete, valeur)| {
            propriete == "image-rendering" && SANS_LISSAGE.iter().any(|v| valeur.contains(v))
        });
        if !sans_lissage {
            continue;
        }
        for selecteur in &regle.selecteurs {
            assert!(
                selecteur == "#vitrine" || selecteur == "canvas#vitrine",
                "`image-rendering` sans lissage pour le sélecteur « {selecteur} » : il ne doit viser que #vitrine (le canvas d'un pixel par case)"
            );
            carre_garde_pixelated = true;
        }
    }
    assert!(
        carre_garde_pixelated,
        "#vitrine (un pixel par case : la DLA et le claquage) ne garde pas `image-rendering: pixelated`"
    );
    // Aucune autre règle ne reprend `image-rendering` pour le canvas d'un pixel par case : une règle qui le lisse (`auto`, `smooth`…) ne nomme pas `#vitrine` (une règle écrite plus loin l'emporterait).
    for regle in &regles {
        let lisse = regle.declarations.iter().any(|(propriete, valeur)| {
            propriete == "image-rendering" && !SANS_LISSAGE.iter().any(|v| valeur.contains(v))
        });
        if !lisse {
            continue;
        }
        for selecteur in &regle.selecteurs {
            assert!(
                !selecteur.replace("#vitrine-nette", "").contains("#vitrine"),
                "`image-rendering` avec lissage pour le sélecteur « {selecteur} » : le canvas d'un pixel par case (#vitrine) doit garder pixelated"
            );
        }
    }
    // Ni `!important` sur `image-rendering` (il renverserait l'ordre des règles), ni feuille de style hors de la page (`<link rel="stylesheet">`, `@import`) : tout le CSS est dans les `<style>` lus ici.
    let css = css_de_la_page(&html).to_lowercase();
    for declaration in css.split([';', '{', '}']) {
        assert!(
            !(declaration.contains("image-rendering") && declaration.contains("!important")),
            "`!important` sur image-rendering : «{declaration}»"
        );
    }
    assert!(!css.contains("@import"), "la feuille de style importe une autre feuille (`@import`)");
    for balise in balises(&html) {
        assert!(
            !(nom_de_balise(balise) == "link"
                && attribut(balise, "rel").is_some_and(|rel| rel.to_lowercase().contains("stylesheet"))),
            "une feuille de style hors de la page : <{balise}>"
        );
    }

    for balise in balises(&html) {
        if let Some(style) = attribut(balise, "style") {
            assert!(
                !style.to_lowercase().contains("image-rendering"),
                "un attribut style règle image-rendering : <{balise}>"
            );
        }
    }
    for (nom, texte) in [("app.js", code_de_app_js()), ("lecture.js", code_de_lecture_js())] {
        let minuscules = texte.to_lowercase();
        for interdit in ["image-rendering", "imagerendering", "pixelated", "imagesmoothingenabled"] {
            assert!(!minuscules.contains(interdit), "docs/{nom} écrit « {interdit} »");
        }
        // Un script ne change ni l'`id`, ni la classe, ni l'attribut `style` d'un élément (il déplacerait une règle CSS sans que la feuille de style le montre).
        for interdit in ["csstext", "setproperty", "setattribute", "classname", "classlist", ".id =", ".id="] {
            assert!(
                !minuscules.contains(interdit),
                "docs/{nom} écrit « {interdit} » : un script ne change ni id, ni classe, ni style écrit en texte"
            );
        }
    }
}

/// forme : `app.js` dessine l'image nette sur son canvas (V-V7-32). Il demande `vitrine-nette` et `legende-nette` par `getElementById` (et garde chacun dans une variable) ; il importe `lireImageNette` et `imagesCachees` de `"./lecture.js"` et les appelle avec le
/// bon nombre d'arguments (3 et 1) ; `lecture.js` les déclare (`export function …(`) et n'emploie ni `document` ni `window` ; la fonction `dessiner` (qui reçoit un seul argument) appelle `lireImageNette(m, …, …)`, écrit `.hidden = caches.carree` du canvas
/// d'un pixel par case, `.hidden = caches.nette` du canvas des hexagones et de la légende (la décision de `imagesCachees`, **sans négation**), pose l'image nette sur le canvas des hexagones (`poser(<canvas des hexagones>, `) et, pour tout autre code, l'image d'un
/// pixel par case (`lireImage(m)`) sur l'autre : **mot pour mot, aux espaces près,** `if (nette.code === 0) { poser(<canvas des hexagones>, <son contexte>, nette); } else { poser(<canvas carré>, <son contexte>, lireImage(m)); }` (ajout du juge : une condition inversée,
/// ou l'image d'un pixel par case posée sur le canvas des hexagones, passait) ; `poser` donne au canvas les dimensions de l'image (`.width = largeur`, `.height = hauteur`) **avant** `putImageData(new ImageData(rgba, largeur, hauteur), 0, 0)` (sans cela l'image de la fin,
/// plus grande que celle de l'animation, serait coupée) ; elle lit `NETTE_COTE_FIN` et `NETTE_COTE_ANIMATION` (et les deux `NETTE_SUR_…`). L'animation dessine à la qualité de l'animation et la dernière image plus fine : `dessiner(false)` dans la fonction qui (re)démarre le modèle
/// (celle qui appelle `givre_demarrer(`), `dessiner(joue === 0)` dans celle qui appelle `requestAnimationFrame(` ; ces deux seuls appels.
///
/// Ne couvre pas : que le DOM réagisse ; ce que `lireImageNette` rend (voir `lecture_js_lit_l_image_nette_sur_un_faux_module` et `etape_v7_module.rs`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn app_js_dessine_l_image_nette_sur_son_canvas() {
    let app = code_de_app_js();
    let lecture = code_de_lecture_js();
    let html = e5::lire_page("index.html");
    for id in ["vitrine", "vitrine-nette", "legende-nette"] {
        assert!(app.contains(&format!("getElementById(\"{id}\")")), "app.js ne demande pas l'élément « {id} »");
        assert!(balise_d_id(&html, id).is_some(), "index.html n'a pas d'élément d'id « {id} »");
    }
    let carre = variable_de(&app, "vitrine").expect("app.js garde le canvas d'un pixel par case dans une variable");
    let nette = variable_de(&app, "vitrine-nette").expect("app.js garde le canvas de l'image nette dans une variable");
    let legende = variable_de(&app, "legende-nette").expect("app.js garde la légende dans une variable");
    assert_ne!(carre, nette);

    for (nom, arite) in [("lireImageNette", 3_usize), ("imagesCachees", 1)] {
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

    // La fonction `dessiner`.
    let appel = app.find("lireImageNette(").expect("app.js appelle lireImageNette");
    let (nom, corps) =
        fonction_qui_contient(&app, appel).expect("l'appel de lireImageNette est dans une `function nom(…)`");
    assert_eq!(nom, "dessiner", "lireImageNette est appelée dans « {nom} »");
    assert_eq!(
        arites_des_appels(&app, "dessiner"),
        [1, 1, 1],
        "dessiner : sa déclaration et deux appels, chacun avec un argument"
    );
    assert!(
        corps.contains(
            "lireImageNette(m, fin ? NETTE_COTE_FIN : NETTE_COTE_ANIMATION, fin ? NETTE_SUR_FIN : NETTE_SUR_ANIMATION)"
        ),
        "dessiner n'appelle pas `lireImageNette(m, fin ? NETTE_COTE_FIN : NETTE_COTE_ANIMATION, fin ? NETTE_SUR_FIN : NETTE_SUR_ANIMATION)` (le côté puis le suréchantillonnage, plus fins à la fin)"
    );
    assert!(
        corps.contains("const caches = imagesCachees(nette.code);"),
        "dessiner ne décide pas d'après le code de l'image nette : `const caches = imagesCachees(nette.code);`"
    );
    assert!(
        corps.contains(&format!("{carre}.hidden = caches.carree")),
        "dessiner n'écrit pas `{carre}.hidden = caches.carree` (sans négation)"
    );
    assert!(
        corps.contains(&format!("{nette}.hidden = caches.nette")),
        "dessiner n'écrit pas `{nette}.hidden = caches.nette` (sans négation)"
    );
    assert!(
        corps.contains(&format!("{legende}.hidden = caches.nette")),
        "dessiner n'écrit pas `{legende}.hidden = caches.nette` (sans négation)"
    );
    assert!(
        corps.contains(&format!("poser({nette}, ")),
        "dessiner ne pose pas l'image nette sur le canvas des hexagones : `poser({nette}, …)`"
    );
    assert!(
        corps.contains(&format!("poser({carre}, ")),
        "dessiner ne pose pas l'image d'un pixel par case sur l'autre canvas : `poser({carre}, …)`"
    );
    assert!(corps.contains("lireImage(m)"), "dessiner ne lit pas l'image d'un pixel par case pour les autres modèles");
    // Le choix, mot pour mot (aux espaces près) : l'image nette sur le canvas des hexagones quand le code est 0, sinon l'image d'un pixel par case sur l'autre ; chacune avec le contexte de SON canvas.
    let contexte_de = |variable: &str| -> String {
        let appel = format!("= {variable}.getContext(\"2d\")");
        let position =
            app.find(&appel).unwrap_or_else(|| panic!("app.js ne demande pas le contexte 2d de `{variable}`"));
        let avant = app[..position].trim_end();
        let nom: String = avant.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
        nom.chars().rev().collect()
    };
    let (contexte_carre, contexte_nette) = (contexte_de(&carre), contexte_de(&nette));
    assert_ne!(contexte_carre, contexte_nette);
    let sans_espaces = |texte: &str| texte.split_whitespace().collect::<Vec<_>>().join(" ");
    let choix = format!(
        "if (nette.code === 0) {{ poser({nette}, {contexte_nette}, nette); }} else {{ poser({carre}, {contexte_carre}, lireImage(m)); }}"
    );
    assert!(
        sans_espaces(&corps).contains(&choix),
        "dessiner ne choisit pas l'image comme la consigne le dit : `{choix}` (l'image nette, et elle seule, sur le canvas des hexagones quand le code est 0)"
    );
    assert_eq!(
        arites_des_appels(&corps, "poser"),
        [3, 3],
        "dessiner pose une image, deux fois écrit (une par branche)"
    );
    assert_eq!(
        corps.matches("lireImage(m)").count(),
        1,
        "dessiner lit l'image d'un pixel par case une seule fois, dans la branche des modèles carrés"
    );
    // `poser` donne au canvas les dimensions de l'image AVANT de la poser : sans cela l'image de la fin (plus grande) serait coupée, et redimensionner après effacerait le dessin.
    let debut_de_poser = app.find("function poser(").expect("app.js déclare `function poser(`");
    let ouvrante = debut_de_poser + app[debut_de_poser..].find(") {").expect("le corps de poser") + 2;
    let corps_de_poser = &app[ouvrante..=fermeture(&app, ouvrante)];
    let (largeur_posee, hauteur_posee, dessin) = (
        corps_de_poser.find(".width = largeur"),
        corps_de_poser.find(".height = hauteur"),
        corps_de_poser.find("putImageData("),
    );
    assert!(
        largeur_posee.is_some() && hauteur_posee.is_some(),
        "poser ne donne pas au canvas les dimensions de l'image (`.width = largeur`, `.height = hauteur`)"
    );
    assert!(dessin.is_some(), "poser ne pose pas l'image (`putImageData(`)");
    assert!(
        largeur_posee < dessin && hauteur_posee < dessin,
        "poser redimensionne le canvas après y avoir posé l'image : le dessin serait effacé"
    );
    assert!(
        corps_de_poser.contains("putImageData(new ImageData(rgba, largeur, hauteur), 0, 0)"),
        "poser ne pose pas l'image entière à l'origine : `putImageData(new ImageData(rgba, largeur, hauteur), 0, 0)`"
    );
    for constante in ["NETTE_COTE_FIN", "NETTE_COTE_ANIMATION", "NETTE_SUR_FIN", "NETTE_SUR_ANIMATION"] {
        assert!(corps.contains(constante), "dessiner n'emploie pas {constante}");
    }
    // Le dessin d'une image plus fine à l'arrêt, plus léger pendant l'animation.
    let demarre = app.find("givre_demarrer(").expect("app.js démarre un modèle");
    let (_, corps_du_demarrage) =
        fonction_qui_contient(&app, demarre).expect("givre_demarrer est appelée dans une `function nom(…)`");
    assert!(
        corps_du_demarrage.contains("dessiner(false)"),
        "la fonction qui démarre le modèle ne dessine pas à la qualité de l'animation : `dessiner(false)`"
    );
    let animation = app.find("requestAnimationFrame(").expect("app.js anime par requestAnimationFrame");
    let (_, corps_de_l_animation) =
        fonction_qui_contient(&app, animation).expect("requestAnimationFrame est appelée dans une `function nom(…)`");
    assert!(
        corps_de_l_animation.contains("dessiner(joue === 0)"),
        "l'image de l'animation ne dessine pas plus fin quand le scénario s'arrête : `dessiner(joue === 0)`"
    );
}

/// forme : les réglages de l'image nette de la page sont dans les bornes du module et ont la forme de la doc (V-V7-33). `app.js` déclare quatre constantes entières, `const NETTE_COTE_ANIMATION`, `NETTE_SUR_ANIMATION`, `NETTE_COTE_FIN` et
/// `NETTE_SUR_FIN`, chacune employée ailleurs que dans sa déclaration ; chaque côté est **pair**, de 2 à 2048, chaque `sur` de 1 à 4 et chaque produit côté × `sur` d'au plus 4096 (sinon `givre_nette_preparer` refuse et la page retomberait sur l'image d'un pixel
/// par case) ; l'image de l'animation a **au moins 256 pixels de côté et 2 × 2 échantillons par pixel** (ajout du juge : plus petite, le navigateur l'agrandirait plus de trois fois ; à `sur = 1` les bords sont crénelés ; mesuré sous Node par le pilote : `(384, 2)` coûte
/// 26 ms à préparer et 28 ms avec la lecture) et tient en **1024 échantillons de côté au plus** (un million d'échantillons : de l'ordre de 20 ms en natif, `MODELE.md` étape V7) et celle de la fin est **strictement plus fine** (plus d'échantillons de côté, plus de pixels et au moins autant d'échantillons par pixel) que celle de l'animation.
///
/// Ne couvre pas : le temps réel d'une image dans le navigateur (mesuré sous Node par le pilote).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn les_reglages_de_l_image_nette_sont_dans_les_bornes() {
    let app = code_de_app_js();
    let lire = |nom: &str| -> u32 {
        let valeur =
            constante_entiere(&app, nom).unwrap_or_else(|| panic!("app.js n'a pas de `const {nom} = <entier>`"));
        assert!(app.matches(nom).count() >= 2, "{nom} n'est pas employée");
        u32::try_from(valeur).unwrap_or(u32::MAX)
    };
    let (cote_animation, sur_animation) = (lire("NETTE_COTE_ANIMATION"), lire("NETTE_SUR_ANIMATION"));
    let (cote_fin, sur_fin) = (lire("NETTE_COTE_FIN"), lire("NETTE_SUR_FIN"));
    for (nom, cote, sur) in [("l'animation", cote_animation, sur_animation), ("la fin", cote_fin, sur_fin)] {
        assert!(
            (2..=COTE_MAX).contains(&cote) && cote.is_multiple_of(2),
            "le côté de {nom} : {cote} (pair, de 2 à {COTE_MAX})"
        );
        assert!((1..=SUR_MAX).contains(&sur), "le suréchantillonnage de {nom} : {sur} (de 1 à {SUR_MAX})");
        assert!(cote * sur <= GRILLE_MAX, "côté × sur de {nom} : {} (au plus {GRILLE_MAX})", cote * sur);
    }
    assert!(
        cote_animation * sur_animation <= 1024,
        "l'image de l'animation a {} échantillons de côté : 1024 au plus",
        cote_animation * sur_animation
    );
    // Un plancher (ajout du juge) : une image d'animation trop petite serait agrandie plus de trois fois par le navigateur (floue), et sans suréchantillonnage ses bords seraient crénelés.
    assert!(cote_animation >= 256, "l'image de l'animation n'a que {cote_animation} pixels de côté : 256 au moins");
    assert!(
        sur_animation >= 2,
        "l'image de l'animation n'a que {sur_animation} × {sur_animation} échantillon(s) par pixel : 2 × 2 au moins"
    );
    assert!(
        cote_fin * sur_fin > cote_animation * sur_animation,
        "l'image de la fin n'est pas plus fine que celle de l'animation"
    );
    assert!(
        cote_fin > cote_animation && sur_fin >= sur_animation,
        "l'image de la fin a plus de pixels et au moins autant d'échantillons par pixel que celle de l'animation : ({cote_fin}, {sur_fin}) contre ({cote_animation}, {sur_animation})"
    );
}

/// juste : `lireImageNette` et `imagesCachees` de `lecture.js`, sur un **faux module** écrit dans le script de Node (`tests/etape_v7/lecture-pure.mjs`), disent ce que la doc dit (V-V7-34). Le faux module est une image **non carrée** (`cote` de large,
/// `cote + 2` de haut) dont la couleur du pixel `(x, y)` est `((3x + 5y + sur) & 255, (7x + 11y) & 255, (13x + 17y + cote) & 255)` d'après le dernier `givre_nette_preparer(cote, sur)` : il change d'état à chaque préparation. `lireImageNette(faux, 6, 2)` rend
/// le code 0, `6x8` pixels et les octets RGBA du calcul à la main (R, V, B puis 255, ligne par ligne de haut en bas : un `(x, y)` lu en `(y, x)`, une largeur et une hauteur échangées, un canal permuté ou l'alpha oublié se voient), un `Uint8ClampedArray` ;
/// `lireImageNette(faux, 4, 3)` sur le même faux module rend l'image de **ces** réglages (la préparation est demandée dans l'ordre `(cote, sur)`, avant toute lecture : les appels sont `preparer, largeur, hauteur`) ; un côté impair est refusé (code 3, `0x0`, aucun
/// pixel lu, un tableau vide) ; `imagesCachees(0)` cache le canvas d'un pixel par case seulement (`carree` vrai, `nette` faux) et tout autre code (1, 2, 3, 6, `u32::MAX`) cache celui des hexagones.
///
/// Ne couvre pas : le vrai module (voir `etape_v7_module.rs`) ; un `>>> 0` oublié sur un pixel de l'image (la couleur `0x00RRGGBB` est toujours positive : cette faute-là ne change rien) ; ce que `app.js` fait de ce que `lecture.js` rend.
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn lecture_js_lit_l_image_nette_sur_un_faux_module() {
    if !e5::node_est_la() {
        return;
    }
    let lecture = chemin_de_lecture();
    let sortie = lancer_le_script("lecture-pure.mjs", &[lecture.as_str()]);
    let erreur = String::from_utf8_lossy(&sortie.stderr);
    assert!(sortie.status.success(), "lecture-pure.mjs : {:?}, sortie d'erreur : {erreur}", sortie.status);
    assert_eq!(String::from_utf8_lossy(&sortie.stdout), attendu_de_la_lecture_pure());

    for arguments in [&[][..], &["a", "b"], &["n-existe-pas.js"]] {
        let sortie = lancer_le_script("lecture-pure.mjs", arguments);
        assert_eq!(sortie.status.code(), Some(1), "lecture-pure.mjs {arguments:?}");
        assert!(sortie.stdout.is_empty(), "lecture-pure.mjs {arguments:?} : la sortie standard n'est pas vide");
    }
    // Le script est du JavaScript valide (modules) : `node --check`.
    let script = std::fs::read_to_string(e5::racine().join("tests").join("etape_v7").join("lecture-pure.mjs"))
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
    assert!(sortie.status.success(), "lecture-pure.mjs n'est pas valide : {}", String::from_utf8_lossy(&sortie.stderr));
}

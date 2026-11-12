//! Étape V7 : la page (`docs/index.html`, `docs/app.js`, `docs/lecture.js`) montre l'image nette des flocons de Reiter, sans `image-rendering: pixelated`. Valeurs attendues : `references/etape-v7-valeurs.md`, identifiants `V-V7-xx`.
//! Comme aux étapes 5, Z1 et D3, presque tous les tests sont des **tests de forme** (ils lisent les fichiers comme du texte) ; un seul fait tourner du code de la page, sous Node :
//! `lecture_js_lit_l_image_nette_sur_un_faux_module` appelle `lireImageNette` et `imagesCachees` de `lecture.js` sur un faux module qui change d'état à chaque préparation et compare à un calcul fait à la main ; il ne lit pas
//! `docs/givre.wasm` et passe dès que `lecture.js` est écrit. La lecture du vrai module est éprouvée dans `etape_v7_module.rs`.
//!
//! **Étape V8 : la page est tous les fichiers de son dossier** (voir `etape_5_page.rs`) : ces tests lisent tout le CSS (`<style>`, feuilles) et tout le JavaScript de la page ; ils ne figent plus l'écriture de `dessiner`, de `poser` ni de leurs constantes.
//!
//! # Ce que fait la page (la consigne des mains qui l'écrivent : celle de V7 ; V8 en a retiré ce qui n'était que l'écriture d'`app.js`)
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
    COTE_MAX, GRILLE_MAX, SUR_MAX, arites_des_appels, attendu_de_la_lecture_pure, attribut, balise_d_id, balises,
    chemin_de_lecture, code_de_app_js, code_de_lecture_js, constante_entiere, css_de_toute_la_page, e5,
    lancer_le_script, nom_de_balise, regles_css,
};

/// Les valeurs de `image-rendering` qui ne lissent pas : un agrandissement en gros carrés (ou presque).
const SANS_LISSAGE: [&str; 5] =
    ["pixelated", "crisp-edges", "-moz-crisp-edges", "optimize-contrast", "-webkit-optimize-contrast"];

/// forme : `index.html` a les deux canvas et la légende de l'image nette (V-V7-30). `<canvas id="vitrine">` (un pixel par case) et `<canvas id="vitrine-nette">` (l'image nette : les modèles sur la grille hexagonale) existent, chacun un `<canvas>`, **à n'importe quelle place** ; l'élément `legende-nette` existe (de n'importe quelle sorte).
/// **V8 ne lit plus** : l'état d'ouverture (`vitrine` non caché et `vitrine-nette` caché au départ : la page rhabillée ouvre sur la neige), les dimensions écrites du canvas, le mot « hexagone » de la légende (la légende peut être écrite par un script), ni que les deux canvas se suivent dans le même cadre.
///
/// Ne couvre pas : que le canvas soit montré ou caché au bon moment (le DOM n'est pas éprouvé).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn la_page_a_les_deux_canvas_et_la_legende() {
    let html = e5::lire_page("index.html");
    for id in ["vitrine", "vitrine-nette"] {
        let balise = balise_d_id(&html, id).unwrap_or_else(|| panic!("pas d'élément d'id « {id} »"));
        assert_eq!(nom_de_balise(balise), "canvas", "l'élément d'id « {id} » n'est pas un <canvas>");
    }
    assert!(balise_d_id(&html, "legende-nette").is_some(), "pas d'élément d'id « legende-nette »");
}

/// forme : **aucun canvas qui montre un flocon ou une neige n'est pixélisé** (V-V7-31, étendu par V8). Les règles CSS de **toute la page** sont lues (les `<style>` de chaque fichier HTML et SVG, chaque feuille `.css` : sans commentaires, les blocs `@media`, `@supports`, `@layer` et `@container` et l'imbrication CSS compris) : chaque règle qui
/// donne à `image-rendering` une valeur qui ne lisse pas (`pixelated`, `crisp-edges`, `-moz-crisp-edges`, `optimize-contrast`, `-webkit-optimize-contrast`) a pour sélecteurs **exactement** `#vitrine` ou `canvas#vitrine` et rien d'autre : un sélecteur général (`canvas`, `*`, `.cadre canvas`, une classe, `#vitrine-nette`, le canvas de
/// n'importe quel panneau) est refusé, même s'il ne vise « que » le bon canvas aujourd'hui. **V8 ne demande plus qu'il en reste une** (`#vitrine` peut ne plus être pixélisé : la DLA et le claquage peuvent être dessinés lisses). Aucun attribut `style` d'aucun fichier HTML ou SVG ne règle `image-rendering`, `image-rendering` ne porte
/// jamais `!important`, et **aucun fichier JavaScript** (tous, scripts en ligne compris) n'écrit, hors de ses commentaires, `image-rendering`, `imageRendering` ni `pixelated`, ni `imageSmoothingEnabled` autrement que `= true` (un style posé par un script ferait la même faute que la feuille de style), ni ne change l'`id` d'un élément (`.id =`,
/// `setAttribute("id"`) : un script qui rebaptiserait un canvas `vitrine` déplacerait la règle sans que la feuille de style le montre. V8 **n'interdit plus** `classList`, `className`, `setAttribute` (hors `id`), `setProperty` ni `cssText` (un tiroir en a besoin) : le mot `image-rendering`, lui, reste interdit dans tout script.
///
/// Ne couvre pas : ce que le navigateur fait vraiment de ces règles (seul un navigateur le montre) ; une règle de `image-rendering` écrite dans un `@media` imbriqué dans une règle ; une valeur composée par un script (`"image-" + "rendering"`) ; un canvas hors de la page.
/// Fiche : V-V8-35 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn le_canvas_des_modeles_hexagonaux_n_est_pas_pixelise() {
    let regles = regles_css(&css_de_toute_la_page());
    assert!(!regles.is_empty(), "aucune règle CSS lue dans la page");
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
        }
    }
    // Ni `!important` sur `image-rendering` (il renverserait l'ordre des règles).
    for declaration in css_de_toute_la_page().to_lowercase().split([';', '{', '}']) {
        assert!(
            !(declaration.contains("image-rendering") && declaration.contains("!important")),
            "`!important` sur image-rendering : «{declaration}»"
        );
    }
    // Aucun attribut `style` d'aucun fichier HTML ou SVG ne règle `image-rendering`.
    for (nom, texte) in e5::textes_des_fichiers(&["html", "htm", "svg"]) {
        for balise in balises(&e5::sans_donnees(&texte)) {
            if let Some(style) = attribut(balise, "style") {
                assert!(
                    !style.to_lowercase().contains("image-rendering"),
                    "{nom} : un attribut style règle image-rendering : <{balise}>"
                );
            }
        }
    }
    // Les scripts : aucun fichier ne touche à `image-rendering`, ni à l'`id` d'un élément.
    for (nom, code) in e5::code_js_de_la_page() {
        let minuscules = code.to_lowercase();
        let sans_lissage_vrai =
            minuscules.replace("imagesmoothingenabled = true", "").replace("imagesmoothingenabled=true", "");
        for interdit in ["image-rendering", "imagerendering", "pixelated", "imagesmoothingenabled"] {
            assert!(!sans_lissage_vrai.contains(interdit), "{nom} écrit « {interdit} »");
        }
        for interdit in ["setattribute(\"id\"", "setattribute('id'", "setattribute(`id`"] {
            assert!(
                !minuscules.contains(interdit),
                "{nom} écrit « {interdit} » : un script ne change pas l'id d'un élément"
            );
        }
        // Une affectation `.id = …` qui nomme `vitrine` (un canvas rebaptisé recevrait la règle de `#vitrine`). Une comparaison (`message.id === attendu`) ou le numéro d'une requête d'un worker (`requete.id = n`) ne sont pas un changement d'`id`.
        for (position, _) in minuscules.match_indices(".id") {
            let apres = minuscules[position + 3..].trim_start();
            let Some(valeur) = apres.strip_prefix('=').filter(|v| !v.starts_with('=')) else { continue };
            let instruction = &valeur[..valeur.find([';', '\n']).unwrap_or(valeur.len())];
            assert!(
                !instruction.contains("vitrine"),
                "{nom} écrit « .id ={instruction} » : un script ne donne pas l'id d'un canvas de la vitrine à un élément"
            );
        }
    }
}

/// forme : `lecture.js` déclare `export function lireImageNette(` et `imagesCachees(` et n'emploie ni `document` ni `window` (Node le charge sans navigateur) ; **chaque appel** de `lireImageNette(` de la page (tous les fichiers JavaScript) a trois arguments et chaque appel de `imagesCachees(` un seul (V-V7-32). **V8 ne lit plus** `dessiner` ni `poser` mot pour mot
/// (« `if (nette.code === 0) { poser(…) } else { … }` », les `.hidden = caches.…` sans négation, `.width = largeur` avant `putImageData`, `dessiner(false)` au démarrage et `dessiner(joue === 0)` à l'image) : ces fonctions sont l'écriture d'`app.js`, que la page rhabillée remplace (le dessin passe par un worker). Que l'image nette n'apparaisse
/// que sur le canvas des hexagones pour le code 0 et l'image d'un pixel par case sur l'autre, **sans résidu**, ne se voit plus ici : voir `MODELE.md`, étape V8.
///
/// Ne couvre pas : ce que `lireImageNette` rend (voir `lecture_js_lit_l_image_nette_sur_un_faux_module` et `etape_v7_module.rs`) ; ce que la page fait de ce que `lecture.js` rend.
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn lecture_js_declare_l_image_nette_et_la_page_l_appelle_bien() {
    let app = code_de_app_js();
    let lecture = code_de_lecture_js();
    for (nom, arite) in [("lireImageNette", 3_usize), ("imagesCachees", 1)] {
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
}

/// forme : **les réglages de l'image nette que la page demande sont dans les bornes du module** (V-V7-33). **Chaque appel** de `givre_nette_preparer(cote, sur)` ou de `lireImageNette(m, cote, sur)` du code de la page dont `cote` ou `sur` est un entier écrit les respecte : `cote` pair de 2 à 2048, `sur` de 1 à 4, `cote × sur` d'au plus 4096 (sinon le
/// module refuse par un code et la page retombe sur l'image d'un pixel par case ou reste vide : le défaut de la planche vide, un côté impair) ; **si** les constantes `NETTE_COTE_ANIMATION`, `NETTE_SUR_ANIMATION`, `NETTE_COTE_FIN` et `NETTE_SUR_FIN` sont déclarées (`const … = <entier>`), chacune est dans ces bornes. **V8 ne demande plus** : ces quatre noms, qu'elles
/// soient employées, que l'image de l'animation ait au moins 256 pixels de côté et 2 × 2 échantillons et au plus 1024 échantillons de côté, ni que celle de la fin soit plus fine (des choix de la page, jugés à l'œil ; voir `MODELE.md`, étape V8) ; un côté calculé (`Math.round(largeur × dpr)`) ne se lit pas ici : c'est à lui d'être pair.
///
/// Ne couvre pas : le temps réel d'une image dans le navigateur ; un côté ou un `sur` qui n'est pas écrit en entier.
/// Fiche : V-V8-38 (`references/etape-v8-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-v7"), ignore)]
fn les_reglages_de_l_image_nette_sont_dans_les_bornes() {
    let app = code_de_app_js();
    let dans_les_bornes = |nom: &str, cote: Option<u32>, sur: Option<u32>| {
        if let Some(cote) = cote {
            assert!(
                (2..=COTE_MAX).contains(&cote) && cote.is_multiple_of(2),
                "{nom} : le côté {cote} (pair, de 2 à {COTE_MAX})"
            );
        }
        if let Some(sur) = sur {
            assert!((1..=SUR_MAX).contains(&sur), "{nom} : le suréchantillonnage {sur} (de 1 à {SUR_MAX})");
        }
        if let (Some(cote), Some(sur)) = (cote, sur) {
            assert!(cote * sur <= GRILLE_MAX, "{nom} : côté × sur = {} (au plus {GRILLE_MAX})", cote * sur);
        }
    };
    // Les appels : les arguments écrits en entiers.
    for (appel, decalage) in [("givre_nette_preparer(", 0_usize), ("lireImageNette(", 1)] {
        for (position, _) in app.match_indices(appel) {
            if app[..position].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let reste = &app[position + appel.len()..];
            let (mut profondeur, mut arguments, mut courant) = (0_i32, Vec::<String>::new(), String::new());
            for c in reste.chars() {
                match c {
                    '(' | '[' | '{' => profondeur += 1,
                    ')' | ']' | '}' if profondeur == 0 => break,
                    ')' | ']' | '}' => profondeur -= 1,
                    ',' if profondeur == 0 => {
                        arguments.push(std::mem::take(&mut courant));
                        continue;
                    }
                    _ => {}
                }
                courant.push(c);
            }
            arguments.push(courant);
            let entier = |i: usize| arguments.get(decalage + i).and_then(|a| a.trim().parse::<u32>().ok());
            dans_les_bornes(appel, entier(0), entier(1));
        }
    }
    // Les constantes, si la page en a.
    let lire = |nom: &str| constante_entiere(&app, nom).map(|v| u32::try_from(v).unwrap_or(u32::MAX));
    dans_les_bornes("l'animation", lire("NETTE_COTE_ANIMATION"), lire("NETTE_SUR_ANIMATION"));
    dans_les_bornes("la fin", lire("NETTE_COTE_FIN"), lire("NETTE_SUR_FIN"));
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

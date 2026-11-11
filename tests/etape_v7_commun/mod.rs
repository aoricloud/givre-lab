//! Ce que les fichiers de test de l'étape V7 (l'image nette de la vitrine) partagent : la table des trois flocons sur la grille hexagonale de la doc de `vitrine` (« L'image nette »),
//! **écrite ici à la main** (jamais lue dans la bibliothèque), le champ dessiné de la doc **recalculé à la main** d'après les masses d'un `Flocon` construit directement par la
//! bibliothèque (jamais par `vitrine`), l'image que `rendu_hex::rendre` en tire, les chaînes que les scripts de Node doivent écrire et le lancement de Node.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Command, Output};

use givre_lab::empreinte::sha256_hex;
use givre_lab::flocon::{Flocon, Parametres};
use givre_lab::rendu_hex::{ReglagesImage, rendre};

/// Les aides de l'étape D3 (la table des modèles de claquage et leur état natif), qui reprennent celles de l'étape 5 (`e5` : la table des quatre premiers modèles, `natif`, `node_est_la`,
/// `chemin_module`, `lire_page`, `racine`) et de l'étape D2.
#[path = "../etape_d3_commun/mod.rs"]
pub mod d3;

pub use d3::e5;

// ---------------------------------------------------------------------------------------------------------------
// La table de la doc de `vitrine` : « L'image nette », « Le champ dessiné ».
// ---------------------------------------------------------------------------------------------------------------

/// Un modèle sur la grille hexagonale (un flocon de Reiter) et ce que la doc en dit pour l'image nette.
pub struct ModeleNet {
    /// Le numéro passé à `givre_demarrer` (1, 2 ou 3).
    pub numero: u32,
    /// Le rayon de l'hexagone : 30, 40 ou 60 (aussi le rayon de la vue).
    pub rayon: u32,
    /// La durée du scénario.
    pub duree: u32,
    /// Les trois paramètres du flocon, écrits comme dans le scénario.
    pub alpha: &'static str,
    pub beta: &'static str,
    pub gamma: &'static str,
    /// `E = γ.brut × durée`, **écrit à la main** d'après la doc : `214 748 365 × 400`, `429 496 730 × 120` et `4 294 967 × 800`.
    pub e: i64,
}

/// Les trois modèles de flocon : la plaque, l'étoile, la dendrite (V-V7-01).
pub const MODELES_NETS: [ModeleNet; 3] = [
    ModeleNet { numero: 1, rayon: 30, duree: 400, alpha: "1", beta: "0", gamma: "0.05", e: 85_899_346_000 },
    ModeleNet { numero: 2, rayon: 40, duree: 120, alpha: "1", beta: "0.45", gamma: "0.1", e: 51_539_607_600 },
    ModeleNet { numero: 3, rayon: 60, duree: 800, alpha: "1", beta: "0.4", gamma: "0.001", e: 3_435_973_600 },
];

/// Les bornes de la doc : `cote` pair de 2 à 2048, `sur` de 1 à 4, `cote × sur` au plus 4096 (V-V7-03).
pub const COTE_MAX: u32 = 2048;
pub const SUR_MAX: u32 = 4;
pub const GRILLE_MAX: u32 = 4096;

/// `UN = 2^32`, le `brut` de la masse 1.
pub const UN: i64 = 1 << 32;

/// L'octet R, V, B de `u32::MAX` hors image : la valeur « hors du tampon ».
pub const HORS: u32 = u32::MAX;

/// Le champ de la doc, **recalculé à la main** (V-V7-02) : `0` si la masse est inférieure à 1 (`brut < UN`), sinon `3 × (brut − UN) + E`. `masses` sont les `brut` dans l'ordre canonique.
pub fn champ_de_la_doc(masses: &[i64], e: i64) -> Vec<i64> {
    masses.iter().map(|&brut| if brut < UN { 0 } else { 3 * (brut - UN) + e }).collect()
}

/// Le flocon du modèle `indice` (0 pour le modèle 1, 1 pour le modèle 2, 2 pour le modèle 3), construit par la bibliothèque seule et avancé de `pas` pas (au plus la durée).
pub fn flocon_du_modele(indice: usize, pas: u32) -> Flocon {
    let modele = &MODELES_NETS[indice];
    let parametres = Parametres::lire(modele.alpha, modele.beta, modele.gamma).expect("réglages valides");
    let mut flocon = Flocon::new(modele.rayon, parametres);
    flocon.avancer(u64::from(pas.min(modele.duree)));
    flocon
}

/// Le champ de la doc du modèle `indice` après `pas` pas : une valeur par case, dans l'ordre canonique.
pub fn champ_du_modele(indice: usize, pas: u32) -> Vec<i64> {
    let masses: Vec<i64> = flocon_du_modele(indice, pas).masses().iter().map(|masse| masse.0).collect();
    champ_de_la_doc(&masses, MODELES_NETS[indice].e)
}

/// L'image nette attendue (les octets R, V, B, `3 × cote²`) : `rendu_hex::rendre` appliqué au champ de la doc, avec la vue de rayon `R`, `plein = 4 E`.
pub fn image_attendue(indice: usize, pas: u32, cote: u32, sur: u32) -> Vec<u8> {
    image_du_champ(indice, &champ_du_modele(indice, pas), cote, sur)
}

/// Comme `image_attendue`, pour un champ déjà calculé (`champ_du_modele`) : évite de refaire jouer le flocon pour chaque réglage.
pub fn image_du_champ(indice: usize, champ: &[i64], cote: u32, sur: u32) -> Vec<u8> {
    let modele = &MODELES_NETS[indice];
    let reglages = ReglagesImage { cote, rayon_vue: modele.rayon, sur, plein: 4 * modele.e };
    rendre(modele.rayon, champ, &reglages).expect("un rendu valide")
}

/// Les octets RGBA d'une image RVB (le canal alpha à 255), comme `lireImage` et `lireImageNette` de `lecture.js` les rangent.
pub fn rgba_de(rvb: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(rvb.len() / 3 * 4);
    for pixel in rvb.chunks(3) {
        rgba.extend_from_slice(pixel);
        rgba.push(255);
    }
    rgba
}

/// Le pixel `(x, y)` d'une image RVB carrée de côté `cote`, empaqueté en `0x00RRGGBB`.
pub fn pixel_de(rvb: &[u8], cote: u32, x: u32, y: u32) -> u32 {
    let debut = 3 * (y as usize * cote as usize + x as usize);
    (u32::from(rvb[debut]) << 16) | (u32::from(rvb[debut + 1]) << 8) | u32::from(rvb[debut + 2])
}

// ---------------------------------------------------------------------------------------------------------------
// Node.
// ---------------------------------------------------------------------------------------------------------------

pub fn racine() -> PathBuf {
    e5::racine().to_path_buf()
}

/// Lance un script de `tests/etape_v7/` avec ces arguments.
pub fn lancer_le_script(script: &str, arguments: &[&str]) -> Output {
    Command::new("node")
        .arg(racine().join("tests").join("etape_v7").join(script))
        .args(arguments)
        .output()
        .unwrap_or_else(|erreur| panic!("le lancement de Node échoue : {erreur}"))
}

/// Le chemin du module commité et de `lecture.js`, en texte.
pub fn chemin_du_module() -> String {
    e5::chemin_module().to_str().expect("chemin en UTF-8").to_string()
}

pub fn chemin_de_lecture() -> String {
    racine().join("docs").join("lecture.js").to_str().expect("chemin en UTF-8").to_string()
}

// ---------------------------------------------------------------------------------------------------------------
// Ce que `nette-wasm.mjs` doit écrire (son en-tête dit chaque ligne).
// ---------------------------------------------------------------------------------------------------------------

/// L'état d'un modèle quelconque (0 à 7) après `pas` pas, tel que le code natif le donne : `(temps, durée, sites, rayon, empreinte, largeur, hauteur, octets RVB de l'image d'un pixel par case)`.
fn etat_natif(modele: u32, graine: u32, pas: u32) -> (u32, u32, u32, u32, String, u32, u32, Vec<u8>) {
    if (4..=7).contains(&modele) {
        let natif = d3::natif_claquage(modele - 4, graine, pas);
        (natif.temps, natif.duree, natif.sites, natif.rayon, natif.empreinte, natif.largeur, natif.hauteur, natif.rvb)
    } else {
        let natif = e5::natif(modele, graine, pas);
        (natif.temps, natif.duree, natif.sites, natif.rayon, natif.empreinte, natif.largeur, natif.hauteur, natif.rvb)
    }
}

/// La ligne `etat_avant` / `etat_apres` du script.
fn ligne_d_etat(modele: u32, graine: u32, pas: u32) -> String {
    let (temps, duree, sites, rayon, empreinte, largeur, hauteur, rvb) = etat_natif(modele, graine, pas);
    format!("{temps},{duree},{sites},{rayon},{empreinte},{largeur}x{hauteur},{}", sha256_hex(&rgba_de(&rvb)))
}

/// Ce que `nette-wasm.mjs` doit écrire pour le modèle `modele` (0 à 7), la graine, `pas` unités d'un seul appel, et l'image de `cote` pixels à `sur × sur` échantillons : l'état et l'image du code
/// natif, lus comme le script les lit. Un modèle sur une grille carrée (0, 4 à 7) n'a pas d'image nette : le code 1, rien dans le tampon.
pub fn attendu_du_script_nette(modele: u32, graine: u32, pas: u32, cote: u32, sur: u32) -> String {
    let (temps, duree, ..) = etat_natif(modele, graine, pas);
    let etat = ligne_d_etat(modele, graine, pas);
    let joue = temps.min(duree);
    let hexagonal = (1..=3).contains(&modele);
    let vide_sha = sha256_hex(&[]);
    let max = HORS;
    // `avancer(1)` joue une unité si le scénario n'est pas fini.
    let jouee = u32::from(temps < duree);
    let (code, dimensions, rgba, pixels, efface) = if hexagonal {
        let image = image_attendue(modele as usize - 1, pas, cote, sur);
        let efface = if jouee == 1 {
            format!("1,0,0,{max}")
        } else {
            format!("0,{cote},{cote},{}", pixel_de(&image, cote, 0, 0))
        };
        (0, format!("{cote}x{cote}"), sha256_hex(&rgba_de(&image)), sha256_hex(&image), efface)
    } else {
        (1, "0x0".to_string(), vide_sha.clone(), vide_sha, format!("{jouee},0,0,{max}"))
    };
    let refus = if hexagonal { "3,3,3,3,3,3,4,4,4,5,5" } else { "1,1,1,1,1,1,1,1,1,1,1" };
    format!(
        "avant=2,0,0,{max}\ndemarre=1\njoue={joue}\netat_avant={etat}\nvide=0,0,{max}\ncode={code}\ndimensions={dimensions}\nrgba={rgba}\npixels={pixels}\n\
         hors={max},{max},{max},{max},{max}\netat_apres={etat}\nrefus={refus}\napres_refus=0,0,{max}\nefface={efface}\nredemarre=1,0,0,{max}\n"
    )
}

/// Ce que `lecture-pure.mjs` doit écrire : le faux module (une image de `cote` de large et `cote + 2` de haut, dont la couleur du pixel `(x, y)` est
/// `((3x + 5y + sur) & 255, (7x + 11y) & 255, (13x + 17y + cote) & 255)`), lu deux fois (côtés 6 puis 4, `sur` 2 puis 3), puis refusé pour un côté impair.
pub fn attendu_de_la_lecture_pure() -> String {
    let image = |cote: u32, sur: u32| {
        let hauteur = cote + 2;
        let mut rgba = Vec::new();
        for y in 0..hauteur {
            for x in 0..cote {
                rgba.push(((3 * x + 5 * y + sur) & 255) as u8);
                rgba.push(((7 * x + 11 * y) & 255) as u8);
                rgba.push(((13 * x + 17 * y + cote) & 255) as u8);
                rgba.push(255);
            }
        }
        format!("0,{cote}x{hauteur},{},Uint8ClampedArray", sha256_hex(&rgba))
    };
    format!(
        "premier={}\nsecond={}\nrefus=3,0x0,0,0\nappels=preparer(6,2),largeur,hauteur,preparer(4,3),largeur,hauteur,preparer(5,1)\n\
         cachees=0:true,false|1:false,true|2:false,true|3:false,true|6:false,true|4294967295:false,true\n",
        image(6, 2),
        image(4, 3)
    )
}

// ---------------------------------------------------------------------------------------------------------------
// Lecture du texte de la page (les mêmes aides que `etape_d3_page.rs`).
// ---------------------------------------------------------------------------------------------------------------

/// Le code d'un script sans ses commentaires `//` et `/* */` (hors des chaînes `"…"`, `'…'` et `` `…` ``, gardées telles quelles).
pub fn sans_commentaires(script: &str) -> String {
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

pub fn code_de_app_js() -> String {
    sans_commentaires(&e5::lire_page("app.js"))
}

pub fn code_de_lecture_js() -> String {
    sans_commentaires(&e5::lire_page("lecture.js"))
}

/// Pour chaque appel `nom(…)` du texte, le nombre d'arguments (les virgules de premier niveau plus un, une virgule finale ne comptant pas, ou 0 si les parenthèses sont vides). Un `nom`
/// précédé d'une lettre, d'un chiffre ou d'un `_` est un autre nom et n'est pas compté.
pub fn arites_des_appels(code: &str, nom: &str) -> Vec<usize> {
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
pub fn balises(html: &str) -> Vec<&str> {
    html.split('<').skip(1).filter_map(|morceau| morceau.split('>').next()).collect()
}

/// La valeur de l'attribut `nom` d'une balise (entre guillemets ou apostrophes, précédé d'une espace).
pub fn attribut(balise: &str, nom: &str) -> Option<String> {
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

pub fn nom_de_balise(balise: &str) -> String {
    balise.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_lowercase()
}

/// La première balise du HTML qui a cet `id`.
pub fn balise_d_id<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    balises(html).into_iter().find(|balise| attribut(balise, "id").as_deref() == Some(id))
}

/// Le texte qui suit la balise d'`id` donné, jusqu'au `<` suivant, sans espaces de début et de fin.
pub fn texte_de(html: &str, id: &str) -> Option<String> {
    for morceau in html.split('<').skip(1) {
        let (balise, texte) = morceau.split_once('>')?;
        if attribut(balise, "id").as_deref() == Some(id) {
            return Some(texte.trim().to_string());
        }
    }
    None
}

/// Le texte visible de la page : tout ce qui n'est pas dans une balise, ni dans `<style>` ni dans `<script>`.
pub fn texte_visible(html: &str) -> String {
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
pub fn a_l_attribut(balise: &str, nom: &str) -> bool {
    balise
        .split_whitespace()
        .skip(1)
        .any(|mot| mot == nom || mot.starts_with(&format!("{nom}=")) || mot == format!("{nom}/"))
}

/// Les `id` des balises contenues dans le `<div>` d'`id` `conteneur` (à n'importe quelle profondeur), dans l'ordre du document.
pub fn ids_dans_le_div(html: &str, conteneur: &str) -> Vec<String> {
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
pub fn fermeture(code: &str, ouvrante: usize) -> usize {
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
pub fn fonction_qui_contient(code: &str, position: usize) -> Option<(String, String)> {
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
pub fn variable_de(code: &str, id: &str) -> Option<String> {
    let appel = format!("getElementById(\"{id}\")");
    let position = code.find(&appel)?;
    let avant = code[..position].trim_end().strip_suffix("document.")?.trim_end().strip_suffix('=')?.trim_end();
    let nom: String = avant.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
    Some(nom.chars().rev().collect())
}

/// La valeur entière de `const NOM = <chiffres>` dans le code.
pub fn constante_entiere(code: &str, nom: &str) -> Option<u64> {
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
// Les règles CSS de la page (pour refuser `image-rendering: pixelated` là où il ne doit pas être).
// ---------------------------------------------------------------------------------------------------------------

/// Une règle CSS : ses sélecteurs (séparés par des virgules dans la page) et ses déclarations `(propriété, valeur)`, en minuscules, sans `!important`.
pub struct RegleCss {
    pub selecteurs: Vec<String>,
    pub declarations: Vec<(String, String)>,
}

/// Le texte des balises `<style>` de `index.html`, sans les commentaires `/* … */`.
pub fn css_de_la_page(html: &str) -> String {
    let mut css = String::new();
    let mut reste = html;
    while let Some(debut) = reste.find("<style") {
        let apres = &reste[debut..];
        let Some(fin_de_la_balise) = apres.find('>') else { break };
        let corps = &apres[fin_de_la_balise + 1..];
        let Some(fin) = corps.find("</style>") else { break };
        css.push_str(&corps[..fin]);
        css.push('\n');
        reste = &corps[fin..];
    }
    let mut sortie = String::new();
    let mut morceaux = css.split("/*");
    sortie.push_str(morceaux.next().unwrap_or(""));
    for morceau in morceaux {
        if let Some((_, suite)) = morceau.split_once("*/") {
            sortie.push_str(suite);
        }
    }
    sortie
}

/// Les règles d'un texte CSS : les règles de premier niveau et celles des blocs `@media`, `@supports`, `@layer` et `@container` ; les autres `@` (`@font-face`, `@keyframes`…) sont sautés. Une règle dont le corps
/// contient un bloc (l'imbrication CSS) n'est pas prise en charge : `panic!`, la page n'en a pas.
pub fn regles_css(css: &str) -> Vec<RegleCss> {
    let c: Vec<char> = css.chars().collect();
    let mut regles = Vec::new();
    lire_les_regles(&c, &mut regles);
    regles
}

fn lire_les_regles(c: &[char], regles: &mut Vec<RegleCss>) {
    let mut i = 0;
    while i < c.len() {
        let debut = i;
        while i < c.len() && c[i] != '{' && c[i] != ';' {
            i += 1;
        }
        if i >= c.len() {
            break;
        }
        if c[i] == ';' {
            i += 1;
            continue;
        }
        let prelude: String = c[debut..i].iter().collect::<String>().trim().to_string();
        let debut_du_corps = i + 1;
        let mut profondeur = 1;
        i += 1;
        while i < c.len() && profondeur > 0 {
            match c[i] {
                '{' => profondeur += 1,
                '}' => profondeur -= 1,
                _ => {}
            }
            i += 1;
        }
        let corps = &c[debut_du_corps..i.saturating_sub(1).max(debut_du_corps)];
        if prelude.starts_with('@') {
            if ["@media", "@supports", "@layer", "@container"].iter().any(|nom| prelude.starts_with(nom)) {
                lire_les_regles(corps, regles);
            }
            continue;
        }
        assert!(!corps.contains(&'{'), "l'imbrication CSS n'est pas prise en charge par le test : {prelude}");
        let corps: String = corps.iter().collect();
        let declarations = corps
            .split(';')
            .filter_map(|declaration| {
                let (propriete, valeur) = declaration.split_once(':')?;
                let valeur = valeur.to_lowercase().replace("!important", "");
                Some((propriete.trim().to_lowercase(), valeur.trim().to_string()))
            })
            .collect();
        regles.push(RegleCss { selecteurs: prelude.split(',').map(|s| s.trim().to_string()).collect(), declarations });
    }
}

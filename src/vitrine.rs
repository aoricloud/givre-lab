//! La vitrine (étape 5) : la DLA et les flocons de Reiter pilotés par des nombres seulement, pour une page web.
//!
//! Ce module est compilé deux fois : en natif (les tests appellent ses fonctions comme n'importe quelle fonction Rust) et en
//! WebAssembly (`wasm32-unknown-unknown`, module sans importation). Chaque fonction `givre_*` est exportée sous son nom
//! (`#[unsafe(no_mangle)] pub extern "C" fn`) et ne prend et ne rend que des nombres : un `u32` devient un `i32` côté
//! WebAssembly (JavaScript le lit signé : `>>> 0` le relit non signé), un `f64` reste un `f64`. Il n'y a ni pointeur ni
//! mémoire à manipuler côté JavaScript. **Onze fonctions, ni plus ni moins**, toutes de ce fichier ; aucune autre n'est
//! exportée (le module exporte aussi `memory`, que la cible ajoute, et rien d'autre).
//!
//! **Aucune de ces fonctions ne panique, jamais** : une panique dans une `extern "C" fn` fait avorter le processus (le
//! binaire de test tout entier, ou, en WebAssembly, une instruction `unreachable` que la page ne peut pas rattraper). Pas de
//! `unwrap`, pas de `expect`, pas d'indice `[i]` hors de portée, pas de `todo!()` : `.get()`, des bornes vérifiées, et les
//! valeurs « hors de portée » dites plus bas. Les fonctions de la bibliothèque qui paniquent (`Amas::ajouter`) ne sont pas
//! appelées ici : on emploie `Amas::essayer_ajouter`.
//!
//! # Le module WebAssembly
//!
//! `docs/givre.wasm` est commité. Il se construit, depuis la racine du dépôt, par :
//!
//! ```text
//! cargo rustc --release --lib --target wasm32-unknown-unknown --crate-type cdylib
//! ```
//!
//! puis `target/wasm32-unknown-unknown/release/givre_lab.wasm` se copie en `docs/givre.wasm` (et **pas** `cargo build` : il
//! produirait le module du binaire `givre`, qui exporte aussi `main`). `Cargo.toml` ne change pas. Les octets du module
//! portent les lignes des `panic!` de la bibliothèque : tout changement de `src/` change le module ; les tests comparent
//! son **comportement**, pas ses octets. `tests/etape_5_artefact.rs` fait tourner ce fichier avec Node et compare à ceux du
//! code natif son empreinte et ce que lisent ses accesseurs : si le code change sans que le module soit reconstruit, il
//! échoue.
//!
//! # Les quatre modèles
//!
//! `givre_demarrer(modele, graine)` choisit un modèle par son numéro. Les réglages sont ceux des scénarios d'exemple du
//! dépôt (les tests relisent ces fichiers et vérifient qu'ils disent bien ceci) :
//!
//! | `modele` | quoi | réglages (le scénario) | `duree` |
//! |---|---|---|---|
//! | 0 | la DLA, [`crate::dla::Amas`] | `scenarios/dla.txt` : `boite 401 401`, donc `Amas::new(graine, 200)` ; `dla 3000` : 3000 sites, germe compris | 2999 |
//! | 1 | Reiter, la plaque, [`crate::flocon::Flocon`] | `scenarios/reiter-plaque.txt` : `hexagone 30`, `reiter 1 0 0.05 400` | 400 |
//! | 2 | Reiter, l'étoile | `scenarios/reiter-etoile.txt` : `hexagone 40`, `reiter 1 0.45 0.1 120` | 120 |
//! | 3 | Reiter, la dendrite | `scenarios/reiter-dendrite.txt` : `hexagone 60`, `reiter 1 0.4 0.001 800` | 800 |
//!
//! Pour les modèles 1 à 3, le flocon est `Flocon::new(rayon, Parametres::lire(alpha, beta, gamma))` avec les trois textes
//! du scénario (`"1"`, `"0.45"`, `"0.1"` par exemple : la lecture exacte de `Parametres::lire`, comme `lancer_reiter`).
//!
//! # Le temps
//!
//! Une **unité de temps** est une particule pour la DLA (un appel de `Amas::essayer_ajouter`), un pas pour un flocon (un
//! appel de `Flocon::pas`). Le **temps** ([`givre_temps`]) est le nombre d'unités jouées depuis le démarrage : pour la DLA
//! le nombre de particules ajoutées, c'est-à-dire le nombre de sites **moins un** (le germe n'est pas une particule) ; pour un
//! flocon `Flocon::temps()`. La **durée** ([`givre_duree`]) est le temps où s'arrête le scénario (dernière colonne du tableau) :
//! `givre_avancer` ne joue jamais au-delà.
//!
//! # L'état global
//!
//! Un seul modèle est en cours : un `static` protégé par un [`std::sync::Mutex`] (la cible WebAssembly n'a pas de fil
//! d'exécution ; en natif le verrou sérialise les appels des tests ; un verrou empoisonné par une panique se reprend par
//! `into_inner`). [`givre_demarrer`] le (re)construit **entièrement** : rien de l'état précédent ne survit (ni le temps, ni
//! l'image, ni l'empreinte).
//!
//! # Avant tout démarrage
//!
//! Tant que [`givre_demarrer`] n'a pas réussi une fois, les fonctions qui rendent un entier rendent 0 (sauf [`givre_pixel`] :
//! `u32::MAX`), [`givre_compacite`] rend `NaN` et [`givre_avancer`] ne fait rien (rend 0). Ce contrat n'est éprouvé que par
//! Node (`tests/etape_5/etat-wasm.mjs`, sur un module neuf) : en natif l'état global est déjà posé par les autres tests du
//! processus.
//!
//! # L'image
//!
//! L'image de l'état courant est celle de l'aperçu de la bibliothèque, et la page la dessine telle quelle :
//! - DLA : `amas_colore(amas.sites(), COTE_APERCU)` de [`crate::dla`] (sa largeur et sa hauteur **changent** quand l'amas
//!   grandit : la fenêtre suit l'amas ; au temps 0, un site, l'image est de 512 × 512 pixels tous blancs) ;
//! - flocon : `Flocon::apercu()`, l'image carrée de côté `2R + 1` (61, 81 ou 121 pixels ; la case `(q, r)` est le pixel
//!   `(q + R + floor(r / 2), r + R)`).
//!
//! [`givre_largeur`] et [`givre_hauteur`] donnent ses dimensions, [`givre_pixel`] la couleur d'un pixel `(x, y)` (`x` vers la
//! droite, `y` vers le bas, `(0, 0)` en haut à gauche, ligne par ligne de haut en bas comme les octets de l'aperçu) **empaquetée
//! en `0x00RRGGBB`** : `(R << 16) | (V << 8) | B`, les trois octets de l'aperçu à cette position. `u32::MAX`
//! (`0xFFFFFFFF`, jamais une couleur : son octet de poids fort n'est pas nul) dit « hors de l'image » : `x >= largeur`, ou
//! `y >= hauteur`, ou rien de démarré.
//!
//! # Les mesures
//!
//! - [`givre_sites`] : DLA, le nombre de sites de l'amas, **germe compris** (1 au démarrage) ; flocon, `Flocon::gelees()` ;
//! - [`givre_rayon`] : DLA, `Amas::rayon()` ; flocon, `Flocon::rayon_cristal()` ;
//! - [`givre_compacite`] : flocon, `Flocon::compacite().vers_f64()` (1 pour un hexagone plein, et au temps 0 : une case gelée
//!   sur une case) ; DLA, `NaN` (sans objet : l'amas n'a pas de compacité).
//!
//! # L'empreinte
//!
//! [`givre_empreinte_mot`]`(i)` est le `i`-ième mot (de 0 à 7) de 32 bits, gros-boutiste, du SHA-256 de l'écriture canonique
//! de l'état courant : l'empreinte de `Amas::empreinte()` (DLA) ou de `Flocon::empreinte()` (flocon). Les huit mots, chacun
//! écrit sur 8 chiffres hexadécimaux minuscules et mis bout à bout dans l'ordre de 0 à 7, redonnent ces 64 caractères. Pour la
//! DLA de graine 1 jouée jusqu'au bout, c'est le contenu de `references/dla-graine-1.empreinte` ; pour la plaque jouée jusqu'au
//! bout, celui de `references/reiter-plaque.empreinte` : **l'empreinte d'une graine dans la page est celle du code natif.**
//! Chaque appel recalcule l'empreinte de l'état (huit appels = huit SHA-256 de tout l'état) : la page ne la lit pas à chaque
//! image.
//!
//! # Hors de portée
//!
//! [`givre_pixel`] rend `u32::MAX` hors de l'image ; [`givre_empreinte_mot`] rend 0 pour `i >= 8` ; [`givre_demarrer`] rend 0
//! pour un modèle qui n'existe pas (et ne change rien) ; [`givre_avancer`] ne joue rien au-delà de la durée.
//!
//! # La page
//!
//! `docs/index.html`, `docs/app.js` et `docs/lecture.js` font tourner ce module dans un navigateur ; leurs règles sont écrites
//! en tête de `tests/etape_5_page.rs`. `lecture.js` (l'empreinte, l'image et la graine que la page lit, sans DOM) est éprouvé
//! sous Node contre le code natif ; le reste de la page ne l'est que par sa forme : voir « Ce que cela n'établit pas » dans
//! `MODELE.md`.
//!
//! # Un conseil d'écriture (pas une règle : seule la description ci-dessus est imposée)
//!
//! L'état est un `enum` (un `Amas` ou un `Flocon`) avec la durée du scénario ; l'image se garde en mémoire et se refait quand
//! l'état a changé (après [`givre_avancer`] et [`givre_demarrer`]) : la page lit `largeur × hauteur` pixels une fois par
//! image, et `amas_colore` coûte à chaque appel une image entière. **Garder l'image est en pratique obligatoire** : sans cela
//! chaque `givre_pixel` refait l'image entière, et les tests, qui lisent des centaines de milliers de pixels, ne finissent plus
//! en dix minutes (mesuré sur le prototype sans cache : plus de 600 s, contre 4 s avec). Une image gardée d'avant un `avancer`
//! est la faute typique : les tests lisent l'image après chaque morceau.

use std::sync::{Mutex, MutexGuard};

use crate::dla::{Amas, COTE_APERCU, amas_colore};
use crate::empreinte::sha256;
use crate::flocon::{Flocon, Parametres};

/// Une image d'aperçu gardée en mémoire : ses dimensions et ses octets R, V, B, ligne par ligne de haut en bas.
struct Image {
    largeur: u32,
    hauteur: u32,
    rvb: Vec<u8>,
}

/// Le modèle en cours : l'amas de DLA ou le flocon de Reiter.
enum Modele {
    Dla(Amas),
    Flocon(Flocon),
}

/// L'état global : le modèle, la durée de son scénario et l'image de l'état courant (refaite quand l'état change).
struct Etat {
    modele: Modele,
    duree: u32,
    image: Image,
}

/// Le seul état de la vitrine ; `None` avant tout démarrage réussi.
static ETAT: Mutex<Option<Etat>> = Mutex::new(None);

/// Prend le verrou de l'état ; un verrou empoisonné par une panique se reprend par `into_inner` (jamais de panique ici).
fn verrouiller() -> MutexGuard<'static, Option<Etat>> {
    ETAT.lock().unwrap_or_else(|empoisonne| empoisonne.into_inner())
}

/// L'image de l'aperçu d'un modèle : `amas_colore` pour la DLA, `Flocon::apercu` pour un flocon.
fn image_de(modele: &Modele) -> Image {
    match modele {
        Modele::Dla(amas) => {
            let (largeur, hauteur, rvb) = amas_colore(amas.sites(), COTE_APERCU);
            Image { largeur, hauteur, rvb }
        }
        Modele::Flocon(flocon) => {
            let cote = 2 * flocon.rayon() + 1;
            Image { largeur: cote, hauteur: cote, rvb: flocon.apercu() }
        }
    }
}

/// Le temps du modèle en cours (voir « Le temps ») : les sites moins le germe pour la DLA, `Flocon::temps` sinon.
fn temps_de(modele: &Modele) -> u32 {
    match modele {
        Modele::Dla(amas) => amas.sites().len().saturating_sub(1) as u32,
        Modele::Flocon(flocon) => u32::try_from(flocon.temps()).unwrap_or(u32::MAX),
    }
}

/// Construit un flocon d'après les trois textes de paramètres du scénario ; `None` s'ils ne se lisent pas (ils se lisent).
fn flocon_de(rayon: u32, alpha: &str, beta: &str, gamma: &str) -> Option<Flocon> {
    let parametres = Parametres::lire(alpha, beta, gamma)?;
    Some(Flocon::new(rayon, parametres))
}

/// (Re)construit le modèle `modele` (0 à 3, voir « Les quatre modèles ») au temps 0 et rend 1 ; `graine` n'est lue que par la
/// DLA (`Amas::new(u64::from(graine), 200)` : un `u32` entier, de 0 à `u32::MAX`, sans troncature ni signe) et ignorée par les
/// flocons, qui n'ont pas de hasard (la même graine n'importe laquelle donne le même flocon).
///
/// Remplace entièrement l'état précédent : rappelée après des pas, elle repart de zéro. Pour un `modele` qui n'est pas 0, 1, 2
/// ou 3, ne change rien (l'état précédent reste en place, ou l'absence d'état) et rend 0.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_demarrer(modele: u32, graine: u32) -> u32 {
    // Les réglages sont ceux des scénarios d'exemple (voir « Les quatre modèles ») : ils sont écrits ici, sans fichier.
    let construit = match modele {
        0 => Some((Modele::Dla(Amas::new(u64::from(graine), 200)), 2999)),
        1 => flocon_de(30, "1", "0", "0.05").map(|flocon| (Modele::Flocon(flocon), 400)),
        2 => flocon_de(40, "1", "0.45", "0.1").map(|flocon| (Modele::Flocon(flocon), 120)),
        3 => flocon_de(60, "1", "0.4", "0.001").map(|flocon| (Modele::Flocon(flocon), 800)),
        _ => None,
    };
    let Some((modele, duree)) = construit else { return 0 };
    let image = image_de(&modele);
    let mut garde = verrouiller();
    *garde = Some(Etat { modele, duree, image });
    1
}

/// Joue `min(n, duree - temps)` unités de temps (voir « Le temps ») et rend ce nombre : `n` si le scénario n'est pas fini, moins
/// (jusqu'à 0) s'il s'arrête avant, 0 pour `n = 0`. Un appel de `n` unités puis un de `m` donne exactement l'état d'un appel de
/// `n + m` (la dynamique ne dépend pas du découpage). Rend 0 et ne fait rien avant tout démarrage.
///
/// Pour la DLA, si une particule collait hors du réseau (un amas de 3000 sites tient largement dans le carré de côté 401 : cela
/// n'arrive pas en pratique), `givre_avancer` s'arrête là, sans panique, et rend ce qu'elle a joué.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_avancer(n: u32) -> u32 {
    let mut garde = verrouiller();
    let Some(etat) = garde.as_mut() else { return 0 };
    let reste = etat.duree.saturating_sub(temps_de(&etat.modele));
    let a_jouer = n.min(reste);
    if a_jouer == 0 {
        return 0;
    }
    let jouees = match &mut etat.modele {
        Modele::Dla(amas) => {
            let mut jouees = 0;
            while jouees < a_jouer {
                if amas.essayer_ajouter().is_none() {
                    break;
                }
                jouees += 1;
            }
            jouees
        }
        Modele::Flocon(flocon) => {
            flocon.avancer(u64::from(a_jouer));
            a_jouer
        }
    };
    if jouees > 0 {
        etat.image = image_de(&etat.modele);
    }
    jouees
}

/// Le temps : les unités jouées depuis le démarrage (voir « Le temps »), de 0 à [`givre_duree`] ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_temps() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| temps_de(&etat.modele))
}

/// La durée du scénario du modèle en cours : 2999, 400, 120 ou 800 (voir « Les quatre modèles ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_duree() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.duree)
}

/// La largeur, en pixels, de l'image de l'état courant (voir « L'image ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_largeur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.image.largeur)
}

/// La hauteur, en pixels, de l'image de l'état courant (voir « L'image ») ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_hauteur() -> u32 {
    let garde = verrouiller();
    garde.as_ref().map_or(0, |etat| etat.image.hauteur)
}

/// La couleur du pixel `(x, y)` de l'image de l'état courant, empaquetée en `0x00RRGGBB` (voir « L'image »).
///
/// Rend `u32::MAX` si `x >= givre_largeur()`, si `y >= givre_hauteur()` et avant tout démarrage.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_pixel(x: u32, y: u32) -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return u32::MAX };
    let image = &etat.image;
    if x >= image.largeur || y >= image.hauteur {
        return u32::MAX;
    }
    let indice = (y as usize)
        .checked_mul(image.largeur as usize)
        .and_then(|ligne| ligne.checked_add(x as usize))
        .and_then(|pixel| pixel.checked_mul(3));
    let Some(indice) = indice else { return u32::MAX };
    let Some(fin) = indice.checked_add(3) else { return u32::MAX };
    match image.rvb.get(indice..fin) {
        Some([r, v, b]) => (u32::from(*r) << 16) | (u32::from(*v) << 8) | u32::from(*b),
        _ => u32::MAX,
    }
}

/// Le nombre de sites de l'amas (germe compris) pour la DLA, de cases gelées pour un flocon (voir « Les mesures ») ; 0 avant
/// tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_sites() -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    match &etat.modele {
        Modele::Dla(amas) => amas.sites().len() as u32,
        Modele::Flocon(flocon) => flocon.gelees() as u32,
    }
}

/// Le rayon de l'amas (DLA) ou du cristal (flocon), voir « Les mesures » ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_rayon() -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    match &etat.modele {
        Modele::Dla(amas) => amas.rayon(),
        Modele::Flocon(flocon) => flocon.rayon_cristal(),
    }
}

/// La compacité du flocon (`Flocon::compacite().vers_f64()`, de 0 à 1) ; `NaN` pour la DLA et avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_compacite() -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    match &etat.modele {
        Modele::Dla(_) => f64::NAN,
        Modele::Flocon(flocon) => flocon.compacite().vers_f64(),
    }
}

/// Le `i`-ième mot (0 à 7) de 32 bits, gros-boutiste, du SHA-256 de l'écriture canonique de l'état (voir « L'empreinte »).
///
/// Les huit mots, chacun écrit sur 8 chiffres hexadécimaux minuscules et mis bout à bout dans l'ordre de 0 à 7, redonnent
/// `Amas::empreinte()` ou `Flocon::empreinte()` (64 caractères). Rend 0 si `i >= 8` et avant tout démarrage.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_empreinte_mot(i: u32) -> u32 {
    if i >= 8 {
        return 0;
    }
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    let ecriture = match &etat.modele {
        Modele::Dla(amas) => amas.ecriture_canonique(),
        Modele::Flocon(flocon) => flocon.ecriture_canonique(),
    };
    let empreinte = sha256(&ecriture);
    let debut = 4 * i as usize;
    let Some(fin) = debut.checked_add(4) else { return 0 };
    match empreinte.get(debut..fin) {
        Some([a, b, c, d]) => u32::from_be_bytes([*a, *b, *c, *d]),
        _ => 0,
    }
}

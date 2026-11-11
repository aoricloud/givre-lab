//! La vitrine (étape 5) : la DLA, les flocons de Reiter et (étape D3) le claquage diélectrique pilotés par des nombres seulement, pour une page web.
//!
//! Ce module est compilé deux fois : en natif (les tests appellent ses fonctions comme n'importe quelle fonction Rust) et en
//! WebAssembly (`wasm32-unknown-unknown`, module sans importation). Chaque fonction `givre_*` est exportée sous son nom
//! (`#[unsafe(no_mangle)] pub extern "C" fn`) et ne prend et ne rend que des nombres : un `u32` devient un `i32` côté
//! WebAssembly (JavaScript le lit signé : `>>> 0` le relit non signé), un `f64` reste un `f64`. Il n'y a ni pointeur ni
//! mémoire à manipuler côté JavaScript. **Vingt-trois fonctions, ni plus ni moins** (les onze de l'étape 5, les quatre de l'étape Z1, voir
//! « La mesure de la corrélation », et les huit de l'étape D3, voir « La courbe D(η) à la demande »), toutes de ce fichier ; aucune autre n'est
//! exportée (le module exporte aussi `memory`, que la cible ajoute, et rien d'autre).
//!
//! **Aucune de ces fonctions ne panique, jamais** : une panique dans une `extern "C" fn` fait avorter le processus (le
//! binaire de test tout entier, ou, en WebAssembly, une instruction `unreachable` que la page ne peut pas rattraper). Pas de
//! `unwrap`, pas de `expect`, pas d'indice `[i]` hors de portée, pas de `todo!()` : `.get()`, des bornes vérifiées, et les
//! valeurs « hors de portée » dites plus bas. Les fonctions de la bibliothèque qui paniquent (`Amas::ajouter`, `Claquage::ajouter`,
//! `point_de_la_courbe`) ne sont pas appelées ici : on emploie `Amas::essayer_ajouter`, `Claquage::essayer_ajouter` et les entrées
//! `essayer_*` de [`crate::courbe`], qui rendent une erreur au lieu de paniquer (étape D3). Un cas absurde donne une valeur « hors de portée » ou
//! un **code d'erreur** (voir « La courbe D(η) à la demande »), jamais une panique.
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
//! # Les huit modèles
//!
//! `givre_demarrer(modele, graine)` choisit un modèle par son numéro. Les réglages des modèles 0 à 3 sont ceux des scénarios d'exemple du
//! dépôt (les tests relisent ces fichiers et vérifient qu'ils disent bien ceci) ; ceux des modèles 4 à 7 sont écrits ici (étape D3) :
//!
//! | `modele` | quoi | réglages (le scénario) | `duree` |
//! |---|---|---|---|
//! | 0 | la DLA, [`crate::dla::Amas`] | `scenarios/dla.txt` : `boite 401 401`, donc `Amas::new(graine, 200)` ; `dla 3000` : 3000 sites, germe compris | 2999 |
//! | 1 | Reiter, la plaque, [`crate::flocon::Flocon`] | `scenarios/reiter-plaque.txt` : `hexagone 30`, `reiter 1 0 0.05 400` | 400 |
//! | 2 | Reiter, l'étoile | `scenarios/reiter-etoile.txt` : `hexagone 40`, `reiter 1 0.45 0.1 120` | 120 |
//! | 3 | Reiter, la dendrite | `scenarios/reiter-dendrite.txt` : `hexagone 60`, `reiter 1 0.4 0.001 800` | 800 |
//! | 4 | le claquage diélectrique `η = 0`, [`crate::claquage::Claquage`] | `Claquage::new(graine, 70, 0)`, jusqu'à 400 sites, germe compris | 399 |
//! | 5 | le claquage `η = 1` | `Claquage::new(graine, 52, 1)`, jusqu'à 250 sites | 249 |
//! | 6 | le claquage `η = 2` | `Claquage::new(graine, 49, 2)`, jusqu'à 150 sites | 149 |
//! | 7 | le claquage `η = 3` | `Claquage::new(graine, 48, 3)`, jusqu'à 100 sites | 99 |
//!
//! Pour les modèles 1 à 3, le flocon est `Flocon::new(rayon, Parametres::lire(alpha, beta, gamma))` avec les trois textes
//! du scénario (`"1"`, `"0.45"`, `"0.1"` par exemple : la lecture exacte de `Parametres::lire`, comme `lancer_reiter`).
//!
//! **Le claquage (étape D3).** Le modèle `4 + η` (`η` de 0 à 3) est le claquage d'exposant `η` : `Claquage::new(u64::from(graine), demi_cote, η)` (la
//! graine est lue comme celle de la DLA : un `u32` entier, de 0 à `u32::MAX`), avec les **sites visés** et le **demi-côté** du tableau (le demi-côté est
//! `demi_cote_de_la_courbe(η, sites)` de [`crate::courbe`], la règle de l'étape D2 : 70, 52, 49 et 48). Le claquage coûte cher (le champ de Laplace est
//! relaxé après chaque site, `MODELE.md` étape D1) : **la taille de l'amas est bornée** par ce tableau, et aucun modèle de la page ne le dépasse (`η = 0`
//! n'a pas de champ et coûte presque rien ; `η = 3` vise 100 sites). Le temps natif d'une croissance entière est mesuré dans « Le coût » de `MODELE.md`
//! (étape D3). La **graine** compte pour ces modèles (comme pour la DLA) ; les flocons l'ignorent.
//!
//! # Le temps
//!
//! Une **unité de temps** est une particule pour la DLA (un appel de `Amas::essayer_ajouter`), un site de plus pour le claquage (un appel de
//! `Claquage::essayer_ajouter`), un pas pour un flocon (un appel de `Flocon::pas`). Le **temps** ([`givre_temps`]) est le nombre d'unités jouées depuis
//! le démarrage : pour la DLA et le claquage le nombre de particules ajoutées, c'est-à-dire le nombre de sites **moins un** (le germe n'est pas une
//! particule) ; pour un flocon `Flocon::temps()`. La **durée** ([`givre_duree`]) est le temps où s'arrête le scénario (dernière colonne du tableau) :
//! `givre_avancer` ne joue jamais au-delà. Une unité de claquage coûte la relaxation du champ (de 0,000 s à `η = 0` à quelques dixièmes de seconde au
//! pire à `η >= 1`, natif, `release` : voir `MODELE.md`) : **un appel de `givre_avancer` qui joue beaucoup d'unités peut durer, pour les modèles 4 à 7,
//! jusqu'au temps de la croissance entière** ; la page joue ces modèles unité par unité, en s'arrêtant quand une courte durée est écoulée.
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
//! - claquage (modèles 4 à 7) : `amas_colore(claquage.sites(), COTE_APERCU)`, **la même fonction que la DLA** (c'est aussi l'aperçu de `lancer` pour la
//!   directive `claquage`) : une image carrée dont le côté suit l'amas (au temps 0, un site : 512 × 512 pixels, le pixel du germe de la couleur du rang 0) ;
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
//! - [`givre_sites`] : DLA et claquage, le nombre de sites de l'amas, **germe compris** (1 au démarrage) ; flocon, `Flocon::gelees()` ;
//! - [`givre_rayon`] : DLA, `Amas::rayon()` ; claquage, `Claquage::rayon()` ; flocon, `Flocon::rayon_cristal()` ;
//! - [`givre_compacite`] : flocon, `Flocon::compacite().vers_f64()` (1 pour un hexagone plein, et au temps 0 : une case gelée
//!   sur une case) ; DLA et claquage, `NaN` (sans objet : l'amas n'a pas de compacité).
//!
//! # L'empreinte
//!
//! [`givre_empreinte_mot`]`(i)` est le `i`-ième mot (de 0 à 7) de 32 bits, gros-boutiste, du SHA-256 de l'écriture canonique
//! de l'état courant : l'empreinte de `Amas::empreinte()` (DLA), de `Claquage::empreinte()` (claquage) ou de `Flocon::empreinte()` (flocon). Les huit mots, chacun
//! écrit sur 8 chiffres hexadécimaux minuscules et mis bout à bout dans l'ordre de 0 à 7, redonnent ces 64 caractères. Pour la
//! DLA de graine 1 jouée jusqu'au bout, c'est le contenu de `references/dla-graine-1.empreinte` ; pour la plaque jouée jusqu'au
//! bout, celui de `references/reiter-plaque.empreinte` : **l'empreinte d'une graine dans la page est celle du code natif.**
//! Chaque appel recalcule l'empreinte de l'état (huit appels = huit SHA-256 de tout l'état) : la page ne la lit pas à chaque
//! image.
//!
//! # La mesure de la corrélation (étape Z1)
//!
//! Quatre fonctions de plus, qui mesurent la dimension de l'amas de la DLA (et, depuis l'étape D3, du claquage) par la corrélation densité-densité
//! de [`crate::correlation`] et rendent la courbe `C(r)` pour la page. Elles ne paniquent jamais non plus.
//!
//! - [`givre_mesurer`] mesure **l'amas du moment** (jamais à chaque image : le coût est en `O(n²)`) et garde le résultat (la fenêtre, les
//!   `C(r)` et la dimension) : pour la DLA par [`crate::correlation::mesure`], **pour le claquage (modèles 4 à 7, étape D3) par
//!   [`crate::courbe::mesure_petit_amas`]** (la fenêtre des petits amas de l'étape D2 : des rayons de 2 à `max(4, étendue / 6)`, au lieu de 4 à
//!   `étendue / 8` pour la DLA). Elle rend le **nombre de points**
//!   de la courbe, c'est-à-dire le nombre de rayons de la fenêtre (`mesure.rayons.len()`), ou **0 pour « pas de mesure »** : avant tout
//!   démarrage, pour un flocon (modèles 1, 2 ou 3), pour un amas trop petit (la fenêtre est vide : pour la DLA, voir
//!   [`crate::correlation::fenetre`], une étendue de moins de 72, soit de 465 à 774 sites sur 40 graines mesurées ; pour le claquage, voir
//!   [`crate::courbe::fenetre_petit_amas`], une étendue de moins de 8), pour un amas de plus de
//!   [`MESURE_SITES_MAX`] sites, ou si la mesure rend `None`. Un appel qui rend 0 laisse « pas de mesure » (aucune mesure n'est
//!   gardée).
//! - [`givre_correlation_rayon`]`(i)` est le `i`-ième rayon de la fenêtre de la mesure gardée (`i` de 0 à `points - 1`, dans l'ordre
//!   croissant : 4, 5, 6, … pour la DLA ; 2, 3, 4, … pour le claquage) ; [`givre_correlation_valeur`]`(i)` est le `C(r)` du même rang, **bit pour bit**
//!   celui de [`crate::correlation::mesure`] (DLA) ou de [`crate::courbe::mesure_petit_amas`] (claquage) ; [`givre_dimension`] est la dimension de la
//!   mesure gardée, bit pour bit celle de `mesure` ou de `mesure_petit_amas`.
//!   Sans mesure gardée (y compris avant tout démarrage), ou pour `i >= points` : `givre_correlation_rayon` rend 0, `givre_correlation_valeur` et `givre_dimension`
//!   rendent `NaN`.
//! - **La mesure gardée appartient à l'amas qu'elle a mesuré** : [`givre_demarrer`] (qui reconstruit tout l'état) l'efface, et
//!   [`givre_avancer`] l'efface dès qu'il joue **au moins une unité** ; un `givre_avancer` qui ne joue rien (scénario fini, `n = 0`) ne
//!   l'efface pas.
//! - [`MESURE_SITES_MAX`] (10 000) est le plafond de taille d'amas de la mesure dans la page. La DLA de la vitrine s'arrête à 3000
//!   sites : le plafond n'est atteint par aucun modèle de la table ci-dessus, il garde la page vive si un modèle plus grand est ajouté.
//!   Le temps de la mesure en natif (profil `release`, une graine) est de 0,008 s à 2000 sites, 0,040 s à 5000, 0,17 s à 10 000, 0,69 s à
//!   20 000 et 1,67 s à 30 000 : il croît comme `n²`. Le module WebAssembly est plus lent d'un facteur qui n'a pas été mesuré (ni Node ni la
//!   cible `wasm32-unknown-unknown` n'étaient là) : 10 000 sites laisse de la marge sur la seconde et demie visée, 20 000 sites n'en laisse
//!   que si ce facteur est inférieur à 2 ; le plafond se relève sur une mesure du module.
//!
//! # La courbe D(η) à la demande (étape D3)
//!
//! Huit fonctions de plus calculent la courbe `D(η)` de [`crate::courbe`] (la dimension de petits amas de claquage pour `η = 0, 1, 2, 3`, moyennée sur
//! des graines) **un amas par appel**, de façon que la page ne se fige jamais : un amas de la taille permise coûte moins d'une seconde en natif (`release`,
//! mesuré : `MODELE.md`, étape D3), et la page rend la main entre deux appels. Elles ne prennent et ne rendent que des nombres, **ne paniquent jamais**,
//! et appellent les entrées `essayer_*` de [`crate::courbe`] : chaque échec devient un **code d'erreur** (la table plus bas).
//!
//! **Le travail.** [`givre_courbe_demarrer`]`(taille, graines)` démarre un travail : quatre points (`η = 0, 1, 2, 3`), chacun fait de `graines` amas de
//! `taille` sites (germe compris), de graines `1, 2, …, graines` (la graine de rang `k`, en partant de 1, est `k`) et de demi-côté
//! `essayer_demi_cote_de_la_courbe(η, taille)`. Les amas se font **dans cet ordre** : tous ceux de `η = 0` (graines `1` à `graines`), puis ceux de
//! `η = 1`, puis `η = 2`, puis `η = 3`. Le **total** de amas est `4 × graines` ; le `k`-ième amas (`k` de 0 à `total - 1`) est celui de `η = k / graines`
//! (division entière) et de graine `k % graines + 1`. [`givre_courbe_pas`] fait **un** amas, le suivant : [`essayer_dimension_d_une_graine`] avec ces
//! `η`, graine, `taille` et demi-côté, et garde sa dimension ; quand le dernier amas d'un `η` est fait, le point de cet `η` est la
//! [`moyenne_et_erreur_type`] des `graines` dimensions de cet `η`, dans l'ordre des graines : **bit pour bit** le `moyenne` et l'`erreur_type` de
//! `essayer_point_de_la_courbe(η, [1, 2, …, graines], taille, demi_cote)`. [`givre_courbe_fait`] est le nombre d'amas faits (0 au démarrage, `total` à
//! la fin) et [`givre_courbe_total`] le total. Un travail fini rend 0 à tout `givre_courbe_pas` de plus et ne change plus.
//!
//! **Les bornes** (la page borne la taille des amas) : [`COURBE_TAILLE_MAX`] (100) sites, de [`COURBE_GRAINES_MIN`] (2) à [`COURBE_GRAINES_MAX`] (8)
//! graines. `taille` doit être au moins 1 et au plus 100, `graines` au moins 2 et au plus 8 : sinon `givre_courbe_demarrer` rend un code d'erreur
//! (1 ou 2) et **ne démarre aucun travail**. Un amas de la taille maximale coûte, au pire de nos mesures, moins d'une seconde en natif (`MODELE.md`).
//!
//! **Un nouveau `givre_courbe_demarrer` efface l'ancien travail** (même s'il est refusé : il n'y a alors aucun travail, `fait` et `total` valent 0).
//! `givre_demarrer` (le modèle en cours) **ne touche pas** au travail de la courbe : les deux sont indépendants, et un `givre_courbe_pas` ne touche pas
//! au modèle en cours.
//!
//! **Les valeurs.** [`givre_courbe_moyenne`]`(eta)` et [`givre_courbe_erreur_type`]`(eta)` rendent la moyenne et l'erreur-type du point de `eta` (0 à 3) **une
//! fois ses `graines` amas faits**, bit pour bit ceux de la bibliothèque, et `NaN` avant (et sans travail, et pour `eta > 3`, jusqu'à `u32::MAX`).
//! [`givre_courbe_repere`]`(eta)` rend le repère publié de `eta` : `crate::courbe::publie(eta)` (`1,71` pour `η = 1`, `1,433` pour `η = 2`, `1,263` pour
//! `η = 3`, voir [`crate::courbe`] : le rappel de la DLA et Hastings 2001) et `NaN` quand il n'y en a pas (`η = 0`, `η >= 4`). **Il vient de la
//! bibliothèque : ni ce fichier ni la page n'écrivent une valeur publiée.** Il ne dépend d'aucun travail (la page le lit aussi pour le modèle en cours).
//!
//! **Les codes d'erreur.** [`givre_courbe_demarrer`] et [`givre_courbe_pas`] rendent un code ; [`givre_courbe_code`] rend **le dernier** (celui de
//! l'appel le plus récent de l'une ou l'autre ; 0 avant tout appel) :
//!
//! | code | sens | qui le rend |
//! |---|---|---|
//! | 0 | aucune erreur | démarrage accepté ; un amas fait ; travail fini |
//! | 1 | `taille` hors borne (0 ou plus de 100) | `givre_courbe_demarrer` |
//! | 2 | `graines` hors borne (moins de 2 ou plus de 8) | `givre_courbe_demarrer` |
//! | 3 | un amas a touché la frontière avant d'avoir `taille` sites ([`ErreurCourbe::FrontiereTouchee`]) | `givre_courbe_pas` |
//! | 4 | un amas n'a pas de dimension : fenêtre vide ou `C(r)` nul ([`ErreurCourbe::FenetreVide`]), par exemple `taille` de 1 ou 5 sites (mesuré : à 32 sites, la graine 1 de `η = 0` et la graine 5 de `η = 1` n'ont pas de dimension ; dès 34 sites les graines 1 à 8 des quatre `η` en ont toutes) | `givre_courbe_pas` |
//! | 5 | aucun travail : rien n'est démarré, ou le démarrage a été refusé | `givre_courbe_pas` |
//! | 6 | une autre erreur de la bibliothèque (un paramètre que ce module ne passe jamais : `η`, demi-côté, graines) | `givre_courbe_pas` |
//!
//! La **première** erreur d'un `givre_courbe_pas` (3, 4 ou 6) **arrête** le travail : il garde les amas déjà faits (les points des `η` finis restent lisibles ;
//! `fait` ne compte pas l'amas qui a échoué et ne bouge plus), n'avance plus, et tout `givre_courbe_pas` de plus rend le même code sans rien faire (`givre_courbe_code`
//! le redit). Un code 1, 2 ou 5 ne change aucun travail existant (il n'y en a pas). Un travail **fini** (`fait == total`) rend 0 à tout `givre_courbe_pas` de plus, et
//! `givre_courbe_code` vaut alors 0.
//!
//! # Hors de portée
//!
//! [`givre_pixel`] rend `u32::MAX` hors de l'image ; [`givre_empreinte_mot`] rend 0 pour `i >= 8` ; [`givre_demarrer`] rend 0
//! pour un modèle qui n'existe pas (et ne change rien : tout modèle au-delà de 7) ; [`givre_avancer`] ne joue rien au-delà de la durée ; les
//! fonctions de la courbe sont dites plus haut (`NaN` hors de `η` de 0 à 3, code 5 sans travail).
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

/// L'état global : le modèle, la durée de son scénario, l'image de l'état courant (refaite quand l'état change) et la mesure
/// gardée de l'amas du moment (effacée dès que l'amas change).
struct Etat {
    modele: Modele,
    duree: u32,
    image: Image,
    mesure: Option<crate::correlation::Mesure>,
}

/// Le plus grand amas que [`givre_mesurer`] mesure : 10 000 sites (voir « La mesure de la corrélation »).
pub const MESURE_SITES_MAX: usize = 10_000;

/// Les sites (germe compris) que les modèles 4 à 7 (le claquage `η = 0` à `3`) font croître : 400, 250, 150 et 100 (voir « Les huit modèles »). La
/// `duree` d'un modèle est ce nombre moins un.
pub const CLAQUAGE_SITES: [u32; 4] = [400, 250, 150, 100];

/// Les demi-côtés des modèles 4 à 7 : `demi_cote_de_la_courbe(η, CLAQUAGE_SITES[η])`, soit 70, 52, 49 et 48 (voir « Les huit modèles »).
pub const CLAQUAGE_DEMI_COTE: [u32; 4] = [70, 52, 49, 48];

/// La plus grande `taille` (sites par amas, germe compris) d'un travail de la courbe : 100 (voir « La courbe D(η) à la demande »).
pub const COURBE_TAILLE_MAX: u32 = 100;

/// Le plus petit nombre de graines d'un travail de la courbe : 2 (une erreur-type n'existe pas pour une seule).
pub const COURBE_GRAINES_MIN: u32 = 2;

/// Le plus grand nombre de graines d'un travail de la courbe : 8.
pub const COURBE_GRAINES_MAX: u32 = 8;

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

/// (Re)construit le modèle `modele` (0 à 7, voir « Les huit modèles ») au temps 0 et rend 1 ; `graine` n'est lue que par la
/// DLA (`Amas::new(u64::from(graine), 200)`) et le claquage (`Claquage::new(u64::from(graine), demi_cote, η)`) : un `u32` entier, de 0 à
/// `u32::MAX`, sans troncature ni signe ; elle est ignorée par les flocons, qui n'ont pas de hasard (la même graine n'importe laquelle
/// donne le même flocon).
///
/// Remplace entièrement l'état précédent : rappelée après des pas, elle repart de zéro. Elle ne touche pas au travail de la courbe. Pour
/// un `modele` qui n'est pas de 0 à 7, ne change rien (l'état précédent reste en place, ou l'absence d'état) et rend 0.
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
    *garde = Some(Etat { modele, duree, image, mesure: None });
    1
}

/// Joue `min(n, duree - temps)` unités de temps (voir « Le temps ») et rend ce nombre : `n` si le scénario n'est pas fini, moins
/// (jusqu'à 0) s'il s'arrête avant, 0 pour `n = 0`. Un appel de `n` unités puis un de `m` donne exactement l'état d'un appel de
/// `n + m` (la dynamique ne dépend pas du découpage). Rend 0 et ne fait rien avant tout démarrage.
///
/// Pour la DLA, si une particule collait hors du réseau (un amas de 3000 sites tient largement dans le carré de côté 401 : cela
/// n'arrive pas en pratique), `givre_avancer` s'arrête là, sans panique, et rend ce qu'elle a joué ; de même pour le claquage si l'amas touchait la
/// frontière avant sa durée (aucun amas mesuré ne l'a fait, voir `MODELE.md`). Pour le claquage, chaque unité coûte la relaxation du champ : un appel de
/// beaucoup d'unités peut durer, au pire, le temps de la croissance entière (voir « Le temps »).
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
        etat.mesure = None;
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

/// La durée du scénario du modèle en cours : 2999, 400, 120, 800, 399, 249, 149 ou 99 (voir « Les huit modèles ») ; 0 avant tout démarrage.
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

/// Le nombre de sites de l'amas (germe compris) pour la DLA et le claquage, de cases gelées pour un flocon (voir « Les mesures ») ; 0 avant
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

/// Le rayon de l'amas (DLA et claquage) ou du cristal (flocon), voir « Les mesures » ; 0 avant tout démarrage.
#[unsafe(no_mangle)]
pub extern "C" fn givre_rayon() -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    match &etat.modele {
        Modele::Dla(amas) => amas.rayon(),
        Modele::Flocon(flocon) => flocon.rayon_cristal(),
    }
}

/// La compacité du flocon (`Flocon::compacite().vers_f64()`, de 0 à 1) ; `NaN` pour la DLA, le claquage et avant tout démarrage.
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

/// Mesure la dimension de l'amas du moment par la corrélation densité-densité (la fenêtre de [`crate::correlation`] pour la DLA, celle des petits amas de
/// [`crate::courbe`] pour le claquage), garde la courbe et rend son nombre de points ; 0 pour « pas de mesure » (voir « La mesure de la corrélation » :
/// flocon, amas trop petit ou de plus de [`MESURE_SITES_MAX`] sites, avant tout démarrage).
#[unsafe(no_mangle)]
pub extern "C" fn givre_mesurer() -> u32 {
    let mut garde = verrouiller();
    let Some(etat) = garde.as_mut() else { return 0 };
    let mesure = match &etat.modele {
        Modele::Dla(amas) => {
            if amas.sites().len() > MESURE_SITES_MAX {
                None
            } else {
                crate::correlation::mesure(amas.sites())
            }
        }
        Modele::Flocon(_) => None,
    };
    match mesure {
        Some(mesure) => {
            let points = mesure.rayons.len() as u32;
            etat.mesure = Some(mesure);
            points
        }
        None => {
            etat.mesure = None;
            0
        }
    }
}

/// Le `i`-ième rayon de la fenêtre de la mesure gardée ; 0 sans mesure ou pour `i` hors de la courbe (voir « La mesure de la
/// corrélation »).
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_correlation_rayon(i: u32) -> u32 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return 0 };
    etat.mesure.as_ref().and_then(|mesure| mesure.rayons.get(i as usize)).copied().unwrap_or(0)
}

/// Le `C(r)` du `i`-ième rayon de la mesure gardée, bit pour bit celui de `correlation::mesure` ; `NaN` sans mesure ou pour `i` hors
/// de la courbe.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_correlation_valeur(i: u32) -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    etat.mesure.as_ref().and_then(|mesure| mesure.valeurs.get(i as usize)).copied().unwrap_or(f64::NAN)
}

/// La dimension de la mesure gardée, bit pour bit celle de `correlation::mesure` ; `NaN` sans mesure.
#[unsafe(no_mangle)]
pub extern "C" fn givre_dimension() -> f64 {
    let garde = verrouiller();
    let Some(etat) = garde.as_ref() else { return f64::NAN };
    etat.mesure.as_ref().map_or(f64::NAN, |mesure| mesure.dimension)
}

/// Démarre un travail de la courbe `D(η)` : `graines` amas de `taille` sites pour chacun des `η = 0, 1, 2, 3` (voir « La courbe D(η) à la demande »).
/// Efface d'abord le travail précédent, quoi qu'il arrive. Rend 0 (accepté ; `givre_courbe_fait` vaut 0 et `givre_courbe_total` vaut `4 × graines`),
/// 1 si `taille` n'est pas de 1 à [`COURBE_TAILLE_MAX`] ou 2 si `graines` n'est pas de [`COURBE_GRAINES_MIN`] à [`COURBE_GRAINES_MAX`] (la taille est
/// vérifiée d'abord ; aucun travail n'est alors démarré). Ne fait croître aucun amas : c'est `givre_courbe_pas` qui calcule. Ne panique jamais.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_demarrer(taille: u32, graines: u32) -> u32 {
    todo!()
}

/// Fait **un** amas du travail de la courbe, le suivant (voir « La courbe D(η) à la demande »), et rend un code : 0 si l'amas est fait (ou si le travail est
/// déjà fini : rien ne change), 3, 4 ou 6 si cet amas échoue (le travail est alors arrêté, et tout appel de plus rend le même code sans rien faire), 5
/// s'il n'y a aucun travail. Quand l'amas fait est le dernier d'un `η`, le point de cet `η` est calculé. Ne panique jamais.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_pas() -> u32 {
    todo!()
}

/// Le nombre d'amas faits du travail de la courbe : 0 au démarrage, `4 × graines` à la fin ; 0 sans travail.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_fait() -> u32 {
    todo!()
}

/// Le nombre total d'amas du travail de la courbe : `4 × graines` ; 0 sans travail.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_total() -> u32 {
    todo!()
}

/// Le code du dernier appel de `givre_courbe_demarrer` ou de `givre_courbe_pas` (voir la table de « La courbe D(η) à la demande ») ; 0 avant tout appel.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_code() -> u32 {
    todo!()
}

/// La moyenne des dimensions du point de `eta` (0 à 3), bit pour bit celle de `essayer_point_de_la_courbe`, une fois les `graines` amas de cet `eta`
/// faits ; `NaN` avant, sans travail et pour `eta > 3`.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_moyenne(eta: u32) -> f64 {
    todo!()
}

/// L'erreur-type de la moyenne du point de `eta` (0 à 3), bit pour bit celle de `essayer_point_de_la_courbe`, une fois les `graines` amas de cet `eta`
/// faits ; `NaN` avant, sans travail et pour `eta > 3`.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_erreur_type(eta: u32) -> f64 {
    todo!()
}

/// Le repère publié de `eta` : `crate::courbe::publie(eta)` (`1,71`, `1,433`, `1,263` pour `η = 1, 2, 3`), `NaN` quand il n'y en a pas (`η = 0` et
/// `η >= 4`). Indépendant de tout travail et de tout modèle.
#[allow(unused_variables)]
#[unsafe(no_mangle)]
pub extern "C" fn givre_courbe_repere(eta: u32) -> f64 {
    todo!()
}

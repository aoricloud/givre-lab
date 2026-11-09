//! La courbe `D(η)` du claquage diélectrique (étape D2) : la dimension fractale `D` des amas de [`crate::claquage`], mesurée par la
//! corrélation densité-densité de [`crate::correlation`] sur une fenêtre faite pour les **petits** amas, moyennée sur des graines,
//! pour `η = 0, 1, 2, 3`, et mise **à côté** des points publiés (Hastings 2001), sans jamais les leur comparer par un test.
//!
//! # Ce que cette étape dit, et ne dit pas
//!
//! À nos tailles (100 à 300 sites ; le coût du champ de Laplace, `MODELE.md` étape D1, interdit mieux), la dimension mesurée est loin de
//! sa valeur limite : D2 ne dit que **l'ordre et la baisse** de `D(η)` (`D(0) > D(1) > D(2) > D(3)`), jamais un accord avec les valeurs
//! publiées. Les repères publiés ne sont que **affichés** avec l'écart mesuré ([`ecart`], [`csv_de_la_courbe`], [`png_de_la_courbe`]) ;
//! l'écart n'a pas un signe unique (mesuré à 100 sites : le banc est au-dessus de Hastings pour `η = 2` et `3`, au-dessous du rappel de la DLA pour
//! `η = 1`) et n'est pas un accord : ses amas sont petits, sa méthode est une corrélation sur un réseau et non une régression de `F_1` sur des
//! applications conformes, et le choix de la fenêtre déplace `D` de ±0,05.
//!
//! # Les conventions
//!
//! Un **site** est un couple d'entiers `(x, y)` (`i32`) ; une **taille** est un nombre de sites, germe compris (comme
//! [`crate::claquage::Claquage::croitre_jusqu_a`]) ; les **graines** sont des `u64` données **dans l'ordre**, sans que rien soit écarté ni
//! réordonné (une graine répétée compte autant de fois qu'elle est donnée). Tout est déterministe au bit près : les comptes de
//! [`crate::correlation`] sont des entiers, les seules fonctions de `f64` employées ici sont `+ - * /`, `sqrt` et `round`
//! (et [`crate::math::logarithme`], par [`crate::correlation`]) ; aucun fil d'exécution, aucune horloge, aucun `HashMap`.
//!
//! # La fenêtre des petits amas
//!
//! La fenêtre de Z1 ([`crate::correlation::fenetre`]) exige une étendue d'au moins 72 : un amas de 100 sites en a de 11 à 49 (mesuré), un
//! amas de 300 sites de 22 à 114, et seuls des amas de `η = 3` à 300 sites atteignent 72. [`fenetre_petit_amas`] la remplace, par une formule qui ne dépend **que de l'étendue** de l'amas
//! ([`crate::correlation::etendue`] : ni de l'ordre des sites, ni de leur position, ni de leur disposition, ni de la valeur que la
//! mesure donne). Avec `e` l'étendue (division entière partout) :
//!
//! ```text
//! dernier = max(FENETRE_DERNIER_MIN, e / FENETRE_DIVISEUR)       (4 et 6 : le dernier rayon est au moins 4, sinon e / 6)
//! la fenêtre est vide si dernier > e / 2  ou  dernier > RAYON_MAX   (RAYON_MAX de crate::correlation : 4096)
//! sinon : les entiers de FENETRE_DEBUT (2) à dernier inclus, dans l'ordre croissant
//! ```
//!
//! Elle a donc toujours au moins trois rayons (2, 3, 4) quand elle n'est pas vide ; elle est vide pour `e <= 7` ; elle est `2, 3, 4`
//! pour `8 <= e <= 29`, `2..=5` pour `30 <= e <= 35`, `2..=6` pour `36 <= e <= 41`, etc. La règle « le dernier rayon est au plus la moitié de
//! l'étendue » écarte un amas trop petit pour qu'un disque de ce rayon tienne dans sa boîte. Le choix de 2, 4 et 6 est expliqué
//! (sensibilité mesurée, formes à solution connue) dans `MODELE.md`, section D2 : il est fait sur des formes dont la dimension est
//! connue (trait, carré, disque, fractale de Vicsek, tapis de Sierpinski), jamais sur ce que donne le claquage.
//!
//! # La dimension d'un amas
//!
//! [`dimension_d_un_amas`]`(sites)` est `Some(`[`crate::correlation::dimension_correlation`]`(sites, &fenetre_petit_amas(sites)))`, **bit pour
//! bit**, et `None` si la fenêtre est vide ou si une valeur `C(r)` de la fenêtre est nulle (aucune paire à distance `<= r` : un amas en
//! morceaux éloignés). Elle ne panique jamais, y compris sur une liste vide (`None`) ou des sites répétés.
//!
//! # Le point de la courbe : la moyenne et l'erreur-type
//!
//! [`point_de_la_courbe`]`(eta, graines, taille, demi_cote)` fait croître, pour **chaque graine `g`, dans l'ordre donné**, un
//! [`crate::claquage::Claquage`] `Claquage::new(g, demi_cote, eta)` jusqu'à `taille` sites (par des appels de `essayer_ajouter`), prend sa
//! [`dimension_d_un_amas`] `D_g`, puis rend la [`moyenne_et_erreur_type`] de la liste `[D_g]` **dans l'ordre des graines** :
//!
//! ```text
//! n = nombre de graines (converti en f64)
//! moyenne      = (((0.0 + D_1) + D_2) + … + D_n) / n                       somme de gauche à droite, en partant de 0.0, puis une division
//! variance     = (((0.0 + e_1) + e_2) + … + e_n) / (n - 1.0)               avec e_i = (D_i - moyenne) * (D_i - moyenne), somme de gauche à droite
//! erreur_type  = sqrt(variance / n)                                        d'abord la division par n, puis la racine
//! ```
//!
//! La variance est la variance **corrigée** (`n - 1`), pas `n` ; l'erreur-type est l'écart-type de la moyenne (`sqrt(variance / n)`), pas
//! l'écart-type d'une graine. Une somme à l'envers, un diviseur `n` au lieu de `n - 1` ou une racine de la variance divisée par `sqrt(n)` donnent
//! d'autres bits (ou une autre valeur).
//!
//! **Les refus** (panique, comme [`crate::claquage::Claquage::ajouter`] : un point de la courbe fait de moins de sites que `taille` ou de moins
//! de graines que dit son effectif serait un mensonge silencieux) : moins de deux graines (message contenant « au moins deux graines », vérifié
//! **avant** de faire croître le moindre amas : ce n'est pas le message de [`moyenne_et_erreur_type`]) ;
//! un amas qui touche la frontière avant d'avoir `taille` sites (`essayer_ajouter` rend `None` ; message contenant « frontière » et la
//! graine) ; une [`dimension_d_un_amas`] qui est `None` (message contenant « fenêtre » : un amas de moins de 8 d'étendue, ou une
//! `taille` de 0 ou 1 par exemple). `demi_cote > DEMI_COTE_MAX` ou `eta > ETA_MAX` : la panique de [`crate::claquage::Claquage::new`].
//!
//! # Le demi-côté de la courbe
//!
//! [`point_de_la_courbe`] prend le `demi_cote` du caller. [`courbe`] le choisit elle-même par [`demi_cote_de_la_courbe`]`(eta, taille)` : une
//! règle affine en la taille, une par `η` (entiers, division entière), plafonnée à [`crate::claquage::DEMI_COTE_MAX`] :
//!
//! ```text
//! demi_cote = min(255, A_eta + B_eta * taille / 100)         (B_eta * taille d'abord, puis / 100, en entiers)
//!
//! eta :   0    1    2    3
//! A   :  30   32   22   12
//! B   :  10    8   18   36        soit, à 100 sites : 40, 40, 40, 48 ; à 300 sites : 60, 56, 76, 120
//! ```
//!
//! Elle est mesurée de 100 à 300 sites (rayon moyen des amas : de 9,6 à 21,7 à 100 sites et de 15,5 à 51,3 à 300 sites pour `η = 0` à `3` ;
//! `D` ne dépend du demi-côté qu'à la marge : de 40 à 80 à 100 sites, de 0,01 à 0,02 au plus) : pour `η >= 1` le demi-côté vaut de 2,1 à 3,1 fois le rayon moyen
//! (`η = 0` n'a pas de champ : le demi-côté n'y sert qu'à arrêter l'amas) et, sur 170 graines à 100 sites et 24 à 300 sites, aucun amas n'a touché la frontière.
//! Au-delà de 300 sites elle est extrapolée, et plafonnée. **`η > ETA_COURBE_MAX` (3) est refusé** (panique, message contenant « η ») : au-delà, le rayon des amas n'est pas
//! mesuré, et la transition `η = 4` est un débat ouvert (`MODELE.md`).
//!
//! # Les repères publiés
//!
//! - [`HASTINGS_2001_TABLE_II`] : M. B. Hastings, *Fractal to Nonfractal Phase Transition in the Dielectric Breakdown Model*, Phys. Rev. Lett. 87,
//!   175502 (2001), arXiv:cond-mat/0103312, tableau II, colonne « full set » : `η = 2` : `1,433` ; `η = 3` : `1,263` (lu dans le PDF
//!   de l'arXiv v2). Le tableau ne donne **aucune barre d'erreur** ; les amas de l'article sont de 15 000 (`η = 2`, 8 réalisations) et de 10 000
//!   particules (`η = 3`, 13 à 14 réalisations), obtenus par applications conformes itérées (**pas un réseau**), et `D` est l'inverse de la pente
//!   d'une régression de `ln F_1` contre `ln n` sur la moyenne des réalisations (`F_1 ∝ n^(1/D)`) : ni la méthode ni la taille ne sont celles d'ici.
//! - [`DLA_RAPPEL`] : `1,71` pour `η = 1`. **Ce n'est pas une mesure de Hastings** : il écrit seulement « roughly 1.7 (η = 1, DLA) »
//!   dans son introduction ; `1,71` est la valeur de la DLA que donnent Mathiesen, Jensen et Bakke (arXiv:0712.1739, section IV : « the DLA
//!   value D=1.71 for η=1 ») et que `MODELE.md` reprend (Davidovitch et coll., `1,713 ± 0,003`, est un calcul sur de petits amas).
//! - [`publie`]`(eta)` rend le repère affiché pour `η` (`Some` pour `1`, `2`, `3` ; `None` sinon, `η = 0` compris : l'amas plein n'a pas de repère
//!   ici, la dimension d'Eden `2` n'est pas une valeur de ce tableau) ; [`ecart`] est `point.moyenne - publie`.
//!
//! **Ces valeurs ne servent à aucun test de comparaison** : seules leur transcription et l'arithmétique de [`ecart`] sont testées.
//!
//! # Le CSV
//!
//! [`csv_de_la_courbe`]`(points)` est un texte ASCII : la ligne d'en-tête `eta,taille,graines,D,erreur,publie,ecart`, puis une ligne par point,
//! dans l'ordre donné, chacune finie par un seul `\n` (aucun `\r`), avec les sept champs séparés par une virgule :
//! `eta`, `taille` et `graines` en entiers décimaux ; `D` = `moyenne` et `erreur` = `erreur_type` écrits avec **six décimales** (`{:.6}` de Rust) ;
//! `publie` = [`publie`]`(eta)` avec **trois décimales** (`1.710`, `1.433`, `1.263`), **vide** quand il n'y en a pas ; `ecart` = [`ecart`] du point à
//! ce repère (calculé sur la `moyenne` non arrondie) avec six décimales, **vide** quand `publie` est vide. Une liste vide donne l'en-tête seul.
//!
//! # L'image
//!
//! [`png_de_la_courbe`]`(points)` est le PNG ([`crate::image::ecrire_png`]) de [`PNG_LARGEUR`]` × `[`PNG_HAUTEUR`] pixels (640 × 320) : un fond
//! [`PNG_FOND`] (blanc), puis, dans cet ordre, pour chaque point (dans l'ordre donné) sa **barre d'erreur** et son **marqueur**, puis, pour chaque point
//! qui a un repère publié, le **marqueur du repère**. Un pixel hors de l'image est ignoré. Avec `x(η) = 40 + 70 η` (entier) et
//! `y(D) = clamp(20 + round((2.2 - D) * 200.0), 0, 319)` (en `f64` : `2.2 - D`, puis `* 200.0`, puis `f64::round` de Rust, qui arrondit une moitié
//! exacte en s'éloignant de zéro ; puis conversion en entier `i64` par `as`, ajout de `20` en entier, et seulement alors le `clamp` entier ; pas de
//! `floor`, pas de troncature, pas d'ajout de `20.0` avant l'arrondi) :
//!
//! - la barre d'erreur du point : le trait vertical de la colonne `x(η)` entre les lignes `y(moyenne + erreur_type)` et `y(moyenne - erreur_type)` (bornes
//!   comprises), et, sur chacune de ces deux lignes, un trait horizontal de cinq pixels (colonnes `x(η) - 2` à `x(η) + 2`), tout en [`PNG_BANC`] ;
//! - le marqueur du point : le carré plein de 5 × 5 pixels centré sur `(x(η), y(moyenne))`, en [`PNG_BANC`] ;
//! - le marqueur du repère : le carré plein de 5 × 5 pixels centré sur `(x(η) + 12, y(publie))`, en [`PNG_PUBLIE`].
//!
//! Les autres pixels de l'image (grille, axes, repères de graduation) sont libres : les tests ne les regardent pas, sauf que **les coins `(0, 0)` et `(639, 319)` sont de fond**,
//! qu'**aucun pixel de grille ou d'axe n'a la couleur [`PNG_BANC`] ni [`PNG_PUBLIE`]**, et que **ce décor, s'il y en a, est dessiné juste après le fond, avant
//! toute barre et tout marqueur** (il ne recouvre jamais un pixel fixé ci-dessus). Un conseil (le prototype le fait, ce n'est pas une règle) : une grille
//! grise `[225, 225, 225]` aux `D = 1,0 ; 1,25 ; 1,5 ; 1,75 ; 2,0` (colonnes 20 à 620), un axe gris foncé `[90, 90, 90]` vertical à la colonne 20 (lignes 20 à 300) et horizontal à
//! la ligne 300 (`D = 0,8` ; colonnes 20 à 620), et un trait de graduation de six pixels sous la colonne de chaque `η` de 0 à 8. Les marqueurs de repère
//! sont dessinés en dernier, donc par-dessus tout le reste.

use crate::claquage::{Claquage, DEMI_COTE_MAX};
use crate::correlation::{RAYON_MAX, correlation, dimension_correlation, etendue};

/// Le premier rayon de la fenêtre des petits amas : 2 (voir [`fenetre_petit_amas`]).
pub const FENETRE_DEBUT: u32 = 2;

/// Le diviseur de l'étendue qui donne le dernier rayon de la fenêtre des petits amas : 6 (voir [`fenetre_petit_amas`]).
pub const FENETRE_DIVISEUR: u32 = 6;

/// Le plus petit dernier rayon de la fenêtre des petits amas : 4 (voir [`fenetre_petit_amas`]).
pub const FENETRE_DERNIER_MIN: u32 = 4;

/// Le plus grand `η` que [`demi_cote_de_la_courbe`] (donc [`courbe`]) accepte : 3.
pub const ETA_COURBE_MAX: u32 = 3;

/// Les repères de Hastings (2001), tableau II, colonne « full set » : les couples `(η, D)` de `η = 2` (`1,433`) et de `η = 3` (`1,263`). Voir
/// « Les repères publiés » dans la documentation du module : sans barre d'erreur, par applications conformes (pas un réseau), amas de 10 000 à
/// 15 000 particules. Jamais comparés à une mesure par un test.
pub const HASTINGS_2001_TABLE_II: [(f64, f64); 2] = [(2.0, 1.433), (3.0, 1.263)];

/// Le rappel de la dimension de la DLA, `1,71`, affiché pour `η = 1` : **pas une mesure de Hastings** (il écrit « roughly 1.7 »), mais la valeur
/// de Mathiesen, Jensen et Bakke, arXiv:0712.1739, section IV. Jamais comparé à une mesure par un test.
pub const DLA_RAPPEL: f64 = 1.71;

/// La largeur en pixels de l'image de [`png_de_la_courbe`] : 640.
pub const PNG_LARGEUR: u32 = 640;

/// La hauteur en pixels de l'image de [`png_de_la_courbe`] : 320.
pub const PNG_HAUTEUR: u32 = 320;

/// La couleur du fond de l'image : blanc `[255, 255, 255]`.
pub const PNG_FOND: [u8; 3] = [255, 255, 255];

/// La couleur des points du banc (marqueurs et barres d'erreur) : bleu `[31, 119, 180]`.
pub const PNG_BANC: [u8; 3] = [31, 119, 180];

/// La couleur des repères publiés : rouge `[214, 39, 40]`.
pub const PNG_PUBLIE: [u8; 3] = [214, 39, 40];

/// La fenêtre d'ajustement des petits amas : les rayons de la corrélation, par la formule de la documentation du module (« La fenêtre des petits
/// amas ») qui ne dépend que de l'étendue de `sites` : de [`FENETRE_DEBUT`] à `max(`[`FENETRE_DERNIER_MIN`]`, étendue / `[`FENETRE_DIVISEUR`]`)`, vide si
/// ce dernier rayon dépasse `étendue / 2` ou [`crate::correlation::RAYON_MAX`]. Ne panique jamais (liste vide : vide).
#[allow(unused_variables)]
pub fn fenetre_petit_amas(sites: &[(i32, i32)]) -> Vec<u32> {
    let e = etendue(sites);
    let dernier = FENETRE_DERNIER_MIN.max(e / FENETRE_DIVISEUR);
    if dernier > e / 2 || dernier > RAYON_MAX {
        return Vec::new();
    }
    (FENETRE_DEBUT..=dernier).collect()
}

/// La dimension de l'amas `sites` : `Some(dimension_correlation(sites, &fenetre_petit_amas(sites)))` bit pour bit, `None` si la fenêtre est vide ou
/// si une valeur `C(r)` de la fenêtre est nulle. Ne panique jamais.
#[allow(unused_variables)]
pub fn dimension_d_un_amas(sites: &[(i32, i32)]) -> Option<f64> {
    let rayons = fenetre_petit_amas(sites);
    let &dernier = rayons.last()?;
    let courbe = correlation(sites, dernier);
    for &rayon in &rayons {
        if courbe[(rayon - 1) as usize] == 0.0 {
            return None;
        }
    }
    Some(dimension_correlation(sites, &rayons))
}

/// La moyenne et l'erreur-type d'une liste de valeurs, à la lettre de la documentation du module (« Le point de la courbe ») : la somme de gauche à
/// droite en partant de `0.0` divisée par `n`, la variance corrigée (somme des `(v - moyenne) * (v - moyenne)` de gauche à droite divisée par
/// `n - 1`), l'erreur-type `sqrt(variance / n)`. Rend `(moyenne, erreur_type)`.
///
/// # Panique
///
/// Si `valeurs` a moins de deux éléments (message contenant « au moins deux »).
#[allow(unused_variables)]
pub fn moyenne_et_erreur_type(valeurs: &[f64]) -> (f64, f64) {
    assert!(valeurs.len() >= 2, "une moyenne et une erreur-type demandent au moins deux valeurs");
    let n = valeurs.len() as f64;
    let mut somme = 0.0;
    for &valeur in valeurs {
        somme += valeur;
    }
    let moyenne = somme / n;
    let mut carres = 0.0;
    for &valeur in valeurs {
        carres += (valeur - moyenne) * (valeur - moyenne);
    }
    let variance = carres / (n - 1.0);
    (moyenne, (variance / n).sqrt())
}

/// Le demi-côté que [`courbe`] donne à ses amas : `min(255, A + B * taille / 100)` avec `(A, B)` = `(30, 10)`, `(32, 8)`, `(22, 18)`, `(12, 36)` pour
/// `η = 0, 1, 2, 3` (voir « Le demi-côté de la courbe » dans la documentation du module ; les produits et la division sont entiers).
///
/// # Panique
///
/// Si `eta > ETA_COURBE_MAX` (message contenant « η »).
#[allow(unused_variables)]
pub fn demi_cote_de_la_courbe(eta: u32, taille: usize) -> u32 {
    let (a, b) = match eta {
        0 => (30_usize, 10_usize),
        1 => (32, 8),
        2 => (22, 18),
        3 => (12, 36),
        _ => panic!("η = {eta} : au-delà de ETA_COURBE_MAX ({ETA_COURBE_MAX})"),
    };
    (a + b * taille / 100).min(DEMI_COTE_MAX as usize) as u32
}

/// Un point de la courbe `D(η)` : la dimension moyenne de `graines` amas de `taille` sites à l'exposant `eta`, et l'erreur-type de cette moyenne.
#[derive(Clone, Debug, PartialEq)]
pub struct PointDeLaCourbe {
    /// L'exposant `η` des amas.
    pub eta: u32,
    /// Le nombre de sites de chaque amas, germe compris.
    pub taille: usize,
    /// Le nombre de graines moyennées.
    pub graines: usize,
    /// La moyenne des [`dimension_d_un_amas`] sur les graines (voir « Le point de la courbe »).
    pub moyenne: f64,
    /// L'erreur-type de cette moyenne : `sqrt(variance corrigée / n)`.
    pub erreur_type: f64,
}

/// Le point de la courbe de l'exposant `eta` : un amas de `taille` sites par graine de `graines` (dans l'ordre donné), de demi-côté `demi_cote`, sa
/// [`dimension_d_un_amas`], puis la [`moyenne_et_erreur_type`] de ces dimensions dans l'ordre des graines (voir « Le point de la courbe » dans la
/// documentation du module). `graines` du résultat est `graines.len()`.
///
/// # Panique
///
/// Moins de deux graines (« au moins deux graines »), un amas qui touche la frontière avant `taille` sites (« frontière »), une dimension indéfinie
/// (« fenêtre »), ou les paniques de [`crate::claquage::Claquage::new`] (`demi_cote > 255`, `eta > 8`).
#[allow(unused_variables)]
pub fn point_de_la_courbe(eta: u32, graines: &[u64], taille: usize, demi_cote: u32) -> PointDeLaCourbe {
    assert!(graines.len() >= 2, "un point de la courbe demande au moins deux graines");
    let mut dimensions = Vec::with_capacity(graines.len());
    for &graine in graines {
        let mut claquage = Claquage::new(graine, demi_cote, eta);
        while claquage.sites().len() < taille {
            assert!(
                claquage.essayer_ajouter().is_some(),
                "la graine {graine} touche la frontière avant d'atteindre {taille} sites"
            );
        }
        let dimension = dimension_d_un_amas(claquage.sites())
            .unwrap_or_else(|| panic!("la graine {graine} : aucune dimension (fenêtre vide)"));
        dimensions.push(dimension);
    }
    let (moyenne, erreur_type) = moyenne_et_erreur_type(&dimensions);
    PointDeLaCourbe { eta, taille, graines: graines.len(), moyenne, erreur_type }
}

/// La courbe : un [`PointDeLaCourbe`] par `η` de `etas`, **dans l'ordre donné** (un `η` répété donne un point répété), chacun étant
/// `point_de_la_courbe(eta, graines, taille, demi_cote_de_la_courbe(eta, taille))`. Une liste vide donne une courbe vide.
///
/// # Panique
///
/// Comme [`point_de_la_courbe`] et [`demi_cote_de_la_courbe`] (`eta > ETA_COURBE_MAX`).
#[allow(unused_variables)]
pub fn courbe(etas: &[u32], graines: &[u64], taille: usize) -> Vec<PointDeLaCourbe> {
    etas.iter().map(|&eta| point_de_la_courbe(eta, graines, taille, demi_cote_de_la_courbe(eta, taille))).collect()
}

/// Le repère publié affiché pour `eta` : `Some(`[`DLA_RAPPEL`]`)` pour `1`, la valeur de [`HASTINGS_2001_TABLE_II`] pour `2` et `3`, `None` pour
/// tout autre `eta` (`0` et `4` ou plus compris).
#[allow(unused_variables)]
pub fn publie(eta: u32) -> Option<f64> {
    match eta {
        1 => Some(DLA_RAPPEL),
        2 => Some(HASTINGS_2001_TABLE_II[0].1),
        3 => Some(HASTINGS_2001_TABLE_II[1].1),
        _ => None,
    }
}

/// L'écart du banc au repère publié : `point.moyenne - publie` (positif quand le banc est au-dessus). Une soustraction en `f64`, rien d'autre.
#[allow(unused_variables)]
pub fn ecart(point: &PointDeLaCourbe, publie: f64) -> f64 {
    point.moyenne - publie
}

/// Le CSV de la courbe (voir « Le CSV » dans la documentation du module).
#[allow(unused_variables)]
pub fn csv_de_la_courbe(points: &[PointDeLaCourbe]) -> String {
    todo!()
}

/// L'image de la courbe (voir « L'image » dans la documentation du module).
#[allow(unused_variables)]
pub fn png_de_la_courbe(points: &[PointDeLaCourbe]) -> Vec<u8> {
    todo!()
}

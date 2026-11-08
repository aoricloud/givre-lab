//! Étape Z1 : la corrélation sur des FORMES à solution connue, et ses invariances EXACTES. Valeurs attendues :
//! `references/etape-z1-valeurs.md`, identifiants `V-Z1-xx`.
//!
//! Les formes (trait, diagonale, carré plein, fractale de Vicsek en croix et en X, tapis de Sierpinski) sont fabriquées ICI
//! (`etape_z1_commun`), comme les images de `etape_1_mesures` : un générateur dans `src/` rendrait le test circulaire.
//!
//! Deux choses sont éprouvées, de natures différentes :
//!
//! - la **justesse du calcul** : `dimension_correlation` sur la fenêtre de la doc donne, à `1e-9`, la valeur qu'un outil indépendant
//!   (Python, un compteur de paires qui passe par les déplacements, `fractions` et `decimal` à 60 chiffres : `essai-z1-outils/z1_ref.py`)
//!   calcule d'après la doc seule. Cette valeur n'est PAS la dimension de la forme (`1`, `2`, `ln 5 / ln 3`, `ln 8 / ln 3`) : sur une
//!   grille finie et une fenêtre finie, la corrélation s'en écarte (de 0,01 à 0,07 pour la fenêtre de la doc) ;
//! - la **fidélité à la forme** : l'écart de cette valeur à la dimension de la forme reste dans une tolérance mesurée sur le prototype.
//!   Les écarts observés pour la fenêtre de la doc (4 à `étendue / 8`) sont, en valeur absolue : trait de 243 sites 0,038, trait de 729
//!   0,028, diagonale de 243 0,069, carré plein de 81 0,072, Vicsek en croix de niveau 5 0,021 et de niveau 6 0,014, Vicsek en X de niveau 5
//!   0,044, tapis de niveau 4 0,063, de niveau 5 0,052 et de niveau 6 0,043. Le plus grand est 0,072 ; la tolérance est `0,09`, soit 25 % de plus
//!   que le pire écart observé, laissée par la discrétisation d'un trait oblique, d'un bord et d'une fenêtre étroite.
//!
//! Les invariances sont EXACTES (égalité d'entiers, de bits) : la corrélation compte toutes les paires, donc ne dépend ni de la
//! position, ni de l'orientation par quart de tour ou miroir, ni de l'ordre des sites.

mod etape_z1_commun;

use etape_z1_commun::{
    bits, carre, correlation_attendue, diagonale, melanger, miroir, paires_naives, proche, quart_de_tour, tapis,
    trait_horizontal, translater, transposer, vicsek_croix, vicsek_x,
};
use givre_lab::correlation::{correlation, dimension_correlation, etendue, fenetre, mesure, paires_par_distance2};
use givre_lab::dla::Amas;

/// `ln 5 / ln 3` (Vicsek) et `ln 8 / ln 3` (tapis de Sierpinski), à 17 chiffres (Python `decimal`, 60 chiffres) : les dimensions de
/// similarité des constructions « 5 carrés sur 9 » et « 8 carrés sur 9 ».
const DIM_VICSEK: f64 = 1.464_973_520_717_927_2;
const DIM_TAPIS: f64 = 1.892_789_260_714_372_3;

/// L'écart maximal toléré entre la corrélation de la fenêtre de la doc et la dimension de la forme (voir l'en-tête).
const TOLERANCE_FORME: f64 = 0.09;

/// La dimension qu'un outil indépendant (Python) donne à chaque forme sur la fenêtre de la doc, à 17 chiffres, avec la forme et sa dimension.
struct Forme {
    nom: &'static str,
    sites: Vec<(i32, i32)>,
    attendu_python: f64,
    dimension_de_la_forme: f64,
}

fn les_formes() -> Vec<Forme> {
    let forme =
        |nom, sites, attendu_python, dimension_de_la_forme| Forme { nom, sites, attendu_python, dimension_de_la_forme };
    vec![
        forme("trait de 243 sites", trait_horizontal(243), 0.961_782_858_957_057_2, 1.0),
        forme("trait de 729 sites", trait_horizontal(729), 0.971_871_100_067_669_7, 1.0),
        forme("diagonale de 243 sites", diagonale(243), 1.068_733_481_900_645_3, 1.0),
        forme("carré plein de 81 × 81", carre(81), 1.928_362_957_541_490_3, 2.0),
        forme("Vicsek en croix, niveau 5", vicsek_croix(5), 1.443_678_860_625_069, DIM_VICSEK),
        forme("Vicsek en X, niveau 5", vicsek_x(5), 1.508_855_199_379_124_8, DIM_VICSEK),
        forme("tapis de Sierpinski, niveau 4", tapis(4), 1.829_664_145_714_223, DIM_TAPIS),
        forme("tapis de Sierpinski, niveau 5", tapis(5), 1.840_848_557_835_892_5, DIM_TAPIS),
    ]
}

/// juste : sur chacune des huit formes, `dimension_correlation(sites, &fenetre(sites))` est la valeur de l'outil indépendant à `1e-9` (V-Z1-13),
/// et reste à moins de `0,09` de la dimension de la forme (V-Z1-14, tolérance mesurée, voir l'en-tête). Les valeurs de Python : trait de 243 sites
/// `0,9617828589570572`, de 729 `0,9718711000676697`, diagonale de 243 `1,0687334819006453`, carré 81 × 81 `1,9283629575414903`, Vicsek en
/// croix de niveau 5 `1,4436788606250689`, en X `1,5088551993791248`, tapis de niveau 4 `1,829664145714223`, de niveau 5 `1,8408485578358925`.
/// Un compte de paires faux, une mauvaise fenêtre, un logarithme oublié, un `+ 2` oublié, ou un `n` pour un `n - 1` qui change la pente se voient ici
/// (et un diviseur `n - 1` au lieu de `n`, qui ne change pas la pente, se voit dans `etape_z1_exact`).
///
/// Ne couvre pas : la dimension asymptotique (le calcul est fait sur des grilles de 80 à 728 de côté, et la discrétisation donne 0,01 à 0,07
/// d'écart), la DLA (voir `etape_z1_dla`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn dimension_des_formes() {
    for forme in les_formes() {
        let rayons = fenetre(&forme.sites);
        assert!(!rayons.is_empty(), "{} : pas de fenêtre", forme.nom);
        let obtenu = dimension_correlation(&forme.sites, &rayons);
        proche(obtenu, forme.attendu_python, 1e-9, &format!("{} : valeur de l'outil indépendant", forme.nom));
        proche(obtenu, forme.dimension_de_la_forme, TOLERANCE_FORME, &format!("{} : dimension de la forme", forme.nom));
    }
}

/// juste : la dimension de chaque forme est la même, au bit près, quand on la calcule par `mesure` (la fonction de la page) et par
/// `dimension_correlation` avec la fenêtre : `mesure(sites).dimension == dimension_correlation(sites, &fenetre(sites))` (V-Z1-15).
///
/// Ne couvre pas : la valeur (voir `dimension_des_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn mesure_et_dimension_sont_la_meme_chose() {
    for forme in les_formes().into_iter().take(6) {
        let rayons = fenetre(&forme.sites);
        let m = mesure(&forme.sites).expect("la forme a une fenêtre");
        assert_eq!(m.rayons, rayons, "{}", forme.nom);
        assert_eq!(m.dimension.to_bits(), dimension_correlation(&forme.sites, &rayons).to_bits(), "{}", forme.nom);
        // L'égalité des deux appels ne suffit pas (deux constantes sont égales) : la valeur est celle de l'outil indépendant.
        proche(m.dimension, forme.attendu_python, 1e-9, &format!("{} : valeur de l'outil indépendant", forme.nom));
    }
}

/// Un amas de DLA de 1500 sites (graine 3, `Amas::new(3, 200)`) : une forme sans symétrie, comme entrée des invariances.
fn amas_dla() -> Vec<(i32, i32)> {
    let mut amas = Amas::new(3, 200);
    amas.croitre_jusqu_a(1500);
    amas.sites().to_vec()
}

/// Les transformations du plan qui laissent les distances entre sites inchangées, ou seulement l'ordre : chacune avec son nom.
fn transformations(sites: &[(i32, i32)]) -> Vec<(&'static str, Vec<(i32, i32)>)> {
    let mut inverse = sites.to_vec();
    inverse.reverse();
    vec![
        ("translation (+137, -2048)", translater(sites, (137, -2048))),
        ("translation par ±2^30", translater(sites, (1 << 30, -(1 << 30)))),
        ("quart de tour", quart_de_tour(sites)),
        ("trois quarts de tour", quart_de_tour(&quart_de_tour(&quart_de_tour(sites)))),
        ("demi-tour", quart_de_tour(&quart_de_tour(sites))),
        ("miroir (-x, y)", miroir(sites)),
        ("échange des axes", transposer(sites)),
        ("ordre inversé", inverse),
        ("ordre mélangé (graine 5)", melanger(sites, 5)),
        ("ordre mélangé (graine 6)", melanger(sites, 6)),
        ("quart de tour, translation et mélange", melanger(&translater(&quart_de_tour(sites), (-9, 31)), 7)),
    ]
}

/// juste : les comptes de paires, `C(r)`, l'étendue, la fenêtre et la mesure d'une forme ne changent PAS, au bit près, quand on translate tous
/// les sites (de `(137, -2048)` puis de `±2^30`), qu'on les tourne d'un quart, de trois quarts de tour ou d'un demi-tour, qu'on les retourne en
/// miroir ou qu'on échange les axes, ou qu'on change l'ordre de la liste (inversé, mélangé de deux façons) (V-Z1-16). Les formes : le Vicsek en
/// croix de niveau 4 (625 sites, sans symétrie diagonale), un amas de DLA de 1500 sites (aucune symétrie), et un petit amas au hasard. Une
/// corrélation qui compterait seulement les paires `(0, j)` depuis le premier site, ou depuis le centre de masse, ou qui dépendrait de l'ordre
/// ou de la position (un `i32` qui déborde à ±2^30), est vue ici.
///
/// Ne couvre pas : les rotations qui ne sont pas des quarts de tour (elles ne laissent pas la grille invariante), ni les amas de plus de 2^31
/// de côté.
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn invariances_exactes() {
    let formes: Vec<(&str, Vec<(i32, i32)>)> = vec![
        ("Vicsek en croix, niveau 4", vicsek_croix(4)),
        ("amas de DLA de 1500 sites", amas_dla()),
        ("amas au hasard", etape_z1_commun::amas_au_hasard(11, 400, 40)),
    ];
    for (nom, sites) in formes {
        let paires = paires_par_distance2(&sites, 40);
        let c = correlation(&sites, 40);
        let ref_mesure = mesure(&sites);
        // L'invariance ne vaut que si la référence est juste (sinon deux listes vides ou deux zéros sont égaux) : les comptes sont ceux du
        // compteur naïf et `C(r)` est la formule de la doc appliquée à ces comptes.
        assert_eq!(paires, paires_naives(&sites, 40), "{nom} : paires contre le compteur naïf");
        assert_eq!(bits(&c), bits(&correlation_attendue(sites.len(), &paires, 40)), "{nom} : C(r) contre la formule");
        assert!(
            paires.iter().skip(1).any(|&p| p > 0) && c.iter().all(|&v| v > 0.0),
            "{nom} : un amas qui a des paires"
        );
        for (transformation, autres) in transformations(&sites) {
            let contexte = format!("{nom}, {transformation}");
            assert_eq!(paires_par_distance2(&autres, 40), paires, "{contexte} : paires");
            assert_eq!(bits(&correlation(&autres, 40)), bits(&c), "{contexte} : C(r)");
            assert_eq!(etendue(&autres), etendue(&sites), "{contexte} : étendue");
            assert_eq!(fenetre(&autres), fenetre(&sites), "{contexte} : fenêtre");
            let m = mesure(&autres);
            assert_eq!(m.is_some(), ref_mesure.is_some(), "{contexte} : mesure présente");
            if let (Some(a), Some(b)) = (&m, &ref_mesure) {
                assert_eq!(a.rayons, b.rayons, "{contexte} : rayons");
                assert_eq!(bits(&a.valeurs), bits(&b.valeurs), "{contexte} : valeurs");
                assert_eq!(a.dimension.to_bits(), b.dimension.to_bits(), "{contexte} : dimension");
            }
        }
    }
}

/// juste : la fenêtre ne dépend QUE de l'amas, jamais de la valeur qu'il donne (V-Z1-17). Une même forme à des tailles différentes (Vicsek de
/// niveaux 4, 5 et 6 ; étendues 80, 242 et 728) a les fenêtres `4..=10`, `4..=30` et `4..=91` ; deux amas très différents de même étendue (le
/// trait de 243 sites et le Vicsek en croix de niveau 5, étendue 242) ont la MÊME fenêtre ; et la fenêtre ne change pas quand on retire ou
/// ajoute des sites à l'intérieur de la boîte de l'amas (le Vicsek de niveau 5 moins le tiers de ses sites, en gardant ceux des coins de la
/// boîte, a la même fenêtre que lui).
///
/// Ne couvre pas : la valeur de la dimension (voir `dimension_des_formes`).
#[test]
#[cfg_attr(not(feature = "etape-z1"), ignore)]
fn fenetre_ne_depend_que_de_l_amas() {
    assert_eq!(fenetre(&vicsek_croix(4)), (4..=10).collect::<Vec<u32>>());
    assert_eq!(fenetre(&vicsek_croix(5)), (4..=30).collect::<Vec<u32>>());
    assert_eq!(fenetre(&vicsek_croix(6)), (4..=91).collect::<Vec<u32>>());
    assert_eq!(fenetre(&trait_horizontal(243)), fenetre(&vicsek_croix(5)));
    assert_eq!(fenetre(&tapis(5)), fenetre(&vicsek_croix(5)));
    let croix = vicsek_croix(5);
    let (x_max, y_max) = croix.iter().fold((0, 0), |(a, b), &(x, y)| (a.max(x), b.max(y)));
    let allege: Vec<(i32, i32)> = croix
        .iter()
        .enumerate()
        .filter(|&(i, &(x, y))| i % 3 != 0 || x == 0 || y == 0 || x == x_max || y == y_max)
        .map(|(_, &site)| site)
        .collect();
    assert!(allege.len() < croix.len() && etendue(&allege) == etendue(&croix));
    assert_eq!(fenetre(&allege), fenetre(&croix));
}

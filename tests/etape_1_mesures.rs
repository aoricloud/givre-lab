//! Étape 1 : les mesures (`givre_lab::mesures`) : la pente des moindres carrés, le comptage de boîtes, la masse-rayon et le
//! rayon de giration. Valeurs attendues : `references/etape-1-valeurs.md`, identifiants `V-PENTE-xx`, `V-BOITE-xx`,
//! `V-MASSE-xx`, `V-GIR-xx`, `V-BMP-xx`.
//!
//! Les images dont la réponse est connue (un segment, un carré plein, la fractale de Vicsek, le tapis de Sierpinski) sont
//! fabriquées ICI, par une boucle de quelques lignes sur les chiffres en base 3 : un générateur dans `src/` rendrait le test
//! circulaire. Les valeurs attendues viennent de calculs exacts (rationnels, entiers) ou à 60 chiffres (`decimal`) faits par
//! Python, un outil indépendant ; les constantes publiées (la dimension de Vicsek et du tapis) sont des définitions
//! classiques. Aucune valeur ne vient du code jugé.

use givre_lab::mesures::{
    Bitmap, boites_occupees, dimension_boites, dimension_masse_rayon, masse_dans_rayon, pente, points_masse_rayon,
    rayon_de_giration,
};

/// `ln 5 / ln 3` (la dimension de Vicsek) et `ln 8 / ln 3` (celle du tapis de Sierpinski), à 17 chiffres (Python `decimal`, 60
/// chiffres).
const DIM_VICSEK: f64 = 1.464_973_520_717_927_2;
const DIM_TAPIS: f64 = 1.892_789_260_714_372_3;

/// La tolérance des mesures qui valent EXACTEMENT une valeur à l'arrondi près : l'erreur mesurée est de l'ordre de `1e-15`
/// (voir la fiche, « tolérances mesurées »), et `1e-12` laisse la place à un autre logarithme de précision `5e-16`.
const TOL: f64 = 1e-12;

fn proche(a: f64, b: f64, tol: f64, ce_qui: &str) {
    assert!((a - b).abs() <= tol, "{ce_qui} : obtenu {a}, attendu {b} (tolérance {tol})");
}

// ------------------------------------------------------------------------------------------------ images synthétiques

/// Une image de côté `cote` où le pixel `(x, y)` est allumé si `(x - dx, y - dy)` est dans `[0, 3^niveau)²` et si chacun de
/// ses `niveau` chiffres base 3 `(x mod 3, y mod 3)` (du plus faible au plus fort) vérifie `ok`.
fn fractale(niveau: u32, cote: usize, dx: usize, dy: usize, ok: impl Fn(usize, usize) -> bool) -> Bitmap {
    let taille = 3_usize.pow(niveau);
    let mut image = Bitmap::new(cote, cote);
    for y in 0..taille {
        for x in 0..taille {
            let (mut a, mut b, mut bon) = (x, y, true);
            for _ in 0..niveau {
                bon &= ok(a % 3, b % 3);
                a /= 3;
                b /= 3;
            }
            if bon {
                image.allumer(x + dx, y + dy);
            }
        }
    }
    image
}

/// La fractale de Vicsek en croix : des neuf cases d'un carré, on garde le centre et les quatre voisines par un côté.
fn vicsek_croix(niveau: u32, cote: usize, dx: usize, dy: usize) -> Bitmap {
    fractale(niveau, cote, dx, dy, |a, b| matches!((a, b), (1, 0) | (0, 1) | (1, 1) | (2, 1) | (1, 2)))
}

/// La fractale de Vicsek en X : on garde le centre et les quatre coins.
fn vicsek_x(niveau: u32, cote: usize) -> Bitmap {
    fractale(niveau, cote, 0, 0, |a, b| matches!((a, b), (0, 0) | (2, 0) | (0, 2) | (2, 2) | (1, 1)))
}

/// Le tapis de Sierpinski : on retire la case centrale.
fn tapis(niveau: u32, cote: usize) -> Bitmap {
    fractale(niveau, cote, 0, 0, |a, b| (a, b) != (1, 1))
}

const COTES_3: [usize; 5] = [1, 3, 9, 27, 81];

fn comptes(image: &Bitmap, cotes: &[usize]) -> Vec<usize> {
    cotes.iter().map(|&c| boites_occupees(image, c)).collect()
}

// ------------------------------------------------------------------------------------------------------------- pente

/// juste : la pente des moindres carrés sur des jeux écrits à la main, avec leur solution exacte en rationnels (Python
/// `fractions`, V-PENTE-01 à 03) : `(0,1) (1,3) (2,4) (3,8)` donne `11/5 = 2,2` (une pente à deux points donnerait `7/3`, la pente
/// de `x` contre `y` donne `11/26`) ; cinq points non triés de signes mêlés donnent `-231/158` ; une droite exacte `y = 3x - 2`
/// donne 3 ; un `y` constant donne 0.
///
/// Ne couvre pas : la précision sur de longues listes (les mesures suivantes s'en servent).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn pente_a_la_main() {
    let xs = [0.0, 1.0, 2.0, 3.0];
    let ys = [1.0, 3.0, 4.0, 8.0];
    proche(pente(&xs, &ys), 2.2, TOL, "pente de y contre x");
    assert!((pente(&xs, &ys) - 7.0 / 3.0).abs() > 0.1, "ce n'est pas la pente entre les deux extrémités");
    proche(pente(&ys, &xs), 11.0 / 26.0, TOL, "pente de x contre y (celle de y contre x n'est pas l'inverse)");
    proche(
        pente(&[2.0, -1.0, 0.5, 4.0, 3.0], &[1.0, 5.0, 2.5, -3.0, 0.0]),
        -231.0 / 158.0, // = -1,4620253164556962
        TOL,
        "cinq points non triés",
    );
    proche(pente(&[0.5, 1.5, 4.0, 9.0], &[-0.5, 2.5, 10.0, 25.0]), 3.0, TOL, "droite exacte y = 3x - 2");
    proche(pente(&[1.0, 2.0, 5.0], &[7.0, 7.0, 7.0]), 0.0, TOL, "y constant");
}

/// juste : la pente ne change pas (à l'arrondi près) quand on permute les points, quand on translate les `y`, et elle est
/// multipliée par `k` quand on multiplie les `y` par `k` ; et elle reste exacte quand les `x` valent `10^8 + i` (une formule non
/// centrée `n·Σxy - Σx·Σy` y perd tous ses chiffres : `Σx²` dépasse `2^53`). Les valeurs viennent du jeu de V-PENTE-01
/// (`2,2`) (V-PENTE-04).
///
/// Ne couvre pas : l'ordre exact des sommes (la doc le fixe, les tests n'exigent que l'arrondi près).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn pente_invariances_et_centrage() {
    let xs = [0.0, 1.0, 2.0, 3.0];
    let ys = [1.0, 3.0, 4.0, 8.0];
    proche(pente(&[3.0, 0.0, 2.0, 1.0], &[8.0, 1.0, 4.0, 3.0]), 2.2, TOL, "points permutés");
    let ys_translates: Vec<f64> = ys.iter().map(|y| y + 1000.0).collect();
    proche(pente(&xs, &ys_translates), 2.2, 1e-10, "y translatés de 1000");
    let ys_fois_7: Vec<f64> = ys.iter().map(|y| 7.0 * y).collect();
    proche(pente(&xs, &ys_fois_7), 15.4, 1e-10, "y multipliés par 7");
    let xs_loin: Vec<f64> = xs.iter().map(|x| 1.0e8 + x).collect();
    proche(pente(&xs_loin, &ys), 2.2, 1e-9, "x autour de 10^8 (formule centrée)");
}

/// juste : trois entrées sans pente sont refusées par une panique (doc de `pente`) : longueurs différentes, un seul point, tous
/// les `x` égaux ; et deux entrées valides au bord du refus sont acceptées (deux points `(0,0) (1,1)` : 1 ; `(3,5) (4,5)` : 0), pour
/// qu'un corps qui panique toujours n'y passe pas (V-PENTE-05).
///
/// Ne couvre pas : le texte des messages.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn pente_entrees_refusees() {
    for (xs, ys) in
        [(vec![0.0, 1.0, 2.0], vec![0.0, 1.0]), (vec![1.0], vec![1.0]), (vec![2.0, 2.0, 2.0], vec![0.0, 1.0, 5.0])]
    {
        let resultat = std::panic::catch_unwind(|| pente(&xs, &ys));
        assert!(resultat.is_err(), "pente({xs:?}, {ys:?}) devait paniquer");
    }
    proche(pente(&[0.0, 1.0], &[0.0, 1.0]), 1.0, TOL, "deux points");
    proche(pente(&[3.0, 4.0], &[5.0, 5.0]), 0.0, TOL, "deux points, y constant");
}

// ------------------------------------------------------------------------------------------------------ Bitmap

/// juste : `Bitmap::new`, `allumer`, `est_allume`, `nombre_allumes`, `largeur`, `hauteur` : une image neuve est éteinte ; allumer
/// deux fois le même pixel le compte une fois ; le pixel `(x, y)` n'est pas le pixel `(y, x)` ; un pixel hors de l'image fait
/// paniquer (`x >= largeur` ne « glisse » pas sur la ligne suivante), de même qu'une image sans pixel (V-BMP-01).
///
/// Ne couvre pas : les images énormes.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn bitmap_acces() {
    let mut image = Bitmap::new(4, 3);
    assert_eq!((image.largeur(), image.hauteur(), image.nombre_allumes()), (4, 3, 0));
    image.allumer(3, 0);
    image.allumer(3, 0);
    image.allumer(1, 2);
    assert_eq!(image.nombre_allumes(), 2);
    assert!(image.est_allume(3, 0) && image.est_allume(1, 2));
    assert!(!image.est_allume(0, 0) && !image.est_allume(2, 1) && !image.est_allume(0, 1));
    assert!(!image.est_allume(2, 1), "(2, 1) n'est pas (1, 2)");
    for (x, y) in [(4, 0), (0, 3), (4, 3)] {
        assert!(std::panic::catch_unwind(|| Bitmap::new(4, 3).est_allume(x, y)).is_err(), "est_allume({x}, {y})");
        assert!(std::panic::catch_unwind(|| Bitmap::new(4, 3).allumer(x, y)).is_err(), "allumer({x}, {y})");
    }
    assert!(std::panic::catch_unwind(|| Bitmap::new(0, 3)).is_err());
    assert!(std::panic::catch_unwind(|| Bitmap::new(3, 0)).is_err());
}

/// juste : `Bitmap::depuis_sites` (V-BMP-02) : l'image est la boîte englobante des sites ET de l'origine, le centre est le pixel
/// de l'origine. `[(0,0), (2,1), (-1,3)]` donne une image de 4 × 4 de centre `(1, 0)` avec les pixels `(1,0)`, `(3,1)`, `(0,3)` ;
/// `[(3,4)]` donne 4 × 5, centre `(0, 0)`, pixel `(3,4)` (l'origine est dans la boîte même si elle n'est pas un site) ;
/// `[(-2,-3)]` donne 3 × 4, centre `(2, 3)`, pixel `(0,0)` ; la liste vide donne 1 × 1 éteinte, centre `(0, 0)` ; des sites
/// répétés allument un seul pixel.
///
/// Ne couvre pas : les très grandes coordonnées.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn bitmap_depuis_sites() {
    let (image, centre) = Bitmap::depuis_sites(&[(0, 0), (2, 1), (-1, 3)]);
    assert_eq!((image.largeur(), image.hauteur(), centre), (4, 4, (1, 0)));
    assert_eq!(image.nombre_allumes(), 3);
    assert!(image.est_allume(1, 0) && image.est_allume(3, 1) && image.est_allume(0, 3));

    let (image, centre) = Bitmap::depuis_sites(&[(3, 4)]);
    assert_eq!((image.largeur(), image.hauteur(), centre), (4, 5, (0, 0)));
    assert!(image.est_allume(3, 4) && image.nombre_allumes() == 1);

    let (image, centre) = Bitmap::depuis_sites(&[(-2, -3)]);
    assert_eq!((image.largeur(), image.hauteur(), centre), (3, 4, (2, 3)));
    assert!(image.est_allume(0, 0) && image.nombre_allumes() == 1);

    let (image, centre) = Bitmap::depuis_sites(&[]);
    assert_eq!((image.largeur(), image.hauteur(), centre, image.nombre_allumes()), (1, 1, (0, 0), 0));

    let (image, _) = Bitmap::depuis_sites(&[(1, 1), (1, 1), (0, 0)]);
    assert_eq!((image.largeur(), image.hauteur(), image.nombre_allumes()), (2, 2, 2));
}

// ----------------------------------------------------------------------------------------------------- comptage de boîtes

/// L'image de 7 × 5 de V-BOITE-01 : les pixels `(0,0) (1,0) (6,4) (3,2) (4,2) (6,0) (0,4)`.
fn image_7_sur_5() -> Bitmap {
    let mut image = Bitmap::new(7, 5);
    for (x, y) in [(0, 0), (1, 0), (6, 4), (3, 2), (4, 2), (6, 0), (0, 4)] {
        image.allumer(x, y);
    }
    image
}

/// juste : `boites_occupees` sur une image dont la largeur n'est pas un multiple du côté (V-BOITE-01), comptée à la main (et
/// recomptée en Python) : pour les côtés 1, 2, 3, 4, 7, 8, 100 la réponse est 7, 6, 5, 4, 1, 1, 1. Les boîtes qui dépassent le
/// bord sont coupées par l'image et comptent (côté 4 : la colonne de boîtes `4..8` ne contient que les pixels `4..7`, et c'est
/// là que sont `(6,0)` et `(6,4)`). Puis (V-BOITE-02) deux pixels voisins qui chevauchent une frontière de boîte, `(1,0)` et `(2,0)`
/// sur une image de 4 × 1, donnent 2 boîtes au côté 2 (boîtes alignées sur le coin de l'image : `x / 2` vaut 0 puis 1), 1 au côté 3
/// et 1 au côté 4 ; un côté 0 fait paniquer.
///
/// Ne couvre pas : les très grandes images.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn boites_occupees_a_la_main() {
    let image = image_7_sur_5();
    assert_eq!(comptes(&image, &[1, 2, 3, 4, 7, 8, 100]), [7, 6, 5, 4, 1, 1, 1]);

    let mut deux = Bitmap::new(4, 1);
    deux.allumer(1, 0);
    deux.allumer(2, 0);
    assert_eq!(comptes(&deux, &[1, 2, 3, 4]), [2, 2, 1, 1]);

    assert!(std::panic::catch_unwind(|| boites_occupees(&Bitmap::new(3, 3), 0)).is_err());
}

/// juste : `dimension_boites` sur l'image de 7 × 5 avec les côtés `[1, 2, 3, 4]` (comptes 7, 6, 5, 4) : la pente de `ln N` contre
/// `-ln c` vaut `0,38541060871400573` (Python `decimal`, 60 chiffres, moindres carrés sur ces quatre points) (V-BOITE-03). Les
/// côtés peuvent être donnés dans un autre ordre. Des côtés ou une image qui n'ont pas de pente font paniquer : moins de deux
/// côtés, tous égaux, image éteinte.
///
/// Ne couvre pas : la valeur de la dimension d'un ensemble réel (voir les tests de fractales).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn dimension_boites_a_la_main() {
    let image = image_7_sur_5();
    proche(dimension_boites(&image, &[1, 2, 3, 4]), 0.385_410_608_714_005_73, TOL, "côtés 1 2 3 4");
    proche(dimension_boites(&image, &[4, 1, 3, 2]), 0.385_410_608_714_005_73, TOL, "côtés dans le désordre");
    for cotes in [&[2_usize][..], &[], &[3, 3]] {
        assert!(std::panic::catch_unwind(|| dimension_boites(&image_7_sur_5(), cotes)).is_err(), "côtés {cotes:?}");
    }
    assert!(std::panic::catch_unwind(|| dimension_boites(&Bitmap::new(9, 9), &[1, 3])).is_err(), "image éteinte");
}

/// juste : la dimension d'objets dont on connaît la réponse, sur une image de côté `243 = 3^5` et les côtés `1, 3, 9, 27, 81`
/// (V-BOITE-04) : un segment horizontal, un segment vertical et une diagonale donnent 1 (`N(c) = 243/c`) ; le carré plein donne
/// 2 (`N(c) = (243/c)²`) ; la fractale de Vicsek, en croix comme en X, donne `ln 5 / ln 3 = 1,4649735207…` (`N(c) = 5^(5-j)` pour
/// `c = 3^j` : les comptes sont 3125, 625, 125, 25, 5) ; le tapis de Sierpinski donne `ln 8 / ln 3 = 1,8927892607…` (comptes 32768,
/// 4096, 512, 64, 8). Les images sont fabriquées dans ce test. Valeurs publiées : la dimension de Hausdorff de la fractale de Vicsek
/// est `log 5 / log 3 ≈ 1,46497` et celle du tapis de Sierpinski `log 8 / log 3 ≈ 1,8928` (Wikipedia, « Vicsek fractal » et
/// « Sierpiński carpet », lus le 8/11/2026 ; ce sont les dimensions de similarité des constructions « 5 carrés sur 9 » et « 8 carrés sur 9 »).
/// Elles sont exactes ici parce que, pour un côté de boîte `3^j`, chaque boîte est exactement l'un des `5^(5-j)` sous-carrés de
/// niveau `j`.
///
/// Ne couvre pas : les côtés qui ne sont pas des puissances de 3 (le comptage y diffère de la loi de puissance, voir
/// `dimension_boites_boites_decalees`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn dimension_boites_images_synthetiques() {
    let mut horizontal = Bitmap::new(243, 243);
    let mut vertical = Bitmap::new(243, 243);
    let mut diagonale = Bitmap::new(243, 243);
    let mut plein = Bitmap::new(243, 243);
    for i in 0..243 {
        horizontal.allumer(i, 121);
        vertical.allumer(60, i);
        diagonale.allumer(i, i);
        for j in 0..243 {
            plein.allumer(i, j);
        }
    }
    assert_eq!(comptes(&horizontal, &COTES_3), [243, 81, 27, 9, 3]);
    for (nom, image) in
        [("segment horizontal", &horizontal), ("segment vertical", &vertical), ("diagonale", &diagonale)]
    {
        proche(dimension_boites(image, &COTES_3), 1.0, TOL, nom);
    }
    assert_eq!(comptes(&plein, &COTES_3), [59049, 6561, 729, 81, 9]);
    proche(dimension_boites(&plein, &COTES_3), 2.0, TOL, "carré plein");

    let croix = vicsek_croix(5, 243, 0, 0);
    assert_eq!(croix.nombre_allumes(), 3125);
    assert_eq!(comptes(&croix, &COTES_3), [3125, 625, 125, 25, 5]);
    proche(dimension_boites(&croix, &COTES_3), DIM_VICSEK, TOL, "Vicsek en croix");
    let en_x = vicsek_x(5, 243);
    assert_eq!(comptes(&en_x, &COTES_3), [3125, 625, 125, 25, 5]);
    proche(dimension_boites(&en_x, &COTES_3), DIM_VICSEK, TOL, "Vicsek en X");

    let carpet = tapis(5, 243);
    assert_eq!(carpet.nombre_allumes(), 32768);
    assert_eq!(comptes(&carpet, &COTES_3), [32768, 4096, 512, 64, 8]);
    proche(dimension_boites(&carpet, &COTES_3), DIM_TAPIS, TOL, "tapis de Sierpinski");
}

/// juste : un niveau de moins (image de 81) avec les côtés `1, 3, 9, 27` donne les mêmes dimensions ; et la fractale de niveau 5
/// posée dans le coin d'une image plus grande (324 × 324, décalage `(0, 0)`) donne le même résultat : l'image n'a pas besoin
/// d'être de côté `3^k` (V-BOITE-05).
///
/// Ne couvre pas : un décalage de la fractale (voir `dimension_boites_boites_decalees`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn dimension_boites_autres_tailles() {
    let petite = vicsek_croix(4, 81, 0, 0);
    assert_eq!(comptes(&petite, &[1, 3, 9, 27]), [625, 125, 25, 5]);
    proche(dimension_boites(&petite, &[1, 3, 9, 27]), DIM_VICSEK, TOL, "Vicsek niveau 4");
    let grande_toile = vicsek_croix(5, 324, 0, 0);
    assert_eq!(comptes(&grande_toile, &COTES_3), [3125, 625, 125, 25, 5]);
    proche(dimension_boites(&grande_toile, &COTES_3), DIM_VICSEK, TOL, "Vicsek niveau 5 sur 324 × 324");
    let mut moitie = Bitmap::new(64, 64);
    for i in 0..64 {
        for j in 0..64 {
            moitie.allumer(i, j);
        }
    }
    proche(dimension_boites(&moitie, &[1, 2, 4, 8, 16]), 2.0, TOL, "carré plein, côtés en puissances de 2");
}

/// juste : les boîtes sont alignées sur le COIN DE L'IMAGE, jamais sur l'objet (V-BOITE-06). La fractale de Vicsek de niveau 5 posée
/// à `(27, 54)` dans une image de 324 × 324 occupe, au côté 81, une rangée de boîtes qui chevauche : les comptes pour `1, 3, 9, 27, 81`
/// sont `3125, 625, 125, 25, 10` (et non `5`), d'où la pente `1,3387875700036357` ; posée à `(1, 0)`, les comptes sont `3125, 938, 188,
/// 38, 8` et la pente `1,3782532733743578`. Comptes et pentes viennent de Python (comptage naïf par `(x // c, y // c)`, `decimal`).
/// Une implémentation qui recolle les boîtes sur la boîte englobante de l'objet rend 1,4649… et échoue ici.
///
/// Ne couvre pas : l'interprétation de ces pentes comme une dimension (elles n'en sont pas : le comptage est faussé exprès).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn dimension_boites_boites_decalees() {
    let decalee = vicsek_croix(5, 324, 27, 54);
    assert_eq!(decalee.nombre_allumes(), 3125);
    assert_eq!(comptes(&decalee, &COTES_3), [3125, 625, 125, 25, 10]);
    proche(dimension_boites(&decalee, &COTES_3), 1.338_787_570_003_635_7, TOL, "décalage (27, 54)");
    let d1 = vicsek_croix(5, 324, 1, 0);
    assert_eq!(comptes(&d1, &COTES_3), [3125, 938, 188, 38, 8]);
    proche(dimension_boites(&d1, &COTES_3), 1.378_253_273_374_357_8, TOL, "décalage (1, 0)");
}

// ------------------------------------------------------------------------------------------------------- masse-rayon

/// Les sites de V-MASSE-01 : sur et près du cercle de rayon 5 autour de l'origine.
const SITES_MASSE: [(i32, i32); 7] = [(0, 0), (3, 4), (4, 4), (-3, -4), (5, 0), (0, 5), (6, 0)];

/// juste : `masse_dans_rayon` : le bord du disque est dedans (`dx² + dy² <= r²`), comptée à la main sur 7 sites et trois centres
/// (V-MASSE-01) : centre `(0, 0)`, rayons `0, 1, 2, 4, 5, 6, 7` : `1, 1, 1, 1, 5, 7, 7` (`(3,4)` à distance exactement 5 compte, `(4,4)` à
/// `√32` non) ; centre `(3, 4)`, rayons `0, 1, 4, 5, 10` : `1, 2, 3, 6, 7` ; centre `(-2, 1)` (hors des sites, négatif), rayons `0, 4, 5, 6,
/// 7, 8` : `0, 1, 2, 4, 5, 6`. Un site répété compte autant de fois qu'il figure ; des coordonnées de l'ordre de `2^31` ne
/// débordent pas.
///
/// Ne couvre pas : l'efficacité.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn masse_dans_rayon_a_la_main() {
    let sites = SITES_MASSE;
    let lire = |centre: (i32, i32), rayons: &[u32]| -> Vec<usize> {
        rayons.iter().map(|&r| masse_dans_rayon(&sites, centre, r)).collect()
    };
    assert_eq!(lire((0, 0), &[0, 1, 2, 4, 5, 6, 7]), [1, 1, 1, 1, 5, 7, 7]);
    assert_eq!(lire((3, 4), &[0, 1, 4, 5, 10]), [1, 2, 3, 6, 7]);
    assert_eq!(lire((-2, 1), &[0, 4, 5, 6, 7, 8]), [0, 1, 2, 4, 5, 6]);
    assert_eq!(masse_dans_rayon(&[(1, 1), (1, 1), (9, 9)], (1, 1), 0), 2);
    let extremes = [(i32::MAX, i32::MAX), (i32::MIN, i32::MIN), (0, 0)];
    assert_eq!(masse_dans_rayon(&extremes, (0, 0), 0), 1);
    assert_eq!(masse_dans_rayon(&extremes, (0, 0), 1 << 31), 1);
    assert_eq!(masse_dans_rayon(&extremes, (0, 0), u32::MAX), 3);
    assert_eq!(masse_dans_rayon(&extremes, (i32::MAX, i32::MAX), 0), 1);
    assert_eq!(masse_dans_rayon(&[], (0, 0), 10), 0);
}

/// Le disque plein de rayon `rayon`, centré en `centre` : tous les sites `(x, y)` avec `(x - cx)² + (y - cy)² <= rayon²`.
fn disque_plein(rayon: i32, centre: (i32, i32)) -> Vec<(i32, i32)> {
    let mut sites = Vec::new();
    for y in -rayon..=rayon {
        for x in -rayon..=rayon {
            if x * x + y * y <= rayon * rayon {
                sites.push((x + centre.0, y + centre.1));
            }
        }
    }
    sites
}

/// juste : sur un disque plein de rayon 100 (31 417 sites), la masse dans le disque de rayon `r <= 10` est le nombre de points entiers du
/// cercle de Gauss : `1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317` pour `r = 0..=10` (suite A000328 de l'OEIS, recomptée par Python) ;
/// pour les rayons `6, 8, 12, 17, 25, 35, 50, 70` elle vaut `113, 197, 441, 901, 1961, 3853, 7845, 15373` (comptage par Python) ; la
/// dimension de masse-rayon sur ces rayons est la pente exacte de ces comptes, `2,0061302306554317` (Python `decimal`) : 2 à `0,6 %`
/// près, l'écart venant de la discrétisation du disque (sa masse est `π r² + O(r^(2/3))`) (V-MASSE-02). Le même disque translaté de
/// `(37, -12)`, mesuré autour de son centre, donne les mêmes comptes et la même dimension (V-MASSE-03).
///
/// Ne couvre pas : un amas fractal (voir `etape_1_dla_statistique`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn masse_rayon_disque_plein() {
    let disque = disque_plein(100, (0, 0));
    assert_eq!(disque.len(), 31417);
    let gauss: Vec<usize> = (0..=10).map(|r| masse_dans_rayon(&disque, (0, 0), r)).collect();
    assert_eq!(gauss, [1, 5, 13, 29, 49, 81, 113, 149, 197, 253, 317]);
    let rayons = [6_u32, 8, 12, 17, 25, 35, 50, 70];
    let comptes_attendus = [113_usize, 197, 441, 901, 1961, 3853, 7845, 15373];
    for (&r, &n) in rayons.iter().zip(&comptes_attendus) {
        assert_eq!(masse_dans_rayon(&disque, (0, 0), r), n, "rayon {r}");
    }
    let dimension = dimension_masse_rayon(&disque, (0, 0), &rayons);
    proche(dimension, 2.006_130_230_655_432, TOL, "dimension du disque plein");
    assert!((dimension - 2.0).abs() < 0.01, "la dimension d'un disque est 2, à la discrétisation près : {dimension}");

    let decale = disque_plein(100, (37, -12));
    for (&r, &n) in rayons.iter().zip(&comptes_attendus) {
        assert_eq!(masse_dans_rayon(&decale, (37, -12), r), n, "rayon {r}, disque translaté");
    }
    proche(dimension_masse_rayon(&decale, (37, -12), &rayons), 2.006_130_230_655_432, TOL, "disque translaté");
}

/// juste : un segment horizontal centré (les sites `(x, 0)` pour `x` de `-100` à `100`) a pour masse `2r + 1` dans le rayon `r <= 100` ;
/// pour les rayons `4, 8, 16, 32, 64` (comptes `9, 17, 33, 65, 129`) la pente exacte est `0,96175094797399986` (Python `decimal` : c'est
/// `d ln(2r + 1) / d ln r = 2r/(2r + 1)` moyenné, un peu sous 1 par discrétisation, à 4 % de la valeur 1 d'un segment) (V-MASSE-04).
/// Un segment vertical donne la même chose.
///
/// Ne couvre pas : les segments obliques.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn masse_rayon_segment_centre() {
    let rayons = [4_u32, 8, 16, 32, 64];
    let horizontal: Vec<(i32, i32)> = (-100..=100).map(|x| (x, 0)).collect();
    let vertical: Vec<(i32, i32)> = (-100..=100).map(|y| (0, y)).collect();
    for (nom, sites) in [("horizontal", &horizontal), ("vertical", &vertical)] {
        let comptes: Vec<usize> = rayons.iter().map(|&r| masse_dans_rayon(sites, (0, 0), r)).collect();
        assert_eq!(comptes, [9, 17, 33, 65, 129], "segment {nom}");
        let dimension = dimension_masse_rayon(sites, (0, 0), &rayons);
        proche(dimension, 0.961_750_947_973_999_9, TOL, nom);
        assert!(
            (dimension - 1.0).abs() < 0.04,
            "la dimension d'un segment est 1, à la discrétisation près : {dimension}"
        );
    }
}

/// juste : `points_masse_rayon` rend les couples `(ln r, ln N(r))` dans l'ordre des rayons donnés : pour le disque plein de rayon 100
/// et les rayons `6, 8, 12`, `(1,7917594692280550, 4,7273878187123406)`, `(2,0794415416798359, 5,2832037287379885)`,
/// `(2,4849066497880003, 6,0890448754468460)` (Python `decimal`, `ln` de `6, 8, 12` et de `113, 197, 441`) ; et l'ordre des rayons est
/// respecté s'ils sont donnés décroissants (V-MASSE-05).
///
/// Ne couvre pas : la précision du logarithme hors de ces points (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn points_masse_rayon_a_la_main() {
    let disque = disque_plein(100, (0, 0));
    let points = points_masse_rayon(&disque, (0, 0), &[6, 8, 12]);
    let attendus = [
        (1.791_759_469_228_055, 4.727_387_818_712_341),
        (2.079_441_541_679_835_9, 5.283_203_728_737_988_5),
        (2.484_906_649_788_000_3, 6.089_044_875_446_846),
    ];
    assert_eq!(points.len(), 3);
    for (obtenu, attendu) in points.iter().zip(attendus) {
        proche(obtenu.0, attendu.0, TOL, "ln r");
        proche(obtenu.1, attendu.1, TOL, "ln N(r)");
    }
    let inverse = points_masse_rayon(&disque, (0, 0), &[12, 6]);
    proche(inverse[0].0, attendus[2].0, TOL, "ordre décroissant : ln 12 d'abord");
    proche(inverse[1].1, attendus[0].1, TOL, "ordre décroissant : ln 113 ensuite");
}

/// juste : les entrées sans masse-rayon font paniquer (doc) : un rayon nul, un disque sans site (centre loin des sites), moins de deux
/// rayons, deux rayons égaux ; et deux rayons distincts sont acceptés : pour les sites de V-MASSE-01 et les rayons `5, 6` (masses 5 et 7) la
/// dimension vaut `ln(7/5) / ln(6/5) = 1,8454879529219202` (Python `decimal`), pour qu'un corps qui panique toujours n'y passe pas (V-MASSE-06).
///
/// Ne couvre pas : le texte des messages.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn masse_rayon_entrees_refusees() {
    let sites = SITES_MASSE;
    assert!(std::panic::catch_unwind(|| points_masse_rayon(&sites, (0, 0), &[0, 5])).is_err(), "rayon 0");
    assert!(std::panic::catch_unwind(|| points_masse_rayon(&sites, (100, 100), &[3, 5])).is_err(), "aucun site");
    assert!(std::panic::catch_unwind(|| dimension_masse_rayon(&sites, (0, 0), &[5])).is_err(), "un seul rayon");
    assert!(std::panic::catch_unwind(|| dimension_masse_rayon(&sites, (0, 0), &[5, 5])).is_err(), "rayons égaux");
    proche(dimension_masse_rayon(&sites, (0, 0), &[5, 6]), 1.845_487_952_921_920_2, TOL, "deux rayons");
}

// ------------------------------------------------------------------------------------------------ rayon de giration

/// juste : le rayon de giration sur des cas à solution exacte (Python `fractions`, V-GIR-01) : un site 0 ; deux sites `(0,0)`, `(3,4)`
/// donnent 2,5 (la moitié de leur distance) ; deux sites identiques 0 ; le carré plein de côté 5 donne exactement 2 (`(s²-1)/6 = 4`) ;
/// le carré de côté 2 `sqrt(1/2)` ; le carré de côté 13 `sqrt(28)` ; le L `(0,0) (1,0) (0,1)` donne `2/3` ; le jeu dissymétrique `(10,0) (10,1)
/// (11,0) (30,40)` donne `sqrt(2943/8) = 19,180067778816633`. Une mesure depuis l'origine plutôt que depuis le centre de masse
/// les distingue (le L donnerait `sqrt(2/3)`).
///
/// Ne couvre pas : la précision à très grand `n` (voir `rayon_de_giration_formule_fermee`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn rayon_de_giration_a_la_main() {
    assert_eq!(rayon_de_giration(&[(7, -3)]), 0.0);
    assert_eq!(rayon_de_giration(&[(0, 0), (3, 4)]), 2.5);
    assert_eq!(rayon_de_giration(&[(5, 5), (5, 5)]), 0.0);
    let carre = |s: i32| -> Vec<(i32, i32)> { (0..s).flat_map(|i| (0..s).map(move |j| (i, j))).collect() };
    assert_eq!(rayon_de_giration(&carre(5)), 2.0);
    proche(rayon_de_giration(&carre(2)), 0.5_f64.sqrt(), 1e-15, "carré de côté 2");
    proche(rayon_de_giration(&carre(13)), 28.0_f64.sqrt(), 1e-14, "carré de côté 13");
    proche(rayon_de_giration(&[(0, 0), (1, 0), (0, 1)]), 2.0 / 3.0, 1e-15, "le L");
    proche(
        rayon_de_giration(&[(10, 0), (10, 1), (11, 0), (30, 40)]),
        19.180_067_778_816_633,
        1e-12,
        "jeu dissymétrique",
    );
    assert!(std::panic::catch_unwind(|| rayon_de_giration(&[])).is_err(), "aucun site");
}

/// juste : le rayon de giration d'un carré plein de côté `s` vérifie `R_g² = (s² - 1)/6` pour `s = 1..=40` (formule fermée : la variance de
/// chaque coordonnée d'une loi uniforme sur `s` entiers est `(s² - 1)/12`, deux coordonnées) ; il ne change pas quand on translate
/// tous les sites (les sommes sont des entiers exacts : le résultat est le MÊME flottant, au bit près) ; il est exact avec des
/// coordonnées de l'ordre de `2^30` (`(−2^30, −2^30)` et `(2^30, 2^30)` donnent `2^30 √2 = 1518500249,98802485`) où des sommes en
/// `i64` ou en `f64` débordent ou arrondissent ; et le jeu `carré de côté 7` translaté de `(-1000, 2500)` donne `sqrt(8)` (V-GIR-02).
///
/// Ne couvre pas : l'ordre des sommes entières (elles sont exactes, l'ordre ne compte pas).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn rayon_de_giration_formule_fermee() {
    let carre = |s: i32, dx: i32, dy: i32| -> Vec<(i32, i32)> {
        (0..s).flat_map(|i| (0..s).map(move |j| (i + dx, j + dy))).collect()
    };
    for s in 1..=40_i32 {
        let rg = rayon_de_giration(&carre(s, 0, 0));
        let attendu = f64::from(s * s - 1) / 6.0;
        proche(rg * rg, attendu, 1e-12 * attendu.max(1.0), &format!("carré de côté {s}"));
        assert_eq!(rg.to_bits(), rayon_de_giration(&carre(s, -1000, 2500)).to_bits(), "translation, côté {s}");
    }
    proche(rayon_de_giration(&carre(7, -1000, 2500)), 8.0_f64.sqrt(), 1e-14, "carré de côté 7 translaté");
    let rg = rayon_de_giration(&[(-(1 << 30), -(1 << 30)), (1 << 30, 1 << 30)]);
    proche(rg, 1_518_500_249.988_024_8, 1e-5, "coordonnées de l'ordre de 2^30");
}

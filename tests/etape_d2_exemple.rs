//! Étape D2 : ce que l'exemple `examples/courbe_d2.rs` écrit, par les deux fonctions de `givre_lab::courbe` qu'il appelle : le CSV (`csv_de_la_courbe`, colonnes
//! `eta,taille,graines,D,erreur,publie,ecart`) et l'image (`png_de_la_courbe`, 640 × 320 pixels). Valeurs attendues : `references/etape-d2-valeurs.md`, identifiants `V-D2-13` à `V-D2-17`.
//!
//! **Pourquoi pas l'exemple lui-même** : un test compile même quand il est ignoré, et le jeu gelé ne livre pas `examples/` (c'est la vague du code qui l'écrit) ; un test qui
//! lance `cargo run --example` dépendrait d'un fichier qui n'existe pas encore, et d'un `cargo` imbriqué. L'exemple n'est qu'un appel de `courbe`, de ces deux fonctions et d'écritures
//! de fichiers (voir `MODELE.md`) : ce que les tests voient, c'est ce qu'il écrit. Ce sont des tests « juste » (texte et pixels écrits à la main d'après la doc) ou « forme » (une
//! courbe réelle, dont on regarde la forme, pas les nombres).

mod common;

use std::sync::OnceLock;

use common::relire_png;
use givre_lab::courbe::{PointDeLaCourbe, courbe, csv_de_la_courbe, ecart, png_de_la_courbe, publie};

const FOND: [u8; 3] = [255, 255, 255];
const BANC: [u8; 3] = [31, 119, 180];
const PUBLIE: [u8; 3] = [214, 39, 40];

/// Une vraie courbe, calculée une fois pour le fichier : `η = 0` à 3, les graines 1 à 4, 60 sites.
fn courbe_reelle() -> &'static [PointDeLaCourbe] {
    static COURBE: OnceLock<Vec<PointDeLaCourbe>> = OnceLock::new();
    COURBE.get_or_init(|| courbe(&[0, 1, 2, 3], &[1, 2, 3, 4], 60))
}

fn point(eta: u32, taille: usize, graines: usize, moyenne: f64, erreur_type: f64) -> PointDeLaCourbe {
    PointDeLaCourbe { eta, taille, graines, moyenne, erreur_type }
}

/// La colonne et la ligne de la documentation : `x(η) = 40 + 70 η` ; `y(D) = clamp(20 + round((2.2 - D) * 200.0), 0, 319)`, écrites ici en dur.
fn colonne(eta: u32) -> i64 {
    40 + 70 * i64::from(eta)
}
fn ligne(dimension: f64) -> i64 {
    (20 + ((2.2 - dimension) * 200.0).round() as i64).clamp(0, 319)
}

/// juste : le CSV est celui de la documentation, au caractère près (V-D2-13). Sept points écrits à la main (`η` = 0, 1, 2, 3 et 4 ; deux autres, `η = 7` de 50 sites et 3 graines et `η = 8` de 10 sites et 2 graines,
/// pour l'arrondi) donnent le texte ci-dessous : l'en-tête `eta,taille,graines,D,erreur,publie,ecart` ; `D` et `erreur` à six décimales (`1.2345678` devient `1.234568`,
/// `0.0000004` devient `0.000000`) ; `publie` à trois décimales (`1.710`, `1.433`, `1.263`), **vide** pour `η = 0` et pour `η` de 4 à 7 (aucun repère) ; `ecart` = `D - publie` à six décimales
/// (négatif quand le banc est SOUS le repère : `-0.060000` pour `η = 1`), vide avec `publie` ; chaque ligne finie par un seul `\n`, aucun `\r`, et l'ordre est celui des points. Une liste
/// vide donne l'en-tête seul. Le texte est du ASCII. Et (ajout du juge) un point `η = 1` de moyenne `1.7600015` et d'erreur `0.0000005` s'écrit
/// `1,100,24,1.760001,0.000000,1.710,0.050002` : `{:.6}` de Rust (le double de `0.0000005` est sous la moitié) et l'écart sur la moyenne NON arrondie.
///
/// Ne couvre pas : une valeur à plus de six chiffres significatifs (un `D` de 100 000) ; la colonne `D` d'un `NaN` ou d'un infini.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn csv_a_la_main() {
    let points = [
        point(0, 100, 24, 1.75, 0.0045),
        point(1, 100, 24, 1.65, 0.01),
        point(2, 100, 24, 1.5, 0.0125),
        point(3, 100, 24, 1.35, 0.012),
        point(4, 100, 24, 1.2, 0.02),
        point(7, 50, 3, 1.2345678, 0.0123457),
        point(8, 10, 2, 0.0000004, 0.0000006),
    ];
    let attendu = "eta,taille,graines,D,erreur,publie,ecart\n\
                   0,100,24,1.750000,0.004500,,\n\
                   1,100,24,1.650000,0.010000,1.710,-0.060000\n\
                   2,100,24,1.500000,0.012500,1.433,0.067000\n\
                   3,100,24,1.350000,0.012000,1.263,0.087000\n\
                   4,100,24,1.200000,0.020000,,\n\
                   7,50,3,1.234568,0.012346,,\n\
                   8,10,2,0.000000,0.000001,,\n";
    let csv = csv_de_la_courbe(&points);
    assert_eq!(csv, attendu);
    assert!(csv.is_ascii() && !csv.contains('\r'));
    // (ajout du juge) `{:.6}` n'est pas `(x * 1e6).round() / 1e6` : le double de `0.0000005` est juste sous la moitié et s'écrit `0.000000` (un arrondi à la
    // main donnerait `0.000001`) ; et `ecart` est calculé sur la moyenne NON arrondie : `1.7600015 - 1.71` s'écrit `0.050002` alors que `D` s'écrit `1.760001`
    // (un écart calculé sur `D` arrondi donnerait `0.050001`). Valeurs tirées du formatage de Rust, pas du prototype.
    assert_eq!(
        csv_de_la_courbe(&[point(1, 100, 24, 1.7600015, 0.0000005)]),
        "eta,taille,graines,D,erreur,publie,ecart\n1,100,24,1.760001,0.000000,1.710,0.050002\n"
    );
    assert_eq!(csv_de_la_courbe(&[]), "eta,taille,graines,D,erreur,publie,ecart\n");
    // l'ordre est celui des points
    let inverse: Vec<PointDeLaCourbe> = points.iter().rev().cloned().collect();
    let csv_inverse = csv_de_la_courbe(&inverse);
    let lignes: Vec<&str> = csv_inverse.lines().skip(1).map(|l| l.split(',').next().unwrap()).collect();
    assert_eq!(lignes, ["8", "7", "4", "3", "2", "1", "0"]);
}

/// forme : le CSV d'une courbe réelle (4 graines, 60 sites, `η = 0` à 3) a la forme de la doc (V-D2-14) : l'en-tête, quatre lignes de sept champs, `eta` 0 à 3, `taille` 60, `graines` 4, `D` et `erreur` égaux aux
/// six décimales de `moyenne` et `erreur_type` du point, `publie` vide pour `η = 0` puis `1.710`, `1.433`, `1.263`, et `ecart` égal à `moyenne - publie` à six décimales ; aucune valeur n'est comparée
/// à un repère, seul `ecart = D - publie` est vérifié comme arithmétique, sur les nombres de NOTRE courbe.
///
/// Ne couvre pas : la valeur de `D` (voir `etape_d2_exact` et `etape_d2_ordre`) ; la taille de 100 sites.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn csv_d_une_courbe_reelle() {
    let points = courbe_reelle();
    let csv = csv_de_la_courbe(points);
    let lignes: Vec<&str> = csv.lines().collect();
    assert_eq!(lignes.len(), 5, "{csv}");
    assert_eq!(lignes[0], "eta,taille,graines,D,erreur,publie,ecart");
    assert!(csv.ends_with('\n') && !csv.ends_with("\n\n"));
    for (k, (ligne, p)) in lignes[1..].iter().zip(points).enumerate() {
        let champs: Vec<&str> = ligne.split(',').collect();
        assert_eq!(champs.len(), 7, "{ligne}");
        assert_eq!((champs[0], champs[1], champs[2]), (k.to_string().as_str(), "60", "4"));
        assert_eq!(champs[3], format!("{:.6}", p.moyenne));
        assert_eq!(champs[4], format!("{:.6}", p.erreur_type));
        match [None, Some(1.710), Some(1.433), Some(1.263)][k] {
            None => assert_eq!((champs[5], champs[6]), ("", ""), "η = 0 n'a pas de repère : {ligne}"),
            Some(repere) => {
                assert_eq!(champs[5], format!("{repere:.3}"));
                assert_eq!(champs[6], format!("{:.6}", p.moyenne - repere), "{ligne}");
            }
        }
    }
}

/// juste : le déterminisme et la sensibilité aux graines (V-D2-17). Deux appels de `courbe` avec les mêmes graines donnent le même CSV, octet pour octet, et le même PNG ; des graines
/// différentes (4 à 6 au lieu de 1 à 3) donnent un autre CSV (la courbe n'ignore pas les graines), mais la même forme (en-tête, deux lignes, mêmes `eta`, `taille`, `graines`).
///
/// Ne couvre pas : l'égalité sous Linux et sous Windows (la doc fixe les bits ; seule une exécution sur chaque système la prouve).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn meme_graine_meme_courbe() {
    let a = courbe(&[0, 2], &[1, 2, 3], 50);
    let b = courbe(&[0, 2], &[1, 2, 3], 50);
    assert_eq!(csv_de_la_courbe(&a), csv_de_la_courbe(&b));
    assert_eq!(png_de_la_courbe(&a), png_de_la_courbe(&b));
    let c = courbe(&[0, 2], &[4, 5, 6], 50);
    assert_ne!(csv_de_la_courbe(&a), csv_de_la_courbe(&c));
    for (x, y) in csv_de_la_courbe(&a).lines().zip(csv_de_la_courbe(&c).lines()) {
        assert_eq!(x.split(',').take(3).collect::<Vec<_>>(), y.split(',').take(3).collect::<Vec<_>>());
    }
}

/// juste : l'image est celle de la documentation (V-D2-15). Six points écrits à la main (`η = 0, 1, 2, 3` à `D` = 1,75 ; 1,66 ; 1,50 ; 1,35, un `η = 5` à 1,10, barres d'erreur de 0,012 à 0,02, et, ajout du juge, un `η = 4` à 1,0275 sans barre, dont la ligne
/// est 255 par l'ordre des opérations de la doc et 254 par `440 - 200 D`) et deux hors cadre (`η = 6` à `D = 3,0`
/// et `η = 7` à `D = -0,5`) donnent un PNG qui se relit (`common::relire_png`, décodeur indépendant : signature, trois chunks, CRC, zlib) de 640 × 320 pixels, dont : le pixel `(0, 0)` et le pixel `(639, 319)` sont
/// de fond (blanc) ; pour chaque point, avec `x(η) = 40 + 70 η` et `y(D) = clamp(20 + round((2.2 - D) * 200.0), 0, 319)` : le centre `(x(η), y(D))` et les quatre coins du carré de 5 × 5 sont en bleu `[31, 119, 180]`, la barre d'erreur va de
/// `y(D + e)` à `y(D - e)` dans la colonne `x(η)` avec un trait de cinq pixels à chaque bout ; **tout pixel bleu est dans la colonne `x(η) - 2` à `x(η) + 2` d'un point** ; les repères publiés sont de petits carrés rouges `[214, 39, 40]`
/// de 5 × 5 pixels centrés sur `(x(η) + 12, y(publie))` pour `η = 1, 2, 3` seulement (`η = 0`, 5, 6 et 7 n'ont pas de repère) : il y a EXACTEMENT 75 pixels rouges, et chaque centre est rouge ; un point hors cadre
/// (`D = 3,0` : ligne `0` ; `D = -0,5` : ligne `319`) est ramené dans l'image (le carré est coupé par le bord, le centre est dessiné). Les pixels que la doc laisse libres (grille, axes, graduations) ne sont pas regardés, sauf que les coins sont de fond et qu'aucun ne prend le bleu ou le rouge des points (le décompte des pixels rouges et le test des colonnes des pixels bleus le voient).
///
/// Ne couvre pas : les axes, la grille et les repères de graduation (libres) ; un `NaN` ; l'image d'une liste vide au-delà de ses dimensions.
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn image_a_la_main() {
    let points = [
        point(0, 100, 24, 1.75, 0.02),
        point(1, 100, 24, 1.66, 0.012),
        point(2, 100, 24, 1.50, 0.015),
        point(3, 100, 24, 1.35, 0.013),
        point(5, 100, 24, 1.10, 0.02),
        point(4, 100, 24, 1.0275, 0.0),
        point(6, 100, 24, 3.0, 0.02),
        point(7, 100, 24, -0.5, 0.02),
    ];
    let png = png_de_la_courbe(&points);
    let lu = relire_png(&png).expect("un PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (640, 320));
    let pixel = |x: i64, y: i64| -> [u8; 3] {
        assert!((0..640).contains(&x) && (0..320).contains(&y));
        let i = 3 * (y as usize * 640 + x as usize);
        [lu.rvb[i], lu.rvb[i + 1], lu.rvb[i + 2]]
    };
    assert_eq!(pixel(0, 0), FOND);
    assert_eq!(pixel(639, 319), FOND);
    // les formules de la doc, à la main : D = 1,75 est à la ligne 110, D = 1,5 à 160, D = 1,35 à 190
    assert_eq!((ligne(1.75), ligne(1.5), ligne(1.35), ligne(3.0), ligne(-0.5)), (110, 160, 190, 0, 319));
    // (ajout du juge) l'ordre des opérations de la doc compte près d'une moitié : `(2.2 - 1.0275) * 200.0` vaut un peu plus de `234.5`, donc la ligne 255 ;
    // `440.0 - 200.0 * 1.0275` vaut un peu moins de `234.5` et donnerait 254 (le point d'`η = 4`, sans barre d'erreur, est là pour le voir).
    assert_eq!(ligne(1.0275), 255);
    for p in &points {
        let (x, y) = (colonne(p.eta), ligne(p.moyenne));
        let (haut, bas) = (ligne(p.moyenne + p.erreur_type), ligne(p.moyenne - p.erreur_type));
        assert_eq!(pixel(x, y), BANC, "centre de η={}", p.eta);
        for (dx, dy) in [(-2, -2), (2, -2), (-2, 2), (2, 2), (-2, 0), (2, 0), (0, -2), (0, 2)] {
            let (px, py) = (x + dx, y + dy);
            if (0..320).contains(&py) {
                assert_eq!(pixel(px, py), BANC, "carré de η={} en ({dx}, {dy})", p.eta);
            }
        }
        for ligne_cap in [haut, bas] {
            for dx in -2..=2 {
                assert_eq!(pixel(x + dx, ligne_cap), BANC, "trait de bout de η={} à la ligne {ligne_cap}", p.eta);
            }
        }
        for yy in haut.min(bas)..=haut.max(bas) {
            assert_eq!(pixel(x, yy), BANC, "barre d'erreur de η={} à la ligne {yy}", p.eta);
        }
    }
    let colonnes: Vec<i64> = points.iter().map(|p| colonne(p.eta)).collect();
    let mut rouges = 0;
    for y in 0..320_i64 {
        for x in 0..640_i64 {
            match pixel(x, y) {
                BANC => {
                    assert!(colonnes.iter().any(|&c| (c - 2..=c + 2).contains(&x)), "pixel bleu isolé en ({x}, {y})")
                }
                PUBLIE => rouges += 1,
                _ => {}
            }
        }
    }
    assert_eq!(rouges, 75, "trois repères de 5 × 5 pixels rouges, ni plus ni moins");
    for (eta, repere) in [(1_u32, 1.71_f64), (2, 1.433), (3, 1.263)] {
        let (x, y) = (colonne(eta) + 12, ligne(repere));
        for dy in -2..=2 {
            for dx in -2..=2 {
                assert_eq!(pixel(x + dx, y + dy), PUBLIE, "repère de η={eta} en ({dx}, {dy})");
            }
        }
    }
}

/// forme : l'image d'une courbe réelle (4 graines, 60 sites, `η = 0` à 3) se relit, a 640 × 320 pixels, a exactement 75 pixels de repères rouges (trois repères publiés) et son centre bleu à `(x(η), y(moyenne))`
/// pour chaque point (V-D2-16) ; l'image d'une liste vide se relit aussi, 640 × 320, sans pixel rouge.
///
/// Ne couvre pas : les nombres du banc (aucune comparaison aux repères) ; le rendu (couleurs au-delà des trois de la doc).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn image_d_une_courbe_reelle() {
    let points = courbe_reelle();
    for liste in [points, &[]] {
        let lu = relire_png(&png_de_la_courbe(liste)).expect("un PNG valide");
        assert_eq!((lu.largeur, lu.hauteur), (640, 320));
        let couleur = |x: i64, y: i64| {
            [
                lu.rvb[3 * (y as usize * 640 + x as usize)],
                lu.rvb[3 * (y as usize * 640 + x as usize) + 1],
                lu.rvb[3 * (y as usize * 640 + x as usize) + 2],
            ]
        };
        let rouges = (0..640 * 320).filter(|&i| lu.rvb[3 * i..3 * i + 3] == PUBLIE).count();
        assert_eq!(rouges, if liste.is_empty() { 0 } else { 75 });
        for p in liste {
            assert_eq!(couleur(colonne(p.eta), ligne(p.moyenne)), BANC, "centre de η={}", p.eta);
        }
    }
}

/// juste : l'arithmétique de l'écart et le repère affiché pour chaque `η` (V-D2-12) : `ecart(point, publie)` est `point.moyenne - publie` à l'octet près (`1.5 - 1.433`, `1.2 - 1.263` négatif, `1.433 - 1.433 = 0`) ;
/// le signe est celui de `moyenne - publie` (positif quand le banc est au-dessus) ; `publie(η)` est `Some(1.71)`, `Some(1.433)`, `Some(1.263)` pour `η = 1, 2, 3` et `None` pour `η = 0` et de 4 à 8.
///
/// Ne couvre pas : la valeur d'un écart réel (jamais testée : voir `MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-d2"), ignore)]
fn ecart_et_reperes_affiches() {
    for (moyenne, repere) in [(1.5, 1.433), (1.2, 1.263), (1.433, 1.433), (1.7, 1.71)] {
        let p = point(2, 100, 24, moyenne, 0.01);
        assert_eq!(ecart(&p, repere).to_bits(), (moyenne - repere).to_bits(), "{moyenne} - {repere}");
    }
    assert!(ecart(&point(2, 100, 24, 1.5, 0.01), 1.433) > 0.0);
    assert!(ecart(&point(3, 100, 24, 1.2, 0.01), 1.263) < 0.0);
    assert_eq!(ecart(&point(2, 100, 24, 1.433, 0.01), 1.433), 0.0);
    let affiches: Vec<Option<f64>> = (0..=8).map(publie).collect();
    assert_eq!(affiches, [None, Some(1.71), Some(1.433), Some(1.263), None, None, None, None, None]);
}
